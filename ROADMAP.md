# Pinpoint — Roadmap

Living document. PRs welcome; see `CONTRIBUTING.md`.

## Shipped (v0.2.0, 2026-11)
- Rust multiplatform client (desktop/Android/iOS/Web) + modular Axum backend
- Consent-based location sharing, MFA, check-ins, emergency, admin
- OSM search/routing/POI, on-device bookmarks, tile packs
- 79-language i18n with auto-detect
- Redundancy: 2 replicas, gateway failover, pg_dump backups
- i18n, Moldova-first defaults, privacy setting

## Next
- In-app tile canvas + downloadable offline packs (Magic-Earth style)
- Real GPS publish (mobile)
- E2E encryption + OS keychain
- Streaming-replica HA by default (Patroni)
- Android APK + WASM CI jobs
- ZAP active scan + final LaTeX report
- Full egui Web UI on wasm (tokio→wasm swap)

## Backlog
- Per-user consent audit export
- Push notifications (opt-in, no FCM — self-hosted)
- GPX import/export for trips