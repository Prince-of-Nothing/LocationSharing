//! Circuit breaker for backend redundancy.
//!
//! Tracks per-backend failures and short-circuits requests when a backend
//! is unhealthy, preventing cascading failures and giving failing services
//! time to recover. States:
//! - `Closed`  — requests flow normally; failures increment the counter.
//! - `Open`    — requests are rejected immediately without calling the backend.
//! - `HalfOpen` — a single trial request is allowed; success closes, failure reopens.
//!
//! Thread-safe via `Arc<Mutex<>>` so multiple clients can share one breaker.
//!
//! Example:
//!   let cb = Arc::new(CircuitBreaker::new("auth-1", 5, Duration::from_secs(30)));
//!   if cb.call(|| async { backend.request(...) }).await.is_err() {
//!       // backend or circuit is unhealthy — try a replica
//!   }

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Current state of the circuit breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Result of a circuit-breaker guarded call.
#[derive(Debug)]
pub enum CircuitResult<T> {
    Success(T),
    CircuitOpen,
    BackendError(anyhow::Error),
}

/// Configuration for a circuit breaker.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening.
    pub failure_threshold: u32,
    /// Time to wait in Open state before trying HalfOpen.
    pub reset_timeout: Duration,
    /// Maximum calls allowed in HalfOpen before deciding (1 = single probe).
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            reset_timeout: Duration::from_secs(30),
            half_open_max_calls: 1,
        }
    }
}

/// A circuit breaker guarding one backend endpoint.
#[derive(Debug)]
pub struct CircuitBreaker {
    name: String,
    config: CircuitBreakerConfig,
    state: Mutex<CircuitState>,
    failures: Mutex<u32>,
    half_open_calls: Mutex<u32>,
    opened_at: Mutex<Option<Instant>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker in `Closed` state.
    pub fn new(name: impl Into<String>, config: CircuitBreakerConfig) -> Arc<Self> {
        Arc::new(Self {
            name: name.into(),
            config,
            state: Mutex::new(CircuitState::Closed),
            failures: Mutex::new(0),
            half_open_calls: Mutex::new(0),
            opened_at: Mutex::new(None),
        })
    }

    /// Create with default config.
    pub fn default_with_name(name: impl Into<String>) -> Arc<Self> {
        Self::new(name, CircuitBreakerConfig::default())
    }

    /// Execute `f` if the circuit allows it. On success, record success.
    /// On failure, record failure and possibly open the circuit.
    pub async fn call<F, Fut, T>(&self, f: F) -> CircuitResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<T>>,
    {
        let should_proceed = self.can_proceed();
        if !should_proceed {
            return CircuitResult::CircuitOpen;
        }

        let result = f().await;

        match result {
            Ok(value) => {
                self.on_success();
                CircuitResult::Success(value)
            }
            Err(e) => {
                self.on_failure();
                CircuitResult::BackendError(e)
            }
        }
    }

    /// Record a successful call: reset failure count, close circuit if half-open.
    pub fn on_success(&self) {
        let mut state = self.state.lock().unwrap();
        let mut failures = self.failures.lock().unwrap();
        let mut half_open_calls = self.half_open_calls.lock().unwrap();

        *failures = 0;
        if *state == CircuitState::HalfOpen {
            *half_open_calls += 1;
            if *half_open_calls >= self.config.half_open_max_calls {
                *state = CircuitState::Closed;
                *half_open_calls = 0;
                *self.opened_at.lock().unwrap() = None;
            }
        }
    }

    /// Record a failed call: increment failures, open circuit if threshold reached.
    pub fn on_failure(&self) {
        let mut state = self.state.lock().unwrap();
        let mut failures = self.failures.lock().unwrap();
        let mut half_open_calls = self.half_open_calls.lock().unwrap();
        let mut opened_at = self.opened_at.lock().unwrap();

        *failures += 1;

        match *state {
            CircuitState::Closed => {
                if *failures >= self.config.failure_threshold {
                    *state = CircuitState::Open;
                    *opened_at = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                *state = CircuitState::Open;
                *opened_at = Some(Instant::now());
                *half_open_calls = 0;
            }
            CircuitState::Open => {
                // Extend the open window on repeated failures while open.
                *opened_at = Some(Instant::now());
            }
        }
    }

    /// Check whether a call should be allowed through right now.
    /// Handles the Open → HalfOpen transition when the reset timeout expires.
    pub fn can_proceed(&self) -> bool {
        let state = *self.state.lock().unwrap();
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                let opened = self.opened_at.lock().unwrap();
                match *opened {
                    Some(at) if at.elapsed() >= self.config.reset_timeout => {
                        drop(opened);
                        let mut st = self.state.lock().unwrap();
                        if *st == CircuitState::Open {
                            *st = CircuitState::HalfOpen;
                            let mut hc = self.half_open_calls.lock().unwrap();
                            *hc = 0;
                        }
                        true
                    }
                    _ => false,
                }
            }
            CircuitState::HalfOpen => {
                let hc = self.half_open_calls.lock().unwrap();
                *hc < self.config.half_open_max_calls
            }
        }
    }

    /// Current state (for diagnostics / UI).
    pub fn state(&self) -> CircuitState {
        *self.state.lock().unwrap()
    }

    /// Consecutive failure count.
    pub fn failures(&self) -> u32 {
        *self.failures.lock().unwrap()
    }

    /// Backend name for diagnostics.
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn closed_circuit_passes_through() {
        let cb = CircuitBreaker::default_with_name("test");
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let res = cb
            .call(move || {
                let c = c.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok::<_, anyhow::Error>(42i32)
                }
            })
            .await;
        match res {
            CircuitResult::Success(42) => {}
            other => panic!("expected Success(42), got {:?}", other),
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn repeated_failures_open_circuit() {
        let cb = CircuitBreaker::new(
            "test",
            CircuitBreakerConfig {
                failure_threshold: 3,
                reset_timeout: Duration::from_secs(60),
                half_open_max_calls: 1,
            },
        );
        let fails = Arc::new(AtomicU32::new(0));
        let f = fails.clone();
        for _ in 0..3 {
            let f = f.clone();
            let res = cb
                .call(move || {
                    let f = f.clone();
                    async move {
                        f.fetch_add(1, Ordering::SeqCst);
                        Err::<i32, anyhow::Error>(anyhow::anyhow!("boom"))
                    }
                })
                .await;
            match res {
                CircuitResult::BackendError(_) => {}
                other => panic!("expected BackendError, got {:?}", other),
            }
        }
        assert_eq!(cb.state(), CircuitState::Open);
        assert_eq!(cb.failures(), 3);

        // Next call should be short-circuited (not reach backend).
        let before = fails.load(Ordering::SeqCst);
        let res = cb
            .call(|| async { Err::<i32, anyhow::Error>(anyhow::anyhow!("boom")) })
            .await;
        match res {
            CircuitResult::CircuitOpen => {}
            other => panic!("expected CircuitOpen, got {:?}", other),
        }
        assert_eq!(fails.load(Ordering::SeqCst), before);
    }

    #[tokio::test]
    async fn half_open_probe_success_closes() {
        let cb = CircuitBreaker::new(
            "test",
            CircuitBreakerConfig {
                failure_threshold: 1,
                reset_timeout: Duration::from_millis(1),
                half_open_max_calls: 1,
            },
        );
        // Force open with one failure.
        cb.call(|| async { Err::<i32, anyhow::Error>(anyhow::anyhow!("boom")) })
            .await;
        assert_eq!(cb.state(), CircuitState::Open);

        // Wait for reset timeout.
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Half-open probe succeeds -> closed.
        let res = cb.call(|| async { Ok::<_, anyhow::Error>(7i32) }).await;
        match res {
            CircuitResult::Success(7) => {}
            other => panic!("expected Success(7), got {:?}", other),
        }
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn half_open_probe_failure_reopens() {
        let cb = CircuitBreaker::new(
            "test",
            CircuitBreakerConfig {
                failure_threshold: 1,
                reset_timeout: Duration::from_millis(1),
                half_open_max_calls: 1,
            },
        );
        cb.call(|| async { Err::<i32, anyhow::Error>(anyhow::anyhow!("boom")) })
            .await;
        tokio::time::sleep(Duration::from_millis(10)).await;
        let res = cb
            .call(|| async { Err::<i32, anyhow::Error>(anyhow::anyhow!("boom")) })
            .await;
        match res {
            CircuitResult::BackendError(_) => {}
            other => panic!("expected BackendError, got {:?}", other),
        }
        assert_eq!(cb.state(), CircuitState::Open);
    }
}
