# 17 — Spiral Execution Plan

**Model:** Spiral — a type of iterative-incremental model. At each turn of the
spiral a **minimal finished version** of the planned product is achieved, so
the customer sees and can react to the product from the beginning to the end of
development. The spiral model is chosen for Pinpoint because it **raises
attention to risk** in a project whose requirements may change.

All lifecycle phases proceed in spirals: design, coding, construction and
testing happen **at every turn**. The same set of processes need not repeat
identically from turn to turn — but each turn's results lead to the main goal.

## 1. The four stages of every spiral turn

Each turn (S0…S6 below) runs the same four stages:

1. **Goal elaboration** — define this turn's objectives, scope (FRs) and
   constraints; refine requirements with the customer.
2. **Risk analysis & prototyping** — identify risks (update
   `14-risk-register-and-assumptions.md`), prototype the risky parts, decide
   mitigation/abort.
3. **Development of an interim version** — implement, test, and produce a
   working increment (the "minimal finished version").
4. **Planning the next turn** — review the increment, capture feedback, and
   scope the next turn.

## 2. Spiral turns

| Turn | Theme | Minimal finished version | Dominant risks | Status |
|------|-------|--------------------------|----------------|--------|
| S0 | Foundation | Accounts, sessions, friendships, directional shares, backend | Wire-contract drift; auth security | ✅ |
| S1 | OSM map parity | Interactive Slippy map, search, routing, POI, bookmarks, offline packs | OSM ToS/rate limits; ODbL attribution | ✅ |
| S2 | Security hardening | MFA, OAuth2 PKCE, scraping detection, lockout, E2E + OS keychain | Crypto misuse; key storage | ✅ |
| S3 | Reliability | Failover, health monitoring, offline queue, multi-source tiles | Single point of failure | ✅ |
| S4 | Multiplatform & mobile | Rust desktop/Android/iOS + Kotlin Android, GPS, deep links | NDK/Xcode availability; mobile API drift | 🟡 |
| S5 | Verification & compliance | ZAP active scan, traceability, full e2e | ZAP findings; traceability gaps | 🟡 |
| S6 | Deployment & thesis report | Prod checklist, LaTeX report, README guide | Prod secrets; report completeness | ⬜ |

## 3. Turn details

### S0 — Foundation ✅

1. **Goal elaboration:** accounts, sessions, friendships, directional shares,
   publish/friend locations, freshness, admin (FR-01…).
2. **Risk analysis & prototyping:** risk = wire-contract drift between clients
   and services. Mitigation = `tests/contract.rs` contract tests + a CLI/SPA
   prototype of the auth/share flow.
3. **Development:** Rust crate (`src/`), modular Axum backend (`src/server/`),
   web SPA (`web/`), contract tests.
4. **Next turn:** freeze the API contract; scope OSM map parity (S1).
- **Exit criteria:** gate green; contract tests pin session cookie +
  `X-CSRF-Token`.

### S1 — OSM map parity ✅

1. **Goal elaboration:** interactive Slippy map, Nominatim search, OSRM
   routing, Overpass POI, bookmarks, offline packs — OSM/ODbL compliant.
2. **Risk analysis & prototyping:** risks = OSM ToS/rate limits, attribution
   removal (R-01…R-06). Mitigation = `scripts/qa.sh`/`.ps1` tile-policy grep,
   multi-source tile fallback, permanent ODbL credit; prototype = static map →
   interactive canvas.
3. **Development:** `src/map.rs`, `src/tilepacks.rs`, `src/osm.rs`,
   `map-qa` CI, ODbL attribution.
4. **Next turn:** scope security hardening (S2).
- **Exit criteria:** `e2e` 38/38; `map-qa` green; no
  `tile.openstreetmap.org`; attribution on every map surface.

### S2 — Security hardening ✅

1. **Goal elaboration:** MFA/TOTP, OAuth2 PKCE, scraping detection, login
   lockout, E2E location encryption, OS keychain persistence.
2. **Risk analysis & prototyping:** risks = crypto misuse, key storage, session
   replay. Mitigation = regression + fail-closed tests per control,
   `cargo audit`, `OsKeychain`/`FileKeychain` fallback; prototype = E2E round
   trip + keychain round trip before wiring.
3. **Development:** `src/security/{mfa,oauth,scrape,e2e,keychain}.rs`,
   `KeyManager` persistence wired into login/register/publish.
4. **Next turn:** scope reliability (S3).
- **Exit criteria:** every control has a regression + fail-closed test;
  `cargo audit` clean.

### S3 — Reliability ✅

1. **Goal elaboration:** multi-replica failover, health monitoring, offline
   mutation queue, multi-source tile proxy.
2. **Risk analysis & prototyping:** risk = single point of failure. Mitigation =
   circuit breaker, health checks, `/sync-status`, failover drill; prototype =
   kill-one-replica drill.
3. **Development:** `src/backend/failover.rs`,
   `src/security/circuit_breaker.rs`, `PendingMutation` queue.
4. **Next turn:** scope multiplatform & mobile (S4).
- **Exit criteria:** `/health` verifies DB; failover drill passes; offline
  queue replays on recovery.

### S4 — Multiplatform & mobile 🟡

1. **Goal elaboration:** one Rust crate for desktop/Android/iOS/Web + a
   distinct Kotlin Android app; GPS publish; `geo:`/`pinpoint://` deep links.
2. **Risk analysis & prototyping:** risks = NDK/Xcode unavailable on the dev
   host, mobile API drift (R-07). Mitigation = keep mobile code isolated in
   `src/mobile` + `:core:map`, add CI jobs; prototype = Rust mobile bootstrap +
   Kotlin M1 scaffold.
3. **Development:** `src/mobile/{mod,android,ios}.rs`, `GpsProvider`,
   `MobileDeepLink`, `mobile/` (Kotlin), `android/`, `ios/`,
   `[package.metadata.android]`.
4. **Next turn:** scope verification & compliance (S5).
- **Exit criteria:**
  - [x] Rust mobile bootstrap + deep-link parser + GPS provider.
  - [x] Kotlin Android app in tree.
  - [ ] Android APK + WASM CI jobs green.
  - [ ] Kotlin app feature-complete (M1 today).
  - [ ] Full egui Web UI on wasm (optional for thesis MVP).

### S5 — Verification & compliance 🟡

1. **Goal elaboration:** independent security verification + complete
   traceability.
2. **Risk analysis & prototyping:** risks = ZAP findings, traceability gaps.
   Mitigation = baseline scan early, then full active scan; prototype = ZAP
   baseline checklist.
3. **Development:** ZAP active-scan report, updated `15-traceability-matrix.md`,
   full e2e run.
4. **Next turn:** scope deployment & report (S6).
- **Exit criteria:**
  - [x] `cargo test --locked` green (153 unit + 10 contract + 5 router).
  - [x] `scripts/e2e.ps1` 38/38 at scale 1.
  - [ ] OWASP ZAP active scan clean + report filed.
  - [ ] Traceability matrix updated for every shipped feature.

### S6 — Deployment & thesis report ⬜

1. **Goal elaboration:** production hardening + final thesis deliverables.
2. **Risk analysis & prototyping:** risks = prod secrets, incomplete report,
   OSM hosting cost. Mitigation = staging deploy, prod checklist, self-hosted
   OSM path; prototype = staging environment.
3. **Development:** prod checklist sign-off, LaTeX report, public README
   run/test guide, updated risk register.
4. **Next turn:** release / v1.0.
- **Exit criteria:**
  - [x] `docker compose up --build` → 6 services healthy; HTTPS live.
  - [ ] Production CA + migrate-only DB role.
  - [ ] Self-hosted OSM path (or documented managed path).
  - [ ] Final report (LaTeX/PDF) + README guide published.

## 4. Pros / cons and mitigations

| Pros (why this model) | Cons | Mitigation in Pinpoint |
|---|---|---|
| High level of risk analysis | High cost | Time-box risk analysis to the top risks per turn; reuse `14-risk-register` |
| Prototype appears early | Difficult for small projects | Keep turns small; prototype only the risky parts |
| Suitable for large projects | High reliance on risk analysis | Make risk review a mandatory stage-2 gate with a written decision |
| Result in the shortest time | Can spiral outward (scope creep) | "Minimal finished version" per turn; defer extras to the next turn |
| Requirements changes don't restart from scratch | Needs experienced risk judgement | ADRs + traceability matrix keep decisions explicit |

## 5. Spiral discipline (rules)

1. **Every turn runs all four stages** — goal, risk, development, next-turn.
2. **Risk analysis is mandatory** and recorded in
   `14-risk-register-and-assumptions.md` before development starts.
3. **Each turn ends with a working increment** ("minimal finished version")
   that passes the repo gate: `cargo fmt/clippy/build/test --locked` (+ e2e
   where applicable).
4. **The customer sees the increment** at the end of each turn (demo/review);
   feedback feeds the next turn's goal elaboration.
5. **Scope is frozen within a turn**; changes wait for the next turn.
6. **Semantic/wire changes** need an ADR (`docs/adr/`) in the same turn.

## 6. Critical path to completion

1. **S4:** Android APK + WASM CI jobs; Kotlin feature completion.
2. **S5:** ZAP active scan + traceability matrix update.
3. **S6:** Production checklist (CA, migrate-only role, OSM hosting).
4. **S6:** Final LaTeX report + README run/test guide.
5. (Optional) Full egui Web UI on wasm; Patroni HA.

## 7. Definition of Done (project)

- [ ] S0–S6 exit criteria met (this document).
- [ ] `docs/BEST_APP_CRITERIA.md` all boxes checked (A/B/C/D).
- [ ] `scripts/e2e.ps1` 38/38 and the cargo gate green on `main`.
- [ ] Final report + README run/test guide published.
- [ ] Risk register and assumptions revisited at release.
