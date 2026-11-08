# Pinpoint — Gap acceptance (explicit decisions)

What the principles matrix leaves open, and the deliberate decision for
the demo/thesis stage. “Accept” = knowingly shipped as-is with a
documented next step, not hidden.

| Gap | Decision for demo | Next step (prod) |
|---|---|---|
| Single Postgres primary | **Accepted.** Connected-DB joins are the product; `pg_dump` every 300s + opt-in streaming overlay (`docker-compose.ha.yml`, `docs/postgres-ha.md`) | Managed HA Postgres or Patroni |
| Single gateway | **Accepted.** Auto-restarts; URL never changes | Two gateways behind your LB |
| E2E encryption + OS keychain | **Accepted.** Server sees locations (thesis scope); bookmark encryption shipped | Design-only until thesis timeline allows |
| ZAP active scan | **Pending.** Baseline passed (headers/CSP); active scan not run | Run + file results before public beta |
| Linux/macOS GUI first-run | **Accepted.** CI builds pass (`rust` ubuntu, `client-macos`, windows `release`); GUI run needs a human on each OS | Manual tick-list in `docs/client-platforms.md` |
| Android APK / WASM Web GUI | **Accepted as scaffolding.** Manifests + stub entries shipped; no CI build | NDK / wasm-toolchain CI jobs |
| Tiles live 200s | **Accepted.** Proxy fails closed with guidance until `CARTO_API_KEY` | Free key from carto.com/basemaps |

Verifier: `docker compose up --build -d` → `docker compose ps`
(all healthy, 2 replicas each) → migrations `1,2,…,11` → `scripts/e2e.ps1`
38/38. Last verified: this run (see commit message).
