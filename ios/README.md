# ios/ — iOS client (Rust, `cargo-xcodebuild`)

Same `pinpoint_desktop` crate as desktop, packaged via `cargo-xcodebuild`
(`eframe`/`winit` → `aarch64-apple-ios`).

```bash
rustup target add aarch64-apple-ios   # + sim target if needed
cargo install cargo-xcodebuild --locked
cargo xcodebuild --locked
```

## Wiring

- `ios/Info.plist` declares bundle ID `com.pinpoint.desktop`, the
  `NSLocationWhenInUseUsageDescription` / `…AlwaysAndWhenInUse…` purpose
  strings, and the app metadata. `cargo-xcodebuild` generates the Xcode
  project from `ios/Info.plist`; no Storyboard/XIB required.
- Backend URL: `$PINPOINT_BACKEND` > `localhost:8000` (simulator) — real
  devices need an HTTPS gateway URL (ADR-0006). Note: iOS sandboxes have
  no `~/.pinpoint` filesystem, so `platform::app_data_dir()` returns
  `None` and bookmarks degrade to in-memory only (see `docs/client-platforms.md`).

## Status

Scaffolding only — cannot be built on this Windows host (no Xcode/macOS;
requires macOS + Xcode 15+). Build on a Mac or in CI (`macos-latest`).
