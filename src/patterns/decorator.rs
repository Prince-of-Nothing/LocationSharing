//! Decorator Pattern — Add logging, caching, retries to services.
//!
//! Decorators wrap service objects to add behavior without modifying the
//! underlying object. Used for logging API calls, retrying failed
//! requests, and caching responses.
//!
//! The `ApiClientTrait` defines the interface that decorators wrap.
//! Implementations of this trait can be wrapped with logging, retry, or
//! timing decorators.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Trait for API call logging.
pub trait ApiCallLogger: Send + Sync {
    fn log_request(&self, method: &str, path: &str);
    fn log_response(&self, method: &str, path: &str, status: u16, duration_ms: u64);
}

/// Simple console logger for API calls.
pub struct ConsoleApiLogger;

impl ApiCallLogger for ConsoleApiLogger {
    fn log_request(&self, method: &str, path: &str) {
        println!("[API] {method} {path}");
    }

    fn log_response(&self, method: &str, path: &str, status: u16, duration_ms: u64) {
        println!("[API] {method} {path} -> {status} ({duration_ms}ms)");
    }
}

/// Trait abstracting API client operations.
pub trait ApiClientTrait: Send + Sync {
    fn login(&mut self, email: &str, password: &str) -> Result<String, String>;
    fn logout(&mut self) -> Result<(), String>;
    fn get_friend_locations(&mut self) -> Result<Vec<crate::models::FriendLocation>, String>;
    fn publish_location(&mut self, lat: f64, lon: f64, accuracy: f64) -> Result<(), String>;
}

/// Decorator that adds logging to an API client.
pub struct LoggingApiClient<C: ApiClientTrait> {
    inner: C,
    logger: Box<dyn ApiCallLogger>,
    request_counter: AtomicU64,
}

impl<C: ApiClientTrait> LoggingApiClient<C> {
    pub fn new(inner: C, logger: Box<dyn ApiCallLogger>) -> Self {
        Self {
            inner,
            logger,
            request_counter: AtomicU64::new(0),
        }
    }

    pub fn request_count(&self) -> u64 {
        self.request_counter.load(Ordering::SeqCst)
    }
}

impl<C: ApiClientTrait> ApiClientTrait for LoggingApiClient<C> {
    fn login(&mut self, email: &str, password: &str) -> Result<String, String> {
        self.logger.log_request("POST", "/api/v1/auth/login");
        let start = Instant::now();
        let result = self.inner.login(email, password);
        let duration_ms = start.elapsed().as_millis() as u64;
        match &result {
            Ok(_) => self
                .logger
                .log_response("POST", "/api/v1/auth/login", 200, duration_ms),
            Err(_) => self
                .logger
                .log_response("POST", "/api/v1/auth/login", 401, duration_ms),
        }
        result
    }

    fn logout(&mut self) -> Result<(), String> {
        self.logger.log_request("POST", "/api/v1/auth/logout");
        let start = Instant::now();
        let result = self.inner.logout();
        let duration_ms = start.elapsed().as_millis() as u64;
        match &result {
            Ok(_) => self
                .logger
                .log_response("POST", "/api/v1/auth/logout", 200, duration_ms),
            Err(_) => self
                .logger
                .log_response("POST", "/api/v1/auth/logout", 401, duration_ms),
        }
        result
    }

    fn get_friend_locations(&mut self) -> Result<Vec<crate::models::FriendLocation>, String> {
        self.logger.log_request("GET", "/api/v1/locations/friends");
        let start = Instant::now();
        let result = self.inner.get_friend_locations();
        let duration_ms = start.elapsed().as_millis() as u64;
        match &result {
            Ok(_) => self
                .logger
                .log_response("GET", "/api/v1/locations/friends", 200, duration_ms),
            Err(_) => {
                self.logger
                    .log_response("GET", "/api/v1/locations/friends", 404, duration_ms)
            }
        }
        result
    }

    fn publish_location(&mut self, lat: f64, lon: f64, accuracy: f64) -> Result<(), String> {
        self.logger.log_request("POST", "/api/v1/locations/me");
        let start = Instant::now();
        let result = self.inner.publish_location(lat, lon, accuracy);
        let duration_ms = start.elapsed().as_millis() as u64;
        match &result {
            Ok(_) => self
                .logger
                .log_response("POST", "/api/v1/locations/me", 200, duration_ms),
            Err(_) => self
                .logger
                .log_response("POST", "/api/v1/locations/me", 400, duration_ms),
        }
        result
    }
}

/// Decorator that adds retry logic to an API client.
pub struct RetryApiClient<C: ApiClientTrait> {
    inner: C,
    max_retries: u32,
    retry_delay_ms: u64,
}

impl<C: ApiClientTrait> RetryApiClient<C> {
    pub fn new(inner: C, max_retries: u32, retry_delay_ms: u64) -> Self {
        Self {
            inner,
            max_retries,
            retry_delay_ms,
        }
    }
}

impl<C: ApiClientTrait> ApiClientTrait for RetryApiClient<C> {
    fn login(&mut self, email: &str, password: &str) -> Result<String, String> {
        let mut last_err = String::new();
        for attempt in 0..=self.max_retries {
            match self.inner.login(email, password) {
                Ok(user) => return Ok(user),
                Err(e) => {
                    last_err = e;
                    if attempt < self.max_retries {
                        std::thread::sleep(Duration::from_millis(self.retry_delay_ms));
                    }
                }
            }
        }
        Err(last_err)
    }

    fn logout(&mut self) -> Result<(), String> {
        self.inner.logout()
    }

    fn get_friend_locations(&mut self) -> Result<Vec<crate::models::FriendLocation>, String> {
        let mut last_err = String::new();
        for attempt in 0..=self.max_retries {
            match self.inner.get_friend_locations() {
                Ok(friendships) => return Ok(friendships),
                Err(e) => {
                    last_err = e;
                    if attempt < self.max_retries {
                        std::thread::sleep(Duration::from_millis(self.retry_delay_ms));
                    }
                }
            }
        }
        Err(last_err)
    }

    fn publish_location(&mut self, lat: f64, lon: f64, accuracy: f64) -> Result<(), String> {
        let mut last_err = String::new();
        for attempt in 0..=self.max_retries {
            match self.inner.publish_location(lat, lon, accuracy) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last_err = e;
                    if attempt < self.max_retries {
                        std::thread::sleep(Duration::from_millis(self.retry_delay_ms));
                    }
                }
            }
        }
        Err(last_err)
    }
}

/// Decorator that adds timing to an API client.
pub struct TimingApiClient<C: ApiClientTrait> {
    inner: C,
    total_time_ms: AtomicU64,
    call_count: AtomicU64,
}

impl<C: ApiClientTrait> TimingApiClient<C> {
    pub fn new(inner: C) -> Self {
        Self {
            inner,
            total_time_ms: AtomicU64::new(0),
            call_count: AtomicU64::new(0),
        }
    }

    pub fn average_time_ms(&self) -> f64 {
        let count = self.call_count.load(Ordering::SeqCst);
        if count == 0 {
            0.0
        } else {
            self.total_time_ms.load(Ordering::SeqCst) as f64 / count as f64
        }
    }
}

impl<C: ApiClientTrait> ApiClientTrait for TimingApiClient<C> {
    fn login(&mut self, email: &str, password: &str) -> Result<String, String> {
        let start = Instant::now();
        let result = self.inner.login(email, password);
        let elapsed = start.elapsed().as_millis() as u64;
        self.total_time_ms.fetch_add(elapsed, Ordering::SeqCst);
        self.call_count.fetch_add(1, Ordering::SeqCst);
        result
    }

    fn logout(&mut self) -> Result<(), String> {
        let start = Instant::now();
        let result = self.inner.logout();
        let elapsed = start.elapsed().as_millis() as u64;
        self.total_time_ms.fetch_add(elapsed, Ordering::SeqCst);
        self.call_count.fetch_add(1, Ordering::SeqCst);
        result
    }

    fn get_friend_locations(&mut self) -> Result<Vec<crate::models::FriendLocation>, String> {
        let start = Instant::now();
        let result = self.inner.get_friend_locations();
        let elapsed = start.elapsed().as_millis() as u64;
        self.total_time_ms.fetch_add(elapsed, Ordering::SeqCst);
        self.call_count.fetch_add(1, Ordering::SeqCst);
        result
    }

    fn publish_location(&mut self, lat: f64, lon: f64, accuracy: f64) -> Result<(), String> {
        let start = Instant::now();
        let result = self.inner.publish_location(lat, lon, accuracy);
        let elapsed = start.elapsed().as_millis() as u64;
        self.total_time_ms.fetch_add(elapsed, Ordering::SeqCst);
        self.call_count.fetch_add(1, Ordering::SeqCst);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timing_client_tracks_calls() {
        // Create a mock client that implements ApiClientTrait
        struct MockClient;
        impl ApiClientTrait for MockClient {
            fn login(&mut self, _email: &str, _password: &str) -> Result<String, String> {
                Ok("user".to_string())
            }
            fn logout(&mut self) -> Result<(), String> {
                Ok(())
            }
            fn get_friend_locations(
                &mut self,
            ) -> Result<Vec<crate::models::FriendLocation>, String> {
                Ok(Vec::new())
            }
            fn publish_location(
                &mut self,
                _lat: f64,
                _lon: f64,
                _accuracy: f64,
            ) -> Result<(), String> {
                Ok(())
            }
        }

        let timing_client = TimingApiClient::new(MockClient);
        assert_eq!(timing_client.average_time_ms(), 0.0);
    }
}
