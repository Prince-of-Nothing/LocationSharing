# Pinpoint — Principles matrix (close-to-all, with evidence)

How each design principle is implemented, where, and how it is tested.
“Shipped” = code + test; “Partial” = implemented but with a known gap;
“Not yet” = tracked gap. The goal is a new service, table, or screen to
be additive — never a rewrite.

## Summary

| # | Principle | Status | Implementation | Evidence |
|---|---|---|---|---|
| 1 | Single Responsibility (SRP) | Shipped | Per-service handlers, per-screen UI, per-domain store modules | `cargo test` (104 lib), handler/router probes |
| 2 | Separation of Concerns | Shipped | transport/domain/OSM/client layers never leak | `tests/contract.rs` pins `api.rs` |
| 3 | Modular Architecture (services) | Shipped | 4 deployables + gateway routing, `scale: 2` | `docker compose config`, e2e 38 |
| 4 | Modular Database (ownership) | Shipped | Per-domain upsert persist + advisory lock, roles, FKs, migrations v1–v11 (seq ids) | ownership + lock-key tests |
| 5 | Open/Closed (OCP) | Shipped | New table = new migration + domain module; new screen = new `ui/` module; new service = new router + `ServiceKind` | ADR-0006/0008, migrations test |
| 6 | Liskov (LSP) | Shipped | `patterns/repository.rs`: `InMemory` + `Postgres` impls, same traits | LSP conformance + Postgres conformance (skips without DB) |
| 7 | Interface Segregation (ISP) | Shipped | Small traits (`UserRepository`, …) + small routers (`auth_router`…) | compile + router probes |
| 8 | Dependency Inversion (DIP) | Shipped | `patterns/factory` builds `OsmClient`; `open_repository()` factory picks `dyn Repository` | factory + trait-object tests |
| 9 | DRY | Shipped | Shared helpers in `handlers.rs`; validation in one place | `security/validation.rs` tests |
| 10 | KISS / YAGNI | Shipped | One crate, std-only crypto, no DI framework | `Cargo.toml`, no enterprise deps |
| 11 | Fail-Closed | Shipped | Loopback-only HTTP, login/logout fail closed, MFA replay rejected | contract tests (10) |
| 12 | Least Privilege | Shipped | Per-service Postgres roles, own-columns counters, `X-Upstream` least_conn | `db/init/01-roles.sh`, `02-grants.sql` |
| 13 | Contract-First | Shipped | `/api/v1` wire contract pinned, error envelope, e2e live suite | `tests/contract.rs`, `scripts/e2e.ps1` (38) |
| 14 | High Cohesion / Low Coupling | Shipped | God-files split: `handlers.rs` 1117→54, `ui.rs` 1727→612, `state.rs` 769→5 modules | line counts + `cargo clippy -D warnings` |
| 15 | Information Hiding | Partial | `RepositoryError`, `user_json` hide internals; URLs/errors leak only via envelope | contract error tests; audit log gaps remain |

## Detail

### 1. Single Responsibility (Shipped)

One reason to change per module: `server/{auth,users,locations}_handlers.rs`
(auth/MFA, users/friendships, shares/locations/check-ins/emergencies),
`server/map_handlers` in `mod.rs` (tiles/attribution), one binary per
deployable (`src/bin/{auth,users,locations,map}.rs`), one OSM module
(`osm.rs`), one platform module (`platform.rs` — every `cfg(target_os)`),
one security mechanism each (`security/*.rs`), one screen per `ui/` file.
The former `Store` god-struct is now five domain files:
`server/state/{mod,store,users,friendships,locations}.rs` (facade shape
in `store.rs`; per-table load/persist in the domain modules).

### 2. Separation of Concerns (Shipped)

Transport (Axum routers) holds no business rules; domain (`models.rs`)
is serde + pure math; OSM (`osm.rs`) only translates HTTP↔types; client
(`api.rs` → `app.rs` → `ui.rs`) never builds URLs or parses JSON in the
UI. `platform.rs` is the only file with OS branches.

### 3–4. Modular Architecture / Database (Shipped)

Four deployables + combined binary behind nginx (`docker/nginx-gateway*.conf`
is the central directory; `docs/service-discovery.md`). DB: one connected
Postgres (joins intact) with per-service table ownership (`ServiceKind`),
per-row upsert + delete-missing persisted in one advisory-locked
transaction (same-service replicas serialize, different services run
concurrently), least-privilege roles + FKs (`db/`), and typed
idempotent migrations v1–v10. Redundancy: `scale: 2` + healthchecks +
`proxy_next_upstream` + pg_dump backups (`docs/redundancy.md`).

### 5. Open/Closed (Shipped)

Additive seams: new table → new `migrations.rs` entry + domain
load/persist in the owning `state/` module; new screen → new `ui/` module
+ `CurrentScreen` variant; new service → new router + `ServiceKind` +
gateway location. No existing code is modified to add a peer.

### 6–8. LSP / ISP / DIP (Shipped)

- **ISP (Shipped):** small handler routers and small `Repository`
  traits (user/friendship/share/location only) — no fat interfaces.
- **LSP (Shipped):** `InMemoryRepository` and `PostgresRepository`
  (`patterns/repository.rs`) both implement `Repository`; a green
  conformance suite runs against either (Postgres skips without a DB).
- **DIP (Shipped):** `patterns/factory::osm_client()` inverts the OSM
  dependency; `open_repository()` inverts the storage backend
  (`dyn Repository`); handlers invert on `AppState`.

### 9–10. DRY / KISS / YAGNI (Shipped)

Shared helpers, one validation module for client+server, one `Store`
facade, std-crypto-only security (no bcrypt/argon crates), no DI/web
frameworks. The 4-service split is the most complex structure kept;
no event bus, no queue.

### 11–13. Fail-Closed / Least Privilege / Contract-First (Shipped)

ADR-0006 (loopback-only HTTP), fail-closed login/logout + MFA replay,
per-service DB roles + own-columns counters + gateway headers, and the
`/api/v1` contract pinned by `tests/contract.rs` + live `scripts/e2e.ps1`.

### 14–15. Cohesion / Information Hiding (Shipped / Partial)

God-files are gone; `cargo clippy -- -D warnings` enforces it. Hiding is
mostly done (envelopes, `user_json`); residual tradeoff: `AppState.store`
is `pub` for handler convenience (could be private behind domain
methods) and `X-Upstream` intentionally exposes replica IPs to operators.

## Remaining honest gaps

- **Single Postgres primary** — streaming-replica overlay (`docker-compose.ha.yml`,
  `docs/postgres-ha.md`) exists but is opt-in; default recovery is volume
  restart / pg_dump restore. **Single gateway** — auto-restarts; two behind
  your own LB is the prod step.
- **E2E encryption + OS keychain** are design-only.
- **ZAP active scan** pending; tiles need `CARTO_API_KEY` for live 200s.
- **Client CI:** Linux/macOS/Windows build jobs green (ubuntu `rust`,
  `client-macos`, windows `release`); Android APK / WASM remain
  scaffolding-only — no CI job yet (need NDK / wasm toolchains; see
  `docs/client-platforms.md`).
