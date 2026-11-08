# 06 — Technical Requirements

## TR-01 — Web client (`web/public/`)

- Vanilla ES2022 modules, **no npm/bundler/CDN JS** (keep it that way).
- Modules: `config` (tile layers + OSM endpoints), `api` (only fetcher),
  `session`, `ui` (esc/toast), `map` (Slippy engine + overlays),
  `geocode` (Nominatim), `routing` (OSRM + polyline), `places` (Overpass),
  `bookmarks` (localStorage), `navigation` (`geo:`/GPX), `main` (wiring).
- Every new JS file is added to `sw.js` SHELL; bump `SW_VERSION`.
- Tile URL stays one line per layer in `config.js`; CSP `img-src` updated
  in the same PR (see `web/nginx.conf`).
- All server/HTML sinks go through `esc()`; no `innerHTML` with untrusted
  data without escaping.
- Keyboard/touch/pointer map input; ARIA labels; no third-party fonts.

## TR-02 — Android app (`mobile/`)

- Kotlin + Jetpack Compose, Clean Architecture:
  `app` (composition root) / `core:{domain,network,data,map}` /
  `feature:{auth,map,friends,sharing}`.
- Module rules (Gradle-enforced): `:feature:*` → `:core:domain` (+ `:core:map`
  public API for map feature only); Retrofit/OkHttp in `:core:network`
  only; MapLibre in `:core:map` only (`MapLibreIntegration.kt` +
  `OfflineMapManager.kt` import `org.maplibre.*`).
- New OSM clients live in `:core:network` (`OsmServices.kt`: Nominatim,
  OSRM, Overpass + DTOs + `OsrmRoutingEngine`); domain models in
  `:core:domain` `place/Places.kt`; repos in `:core:data`
  (`OsmRepositories.kt`); styles in `:core:map` (`MapLayers.kt`);
  screens in `:feature:map` (`SearchScreen`, `RoutePlannerScreen`,
  `PlacesScreen`).
- DI is manual in `app/di/Container.kt` (no Hilt/KSP). `RoamMapConfig`
  stays data-only; `MapConfig.DEFAULT` remains the privacy default.
- 0 trackers: no Firebase/FCM/analytics; `network_security_config.xml`
  blocks cleartext except dev hosts; tokens in Keystore AES-GCM.

## TR-03 — Rust desktop (`src/`, root crate `pinpoint-desktop`)

- eframe/egui + `reqwest`; session-cookie auth + `X-CSRF-Token` double-submit;
  plain HTTP only for loopback (fail closed, ADR-0006).
- Map-first UX (ADR-0008): `CurrentScreen::Map` is the default landing screen.
  Demo locations, search, and navigation work offline without sign-in.
  Sign-in-required screens (Friends/Sharing/Admin/ChangePassword) show a
  sign-in prompt when unauthenticated. Logout returns to the demo map,
  not to a blank/login-only state.
- Map links are OSM-only defaults (OSM standard, OSM directions, Organic
  Maps); ODbL credit rendered on the map panel. `cargo fmt --check` +
  `cargo clippy -- -D warnings` + `cargo test --locked` green.
- Design patterns (`src/patterns/`): Repository, Strategy, Command, State,
  Observer, Factory, Decorator, Proxy — see `docs/design-patterns.md`.
- Security (`src/security/`): validation, rate limiting, AES-256-GCM
  encryption at rest, audit logging, session timeout, HMAC request signing
  — see `docs/security-architecture.md`.
- `PinpointApp` owns `StateContext`, `RateLimiter`, `AuditLogger`, and
  `KeyManager`; every network action is validated, rate-limited, and
  audit-logged.

## TR-04 — Backend (Go default, Python alternate)

- `/api/v1` contract unchanged: auth, users, friendships, locations
  (shares/publish/friends). Precision downgrade + consent checks server-side.
- Store interface: memory (demo/tests) → Postgres (production, `T01`).
- Errors: generic client messages; details in logs. Rate limits + audit.

## TR-05 — Build & release

- Web: static files only; nginx (compose) or `WEB_ROOT` (single binary).
- Mobile: `gradle :app:assembleDebug` (JDK 17, SDK 35); new deps only via
  `mobile/gradle/libs.versions.toml` + `THIRD_PARTY_NOTICES.md`.
- Rust: `cargo build --locked && cargo test --locked`.
- QA gate: `scripts/qa.sh` (see 11) must pass; CI `map-qa` job mirrors it.

## TR-06 — Compatibility matrix

| Client | Backend | Tiles | Search/Routing/POI | Offline |
|--------|---------|-------|-------------------|---------|
| Web PWA | Go :8000 / Python :8001 (via nginx) | CARTO/Wikimedia raster | Nominatim/OSRM/Overpass (public demo) | Shell + bookmarks |
| Android | same `/api/v1` | OpenFreeMap vector | same OSM services | Packs + hand-off |
| Rust | same `/api/v1` | browser hand-off | browser hand-off | n/a (links) |
