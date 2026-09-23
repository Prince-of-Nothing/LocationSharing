import { Controller, Get } from '@nestjs/common';

/** Liveness & readiness probes for Kubernetes (deploy/k8s manifests). */
@Controller()
export class HealthController {
  @Get('health/live')
  live() {
    return { status: 'ok' };
  }

  @Get('health/ready')
  ready() {
    // Extend with Prisma $queryRaw`SELECT 1` and Redis PING when wired.
    return { status: 'ready' };
  }
}
