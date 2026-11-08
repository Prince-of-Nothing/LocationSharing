# android/ — Android client (Rust, `cargo-apk`)

Same `pinpoint_desktop` crate as desktop, packaged as an APK via
`cargo-apk` (`eframe`/`winit` → `aarch64-linux-android`).

```bash
rustup target add aarch64-linux-android
cargo install cargo-apk --locked
cargo apk build --locked
adb install target/aarch64-linux-android/debug/pinpoint-debug.apk
```

## Wiring

- `cargo-apk` reads `android/AndroidManifest.xml` (package
  `com.pinpoint.desktop`, `INTERNET` + fine/coarse location). The Rust
  entry is the default desktop binary; `cargo-apk` generates `MainActivity`
  (winit) automatically — no Kotlin/Java required.
- `android/network_security_config.xml` allows cleartext only for the
  emulator/dev hosts (`10.0.2.2`, `localhost`). Release builds must use
  HTTPS (ADR-0006); plain HTTP to any other host is rejected client-side
  by `api::validate_backend_url` and blocked here.
- Backend URL: `PINPOINT_BACKEND` env > `10.0.2.2:8000` (emulator) >
  `localhost:8000`. Real devices need `PINPOINT_BACKEND=https://…`.

## Status

Scaffolding only — cannot be built or run on this Windows host (no
Android NDK/SDK; the archived Kotlin stack lives under `archive/`).
Build it on a Linux/macOS host with the NDK, or in CI with the
`android` job (see `docs/client-platforms.md`).
