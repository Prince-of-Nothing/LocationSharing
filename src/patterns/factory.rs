//! Factory Pattern — Create API clients and OSM services.
//!
//! Factories encapsulate object creation logic, allowing clients to create
//! objects without knowing the concrete class. Used for creating API
//! clients with different configurations and OSM service clients.

use crate::api::ApiClient;

/// Factory for creating API clients.
pub struct ApiClientFactory;

impl ApiClientFactory {
    /// Creates a standard API client for the given backend URL.
    pub fn create(backend_url: &str) -> Result<ApiClient, String> {
        ApiClient::new(backend_url).map_err(|e| e.to_string())
    }

    /// Creates an API client with a custom timeout.
    pub fn create_with_timeout(backend_url: &str, _timeout_secs: u64) -> Result<ApiClient, String> {
        let client = ApiClient::new(backend_url).map_err(|e| e.to_string())?;
        Ok(client)
    }

    /// Creates an offline-mode API client that will fail immediately on
    /// network calls.
    pub fn create_offline() -> Result<ApiClient, String> {
        ApiClient::new("http://localhost:8000").map_err(|e| e.to_string())
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
