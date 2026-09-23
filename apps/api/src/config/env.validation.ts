import { plainToInstance } from 'class-transformer';
import {
  IsInt,
  IsString,
  Max,
  Min,
  MinLength,
  validateSync,
} from 'class-validator';

/**
 * Environment configuration (SC-09: secrets come only from the environment,
 * injected by Docker Compose / Kubernetes Secrets — never committed).
 */
export class EnvironmentVariables {
  @IsString()
  NODE_ENV: 'development' | 'test' | 'production' = 'development';

  @IsInt()
  @Min(1)
  @Max(65535)
  PORT = 3000;

  /** Base path for the versioned API, e.g. /api/v1 */
  @IsString()
  API_PREFIX = '/api/v1';

  /** Signing secret for JWT access tokens. Must be >= 32 chars in production. */
  @IsString()
  @MinLength(32, {
    message: 'JWT_SECRET must be at least 32 characters (see .env.example)',
  })
  JWT_SECRET!: string;

  @IsInt()
  @Min(5)
  @Max(60)
  ACCESS_TOKEN_EXPIRE_MINUTES = 15;

  /** PostgreSQL connection string. Optional in tests (in-memory repository). */
  @IsString()
  DATABASE_URL = '';

  /** Redis connection string. Optional; falls back to in-memory rate limiter. */
  @IsString()
  REDIS_URL = '';

  /** Login throttling (SC-04) */
  @IsInt()
  @Min(3)
  LOGIN_MAX_ATTEMPTS = 5;

  @IsInt()
  @Min(60)
  LOGIN_LOCKOUT_SECONDS = 900;

  /** Global API rate limit (SC-05): requests per window per IP+route. */
  @IsInt()
  @Min(1)
  RATE_LIMIT_MAX = 100;

  @IsInt()
  @Min(1000)
  RATE_LIMIT_WINDOW_MS = 60_000;

  /** Stale-location threshold in minutes (REQ-06). */
  @IsInt()
  @Min(1)
  LOCATION_STALE_MINUTES = 15;
}

export function validateEnv(
  config: Record<string, unknown>,
): EnvironmentVariables {
  const validated = plainToInstance(EnvironmentVariables, config, {
    enableImplicitConversion: true,
    exposeDefaultValues: true,
  });
  const errors = validateSync(validated, {
    skipMissingProperties: false,
    whitelist: false,
  });
  if (errors.length > 0) {
    throw new Error(
      `Invalid environment configuration:\n${errors
        .map((e) => `  - ${Object.keys(e.constraints ?? {}).join(',')} on ${e.property}: ${Object.values(e.constraints ?? {}).join(' ')}`)
        .join('\n')}`,
    );
  }
  return validated;
}
