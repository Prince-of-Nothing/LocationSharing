//! Pinpoint — Rust multiplatform client for the Pinpoint API.
//!
//! One codebase for desktop (Windows/Linux/macOS), Android (`cargo-apk` +
//! `eframe`/`winit`) and iOS (`cargo-xcodebuild`), plus a wasm stub target.
//! The library target holds the portable core so contract tests under
//! `tests/` can drive [`api::ApiClient`] and [`osm::OsmClient`] without
//! launching a window.

pub mod api;
pub mod app;
pub mod bookmarks;
pub mod models;
pub mod osm;
pub mod patterns;
pub mod platform;
pub mod security;
pub mod server;
// The GUI modules depend on native-only crates (tokio runtime, `which`),
// so they are compiled out of the WebAssembly target.
#[cfg(not(target_arch = "wasm32"))]
pub mod map;
#[cfg(not(target_arch = "wasm32"))]
pub mod tilepacks;
#[cfg(not(target_arch = "wasm32"))]
pub mod ui;
