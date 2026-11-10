//! Backend abstraction layer — makes the client modular and adaptable to
//! changed backend architectures.
//!
//! Design goals:
//! - `PinpointApp` and UI code depend on [`Backend`], not on a concrete HTTP
//!   client or wire format.
//! - The current REST/session-cookie implementation lives in [`crate::api`] and
//!   implements [`Backend`]. Swapping to a different backend architecture (new
//!   endpoints, auth scheme, protocol, or API version) means adding/adjusting an
//!   adapter behind this trait, not rewriting app logic.
//! - [`BackendConfig`] + [`build_backend`] provide a single construction point
//!   driven by config/env, so deployment targets can select the backend adapter.
//!
//! Security invariants (must hold for every adapter):
//! - Fail closed on transport/auth/session errors (ADR-0006 semantics).
//! - Never leak secrets/tokens in error messages or logs.
//! - Server-side enforcement only; clients are untrusted.

pub mod rest;

use crate::models::{CheckIn, EmergencyAlert, FriendLocation, Friendship, LocationShare, User};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Boxed future used by [`Backend`] for dyn-compatibility without extra
/// dependencies. Native targets require `Send`; wasm runs single-threaded.
#[cfg(not(target_arch = "wasm32"))]
pub type BackendFuture<'a, T> = Pin<Box<dyn Future<Output = anyhow::Result<T>> + Send + 'a>>;
#[cfg(target_arch = "wasm32")]
pub type BackendFuture<'a, T> = Pin<Box<dyn Future<Output = anyhow::Result<T>> + 'a>>;

/// Bounds required for backend trait objects: `Send + Sync` on native, relaxed on wasm.
#[cfg(not(target_arch = "wasm32"))]
pub trait BackendBounds: Send + Sync {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Send + Sync + ?Sized> BackendBounds for T {}

#[cfg(target_arch = "wasm32")]
pub trait BackendBounds {}
#[cfg(target_arch = "wasm32")]
impl<T: ?Sized> BackendBounds for T {}

/// Shared backend handle used across app/UI/webapp layers.
pub type DynBackend = Arc<dyn Backend>;

/// Abstracts all backend operations the Pinpoint clients need.
///
/// Implementations must preserve the security/contract expectations documented
/// on each method. The default REST adapter (`api::ApiClient`) maps these to
/// the current `/api/v1/*` contract; future adapters may target a different
/// backend architecture while keeping this surface stable.
pub trait Backend: BackendBounds {
    /// Base URL / identifier for diagnostics (never contains secrets).
    fn backend_url(&self) -> &str;
    /// Whether an authenticated session/state is currently held.
    fn session_active(&self) -> bool;

    fn login<'a>(&'a self, email: &'a str, password: &'a str) -> BackendFuture<'a, User>;
    fn register<'a>(
        &'a self,
        email: &'a str,
        password: &'a str,
        full_name: &'a str,
    ) -> BackendFuture<'a, User>;
    /// Confirm logout with the backend before dropping local session state.
    fn logout<'a>(&'a self) -> BackendFuture<'a, ()>;

    fn mfa_enroll<'a>(&'a self) -> BackendFuture<'a, (String, String)>;
    fn mfa_verify<'a>(&'a self, code: &'a str) -> BackendFuture<'a, ()>;
    fn mfa_status<'a>(&'a self) -> BackendFuture<'a, bool>;
    fn mfa_disable<'a>(&'a self) -> BackendFuture<'a, ()>;

    /// Returns `Ok(true)` when the backend revoked sessions (regular change),
    /// `Ok(false)` when the current session remains valid (forced change).
    fn change_password<'a>(
        &'a self,
        current_password: Option<&'a str>,
        new_password: &'a str,
        confirm_password: &'a str,
    ) -> BackendFuture<'a, bool>;

    fn get_friendships<'a>(&'a self) -> BackendFuture<'a, Vec<Friendship>>;
    fn send_friend_request<'a>(&'a self, email: &'a str) -> BackendFuture<'a, ()>;
    fn respond_to_friendship<'a>(&'a self, id: i64, action: &'a str) -> BackendFuture<'a, ()>;
    fn delete_friendship<'a>(&'a self, id: i64) -> BackendFuture<'a, ()>;

    fn get_location_shares<'a>(&'a self) -> BackendFuture<'a, Vec<LocationShare>>;
    fn create_location_share<'a>(
        &'a self,
        friendship_id: i64,
        direction: &'a str,
        precision_level: &'a str,
    ) -> BackendFuture<'a, ()>;
    fn create_share_with_expiry<'a>(
        &'a self,
        friendship_id: i64,
        direction: &'a str,
        precision_level: &'a str,
        duration_hours: Option<i64>,
        label: Option<&'a str>,
    ) -> BackendFuture<'a, ()>;
    fn update_location_share<'a>(
        &'a self,
        share_id: i64,
        is_active: Option<bool>,
        precision: Option<String>,
    ) -> BackendFuture<'a, ()>;
    fn delete_share<'a>(&'a self, share_id: i64) -> BackendFuture<'a, ()>;

    fn get_friend_locations<'a>(&'a self) -> BackendFuture<'a, Vec<FriendLocation>>;
    fn publish_location<'a>(
        &'a self,
        latitude: f64,
        longitude: f64,
        accuracy: f64,
    ) -> BackendFuture<'a, ()>;
    fn get_my_location<'a>(&'a self) -> BackendFuture<'a, FriendLocation>;

    fn list_users<'a>(&'a self) -> BackendFuture<'a, Vec<User>>;
    fn search_users<'a>(&'a self, q: &'a str) -> BackendFuture<'a, Vec<User>>;
    fn update_user<'a>(&'a self, id: i64, body: serde_json::Value) -> BackendFuture<'a, ()>;
    fn delete_user<'a>(&'a self, id: i64) -> BackendFuture<'a, ()>;

    fn create_check_in<'a>(
        &'a self,
        lat: f64,
        lon: f64,
        note: &'a str,
        kind: &'a str,
    ) -> BackendFuture<'a, ()>;
    fn list_check_ins<'a>(&'a self) -> BackendFuture<'a, Vec<CheckIn>>;

    fn trigger_emergency<'a>(&'a self, lat: f64, lon: f64) -> BackendFuture<'a, ()>;
    fn list_emergencies<'a>(&'a self) -> BackendFuture<'a, Vec<EmergencyAlert>>;
}

/// Selects which backend adapter to construct. Add variants here when the
/// backend architecture changes (e.g. a new API version, protocol, or auth
/// model) and implement the adapter behind [`Backend`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendKind {
    /// Current REST contract (`/api/v1/*`, session cookie + CSRF double-submit).
    RestV1,
    // Future: RestV2, GraphQL, GrpcWebBridge, Mock, etc.
}

/// Configuration for constructing a backend client.
#[derive(Debug, Clone)]
pub struct BackendConfig {
    pub kind: BackendKind,
    /// Base URL for network backends (validated per adapter policy).
    pub base_url: String,
    /// Optional request timeout override in seconds.
    pub timeout_secs: Option<u64>,
    /// Free-form adapter options for future backend-specific settings
    /// (auth mode, tenant, API version pin, feature flags). Keys/values must
    /// never contain secrets in logs; adapters should treat unknown keys as
    /// non-fatal.
    pub options: std::collections::BTreeMap<String, String>,
}

impl BackendConfig {
    pub fn rest_v1(base_url: impl Into<String>) -> Self {
        Self {
            kind: BackendKind::RestV1,
            base_url: base_url.into(),
            timeout_secs: None,
            options: std::collections::BTreeMap::new(),
        }
    }

    /// Resolve config from environment/platform defaults, preserving the
    /// existing resolution order: `PINPOINT_BACKEND` > platform default.
    pub fn from_env() -> Self {
        Self::rest_v1(crate::platform::backend_url_from_env_or_default())
    }
}

/// Build a backend from config. This is the single extension point for changed
/// backend architectures.
pub fn build_backend(config: &BackendConfig) -> anyhow::Result<DynBackend> {
    match config.kind {
        BackendKind::RestV1 => rest::build_rest_backend(config),
    }
}

/// Convenience: build the default REST backend from a base URL.
pub fn build_default_backend(base_url: impl Into<String>) -> anyhow::Result<DynBackend> {
    build_backend(&BackendConfig::rest_v1(base_url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_from_env_has_non_empty_url() {
        let cfg = BackendConfig::from_env();
        assert!(!cfg.base_url.is_empty());
        assert_eq!(cfg.kind, BackendKind::RestV1);
    }
}
