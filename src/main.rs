//! Main entry point for Pinpoint Desktop (Windows / Linux / macOS).
//!
//! One `eframe` binary for all desktop OSes; Android/iOS reuse the same
//! `src/` via `cargo-apk` / `cargo-xcodebuild` (see `docs/client-platforms.md`),
//! and the Web (WASM) client shares `src/` via a separate entry
//! (`src/bin/pinpoint-web.rs` + `web/`). Backend URL resolution:
//! `--backend <url>` > `PINPOINT_BACKEND` env > per-platform default.

#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
fn print_help_and_exit() -> ! {
    eprintln!(
        "Pinpoint {} ({})\n\
         \n\
         USAGE:\n\
         \x20  pinpoint-desktop [--backend <URL>] [--screenshot <PATH>] [--help]\n\
         \n\
         OPTIONS:\n\
         \x20  --backend <URL>     Backend base URL (default: {} or $PINPOINT_BACKEND).\n\
         \x20                      Plain HTTP is only allowed for loopback (ADR-0006);\n\
         \x20                      anything else must be HTTPS.\n\
         \x20  --screenshot <PATH> Headless screenshot mode (CI/test artifact).\n\
         \x20  --help              Show this help.\n\
         ",
        pinpoint_desktop::platform::build_identity(),
        pinpoint_desktop::platform::Platform::current().label(),
        pinpoint_desktop::platform::default_backend_url(),
    );
    std::process::exit(0);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut screenshot_path = None;
    let mut backend_url = pinpoint_desktop::platform::backend_url_from_env_or_default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => print_help_and_exit(),
            "--screenshot" => {
                if i + 1 < args.len() {
                    screenshot_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("--screenshot requires a file path");
                    std::process::exit(1);
                }
            }
            "--backend" => {
                if i + 1 < args.len() {
                    backend_url = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("--backend requires a URL");
                    std::process::exit(1);
                }
            }
            unknown => {
                eprintln!("unknown argument: {unknown} (try --help)");
                std::process::exit(1);
            }
        }
    }

    tracing_subscriber::fmt::init();

    // The map is fully functional offline. We create the API client
    // optimistically; if the backend is unreachable the app degrades to
    // offline mode (demo friend locations, OSM navigation hand-off,
    // no publishing/sharing). Plain HTTP is only accepted for loopback
    // development URLs (ADR-0006).
    let api_client = match pinpoint_desktop::api::ApiClient::new(&backend_url) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Backend URL {backend_url:?} rejected ({e}); starting in offline mode.");
            // Fall back to the loopback default so the map still opens offline.
            pinpoint_desktop::api::ApiClient::new("http://localhost:8000").unwrap_or_else(|e2| {
                eprintln!("Cannot create API client: {e2}");
                std::process::exit(1);
            })
        }
    };

    let mut app = pinpoint_desktop::ui::PinpointAppUI::new(Arc::new(api_client), screenshot_path);

    app.set_offline_mode(true);

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Pinpoint — Map First"),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native("Pinpoint", options, Box::new(|_cc| Ok(Box::new(app)))) {
        eprintln!("Error running app: {:?}", e);
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eprintln!("Web target - use wasm-bindgen entry point");
}
