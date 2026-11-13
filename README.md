# Pinpoint

**A privacy-first location-sharing platform for trusted friends.**

Friendship does not grant access to anyone's location: each person explicitly
chooses who can see them, at what precision, and for how long. Every share can
be revoked, and recipients can see whether a location is fresh or stale.

**Current release: [1.1.0](./RELEASE_NOTES.md)** — thesis demo, not
production-hardened. See the functional-state matrix and per-item
`[FEATURE]` / `[FIX]` flags in the release notes.

---

## What works today

- **Accounts & sessions** — email/password (Argon2id) and anonymous
  device-bound accounts, opaque `HttpOnly` session cookies + CSRF, session
  list/revoke, logout-all, lockout, TOTP MFA.
- **Web UI at `/`** — map-first Leaflet UI (locally vendored), place search,
  A→B routing with alternatives/steps (drive; walk/bike when self-hosted),
  saved/recent places per account, locate-me, plus the 1.0 auth, publish,
  friends, shares and sessions surfaces in a drawer.
- **Desktop client** — map, Nominatim search, OSRM routing (drive/walk/bike),
  Overpass POIs, bookmarks, check-ins, emergency, offline tile packs.
- **Consent model** — directional, precision-scoped (`exact` / `approximate` /
  `city_only`), purpose-bound, expiring, revocable; enforced server-side.
- **Privacy** — no trackers, no analytics, no raw IP storage, no IP/WebRTC
  geolocation; local-only web assets (no Google Fonts/CDNs); ODbL attribution
  everywhere.
- **OSM self-hosting** — `PINPOINT_TILE_URL`, `PINPOINT_NOMINATIM_URL`,
  `PINPOINT_OSRM_URL`, `PINPOINT_OVERPASS_URL`; public providers are demo
  fallbacks only.

**Not in 1.0.0:** Alice/Bob as client containers (still seeded), Postgres
write-through for users/locations, OAuth/passkeys/magic links, Android/iOS
builds. (Web search + A→B routing landed after the 1.0.0 tag — see
`CHANGELOG.md`.) See [`ROADMAP.md`](./ROADMAP.md) and
[`RELEASE_NOTES.md`](./RELEASE_NOTES.md).

---

## Quick start

### Web (single binary)

```bash
cargo run --bin pinpoint-server
# → http://127.0.0.1:8000/  (in-memory demo store, seeded Alice/Bob)
# demo login: alice@example.com / DemoPass123!
```

If port 8000 is taken (Docker), use `BIND_ADDR=127.0.0.1:8080`.

### Desktop

```bash
cargo run -- --backend http://127.0.0.1:8000
```

### Docker

```bash
docker compose up --build -d
# gateway → http://127.0.0.1:8000/
```

### Release artifact

Unzip `dist/pinpoint-1.0.0-windows-x64.zip`, run `pinpoint-server.exe`, open
`http://127.0.0.1:8080/`.

---

## Architecture

One Rust crate (`pinpoint-desktop`): portable core in `src/lib.rs`, desktop
entry in `src/main.rs`, Axum API in `src/server/` (auth/users/locations/map
routers behind a gateway), OSM clients in `src/osm.rs`, platform code in
`src/platform.rs`. Desktop, Android, iOS and a WASM stub share the core.

- [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) — current module map
- [`docs/adr/`](./docs/adr/) — decisions (ADR-0007…0012)
- [`docs/SECURITY.md`](./docs/SECURITY.md) — security model and gaps
- [`docs/PRIVACY.md`](./docs/PRIVACY.md) — GDPR posture
- [`docs/reference/api-contract.md`](./docs/reference/api-contract.md) — live API

---

## Quality gates

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo build --locked && cargo test --locked
```

CI adds `cargo-audit`, map QA, and the flagged release artifact job.

---

## License

MIT © 2026 Pinpoint. Map data © OpenStreetMap contributors, ODbL.

## Contributing

Small PRs, conventional commits, docs in the same PR. Security changes need a
regression test and a fail-closed test. Commits are authored by the repository
owner only — no co-author trailers (see `CONTRIBUTING.md`).
