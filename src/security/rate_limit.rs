//! Security — Client-side rate limiting.
//!
//! Rate limits API calls from the client to prevent abuse, accidental
//! loops, and to respect Nominatim/OSRM/Overpass usage policies.

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Rate limit configuration for an endpoint.
#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub window_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 60,
            window_seconds: 60,
        }
    }
}

/// Default rate limit configs for known endpoints.
pub mod default_configs {
    use super::RateLimitConfig;

    pub static NOMINATIM: RateLimitConfig = RateLimitConfig {
        max_requests: 1,
        window_seconds: 1,
    };
    pub static OSRM: RateLimitConfig = RateLimitConfig {
        max_requests: 2,
        window_seconds: 1,
    };
    pub static OVERPASS: RateLimitConfig = RateLimitConfig {
        max_requests: 1,
        window_seconds: 5,
    };
    pub static API_LOGIN: RateLimitConfig = RateLimitConfig {
        max_requests: 5,
        window_seconds: 60,
    };
    pub static API_GENERAL: RateLimitConfig = RateLimitConfig {
        max_requests: 60,
        window_seconds: 60,
    };
}

/// Record of requests within a time window.
#[derive(Debug, Clone)]
struct RequestWindow {
    requests: Vec<Instant>,
    window_seconds: u64,
}

impl RequestWindow {
    fn new(window_seconds: u64) -> Self {
        Self {
            requests: Vec::new(),
            window_seconds,
        }
    }

    fn cleanup(&mut self, now: Instant) {
        let cutoff = now - Duration::from_secs(self.window_seconds);
        self.requests.retain(|t| *t > cutoff);
    }

    fn can_add(&mut self, now: Instant, max_requests: u32) -> bool {
        self.cleanup(now);
        self.requests.len() < max_requests as usize
    }

    fn add(&mut self, now: Instant) {
        self.cleanup(now);
        self.requests.push(now);
    }

    fn remaining(&mut self, now: Instant, max_requests: u32) -> u32 {
        self.cleanup(now);
        let used = self.requests.len() as u32;
        max_requests.saturating_sub(used)
    }

    fn retry_after(&mut self, now: Instant) -> Option<Duration> {
        self.cleanup(now);
        if self.requests.is_empty() {
            return None;
        }
        let oldest = *self.requests.iter().min().unwrap();
        let retry_at = oldest + Duration::from_secs(self.window_seconds);
        if retry_at > now {
            Some(retry_at - now)
        } else {
            None
        }
    }
}

/// Rate limiter that tracks requests per endpoint.
pub struct RateLimiter {
    configs: std::collections::HashMap<&'static str, RateLimitConfig>,
    windows: Mutex<std::collections::HashMap<String, RequestWindow>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            configs: std::collections::HashMap::new(),
            windows: Mutex::new(std::collections::HashMap::new()),
        }
    }

    pub fn with_defaults() -> Self {
        let mut configs = std::collections::HashMap::new();
        configs.insert("nominatim", default_configs::NOMINATIM);
        configs.insert("osrm", default_configs::OSRM);
        configs.insert("overpass", default_configs::OVERPASS);
        configs.insert("api_login", default_configs::API_LOGIN);
        configs.insert("api_general", default_configs::API_GENERAL);
        Self {
            configs,
            windows: Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Register a custom rate limit config for an endpoint.
    pub fn register_config(&mut self, endpoint: &'static str, config: RateLimitConfig) {
        self.configs.insert(endpoint, config);
    }

    /// Check if a request to the endpoint is allowed.
    pub fn check(&self, endpoint: &str) -> RateLimitResult {
        let config = self
            .configs
            .get(endpoint)
            .copied()
            .unwrap_or(RateLimitConfig::default());

        let now = Instant::now();
        let mut windows = self.windows.lock().unwrap();
        let window = windows
            .entry(endpoint.to_string())
            .or_insert_with(|| RequestWindow::new(config.window_seconds));

        if window.can_add(now, config.max_requests) {
            window.add(now);
            let remaining = window.remaining(now, config.max_requests);
            RateLimitResult::Allowed { remaining }
        } else {
            let retry_after = window.retry_after(now);
            RateLimitResult::RateLimited { retry_after }
        }
    }

    /// Get remaining requests for an endpoint without consuming a slot.
    pub fn remaining(&mut self, endpoint: &str) -> u32 {
        let config = self
            .configs
            .get(endpoint)
            .copied()
            .unwrap_or(RateLimitConfig::default());

        let now = Instant::now();
        let mut windows = self.windows.lock().unwrap();
        if let Some(window) = windows.get_mut(endpoint) {
            window.remaining(now, config.max_requests)
        } else {
            config.max_requests
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Result of a rate limit check.
#[derive(Debug, Clone)]
pub enum RateLimitResult {
    Allowed { remaining: u32 },
    RateLimited { retry_after: Option<Duration> },
}

impl RateLimitResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn allows_requests_within_limit() {
        let limiter = RateLimiter::with_defaults();
        for _ in 0..5 {
            assert!(limiter.check("api_general").is_allowed());
        }
    }

    #[test]
    fn rejects_requests_over_limit() {
        let mut limiter = RateLimiter::new();
        limiter.register_config(
            "test",
            RateLimitConfig {
                max_requests: 2,
                window_seconds: 60,
            },
        );

        assert!(limiter.check("test").is_allowed());
        assert!(limiter.check("test").is_allowed());
        assert!(!limiter.check("test").is_allowed());
    }

    #[test]
    fn separate_endpoints_have_separate_limits() {
        let limiter = RateLimiter::new();
        assert!(limiter.check("api_login").is_allowed());
        assert!(limiter.check("api_general").is_allowed());
    }

    #[test]
    fn remaining_count_decreases() {
        let mut limiter = RateLimiter::new();
        limiter.register_config(
            "test",
            RateLimitConfig {
                max_requests: 3,
                window_seconds: 60,
            },
        );

        assert_eq!(limiter.remaining("test"), 3);
        limiter.check("test");
        assert_eq!(limiter.remaining("test"), 2);
        limiter.check("test");
        assert_eq!(limiter.remaining("test"), 1);
    }

    #[test]
    fn rate_limited_reports_retry_after() {
        let mut limiter = RateLimiter::new();
        limiter.register_config(
            "test",
            RateLimitConfig {
                max_requests: 1,
                window_seconds: 60,
            },
        );

        assert!(limiter.check("test").is_allowed());
        let result = limiter.check("test");
        assert!(!result.is_allowed());
        match result {
            RateLimitResult::RateLimited { retry_after } => {
                assert!(retry_after.is_some());
                assert!(retry_after.unwrap() <= Duration::from_secs(60));
            }
            RateLimitResult::Allowed { .. } => panic!("Expected rate limited"),
        }
    }
}
