import { Injectable, NestMiddleware } from '@nestjs/common';
import { Request, Response, NextFunction } from 'express';
import { problemDetails } from '../problem-details';

interface Bucket { count: number; resetAt: number; }

/**
 * SC-05 — Fixed-window rate limiter keyed on client IP + route.
 * In-memory for MVP/dev/test; behind multiple Kubernetes replicas the same
 * interface is backed by Redis (INCR + EXPIRE) so limits are global —
 * see deploy/k8s and docs/architecture/deployment-view.md.
 */
@Injectable()
export class RateLimitMiddleware implements NestMiddleware {
  private buckets = new Map<string, Bucket>();
  private readonly max: number;
  private readonly windowMs: number;

  constructor(max = 100, windowMs = 60_000) {
    this.max = max;
    this.windowMs = windowMs;
    // Periodic cleanup so the map cannot grow unboundedly (memory DoS).
    setInterval(() => {
      const now = Date.now();
      for (const [k, b] of this.buckets) if (b.resetAt <= now) this.buckets.delete(k);
    }, windowMs).unref?.();
  }

  use(req: Request, res: Response, next: NextFunction): void {
    const ip =
      (req.headers['x-forwarded-for'] as string | undefined)?.split(',')[0].trim() ??
      req.socket.remoteAddress ??
      'unknown';
    const key = `${ip}:${req.method} ${req.baseUrl ?? ''}${req.route?.path ?? req.path}`;
    const now = Date.now();
    let bucket = this.buckets.get(key);
    if (!bucket || bucket.resetAt <= now) {
      bucket = { count: 0, resetAt: now + this.windowMs };
      this.buckets.set(key, bucket);
    }
    bucket.count += 1;

    res.setHeader('X-RateLimit-Limit', String(this.max));
    res.setHeader('X-RateLimit-Remaining', String(Math.max(0, this.max - bucket.count)));
    res.setHeader('X-RateLimit-Reset', String(Math.ceil(bucket.resetAt / 1000)));

    if (bucket.count > this.max) {
      res
        .status(429)
        .setHeader('Retry-After', String(Math.ceil((bucket.resetAt - now) / 1000)))
        .type('application/problem+json')
        .json(
          problemDetails({
            title: 'Too Many Requests',
            status: 429,
            code: 'RATE_LIMITED',
            detail: 'Rate limit exceeded. Try again later.',
            instance: req.url,
          }),
        );
      return;
    }
    next();
  }
}
