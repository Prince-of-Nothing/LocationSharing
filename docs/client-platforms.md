# Pinpoint — Client Platforms (one Rust crate, every OS)

Every client is the same crate (`pinpoint-desktop`, `src/`) — the only
per-OS code lives in `src/platform.rs`. The table below is the honest
status as of this commit; **nothing in this repo has been GUI-tested on
Linux/macOS/Android/iOS/Web from a Windows host** — the desktop CI job
(`windows-latest`) plus manual first-run checks on each OS are the
verification path.

| Client | Entry | Build | Status |
|---|---|---|---|
| Windows desktop | `src/main.rs` (`pinpoint-desktop`) | `cargo build --locked` | Builds in CI; run: `cargo run -- --backend http://localhost:8000` |
| Linux desktop | same binary | `Dockerfile.client` (X11) + `packaging/linux/` (.deb/.desktop) | Image builds; GUI needs X11 host |
| macOS desktop | same binary | `cargo build --locked` + `packaging/macos/` (.app via `cargo-bundle`) | Needs macOS/Xcode — not built here |
| Android | same `src/` via `cargo-apk` | `cargo apk build --locked` (`android/`) | Scaffolding (manifest + NCSC); needs NDK/SDK |
| iOS | same `src/` via `cargo-xcodebuild` | `cargo xcodebuild --locked` (`ios/Info.plist`) | Scaffolding; needs macOS/Xcode |
| Web (Rust/WASM) | `src/bin/pinpoint-web.rs` + `web/` | `trunk serve --release` | Static shell + wasm hook; full egui UI pending (see below) |

## Shared behavior (all platforms)

- Map-first landing with offline demo pins; sign-in optional; sign-in
  screens gate Friends/Sharing/Admin/Password.
- Backend URL: `--backend` > `$PINPOINT_BACKEND` > `platform::default_backend_url()`
  (`http://localhost:8000` desktop, `http://10.0.2.2:8000` Android emulator).
  Plain HTTP off-loopback is refused client-side (`api::validate_backend_url`, ADR-0006).
- Transport: session cookie + `X-CSRF-Token` double-submit (`src/api.rs`).
- Privacy: friend coordinates are never sent to Nominatim/OSRM/Overpass
  (`src/osm.rs`); OSM hand-off links only; ODbL attribution always rendered.
- Storage: bookmarks/tilepacks under `~/.pinpoint` on desktop;
  `platform::app_data_dir()` is `None` on Android/iOS/Web → in-memory only.
  OS keychain for tokens is design-only (see `docs/security-architecture.md`).

## Per-platform build/verify

### Desktop (Windows/Linux/macOS)

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo build --locked && cargo test --locked
cargo run -- --backend http://localhost:8000
./target/debug/pinpoint-desktop --help
```

### Linux

- Native: deps in `Dockerfile.client` (xcb, libssl, fontconfig).
- Container: `docker compose --profile client run --rm client` (X11 host only).
- `.deb`: `cargo deb --locked` (`[package.metadata.deb]` + `packaging/linux/`).

### macOS

- `xcode-select --install` then `cargo build --locked`.
- `.app`: `cargo bundle --release --format app` (`[package.metadata.bundle]`).
- iOS: `cargo xcodebuild --locked` (`ios/Info.plist`).

### Android

```bash
rustup target add aarch64-linux-android
cargo install cargo-apk --locked
cargo apk build --locked
adb install target/aarch64-linux-android/debug/pinpoint-debug.apk
```

Cleartext is allowlisted only for `10.0.2.2`/`localhost` (`android/network_security_config.xml`).

### iOS

```bash
rustup target add aarch64-apple-ios
cargo install cargo-xcodebuild --locked
cargo xcodebuild --locked
```

### Web (Rust/WASM)

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
trunk serve --release   # http://localhost:8080
```

The static shell in `web/index.html` drives first paint today; the full
egui UI on wasm requires replacing the desktop `tokio` runtime with a
wasm-friendly spawn (`wasm_bindgen_futures`), moving bookmarks to
`localStorage`, and stubbing tilepacks — tracked as the next web step.
`src/bin/pinpoint-web.rs`'s WASM entry is the seam for that work.

## What can't be tested here

- No Linux X11 display, no macOS/Xcode, no Android NDK/SDK, no wasm
  toolchain in this session. CI covers Windows build + clippy + fmt +
  tests; the other matrices need `macos-latest` / `ubuntu-latest` /
  Android-SDK jobs, or a manual first-run on each OS above.
