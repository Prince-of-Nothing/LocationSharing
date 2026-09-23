import { MiddlewareConsumer, Module, NestModule } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { APP_FILTER } from '@nestjs/core';
import { AuthModule } from './auth/auth.module';
import { AuthController } from './auth/auth.controller';
import { AuthService } from './auth/auth.service';
import { FriendshipsController } from './friendships/friendships.controller';
import { LocationsController } from './locations/locations.controller';
import { HealthController } from './health/health.controller';
import { JwtAuthGuard } from './security/auth.guard';
import { AllExceptionsFilter } from './common/filters/all-exceptions.filter';
import { RateLimitMiddleware } from './common/middleware/rate-limit.middleware';
import { DATA_STORE, InMemoryStore } from './db/in-memory.repository';
import { validateEnv } from './config/env.validation';
import type { NextFunction, Request, Response } from 'express';

/**
 * Always Together MVP — modular monolith (one deployable API).
 * The DATA_STORE provider binds the in-memory implementation for dev/test;
 * production binds a Prisma-backed implementation with identical interface.
 */
@Module({
  imports: [
    ConfigModule.forRoot({ isGlobal: true, validate: validateEnv }),
    AuthModule,
  ],
  controllers: [
    AuthController,
    FriendshipsController,
    LocationsController,
    HealthController,
  ],
  providers: [
    AuthService,
    JwtAuthGuard,
    { provide: DATA_STORE, useClass: InMemoryStore },
    { provide: APP_FILTER, useClass: AllExceptionsFilter },
  ],
})
export class AppModule implements NestModule {
  configure(consumer: MiddlewareConsumer): void {
    const max = Number(process.env.RATE_LIMIT_MAX ?? 100);
    const windowMs = Number(process.env.RATE_LIMIT_WINDOW_MS ?? 60_000);
    const limiter = new RateLimitMiddleware(max, windowMs);
    consumer
      .apply(limiter.use.bind(limiter) as unknown as (req: Request, res: Response, next: NextFunction) => void)
      .forRoutes('*');
  }
}
