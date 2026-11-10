//! Factory Pattern — Create backend clients and OSM services.
//!
//! Factories encapsulate object creation logic. Backend creation is routed
//! through [`crate::backend`] so changed backend architectures can be selected
//! by config without touching call sites.

use crate::api::ApiClient;
use crate::backend::{build_backend, BackendConfig, BackendKind, DynBackend};

/// Factory for creating API clients (REST v1 concrete) and modular backends.
pub struct ApiClientFactory;

impl ApiClientFactory {
    /// Creates a standard REST API client for the given backend URL.
    /// Preserved for compatibility; prefer [`Self::create_backend`] for new code.
    pub fn create(backend_url: &str) -> Result<ApiClient, String> {
        ApiClient::new(backend_url).map_err(|e| e.to_string())
    }

    /// Creates a REST API client with a custom timeout (timeout honored by
    /// future adapters; current REST client keeps its default).
    pub fn create_with_timeout(backend_url: &str, timeout_secs: u64) -> Result<ApiClient, String> {
        let _ = timeout_secs;
        let client = ApiClient::new(backend_url).map_err(|e| e.to_string())?;
        Ok(client)
    }

    /// Creates an offline-mode REST client (loopback; calls will fail when no
    /// local backend is running, and the app degrades to offline state).
    pub fn create_offline() -> Result<ApiClient, String> {
        ApiClient::new("http://localhost:8000").map_err(|e| e.to_string())
    }

    /// Create a modular backend selected by config (preferred extension point).
    pub fn create_backend(config: &BackendConfig) -> Result<DynBackend, String> {
        build_backend(config).map_err(|e| e.to_string())
    }

    /// Create the default modular REST v1 backend.
    pub fn create_default_backend(backend_url: &str) -> Result<DynBackend, String> {
        Self::create_backend(&BackendConfig {
            kind: BackendKind::RestV1,
            base_url: backend_url.to_string(),
            timeout_secs: None,
            options: Default::default(),
        })
    }
}

/// Factory for creating OSM service clients.
pub struct OsmClientFactory;

impl OsmClientFactory {
    /// Creates a standard OSM client.
    pub fn create() -> Result<crate::osm::OsmClient, String> {
        crate::osm::OsmClient::new().map_err(|e| e.to_string())
    }
}

/// Factory for creating precision strategies.
pub struct PrecisionStrategyFactory;

impl PrecisionStrategyFactory {
    pub fn create(
        level: crate::patterns::strategy::PrecisionLevel,
    ) -> Box<dyn crate::patterns::strategy::PrecisionStrategy> {
        match level {
            crate::patterns::strategy::PrecisionLevel::Exact => {
                Box::new(crate::patterns::strategy::ExactStrategy)
            }
            crate::patterns::strategy::PrecisionLevel::Approximate => {
                Box::new(crate::patterns::strategy::ApproximateStrategy)
            }
            crate::patterns::strategy::PrecisionLevel::CityOnly => {
                Box::new(crate::patterns::strategy::CityOnlyStrategy)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_client_factory_creates_client() {
        let client = ApiClientFactory::create("http://localhost:8000");
        assert!(client.is_ok());
    }

    #[test]
    fn osm_client_factory_creates_client() {
        let client = OsmClientFactory::create();
        assert!(client.is_ok());
    }

    #[test]
    fn precision_strategy_factory_creates_strategies() {
        let exact =
            PrecisionStrategyFactory::create(crate::patterns::strategy::PrecisionLevel::Exact);
        assert_eq!(
            exact.level(),
            crate::patterns::strategy::PrecisionLevel::Exact
        );

        let approx = PrecisionStrategyFactory::create(
            crate::patterns::strategy::PrecisionLevel::Approximate,
        );
        assert_eq!(
            approx.level(),
            crate::patterns::strategy::PrecisionLevel::Approximate
        );

        let city =
            PrecisionStrategyFactory::create(crate::patterns::strategy::PrecisionLevel::CityOnly);
        assert_eq!(
            city.level(),
            crate::patterns::strategy::PrecisionLevel::CityOnly
        );
    }
}
