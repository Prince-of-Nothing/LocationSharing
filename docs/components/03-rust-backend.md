# 03 — Rust / Axum Backend (current)

> **Status:** ✅ **Current / live.** This is the backend that ships today.
> Dir: `src/server/` (crate `pinpoint-desktop`).

## Overview

The Rust backend replaces the archived Go and Python services. It implements
the same `/api/v1` wire contract with Axum 0.7 + sqlx 0.8, keeps an in-memory
store for demo/tests, persists to PostgreSQL when `DATABASE_URL` is set, and
adds a map tile proxy with ODbL-compliant multi-source fallback. It is split
into independently deployable services (auth / users / locations / map) plus a
combined `pinpoint-server` binary.

## Stack

| Concern | Choice |
|---|---|
| Language | Rust 1.99 (edition 2021) |
| HTTP | Axum 0.7 (matchit `:param` syntax) |
| Async runtime | tokio |
| DB | sqlx 0.8 (postgres, rustls) |
| Auth | session cookie + `X-CSRF-Token` double-submit (`src/api.rs`) |
| Passwords | Argon2id (`src/security/password.rs`) |
| MFA | TOTP (`src/security/mfa.rs`) |
| Config | env (`BIND_ADDR`, `DATABASE_URL`, `CARTO_API_KEY`, …) |

## Module map

```
src/server/
├── mod.rs                 # routers, converge middleware, map_handlers (tile proxy)
├── handlers.rs            # health, sync_status, user/share JSON helpers
├── auth_handlers.rs       # login/register/logout/session/me/password/mfa
├── users_handlers.rs      # users + friendships CRUD/search/respond
├── locations_handlers.rs  # shares, publish, friend locations, check-ins, emergency
├── migrations.rs          # typed idempotent migrations v1..v12
└── state/
    ├── mod.rs             # AppState, ServiceKind ownership, refresh/persist
    ├── store.rs           # aggregate Store shape + snapshot
    ├── users.rs           # users table load/persist
    ├── friendships.rs     # friendships table load/persist
    └── locations.rs       # shares/locations/checkins/emergencies load/persist

src/bin/
├── server.rs              # combined binary (all services)
├── auth.rs                # auth-only service
├── users.rs               # users + friendships service
├── locations.rs           # locations + shares service
└── map.rs                 # map/tile-proxy service
```

## API surface

| Method | Path | Service | Notes |
|---|---|---|---|
| GET | `/health` | all | DB check; `degraded` + 503 when Postgres unreachable |
| GET | `/api/v1/sync-status` | all | replication freshness (`last_persist_at`, `fresh`) |
| POST | `/api/v1/auth/login` | auth | `{access_token, csrf_token, user}` + cookie |
| POST | `/api/v1/auth/register` | auth | create account |
| POST | `/api/v1/auth/logout` | auth | revoke session |
| POST | `/api/v1/auth/session` | auth | refresh idle timer + CSRF |
| GET | `/api/v1/auth/me` | auth | current profile |
| POST | `/api/v1/auth/password/change` | auth | revokes all sessions |
| POST | `/api/v1/auth/password/force-change` | auth | `must_change_password` |
| POST/GET/DELETE | `/api/v1/auth/mfa/*` | auth | enroll / verify / status / disable |
| GET/POST | `/api/v1/users/` | users | list / admin create |
| GET/PATCH/DELETE | `/api/v1/users/:id` | users | fetch / update / delete |
| GET | `/api/v1/users/search` | users | search |
| GET/POST | `/api/v1/friendships/` | users | list / create |
| POST | `/api/v1/friendships/:id/respond` | users | accept/reject |
| DELETE | `/api/v1/friendships/:id` | users | delete |
| GET/POST | `/api/v1/locations/shares` | locations | list / create |
| PATCH/DELETE | `/api/v1/locations/shares/:id` | locations | pause/resume/precision / delete |
| POST/GET | `/api/v1/locations/me` | locations | publish / own location |
| GET | `/api/v1/locations/friends` | locations | visible friend locations |
| GET/POST | `/api/v1/locations/check-ins` | locations | list / create |
| DELETE | `/api/v1/locations/check-ins/:id` | locations | delete |
| GET/POST | `/api/v1/locations/emergency` | locations | list / trigger |
| PATCH | `/api/v1/locations/emergency/:id` | locations | resolve |
| GET | `/map/tiles/:z/:x/:y` | map | ODbL tile proxy (CARTO → Wikimedia → OpenFreeMap) |
| GET | `/map/attribution` | map | ODbL attribution + source list |
| GET | `/map/health` | map | service health + cache size |

## Data model

`src/models.rs` defines the shared domain types (also used by the client):

- **User** — id, email, full_name, role, is_active, must_change_password,
  `mfa_enabled`, `mfa_last_step`, `failed_login_attempts`, `locked_until`.
- **LoginResponse** — `{access_token, csrf_token, user}`.
- **Friendship** — requester/receiver, status, timestamps.
- **LocationShare** — owner, friendship, direction, precision, is_active,
  `last_location_update_at`, `expires_at`, `label`.
- **FriendLocation** — user_id, user_name, lat/lon, precision, updated_at,
  accuracy.
- **CheckIn** — user, lat/lon, note, kind, created_at.
- **EmergencyAlert** — user, lat/lon, created_at, resolved.
- **Settings** — refresh interval, metric, default precision, visibility,
  language.

## Storage & replication

- **Target schema:** normalized + application-encrypted
  (`docs/database/schema.md`, ADR-0010, `db/schema/00-target.sql`); the
  runtime currently uses the `pinpoint_*` tables + a `pinpoint_state`
  rollback snapshot and must migrate to the target.
- **In-memory:** `Store` behind `Arc<Mutex<…>>` for demo/tests/local.
- **Postgres:** `sqlx::PgPool`; `AppState::persist()` writes normalized tables
  (`users`, `friendships`, `shares`, `locations`, `checkins`, `emergencies`)
  plus a JSON snapshot row, under a `pg_advisory_xact_lock` per `ServiceKind`.
- **Service ownership (`ServiceKind`):** each deployable only persists tables it
  owns (`All`, `Auth`, `Users`, `Locations`, `MapOnly`), so replicas never
  clobber each other.
- **Convergence middleware:** every request refreshes from Postgres before the
  handler and persists after mutations, giving read-after-write consistency
  across replicas (no-op in in-memory mode).
- **Migrations:** `src/server/migrations.rs` (v1–v12, typed, idempotent,
  transactional) + `db/02-grants.sql` + `db/init/*` for least-privilege roles.

## Security controls

- **Passwords:** Argon2id (PHC, memory-hard; OWASP-aligned) in
  `src/security/password.rs`; legacy PBKDF2 hashes verify and re-hash on login.
- **Sessions/CSRF:** session cookie + `X-CSRF-Token` double-submit.
- **MFA:** TOTP enroll/verify/status/disable with replay protection
  (RFC 6238 HMAC-SHA256, 30 s step, ±1 drift).
- **OAuth2/OIDC + PKCE:** provider/PKCE/state/redirect primitives in
  `src/security/oauth.rs`; login wiring pending.
- **Rate limiting / lockout:** failed-login lockout with expiry; client-side
  rate limiter in `src/security/rate_limit.rs`; server-side limits pending.
- **Authorization:** directional share checks on every location read/write;
  server-side enforcement (client is untrusted).
- **Transport:** HTTPS via nginx/Caddy in deployment (ADR-0006; plain HTTP only
  for loopback dev).
- **Map policy:** never `tile.openstreetmap.org`; ODbL attribution always
  returned.

## Authentication & sessions

- **Models (ADR-0009):** email+password (identity) and anonymous device-bound
  accounts (one-time recovery code), with an upgrade path (link an identity
  later).
- **Session model (ADR-0012 + `docs/security/session-management.md`):**
  opaque 256-bit `pinpoint_sessions` row delivered as an `HttpOnly;
  SameSite=Lax` cookie + `X-CSRF-Token` double-submit; sliding idle timeout;
  immediate revocation on logout/logout-all/session revoke/password change;
  `mfa_pending` gate. No JWT, no refresh token.
- **Shared across services:** sessions live in Postgres so auth/users/
  locations replicas validate the same session; in-memory fallback for the
  single `pinpoint-server` binary.
- **Status:** implemented — handlers (`src/server/auth_handlers.rs`), store
  (`state/sessions.rs`), middleware (`session_auth`) and the web UI. Remaining
  follow-ups: anonymous device challenge-response and rate limiting on
  anonymous creation/recovery (`session-management.md` §10).

## Build / run / test

```bash
cargo run --bin pinpoint-server                 # 127.0.0.1:8000 (in-memory)
DATABASE_URL=postgres://… cargo run --bin pinpoint-server
cargo run --bin pinpoint-auth                   # auth-only service
cargo run --bin pinpoint-map                    # map/tile-proxy service
cargo test --locked                             # unit + contract + router probes
```

## Deployment

- Binaries: `pinpoint-server` (all-in-one) or the per-service binaries.
- Gateway: `docker/nginx-gateway.conf` / `nginx-gateway-https.conf`.
- Compose: `docker-compose.yml` (+ `.ha.yml`, `.https.yml`).
- Health: `/health`, `/api/v1/sync-status`; map: `/map/health`.

## Relationship to other components

- **Supersedes:** components 1 (Go) and 2 (Python) — same `/api/v1` contract.
- **Serves:** component 4 (Rust client) primarily; also the Kotlin app
  (component 5) and the web stubs (component 6 successor in `web/`).
- **Shares code with:** component 4 (`src/models.rs`, `src/api.rs` contract,
  `src/security/*`).
