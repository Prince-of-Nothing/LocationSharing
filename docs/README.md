# Practice Status — October 2025

## Overview

This document summarizes the progress of the Always Together / Pinpoint project during October 2025 and confirms deployment readiness.

## Completed Milestones

- Go Postgres conformance (`9f35f127`): `nowUTC()` microsecond truncation, login-attempt email lowercase, `sql.NullFloat64` scan for lat/lon NULL→0, shared `rowScanner`
- Mobile CI fixes: MapLibre SDK (`org.maplibre.gl:android-sdk:11.5.1`), Kotlin signature fixes (`OfflineRegionObserver`, `OfflineRegionStatusCallback`), `JsonConverter.kt` hand-rolled, `AppClient.kt` fixed
- `render.yaml` created (Go + Python + Web + Redis; Supabase reference)
- Mobile `build.gradle.kts`: Render URL set; `core:network` dependency added
- `docs/PRACTICE_STATUS.md` (this file) created
- Markdown restyle: root `README.md` polished; `AGENTS.md` updated (6 co-authors, MapLibre gotcha, Android build note)

## Security Baseline

- Server-side enforcement only (no secrets in tests)
- AES-GCM session storage (Keystore-backed)
- Secure cookies; CSP tile-host allow-list (`maps.wikimedia.org`)
- No Firebase/FCM/analytics/crash SDKs (0 trackers)
- MapLibre Native BSD-2 (zero telemetry)

## Artifacts

- APK: `app-debug.apk` (51 MB) — local debug build
- Config: `render.yaml` (free Render tier + Supabase PostgreSQL reference)
- Status doc: `docs/PRACTICE_STATUS.md`
