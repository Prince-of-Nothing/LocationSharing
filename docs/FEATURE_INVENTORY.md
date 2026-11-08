# Pinpoint — Complete Functionality Inventory

Every user-facing + backend feature, with its code location and automation
coverage. Live automation: `scripts/e2e.ps1` (needs stack up; exit code =
# failed checks). Unit/contract probes: `cargo test` (86 lib + 10 contract +
3 router + 1 ignored live round-trip).

## Desktop client (eframe/egui, 10 screens)

| # | Feature | Where |
|---|---|---|
| C1 | Map-first landing (offline demo pins, precision colors) | `ui.rs` Map, `map.rs` |
| C2 | Publish location (lat/lon + accuracy, validated, rate-limited) | `ui.rs` Map, `app.rs` |
| C3 | Navigate to friend (OSM directions deep link) + Open on OSM | `ui.rs`, `map.rs` |
| C4 | Nominatim search (debounced, 1/s throttle) + navigate results | `ui.rs`, `osm.rs` |
| C5 | Friends: send request, accept/reject, sent-list, unfriend | Friends screen |
| C6 | Sharing: create (direction × precision × expiry × label), pause/resume, delete, freshness badge | Sharing screen |
| C7 | Places: OSRM route (foot/bike/driving), Overpass POIs (9 categories), reverse geocode | Places screen, `osm.rs` |
| C8 | Saved places (on-device bookmarks, ★ Save from map) | Saved screen, `bookmarks.rs` |
| C9 | Trips & check-ins (manual/scheduled/trip + history) | Trips screen |
| C10 | Emergency panic (activates all shares, alerts list) | Emergency screen |
| C11 | Profile & settings (rename, metric/imperial, MFA enroll/verify/disable) | Profile screen |
| C12 | Admin guard screen, forced password change, spacing/refresh + auto-refresh poll (30s) | Admin/Password screens, `app.rs::start_auto_refresh` |

## Server (4 modular services via gateway :8000, Postgres)

| # | Feature | Service | Endpoints |
|---|---|---|---|
| S1 | Register (duplicate email rejected, PBKDF2) | auth :8001 | `POST /auth/register` |
| S2 | Login (wrong pw 401, MFA gate, seeded accounts) | auth | `POST /auth/login` |
| S3 | me / session / logout (confirmed) | auth | `GET /auth/me`, `POST /auth/session`, `POST /auth/logout` |
| S4 | Password change + force-change (real rotation) | auth | `POST /auth/password/*` |
| S5 | MFA enroll/verify/status/disable (replay-protected) | auth | `POST /auth/mfa/enroll`, `/verify`, `GET /status`, `DELETE` |
| S6 | Users CRUD + search (direct SQL so Auth's wipe never clobbers) | users :8002 | `/users/` list/create, `/users/:id` get/patch/delete, `/users/search` |
| S7 | Friendships: send, respond accept/reject, delete (+ share cascade) | users | `/friendships/` + `/:id/respond` |
| S8 | Shares: create w/ `duration_hours`+`label`, patch `is_active`/precision/direction, delete | locations :8003 | `/locations/shares` (`:id`) |
| S9 | Publish + own location, friend locations (server precision) | locations | `/locations/me` POST/GET, `/locations/friends` |
| S10 | Check-ins: create/list/delete | locations | `/locations/check-ins` (`:id`) |
| S11 | Emergency: trigger (publishes + activates shares), list, resolve | locations | `/locations/emergency` (`:id`) |
| S12 | Map: tile proxy (CARTO + cache + attribution; `CARTO_API_KEY` env), attribution, health | map :8004 | `/map/tiles/:z/:x/:y`, `/map/attribution`, `/map/health` |
| S13 | Gateway: path routing, security headers, version-free Server, HSTS/CSP (HTTPS profile) | gateway :8000/:8443 | nginx profiles in `docker/` |

## Cross-cutting

| # | Feature | Where |
|---|---|---|
| X1 | OSM privacy envelope (friend coords never to Nominatim/OSRM/Overpass) | `osm.rs` header |
| X2 | Rate limiting (login 5/min, search 1/s, publish general) | `security/rate_limit.rs` |
| X3 | Audit logging (auth/friends/publish/MFA/share) | `security/audit.rs` |
| X4 | Bookmark encryption (AES-256-GCM, zeroized keys) | `security/encryption.rs` |
| X5 | Input validation everywhere (fail-closed) | `security/validation.rs` |
| X6 | DB: 8 normalized tables, per-service ownership writes (`ServiceKind`), JSONB rollback, role seeds | `server/state.rs`, `db/` |
| X7 | Contracts: session+CSRF double-submit pinned by mock tests; loopback-only HTTP (ADR-0006) | `tests/contract.rs`, `api.rs` |
| X8 | Geo: haversine/bearing/format/compass/freshness + tile math (offline packs) | `models.rs`, `tilepacks.rs` |

## Automation coverage

- `scripts/e2e.ps1`: 38 live checks — headers+registry(4), auth(7),
  users+friendships(9), locations+shares+check-ins+emergency(13), map(3).
  Full run = exit 0.
- `cargo test`: 89 lib (validation/rate/encryption/audit/session/signing/MFA/
  password/patterns/osm/bookmarks/geo/tilepacks + ownership/lock-key) +
  10 api contract + 4 router probes (`tests/router_probe.rs`) + 1 ignored
  on-demand live round-trip.
