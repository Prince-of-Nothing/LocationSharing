# Pinpoint 1.1.0 — Release Notes

**Released:** 2026-11-13 · **Version:** `1.1.0` (from `1.0.0`) ·
**Status:** thesis demo — navigation + operations surface complete, identity
privacy and diagnostics still in progress.

**Artifact:** `dist/pinpoint-1.1.0-windows-x64.zip` — server + desktop +
split services + web assets + docs. SHA-256 in `dist/SHA256SUMS.txt` and
inside the archive.

Flags: `[FEATURE]` new capability · `[FIX]` bug fix · `[SECURITY]` hardening ·
`[DOCS]` documentation · `[INFRA]` build/deploy.

---

## 1. What is functional now

| Area | State | What works |
|---|---|---|
| **Auth & sessions** | ✅ | Email/password (Argon2id), opaque hashed sessions, CSRF, logout/logout-all, session list/revoke, MFA, password change, lockout |
| **Anonymous accounts** | ✅ | Create device-bound account, one-time recovery code, recovery rotation |
| **Web navigation** | ✅ | Leaflet map (local), place search + reverse, place sheet, A→B + multi-stop routes, alternatives, steps, navigate preview with live next-stop, GPX/KML/GeoJSON, route history, saved-place filters, map appearance, locate-me |
| **Route interop** | ✅ | Google Maps + Organic Maps links, Magic Earth `magicearth://` links, KML/GeoJSON/GPX import files |
| **Per-friend sharing** | ✅ | On/off, exact/approximate/city, 1h–7d/forever, immediate stop, real expiry |
| **Admin console** | ✅ | Role-gated modular tab: overview, all accounts (incl. anonymous/never signed in), role/active changes, safe purge revoking sessions |
| **Diagnostics service** | ❌ Pending | Designed (`pinpoint-diag` 8005, report table); admin module placeholder only |
| **Usernames/privacy** | ❌ Pending | Anonymous users still cannot be added by username; email/full-name search remains internal |
| **Alice/Bob containers** | ❌ Pending | Still server-side seed data |
| **Desktop client** | ✅ | Map, OSM search/routing/POIs, bookmarks, check-ins, emergency, offline packs via backend |
| **Storage** | ⚠️ Partial | Schema/migrations 1–14, shared sessions; API users/locations still in-memory on demo path |
| **OSM self-host** | ⚠️ Config | `PINPOINT_*_URL` overrides wired; public providers are demo fallbacks |
| **Mobile / WASM** | ⚠️ Scaffold | Android/iOS/WASM not built in this release |
| **Privacy** | ✅ | No trackers/IPs, local assets, explicit consent, self-host-first OSM |
| **Verification** | ✅ | fmt/clippy/build/test green; browser checks for auth, map, routing, sharing, admin |

### Known limitations (honest)

- No live traffic, weather, lane guidance, road signs or voice guidance.
- Walk/bike routing requires self-hosted OSRM profiles; avoid options require
  a self-hosted profile with exclusions.
- Navigation is a preview with live next-stop; no off-route auto-reroute.
- Anonymous friends need a username system (next release); diagnostics ports
  and audit table are designed but not yet built.
- Demo users are seeded; client containers are next.
- Android/iOS builds need NDK/JDK 21 and macOS/Xcode.

---

## 2. Change log (flagged)

### `[FEATURE]`
- Map-first web client with locally vendored Leaflet 1.9.4 (BSD-2).
- Place search/reverse, category chips, place sheet.
- A→B routing with alternatives, steps, multi-stop (`via`), avoid controls
  (self-hosted OSRM only), GPX/KML/GeoJSON export.
- Route progress A→stops→B, next-stop ETA from OSRM legs, live advancement
  within 60 m; draggable/collapsible route panel.
- Per-account route history and saved-place category filters.
- Map appearance: label/icon size persisted on device.
- Per-friend sharing controls with precision/duration and real expiry.
- Modular role-gated admin console: overview, users, role/active, purge,
  diagnostics/audit module placeholders.
- Route/place hand-off: Google Maps, Organic Maps, Magic Earth.
- Local favicon, vendor assets, asset-locality guard.

### `[FIX]`
- CSRF token re-armed after reload (`/auth/me` header).
- Session IDs stored as SHA-256, not plaintext.
- Tile cache key truncation, provider UA/ordering, watermark fallback.
- Share expiry uses real UTC clock (not the seed date).
- Navigation Start works when geolocation is blocked, with retry.
- “You” marker + accuracy circle after locating.
- Admin user list refreshes from Postgres; self-protection guards.
- First search-result click uses a delegated handler.
- Cross-account saved-place flash cleared on logout.

### `[SECURITY]`
- Argon2id; hashed sessions; strict CSP; local assets; admin role gates;
  query strings redacted from logs; OSM calls never carry user/friend data.

### `[DOCS]`
- ADR-0009…0012, session/auth/dual-auth/DB/OSM guides, admin console guide,
  API contract, roadmap, SDLC/UML updates, per-feature flags.

### `[INFRA]`
- 1.1.0 packaging and checksums; CI gate remains fmt/clippy/build/test/audit.

---

## 3. Run

```powershell
$env:BIND_ADDR = "127.0.0.1:8080"
.\pinpoint-server.exe
# http://127.0.0.1:8080/ · alice@example.com / DemoPass123!
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
- **7 asset + 10 contract + 12 router + unit tests** ✅
- Browser: map tiles, search, multi-stop, next-stop, sharing, admin users,
  route history, appearance, GPX/KML/GeoJSON downloads.

## 5. Next (1.2)

1. Usernames + public/private profiles; anonymous friend requests.
2. `pinpoint-diag` on 8005 + `pinpoint-admin` on 8006 + migration 15
   (`diagnostic_reports`, `admin_audit`, `users.created_at`), replacing the
   placeholder admin modules.
3. Alice/Bob as headless client containers.
4. Postgres write-through for API-created users/locations.
5. ZAP scan, Android build, final thesis report.
