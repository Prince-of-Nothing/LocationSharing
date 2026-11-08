# 11 — QA and Verification (automatic QA)

## Strategy

Thesis rule: every security-relevant behavior has executable evidence
(requirement → implementation → test). This release adds **automatic map
and compliance QA** on top of the existing CI.

| Layer | Tools | Scope | Gate |
|-------|-------|-------|------|
| Unit (Go/Python) | `go test`, `pytest` | auth, consent, precision, freshness | CI `backend*` |
| Web syntax | `node --input-type=module --check` (all `web/public/js/*.js` + `sw.js`) | ES modules | CI `web` |
| Mobile | `gradle :core:domain:test :core:network:test :app:assembleDebug` | JVM + APK | CI `mobile` |
| Rust | `fmt --check`, `clippy -D warnings`, `build/test --locked`, `cargo-audit` | desktop | CI `rust` |
| **Map compliance (new)** | `scripts/qa.sh` / `scripts/qa.ps1` | tile policy, attribution, CSP, SHELL sync, Mermaid, links | CI `map-qa` |
| E2E (planned) | Playwright smoke (`e2e/`) | login → map → search → route | manual |
| Load (planned) | k6 on `/locations` | hot path | manual |

## `scripts/qa.sh` checks (run locally + CI)

1. **Tile policy:** no `tile.openstreetmap.org` in `web/`, `mobile/`,
   `backend/web/` (blocker). Must find OSM wiki + Organic Maps references
   in docs (informational).
2. **Attribution:** `openstreetmap.org/copyright` + ODbL present in web
   `config.js`, mobile notices, Rust map panel (blocker if missing).
3. **CSP sync:** `nginx.conf` `connect-src` contains Nominatim/OSRM/Overpass
   and `img-src` contains CARTO/Wikimedia; `config.js` layers match.
4. **SW SHELL sync:** every `web/public/js/*.js` listed in `sw.js` SHELL;
   `SW_VERSION` bumped (warn if not).
5. **Syntax:** all web JS `node --check`; manifest JSON parses.
6. **Mermaid:** all `docs/uml/*.md` + `docs/sdlc/*.md` contain ```mermaid
   fences closed (simple lint, no renderer needed).
7. **Link hygiene:** OSM wiki / Organic Maps / OSMF policy URLs present in
   SDLC set (informational).
8. **Notices:** both `THIRD_PARTY_NOTICES.md` mention Nominatim/OSRM/Overpass.

Exit 0 = clean tree. Failures print `FAIL:` lines with file/line context.

## Manual QA protocol (release candidate)

- [ ] `docker compose up --build` boots; demo accounts sign in on web.
- [ ] Web: pan/zoom all modes; layer switch; search ≥3 chars; tap reverse;
      route all 3 modes; POI all 9 cats; bookmarks persist + export.
- [ ] Mobile: map loads; offline pack downloads; search/route/POI/bookmarks
      screens work; `geo:` hand-off opens a nav app.
- [ ] Rust: map shows pins; all 3 OSM buttons + credit correct.
- [ ] Offline banner appears with no network; shell + bookmarks work.
- [ ] Logout clears session + geocode cache; no friend data in storage.

## Exit criteria

All automatic gates green + manual checklist signed off in the PR.
Security fixes require regression + fail-closed tests (per CONTRIBUTING).

## Known flakes / limits

- `tests/test_auth.py::test_session_limit_revokes_oldest` flakes in full
  local runs (pre-existing on `origin/main`) — not a release blocker.
- Public OSM services can rate-limit; QA does not call them live (no
  flakiness in CI).
