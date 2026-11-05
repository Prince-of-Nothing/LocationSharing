# Pinpoint: Monthly Progress and Rust Transition

**Period:** 29 September–4 November 2026  
**Prepared:** 4 November 2026  
**Audience:** project team, assessors, and future contributors

> **Direction:** We are switching Pinpoint to Rust and intend to keep Rust as its long-term primary implementation. The migration has started, but is not complete. The Go and Python services remain the existing APIs; the Rust code now present is an early desktop-client scaffold. The plan is to migrate in stages, verify behavioral and security parity, and retire older implementations only after clients, data, and deployment have a tested transition path.

## Summary

Pinpoint is a privacy-first location-sharing platform for trusted friends. Its defining rule is that friendship is not consent: sharing is directional, can be limited by precision and duration, and must be revocable. The server—not the client—must enforce every access decision.

Since the September MVP, the project added PostgreSQL support behind Go store interfaces, a database conformance suite, expanded CI, a privacy-conscious web client, Android development, public registration, and substantially more architecture and security documentation. The newest strategic change is the decision to retain Rust as the destination language.

The immediate constraint is that the Rust desktop scaffold does **not** compile. On 4 November, `cargo test --locked` stopped with 39 compiler errors; there are no Rust tests in `src/`, and the inspected CI workflow has no Rust job. Existing Go/Python work remains useful as a compatibility baseline, but its security controls do not automatically carry over to Rust. The project is in **active migration and development**, not production-ready.

## Product and system at a glance

| Area | Scope and current state |
|---|---|
| **Product** | Private-by-default accounts; login, sessions and password management; friend requests, blocking and removal; explicit one-way location shares with precision, purpose, expiry, pause/revocation; current-location publishing and authorized viewing; freshness/staleness states; safety check-ins; administrator operations. |
| **Sensitive data** | Account identifiers, credentials/session material, social graph, and especially location coordinates and timestamps. Locations can reveal routines, home, work, and relationships. Minimize retention and never put credentials or precise locations in logs. |
| **Go API** | Existing default backend in `backend/cmd/server` and `backend/internal/`. Has in-memory and PostgreSQL stores behind repository interfaces. PostgreSQL schema bootstrap exists; managed migration tooling remains open. |
| **Python API** | Separate FastAPI implementation in `backend/app`, with its own migrations, tests, and `backend/web/` client. Remains an alternate/legacy implementation during the Rust transition. Some pattern-catalogue modules are ported artifacts; verify active imports before claiming runtime protection. |
| **Web clients** | `web/` is the Go-stack modular vanilla-JS client; `backend/web/` is a separate Python-served client. Offline-shell and tile-policy/CSP work is documented. Preserve visible attribution and tile-provider policy. |
| **Android** | Kotlin/Jetpack Compose, modular `core:*` and `feature:*` structure, MapLibre. Encrypted session storage and combined auth are documented; background location, release signing, and broader device validation remain. |
| **Rust (new direction)** | Root crate `pinpoint-desktop`, Rust 2021, `eframe`/`egui`, `reqwest`, and Serde. Current code targets the existing HTTP API; it is not yet a Rust backend or a working release client. |
| **Delivery evidence** | CI has Go/PostgreSQL, Python, web, and Android jobs. The October report records all four green on 2 November; that result predates the 3 November rename. Deployment configuration was prepared, but the latest report still listed credentials/live verification as outstanding. |

### Product invariants to carry into Rust

- **Deny by default:** every sensitive read/write requires server-side authorization.
- **Consent is directional:** A sharing with B never implies B shares with A; access respects active friendship, blocks, expiry, and revocation.
- **Precision is server-controlled:** a client cannot upgrade its own visibility or request an unauthorized exact location.
- **Data minimization:** the MVP retains the latest location, not a movement history, unless a separately reviewed feature changes that policy.
- **Fail closed:** security changes require both a regression test and a test proving denied/invalid cases remain denied.
- **Privacy is product behavior:** no secrets or real user data in source/tests; preserve the mobile no-tracker baseline and map licensing/attribution.

## Progress since September

| Date | Milestone |
|---|---|
| **29–30 September** | Go-based MVP and demo baseline: authentication/sessions, rate limiting, friendship/blocking, directional sharing, location/check-in features, administrator behavior, web map, Docker/CI, and attack-oriented security tests/documentation. |
| **13 October** | Project consolidation and an eight-week, 40-task board established. |
| **28–29 October** | Go repository interfaces, PostgreSQL driver and conformance suite; expanded CI; Android scaffold; modular privacy-first web client; security architecture, ADR, and pattern documentation. |
| **2 November** | Public registration with fail-closed tests, mobile combined sign-in/sign-up, deployment preparation, and documentation refresh. The progress report records all four then-existing CI jobs passing. |
| **3 November** | Repository-wide rename to Pinpoint. A fresh CI result after this change is not documented here. |
| **4 November** | Rust desktop source is present; the team’s direction is to keep Rust. `cargo test --locked` fails to compile, making Rust stabilization the immediate priority. |

## Rust transition: what exists and what it means

### Current code

| Source | Intended role | Current limitation |
|---|---|---|
| `Cargo.toml`, `Cargo.lock` | Rust 2021 desktop crate; `eframe`/`egui`, Serde, HTTP and async dependencies. | No Rust CI job. |
| `src/main.rs` | Native window setup, `--backend` and `--screenshot` options; default API URL `http://localhost:8000`. | Module/API mismatch prevents compilation; WebAssembly entry point is only a placeholder. |
| `src/models.rs`, `src/lib.rs` | User, friendship, share, location and application data. | Duplicate models; most domain states are unrestricted strings. |
| `src/api.rs`, `src/auth.rs` | Login/register/logout and friendship, share, and location HTTP calls. | Overlapping implementations; `auth.rs` is not wired from `main.rs`. Cookie/CSRF behavior is not adequately tested. |
| `src/app.rs`, `src/ui.rs` | App state and screens for login, registration, map, friends, sharing, and admin. | Form widgets use temporary values; several actions are placeholders; some errors are discarded; async calls are attempted inside synchronous UI closures. |
| `src/map.rs` | Draw location markers/labels. | Prototype visualization; currently imports types that do not match the model definitions. |
| `build.rs` | Cargo build hook. | Placeholder only. |

The Rust client begins targeting existing routes for authentication, friendships, location shares, friend locations, and publishing a location. These are starting points, not proof of API parity. Compare payloads, status codes, session behavior, and CSRF requirements against the active server before treating them as a stable contract.

### Build evidence

`cargo test --locked` on 4 November **failed to compile with 39 errors**. Representative issues include unresolved or private model imports, incompatible `egui` types, `.await` inside synchronous UI closures, and mismatched optional fields and method signatures. No `#[test]` or `#[tokio::test]` functions were found under `src/`; therefore, there is no passing Rust behavioral-test evidence yet.

This is a compiler/build failure, not evidence that Rust tests ran and failed. First establish a compiling module and state model. Then add unit, API-contract, and security tests and make Rust a required CI check.

## Design patterns

### Already evidenced

| Pattern | Where | Qualification |
|---|---|---|
| **Repository / interchangeable adapters** | Go `backend/internal/store`: common contracts, memory and PostgreSQL implementations, driver selection, conformance tests. | Implemented in Go; a useful model for Rust persistence. |
| **Middleware and dependency boundaries** | Existing Go/Python APIs centralize request controls; Android separates domain, network, data, map, and feature modules. | Present in those systems; must be implemented and verified independently in Rust. |
| **Clean Architecture** | Kotlin Android module boundaries. | Android architecture, not evidence of Rust architecture. |
| **Facade, Strategy/policy, Factory, Chain of Responsibility, Template Method, Null Object, Observer** | Catalogued in `docs/reference/design-patterns.md` and ADR-0003 for a Python security-hardening design. | Documentation is not proof every module is wired. Do not claim these patterns protect the Go API or Rust client without checking active imports and tests. |

### Apply in Rust where they clarify security

1. **Ports/adapters and Repository traits:** isolate database and HTTP details from domain decisions.
2. **Use-case/service layer:** centralize registration, share creation/revocation, and authorized location reads; check actor, ownership, consent, expiry, and precision in one testable path.
3. **Authorization/precision policies:** explicit rules with fail-closed behavior for unknown states.
4. **Typed domain models:** enums/newtypes for role, share state, precision, and relationship status instead of arbitrary strings.
5. **API middleware/extractors:** consistent authentication, CSRF, rate limits, role checks, request limits, security headers, and safe errors.
6. **Desktop HTTP adapter and app state:** one configured client, timeouts, HTTPS outside loopback development, tested cookie/CSRF handling, explicit errors, and UI/network separation.
7. **Audit observer only where useful:** record privacy-safe events without logging tokens, passwords, or coordinates.

A practical future split is `domain/`, `application/`, `adapters/`, `api/`, and `desktop/` (or separate Cargo workspace members). Do not add layers just to count patterns.

## Security risks and priorities

This is a repository-grounded risk summary, not a new penetration test.

| Priority | Risk | Action / release evidence |
|---|---|---|
| **Critical blocker** | Rust does not compile; no Rust tests or CI job. | Fix the crate, then require build, test, format, lint, and dependency checks in CI. |
| **High** | The 2 November report recorded 20 Dependabot alerts (8 critical, 4 high, 8 moderate); current count not rechecked. | Re-query alerts; remediate critical/high first and record remaining exposure. Add a Rust dependency audit. |
| **High** | Rust defaults to HTTP localhost and accepts a backend URL; production transport policy is not implemented. | Allow HTTP only for loopback development; require HTTPS for non-loopback/release use. |
| **High** | Rust manually extracts/sends a session cookie; CSRF behavior is not evident. | Match the server contract; test cookie handling and CSRF for every state-changing request. |
| **High** | UI actions are incomplete; logout ignores server failure; refresh errors can be discarded. | Wire real form state/actions; surface failures; never report server logout success if revocation failed. |
| **High** | MFA/recovery remain incomplete for public use; registration was newly added. | Threat-model registration/recovery, abuse limits, and administrator MFA before broad onboarding. |
| **High** | Production deployment was prepared but not live-verified in the latest status report. | Verify TLS, secrets, database, seed flags, CORS/CSP, and health checks using team-controlled credentials. |
| **Medium–High** | In-process rate-limit fallback weakens limits across multiple workers; shared Redis remains planned. | Require shared limits in multi-instance deployment and test fallback/outage behavior. |
| **Medium–High** | Schema migrations remain incomplete for Go; Python risk register also identifies migration wiring debt. | Version schema changes and test fresh install, upgrade, and recovery before production. |
| **Medium** | Audit/monitoring is partial; location reads and alerting are not fully covered. | Define privacy-safe read auditing, retention, alert thresholds, and incident exercises. |
| **Medium** | Rust duplicates models and auth logic; frequent `Mutex::lock().unwrap()` can panic; backend errors may reach UI. | Consolidate sources of truth, avoid locks across awaits, propagate errors safely, and prevent sensitive detail leakage. |
| **Medium** | Android release signing/background-location controls and Rust endpoint parity remain open. | Protect release keys, gate background access on consent, and test clients against the Rust contract. |

**Assessment:** the legacy stacks have a meaningful privacy/security baseline, but those controls are **not verified in Rust** until reimplemented and tested. Keep the project in development/beta preparation; do not describe the Rust path as production-ready.

## Plan for the coming months

| Window | Priority work | Exit evidence |
|---|---|---|
| **November — stabilize** | Fix Rust compilation; remove duplicate models/auth code; define module boundaries; implement stateful login/register/logout and core client actions; restrict HTTP to local development; re-run CI after rename and add Rust CI. | `cargo build/test --locked`, format and lint pass; contract tests cover auth, logout failure, and CSRF; CI reports Rust status. |
| **December — Rust backend foundation** | Define API contract and typed domain; create Rust server/health endpoint; implement session/auth and deny-by-default authorization; add repository traits, database adapter, and migration decision. Keep Go/Python for comparison. | Rust server starts in CI; migrations work from an empty database; authorization and session fail-closed tests pass; initial endpoint parity checks pass. |
| **January — migrate core behavior** | Port users, friendship/blocking, directional shares, precision filtering, location publish/read, and check-ins; add rate limiting, audit events, and structured errors. | Integration tests prove API parity, denied foreign reads, expiry/revocation, precision downgrade, rate-limit behavior, and privacy-safe logs. |
| **February — operations and clients** | Complete PostgreSQL migrations, backup/restore, secrets and deployment guidance; point clients to the Rust contract; add dependency/security scanning and monitoring. | Restore drill passes; clients smoke-test against Rust; current threat/dependency reviews and deployment checks are documented. |
| **After parity — retire carefully** | Freeze features in superseded API(s); plan data/client transition; deprecate only after team review. | Parity sign-off, verified backup/migration, no dependent clients, and tested rollback before removal. |

### Gates before Rust becomes the production path

- **Build and tests:** reproducible locked build, unit/integration tests, and required CI.
- **Behavior:** contract and parity tests for payloads, status codes, auth, and failures.
- **Security:** IDOR/access matrix, fail-closed, CSRF/session, rate-limit, and dependency tests.
- **Privacy:** consent/precision behavior, retention decision, safe logs, and permission review.
- **Operations:** HTTPS, secrets, migrations/rollback, backup/restore, health and incident process.
- **Retirement:** do not remove Go/Python until clients, data, and rollback are accounted for.

## References

- [October Progress and Project Status](./october-progress-and-project-status.md)
- [Task 1 Security Requirements](./owasp-top10-and-security-requirements.md)
- [Task board](../TASKS.md), [roadmap](../ROADMAP.md), [planned tasks](../PLANNED_TASKS.md)
- [Architecture](../ARCHITECTURE.md), [Risks and Technical Debt](../architecture/risks-and-technical-debt.md)
- [Design Patterns and Principles](../reference/design-patterns.md), [ADR-0003](../adr/0003-structure-backend-around-design-patterns.md)
- [Security Control Register](../reference/security-control-register.md), [Privacy](../PRIVACY.md)
- Rust evidence: root `Cargo.toml`, `Cargo.lock`, `src/`, and the 4 November `cargo test --locked` attempt.

**Bottom line:** Rust is the chosen long-term direction and will be retained. The migration has begun, but today’s Rust scaffold is a non-compiling desktop client, not a replacement backend. Stabilize it, port security and behavior with tests, and retire the existing APIs only after parity and release gates pass.
