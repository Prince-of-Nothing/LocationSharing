# 05 — Kotlin Android App (current)

> **Status:** ✅ **Current.** The Android GUI alongside the Rust client.
> Dir: `mobile/`.

## Overview

A native Android app built with Kotlin + Jetpack Compose, using Clean
Architecture (`:core:domain` → `:core:data` → `:feature:*`) and MapLibre for
maps. It talks to the same `/api/v1` contract as the Rust client and adds
on-device offline map packs, Nominatim search, OSRM routing, Overpass POI and
DataStore bookmarks. Session tokens are encrypted with an **Android Keystore**
key before touching disk.

## Stack

| Concern | Choice |
|---|---|
| Language | Kotlin 2.0.21 |
| Build | Gradle 8.14, AGP 8.7.2 |
| JDK | 17 (target) — **not** JDK 25 (Gradle 8.14 limit) |
| SDK | compileSdk 35, minSdk 26, targetSdk 35 |
| UI | Jetpack Compose (BOM 2024.10.01), Material3, Navigation Compose 2.8.4 |
| Maps | MapLibre Native Android 11.5.1 (`org.maplibre.gl:android-sdk`) |
| Network | Retrofit 2.11 + OkHttp 4.12 + kotlinx-serialization 1.7.3 |
| Storage | DataStore 1.1.1 (encrypted via Android Keystore) |
| Async | Kotlin Coroutines 1.9.0 |
| Test | JUnit 4.13.2, Truth 1.4.4 |

## Module map

```
mobile/
├── app/                          # composition root
│   ├── src/main/java/org/pinpoint/mobile/
│   │   ├── MainActivity.kt       # NavHost + bottom navigation + crash fallback
│   │   ├── MobileApp.kt          # Application: builds the Container
│   │   ├── di/Container.kt       # manual DI (no Hilt)
│   │   ├── CrashLogger.kt        # log scrubbing (no tokens/locations)
│   │   └── CrashReport.kt        # shareable crash file
│   ├── build.gradle.kts          # applicationId org.pinpoint.mobile
│   └── proguard-rules.pro
├── core/
│   ├── domain/                   # pure Kotlin: models, repository interfaces,
│   │                             #   RoutingEngine, LocationProvider, Places
│   ├── network/                  # Retrofit/OkHttp only: ApiClient, ApiServices,
│   │                             #   OsmServices, DTOs, JsonConverter
│   ├── data/                     # repository impls + Keystore SessionStore
│   └── map/                      # MapLibre only: MapLibreIntegration,
│                                 #   OfflineMapManager, MapLayers, MapConfig,
│                                 #   GeoNavigation, DefaultRoutingEngine
├── feature/
│   ├── auth/                     # LoginScreen + AuthViewModel
│   ├── map/                      # MapScreen, SearchScreen, RoutePlannerScreen,
│   │                             #   PlacesScreen, SafetyAlerts, RouteCalculator
│   ├── friends/                  # FriendsScreen + FriendsViewModel
│   └── sharing/                  # SharingScreen + SharingViewModel
├── gradle/libs.versions.toml     # version catalog
└── gradle/wrapper/               # Gradle 8.14
```

**Layering rules (enforced by module boundaries):**
- `:feature:*` → `:core:domain` only (screens never see HTTP/storage/map SDKs).
- `:core:data` implements domain interfaces over `:core:network` + Keystore.
- `:core:network` is the only module importing Retrofit/OkHttp.
- `:core:map` is the only module importing MapLibre.

## Screens & navigation

`MainActivity` hosts a `NavHost` with a bottom `NavigationBar`:

| Route | Screen | ViewModel |
|---|---|---|
| `splash` | session restore | – |
| `login` | login | `AuthViewModel` |
| `map` | MapLibre map + offline packs | `MapViewModel` |
| `search` | Nominatim search | `SearchViewModel` |
| `route` | OSRM route planner | `RouteViewModel` |
| `places` | Overpass POI + bookmarks | `PlacesViewModel` |
| `friends` | friend requests / circle | `FriendsViewModel` |
| `sharing` | per-friend share consent | `SharingViewModel` |

## Features

- **Login + session restore:** encrypted tokens survive restarts; logout wipes
  memory + disk.
- **Map:** MapLibre with OpenFreeMap vector styles; offline packs
  (download/list/delete) via `OfflineMapManager`.
- **Search:** Nominatim (user-initiated).
- **Routing:** OSRM online + straight-line offline fallback
  (`DefaultRoutingEngine`).
- **POI:** Overpass browse (user-initiated).
- **Bookmarks:** on-device DataStore JSON (no sync, no friend data).
- **Friends / Sharing:** friend requests and per-friend location-share consent.
- **Safety alerts:** emergency UI (`SafetyAlerts.kt`).
- **Crash reporting:** local crash file + share button (no Firebase/Crashlytics).

## Security controls

- **Session storage:** AES-GCM with an **Android Keystore** key
  (`SessionStore`); key never leaves the platform (hardware-backed when
  available). Plaintext tokens live in memory only.
- **Auth models:** identity (email/password) and anonymous device-key accounts
  share the same session contract (`docs/security/session-management.md`).
  Anonymous keys/recovery codes live in the Keystore-backed store; refresh on
  401 is pending.
- **Transport:** Bearer token + `X-CSRF-Token` double-submit on
  POST/PATCH/DELETE (`ApiClient` auth interceptor).
- **Log scrubbing:** no tokens, passwords or locations in logcat
  (`CrashLogger`, `HttpLoggingInterceptor` at BASIC).
- **0 trackers:** no analytics/crash SDKs.
- **Privacy:** friend coordinates never sent to OSM services; only
  user-initiated queries.

## Configuration

- `BuildConfig.API_BASE_URL` — debug points at the deployed API; release
  injects the production URL at build time (never committed).
- `core/map/MapConfig.kt` — map style/attribution defaults.
- `network_security_config.xml` — cleartext allowlist for dev hosts.

## Build / run / test

```bash
cd mobile
./gradlew :app:assembleDebug          # debug APK
./gradlew :core:domain:test :core:network:test
adb install app/build/outputs/apk/debug/app-debug.apk
```

Requirements: **JDK 17 or 21** (not 25), Android SDK 35, Gradle 8.10+.

## Deployment

- Debug APK: `app/build/outputs/apk/debug/app-debug.apk`.
- Release: signed APK/AAB with injected production API URL.
- Attribution: OSM/ODbL + MapLibre BSD-2 in `THIRD_PARTY_NOTICES.md`.

## Relationship to other components

- **Consumes:** component 3 (Rust/Axum backend) via `/api/v1`.
- **Parallel to:** component 4 (Rust client) — same contract, native Android UI.
- **Distinct from:** the archived Kotlin app (same name/package, pre-Rust
  version) now in `archive/mobile/`.
