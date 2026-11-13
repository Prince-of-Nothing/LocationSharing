# Pinpoint — "Best App" Pass Criteria (measurable)

Goal `have the best app ever` is only met when ALL criteria below hold. Status verified
2026-11-08 (full E2E `scripts/e2e.ps1` + cargo QA below).

## A. Security (must pass)
- [x] Real password hashing (Argon2id PHC, memory-hard) — correct password accepted, wrong rejected (`security/password.rs` tests)
- [x] Path-param routing fixed (matchit 0.7 `:id` syntax) — `tests/router_probe.rs` 3/3 in-process + live (`GET /users/2` → Alice, `PATCH /shares/:id` → proper JSON errors)
- [x] Gateway hardening (Z-1/Z-3/Z-5) live on HTTP profile: `nosniff`/`DENY`/CSP + version-free `Server: nginx` (curl-verified)
- [x] **MFA E2E live**: enroll → verify → login gate (`MFA_REQUIRED`) → login with fresh TOTP 200, replay rejected (`scripts/e2e.ps1` checks 7–10)
- [x] Normalized DB: 8 typed tables, dual-write + load-normalized authoritative, per-service ownership writes (`ServiceKind`), roles created (`pinpoint_auth/users/locations`)
- [x] Per-service DB wiring live (2026-11-08): auth/users/locations connect with own roles; `db/02-grants.sql` applied + probes verified (`psql -U pinpoint_auth/locations`); E2E 36/36 under least-privilege
- [x] Typed migration history: `src/server/migrations.rs` (v1..v10, `pinpoint_schema_migrations` version table, transactional, idempotent; legacy DDL converges by recording versions)
- [x] Store convergence: `refresh_from_db()` after each `persist()` tick (5s) — cross-service reads converge without restarts
- [ ] Normalized DB residual: superuser kept as bootstrap/migration operator only (documented above); revoke fully when prod runs migrations under a migrate-only role.
- [x] Tile proxy code-complete (CARTO + Wikimedia + OpenFreeMap multi-source ODbL fallback + TTL cache + attribution)
- [x] HTTPS live: self-signed `:8443/health` → `{"tls":"on"}`, HSTS/CSP/nosniff/DENY, HTTP→HTTPS 301, login over TLS hits MFA gate
- [x] E2E encryption + OS-keychain (`src/security/e2e.rs` + `src/security/keychain.rs`, wired on login/register/publish, `docs/security/e2e-mfa-design.md`)

## B. Features (all screens render offline-safe)
- [x] 10 screens: Map, Friends, Sharing (+expiry/delete), Places (OSRM/Overpass/reverse), Saved, Trips, Emergency, Profile, Admin, Password
- [x] Server: shares/friendships CRUD + delete, check-ins, emergency, user search, my-location — **36/36 e2e checks pass, twice in a row** (`scripts/e2e.ps1`)
- [x] Realtime updates: auto-refresh poller + freshness badges (in code + unit tests)
- [x] Offline map packs: `src/tilepacks.rs` (Slippy math + throttled CARTO download, resume-safe, 4096 cap)

## C. Quality gates
- [x] `cargo fmt --all -- --check` → 0
- [x] `cargo clippy --all-targets -- -D warnings` → 0
- [x] `cargo test` → **104 lib + 10 contract + 4 router**, 1 ignored (live round-trip)
- [x] `docker compose up --build` → 6 services healthy (2 replicas each), gateway :8000
- [x] `scripts/e2e.ps1` → **38/38 at scale 1** (`--scale ...=1`); scale 2 is eventually consistent (≤5s persist→refresh) and verified by the failover drill + `X-Upstream`/`/registry` (see `docs/redundancy.md`, `docs/gap-acceptance.md`)
- [ ] OWASP ZAP active scan clean (Z-6/Z-7) — baseline checklist in `docs/evidence/zap-scan-report.md`, full ZAP GUI run pending
- [x] No `tile.openstreetmap.org` tile URLs in code (OSMF policy, CI `map-qa.yml` grep) + `openstreetmap.org/copyright` ODbL attribution in `src/map.rs`

## D. Docs (DSA)
- [x] Threat model (STRIDE/DREAD/PASTA, attack tree) + incident response plan + OWASP requirements
- [x] Feature inventory (`docs/FEATURE_INVENTORY.md`) + design principles (`docs/design-principles.md`)
- [ ] Final report (LaTeX/PDF, faculty format) + README run/test guide public

**Current verdict (2026-11-08 late):** A ✓ (except tile-200 needs key + grants/migrations), B ✓ (36/36 e2e), C ✓ (except ZAP GUI run + CI grep formalities), D partial. Remaining blockers needing the user: free CARTO API key (tiles 200), prod CA/role rollout, ZAP desktop run, final LaTeX report.
