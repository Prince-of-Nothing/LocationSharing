//! Mobile (Android/iOS) app bootstrap — modular and backend-adaptable.
//!
//! Both mobile targets share the same core, UI, and [`crate::backend::Backend`]
//! abstraction as desktop. Platform-specific integration (activity lifecycle,
//! storage, permissions) belongs behind `cfg(target_os)` and thin wrappers so
//! backend architecture changes never require mobile-only rewrites.

#[cfg(target_os = "android")]
pub mod android;
#[cfg(target_os = "ios")]
pub mod ios;

use crate::backend::BackendConfig;

/// Shared mobile launch options derived from host/platform config.
#[derive(Debug, Clone)]
pub struct MobileLaunchConfig {
    pub backend: BackendConfig,
    pub offline_first: bool,
}

impl Default for MobileLaunchConfig {
    fn default() -> Self {
        Self {
            backend: BackendConfig::from_env(),
            offline_first: true,
        }
    }
}

/// Resolved backend config for mobile launches (env/host override > platform
/// default). Adapters enforce transport policy (ADR-0006).
pub fn mobile_backend_config() -> BackendConfig {
    BackendConfig::from_env()
}

/// Build a backend description for diagnostics (no network side effects beyond
/// what the adapter constructor does).
pub fn mobile_backend_label(cfg: &BackendConfig) -> String {
    format!("{:?}@{}", cfg.kind, cfg.base_url)
}
