# Pinpoint — Docker Deployment Guide (Pure Rust, Modular)

Everything runs in Docker: 4 Rust services + Postgres + nginx gateway +
desktop client image. The Go backend is archived under `archive/`; the
server code lives in `src/server/` and builds as four binaries
(ADR-0008):

- `pinpoint-auth` (`src/bin/auth.rs`)
- `pinpoint-users` (`src/bin/users.rs`)
- `pinpoint-locations` (`src/bin/locations.rs`)
- `pinpoint-map` (`src/bin/map.rs`)
- `pinpoint-server` (`src/bin/server.rs`) — combined router for local dev

## Services (docker-compose.yml)

| Service     | Image                        | Ports                  | Role |
|-------------|------------------------------|------------------------|------|
| `db`        | `postgres:16-alpine`         | `5432` (published)     | Postgres persistence (`pinpoint_state` JSONB snapshot, volume `pinpoint-db-data`) |
| `auth`      | `Dockerfile.service` (`BIN=pinpoint-auth`) | `8001` (published) | Rust Axum auth (`/api/v1/auth/*`) |
| `users`     | `Dockerfile.service` (`BIN=pinpoint-users`) | `8002` (published) | Rust Axum users + friendships (`/api/v1/users/*`, `/api/v1/friendships/*`) |
| `locations` | `Dockerfile.service` (`BIN=pinpoint-locations`) | `8003` (published) | Rust Axum locations + shares (`/api/v1/locations/*`) |
| `map`       | `Dockerfile.service` (`BIN=pinpoint-map`) | `8004` (published) | Rust Axum map (`/map/*`, OSM ODbL attribution) |
| `gateway`   | `nginx:1.27-alpine`          | `8000` → 80 (published) | Reverse proxy by path (docker/nginx-gateway.conf); single client entrypoint |
| `client`    | `Dockerfile.client` runtime  | — (profile `client`)   | Desktop GUI (Linux X11 host only) |

Scale any service independently (e.g. `docker compose up --scale locations=3`).
Direct ports 8001–8004 are for debugging; clients use the gateway on 8000.

## Quick start

```powershell
docker compose up --build -d
docker compose ps
Invoke-RestMethod http://127.0.0.1:8000/health   # -> {"status":"healthy","gateway":"pinpoint-gateway"}
Invoke-RestMethod http://127.0.0.1:8000/map/health
docker logs pinpoint-auth
docker exec pinpoint-db pg_isready -U pinpoint -d pinpoint
```

Then run the desktop client natively:

```powershell
cargo run -- --backend http://localhost:8000
```

Demo login (any password in demo mode for seeded users):

```powershell
Invoke-RestMethod http://127.0.0.1:8000/api/v1/auth/login -Method Post `
  -Body '{"email":"alice@example.com","password":"DemoPass123!"}' `
  -ContentType 'application/json'
```

Verify persistence:

```powershell
docker exec pinpoint-db psql -U pinpoint -d pinpoint -c "\dt"
docker exec pinpoint-db psql -U pinpoint -d pinpoint -c "SELECT count(*) FROM pinpoint_state;"
```

## How persistence works

`AppState::from_env()` (src/server/state.rs):
- If `DATABASE_URL` is empty → in-memory demo mode.
- If set → connects via `sqlx` (PgPool), creates `pinpoint_state(id, data JSONB)`,
  loads the snapshot or seeds demo data, and a background task re-saves every
  5 seconds. Data survives container restarts via the `pinpoint-db-data` volume.

Production hardening: normalize the JSONB snapshot into typed tables,
add bcrypt password hashing, and move secrets to a vault.

## Client in Docker (Linux X11 host only)

```bash
docker compose --profile client run --rm client
```

Needs X11 forwarding. On Windows/macOS Docker Desktop, prefer the native
`cargo run` for the GUI and Docker for `db` + `api` only.

## Full CI gate in a container

```bash
docker build -f Dockerfile.client --target tester -t pinpoint-desktop:tester .
docker run --rm pinpoint-desktop:tester
```

## Troubleshooting

| Symptom | Fix |
|---|---|
| `api` crash-loops, `Postgres unavailable` warning | Check `docker compose logs db`; ensure `DATABASE_URL` matches `POSTGRES_*` |
| Port 8000 already in use | Stop stray `pinpoint-server` processes or change published port |
| API not healthy | `docker compose logs api`; `docker compose ps` for health status |
| Login returns 401 | Use seeded demo email; any password is accepted in demo mode |
