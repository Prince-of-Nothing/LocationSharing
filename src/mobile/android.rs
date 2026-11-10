//! Android-specific launch wiring (cargo-apk / eframe+winit).
//!
//! Kept separate so backend/bootstrap logic remains shared and modular.
//! eframe/winit Android integration depends on the selected activity backend
//! and crate features; this module provides the Pinpoint-side launch path.

#[cfg(target_os = "android")]
use crate::backend::build_backend;
#[cfg(target_os = "android")]
use crate::mobile::MobileLaunchConfig;

#[cfg(target_os = "android")]
pub fn launch_android_ui() -> Result<(), eframe::Error> {
    let cfg = MobileLaunchConfig::default();
    let backend = match build_backend(&cfg.backend) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Android backend init failed ({e}); using default");
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

// Note: cargo-apk/winit may require an `android_main` entry depending on the
// activity feature in use (e.g. android-activity). Wire the exact signature
// required by the target toolchain here, delegating to `launch_android_ui()`.
#[cfg(all(target_os = "android", feature = "android-native-activity"))]
#[no_mangle]
fn android_main(_app: eframe::winit::platform::android::activity::AndroidApp) {
    // Placeholder for feature-gated integration; keep backend/UI path shared.
    let _ = launch_android_ui();
}
