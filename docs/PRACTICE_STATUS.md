# Practice Status — October 2025 → Current

**Prepared for project progress documentation.**

| Period | Key Milestone | Status |
|---|---|---|
| 26–30 Sept 2025 | MVP baseline defined (Go + Python APIs, Android, web clients) | ✅ Completed |
| Early Oct | Mobile CI fixed (MapLibre SDK `org.maplibre.gl:android-sdk:11.5.1`) | ✅ Resolved |
| Mid Oct | Kotlin offline-manager signature drift (`Array<OfflineRegion>?`, nullable callbacks) fixed; `OfflineMapManager.kt` compiles; `SigProbe.kt` removed; `destroy()` removed; `bytesRemaining` → `completedBytes` | ✅ Committed |
| Mid Oct | Hand-rolled `JsonConverterFactory` (replaces jakewharton `retrofit2-kotlinx-serialization-converter` 1.0.0 whose `Factory`/`Serializer` are `internal`) created; `core:network/build.gradle.kts` cleaned; `ApiClient.kt` fixed; `ApiServices.kt` `FriendsLocationsResponse` import added | ✅ Compiles |
| Late Oct | `:core:data` `SessionStore.login()` → `SessionStore.save()` (satisfies `SessionRepository` contract); `:core:map` observer interfaces corrected (`onStatus(OfflineRegionStatus?)`, `onError(String?)`, `onList(Array<OfflineRegion>?)`) | ✅ `BUILD SUCCESSFUL` (51s) |
| Late Oct | `mobile/app/build.gradle.kts` updated: `core:network` added to app dependencies; `API_BASE_URL` updated to Render; `render.yaml` created (Go API + Python API + Web Client + Redis); `docs/practice/` structure prepared | ✅ Deployed-ready |
| Current | `docs/PRACTICE_STATUS.md` written; `render.yaml` available; `app-debug.apk` (51 MB) exported locally; `AGENTS.md` owner-only commit convention in force; ADRs immutable (formatting only); docs conventions (Mermaid, Diátaxis) maintained | ✅ Ready for share |

## Live Artifacts

- **APK (local debug):** `C:\Users\Unknown\Documents\Pinpoint\app-debug.apk` (51 MB)
- **Render config:** `render.yaml` (Go + Python services, static web, Redis; Supabase PostgreSQL reference)
- **Mobile build config:** `mobile/app/build.gradle.kts` (Render URL; `core:network` dependency)
- **Backend fixes (committed):** `9f35f127` (Go Postgres conformance: `nowUTC()` microsecond truncation; login-attempt email lowercase; `sql.NullFloat64` for lat/lon NULL→0; shared `rowScanner`/`scanShareRow`)
- **Mobile fixes (committed):** MapLibre coordinate (`org.maplibre.gl:android-sdk:11.5.1`); `MapLibreMap` casing; `OfflineMapManager` observer interface; `ApiClient` converter; `SessionStore` contract

## Security Baseline (Unchanged)

- **Server-side enforcement only** — no secrets or real user data in tests (`CONTRIBUTING.md`).
- **No Firebase / FCM / analytics / crash SDKs ever** (`mobile/THIRD_PARTY_NOTICES.md`).
- **AES-GCM session encryption** (Keystore-backed; `SessionStore`).
- **Secure cookies**; CSP allow-list for tile host (`maps.wikimedia.org`).
- **0 trackers** (MapLibre Native BSD-2, zero telemetry).

## Pending for Final Verification

- `go build ./... && go vet ./...` on committed `9f35f127` (CI only; no local Postgres binary; `C:\Program Files\PostgreSQL\18` has only `data` dir; no Docker on machine).
- `docker compose up --build` verification (Docker unavailable locally; CI job handles full-stack demo).
- Push `9f35f127` + mobile commits so `backend-go` + `mobile` CI jobs go green.

## Deployment Path (Free Tier)

1. **Render** (free web + static + Redis): deploy `render.yaml`.
2. **Supabase** (free PostgreSQL 500 MB): set `DATABASE_URL` in Render dashboard.
3. **Mobile** (`build.gradle.kts`): `API_BASE_URL` points at Render service URL.
4. **Web CSP** (`nginx.conf` / `render.yaml` headers): allow `maps.wikimedia.org` in `img-src`; keep ODbL attribution visible.
5. No secrets committed; session secret generated (`openssl rand -base64 32`) and set via Render dashboard only.
