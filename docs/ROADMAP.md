# Roadmap & Future Plans

Statuses: ✅ done · 🚧 in progress · 📋 planned · 🔬 research

## Phase 0 — Foundation (MVP) ✅

- [x] Rust multiplatform pivot (ADR-0007): one crate (`pinpoint-desktop`) for
  desktop, Android, iOS and a wasm stub
- [x] Axum API with split auth/users/locations services behind a gateway
  (ADR-0008) + single combined binary for the demo
- [x] Directional consent model (precision, purpose, expiry, pause/resume)
- [x] Friendship workflow (request/accept/reject)
- [x] Location publishing + Slippy map (OSM tiles via the server proxy,
  Nominatim/OSRM/Overpass in `src/osm.rs`)
- [x] Real server-side sessions (ADR-0012): opaque `HttpOnly` cookie + CSRF
  double-submit, login/register/logout, session list/revoke, logout-all,
  sliding idle timeout
- [x] Argon2id password hashing with transparent legacy-PBKDF2 rehash
- [x] Anonymous device-bound accounts (ADR-0009 phase 2): one-time recovery
  code, rotation on recovery
- [x] Web UI at `/` (`web/app/`): auth (email + anonymous), map, publish,
  friends, shares, active sessions; strict CSP, no third-party scripts
- [x] Docker deployment: split services + gateway; zero-config demo seeding
- [x] Contribution/process docs (`CONTRIBUTING.md`, `docs/`)

## Phase 1 — Hardening & persistence 🚧 *(next up)*

- [x] Postgres **schema** + migrations 1–14 (`src/server/migrations.rs`),
  reference DDL (`db/schema/00-target.sql`) and ADR-0010
- [x] Session store in Postgres (shared across replicas); in-memory fallback
- [ ] Persist users / friendships / locations writes through the Postgres
  repositories (schema and load/persist helpers exist; the demo path is
  in-memory)
- [ ] Redis-backed rate limiting + job queue (scheduled check-ins)
- [ ] WebSocket channel for realtime location updates (replaces polling)
- [ ] Audit log table (who accessed whose location, when, under which consent)
- [ ] Structured logging + request IDs (no IPs, no session ids)
- [x] CI pipeline: `fmt`, `clippy -D warnings`, locked build/test,
  `cargo-audit`, map QA
- [ ] ZAP baseline/active scan in CI + archived report

## Phase 2 — Clients 🚧

- [x] Web app (`web/app/`) — login/register/anonymous, map, friends, shares,
  sessions
- [x] Rust desktop client (`src/main.rs`, eframe/egui) against the same API
- [ ] **Android** (`cargo-apk`, `android/`): scaffold + `network_security_config`
  landed; build blocked on NDK/JDK 21 installation
- [ ] **iOS** (`cargo-xcodebuild`, `ios/Info.plist`): scaffold landed; needs
  macOS/Xcode
- [ ] **WASM web** (`src/bin/pinpoint-web.rs` via Trunk): stub; needs the wasm
  toolchain and a browser test pass
- [ ] `weba` — dedicated admin console
- [ ] `webk` — lightweight kiosk/check-in view

## Phase 3 — Feature extensions (each gated by its own threat analysis) 🔬

- [ ] Scheduled & trip check-ins (missed-check-in workflows)
- [ ] Movement trails + search corridors — *high privacy risk; opt-in, expiring
  and encrypted*
- [ ] Visited-region maps — *aggregation leaks habits; k-anonymity required*
- [ ] Approximate-area presets ("home zone", custom geofences)
- [ ] Emergency mode (panic share to preselected contacts with expiry)
- [ ] Friend mosaics / group views — *only with unanimous per-member consent*

## Phase 4 — Trust & scale 📋

- [ ] OAuth 2.0/OIDC with PKCE, magic links and passkeys (ADR-0009 phases 4–5)
- [ ] Anonymous device challenge-response (`/auth/anonymous/verify`, Ed25519)
- [ ] Rate limiting on anonymous creation/recovery (keyed HMAC of the client
  address, never raw IP, short retention)
- [ ] Self-host Nominatim/OSRM/Overpass/tiles in production (ADR-0011; env
  overrides already wired)
- [ ] End-to-end encryption options for location payloads
- [ ] Horizontal scaling: stateless API + shared Postgres/Redis
- [ ] Moderation tooling, abuse reporting, GDPR export/delete jobs
- [ ] Public hosting + mobile-store releases

## Backlog / ideas parking lot

- Battery-aware adaptive precision (auto-downgrade when battery < 15%)
- Accessibility pass (screen-reader labels, reduced motion, high-contrast map)
- i18n for the web client (the `i18n` registry already has 79 languages)
- Load testing harness (k6) for the locations hot path
- Terraform deployment for a small VPS

## Decision records worth noting

1. **Rust over Go/Python** (ADR-0007) — one codebase for desktop, Android, iOS
   and the web stub; the Go/Python/Kotlin trees are archived.
2. **Opaque sessions over JWT + refresh** (ADR-0012) — server-side revocation
   is immediate and one credential is easier to protect than two.
3. **Normalized, encrypted Postgres schema** (ADR-0010) — no single JSONB blob;
   application-level AES-256-GCM columns with blind indexes.
4. **Self-hosted OSM** (ADR-0011) — public tile/geocoding services are demo
   fallbacks only; ODbL attribution always stays.
5. **Vanilla-JS web first** — zero build chain keeps the thesis reproducible;
   the CSP forbids third-party scripts.

## Non-goals (explicit)

- No advertising, profiling, data sales or third-party analytics.
- No IP/WebRTC geolocation and no raw-IP logging (see `PRIVACY.md` and the
  threat model).
- No background tracking: only explicitly published locations are stored.
