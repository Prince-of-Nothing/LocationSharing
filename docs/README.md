# Pinpoint — Documentation Index

Start here. Every doc is Markdown (Mermaid for diagrams). See also the root
[`AGENTS.md`](../AGENTS.md) (build commands + commit rules) and
[`CONTRIBUTING.md`](../CONTRIBUTING.md).

## Quick links

- **Run it:** [`QUICKSTART.md`](../QUICKSTART.md) · [`docs/SETUP.md`](SETUP.md) · [`docs/TROUBLESHOOTING.md`](TROUBLESHOOTING.md)
- **What it does:** [`docs/FEATURE_INVENTORY.md`](FEATURE_INVENTORY.md) · [`docs/BEST_APP_CRITERIA.md`](BEST_APP_CRITERIA.md)
- **Architecture:** [`docs/architecture-data-domains.md`](architecture-data-domains.md) · [`docs/design-principles.md`](design-principles.md) · [`docs/principles-matrix.md`](principles-matrix.md) · [`docs/service-discovery.md`](service-discovery.md) · [`docs/redundancy.md`](redundancy.md) · [`docs/postgres-ha.md`](postgres-ha.md)
- **Clients:** [`docs/client-platforms.md`](client-platforms.md) · [`docs/docker-deployment.md`](docker-deployment.md) · [`docs/docker-podman.md`](docker-podman.md) · [`docs/gap-acceptance.md`](gap-acceptance.md)
- **Security:** `docs/security/` (architecture, threat model, incident response, `dual-auth-model.md`, `session-management.md`, `auth-mechanisms.md`) · `docs/evidence/` (ZAP)
- **Components (all architectures):** [`docs/components/`](components/) — Go · Python · Rust/Axum · Rust client · Kotlin · archived web
- **Database:** [`docs/database/schema.md`](database/schema.md) (normalized, encrypted, modular) · DDL [`db/schema/00-target.sql`](../db/schema/00-target.sql)
- **Maps:** [`docs/maps/osm-feature-parity.md`](maps/osm-feature-parity.md) (Organic Maps/Magic Earth/OsmAnd parity) · [`docs/maps/self-hosted-osm.md`](maps/self-hosted-osm.md) (self-hosting)
- **SDLC:** `docs/sdlc/00–17` (17 = spiral execution plan) · **UML:** `docs/uml/01–15` · **ADRs:** `docs/adr/`

## Module map (for humans and AI agents)

```
src/
  lib.rs                 portable core (modules listed below)
  main.rs                desktop entry (eframe); --help, --backend, PINPOINT_BACKEND
  api.rs                 HTTP client: session cookie + X-CSRF-Token, URL policy (ADR-0006)
  app.rs                 PinpointApp state + screens (depends on DynBackend)
  ui.rs                  egui shell; per-screen modules in src/ui/
  ui/                    auth, map, friends, sharing, places, bookmarks, trips,
                         emergency, profile, admin
  osm.rs                 Nominatim / OSRM / Overpass (friend coords NEVER sent)
  map.rs                 interactive Slippy canvas + pins + OSM hand-off + ODbL credit
  models.rs              serde domain models + pure math (haversine, bearing, freshness)
  platform.rs            all cfg(target_os) branches
  bookmarks.rs           on-device JSON store
  tilepacks.rs           offline tile download (Slippy math)
  i18n.rs                79-language registry + t() + auto-detect
  security/              validation, rate_limit, encryption, audit, session, signing, mfa, password
  patterns/              repository (InMemory + Postgres), strategy, command, state, observer,
                         factory, decorator, proxy
  backend/               DynBackend trait + REST adapter (swappable backend architectures)
  server/                Axum services
    mod.rs               routers + /health + convergence middleware
    state/               mod, store (facade), users, friendships, locations
    auth_handlers.rs     /api/v1/auth/*
    users_handlers.rs    /api/v1/users/*, /api/v1/friendships/*
    locations_handlers.rs /api/v1/locations/*
    migrations.rs        typed idempotent migrations v1..v12
  webapp/                portable web clients (admin, kiosk, shared view-model)
  mobile/                platform adapters (android, ios)
src/bin/                 pinpoint-{desktop,server,auth,users,locations,map,web,weba,webk}
tests/                   contract.rs (10), router_probe.rs (4), live_api.rs (1 ignored)
```

## How to add a feature (additive, never a rewrite)

1. **New table** → add entry in `src/server/migrations.rs`, add load/persist in the owning `src/server/state/*.rs`, add `ServiceKind` ownership if a new service.
2. **New screen** → new `src/ui/<name>.rs` sharing `PinpointAppUI`, add `CurrentScreen` variant, register in `ui.rs` dispatcher.
3. **New service** → new router in `src/server/mod.rs`, new binary in `src/bin/`, gateway location + `/registry` entry in `docker/nginx-gateway*.conf`.
4. **New language** → add a `Lang` variant + a translation block in `src/i18n.rs` (English fallback covers gaps).
5. **New platform** → branch in `src/platform.rs` only; UI stays untouched.

## Quality gate (must pass before pushing)

```
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo build --locked && cargo test --locked
docker compose config --quiet
```

## Last verified

2026-11-12: fmt 0, clippy 0, **153 lib + 10 contract + 5 router**, e2e
38/38 at scale 1, migrations 1–12, HA overlay valid.