//! iOS-specific launch wiring (cargo-xcodebuild / eframe+winit).
//!
//! Kept separate so backend/bootstrap logic remains shared and modular.

#[cfg(target_os = "ios")]
use crate::backend::build_backend;
#[cfg(target_os = "ios")]
use crate::mobile::MobileLaunchConfig;

#[cfg(target_os = "ios")]
pub fn launch_ios_ui() -> Result<(), eframe::Error> {
    let cfg = MobileLaunchConfig::default();
    let backend = match build_backend(&cfg.backend) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("iOS backend init failed ({e}); using default");
            crate::backend::build_default_backend(crate::platform::default_backend_url())
                .expect("default backend must be constructible")
        }
    };

    let mut app = crate::ui::PinpointAppUI::new(backend, None);
    app.set_offline_mode(cfg.offline_first);

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_title("Pinpoint"),
        ..Default::default()
    };
    eframe::run_native("Pinpoint", options, Box::new(|_cc| Ok(Box::new(app))))
}
