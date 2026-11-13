# 15 — Traceability Matrix (current)

Requirement → business goal → design → implementation → evidence.
References are to the **current** tree (Rust + Kotlin); archived stacks are not
listed here (see `docs/components/` for their history).

| Req | Business goal | Design | Implementation | Evidence |
|-----|---------------|--------|----------------|----------|
| FR-01 | BG-1 | identity sign-in | `src/server/auth_handlers.rs` (login), `src/security/password.rs` (Argon2id), `src/api.rs` (cookie + CSRF) | `tests/contract.rs` login/logout; `cargo test`; `scripts/e2e.ps1` auth checks |
| FR-02 | BG-1 | friendships | `src/server/users_handlers.rs`, `src/server/state/friendships.rs` | `cargo test`; e2e friendship checks |
| FR-03/04 | BG-1 | shares/locations | `src/server/locations_handlers.rs`, `src/server/state/locations.rs` | `cargo test`; e2e share/publish checks |
| FR-05 | BG-1 | freshness | `src/models.rs` (`freshness_state`, `age_secs`), `src/map.rs` (badges) | `models::geo_tests::freshness_thresholds` |
| FR-06 | BG-2 | Slippy engine | `src/map.rs` (canvas), `src/tilepacks.rs` (projection), `src/ui/map.rs` | `map::tests`, `tilepacks::tests`; `scripts/qa.sh` |
| FR-07/08 | BG-2 | Nominatim | `src/osm.rs` (`search`/`reverse`) | `cargo test`; manual search QA; policy grep |
| FR-09 | BG-2 | OSRM | `src/osm.rs` (`route`), `src/ui/places.rs` | `cargo test`; manual route QA |
| FR-10 | BG-2 | Overpass | `src/osm.rs` (`pois`), `src/ui/places.rs` | `cargo test`; manual POI QA |
| FR-11 | BG-2/3 | bookmarks | `src/bookmarks.rs` (JSON + GeoJSON), `src/ui/bookmarks.rs` | `bookmarks::tests`; offline bookmark QA |
| FR-12 | BG-2 | hand-off | `src/map.rs` (OSM/directions/Organic), `src/mobile/mod.rs` (`geo:` deep link) | `mobile::tests`; manual `geo:` QA |
| FR-13/14 | BG-2 | offline+styles | `src/tilepacks.rs`, `mobile/core/map/` (MapLibre, offline manager) | `tilepacks::tests`; APK offline QA |
| FR-15/16/17 | BG-2 | mobile OSM UI | `mobile/feature/map/` (search/routing/places), `mobile/feature/sharing/` | Kotlin build + manual mobile QA |
| FR-18 | BG-2 | desktop links | `src/map.rs` (OSM/directions/Organic Maps) | `map::tests::copyright_url_is_odbl_compliant` |
| FR-19 | BG-4 | docs | `docs/sdlc/`, `docs/components/`, `docs/uml/` | docs review |
| FR-20 | BG-4 | auto QA | `.github/workflows/map-qa.yml`, `scripts/qa.*` | CI `map-qa` green |
| FR-21 | BG-5 | licensing | `THIRD_PARTY_NOTICES.md`, `mobile/THIRD_PARTY_NOTICES.md` | `scripts/qa.sh` notices check |
| FR-22 | BG-4 | this file | this matrix | reviewer sign-off |
| FR-23 | BG-1 | **identity auth model** | `src/server/auth_handlers.rs`, `src/security/password.rs`, `docs/security/session-management.md` | `tests/contract.rs`; e2e login |
| FR-24 | BG-1 | **anonymous auth model** (ADR-0009) | `src/models.rs` (`auth_method`/`public_key`/`recovery_code_hash`), migration 14, `docs/security/dual-auth-model.md` | pending (`session-management.md` §10) |
| FR-25 | BG-1 | **server-side sessions + CSRF** | `pinpoint_sessions` (migration 14), `docs/security/session-management.md` | pending session tests |
| FR-26 | BG-1 | **session revocation** (logout/password change/refresh reuse) | session store + handlers (pending), ADR-0003 | pending |
| FR-27 | BG-1 | **MFA gate** | `src/security/mfa.rs`, `src/server/auth_handlers.rs`, `mfa_pending` column | `mfa::tests`; e2e MFA checks |
| FR-28 | BG-1 | **E2E location encryption + keychain** | `src/security/e2e.rs`, `src/security/keychain.rs`, `src/app.rs` | `e2e::tests`, `keychain::tests`; manual keychain QA |
| FR-29 | BG-3 | **GPS publish** | `src/platform.rs` (`GpsProvider`), `src/app.rs` (`publish_my_location`) | `platform::tests::parse_gps_fix_csv_and_json` |
| FR-30 | BG-4 | **dual mobile apps** | `mobile/` (Kotlin), `android/` + `ios/` + `src/mobile/` (Rust) | Kotlin build; CI (pending NDK/Xcode) |
| FR-31 | BG-1 | **OAuth2/OIDC + PKCE primitives** | `src/security/oauth.rs` (providers, S256, state, redirect allowlist) | `oauth::tests` (RFC 7636 vector, open-redirect) |
| FR-32 | BG-1 | **passkeys / magic links** | planned (ADR-0003 target; `auth-mechanisms.md` §4–5) | pending |
| FR-33 | BG-1 | **password reset** | documented gap (`auth-mechanisms.md` §7) | pending |
| FR-34 | BG-1 | **auth abuse controls** | lockout (`auth_handlers`), client rate limiter, scraping detection | `rate_limit`/`scrape` tests; server-side limits pending |
| NFR-01…18 | BG-3/4/5 | see `04-non-functional-requirements.md` | see `06/07/08` | see `11-qa-and-verification.md` |

## Deferred / out of scope

- Movement trails, heatmaps, friend mosaics, messaging/push, OAuth **login
  wiring**, passkeys, magic links — no implementation rows until their threat
  analysis lands (ADR-0009 covers the dual-auth design; OAuth/passkey
  endpoints remain future work).
- Bookmark encryption wiring (`EncryptedBookmarkStore`) is implemented as a
  primitive but not yet wired into `BookmarkStore`.

## Notes

- Rows FR-24…FR-26 are **design + schema landed, handlers pending**; the
  matrix marks them as pending evidence rather than claiming completion.
- The wire contract is pinned by `tests/contract.rs` (session cookie +
  `X-CSRF-Token`) and `tests/router_probe.rs` (routes).
