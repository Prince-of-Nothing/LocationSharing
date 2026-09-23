import {
  BadRequestException,
  Body,
  ConflictException,
  Controller,
  ForbiddenException,
  Get,
  Inject,
  NotFoundException,
  Param,
  Post,
  UseGuards,
} from '@nestjs/common';
import { IsUUID } from 'class-validator';
import { DATA_STORE, DataStore } from '../db/in-memory.repository';
import { JwtAuthGuard } from '../security/auth.guard';
import { CurrentUser } from '../common/decorators/current-user.decorator';
import { AuthenticatedUser } from '../security/auth.guard';

class FriendshipRequestDto {
  @IsUUID()
  targetUserId!: string;
}

/**
 * Friendships API (REQ-04): request/accept/decline/block. Consent for
 * location sharing is separate and directional — accepting a friendship does
 * NOT automatically share location.
 */
@Controller('friendships')
@UseGuards(JwtAuthGuard)
export class FriendshipsController {
  constructor(@Inject(DATA_STORE) private readonly store: DataStore) {}

  @Post()
  create(
    @CurrentUser() me: AuthenticatedUser,
    @Body() dto: FriendshipRequestDto,
  ) {
    if (dto.targetUserId === me.id) {
      throw new BadRequestException({ title: 'Bad Request', status: 400, code: 'SELF_REQUEST' });
    }
    const target = this.store.users.findById(dto.targetUserId);
    if (!target || !target.isActive) throw new NotFoundException({ title: 'Not Found', status: 404 });
    const existing = this.store.friendships.between(me.id, dto.targetUserId);
    if (existing) {
      if (existing.status === 'blocked') {
        // Blocked relationship: respond as if the user does not exist
        // (anti-probing, per privacy-and-data-handling.md).
        throw new NotFoundException({ title: 'Not Found', status: 404 });
      }
      throw new ConflictException({ title: 'Conflict', status: 409, code: 'FRIENDSHIP_EXISTS' });
    }
    const f = this.store.friendships.create({
      requesterId: me.id,
      receiverId: dto.targetUserId,
      status: 'pending',
    });
    this.store.audit.append({ action: 'friendship.requested', outcome: 'success', userId: me.id });
    return { id: f.id, status: f.status, createdAt: f.createdAt };
  }

  @Get()
  listMine(@CurrentUser() me: AuthenticatedUser) {
    return this.store.friendships
      .acceptedWith(me.id)
      .map((f) => ({
        id: f.id,
        friendId: f.requesterId === me.id ? f.receiverId : f.requesterId,
        status: f.status,
        since: f.respondedAt ?? f.createdAt,
      }));
  }

  @Get(':id/status')
  status(@CurrentUser() me: AuthenticatedUser, @Param('id') id: string) {
    const f = this.store.friendships.findById(id);
    if (!f || (f.requesterId !== me.id && f.receiverId !== me.id)) {
      throw new NotFoundException({ title: 'Not Found', status: 404 });
    }
    return { state: f.status, createdAt: f.createdAt, respondedAt: f.respondedAt };
  }

  @Post(':id/accept')
  accept(@CurrentUser() me: AuthenticatedUser, @Param('id') id: string) {
    const f = this.store.friendships.findById(id);
    if (!f) throw new NotFoundException({ title: 'Not Found', status: 404 });
    if (f.receiverId !== me.id) {
      throw new ForbiddenException({ title: 'Forbidden', status: 403, code: 'NOT_YOUR_REQUEST' });
    }
    if (f.status !== 'pending') {
      throw new ConflictException({ title: 'Conflict', status: 409, code: 'NOT_PENDING' });
    }
    const updated = this.store.friendships.update(f.id, {
      status: 'accepted',
      respondedAt: new Date(),
    })!;
    this.store.audit.append({ action: 'friendship.accepted', outcome: 'success', userId: me.id });
    return { id: updated.id, status: updated.status };
  }

  @Post(':id/decline')
  decline(@CurrentUser() me: AuthenticatedUser, @Param('id') id: string) {
    const f = this.store.friendships.findById(id);
    if (!f) throw new NotFoundException({ title: 'Not Found', status: 404 });
    if (f.receiverId !== me.id) {
      throw new ForbiddenException({ title: 'Forbidden', status: 403, code: 'NOT_YOUR_REQUEST' });
    }
    const updated = this.store.friendships.update(f.id, {
      status: 'declined',
      respondedAt: new Date(),
    })!;
    return { id: updated.id, status: updated.status };
  }

  @Post(':id/block')
  block(@CurrentUser() me: AuthenticatedUser, @Param('id') id: string) {
    const f = this.store.friendships.findById(id);
    if (!f || (f.requesterId !== me.id && f.receiverId !== me.id)) {
      throw new NotFoundException({ title: 'Not Found', status: 404 });
    }
    const updated = this.store.friendships.update(f.id, { status: 'blocked' })!;
    // Blocking immediately revokes all location shares in both directions.
    for (const s of this.store.shares.allForPair?.(f.requesterId, f.receiverId) ?? []) {
      this.store.shares.update(s.id, { isActive: false });
    }
    this.store.audit.append({ action: 'friendship.blocked', outcome: 'success', userId: me.id });
    return { id: updated.id, status: updated.status };
  }
}
