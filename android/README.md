# android/ — Android client (Rust, `cargo-apk`, modular)

Same `pinpoint_desktop` crate as desktop/web, packaged as an APK via
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
  entry/activity wiring is provided by the target stack; shared bootstrap logic
  lives in `src/mobile/mod.rs` so Android/iOS don't fork app code.
- `android/network_security_config.xml` allows cleartext only for the
  emulator/dev hosts (`10.0.2.2`, `localhost`). Release builds must use
  HTTPS (ADR-0006); plain HTTP to any other host is rejected client-side by
  backend adapter transport policy (`api::validate_backend_url`) and blocked
  here.
- Backend URL/config: `PINPOINT_BACKEND` env > `10.0.2.2:8000` (emulator) >
  platform default. Real devices need `PINPOINT_BACKEND=https://…`.

## Modularity / backend changes

- Mobile uses `crate::backend::{Backend, BackendConfig, DynBackend}` through
  `src/mobile` and shared `app`/`ui` code. To adapt to changed backend
  architecture, add/select a `BackendKind` adapter and keep mobile UI/platform
  glue unchanged.
- Platform-specific code belongs in `src/platform.rs` and `src/mobile/mod.rs`;
  do not embed endpoint/auth assumptions in Android UI/activity code.

## Status

Scaffolding — cannot be built or run on this Windows host (no Android NDK/SDK;
archived Kotlin stack is under `archive/`). Build on a Linux/macOS host with
the NDK, or in CI (see `docs/client-platforms.md`).
