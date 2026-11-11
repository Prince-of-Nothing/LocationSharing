# Pinpoint

**A privacy-first location-sharing platform for trusted friends.**

Friendship does not grant access to anyone's location: each person
explicitly chooses who can see them, at what precision, and for how long.
Every share can be revoked, and recipients can see whether a location is
fresh or stale.

## Map First

Open the app and you're immediately on the map — no login required. The map
works offline with demo locations. Sign in to unlock friends, location
sharing, and admin features.

```
Open app → See map → (optional) Sign in → Unlock friends/sharing
```

---

## Features

- **Map-first UX** — Interactive map opens immediately, works offline
- **OpenStreetMap (ODbL)** — All map data from OSM; attribution always visible
- **Search places** — Nominatim search for cafés, pharmacies, parks, etc.
- **Navigate** — OpenStreetMap directions and OSM map links for any location
- **Consent-based sharing** — Directional shares (to_friend/from_friend/
  mutual) with precision (exact/approximate/city_only) and expiry
- **Honest freshness** — Every location shows age/accuracy; stale states
  are never displayed as live
- **0 trackers** — No analytics, crash reporting, or advertising SDKs
- **Server-side enforcement** — All consent decisions made on the server
- **Multiplatform** — Rust client for desktop (Windows/Linux/macOS),
  Android, and iOS from one codebase

---

## Quick Start

### Terminal 1 — Rust API server (pure Rust, no Go)

```bash
cargo run --bin pinpoint-server
# → http://localhost:8000/ (demo accounts pre-seeded, in-memory store)
```

### Terminal 2 — Desktop client

```bash
cargo run -- --backend http://localhost:8000
# (map opens immediately; sign in optional)
```

### With Docker (backend + Postgres, all Rust)

```bash
# Start Postgres + Rust API server
docker compose up --build -d db api

# Run the desktop client against the containerized API
cargo run -- --backend http://localhost:8000

# Full CI gate inside a container (fmt/clippy/build/test)
docker build -f Dockerfile.client --target tester -t pinpoint-desktop:tester .
docker run --rm pinpoint-desktop:tester
```

### With Podman

```bash
podman build -t pinpoint-desktop:latest -f Containerfile.podman --target runtime .
podman run --rm -e PINPOINT_BACKEND=http://host.containers.internal:8000 \
  pinpoint-desktop:latest
```

---

## Sign In

Click **Sign in** in the header to unlock:

- Publish your location
- Send/accept friend requests
- Create location shares (precision, purpose, expiry)
- Admin panel (administrator role only)
- Change password

Demo accounts: `alice@example.com` / `DemoPass123!` (if backend is seeded).

---

## Documentation

| Document | Description |
|----------|-------------|
| `docs/USER_GUIDE.md` | Complete user guide — map, sign in, share, navigate |
| `docs/SETUP.md` | Installation and setup for all platforms |
| `docs/client-platforms.md` | Per-OS client status: desktop/Linux/macOS/Android/iOS/Web build+verify |
| `docs/TROUBLESHOOTING.md` | Common problems and fixes |
| `docs/FEATURES.md` | Complete feature reference |
| `docs/END_OF_SERVICE.md` | Data export and code reuse after service ends |
| `docs/sdlc/` | Full SDLC documentation set (00–16) |
| `docs/uml/` | Mermaid diagrams (01–15) |

---

## Quality Gates

All four commands must pass before pushing:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo build --locked
cargo test --locked
```

---

## License

MIT © 2026 Pinpoint

---

## Contributing

Small PRs, conventional commits, docs in the same pr. Security changes need
a regression test and a fail-closed test. Commits are authored by the
repository owner only — no co-author trailers (see `CONTRIBUTING.md`).
