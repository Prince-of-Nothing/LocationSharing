# Pinpoint — Component Documentation (all architectures)

This directory documents **every component/architecture that has existed in
Pinpoint**, current and archived, with equal depth — redundancy is intentional
so each component can be read standalone.

## The five architectures

| # | Component | Role | Status | Primary dir |
|---|-----------|------|--------|-------------|
| 1 | **Go backend** | HTTP API (`/api/v1`) | 🗄️ Archived (ADR-0007) | `archive/backend/` (Go) |
| 2 | **Python/FastAPI backend** | HTTP API (`/api/v1`) | 🗄️ Archived (ADR-0007) | `archive/backend/app/` |
| 3 | **Rust/Axum backend** | HTTP API (`/api/v1`) + map proxy | ✅ **Current** | `src/server/` |
| 4 | **Rust multiplatform client** | Desktop / Android / iOS / Web GUI | ✅ **Current** | `src/` |
| 5 | **Kotlin Android app** | Android GUI (Jetpack Compose) | ✅ Current (M1) | `mobile/` |
| 6 | *(historical)* **Vanilla-JS web client** | Web PWA | 🗄️ Archived | `archive/web/` |

> The **live contract** is `src/api.rs` + `tests/contract.rs` (Rust client ↔
> Rust/Axum server). Components 1, 2 and 6 are historical: read them for design
> intent only, never as a live dependency (see `archive/README.md`).

## Documents

| Doc | Component |
|-----|-----------|
| [01-go-backend.md](01-go-backend.md) | Go `cmd/server` + `internal/` |
| [02-python-backend.md](02-python-backend.md) | FastAPI `app/` + `alembic/` |
| [03-rust-backend.md](03-rust-backend.md) | Axum `src/server/` |
| [04-rust-client.md](04-rust-client.md) | `src/` multiplatform client |
| [05-kotlin-android.md](05-kotlin-android.md) | `mobile/` Jetpack Compose |
| [06-archived-web-client.md](06-archived-web-client.md) | `archive/web/` vanilla JS |
| [07-competitive-analysis.md](07-competitive-analysis.md) | Similar-app analysis (Google Maps LS, Find My, Life360, Glympse, WhatsApp/Telegram, Strava, Organic Maps, OsmAnd) + prioritised recommendations |

## How the components relate

```
                    ┌──────────────────────────────┐
                    │  Clients (component 4, 5, 6) │
                    │  Rust client · Kotlin · Web  │
                    └──────────────┬───────────────┘
                                   │  HTTPS /api/v1  (session cookie + X-CSRF-Token)
                                   ▼
                    ┌──────────────────────────────┐
                    │  Backends (component 1, 2, 3) │
                    │  Go · Python · Rust/Axum      │
                    └──────────────┬───────────────┘
                                   │  SQL / repository
                                   ▼
                    ┌──────────────────────────────┐
                    │  Storage: Postgres / memory   │
                    └──────────────────────────────┘
```

- **Only one backend is live at a time.** Today that is **component 3
  (Rust/Axum)**; the Go and Python services were the pre-Rust implementations
  and are kept for reference.
- **Two GUI families are live:** the **Rust client** (component 4, desktop +
  mobile + web stubs) and the **Kotlin Android app** (component 5). Both talk to
  the same `/api/v1` contract.
- The **archived web client** (component 6) was the original PWA; it is
  superseded by the Rust web stubs in `web/`.

## Shared contract (all backends implement it)

| Area | Contract |
|------|----------|
| Auth | `POST /api/v1/auth/login` → `{access_token, csrf_token, user}` + session cookie |
| Session | cookie + `X-CSRF-Token` double-submit on state-changing requests |
| Users | `GET/POST /api/v1/users/`, `GET/PATCH/DELETE /api/v1/users/:id`, `GET /api/v1/users/search` |
| Friendships | `POST/GET /api/v1/friendships/`, `POST /api/v1/friendships/:id/respond`, `DELETE /api/v1/friendships/:id` |
| Shares | `GET/POST /api/v1/locations/shares`, `PATCH/DELETE /api/v1/locations/shares/:id` |
| Locations | `POST/GET /api/v1/locations/me`, `GET /api/v1/locations/friends` |
| Check-ins | `GET/POST /api/v1/locations/check-ins`, `DELETE /api/v1/locations/check-ins/:id` |
| Emergency | `GET/POST /api/v1/locations/emergency`, `PATCH /api/v1/locations/emergency/:id` |
| MFA | `POST /api/v1/auth/mfa/enroll`, `/verify`, `GET /status`, `DELETE /api/v1/auth/mfa` |
| Health | `GET /health`, `GET /api/v1/sync-status` |
| Map | `GET /map/tiles/:z/:x/:y`, `GET /map/attribution`, `GET /map/health` |

## Version snapshot

| Component | Language / runtime | Framework | Storage |
|---|---|---|---|
| Go backend | Go 1.25 | chi v5, golang-jwt/v5, pgx/v5 | memory / Postgres |
| Python backend | Python ≥3.11 | FastAPI, SQLAlchemy, Alembic | Postgres |
| Rust backend | Rust 1.99 | Axum 0.7, sqlx 0.8, tokio | memory / Postgres |
| Rust client | Rust 1.99 | eframe/egui 0.32, reqwest 0.12 | on-device JSON / keychain |
| Kotlin app | Kotlin 2.0.21 / JDK 17 | Compose BOM 2024.10, MapLibre 11.5.1 | DataStore / app-private |

## Competitive context

See [07-competitive-analysis.md](07-competitive-analysis.md) for the full
similar-app analysis and the prioritized recommendations it implies. Headline
positioning:

- **Pinpoint is the only app in the set with directional, per-recipient consent
  (direction + precision + expiry) and E2E-encrypted locations in an OS
  keychain.**
- **0 trackers / 0 ads / OSM-only** matches Organic Maps/OsmAnd, unlike
  Life360/Google.
- **Gaps to close:** real-time streaming, Glympse-style temporary share links,
  offline routing, battery-aware sampling.
