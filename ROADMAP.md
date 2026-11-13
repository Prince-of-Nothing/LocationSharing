# Pinpoint — Roadmap

Living document. PRs welcome; see `CONTRIBUTING.md`.

## Shipped (v0.2.0, 2026-11)
- Rust multiplatform client (desktop/Android/iOS/Web) + modular Axum backend
- Consent-based location sharing, MFA, check-ins, emergency, admin
- OSM search/routing/POI, on-device bookmarks, tile packs
- 79-language i18n with auto-detect
- Redundancy: 2 replicas, gateway failover, pg_dump backups
- i18n, Moldova-first defaults, privacy setting
- Interactive Slippy tile canvas (`src/map.rs`) + downloadable multi-source ODbL offline packs (`src/tilepacks.rs`) + GeoJSON bookmark export/import
- Dual mobile apps in tree: Kotlin/Jetpack Compose Android (`mobile/`) + Rust Android/iOS (`android/`, `ios/`, `src/mobile` with `geo:`/`pinpoint://` deep links & `GpsProvider`)
- E2E location encryption (`src/security/e2e.rs`) + OS keychain persistence (`src/security/keychain.rs`, wired on login/register/publish)

## Next
- Streaming-replica HA by default (Patroni)
- Android APK + WASM CI jobs (requires NDK / Trunk in CI)
- ZAP active scan + final LaTeX report
- Full egui Web UI on wasm (tokio→wasm swap)

## Backlog
- Per-user consent audit export
- Push notifications (opt-in, no FCM — self-hosted)
- GPX import/export for trips