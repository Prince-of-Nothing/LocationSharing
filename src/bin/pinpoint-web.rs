//! Pinpoint Web client (Rust/WASM, `eframe` via `trunk`).
//!
//! This is the web entry point for the same `pinpoint_desktop` crate that
//! serves desktop (Windows/Linux/macOS) and mobile (Android/iOS). Build with:
//!
//! ```bash
//! rustup target add wasm32-unknown-unknown
//! cargo install trunk --locked
//! trunk build --release        # serves web/index.html + pinpoint_web.wasm
//! trunk serve --release        # http://localhost:8080
//! ```
//!
//! Status: the native shell below is a stub so `cargo build --locked`
//! (Windows/Linux/macOS CI) keeps working without the wasm toolchain.
//! The WASM entry is gated behind `cfg(target_arch = "wasm32")` and shares
//! `api` / `app` / `models` / `platform` with the desktop build; the desktop
//! GUI modules (`ui`, `map`, `tilepacks`) are excluded on wasm today (they
//! need `tokio` + the filesystem), so the web UI is the minimal shell in
//! `web/index.html`. Filling the full egui UI on wasm requires the `tokio`
//! → `wasm` runtime swap tracked in `docs/client-platforms.md`.
//!
//! Backend URL: same resolution as desktop — `PINPOINT_BACKEND` env >
//! per-platform default (`platform::backend_url_from_env_or_default`).
//! In the browser, `http://localhost:8000` is a loopback URL only when
//! the page itself is served from loopback; production web builds must
//! point at the deployed gateway's HTTPS URL (ADR-0006).

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Native stub: keeps `cargo build` / `cargo test` green on desktop CI
    // where the wasm target is not installed.
    let backend = pinpoint_desktop::platform::backend_url_from_env_or_default();
    eprintln!("pinpoint-web (native stub)");
    eprintln!(
        "  platform : {}",
        pinpoint_desktop::platform::Platform::current().label()
    );
    eprintln!("  backend  : {backend}");
    eprintln!("  build web with: trunk build --release (see web/README.md)");
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::start]
fn start() {
    // Minimal WASM entry: the static shell in web/index.html already drives
    // the first paint; the full egui loop lands behind the tokio→wasm swap
    // (docs/client-platforms.md §Web). Kept as a start hook so trunk/wasm-
    // bindgen wiring is exercised without requiring the desktop GUI stack.
    eprintln!(
        "pinpoint-web (wasm) backend={}",
        pinpoint_desktop::platform::backend_url_from_env_or_default()
    );
}
