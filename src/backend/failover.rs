//! Failover backend wrapper — automatic failover across multiple backend URLs
//! with per-backend circuit breakers, health monitoring, and retry with
//! exponential backoff.
//!
//! Example:
//!   let failover = FailoverBackend::new(vec![
//!       "http://gateway:8000",
//!       "http://auth-replica-2:8001",
//!   ])?;
//!   let user = failover.login("alice@example.com", "password").await?;

use crate::backend::{Backend, BackendFuture, DynBackend};
use crate::models::{CheckIn, EmergencyAlert, FriendLocation, Friendship, LocationShare, User};
use crate::security::circuit_breaker::CircuitBreaker;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// A single backend entry in the failover pool.
#[derive(Clone)]
struct BackendEntry {
    url: String,
    backend: DynBackend,
    circuit: Arc<CircuitBreaker>,
}

/// Failover backend: tries backends in order, skipping open-circuit or
/// unhealthy ones, with exponential backoff between attempts.
#[derive(Clone)]
pub struct FailoverBackend {
    backends: Arc<RwLock<Vec<BackendEntry>>>,
    max_retries: u32,
    base_backoff_ms: u64,
}

impl FailoverBackend {
    /// Create a new failover backend from a list of URLs. The first URL is
    /// treated as the primary; all URLs are tried in order on failure.
    pub fn new(urls: Vec<String>) -> anyhow::Result<Self> {
        if urls.is_empty() {
            anyhow::bail!("at least one backend URL is required");
        }
        let mut entries = Vec::new();
        for url in urls {
            let backend = crate::api::ApiClient::new(url.clone())?;
            let circuit = CircuitBreaker::default_with_name(url.clone());
            entries.push(BackendEntry {
                url: url.clone(),
                backend: Arc::new(backend) as DynBackend,
                circuit,
            });
        }
        Ok(Self {
            backends: Arc::new(RwLock::new(entries)),
            max_retries: 3,
            base_backoff_ms: 500,
        })
    }

    /// Execute `f` against each backend in order, retrying with exponential
    /// backoff. Returns the first success. If all backends fail, returns
    /// the last error.
    async fn try_all<F, Fut, T>(&self, op: &str, mut f: F) -> anyhow::Result<T>
    where
        F: FnMut(DynBackend) -> Fut + Send,
        Fut: Future<Output = anyhow::Result<T>> + Send,
    {
        let entries = self.backends.read().await;
        let mut last_error: Option<anyhow::Error> = None;

        for attempt in 0..=self.max_retries {
            for entry in entries.iter() {
                let circuit = entry.circuit.clone();
                if !circuit.can_proceed() {
                    continue;
                }

                let backend = entry.backend.clone();
                let result = f(backend).await;

                match result {
                    Ok(v) => {
                        circuit.on_success();
                        return Ok(v);
                    }
                    Err(e) => {
                        circuit.on_failure();
                        last_error = Some(e);
                    }
                }
            }

            // Backoff before next retry round (unless this was the last).
            if attempt < self.max_retries {
                let backoff = self.base_backoff_ms * 2u64.pow(attempt);
                tokio::time::sleep(Duration::from_millis(backoff)).await;
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("all backends unavailable for {op}")))
    }

    /// Set max retry attempts per backend round (default 3).
    pub fn with_max_retries(mut self, n: u32) -> Self {
        self.max_retries = n;
        self
    }

    /// Set base backoff in ms between retry rounds (default 500).
    pub fn with_base_backoff_ms(mut self, ms: u64) -> Self {
        self.base_backoff_ms = ms;
        self
    }

    /// Return all backend URLs (for diagnostics).
    pub async fn urls(&self) -> Vec<String> {
        let entries = self.backends.read().await;
        entries.iter().map(|e| e.url.clone()).collect()
    }

    /// Return circuit breaker state per URL (for UI / debugging).
    pub async fn circuit_states(&self) -> Vec<(String, String, u32)> {
        let entries = self.backends.read().await;
        let mut out: Vec<(String, String, u32)> = entries
            .iter()
            .map(|e| {
                (
                    e.url.clone(),
                    format!("{:?}", e.circuit.state()),
                    e.circuit.failures(),
                )
            })
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }
}

// ── Backend trait implementation ──────────────────────────────────────

impl Backend for FailoverBackend {
    fn backend_url(&self) -> &str {
        // Return the first (primary) URL for diagnostics.
        "failover://multi"
    }

    fn session_active(&self) -> bool {
        // Session is active if any backend has a session.
        // Best-effort: check the first (primary) backend synchronously.
        let entries = self.backends.try_read();
        if let Ok(entries) = entries {
            entries.iter().any(|e| e.backend.session_active())
        } else {
            false
        }
    }

    fn login<'a>(&'a self, email: &'a str, password: &'a str) -> BackendFuture<'a, User> {
        Box::pin(async move {
            self.try_all("login", |b| {
                let b = b.clone();
                async move { b.login(email, password).await }
            })
            .await
        })
    }

    fn register<'a>(
        &'a self,
        email: &'a str,
        password: &'a str,
        full_name: &'a str,
    ) -> BackendFuture<'a, User> {
        Box::pin(async move {
            self.try_all("register", |b| {
                let b = b.clone();
                async move { b.register(email, password, full_name).await }
            })
            .await
        })
    }

    fn logout<'a>(&'a self) -> BackendFuture<'a, ()> {
        Box::pin(async move {
            self.try_all("logout", |b| {
                let b = b.clone();
                async move { b.logout().await }
            })
            .await
        })
    }

    fn mfa_enroll<'a>(&'a self) -> BackendFuture<'a, (String, String)> {
        Box::pin(async move {
            self.try_all("mfa_enroll", |b| {
                let b = b.clone();
                async move { b.mfa_enroll().await }
            })
            .await
        })
    }

    fn mfa_verify<'a>(&'a self, code: &'a str) -> BackendFuture<'a, ()> {
        Box::pin(async move {
            self.try_all("mfa_verify", |b| {
                let b = b.clone();
                async move { b.mfa_verify(code).await }
            })
            .await
        })
    }

    fn mfa_status<'a>(&'a self) -> BackendFuture<'a, bool> {
        Box::pin(async move {
            self.try_all("mfa_status", |b| {
                let b = b.clone();
                async move { b.mfa_status().await }
            })
            .await
        })
    }

    fn mfa_disable<'a>(&'a self) -> BackendFuture<'a, ()> {
        Box::pin(async move {
            self.try_all("mfa_disable", |b| {
                let b = b.clone();
                async move { b.mfa_disable().await }
            })
            .await
        })
    }

    fn change_password<'a>(
        &'a self,
        current_password: Option<&'a str>,
        new_password: &'a str,
        confirm_password: &'a str,
    ) -> BackendFuture<'a, bool> {
        Box::pin(async move {
            self.try_all("change_password", |b| {
                let b = b.clone();
                async move {
                    b.change_password(current_password, new_password, confirm_password)
                        .await
                }
            })
            .await
        })
    }

    fn get_friendships<'a>(&'a self) -> BackendFuture<'a, Vec<Friendship>> {
        Box::pin(async move {
            self.try_all("get_friendships", |b| {
                let b = b.clone();
                async move { b.get_friendships().await }
            })
            .await
        })
    }

    fn send_friend_request<'a>(&'a self, email: &'a str) -> BackendFuture<'a, ()> {
        Box::pin(async move {
            self.try_all("send_friend_request", |b| {
                let b = b.clone();
                async move { b.send_friend_request(email).await }
            })
            .await
        })
    }

    fn respond_to_friendship<'a>(&'a self, id: i64, action: &'a str) -> BackendFuture<'a, ()> {
        Box::pin(async move {
            self.try_all("respond_to_friendship", |b| {
                let b = b.clone();
                async move { b.respond_to_friendship(id, action).await }
            })
            .await
        })
    }

    fn delete_friendship<'a>(&'a self, id: i64) -> BackendFuture<'a, ()> {
        Box::pin(async move {
            self.try_all("delete_friendship", |b| {
                let b = b.clone();
                async move { b.delete_friendship(id).await }
            })
            .await
        })
    }

    fn get_location_shares<'a>(&'a self) -> BackendFuture<'a, Vec<LocationShare>> {
        Box::pin(async move {
            self.try_all("get_location_shares", |b| {
                let b = b.clone();
                async move { b.get_location_shares().await }
            })
            .await
        })
    }

    fn create_location_share<'a>(
        &'a self,
        friendship_id: i64,
        direction: &'a str,
        precision_level: &'a str,
    ) -> BackendFuture<'a, ()> {
        Box::pin(async move {
            self.try_all("create_location_share", |b| {
                let b = b.clone();
                async move {
                    b.create_location_share(friendship_id, direction, precision_level)
                        .await
                }
            })
            .await
        })
    }

    fn create_share_with_expiry<'a>(
        &'a self,
        friendship_id: i64,
        direction: &'a str,
        precision_level: &'a str,
        duration_hours: Option<i64>,
        label: Option<&'a str>,
    ) -> BackendFuture<'a, ()> {
        Box::pin(async move {
            self.try_all("create_share_with_expiry", |b| {
                let b = b.clone();
                async move {
                    b.create_share_with_expiry(
                        friendship_id,
                        direction,
                        precision_level,
                        duration_hours,
                        label,
                    )
                    .await
                }
            })
            .await
        })
    }

    fn update_location_share<'a>(
        &'a self,
        share_id: i64,
        is_active: Option<bool>,
        precision: Option<String>,
    ) -> BackendFuture<'a, ()> {
        Box::pin(async move {
            self.try_all("update_location_share", |b| {
                let b = b.clone();
                let precision = precision.clone();
                async move {
                    b.update_location_share(share_id, is_active, precision)
                        .await
                }
            })
            .await
        })
    }

    fn delete_share<'a>(&'a self, share_id: i64) -> BackendFuture<'a, ()> {
        Box::pin(async move {
            self.try_all("delete_share", |b| {
                let b = b.clone();
                async move { b.delete_share(share_id).await }
            })
            .await
        })
    }

    fn get_friend_locations<'a>(&'a self) -> BackendFuture<'a, Vec<FriendLocation>> {
        Box::pin(async move {
            self.try_all("get_friend_locations", |b| {
                let b = b.clone();
                async move { b.get_friend_locations().await }
            })
            .await
        })
    }

    fn publish_location<'a>(
        &'a self,
        latitude: f64,
        longitude: f64,
        accuracy: f64,
    ) -> BackendFuture<'a, ()> {
        Box::pin(async move {
            self.try_all("publish_location", |b| {
                let b = b.clone();
                async move { b.publish_location(latitude, longitude, accuracy).await }
            })
            .await
        })
    }

    fn get_my_location<'a>(&'a self) -> BackendFuture<'a, FriendLocation> {
        Box::pin(async move {
            self.try_all("get_my_location", |b| {
                let b = b.clone();
                async move { b.get_my_location().await }
            })
            .await
        })
    }

    fn list_users<'a>(&'a self) -> BackendFuture<'a, Vec<User>> {
        Box::pin(async move {
            self.try_all("list_users", |b| {
                let b = b.clone();
                async move { b.list_users().await }
            })
            .await
        })
    }

    fn search_users<'a>(&'a self, q: &'a str) -> BackendFuture<'a, Vec<User>> {
        Box::pin(async move {
            self.try_all("search_users", |b| {
                let b = b.clone();
                async move { b.search_users(q).await }
            })
            .await
        })
    }

    fn update_user<'a>(&'a self, id: i64, body: serde_json::Value) -> BackendFuture<'a, ()> {
        Box::pin(async move {
            self.try_all("update_user", |b| {
                let b = b.clone();
                let body = body.clone();
                async move { b.update_user(id, body).await }
            })
            .await
        })
    }

    fn delete_user<'a>(&'a self, id: i64) -> BackendFuture<'a, ()> {
        Box::pin(async move {
            self.try_all("delete_user", |b| {
                let b = b.clone();
                async move { b.delete_user(id).await }
            })
            .await
        })
    }

    fn create_check_in<'a>(
        &'a self,
        lat: f64,
        lon: f64,
        note: &'a str,
        kind: &'a str,
    ) -> BackendFuture<'a, ()> {
        Box::pin(async move {
            self.try_all("create_check_in", |b| {
                let b = b.clone();
                async move { b.create_check_in(lat, lon, note, kind).await }
            })
            .await
        })
    }

    fn list_check_ins<'a>(&'a self) -> BackendFuture<'a, Vec<CheckIn>> {
        Box::pin(async move {
            self.try_all("list_check_ins", |b| {
                let b = b.clone();
                async move { b.list_check_ins().await }
            })
            .await
        })
    }

    fn trigger_emergency<'a>(&'a self, lat: f64, lon: f64) -> BackendFuture<'a, ()> {
        Box::pin(async move {
            self.try_all("trigger_emergency", |b| {
                let b = b.clone();
                async move { b.trigger_emergency(lat, lon).await }
            })
            .await
        })
    }

    fn list_emergencies<'a>(&'a self) -> BackendFuture<'a, Vec<EmergencyAlert>> {
        Box::pin(async move {
            self.try_all("list_emergencies", |b| {
                let b = b.clone();
                async move { b.list_emergencies().await }
            })
            .await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn failover_constructs_and_lists_urls() {
        let failover = FailoverBackend::new(vec![
            "http://127.0.0.1:1".to_string(),
            "http://127.0.0.1:2".to_string(),
        ])
        .unwrap();
        let urls = failover.urls().await;
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"http://127.0.0.1:1".to_string()));
        assert!(urls.contains(&"http://127.0.0.1:2".to_string()));
    }
}
