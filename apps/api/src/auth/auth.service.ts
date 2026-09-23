import {
  BadRequestException,
  Inject,
  Injectable,
  UnauthorizedException,
} from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { randomUUID } from 'node:crypto';
import {
  checkPasswordStrength,
  generateRefreshToken,
  hashPassword,
  normalizeEmail,
  sha256,
  signAccessToken,
  verifyPassword,
} from '../security/crypto';
import { DATA_STORE, DataStore } from '../db/in-memory.repository';
import type { EnvironmentVariables } from '../config/env.validation';
import { User } from '../common/domain';

export interface PublicUser {
  id: string;
  email: string;
  fullName: string;
  role: 'resident' | 'administrator';
  mustChangePassword: boolean;
  createdAt: Date;
  lastLoginAt?: Date;
}

export function toPublicUser(u: User): PublicUser {
  return {
    id: u.id,
    email: u.email,
    fullName: u.fullName,
    role: u.role,
    mustChangePassword: u.mustChangePassword,
    createdAt: u.createdAt,
    lastLoginAt: u.lastLoginAt,
  };
}

@Injectable()
export class AuthService {
  private readonly maxAttempts: number;

  constructor(
    @Inject(DATA_STORE) private readonly store: DataStore,
    config: ConfigService<EnvironmentVariables, true>,
  ) {
    this.maxAttempts = config.get('LOGIN_MAX_ATTEMPTS', { infer: true });
  }

  /** SC-01/SC-02: Argon2id hashing + strength policy at registration. */
  async register(input: {
    email: string;
    password: string;
    fullName: string;
  }): Promise<PublicUser> {
    const emailNormalized = normalizeEmail(input.email);
    const issues = checkPasswordStrength(input.password);
    if (issues.length > 0) {
      throw new BadRequestException({
        title: 'Weak Password',
        status: 400,
        code: 'WEAK_PASSWORD',
        detail: 'Password does not meet the security policy.',
        fields: { password: issues },
      });
    }
    if (this.store.users.findByEmailNormalized(emailNormalized)) {
      // Generic message: do not reveal whether an account exists
      // (user-enumeration defense).
      throw new BadRequestException({
        title: 'Registration Failed',
        status: 400,
        code: 'REGISTRATION_FAILED',
        detail: 'Could not create the account with the provided data.',
      });
    }
    const user = this.store.users.create({
      email: input.email.trim(),
      emailNormalized,
      passwordHash: await hashPassword(input.password),
      fullName: input.fullName.trim(),
      role: 'resident',
      isActive: true,
      isLocked: false,
      failedLoginAttempts: 0,
      mustChangePassword: false,
    });
    this.store.audit.append({
      action: 'auth.register',
      outcome: 'success',
      userId: user.id,
    });
    return toPublicUser(user);
  }

  /**
   * SC-04: login with per-account lockout after N failed attempts and
   * generic error messages for unknown/inactive/locked accounts.
   */
  async login(
    email: string,
    password: string,
    ip?: string,
  ): Promise<{
    accessToken: string;
    refreshToken: string;
    expiresIn: number;
    user: PublicUser;
  }> {
    const emailNormalized = normalizeEmail(email);
    const user = this.store.users.findByEmailNormalized(emailNormalized);

    const failGeneric = (): never => {
      if (user) {
        const attempts = user.failedLoginAttempts + 1;
        const shouldLock = attempts >= this.maxAttempts;
        this.store.users.update(user.id, {
          failedLoginAttempts: attempts,
          ...(shouldLock ? { isLocked: true } : {}),
        });
        this.store.audit.append({
          action: 'auth.login.failure',
          outcome: 'failure',
          userId: user.id,
          ip,
          meta: { attempts, locked: shouldLock },
        });
      } else {
        this.store.audit.append({
          action: 'auth.login.unknown_email',
          outcome: 'failure',
          ip,
        });
      }
      throw new UnauthorizedException({
        title: 'Unauthorized',
        status: 401,
        code: 'AUTHENTICATION_FAILED',
        detail: 'Invalid credentials.',
      });
    };

    if (!user || !user.isActive || user.isLocked) {
      // Dummy verification keeps response time uniform (timing side-channel
      // mitigation against user enumeration).
      await verifyPassword(
        '$argon2id$v=19$m=19456,t=2,p=1$AAAAAAAAAAAAAAAAAAAAAA$0000000000000000000000000000000000000000000',
        password,
      ).catch(() => undefined);
      throw failGeneric();
    }

    const ok = await verifyPassword(user.passwordHash, password);
    if (!ok) throw failGeneric();

    this.store.users.update(user.id, {
      failedLoginAttempts: 0,
      lastLoginAt: new Date(),
    });
    this.store.audit.append({
      action: 'auth.login.success',
      outcome: 'success',
      userId: user.id,
      ip,
    });

    return this.issueTokens(user);
  }

  /** SC-03: short-lived JWT + opaque rotating refresh token (hash-at-rest). */
  async issueTokens(user: User) {
    const secret = process.env.JWT_SECRET ?? '';
    const expiresInMinutes = Number(
      process.env.ACCESS_TOKEN_EXPIRE_MINUTES ?? 15,
    );
    const accessToken = await signAccessToken(
      secret,
      { sub: user.id, role: user.role },
      expiresInMinutes,
    );
    const raw = generateRefreshToken();
    this.store.refreshTokens.create({
      tokenHash: sha256(raw),
      userId: user.id,
      familyId: randomUUID(),
      expiresAt: new Date(Date.now() + 7 * 24 * 3600 * 1000),
    });
    return {
      accessToken,
      refreshToken: raw,
      expiresIn: expiresInMinutes * 60,
      user: toPublicUser(user),
    };
  }

  /**
   * SC-03 rotation with reuse detection: presenting an already-rotated or
   * revoked token revokes the whole family (possible token theft).
   */
  async refresh(rawToken: string) {
    const record = this.store.refreshTokens.findByHash(sha256(rawToken));
    if (!record) {
      throw new UnauthorizedException({
        title: 'Unauthorized',
        status: 401,
        code: 'INVALID_REFRESH_TOKEN',
      });
    }
    if (record.revokedAt) {
      this.store.refreshTokens.revokeFamily(record.familyId);
      this.store.audit.append({
        action: 'auth.refresh.reuse_detected',
        outcome: 'denied',
        userId: record.userId,
        meta: { familyId: record.familyId },
      });
      throw new UnauthorizedException({
        title: 'Unauthorized',
        status: 401,
        code: 'REFRESH_REUSE_DETECTED',
        detail: 'Token reuse detected. All sessions revoked.',
      });
    }
    if (record.expiresAt <= new Date()) {
      throw new UnauthorizedException({
        title: 'Unauthorized',
        status: 401,
        code: 'REFRESH_EXPIRED',
      });
    }
    const user = this.store.users.findById(record.userId);
    if (!user || !user.isActive || user.isLocked) {
      throw new UnauthorizedException({
        title: 'Unauthorized',
        status: 401,
        code: 'ACCOUNT_INACTIVE',
      });
    }

    const nextRaw = generateRefreshToken();
    const next = this.store.refreshTokens.create({
      tokenHash: sha256(nextRaw),
      userId: user.id,
      familyId: record.familyId,
      expiresAt: new Date(Date.now() + 7 * 24 * 3600 * 1000),
    });
    this.store.refreshTokens.update(record.id, {
      revokedAt: new Date(),
      replacedBy: next.id,
    });
    const secret = process.env.JWT_SECRET ?? '';
    const expiresInMinutes = Number(
      process.env.ACCESS_TOKEN_EXPIRE_MINUTES ?? 15,
    );
    const accessToken = await signAccessToken(
      secret,
      { sub: user.id, role: user.role },
      expiresInMinutes,
    );
    return {
      accessToken,
      refreshToken: nextRaw,
      expiresIn: expiresInMinutes * 60,
    };
  }

  async revoke(rawToken: string, userId?: string) {
    const record = this.store.refreshTokens.findByHash(sha256(rawToken));
    if (record && (!userId || record.userId === userId)) {
      this.store.refreshTokens.update(record.id, { revokedAt: new Date() });
    }
    return { message: 'Token revoked.' };
  }

  /** Change password: requires current password; revokes all refresh tokens. */
  async changePassword(
    userId: string,
    currentPassword: string,
    newPassword: string,
  ) {
    const user = this.store.users.findById(userId);
    if (!user) {
      throw new UnauthorizedException({ title: 'Unauthorized', status: 401 });
    }
    if (!(await verifyPassword(user.passwordHash, currentPassword))) {
      throw new BadRequestException({
        title: 'Bad Request',
        status: 400,
        code: 'INVALID_PASSWORD',
        detail: 'Current password is incorrect.',
      });
    }
    const issues = checkPasswordStrength(newPassword);
    if (issues.length > 0) {
      throw new BadRequestException({
        title: 'Weak Password',
        status: 400,
        code: 'WEAK_PASSWORD',
        fields: { newPassword: issues },
      });
    }
    this.store.users.update(user.id, {
      passwordHash: await hashPassword(newPassword),
      mustChangePassword: false,
    });
    // Session invalidation on credential change (OWASP ASVS G2).
    this.store.refreshTokens.revokeAllForUser(user.id);
    this.store.audit.append({
      action: 'auth.password.changed',
      outcome: 'success',
      userId: user.id,
    });
    return { message: 'Password changed. All other sessions were revoked.' };
  }
}
