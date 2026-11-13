# 02 — Python / FastAPI Backend (archived)

> **Status:** 🗄️ **Archived** (ADR-0007). Not built or shipped. Kept for
> design intent. Live dir: `archive/backend/app/` + `archive/backend/alembic/`.

## Overview

The Python service is a second, parallel implementation of the `/api/v1`
contract, built with FastAPI + SQLAlchemy + Alembic. It was maintained
alongside the Go service so the two could be compared and so the contract could
be validated independently. It targets PostgreSQL and serves the web client
from the same origin.

## Stack

| Concern | Choice |
|---|---|
| Language | Python ≥ 3.11 |
| Framework | FastAPI |
| ORM | SQLAlchemy |
| Migrations | Alembic |
| Schemas | Pydantic |
| DB | PostgreSQL |
| Lint/test | `ruff` (line-length 100, `target-version py311`), `pytest`, `bandit` |
| Packaging | `pyproject.toml` + `requirements*.txt` |

## Module map

```
archive/backend/
├── app/
│   ├── main.py            # FastAPI factory: middleware, routers, static web
│   ├── cli.py             # admin CLI (create/seed users)
│   ├── api/               # route modules
│   │   ├── auth.py        # login/register/logout/sessions/password
│   │   ├── users.py       # /me, user lookup
│   │   ├── friendships.py # request/list/accept/decline/delete
│   │   ├── blocks.py      # block list
│   │   ├── location_shares.py  # directional shares CRUD
│   │   ├── locations.py   # publish own / visible locations
│   │   ├── checkins.py    # check-ins + shared check-ins
│   │   ├── admin.py       # user unlock/deactivate/activate, audit events
│   │   ├── deps.py        # auth/admin dependencies
│   │   └── errors.py      # error handlers
│   ├── core/              # config, errors, middleware, timeutils
│   ├── db/                # SQLAlchemy base + session
│   ├── middleware/        # security headers
│   ├── models/            # ORM models: user, auth_session, social, location,
│   │                      #   checkin, audit (+ _mixins)
│   ├── repositories/      # people + locations repositories
│   ├── schemas/           # Pydantic request/response schemas
│   └── services/          # auth, sessions, tokens, passwords, rate limit,
│                          #   authorization, relationships, audit, security
├── alembic/               # migrations (versions/)
├── tests/                 # pytest suite
├── Dockerfile.python
└── requirements*.txt
```

## API surface

Mounted under `/api/v1` (see `app/main.py`).

| Area | Routes |
|---|---|
| Auth | `POST /auth/login`, `POST /auth/register`, `POST /auth/logout`, `POST /auth/logout-all`, `GET /auth/sessions`, `DELETE /auth/sessions/{id}`, `POST /auth/password/change`, `POST /auth/password/force-change`, `GET /auth/me` |
| Users | `GET /users/me`, `PATCH /users/me`, `GET /users/{id}`, `GET /users/` |
| Friendships | `GET /friendships`, `POST /friendships`, `POST /friendships/{id}/accept`, `POST /friendships/{id}/decline`, `DELETE /friendships/{id}` |
| Blocks | `GET /blocks`, `POST /blocks`, `DELETE /blocks/{username}` |
| Location shares | `GET /location-shares`, `POST /location-shares`, `GET/PATCH/DELETE /location-shares/{id}` |
| Locations | `PUT /locations` (publish), `GET /locations/me`, `DELETE /locations/me`, `GET /locations/visible` |
| Check-ins | `GET /check-ins`, `POST /check-ins`, `POST /check-ins/{id}/complete`, `POST /check-ins/{id}/cancel`, `GET /check-ins/shared` |
| Admin | `GET /admin/users`, `POST /admin/users/{id}/unlock`, `…/deactivate`, `…/activate`, `GET /admin/audit-events` |
| Health | `GET /api/v1/health` → `{"status":"ok","version":…}` |

## Data model

SQLAlchemy models in `app/models/`:

- **User** — id, email, password hash, full name, role, active,
  `must_change_password`.
- **AuthSession** — token, user, expiry, CSRF token, metadata.
- **Friendship / Block** — social graph edges with status.
- **LocationShare** — owner, friendship, direction, precision, active, expiry.
- **Location** — owner, lat/lon, accuracy, timestamp (latest-only).
- **Checkin** — user, lat/lon, note, kind, status, timestamps.
- **AuditEvent** — actor, action, target, outcome, timestamp.

## Security controls

- **Passwords:** hashed in `app/services/passwords.py` (policy enforced by
  `password_policy.py`).
- **Sessions/tokens:** `app/services/sessions.py` + `tokens.py` /
  `token_service.py`.
- **Rate limiting:** `app/services/ratelimit.py` / `rate_limit.py`.
- **Authorization:** `app/services/authorization.py` + `relationships.py`.
- **Audit:** `app/services/audit.py` (append-only audit events).
- **Middleware:** security headers (`app/middleware/security_headers.py`),
  body-size limit, CORS (`app/core/middleware.py`).
- **Errors:** consistent envelope (`app/core/errors.py` + `app/api/errors.py`).

## Storage

- **PostgreSQL** via SQLAlchemy; migrations via **Alembic** (`alembic/versions`).
- **Redis** optional for rate limiting (see `archive/docker-compose.yml`).

## Build / run / test

```bash
cd archive/backend
pip install -r requirements-dev.txt
alembic upgrade head
pytest
ruff check app
bandit -r app
```

## Deployment

- Container: `archive/backend/Dockerfile.python`.
- Compose: `archive/docker-compose.yml` (Python API :8001 + web + Postgres +
  Redis).
- Static web: served from `archive/backend/web/` (same origin).

## Relationship to other components

- **Superseded by:** component 3 (Rust/Axum backend), which implements the same
  `/api/v1` contract.
- **Consumed by:** the archived web client (component 6).
- **Value:** independent implementation of the same contract — useful as an
  oracle for conformance tests and for recovering design intent (audit,
  authorization, sessions).
