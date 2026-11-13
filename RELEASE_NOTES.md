# Pinpoint 1.2.0 — Release Notes

**Released:** 2026-11-13 · **Version:** `1.2.0` (from `1.1.0`) ·
**Status:** thesis demo — navigation, identity privacy, diagnostics and demo
clients are in place.

**Artifact:** `dist/pinpoint-1.2.0-windows-x64.zip` — combined server, desktop,
auth/users/locations/map/diag/admin services, the headless demo client, web
assets and docs. SHA-256 in `dist/SHA256SUMS.txt` and inside the archive.

Flags: `[FEATURE]` new capability · `[FIX]` bug fix · `[SECURITY]` hardening ·
`[DOCS]` documentation · `[INFRA]` build/deploy.

---

## 1. What is functional now

| Area | State | What works |
|---|---|---|
| **Auth & sessions** | ✅ | Email/password (Argon2id), opaque hashed sessions, CSRF, logout/logout-all, session list/revoke, MFA, password change, lockout |
| **Anonymous accounts** | ✅ | Create device-bound account, one-time recovery code, recovery rotation |
| **Usernames & privacy** | ✅ | Stable public handle (`calm-otter-42`), exact-match lookup, friend requests by handle, pending requests show handles only, names/emails revealed to accepted friends, numeric IDs not walkable, free-text search admin-only |
| **Web navigation** | ✅ | Leaflet map (local), place search + reverse, place sheet, A→B + multi-stop routes, alternatives, steps, navigate preview with live next-stop, GPX/KML/GeoJSON, route history, saved-place filters, map appearance, locate-me |
| **Route interop** | ✅ | Google Maps + Organic Maps links, Magic Earth `magicearth://` links, KML/GeoJSON/GPX import files |
| **Per-friend sharing** | ✅ | On/off, exact/approximate/city, 1h–7d/forever, immediate stop, real expiry |
| **Admin console** | ✅ | Role-gated modular tab: overview, all accounts (incl. anonymous/never signed in), role/active changes, safe purge revoking sessions |
| **Diagnostics** | ✅ | `pinpoint-diag` on **8005**, PII-free reports (category + message + public handle only), admin triage, no IP/UA/coordinates stored |
| **Admin API** | ✅ | `pinpoint-admin` on **8006**, same fail-closed role gates; report triage + append-only audit |
| **Audit trail** | ✅ | Role/active changes, account deletions and report triage in `pinpoint_admin_audit`; ids/handles only, no email or coordinates |
| **Alice/Bob containers** | ✅ | `pinpoint-demo-client` logs in through the API and publishes real locations on an interval; compose profile `demo` |
| **Desktop client** | ✅ | Map, OSM search/routing/POIs, bookmarks, check-ins, emergency, offline packs; shows the user's public handle and accepts handles or emails for friend requests |
| **Storage** | ⚠️ Partial | Schema/migrations 1–15; Postgres write-through paths wired; live Postgres run pending (Docker daemon unavailable in this environment) |
| **OSM self-host** | ⚠️ Config | `PINPOINT_*_URL` overrides wired; public providers are demo fallbacks |
| **Mobile / WASM** | ⚠️ Scaffold | Android/iOS/WASM not built in this release |
| **Privacy** | ✅ | No trackers/IPs, local assets, explicit consent, self-host-first OSM, user-initiated diagnostics |
| **Verification** | ✅ | fmt/clippy/build/test green; browser checks for identity, report → triage → audit; 8005/8006 ports; demo-client login/publish |

### Known limitations (honest)

- No live traffic, weather, lane guidance, road signs or voice guidance.
- Walk/bike routing requires self-hosted OSRM profiles; avoid options require
  a self-hosted profile with exclusions.
- Navigation is a preview with live next-stop; no off-route auto-reroute.
- Postgres persistence is implemented but was not exercised against a live
  database in this environment; the in-memory demo path is verified.
- ZAP scan, Android APK (NDK/JDK 21) and iOS (macOS/Xcode) remain external.
- Compose config is validated; images were not built here because the Docker
  daemon was not running.

---

## 2. Change log (flagged)

### `[FEATURE]`
- Generated public usernames + public/friend/private profile split.
- Friend requests by exact handle (legacy exact email fallback for older
  clients); pending requests no longer leak legal names.
- Step-up admin actions: `pinpoint-diag` 8005, `pinpoint-admin` 8006,
  `/api/v1/diagnostics/report`, admin diagnostics triage, `/api/v1/admin/audit`.
- Migration 15: `pinpoint_diagnostic_reports`, append-only
  `pinpoint_admin_audit`, `users.created_at`, report/audit counters.
- Real Diagnostics and Audit admin modules replacing the placeholders.
- Help & diagnostics report form in the web client (category + message only).
- Headless `pinpoint-demo-client` for Alice/Bob containers; compose profile
  `demo`.
- Explicit `PINPOINT_ALLOW_PLAINTEXT_HOSTS` exact-host opt-in for internal
  container DNS (HTTPS remains the default).
- GPX/KML/GeoJSON route export and Google/Organic/Magic Earth hand-off.

### `[FIX]`
- Location publish and emergency alerts now use the real UTC clock instead of
  the seed timestamp; check-ins and friend requests do the same.
- Duplicate friend requests are rejected instead of stacking rows.
- Accepted-friend vs pending vs unrelated profile responses are now distinct.
- Admin deletion records an audit row and removes the account without putting
  its email in the audit trail.

### `[SECURITY]`
- Argon2id; hashed sessions; strict CSP; local assets; admin role gates.
- No raw IP/UA storage; diagnostics explicitly exclude location and device
  identifiers; audit is ids/handles only; OSM calls never carry user/friend
  data.
- Numeric user IDs are not walkable; non-admin user search is exact-handle
  only.

### `[DOCS]`
- Dual-auth model updated with the public-identity matrix and lookup rules.
- Admin console guide now documents the real modules, 8005/8006 split and the
  append-only audit guarantees.
- Client-platforms guide documents the headless demo clients and plaintext
  opt-in.

### `[INFRA]`
- New binaries: `pinpoint-diag`, `pinpoint-admin`, `pinpoint-demo-client`.
- Compose: diag/admin services + gateway routes + `demo` profile for Alice/Bob.
- Per-service grants: diag gets report INSERT/SELECT only; admin reuses the
  users role with sessions/report/audit grants.

---

## 3. Run

Combined demo (web UI + all APIs):

```powershell
$env:BIND_ADDR = "127.0.0.1:8080"
.\pinpoint-server.exe
# http://127.0.0.1:8080/ · alice@example.com / DemoPass123!
# admin: admin@example.com / ChangeMe123!
```

Split ports:

```powershell
$env:BIND_ADDR = "127.0.0.1:8005"; .\pinpoint-diag.exe
$env:BIND_ADDR = "127.0.0.1:8006"; .\pinpoint-admin.exe
```

Alice/Bob as real clients:

```powershell
$env:PINPOINT_BACKEND = "http://127.0.0.1:8080"
$env:PINPOINT_EMAIL = "alice@example.com"
$env:PINPOINT_PASSWORD = "DemoPass123!"
$env:PINPOINT_LAT = "47.0105"
$env:PINPOINT_LON = "28.8638"
.\pinpoint-demo-client.exe
```

Docker compose:

```powershell
docker compose up --build -d
docker compose --profile demo up -d alice bob
# gateway :8000 · diagnostics :8005 · admin API :8006
```

Desktop: `pinpoint-desktop.exe --backend http://127.0.0.1:8080`.

Self-hosted OSM:

```powershell
$env:PINPOINT_TILE_URL = "https://tiles.example.com/{z}/{x}/{y}.png"
$env:PINPOINT_NOMINATIM_URL = "https://nominatim.example.com"
$env:PINPOINT_OSRM_URL = "https://osrm.example.com"
$env:PINPOINT_OVERPASS_URL = "https://overpass.example.com/api/interpreter"
```

## 4. Verification

- fmt ✅ · clippy `-D warnings` ✅ · build ✅ · release build ✅
- **174 unit + 7 asset + 10 contract + 14 router tests** ✅
- Browser: username display, request by handle, pending/accept, duplicate and
  unknown-handle states; report → triage → audit in the admin console.
- Ports: 8005 ingestion 201, invalid category 400, admin route isolated 404;
  8006 health 200, unauthenticated admin 401.
- Demo client: logged in as `@bright-badger-2`, published through
  `/locations/me`, and a friend read back the fresh `updated_at`.
- Compose: `docker compose config --quiet` ✅, 11 services including
  diag/admin/alice/bob.

## 5. Next

1. Live Postgres verification of migrations 1–15 + write-through.
2. ZAP baseline scan and fixes.
3. Android APK (NDK + JDK 21) and iOS build on macOS.
4. Final thesis report / demo script.
