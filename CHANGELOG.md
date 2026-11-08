# Changelog

All notable changes to this project are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
Semantic Versioning. This is a pre-1.0 (thesis MVP) project.

## [Unreleased]
### Added
- OSM map parity (web + Android + Rust desktop): interactive Slippy web map
  with CARTO/Wikimedia layers, Nominatim search/reverse, OSRM routing
  (driving/foot/bike + GPX), Overpass POI browse (9 categories), on-device
  bookmarks (GeoJSON export); Android MapLibre with 3 OpenFreeMap styles,
  offline packs, Nominatim/OSRM/Overpass screens, DataStore bookmarks;
  Rust desktop OSM hand-off links fixed (OSM/directions/Organic Maps + ODbL credit)
- Full SDLC documentation set (`docs/sdlc/00-16`): BRD, FR/NFR/TRS, OSM
  integration guide (ODbL, Nominatim/OSRM/Overpass policies, self-hosting),
  architecture, security/privacy, QA, deployment, risks, traceability
- UML diagrams 12–15: map search/places, routing/navigation, offline/
  bookmarks, web Slippy engine (Mermaid, with FR traceability)
- Automatic QA: `scripts/qa.sh` + `scripts/qa.ps1` (tile policy, attribution,
  CSP sync, SW SHELL, Mermaid, notices) + CI `map-qa` job
- Rust desktop client (`pinpoint-desktop`, root crate): eframe/egui UI with
  stateful login/register/logout/password-change, `reqwest` API client bound to
  the server contract (session cookie + `X-CSRF-Token` double-submit), transport
  policy requiring HTTPS off-loopback (ADR-0006), and unit + API-contract tests
  (`tests/contract.rs`) covering fail-closed login/logout and CSRF placement
- CI `rust` job: `cargo fmt --check`, `clippy -D warnings`, locked build/test,
  and a `cargo-audit` dependency vulnerability audit
- CI `release` job (`windows-latest`): builds the `--release` binary, packages it
  into `pinpoint-desktop.zip`, and uploads it as a GitHub Actions artifact
- `docs/DEPLOYMENT.md` §4 Rust Desktop Deployment: build, run flags, ADR-0006
  transport policy, verified distributable artifact path, and demo credentials
- Kotlin/Jetpack Compose Android app scaffold (`mobile/`) — planned next
- `weba` admin console and `webk` kiosk check-in clients — planned
- Project governance docs: CONTRIBUTING, CODE_OF_CONDUCT, docs/{PROJECT, ROADMAP,
  ARCHITECTURE, SECURITY, PRIVACY, TESTING, DEPLOYMENT, GLOSSARY}, LICENSE template

### Changed
- Rust desktop client: bumped `eframe` 0.30 → 0.32, moving the Linux
  accessibility stack off `quick-xml 0.30.0` (RUSTSEC-2026-0194,
  RUSTSEC-2026-0195, both high) to `0.41.0`; `cargo audit` now exits 0 with
  0 vulnerabilities (5 unmaintained warnings only)

## [0.3.0] - 2026-10-13
### Added
- Docker deployment: `docker-compose.yml`, multi-stage `backend/Dockerfile` (non-root,
  healthcheck), `web/Dockerfile` + `web/nginx.conf` (SPA + `/api` reverse proxy)
- Idempotent demo seeding (`SEED_DEMO_DATA`): admin (forced password change), Alice & Bob
  with mutual shares and Paris locations
- Static file serving from the Go binary when `WEB_ROOT` is set (single-container option)
- `COOKIE_INSECURE` dev flag; `.env.example` documenting all settings
- Web SPA v1: login/CSRF flow, OSM map with friend pins, freshness/stale badges, precision
  badges, navigation export, location publishing, friend requests, directional share
  management, admin provisioning, forced-password-change UX

### Changed
- Mobile client decision: Flutter → **Kotlin (Android-first)** due to no macOS/iPhone
  development hardware; Android emulator covers the full test loop

## [0.2.0]
### Added
- Full Go backend feature parity: JWT + session auth, bcrypt, CSRF double-submit,
  rate limiting, lock/unlock, must-change-password gate
- Authorization + security test suite (`go test ./...` green, incl. IDOR fail-closed cases)

### Removed
- Entire legacy Python/FastAPI backend (refactor complete; no `.py` files remain)

## [0.1.0]
### Added
- Initial Python/FastAPI prototype (superseded by 0.2.0)
- Core domain model: users, friendships, directional share permissions, current location
