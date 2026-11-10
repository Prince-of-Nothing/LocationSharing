//! Pinpoint Web client (Rust/WASM via `trunk`).
//!
//! General map-first web entry point sharing the modular backend/core with
//! desktop/mobile/weba/webk. Backend architecture changes are isolated to
//! `crate::backend` adapters and `webapp::config`.

use pinpoint_desktop::webapp::{config::WebAppConfig, WebAppKind};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let cfg = WebAppConfig::new(WebAppKind::Web);
    eprintln!("pinpoint-web (native stub)");
    eprintln!("  app      : {}", WebAppKind::Web.label());
    eprintln!(
        "  platform : {}",
        pinpoint_desktop::platform::Platform::current().label()
    );
    eprintln!("  backend  : {}", cfg.backend.base_url);
    eprintln!("  build web with: trunk build --config web/Trunk.toml --release");
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    let cfg = WebAppConfig::new(WebAppKind::Web);
    log_info(&format!(
        "pinpoint-web ready backend={} kind={:?}",
        cfg.backend.base_url, cfg.backend.kind
    ));
}

/// JSON snapshot of web app identity/config for host-page bootstrap.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn web_app_info() -> String {
    let cfg = WebAppConfig::new(WebAppKind::Web);
    let app = pinpoint_desktop::webapp::web::WebApp::new(cfg);
    app.snapshot_json()
}

#[cfg(target_arch = "wasm32")]
fn log_info(_msg: &str) {}
