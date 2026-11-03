# Pinpoint

**A privacy-first location-sharing platform for trusted friends.**

Friendship does not grant access to anyone's location: each person explicitly chooses who can see them, at what precision, and for how long. Every share can be revoked, and recipients can see whether a location is fresh or stale.

This repository contains two backend implementations sharing the same API:

- **Go API** (default Docker stack): `backend/cmd/server` + `backend/internal/`
- **Python/FastAPI API** (alternate): `backend/app`, migrations in `backend/alembic/`, tests in `backend/tests/`

Web clients: `web/` (Go stack, nginx) and `backend/web/` (Python-served). Android app: `mobile/` — Kotlin/Jetpack Compose, Clean Architecture modules (`app` composition root + `core:{domain,network,data,map}` + `feature:{auth,map,friends,sharing}`), MapLibre maps, 0 trackers.

## Features

- Private-by-default accounts, sessions, and password management
- Friend requests, removal, and blocking
- Directional location sharing with precision, purpose, expiry, and revocation
- Live, stale, and expired location states
- Safety check-ins, admin account management, and audit logging
- Browser client with an OpenStreetMap map (MapLibre Native, zero telemetry, ODbL attribution visible)

## Quick Start

### Go API (default Docker stack)

```bash
cd backend
go build ./... && go vet ./...
DB_DRIVER=memory SEED_DEMO_DATA=true WEB_ROOT=...web/public go run ./cmd/server   # port 8000
```

### Python/FastAPI API (alternate)

```bash
cd backend
pip install -r requirements-dev.txt
pytest                                   # addopts: -q, testpaths=tests
alembic upgrade head                     # needs DATABASE_URL (PostgreSQL)
```

Full stack: `docker compose up --build` → http://localhost:8080

## Contribution Rules

- Small PRs, Conventional Commits, docs in the same PR
- Security changes need a regression test *and* a fail-closed test
- Every commit co-authored by the human team members (see `CONTRIBUTING.md`)

## License

MIT © 2025 Pinpoint