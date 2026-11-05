//! Pinpoint Desktop — Rust client for the Pinpoint API.
//!
//! The modules live in the library target (rather than the binary) so the
//! contract tests under `tests/` can drive [`api::ApiClient`] directly
//! against a recorded HTTP conversation.

pub mod api;
pub mod app;
pub mod models;

// The GUI modules depend on native-only crates (tokio runtime, `which`),
// so they are compiled out of the WebAssembly target.
#[cfg(not(target_arch = "wasm32"))]
pub mod map;
#[cfg(not(target_arch = "wasm32"))]
pub mod ui;
