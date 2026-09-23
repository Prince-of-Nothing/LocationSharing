import {
  CanActivate,
  ExecutionContext,
  Injectable,
  SetMetadata,
  UnauthorizedException,
} from '@nestjs/common';
import { Reflector } from '@nestjs/core';
import { Request } from 'express';
import { verifyAccessToken } from './crypto';
import { DATA_STORE } from '../db/in-memory.repository';
import { Inject } from '@nestjs/common';
import type { DataStore } from '../db/in-memory.repository';

export const AUTHENTICATED_KEY = 'authenticated_user';
export const ROLES_KEY = 'roles';

/** Attach required roles to a handler (RBAC — SC-06). */
export const RequireRoles = (...roles: Array<'resident' | 'administrator'>) =>
  SetMetadata(ROLES_KEY, roles);

export interface AuthenticatedUser {
  id: string;
  role: 'resident' | 'administrator';
  jti: string;
}

/**
 * Bearer-JWT guard. Validates signature + expiry + algorithm + issuer, then
 * re-checks live account state from the database on every request: a locked,
 * deactivated, or deleted user is rejected even while their token is still
 * within its validity window (backend authorization independent of token
 * validity, per the stack decision record).
 */
@Injectable()
export class JwtAuthGuard implements CanActivate {
  constructor(
    private readonly reflector: Reflector,
    @Inject(DATA_STORE) private readonly store: DataStore,
  ) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    const request = context.switchToHttp().getRequest<Request>();
    const header = request.headers.authorization ?? '';
    const [scheme, token] = header.split(' ');
    if (scheme !== 'Bearer' || !token) {
      throw new UnauthorizedException({
        type: 'https://always-together.local/errors/unauthorized',
        title: 'Unauthorized',
        status: 401,
        code: 'MISSING_TOKEN',
      });
    }

    const secret = process.env.JWT_SECRET ?? '';
    const claims = await verifyAccessToken(secret, token);
    if (!claims) {
      throw new UnauthorizedException({
        type: 'https://always-together.local/errors/unauthorized',
        title: 'Unauthorized',
        status: 401,
        detail: 'Invalid or expired token.',
        code: 'INVALID_TOKEN',
      });
    }

    const user = this.store.users.findById(claims.sub);
    if (!user || !user.isActive || user.isLocked) {
      throw new UnauthorizedException({
        type: 'https://always-together.local/errors/unauthorized',
        title: 'Unauthorized',
        status: 401,
        detail: 'Account is not active.',
        code: 'ACCOUNT_INACTIVE',
      });
    }

    // RBAC check (SC-06): if the route declares roles, enforce them here.
    const required =
      this.reflector.getAllAndOverride<Array<'resident' | 'administrator'>>(
        ROLES_KEY,
        [context.getHandler(), context.getClass()],
      );
    if (required && required.length > 0 && !required.includes(user.role)) {
      throw new UnauthorizedException({
        type: 'https://always-together.local/errors/forbidden',
        title: 'Forbidden',
        status: 403,
        detail: 'Insufficient role for this operation.',
        code: 'INSUFFICIENT_ROLE',
      });
    }

    (request as Request & { user?: AuthenticatedUser }).user = {
      id: user.id,
      role: user.role,
      jti: String(claims.jti ?? ''),
    };
    return true;
  }
}
