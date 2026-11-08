# ADR-0007: Rust Multiplatform as the Sole Application Codebase

Date: 2026-11-08
Status: Accepted
Supersedes: Go/Python dual-backend + Kotlin mobile + vanilla-JS web (archived)

## Context

Pinpoint shipped four stacks over one wire contract: Go backend, Python
backend, vanilla-JS web, Kotlin Android (plus the Rust desktop crate).
Maintenance cost dominated: every consent/tile/JS rule had to be mirrored
by hand (mobile module rules, `sw.js` SHELL list, dual CSPs, two notice
files), and the map parity work (Slippy/Nominatim/OSRM/Overpass/bookmarks)
existed in three different languages.

The team wants maximum of the app in Rust — C/C++-class performance and
memory safety — with one codebase across desktop, Android and iOS.

## Decision

Rust is the sole application codebase. The root crate (`pinpoint-desktop`
→ multiplatform client) serves desktop (`eframe::run_native`), Android
(`cargo-apk` + `winit`, `aarch64-linux-android`) and iOS
(`cargo-xcodebuild`, `aarch64-apple-ios`) from `src/` via
`cfg(target_os)` (see `src/platform.rs`). All non-Rust implementations
move to `archive/` and stop building.

- OSM search/routing/POI are reimplemented in pure Rust (`src/osm.rs`,
  `reqwest` + `serde` only — zero new deps) replacing the archived web
  `geocode/routing/places.js` and Kotlin `OsmServices.kt`.
- Bookmarks are on-device JSON (`src/bookmarks.rs`) replacing
  `js/bookmarks.js` (localStorage) and Kotlin DataStore.
- Map hand-off stays OSM-only (OSM standard, OSM directions, Organic Maps;
  ODbL credit in `src/map.rs`). No bulk tile caching in-app; offline UX
  comes from OS-level packs / user downloads, not a bundled engine.

## Consequences

- One CI gate (Rust fmt/clippy/build/test/audit); archived jobs deleted.
- Android/iOS need Rust NDK/Xcode toolchains instead of Gradle/SDK.
- Archived stacks remain readable under `archive/` (reference only).
- Follow-up: background location + offline pack manager behind
  `src/platform.rs` traits; binary size / startup measured per release.
