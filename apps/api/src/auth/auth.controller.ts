import {
  Body,
  Controller,
  Get,
  HttpCode,
  Post,
  Req,
  UseGuards,
} from '@nestjs/common';
import {
  IsEmail,
  IsNotEmpty,
  IsOptional,
  IsString,
  MaxLength,
  MinLength,
} from 'class-validator';
import { Request } from 'express';
import { AuthService, PublicUser, toPublicUser } from './auth.service';
import { JwtAuthGuard } from '../security/auth.guard';
import { CurrentUser } from '../common/decorators/current-user.decorator';
import { DATA_STORE } from '../db/in-memory.repository';
import { Inject } from '@nestjs/common';
import type { DataStore } from '../db/in-memory.repository';

class RegisterDto {
  @IsEmail()
  @MaxLength(255)
  email!: string;

  @IsString()
  @MinLength(10)
  @MaxLength(128)
  password!: string;

  @IsString()
  @IsNotEmpty()
  @MaxLength(100)
  fullName!: string;
}

class LoginDto {
  @IsEmail()
  @MaxLength(255)
  email!: string;

  @IsString()
  @MinLength(1)
  @MaxLength(128)
  password!: string;
}

class RefreshDto {
  @IsString()
  @IsNotEmpty()
  refreshToken!: string;
}

class ChangePasswordDto {
  @IsString()
  @MinLength(1)
  @MaxLength(128)
  currentPassword!: string;

  @IsString()
  @MinLength(10)
  @MaxLength(128)
  newPassword!: string;
}

/**
 * Auth API — implements the auth section of docs/reference/api-contract.md
 * with a local-credentials MVP flow (OIDC/PKCE external IdP planned for the
 * Oct–Dec phase). Security controls: SC-01..SC-04, SC-07.
 */
@Controller('auth')
export class AuthController {
  constructor(
    private readonly auth: AuthService,
    @Inject(DATA_STORE) private readonly store: DataStore,
  ) {}

  @Post('register')
  async register(@Body() dto: RegisterDto): Promise<PublicUser> {
    return this.auth.register(dto);
  }

  @Post('login')
  @HttpCode(200)
  async login(@Body() dto: LoginDto, @Req() req: Request) {
    const ip =
      (req.headers['x-forwarded-for'] as string | undefined)?.split(',')[0].trim() ??
      req.ip;
    return this.auth.login(dto.email, dto.password, ip);
  }

  @Post('refresh')
  @HttpCode(200)
  async refresh(@Body() dto: RefreshDto) {
    return this.auth.refresh(dto.refreshToken);
  }

  @Post('logout')
  @UseGuards(JwtAuthGuard)
  @HttpCode(200)
  async logout(
    @Body() dto: RefreshDto,
    @CurrentUser() user: { id: string },
  ) {
    return this.auth.revoke(dto.refreshToken, user.id);
  }

  @Post('password/change')
  @UseGuards(JwtAuthGuard)
  @HttpCode(200)
  async changePassword(
    @Body() dto: ChangePasswordDto,
    @CurrentUser() user: { id: string },
  ) {
    return this.auth.changePassword(user.id, dto.currentPassword, dto.newPassword);
  }

  @Get('me')
  @UseGuards(JwtAuthGuard)
  async me(@CurrentUser() user: { id: string }): Promise<PublicUser> {
    const record = this.store.users.findById(user.id)!;
    return toPublicUser(record);
  }
}
