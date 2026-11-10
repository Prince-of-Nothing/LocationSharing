//! Pinpoint — Rust multiplatform client for the Pinpoint API.
//!
//! One codebase for desktop (Windows/Linux/macOS), Android (`cargo-apk` +
//! `eframe`/`winit`) and iOS (`cargo-xcodebuild`), plus a wasm stub target.
//! The library target holds the portable core so contract tests under
//! `tests/` can drive [`api::ApiClient`] and [`osm::OsmClient`] without
//! launching a window.
//!
//! Backend access is modular via [`backend::Backend`] / [`backend::DynBackend`]
//! so client apps (desktop/mobile/webk/weba) can adapt to changed backend
//! architectures by swapping adapters, not app logic.

pub mod api;
pub mod app;
pub mod backend;
pub mod bookmarks;
pub mod mobile;
pub mod models;
pub mod osm;
pub mod patterns;
pub mod platform;
pub mod security;
pub mod server;
pub mod webapp;
// The GUI modules depend on native-only crates (tokio runtime, `which`),
// so they are compiled out of the WebAssembly target.
#[cfg(not(target_arch = "wasm32"))]
pub mod map;
#[cfg(not(target_arch = "wasm32"))]
pub mod tilepacks;
#[cfg(not(target_arch = "wasm32"))]
pub mod ui;
