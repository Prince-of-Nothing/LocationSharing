# AGENTS.md

## Repo shape

Two backend implementations of the same API, per the root `README.md`:

- **Go API** (default Docker stack): `backend/cmd/server` + `backend/internal/`.
- **Python/FastAPI API** (alternate): `backend/app`, migrations in `backend/alembic/`, tests in `backend/tests/`.
- Web clients: `web/` (Go stack, nginx) and `backend/web/` (Python-served).
  - `web/public/js/` is modular vanilla ES modules (no npm/bundler — keep it that way; every new file goes in the `sw.js` SHELL list and gets `node --input-type=module --check` in CI). Tile source is one line in `js/config.js`; never `tile.openstreetmap.org` (OSMF policy) and keep the ODbL attribution visible. CSP in `web/nginx.conf` must allow-list the tile host in `img-src`. Licensing: `web/THIRD_PARTY_NOTICES.md`.
  - `backend/web/` (Python stack) is a separate Leaflet-based client with its own tests (`backend/tests/test_web_client.py`, incl. a tile-policy regression test) — mirror tile/compliance changes there deliberately.
- Android app: `mobile/` — Kotlin/Jetpack Compose, Clean Architecture modules (`app` composition root + `core:{domain,network,data,map}` + `feature:{auth,map,friends,sharing}`), MapLibre maps, 0 trackers. See `mobile/README.md`.
- **Rust desktop client** (root crate `pinpoint-desktop`): Rust is the long-term primary direction; migration tracked in `docs/task1/november-project-progress-and-plan.md`. eframe/egui UI + `reqwest` client matching the Go/Python API contract — session cookie auth with `X-CSRF-Token` double-submit on unsafe methods, and plain HTTP allowed **only** for loopback URLs (fail closed, ADR-0006). `tests/contract.rs` pins the wire contract; the `rust` CI job runs fmt/clippy/build/test/audit.
- Docs in `docs/` (arc42 + Diátaxis + ADRs); full-stack demo via `docker compose`.

Contribution rules live in root `CONTRIBUTING.md` (Go) and `docs/CONTRIBUTING.md` (docs): small PRs, Conventional Commits, docs in the same PR, security changes need a regression test *and* a fail-closed test.

## Commands

Go (run from `backend/`; must pass before pushing):

```bash
go build ./... && go vet ./...   # go test ./...   # gofmt -l .  (must print nothing)
go run ./cmd/server              # port 8000 (PORT to override)
```

Python (run from `backend/`):

```bash
pip install -r requirements-dev.txt
pytest                                   # addopts from pyproject: -q, testpaths=tests
pytest --cov=app --cov-report=term-missing   # what CI runs
ruff check . && ruff format --check .    # line-length 100, py311
bandit -q -c pyproject.toml -r app
pip-audit -r requirements.txt
alembic upgrade head && alembic check   # needs DATABASE_URL (PostgreSQL)
```

Full stack: `docker compose up --build` → http://localhost:8080 (seeds demo accounts; local-only credentials). Python API on :8001 via `docker compose --profile python up --build python-api`.

Android (from `mobile/`; needs JDK 17 + Android SDK 35. CI's `mobile` job compiles every push; a local build also works once those are installed — set `JAVA_HOME`/`ANDROID_HOME` and run from `mobile/`):

```bash
gradle :app:assembleDebug          # debug APK; points at http://10.0.2.2:8000/api/v1/
gradle :core:domain:test :core:network:test   # pure-JVM module tests
```

Rust (from repo root; the `rust` CI job runs the same four gates):

```bash
cargo fmt --all -- --check                 # formatting
cargo clippy --all-targets -- -D warnings  # lints, warnings are errors
cargo build --locked && cargo test --locked # build + unit/contract tests
```

CI (`.github/workflows/ci.yml`) runs all of the above plus migrations against a Postgres 16 service; `DATABASE_URL=postgresql+psycopg://postgres:postgres@localhost:5432/pinpoint`. The `backend-go` job sets `TEST_DATABASE_URL` so the store conformance suite (T01) runs against real Postgres.

## Gotchas

- Known pre-existing flake: `tests/test_auth.py::test_session_limit_revokes_oldest` passes alone but fails in a full local run (verified on pristine `origin/main`). Don't chase it as your regression.
- `backend/dump.rdb` is a stray committed Redis dump — leave it unless asked.
- Security is thesis-mandated: server-side enforcement only, no secrets or real user data anywhere including tests (root `CONTRIBUTING.md`).
- Password/session/token logic differs between the Go and Python implementations — mirror changes deliberately, and check which stack a doc describes before editing.
- Mobile module rules (enforced by Gradle deps): `:feature:*` sees `:core:domain` only; Retrofit/OkHttp stay in `:core:network`; MapLibre stays in `:core:map` (only `MapLibreIntegration.kt` + `OfflineMapManager.kt` import `org.maplibre.*`). New dependencies go in `mobile/gradle/libs.versions.toml`.
- MapLibre Android coordinates are `org.maplibre.gl:android-sdk` on **Maven Central** — `org.maplibre:maplibre-android` does not exist anywhere and breaks dependency resolution.
- Mobile privacy baseline is thesis-relevant: no Firebase/FCM/analytics/crash SDKs, ever. Any new third-party lib must be added to `mobile/THIRD_PARTY_NOTICES.md` in the same PR. OSM attribution must stay visible on the map screen (ODbL).

## Commits

Every commit is co-authored by the human team members (not bots). Add trailers on each commit:

```
Co-authored-by: Mordret <blacksnake@noreply.codeberg.org>
Co-authored-by: Andrei Cliucovschi <cliucovschiandrei@gmail.com>
Co-authored-by: Pleșu Dinu <dinu.plesu@isa.utm.md>
Co-authored-by: Nedelea Dragoș <63261775+dragos-nedelea@users.noreply.github.com>
Co-authored-by: dana-r19 <200202105+dana-r19@users.noreply.github.com>
Co-authored-by: Tcacenco Nichita <nichita.tcacenco@isa.utm.md>
```

Exclude bots: `qwen.ai[bot]`, copilot, dependabot. Local git author is already Mordret.

## Docs conventions

- Diagrams are Mermaid text, never binary images; keep the table in `docs/architecture/diagrams/README.md` in sync with files that exist.
- ADRs are immutable once accepted, named `NNNN-kebab-title.md`, indexed in `docs/adr/README.md`; architecture-significant changes need an ADR in the same PR.

## Ported artifacts (not yet integrated)

These came from a separate Python-hardening port and are **not imported** by the current `backend/app` — don't assume they're wired in, and beware near-duplicate module names (`services/rate_limit.py` vs main's `services/ratelimit.py`, `token_service.py` vs `tokens.py`):

- `backend/app/services/{auth_service,authorization,password_policy,rate_limit,token_service,audit}.py`, `backend/app/repositories/`, `backend/app/middleware/`
- `docs/reference/design-patterns.md`, `docs/architecture/diagrams/` (9 C4/UML files), `docs/adr/0003-structure-backend-around-design-patterns.md`, root `AGENTS.md` (this file)
