# ios/ — iOS client (Rust, `cargo-xcodebuild`, modular)

Same `pinpoint_desktop` crate as desktop/web, packaged via `cargo-xcodebuild`
(`eframe`/`winit` → `aarch64-apple-ios`).

```bash
rustup target add aarch64-apple-ios   # + sim target if needed
cargo install cargo-xcodebuild --locked
cargo xcodebuild --locked
```

## Wiring

- `ios/Info.plist` declares bundle ID `com.pinpoint.desktop`, the
  `NSLocationWhenInUseUsageDescription` / `…AlwaysAndWhenInUse…` purpose
  strings, and app metadata. `cargo-xcodebuild` generates the Xcode project
  from `ios/Info.plist`; no Storyboard/XIB required.
- Backend URL/config: `$PINPOINT_BACKEND` > platform default. Real devices need
  an HTTPS gateway URL (ADR-0006). iOS sandboxes have no `~/.pinpoint`
  filesystem, so `platform::app_data_dir()` returns `None` and bookmarks degrade
  to in-memory only (see `docs/client-platforms.md`).
- Shared mobile bootstrap/backend config lives in `src/mobile/mod.rs`.

## Modularity / backend changes

- iOS uses the same `crate::backend::{Backend, BackendConfig, DynBackend}`
  abstraction as Android/desktop/web. Changed backend architectures are handled
  by backend adapters/config selection; iOS lifecycle/glue stays thin.

## Status

Scaffolding — cannot be built on this Windows host (no Xcode/macOS; requires
macOS + Xcode 15+). Build on a Mac or in CI (`macos-latest`).
