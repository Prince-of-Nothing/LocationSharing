# Pinpoint — Redundancy (backup if one service fails)

## What "backup" means here

Kill any single `auth` / `users` / `locations` / `map` replica and
requests keep working through the surviving one. The gateway resolves
upstreams per-request (Docker DNS `127.0.0.11`) and retries the next
replica on connection errors / 502–504 (`proxy_next_upstream` in
`docker/nginx-gateway*.conf`), so a failed replica is invisible to
clients. Every service also has `restart: unless-stopped` + a Docker
`HEALTHCHECK` (`/health` on each service, `/map/health` on map) and
Compose `depends_on: service_healthy` for the gateway.

## Layout (default)

| Service | Replicas | State | Failover cost |
|---|---|---|---|
| `auth` (`:8001`) | **2** (`scale`/`deploy.replicas`) | Shared Postgres rows; in-memory store converges every 5s (`persist()` → `refresh_from_db()`) | One request may hit the replica that hasn't converged yet ≤5s — acceptable, no data loss (both re-read the DB) |
| `users` (`:8002`) | **2** | Same convergence; direct-SQL mutations bypass the wipe-rewrite | Same ≤5s staleness window |
| `locations` (`:8003`) | **2** | Same; share publish is idempotent per `user_id` | Same window |
| `map` (`:8004`) | **2** | Stateless; tile cache is per-process (miss → upstream fetch) | A tile may be refetched once after failover |
| `gateway` (`:8000`) | 1 | Restarts automatically | Single LB today — two gateways behind your own LB is the prod step (published port can't be shared) |
| `db` (Postgres) | 1 primary | `pinpoint-db-data` volume | **Still single-primary** — see below |
| `db-backup` | 1 | `pg_dump` every 300s → `pinpoint-backups` (keep 24) | Point-in-time recovery, not zero-loss failover |

```bash
docker compose up --build -d          # 2 replicas of each service by default
docker compose ps                     # auth-1/auth-2 ... all (healthy)
docker compose up -d --scale auth=3 --scale locations=2
```

Direct service ports (`8001–8004`) are intentionally **unpublished**
(`expose` only) so replicas never collide on host ports. Clients use
the gateway only. To debug one replica directly:

```bash
docker compose run --rm --no-deps -p 8001:8001 auth
```

## What's *not* redundant (honest gaps)

- **Postgres primary** — one instance by design (the DB stays connected
  for joins; see `docs/architecture-data-domains.md`). Recovery today is
  `pinpoint-db-data` volume restart or restoring the latest
  `pinpoint-backups/pinpoint-*.sql`. Zero-loss HA (streaming replica +
  Patroni/Stolon, or a managed Postgres with automated failover) is the
  production step.
- **Concurrent writers** — replicas converge via the DB every 5s, but
  `persist()` is wipe-rewrite per owned table: two replicas of the *same*
  service can clobber each other if both mutate in the same window.
  At demo scale (low write rate, demo clock) this is rare; the
  production fix is row-level upserts + per-service leader election,
  not more replicas.
- **Gateway** is a single nginx container (auto-restarts). Client cache
  per replica means one tile miss after a map failover.
- **`container_name` conflicts** (the old `docker compose up` errors)
  are gone for the scaled services; `pinpoint-db` / `pinpoint-gateway`
  keep names because they are singletons.

## Failover check (manual)

```bash
docker compose up --build -d
Invoke-RestMethod http://127.0.0.1:8000/health          # 200
docker kill $(docker compose ps -q auth | Select-Object -First 1)
Invoke-RestMethod http://127.0.0.1:8000/api/v1/auth/me # still served by the survivor
docker compose ps                                       # killed replica (exited), survivor (healthy)
docker compose up -d auth                               # replica set back to 2
powershell -ExecutionPolicy Bypass -File scripts/e2e.ps1  # full suite still green
```
