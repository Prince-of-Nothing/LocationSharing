# 12 — Deployment and Operations

## Environments

| Env | Shape | Commands |
|-----|-------|----------|
| Local dev (Go) | `backend/cmd/server` :8000 + static `web/public` | `DB_DRIVER=memory SEED_DEMO_DATA=true WEB_ROOT=../web/public go run ./cmd/server` |
| Full demo (Compose) | nginx :8080 → Go :8000 + Postgres + Redis | `docker compose up --build` → http://localhost:8080 |
| Python alt | FastAPI :8001 + `backend/web/` client | `pip install -r requirements-dev.txt && pytest && alembic upgrade head` |
| Android debug | emulator → `http://10.0.2.2:8000/api/v1/` | `gradle -p mobile :app:assembleDebug` |
| Rust desktop | `pinpoint-desktop` binary | `cargo run` / `cargo build --release` |

Demo seeds: `alice@example.com / DemoPass123!`, `bob@example.com /
DemoPass456!`, `admin@example.com / ChangeMe123!` (local-only credentials).

## Production checklist (OSM + privacy)

- [ ] Self-host tiles (Martin/TileServer GL over Geofabrik extracts) +
      update `config.js`/`MapConfig` + CSP + notices.
- [ ] Self-host Nominatim / OSRM / Overpass **or** proxy with caching +
      monitoring; demo endpoints replaced before real traffic.
- [ ] Postgres (migrations, backups, least-privilege role) + Redis
      (rate limits/sessions) + structured logging + request IDs.
- [ ] HTTPS everywhere (Caddy/nginx reverse proxy), secure cookie flags,
      CORS locked to own origin.
- [ ] `alle: scripts/qa.sh` green; `ci.yml` all jobs green; offline banner
      + OSM outage playbook rehearsed.
- [ ] Data export/delete (GDPR) + audit log retention reviewed.
- [ ] Store listings: OSM attribution + open-source notices complete.

## CI/CD

- `.github/workflows/ci.yml`: `backend`, `backend-go`, `web`, `mobile`,
  `rust`, `release` (Windows zip).
- `.github/workflows/map-qa.yml`: runs `scripts/qa.sh` on every push/PR
  (fast, no network calls to OSM services).

## Rollback

- Web/mobile are static/APK artifacts: revert commit → redeploy.
- Backend: image tag rollback; Postgres migrations need explicit down-path
  (Alembic `alembic downgrade` in dev; `goose`/`golang-migrate` in prod).
- OSM provider outage: switch layer/config to self-host; clients keep
  working with cached packs (mobile) + shell (web).

## Monitoring

- Health: `GET /health` → `{"status":"healthy","version":"…"}`.
- Logs: request ID, authz failures, rate-limit hits, OSM service 429/5xx.
- Client: offline banner frequency, search/route error rate (local logs
  only — no telemetry).
- Alerts: login 429 spikes, `/locations/friends` latency, tile 5xx.
