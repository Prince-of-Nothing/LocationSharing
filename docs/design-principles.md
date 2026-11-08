# Pinpoint — Design Principles (modularity + quality)

How the codebase applies five widely-used design principles, with the
exact files that implement each. The goal: every feature lives in its own
module, services own their data, and seams are explicit enough that a new
service or table is additive, never a rewrite.

## 1. Single Responsibility — one reason to change

| Module | Responsibility | Evidence |
|---|---|---|
| `server/auth_handlers.rs` | `/api/v1/auth/*` (login, MFA, passwords) | 353 lines, own routes in `mod.rs` |
| `server/users_handlers.rs` | `/api/v1/users/*`, `/api/v1/friendships/*` | 348 lines |
| `server/locations_handlers.rs` | `/api/v1/locations/*` | 460 lines |
| `server/map_handlers` (mod.rs) | `/map/*` tiles/attribution | isolated router |
| `server/` binaries | one per deployable (`src/bin/{auth,users,locations,map}.rs`) | each serves only its router |
| `osm.rs` | OSM network calls (Nominatim/OSRM/Overpass) | contract tests pin it |
| `platform.rs` | every `cfg(target_os)` branch — nothing else | AGENTS.md rule |
| `bookmarks.rs` | on-device JSON store | 2 tests |
| `tilepacks.rs` | offline tile download (SLIPPY+BBOX) | 3 tests |
| `map.rs` | pin rendering + location list (distance/compass/freshness from `models.rs`) | borrow-safe |
| `security/{validation,rate_limit,encryption,audit,session,signing,mfa,password}.rs` | one mechanism each | 30+ unit tests |

`handlers.rs` is now shared-only (helpers + health, 54 lines) — the god-file split happened 2026-11-08.
`ui.rs` is now the 612-line app shell (dispatch, header, toolbar, pending actions);
each screen is its own module (`src/ui/{auth,map,friends,sharing,places,bookmarks,
trips,emergency,profile,admin}.rs`, 55–279 lines each) sharing `PinpointAppUI` via
`impl` blocks — render functions are independently compilable and testable,
state ownership unchanged.

## 2. Separation of Concerns — layers never leak

- **Transport** (Axum routers/handlers) never contains business rules:
  precision rounding, consent checks, and expiry live in handlers → `state.rs` domain data.
- **Domain** (`models.rs`) is serde-only data + pure math (haversine/bearing/formatting).
- **OSM** (`osm.rs`) only translates HTTP↔types; never knows about friendships.
- **Client** (`api.rs` → `app.rs` → `ui.rs`): HTTP contract / state orchestration /
  egui rendering. UI never builds URLs or parses JSON (tests/contract.rs pins api.rs).

## 3. Modular Architecture — services & database

- Four deployables + combined binary; nginx gateway routes by prefix
  (`docker/nginx-gateway.conf`); each service scales independently
  (`docker compose up --scale locations=3`).
- **Modular database:** 8 normalized tables with per-service ownership enforced
  in code (`ServiceKind` in `state.rs`): Auth owns users/counters, Users owns
  users+friendships, Locations owns shares/locations/check-ins/emergencies,
  Map owns nothing. `persist()` upserts per-row + deletes only the missing
  keys inside one transaction serialized by `pg_advisory_xact_lock` (no
  wipe-rewrite; replicas of the same service take turns, different
  services run concurrently).
  Roles + per-table grants staged in `db/init/01-roles.sh` / `db/02-grants.sql`.
- Migration path: JSONB snapshot (`pinpoint_state`) stays as rollback; normalized
  load is authoritative once present.

## 4. DRY / Dependency Inversion

- Shared helpers (`current_user`, `user_json`, `share_json`, `is_expired`) live once
  in `handlers.rs` as `pub(crate)`.
- `OsmClient` constructed via `patterns::factory` (`osm_client()`);
  client/state use `StateContext` transitions; repository pattern in
  `patterns/repository.rs` (in-memory + future Postgres impl behind a trait).
- Validation lives in `security/validation.rs`; handlers and client call the
  same functions (no duplicated regex/range checks).

## 5. Fail-Closed & Contract-First

- Transport policy (ADR-0006): non-loopback HTTP refused.
- Login/Logout fail closed (contract tests).
- MFA replay protection (consumed TOTP steps).
- Errors never leak internals (envelope + upstream status only for tile proxy).

## 6. What remains (honest gaps)

- ~~`ui.rs` (1652 lines)~~ → `ui.rs` 612-line shell + `src/ui/{auth,map,friends,sharing,
  places,bookmarks,trips,emergency,profile,admin}.rs` (55–279 lines each, done 2026-11-08).
- DB roles (`pinpoint_auth/users/locations`) created by `db/init/01-roles.sh` and
  **live in compose since 2026-11-08** (per-service `DATABASE_URL`s); per-table
  grants from `db/02-grants.sql` applied + verified (`current_user` probes).
- The `pinpoint` superuser remains ONLY as DB bootstrap (`docker-entrypoint`
  init scripts create roles) and migration/rollback operator — application
  services never connect as it.
- Stores converge: every service runs `persist()` then `refresh_from_db()` each
  5s tick, so cross-service writes are visible within ~5s without restarts.
- In-memory `Store` per service is eventually consistent via DB; a shared
  cache/queue is the production step (not needed at demo scale).
