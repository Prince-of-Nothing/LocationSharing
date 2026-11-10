//! REST backend adapter construction.
//!
//! The concrete implementation lives in [`crate::api::ApiClient`] to preserve
//! the existing module path, contract tests, and wire-format logic. This module
//! centralizes REST-specific construction and makes the "REST v1" choice explicit
//! in [`super::BackendConfig`].

use super::{BackendConfig, DynBackend};
use std::sync::Arc;

/// Construct the REST v1 backend. Endpoint paths, auth (session cookie + CSRF),
/// and the error envelope are owned by `api::ApiClient`; change them there (or
/// add a new adapter variant) when the backend architecture evolves.
pub fn build_rest_backend(config: &BackendConfig) -> anyhow::Result<DynBackend> {
    // `timeout_secs` is reserved for future use; ApiClient currently applies its
    // own default (15s) matching existing behavior. Adapters may honor it.
    let client = crate::api::ApiClient::new(config.base_url.clone())?;
    let _ = config.timeout_secs;
    Ok(Arc::new(client) as DynBackend)
}
