# 13 — Maintenance and Sustainability

## Change management

- Small PRs (Conventional Commits), docs in the same PR.
- Architecture-significant changes: ADR in the same PR (`docs/adr/NNNN-…`).
- Security changes: regression test **and** fail-closed test.
- New deps: license + ToS review + `THIRD_PARTY_NOTICES.md` update.
- Map changes: update `config.js` + CSP + notices + SDLC 07 together.

## Versioning

- Backend `Version` constant (`handlers`); web `SW_VERSION`; mobile
  `versionCode`; Rust `Cargo.toml` version. CHANGELOG entry per release.

## Tech debt register (known)

| Debt | Impact | Plan |
|------|--------|------|
| Public OSM demo endpoints | Rate-limit / ToS risk at scale | Self-host per 07/12 |
| Web Slippy is custom (not Leaflet/MapLibre) | Ongoing ownership | Keep zero-dep; revisit if 3D needed |
| Mobile offline routing is stub only | Offline nav relies on hand-off | GraphHopper/Valhalla plugin via `RoutingEngine` |
| Python/Go duplication | Drift risk | Mirror deliberately; contract CI |
| In-memory store default | Demo-only persistence | Postgres path (`T01`) for prod |

## Support

- Issues: GitHub Issues; security: see `docs/SECURITY.md` (private disclosure).
- Support contacts: `docs/SUPPORT_CONTACTS.md`.
- Long-term: tile/OSM hosting costs (donate/self-host), CI runner costs,
  store review cycles.

## End-of-life

- No friend data to migrate on shutdown: latest-only locations die with the
  store; bookmarks die with the device; offline packs are user downloads.
- Export: bookmarks GeoJSON + account data export (roadmap) for user exit.
