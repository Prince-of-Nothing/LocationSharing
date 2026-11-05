//! Main entry point for Pinpoint Desktop.

#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut screenshot_path = None;
    let mut backend_url = "http://localhost:8000".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
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
            _ => {
                i += 1;
            }
        }
    }

    tracing_subscriber::fmt::init();

    // Fail closed on transports outside the policy (ADR-0005): plain HTTP
    // is only accepted for loopback development URLs.
    let api_client = match pinpoint_desktop::api::ApiClient::new(backend_url) {
        Ok(client) => Arc::new(client),
        Err(e) => {
            eprintln!("Cannot start: {e}");
            std::process::exit(1);
        }
    };

    let app = pinpoint_desktop::ui::PinpointAppUI::new(api_client, screenshot_path);

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Pinpoint"),
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
