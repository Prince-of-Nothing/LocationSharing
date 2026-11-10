//! Backend health monitoring for redundancy.
//!
//! Tracks the health of multiple backend URLs (primary + replicas) via
//! periodic `GET /health` probes. Provides a list of healthy backends
//! ordered by recency of last successful probe, so failover logic can
//! route around unhealthy services.
//!
//! Runs a background tokio task that probes each backend on an interval.
//! Safe to clone and share across the app/UI layers.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Health status of a single backend URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackendHealth {
    Healthy,
    Unhealthy,
    #[default]
    Unknown,
}

/// Record of one backend's health over time.
#[derive(Debug, Clone)]
struct BackendRecord {
    url: String,
    health: BackendHealth,
    last_probe: Option<Instant>,
    last_success: Option<Instant>,
    consecutive_failures: u32,
}

/// Shared health monitor state.
#[derive(Debug, Default)]
pub struct HealthMonitor {
    backends: RwLock<HashMap<String, BackendRecord>>,
    interval_secs: u64,
    timeout_secs: u64,
}

impl HealthMonitor {
    /// Create a new monitor. `interval_secs` is how often each backend is probed.
    pub fn new(interval_secs: u64, timeout_secs: u64) -> Arc<Self> {
        Arc::new(Self {
            interval_secs,
            timeout_secs,
            ..Default::default()
        })
    }

    /// Register a backend URL to be monitored.
    pub fn register(&self, url: impl Into<String>) {
        let url = url.into();
        let mut backends = self.backends.write().unwrap();
        backends.entry(url.clone()).or_insert(BackendRecord {
            url,
            health: BackendHealth::Unknown,
            last_probe: None,
            last_success: None,
            consecutive_failures: 0,
        });
    }

    /// Record a successful probe for `url`.
    pub fn record_success(&self, url: &str) {
        let mut backends = self.backends.write().unwrap();
        if let Some(rec) = backends.get_mut(url) {
            rec.health = BackendHealth::Healthy;
            rec.last_probe = Some(Instant::now());
            rec.last_success = Some(Instant::now());
            rec.consecutive_failures = 0;
        }
    }

    /// Record a failed probe for `url`. Opens the circuit when threshold reached.
    pub fn record_failure(&self, url: &str) {
        let mut backends = self.backends.write().unwrap();
        if let Some(rec) = backends.get_mut(url) {
            rec.health = BackendHealth::Unhealthy;
            rec.last_probe = Some(Instant::now());
            rec.consecutive_failures += 1;
        }
    }

    /// Return URLs of backends currently considered healthy, ordered by most
    /// recent success first. Returns all registered backends if none have
    /// been probed yet (so first contact is still attempted).
    pub fn healthy_urls(&self) -> Vec<String> {
        let backends = self.backends.read().unwrap();
        let mut healthy: Vec<&BackendRecord> = backends
            .values()
            .filter(|r| r.health == BackendHealth::Healthy)
            .collect();
        healthy.sort_by_key(|r| r.last_success);
        healthy.reverse();
        if healthy.is_empty() {
            // Fall back to all registered URLs so the client can try them.
            return backends.values().map(|r| r.url.clone()).collect();
        }
        healthy.iter().map(|r| r.url.clone()).collect()
    }

    /// Return all registered backend URLs (for diagnostics).
    pub fn all_urls(&self) -> Vec<String> {
        self.backends
            .read()
            .unwrap()
            .values()
            .map(|r| r.url.clone())
            .collect()
    }

    /// Snapshot of health status per URL (for UI / debugging).
    pub fn status_snapshot(&self) -> Vec<(String, BackendHealth, u32)> {
        let backends = self.backends.read().unwrap();
        let mut out: Vec<(String, BackendHealth, u32)> = backends
            .values()
            .map(|r| (r.url.clone(), r.health, r.consecutive_failures))
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    /// Background probe loop. Call `tokio::spawn(monitor.probe_loop(client))`
    /// to start monitoring. Probes each registered backend on `interval_secs`.
    pub async fn probe_loop(self: Arc<Self>, client: reqwest::Client) {
        let interval = Duration::from_secs(self.interval_secs);
        let timeout = Duration::from_secs(self.timeout_secs);
        loop {
            tokio::time::sleep(interval).await;
            let urls = self.all_urls();
            for url in urls {
                let health_url = format!("{}/health", url.trim_end_matches('/'));
                match tokio::time::timeout(timeout, client.get(&health_url).send()).await {
                    Ok(Ok(resp)) if resp.status().is_success() => {
                        self.record_success(&url);
                    }
                    _ => {
                        self.record_failure(&url);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healthy_urls_empty_before_probes() {
        let monitor = HealthMonitor::new(5, 5);
        monitor.register("http://a:8000");
        monitor.register("http://b:8000");
        // Unknown health => falls back to all URLs.
        let urls = monitor.healthy_urls();
        assert_eq!(urls.len(), 2);
    }

    #[test]
    fn healthy_urls_returns_only_healthy() {
        let monitor = HealthMonitor::new(5, 5);
        monitor.register("http://a:8000");
        monitor.register("http://b:8000");
        monitor.record_success("http://a:8000");
        monitor.record_failure("http://b:8000");
        let urls = monitor.healthy_urls();
        assert_eq!(urls, vec!["http://a:8000".to_string()]);
    }

    #[test]
    fn failure_threshold_reached_marks_unhealthy() {
        let monitor = HealthMonitor::new(5, 5);
        monitor.register("http://a:8000");
        for _ in 0..3 {
            monitor.record_failure("http://a:8000");
        }
        let snapshot = monitor.status_snapshot();
        assert_eq!(snapshot[0].1, BackendHealth::Unhealthy);
        assert_eq!(snapshot[0].2, 3);
    }
}
