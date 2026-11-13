# Pinpoint — Roadmap

Living document. PRs welcome; see `CONTRIBUTING.md`.

## Shipped (v1.0.0, 2026-11-13)

- Rust multiplatform client (desktop verified; Android/iOS/wasm scaffolds) and
  a modular Axum backend (auth/users/locations/map) behind a gateway
- Real server-side sessions (ADR-0012): opaque `HttpOnly` cookie, CSRF
  double-submit, hash-stored session ids, session list/revoke, logout-all,
  sliding idle timeout
- Argon2id passwords with transparent legacy-PBKDF2 rehash
- Anonymous device-bound accounts with one-time recovery codes (rotated on use)
- Web UI at `/`: auth (email + anonymous), Slippy map via the `/map/tiles`
  proxy, publish, friends, shares, active sessions; strict CSP; local-only
  assets (no Google Fonts/CDNs), local favicon
- Desktop client: map, OSM search/reverse, OSRM routing, Overpass POIs,
  bookmarks, check-ins, emergency, MFA, profile; offline tile packs through
  the backend proxy
- Consent model (directional, precision-scoped, expiring, revocable) enforced
  server-side on every request
- Postgres schema/migrations (v1–14) incl. `pinpoint_sessions`; in-memory
  fallback for the single-binary demo
- Self-host-first OSM configuration (`PINPOINT_*_URL`, ADR-0011) and ODbL
  attribution everywhere
- 79-language i18n registry with auto-detect
- Reliability: 2 replicas + gateway failover + pg_dump backups
- 159 unit + 10 contract + 7 router + 5 asset tests; fmt/clippy/build green

## v1.1 — Navigation & map experience (in progress)

- [x] **Web search + routing**: `/map/search`, `/map/reverse`, `/map/route`
  wrappers over `src/osm.rs` with validation, cache and capabilities
- [x] **Map-first UI**: locally vendored Leaflet 1.9.4, dark tiles, friend
  pins, place sheet, route panel (alternatives, steps, swap, clear), scale
- [x] **Saved places / recent places** per account, locate-me, category chips,
  submit-on-Enter search with opt-in autocomplete
- [ ] Walk/bike routing needs `PINPOINT_OSRM_FOOT_URL` / `BIKE_URL`
  (public OSRM only serves a car profile)
- [ ] Mobile/390 px polish pass and keyboard-only audit
- [ ] Turn-by-turn follow screen with voice (v1.2)

## v1.2 — Real demo clients

- Replace the hardcoded Alice/Bob seed with `alice`/`bob` **client
  containers** that register, befriend, share and publish through the API
- `docker compose --profile demo` brings up the gateway + both clients
- Optional movement simulation and a `--headless` demo client binary

## Hardening & verification

- Postgres write-through for users/friendships/locations (demo path is still
  in-memory)
- Rate limiting on login, `/auth/anonymous` and recovery (keyed HMAC, no IPs)
- Anonymous device challenge-response (`/auth/anonymous/verify`, Ed25519)
- ZAP baseline + active scan, archived evidence
- Android APK (needs NDK + JDK 21) and WASM build (Trunk) in CI
- Final LaTeX report + thesis defense artefacts

### From the competitive analysis (`docs/components/07-competitive-analysis.md`)

- **P2:** temporary share link (Glympse-style, account-free recipients,
  expiring/revocable)
- **P2:** real-time streaming option (WebSocket/SSE) with polling fallback
- **P3:** battery-aware sampling (adapt refresh interval to battery level)
- **P3:** optional E2E-encrypted bookmark sync
