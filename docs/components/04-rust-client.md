# 04 — Rust Multiplatform Client (current)

> **Status:** ✅ **Current / live.** This is the primary client and the only
> multiplatform GUI. Dir: `src/` (crate `pinpoint-desktop`).

## Overview

One Rust crate targets **desktop (Windows/Linux/macOS)**, **Android**
(`cargo-apk`), **iOS** (`cargo-xcodebuild`) and **Web** (wasm via Trunk). The
portable core (`src/lib.rs`) holds all logic; `src/platform.rs` holds every
`cfg(target_os)` branch; `src/ui/` is the egui shell. It talks to the
`/api/v1` contract in `src/api.rs` and can swap backends via `DynBackend`.

## Stack

| Concern | Choice |
|---|---|
| Language | Rust 1.99 (edition 2021) |
| GUI | eframe / egui 0.32 (glow) |
| HTTP | reqwest 0.12 |
| Image | image 0.25 (png) |
| Crypto | aes-gcm, pbkdf2, hmac, sha2, rand, zeroize |
| OS keychain | keyring 4 (desktop) + FileKeychain fallback |
| Async (desktop) | tokio |
| Web | wasm-bindgen, web-sys, js-sys |

## Module map

```
src/
├── lib.rs              # module declarations (GUI modules cfg-gated off wasm)
├── main.rs             # desktop entry (--help, --backend, PINPOINT_BACKEND)
├── api.rs              # HTTP client: session cookie + X-CSRF-Token, URL policy
├── app.rs              # PinpointApp state, screens, mutations, E2E key wiring
├── models.rs           # domain models + pure math (haversine, bearing, freshness)
├── platform.rs         # all cfg(target_os) branches + GpsProvider + app_data_dir
├── backend/            # DynBackend trait, REST adapter, failover, health
├── ui.rs + ui/         # egui shell + 10 per-screen modules
├── map.rs              # interactive Slippy canvas + pins + ODbL hand-off
├── tilepacks.rs        # offline tile packs (multi-source ODbL, Slippy math)
├── osm.rs              # Nominatim / OSRM / Overpass (friend coords never sent)
├── bookmarks.rs        # on-device JSON bookmarks + GeoJSON export/import
├── i18n.rs             # 79-language registry + t() + auto-detect
├── security/           # validation, rate_limit, encryption, audit, session,
│                       #   signing, mfa, password, oauth, scrape, e2e,
│                       #   keychain, circuit_breaker
├── patterns/           # Repository, Strategy, Command, State, Observer,
│                       #   Factory, Decorator, Proxy
├── server/             # Rust/Axum backend (component 03)
├── webapp/             # portable web clients (admin, kiosk, view-model)
├── mobile/             # Android/iOS bootstrap + deep links + lifecycle
└── bin/                # pinpoint-{desktop,server,auth,users,locations,map,web,weba,webk}
```

## Screens (egui)

`CurrentScreen` in `src/app.rs`; rendering in `src/ui/`:

| Screen | File | Purpose |
|---|---|---|
| Map | `ui/map.rs` | Slippy canvas, publish (manual + GPS), friends, shares, search |
| Login / Register | `ui/auth.rs` | sign-in / sign-up |
| Friends | `ui/friends.rs` | requests, accept/reject |
| Sharing | `ui/sharing.rs` | directional shares, expiry, pause/resume |
| Places | `ui/places.rs` | OSRM routing, Overpass POI, reverse geocode |
| Bookmarks | `ui/bookmarks.rs` | saved places + GeoJSON export |
| Trips | `ui/trips.rs` | check-ins |
| Emergency | `ui/emergency.rs` | emergency alerts |
| Profile | `ui/profile.rs` | profile, units, MFA enrolment |
| Admin | `ui/admin.rs` | user management |
| ChangePassword | `ui/auth.rs` | forced/regular password change |

## Features

- **Auth:** login/register/logout, session cookie + CSRF, forced password
  change, MFA enrolment.
- **Location sharing:** directional shares with precision (`exact` /
  `approximate` / `city_only`), purpose/label, expiry, pause/resume/revoke.
- **Map:** interactive Slippy canvas (pan/zoom, cached tiles, precision rings),
  OSM hand-off links, ODbL attribution.
- **Offline:** tile packs (multi-source ODbL), on-device bookmarks (GeoJSON),
  `PendingMutation` queue with auto-replay.
- **Redundancy:** `FailoverBackend` with circuit breakers, health monitoring,
  exponential backoff.
- **Security:** E2E location encryption (AES-256-GCM) + OS keychain persistence;
  Argon2id passwords; rate limiting; audit logging; input validation.
- **i18n:** 79 languages with auto-detect + picker.
- **GPS:** `GpsProvider` (env/file) + "Use GPS" publish.

## Data flow

```
UI (egui)  ──► PinpointApp ──► DynBackend (REST adapter) ──► /api/v1
                 │                                            │
                 ├─ AppData (locations, shares, friendships)  ◄┘
                 ├─ KeyManager (E2E key) ──► OsKeychain / FileKeychain
                 ├─ OsmClient (Nominatim/OSRM/Overpass)  ◄── user-initiated only
                 └─ Tilepacks / Bookmarks (on-device)
```

## Security controls

- **Passwords:** Argon2id (PHC) (`src/security/password.rs`).
- **Sessions/CSRF:** session cookie + `X-CSRF-Token` double-submit.
- **E2E:** `src/security/e2e.rs` AES-256-GCM with PBKDF2-derived per-user keys;
  key persisted in the OS keychain (`src/security/keychain.rs`).
- **Keychain:** `OsKeychain` (keyring: Windows Credential Manager / macOS
  Keychain / Linux Secret Service) + `FileKeychain` fallback.
- **Rate limiting / validation / audit:** `src/security/{rate_limit,validation,audit}.rs`.
- **Fail-closed:** `src/security/session.rs`, `signing.rs`, `oauth.rs`,
  `scrape.rs`.
- **Transport policy:** plain HTTP only for loopback (ADR-0006).

## Authentication & sessions

- **Identity model:** email+password (Argon2id) via `src/api.rs`; the client
  captures the session cookie + `X-CSRF-Token` from login/register and
  attaches them on every request.
- **Anonymous model:** device key stored in the OS keychain
  (`src/security/keychain.rs`) + recovery code; same session/CSRF machinery
  (see `docs/security/dual-auth-model.md`).
- **Session refresh:** access token is short-lived; the client refreshes on
  401 (pending) and drops local state on confirmed logout or password change.
- **Status:** cookies/CSRF plumbing exists; the server-side session store and
  anonymous endpoints are pending (`session-management.md` §10).

## Build / run / test

```bash
cargo build --locked
cargo run -- --backend http://localhost:8000
cargo run --bin pinpoint-server          # backend (component 03)
cargo test --locked                      # 153 unit + 10 contract + 5 router
```

Mobile/web:

```bash
rustup target add aarch64-linux-android aarch64-apple-ios wasm32-unknown-unknown
cargo apk build --locked                 # Android (needs NDK)
cargo xcodebuild --locked                # iOS (needs macOS/Xcode)
trunk build --config web/Trunk.toml      # Web (wasm)
```

## Deployment

- Desktop: `cargo build --release` + `packaging/linux` (.deb) / `packaging/macos`
  (.app); CI `release` job uploads a Windows zip.
- Android: `android/` manifest + network security config; `cargo-apk`.
- iOS: `ios/Info.plist`; `cargo-xcodebuild`.
- Web: `web/` + Trunk; CI `wasm` job compile-checks the bins.

## Relationship to other components

- **Consumes:** component 3 (Rust/Axum backend) via `/api/v1`.
- **Shares code with:** component 3 (`src/models.rs`, `src/api.rs`,
  `src/security/*`).
- **Parallel to:** component 5 (Kotlin Android app) — same contract, different
  UI stack.
- **Successor to:** component 6 (archived vanilla-JS web client).
