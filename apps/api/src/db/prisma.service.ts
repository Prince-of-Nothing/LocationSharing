import { Injectable, OnModuleInit } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { PrismaClient } from '@prisma/client';

/**
 * Thin wrapper around PrismaClient so the rest of the app depends on an
 * injectable provider rather than a global singleton. When DATABASE_URL is
 * empty (unit/security tests), Prisma is not connected and callers fall back
 * to InMemoryRepository.
 */
@Injectable()
export class PrismaService extends PrismaClient implements OnModuleInit {
  public readonly enabled: boolean;

  constructor(private readonly config: ConfigService) {
    super({ log: ['warn', 'error'] });
    this.enabled = !!this.config.get<string>('DATABASE_URL');
  }

  async onModuleInit(): Promise<void> {
    if (this.enabled) {
      await this.$connect();
    }
  }
}
