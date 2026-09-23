import {
  Body,
  Controller,
  Delete,
  ForbiddenException,
  Get,
  HttpCode,
  Inject,
  NotFoundException,
  Param,
  ParseUUIDPipe,
  Post,
  UseGuards,
} from '@nestjs/common';
import { IsIn, IsInt, IsNumber, IsOptional, IsString, IsUUID, Max, MaxLength, Min } from 'class-validator';
import { ConfigService } from '@nestjs/config';
import { DATA_STORE, DataStore } from '../db/in-memory.repository';
import { JwtAuthGuard } from '../security/auth.guard';
import { CurrentUser } from '../common/decorators/current-user.decorator';
import { AuthenticatedUser } from '../security/auth.guard';
import type { EnvironmentVariables } from '../config/env.validation';
import type { LatestLocation, LocationShare, SharePrecision } from '../common/domain';

const MAX_SHARE_HOURS = 24; // time-boxed consent by default (privacy policy)

class CreateShareDto {
  @IsUUID()
  recipientUserId!: string;

  @IsIn(['exact', 'approximate', 'city_only'])
  precision!: SharePrecision;

  @IsOptional()
  @IsString()
  @MaxLength(200)
  purpose?: string;

  /** Minutes until automatic expiry (1..MAX_SHARE_HOURS*60). */
  @IsInt()
  @Min(1)
  @Max(MAX_SHARE_HOURS * 60)
  durationMinutes!: number;
}

class ReportLocationDto {
  @IsNumber({ maxDecimalPlaces: 6 })
  @Min(-90)
  @Max(90)
  lat!: number;

  @IsNumber({ maxDecimalPlaces: 6 })
  @Min(-180)
  @Max(180)
  lng!: number;

  @IsOptional()
  @IsInt()
  @Min(0)
  @Max(10000)
  accuracyMeters?: number;

  @IsOptional()
  @IsInt()
  @Min(0)
  @Max(100)
  batteryLevel?: number;
}

/** Coarsen coordinates according to the consented precision level. */
function applyPrecision(loc: LatestLocation, precision: SharePrecision) {
  switch (precision) {
    case 'exact':
      return { lat: loc.lat, lng: loc.lng };
    case 'approximate': {
      // ~5 km grid rounding
      const q = 0.05;
      return { lat: Math.round(loc.lat / q) * q, lng: Math.round(loc.lng / q) * q };
    }
    case 'city_only': {
      const q = 0.5; // ~50 km grid
      return { lat: Math.round(loc.lat / q) * q, lng: Math.round(loc.lng / q) * q };
    }
  }
}

/**
 * Directional location sharing + latest-location API (REQ-05/REQ-06).
 * Core authorization rule: a recipient can read an owner's location ONLY if
 * an active, unexpired LocationShare exists — checked on every request
 * (cache is never authoritative for consent).
 */
@Controller()
@UseGuards(JwtAuthGuard)
export class LocationsController {
  private readonly staleMs: number;

  constructor(
    @Inject(DATA_STORE) private readonly store: DataStore,
    config: ConfigService<EnvironmentVariables, true>,
  ) {
    this.staleMs = config.get('LOCATION_STALE_MINUTES', { infer: true }) * 60_000;
  }

  @Post('location-shares')
  createShare(@CurrentUser() me: AuthenticatedUser, @Body() dto: CreateShareDto) {
    const recipient = this.store.users.findById(dto.recipientUserId);
    if (!recipient || !recipient.isActive) throw new NotFoundException({ title: 'Not Found', status: 404 });
    const friendship = this.store.friendships.between(me.id, dto.recipientUserId);
    if (!friendship || friendship.status !== 'accepted') {
      throw new ForbiddenException({
        title: 'Forbidden', status: 403, code: 'FRIENDSHIP_REQUIRED',
        detail: 'Location can only be shared with accepted friends.',
      });
    }
    const existing = this.store.shares.activeFor(me.id, dto.recipientUserId);
    if (existing) throw new ForbiddenException({ title: 'Conflict', status: 409, code: 'SHARE_EXISTS' });
    const share = this.store.shares.create({
      ownerId: me.id,
      recipientId: dto.recipientUserId,
      precision: dto.precision,
      purpose: dto.purpose,
      isActive: true,
      expiresAt: new Date(Date.now() + dto.durationMinutes * 60_000),
    });
    this.store.audit.append({
      action: 'consent.share.created', outcome: 'success', userId: me.id,
      meta: { recipient: dto.recipientUserId, precision: dto.precision },
    });
    return {
      id: share.id, recipientUserId: share.recipientId, precision: share.precision,
      expiresAt: share.expiresAt, createdAt: share.createdAt,
    };
  }

  @Get('location-shares/:id')
  getShare(@CurrentUser() me: AuthenticatedUser, @Param('id', ParseUUIDPipe) id: string) {
    const s = this.store.shares.findById(id);
    if (!s || (s.ownerId !== me.id && s.recipientId !== me.id)) {
      throw new NotFoundException({ title: 'Not Found', status: 404 });
    }
    return {
      id: s.id, ownerId: s.ownerId, recipientUserId: s.recipientId,
      precision: s.precision, expiresAt: s.expiresAt, isActive: s.isActive,
    };
  }

  @Delete('location-shares/:id')
  revokeShare(@CurrentUser() me: AuthenticatedUser, @Param('id', ParseUUIDPipe) id: string) {
    const s = this.store.shares.findById(id);
    if (!s) throw new NotFoundException({ title: 'Not Found', status: 404 });
    if (s.ownerId !== me.id) {
      // Only the data subject (owner) may revoke consent.
      throw new ForbiddenException({ title: 'Forbidden', status: 403, code: 'NOT_SHARE_OWNER' });
    }
    this.store.shares.update(s.id, { isActive: false });
    this.store.audit.append({ action: 'consent.share.revoked', outcome: 'success', userId: me.id, meta: { shareId: id } });
    return { message: 'Location share revoked. Recipient access ends immediately.' };
  }

  @Post('me/location')
  reportMyLocation(@CurrentUser() me: AuthenticatedUser, @Body() dto: ReportLocationDto) {
    // Storing my own latest location requires no third-party consent.
    const loc = this.store.locations.upsertLatest({
      ownerId: me.id,
      lat: dto.lat,
      lng: dto.lng,
      accuracyMeters: dto.accuracyMeters,
      batteryLevel: dto.batteryLevel,
      observedAt: new Date(),
    });
    return { stored: true, observedAt: loc.observedAt };
  }

  @Get('latest-locations')
  friendsLatestLocations(@CurrentUser() me: AuthenticatedUser) {
    const now = new Date();
    const shares = this.store.shares.activeReceivedBy(me.id, now);
    const out: Array<Record<string, unknown>> = [];
    for (const s of shares) {
      const loc = this.store.locations.latestFor(s.ownerId);
      if (!loc) continue;
      const coarse = applyPrecision(loc, s.precision);
      const ageMs = now.getTime() - new Date(loc.observedAt).getTime();
      out.push({
        friendUserId: s.ownerId,
        ...coarse,
        precision: s.precision,
        observedAt: loc.observedAt,
        stale: ageMs > this.staleMs,
        dead: ageMs > this.staleMs * 4,
      });
    }
    return { items: out };
  }
}
