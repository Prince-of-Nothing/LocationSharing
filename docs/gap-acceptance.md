# Pinpoint — Gap acceptance (explicit decisions)

What the principles matrix leaves open, and the deliberate decision for
the demo/thesis stage. “Accept” = knowingly shipped as-is with a
documented next step, not hidden.

| Gap | Decision for demo | Next step (prod) |
|---|---|---|
| Single Postgres primary | **Accepted.** Connected-DB joins are the product; `pg_dump` every 300s + opt-in streaming overlay (`docker-compose.ha.yml`, `docs/postgres-ha.md`) | Managed HA Postgres or Patroni |
| Single gateway | **Accepted.** Auto-restarts; URL never changes. `/health` now verifies DB connectivity and returns `degraded` when unreachable; `/api/v1/sync-status` reports replication freshness per replica | Two gateways behind your LB |
| Client offline resilience | **Resolved.** `PendingMutation` queue in `PinpointApp` — all mutation methods queue locally when all backends are unavailable; `sync_pending_mutations()` auto-replays on recovery; `pending_mutation_count()` / `pending_mutation_descriptions()` exposed for UI; offline mode auto-engaged on failure | None (shipped) |
| Client backend failover | **Resolved.** `FailoverBackend` wraps multiple REST backends with per-backend circuit breakers (Closed/HalfOpen/Open), health monitoring, and exponential backoff retry. `PINPOINT_FAILOVER_URLS` env var configures replicas | None (shipped) |
| E2E encryption + OS keychain | **Accepted.** Server sees locations (thesis scope); bookmark encryption shipped | Design-only until thesis timeline allows |
| ZAP active scan | **Pending.** Baseline passed (headers/CSP); active scan not run | Run + file results before public beta |
| Linux/macOS GUI first-run | **Accepted.** CI builds pass (`rust` ubuntu, `client-macos`, windows `release`); GUI run needs a human on each OS | Manual tick-list in `docs/client-platforms.md` |
| Android APK / WASM Web GUI | **Accepted as scaffolding.** Manifests + stub entries shipped; no CI build | NDK / wasm-toolchain CI jobs |
| Tiles live 200s | **Resolved.** Tile proxy now tries multiple ODbL-compliant sources in order (CARTO with key → Wikimedia → OpenFreeMap) instead of failing closed at 502. Map works without any external key. `X-Tile-Source` header reports which source served each tile | None (shipped) |

Verifier: `docker compose up --build -d` → `docker compose ps`
(all healthy, 2 replicas each) → migrations `1,2,3,4,5,6,7,8,9,10,11` (v10
now recorded: out-of-transaction end-state check) → with
`--scale {auth,users,locations,map}=1` then `scripts/e2e.ps1` 38/38.
Note: at `scale: 2` the stack is eventually consistent (≤5s
persist→refresh window), so the deterministic e2e is run at scale 1;
HA failover (kill one replica, `X-Upstream` + `/registry` still serve)
is verified at scale 2. Last verified: this run (see commit message).
