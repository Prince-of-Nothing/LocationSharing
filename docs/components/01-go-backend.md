# 01 — Go Backend (archived)

> **Status:** 🗄️ **Archived** (ADR-0007 Rust-multiplatform pivot). Not built or
> shipped. Kept for design intent. Live dir: `archive/backend/` (Go part).

## Overview

The Go service was Pinpoint's first production backend. It exposes the
`/api/v1` HTTP contract, authenticates with JWT + server-side sessions, hashes
passwords with bcrypt, and stores data in an in-memory repository that can be
swapped for PostgreSQL (pgx) without touching handlers.

## Stack

| Concern | Choice |
|---|---|
| Language | Go 1.25 (`go 1.25.0` in `go.mod`) |
| Router | `github.com/go-chi/chi/v5` v5.0.12 |
| Auth tokens | `github.com/golang-jwt/jwt/v5` v5.2.1 |
| DB driver | `github.com/jackc/pgx/v5` v5.11.0 |
| Password hashing | `golang.org/x/crypto` v0.17.0 (bcrypt) |
| Config | env vars (`.env.example`) |
| Testing | `go vet ./...`, `go test ./...` |

## Module map

```
archive/backend/
├── cmd/server/            # entry point: graceful shutdown, PORT env
├── internal/
│   ├── config/            # env-driven settings
│   ├── models/            # User, Session, Friendship, LocationShare, ...
│   ├── security/          # password hashing/validation, JWT, session+CSRF tokens
│   ├── store/             # repository interface + memory + postgres + seed
│   │   ├── repository.go  # Repository contract
│   │   ├── store.go       # in-memory implementation
│   │   ├── postgres.go    # pgx implementation
│   │   ├── open.go        # DB_DRIVER selection (memory|postgres)
│   │   └── seed.go        # SEED_DEMO_DATA demo accounts
│   └── handlers/          # HTTP layer
│       ├── app.go         # router + middleware + route registration
│       ├── auth.go        # login/register/logout/me/password/session
│       ├── users.go       # user CRUD + search
│       ├── friendships.go # request/list/respond/delete
│       ├── locations.go   # shares + publish + friend locations
│       ├── middleware.go  # requireAuth / requireAdmin / CORS / rate limit
│       ├── respond.go     # JSON + error envelopes
│       └── static.go      # bundled SPA static file server (WEB_ROOT)
└── web/                   # static web client copied at build time
```

## API surface

All routes are mounted under `/api/v1` (see `internal/handlers/app.go`).

| Method | Path | Auth | Notes |
|---|---|---|---|
| GET | `/` | – | service name + version |
| GET | `/health` | – | `{"status":"healthy","version":…}` |
| POST | `/api/v1/auth/login` | – | session cookie + JWT + CSRF token |
| POST | `/api/v1/auth/register` | – | create account |
| POST | `/api/v1/auth/logout` | ✓ | revoke session, clear cookie |
| POST | `/api/v1/auth/session` | ✓ | refresh idle timer, re-issue CSRF |
| GET | `/api/v1/auth/me` | ✓ | current profile |
| POST | `/api/v1/auth/password/change` | ✓ | requires current password; revokes all sessions |
| POST | `/api/v1/auth/password/force-change` | ✓ | for `must_change_password` accounts |
| POST | `/api/v1/users/` | admin | provision user |
| GET | `/api/v1/users/` | ✓ | list users |
| GET | `/api/v1/users/{id}` | ✓ | fetch user |
| PATCH | `/api/v1/users/{id}` | ✓ | update profile |
| POST | `/api/v1/friendships/` | ✓ | send request |
| GET | `/api/v1/friendships/` | ✓ | list friendships |
| POST | `/api/v1/friendships/{id}/respond` | ✓ | accept/reject/block |
| POST | `/api/v1/locations/shares` | ✓ | create directional share |
| GET | `/api/v1/locations/shares` | ✓ | list shares |
| PATCH | `/api/v1/locations/shares/{id}` | ✓ | pause/resume/precision |
| POST | `/api/v1/locations/me` | ✓ | publish own location |
| GET | `/api/v1/locations/friends` | ✓ | visible friend locations |

## Data model

`internal/models/models.go` defines the domain entities:

- **User** — id, email, password hash, full name, role, active flag,
  `must_change_password`.
- **Session** — token, user, expiry, CSRF token, created/last-seen.
- **Friendship** — requester, receiver, status (`pending`/`accepted`/
  `rejected`), timestamps.
- **LocationShare** — owner, friendship, direction (`to_friend`/`from_friend`/
  `mutual`), precision (`exact`/`approximate`/`city_only`), active, expiry,
  purpose/label.
- **Location** — owner, lat/lon, accuracy, timestamp (latest-only).
- **AuditEvent** — actor, action, target, outcome, timestamp.

## Security controls

- **Password hashing:** bcrypt (`golang.org/x/crypto/bcrypt`).
- **Sessions:** server-side; cookie `HttpOnly`, `Secure`, `SameSite=Lax`.
- **CSRF:** double-submit — login returns a CSRF token (JSON body +
  `X-CSRF-Token` header); state-changing requests must echo it.
- **Rate limiting:** fixed window per IP; account lockout after N failed logins
  with automatic unlock after `LOCKOUT_DURATION_MINUTES`.
- **Authorization:** `requireAuth` / `requireAdmin` middleware; directional
  share checks on every location read/write.
- **Transport:** TLS terminated by nginx/Caddy in deployment.

## Storage

- **Default:** thread-safe in-memory store (`internal/store/store.go`).
- **Production:** PostgreSQL via pgx (`internal/store/postgres.go`), selected
  with `DB_DRIVER=postgres` + `DATABASE_URL`; graceful fallback to memory.
- **Demo data:** `SEED_DEMO_DATA=true` seeds admin/alice/bob.

## Build / run / test

```bash
cd archive/backend
go run ./cmd/server          # listens on :8000 (override with PORT)
curl localhost:8000/health
go vet ./...
go test ./...
```

## Deployment

- Container: `archive/backend/Dockerfile` (Go) / `Dockerfile.python`.
- Compose: `archive/docker-compose.yml` (Go API + Python API + web + Postgres +
  Redis + nginx gateway).
- Cloud: `archive/render.yaml`.

## Relationship to other components

- **Superseded by:** component 3 (Rust/Axum backend), which mirrors its
  handlers and `/api/v1` contract exactly.
- **Consumed by:** the archived web client (component 6) and the archived
  Kotlin app (component 5's predecessor).
- **Design intent reused:** directional shares, precision levels, freshness,
  CSRF double-submit — all preserved in the Rust backend.
