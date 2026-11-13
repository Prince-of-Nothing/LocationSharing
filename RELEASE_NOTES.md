# Pinpoint 1.0.0 — Release Notes

**Released:** 2026-11-13 · **Version:** `1.0.0` (from `0.2.0`) ·
**Status:** thesis demo release — functionally complete for the demo,
not production-hardened.

**Artifact:** `dist/pinpoint-1.0.0-windows-x64.zip` (19 MB, server + desktop
+ auth/users/locations/map services + web assets + docs). SHA-256 is published
in `dist/SHA256SUMS.txt` and inside the archive (`SHA256SUMS.txt` covers every
file in the bundle).

**Flags used below:** `[FEATURE]` new capability · `[FIX]` bug fix ·
`[SECURITY]` hardening · `[DOCS]` documentation · `[INFRA]` build/deploy.

---

## 1. What is functional now

| Area | State | What works |
|---|---|---|
| **Auth & sessions** | ✅ Functional | Email/password login + registration, Argon2id hashing, opaque `HttpOnly` session cookie, CSRF double-submit, logout, logout-all, session list/revoke, sliding idle timeout, lockout after 5 failures, TOTP MFA, password change (revokes all sessions) |
| **Anonymous accounts** | ✅ Functional | Create device-bound account (no PII), one-time 160-bit recovery code shown once, recovery with rotation of the code |
| **Web UI** (`/`) | ✅ Functional | Sign in / register / anonymous / recover, Slippy map through the `/map/tiles` proxy, publish location, friends (request/accept/reject), shares (pause/resume), active-sessions card, sign out everywhere, ODbL attribution, strict CSP, local-only assets |
| **Web search & routing** | ❌ Not in 1.0.0 | No search box or A→B navigation yet; planned for 1.1 (the Rust OSM engine already exists) |
| **Desktop client** | ✅ Functional | Login/register/logout/password change, map with pan/zoom and on-disk tiles, OSM search/reverse, OSRM routing (drive/walk/bike, steps, GPX), Overpass POIs, bookmarks + GeoJSON, check-ins, emergency, MFA, profile, offline tile packs via the backend |
| **Consent model** | ✅ Functional | Directional shares, precision (`exact`/`approximate`/`city_only`), purpose, expiry, pause/revoke; friendship alone grants nothing; enforced server-side every request |
| **Storage** | ⚠️ Partial | Postgres schema + migrations 1–14 and shared session storage work; users/friendships/locations API writes still use the in-memory demo store |
| **OSM / maps** | ⚠️ Demo fallbacks | `/map/tiles` proxy works (OSM-DE primary, Wikimedia/CARTO fallbacks); self-hosting via `PINPOINT_TILE_URL` / `PINPOINT_NOMINATIM_URL` / `PINPOINT_OSRM_URL` / `PINPOINT_OVERPASS_URL`; public providers are demo-only |
| **Android / iOS** | ⚠️ Scaffold | `cargo-apk` / `cargo-xcodebuild` setup exists; builds blocked by missing NDK/JDK 21 and macOS/Xcode |
| **WASM web client** | ⚠️ Stub | Trunk entry compiles the portable core; full egui web UI not wired |
| **Privacy / GDPR** | ✅ Functional | No trackers, no analytics, no raw IP storage, no IP/WebRTC geolocation; local fonts/scripts/icons; ODbL attribution everywhere; data-minimising shares |
| **Operations** | ✅ Functional | Docker compose (gateway + auth/users/locations/map replicas), Postgres, pg_dump backups, failover backend, offline mutation queue, `/health` + `/api/v1/sync-status` |
| **Verification** | ✅ Green | `fmt`, `clippy -D warnings`, locked build, **159 unit + 10 contract + 7 router + 5 asset** tests; release binaries built |
| **ZAP / thesis report** | ❌ Pending | Active scan and final LaTeX report still to do |

### Known limitations (honest)

- The web client cannot search or navigate yet — the OSM engine is wired only
  to the desktop client.
- Alice/Bob are still seeded in the server store (`Store::seed_demo()`), not
  real client containers.
- API-created users/friendships/locations are not written through to Postgres
  on the demo path.
- OAuth/OIDC login, passkeys, magic links and password reset are designed but
  not implemented.
- No rate limiting on anonymous creation/recovery yet.
- Docker Desktop was not running when this artifact was packaged; compose
  services are defined but not exercised in this build.

---

## 2. Release change log

### `[FEATURE]` New features

- `[FEATURE]` **Real server-side sessions** — opaque 256-bit session tokens,
  `HttpOnly; SameSite=Lax` cookie, CSRF token, sliding idle timeout,
  immediate revocation (ADR-0012).
- `[FEATURE]` **Session device management** — `GET /auth/sessions`,
  `DELETE /auth/sessions/:id` (own sessions only), `POST /auth/logout-all`;
  the full token is never returned.
- `[FEATURE]` **Anonymous accounts** — `/auth/anonymous` +
  `/auth/anonymous/recover`, one-time 160-bit recovery code, rotation on use.
- `[FEATURE]` **Web UI served at `/`** — login/register/anonymous/recovery,
  map, publish, friends, shares, active sessions, strict CSP.
- `[FEATURE]` **Slippy map via a server tile proxy** — OSM-DE primary,
  Wikimedia/CARTO fallbacks, source-aware cache, retry, `X-Tile-Source`.
- `[FEATURE]` **Local-only web assets** — no Google Fonts/CDNs; local
  `/favicon.svg`; `system-ui` typography; CSP pins `font-src`/`script-src`/
  `connect-src` to `'self'`.
- `[FEATURE]` **Desktop offline packs via the backend** — `<backend>/map/tiles`
  is tried before any public provider.
- `[FEATURE]` **Argon2id password hashing** with transparent rehash of legacy
  PBKDF2 hashes at login.
- `[FEATURE]` **Full desktop OSM toolkit** — Nominatim search/reverse, OSRM
  routing with steps and GPX, Overpass POIs, bookmarks GeoJSON, offline tiles.
- `[FEATURE]` **E2E location encryption + OS keychain** — AES-256-GCM payloads,
  per-user key stored in Windows Credential Manager/macOS Keychain/Secret
  Service, wiped on logout.
- `[FEATURE]` **Reliability** — failover backend with circuit breakers,
  offline mutation queue and replay, health monitoring, replica freshness.
- `[FEATURE]` **79-language i18n** registry with auto-detect.
- `[FEATURE]` **Self-host-first OSM configuration** — tiles, Nominatim, OSRM
  and Overpass URLs are environment overrides (ADR-0011).
- `[FEATURE]` **Normalized encrypted database design** — ADR-0010, reference
  DDL, blind indexes, per-service roles, migration path.
- `[FEATURE]` **Anonymous → identity upgrade path** designed (ADR-0009);
  email identity is live, OAuth/passkeys pending.

### `[FIX]` Fixed issues

- `[FIX]` **CSRF lost after reload** — the page kept the `HttpOnly` cookie but
  had no CSRF token, so publish/revoke returned `403 CSRF_FAILED`. `GET
  /auth/me` now returns `X-CSRF-Token`; the client re-arms on any response.
- `[FIX]` **Session ids stored in plaintext** — a database dump could be
  replayed as cookies. Sessions are stored as SHA-256 hashes only.
- `[FIX]` **Tile cache key truncation** — `(u8,u8,u8)` wrapped x/y at z ≥ 9
  and could serve a neighbouring tile; now `(u32,u32,u32)`.
- `[FIX]` **Tile proxy 502s** — providers rejected the custom User-Agent;
  CARTO was skipped without a key, and the OpenFreeMap raster URL 404'd.
  Fixed ordering, UA, retry, source-aware cache; `/map/tiles` returns 200.
- `[FIX]` **Duplicate `/health` route panic** — the combined server panicked
  on startup because service routers each embedded `/health`; health is now
  added once per binary/router.
- `[FIX]` **Favicon 404** — a local SVG icon is served at `/favicon.svg` and
  `/favicon.ico`.
- `[FIX]` **Desktop tile packs called CDNs directly** — now proxied through
  the configured Pinpoint backend.

### `[SECURITY]` Hardening

- `[SECURITY]` Argon2id replaces PBKDF2 for new passwords (OWASP-aligned
  m=19456/t=2/p=1); bcrypt rejected for its 72-byte limit and a DoS advisory.
- `[SECURITY]` Session tokens and CSRF tokens are never logged; session rows
  store hashes.
- `[SECURITY]` Strict CSP (`default-src 'self'`, no inline styles/scripts,
  no remote fonts or connections), `X-Frame-Options: DENY`, `nosniff`,
  `no-referrer`, `no-store` on the UI.
- `[SECURITY]` Password change, logout-all and per-session revoke take effect
  on the next request (no JWT grace window).
- `[SECURITY]` No raw IP storage, no IP/WebRTC geolocation, no third-party
  analytics (documented in `PRIVACY.md` and the threat model).
- `[SECURITY]` `tests/assets.rs` fails the build if a third-party asset host
  or absolute subresource is added to the shipped UI.

### `[DOCS]` Documentation

- `[DOCS]` ADR-0012 (opaque sessions) and updated session/dual-auth/security
  docs; ADR-0009/0010/0011 status kept current.
- `[DOCS]` `ARCHITECTURE.md`, `docs/ROADMAP.md`, API contract, SDLC set,
  UML set, component docs, demo script and presentation speech updated to the
  Rust/Axum reality (no Go/FastAPI/JWT claims).
- `[DOCS]` Release notes (this file) with functional-state matrix and
  per-item flags.

### `[INFRA]` Build / deploy

- `[INFRA]` Version bumped `0.2.0 → 1.0.0`; `/health` reports the crate
  version; User-Agent strings derive from `CARGO_PKG_VERSION`.
- `[INFRA]` Release binaries for server + desktop packaged with docs and
  checksums.
- `[INFRA]` CI `rust` job (fmt/clippy/build/test/audit), `map-qa`, and release
  artifact job.

---

## 3. Run it

### Windows release artifact

```powershell
# from the unzipped dist/pinpoint-1.0.0-windows-x64 directory
$env:BIND_ADDR = "127.0.0.1:8080"
.\pinpoint-server.exe
# open http://127.0.0.1:8080/
# demo login: alice@example.com / DemoPass123!
```

`pinpoint-desktop.exe --backend http://127.0.0.1:8080` runs the native client.
Port 8000 is avoided in the packaged example because Docker often owns it.

### Docker (requires Docker Desktop running)

```bash
docker compose up --build -d
# gateway on http://127.0.0.1:8000/
```

### Self-hosted OSM

```bash
PINPOINT_TILE_URL=https://tiles.example.com/{z}/{x}/{y}.png
PINPOINT_NOMINATIM_URL=https://nominatim.example.com
PINPOINT_OSRM_URL=https://osrm.example.com
PINPOINT_OVERPASS_URL=https://overpass.example.com/api/interpreter
```

---

## 4. Verification evidence

- `cargo fmt --all -- --check` ✅
- `cargo clippy --all-targets -- -D warnings` ✅
- `cargo build --locked` ✅ / `cargo build --release --locked` ✅
- `cargo test --locked` ✅ — 159 unit, 10 contract, 7 router, 5 asset tests
- Browser-verified flow: login, map (15/15 tiles), publish, second session +
  revoke, logout-all, anonymous create + recovery with rotation, zero console
  errors, **zero external asset requests**.

## 5. Next release (1.1)

Web search + A→B navigation (drive/walk/bike) over the existing OSM engine,
a map-first UI redesign, saved places/recent searches, then Alice/Bob as real
client containers. See [`ROADMAP.md`](./ROADMAP.md).
