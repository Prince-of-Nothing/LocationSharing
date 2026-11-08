# 0008. Split Rust API into Independently Scalable Services Behind a Gateway

Date: 2026-11-08

## Status

Accepted

## Context

What is the issue we're addressing? What forces (technical, business, constraints)
are at play? Describe the problem neutrally, without arguing for a solution yet.

Pinpoint originally shipped as a single Axum process (`pinpoint-server`,
`src/bin/server.rs`) serving every `/api/v1` route plus health. That is fine
for a 3-day demo but cannot scale by capability: map tiles/attribution
(OSM ODbL), auth sessions, users/friendships, and location shares have
very different load profiles and failure isolation needs. Review asked
whether the stack should be "very modular, so we can split processing
power for different stuff on different machines" instead of two
deployables (app + DB) only.

Constraints: everything must stay pure Rust (ADR-0007), no Go/Python/JS
outside `archive/`; OpenStreetMap policy forbids `tile.openstreetmap.org`
for app use (`docs/sdlc/07-openstreetmap-integration.md`).

## Decision

We will keep one Rust codebase and one shared handler library
(`src/server/`) but build **four independently runnable binaries** from
it, each with its own Axum router constructor:

- `pinpoint-auth` (`src/bin/auth.rs`, `auth_router()`) — `/api/v1/auth/*`
- `pinpoint-users` (`src/bin/users.rs`, `users_router()`) — `/api/v1/users/*`, `/api/v1/friendships/*`
- `pinpoint-locations` (`src/bin/locations.rs`, `locations_router()`) — `/api/v1/locations/*`
- `pinpoint-map` (`src/bin/map.rs`, `map_router()`) — `/map/tiles/{z}/{x}/{y}`, `/map/attribution`, `/map/health`

A stateless combinator `router(state)` merges them for local dev
(`cargo run --bin pinpoint-server`, `BIND_ADDR` default `127.0.0.1:8000`).
In Docker, an nginx reverse proxy (`docker/nginx-gateway.conf`, service
`gateway` on host port 8000) routes by path prefix to the service
containers; `db` (Postgres 16) is the shared persistence layer via
`pinpoint_state`. Every service image is built from the same
`Dockerfile.service` with `--build-arg BIN=...`, non-root user, and a
`HEALTHCHECK`. Compose (`docker-compose.yml`) publishes 8001–8004 for
direct debugging and 8000 via the gateway for clients.

Alternatives rejected:

- **Single monolith only.** Simpler, but auth and location hot paths
  contend with tile load; no independent scaling or rolling deploys.
- **Separate repos per service.** Rejected: ADR-0007 mandates one Rust
  codebase; the router-split achieves the same isolation without code
  duplication.
- **Serverless / managed-map proxy.** Rejected for now: per OSM policy
  Pinpoint must serve its own attribution and never scrape tiles in
  bulk; a thin Rust map service keeps attribution + health co-located
  and swappable for a self-hosted tile cache later.

## Consequences

Easier:

- Scale any service independently (`docker compose up --scale locations=3`);
  map tile load can no longer starve login or publish.
- Roll, restart, or test one service without touching others
  (`docker compose up -d auth`).
- Direct per-service debugging on ports 8001–8004 plus a single gateway
  port 8000 for the desktop client (`--backend http://localhost:8000`).

Harder / risks:

- Cross-service consistency still depends on the shared `pinpoint_state`
  JSONB snapshot; production must normalize it into typed tables and
  consider schema ownership per service (see follow-up).
- nginx config drift: new routes require a gateway location block, and
  `auth`/`users`/`locations` each persist — concurrent writers can
  clobber snapshots until normalized tables land.
- Four images to build in CI (fine — same `Dockerfile.service`, cached
  layers).

Follow-ups:

- Normalize `pinpoint_state` into per-service tables; add a migration
  tool; hash passwords with bcrypt/Argon2id instead of demo any-password.
- Add mTLS / service tokens between gateway and services; move
  `DATABASE_URL` into a vault; add tracing IDs propagated by nginx.
- Graduate the map service from a `410 GONE` tile proxy to a
  policy-compliant self-hosted cache (CARTO/S3) with proper ODbL
  credit on every response.
