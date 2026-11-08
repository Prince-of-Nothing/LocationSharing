//! Proxy Pattern — Offline/online proxy for network services.
//!
//! The proxy sits between the application and the network, intercepting
//! calls and returning cached or offline data when the network is
//! unavailable. This enables the app to work offline gracefully.

use crate::api::ApiClient;
use crate::models::FriendLocation;
use std::time::{Duration, Instant};

/// Real service that makes actual API calls.
pub struct RealLocationService {
    client: ApiClient,
    last_success: Option<Instant>,
}

impl RealLocationService {
    pub fn new(client: ApiClient) -> Self {
        Self {
            client,
            last_success: None,
        }
    }

    pub async fn get_friend_locations(&mut self) -> Result<Vec<FriendLocation>, String> {
        let result = self.client.get_friend_locations().await;
        if result.is_ok() {
            self.last_success = Some(Instant::now());
        }
        result.map_err(|e| e.to_string())
    }

    pub async fn publish_location(
        &mut self,
        lat: f64,
        lon: f64,
        accuracy: f64,
    ) -> Result<(), String> {
        let result = self.client.publish_location(lat, lon, accuracy).await;
        if result.is_ok() {
            self.last_success = Some(Instant::now());
        }
        result.map_err(|e| e.to_string())
    }

    pub fn is_online(&self) -> bool {
        match self.last_success {
            Some(t) => t.elapsed() < Duration::from_secs(300),
            None => false,
        }
    }
}

/// Proxy service that caches data and falls back to cache when offline.
pub struct ProxyLocationService {
    real_service: Option<RealLocationService>,
    cached_locations: Vec<FriendLocation>,
    is_online: bool,
}

impl ProxyLocationService {
    pub fn new(client: ApiClient) -> Self {
        Self {
            real_service: Some(RealLocationService::new(client)),
            cached_locations: Vec::new(),
            is_online: true,
        }
    }

    /// Force the proxy into offline mode.
    pub fn set_offline(&mut self) {
        self.is_online = false;
    }

    /// Force the proxy into online mode.
    pub fn set_online(&mut self) {
        self.is_online = true;
    }

    /// Returns whether the proxy is currently online.
    pub fn is_online(&self) -> bool {
        self.is_online
    }

    /// Get friend locations, falling back to cache when offline.
    pub async fn get_friend_locations(&mut self) -> Result<Vec<FriendLocation>, String> {
        if self.is_online {
            if let Some(real) = self.real_service.as_mut() {
                match real.get_friend_locations().await {
                    Ok(locations) => {
                        self.cached_locations = locations.clone();
                        Ok(locations)
                    }
                    Err(e) => {
                        self.is_online = false;
                        if !self.cached_locations.is_empty() {
                            Ok(self.cached_locations.clone())
                        } else {
                            Err(format!("Offline and no cached data: {e}"))
                        }
                    }
                }
            } else {
                if !self.cached_locations.is_empty() {
                    Ok(self.cached_locations.clone())
                } else {
                    Err("No real service and no cached data".to_string())
                }
            }
        } else {
            if !self.cached_locations.is_empty() {
                Ok(self.cached_locations.clone())
            } else {
                Err("Offline mode - no cached location data".to_string())
            }
        }
    }

    /// Publish location, failing if offline.
    pub async fn publish_location(
        &mut self,
        lat: f64,
        lon: f64,
        accuracy: f64,
    ) -> Result<(), String> {
        if self.is_online {
            if let Some(real) = self.real_service.as_mut() {
                match real.publish_location(lat, lon, accuracy).await {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        self.is_online = false;
                        Err(format!("Failed to publish (offline now): {e}"))
                    }
                }
            } else {
                Err("Cannot publish while offline".to_string())
            }
        } else {
            Err("Cannot publish while offline".to_string())
        }
    }

    /// Refresh the offline cache from the server.
    pub async fn refresh_cache(&mut self) -> Result<(), String> {
        if self.is_online {
            if let Some(real) = self.real_service.as_mut() {
                let locations = real.get_friend_locations().await?;
                self.cached_locations = locations.clone();
                Ok(())
            } else {
                Err("No real service available".to_string())
            }
        } else {
            Err("Cannot refresh cache while offline".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_starts_online() {
        let client = crate::api::ApiClient::new("http://localhost:8000").unwrap();
        let proxy = ProxyLocationService::new(client);
        assert!(proxy.is_online());
    }

    #[test]
    fn proxy_can_toggle_offline() {
        let client = crate::api::ApiClient::new("http://localhost:8000").unwrap();
        let mut proxy = ProxyLocationService::new(client);
        proxy.set_offline();
        assert!(!proxy.is_online());
        proxy.set_online();
        assert!(proxy.is_online());
    }
}
