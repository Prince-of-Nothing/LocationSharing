//! Scraping detection — server-side anomaly detection for automated abuse.
//!
//! Tracks per-key (IP / user-agent / account) request patterns in sliding
//! windows and flags scraping: rapid sequential enumeration, missing/odd
//! user agents, and volume spikes. Decisions are advisory (log + block
//! list); the gateway still enforces rate limits.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// A single observation for a key.
#[derive(Debug, Clone)]
pub struct Observation {
    pub at: Instant,
    pub path: String,
    pub user_agent: String,
}

/// Detector state for one key.
#[derive(Debug, Clone)]
pub struct KeyState {
    pub window: VecDeque<Observation>,
    pub window_secs: u64,
    pub max_in_window: usize,
    pub blocked_until: Option<Instant>,
}

impl KeyState {
    pub fn new(window_secs: u64, max_in_window: usize) -> Self {
        Self {
            window: VecDeque::new(),
            window_secs,
            max_in_window,
            blocked_until: None,
        }
    }

    /// Record an observation; returns `true` if it looks like scraping.
    pub fn observe(&mut self, obs: Observation) -> bool {
        let now = obs.at;
        if let Some(until) = self.blocked_until {
            if now < until {
                return true;
            }
            self.blocked_until = None;
        }
        // Evict old.
        while let Some(front) = self.window.front() {
            if now.duration_since(front.at) > Duration::from_secs(self.window_secs) {
                self.window.pop_front();
            } else {
                break;
            }
        }
        let mut suspicious = false;
        // Volume spike.
        if self.window.len() + 1 >= self.max_in_window {
            suspicious = true;
        }
        // Missing/empty UA is suspicious for browser-facing endpoints.
        if obs.user_agent.trim().is_empty() {
            suspicious = true;
        }
        // Rapid enumeration: many distinct paths in a short window.
        let distinct_paths: std::collections::HashSet<&str> =
            self.window.iter().map(|o| o.path.as_str()).collect();
        if distinct_paths.len() > self.max_in_window / 2 + 1 && self.window.len() > 4 {
            suspicious = true;
        }
        self.window.push_back(obs);
        if suspicious {
            self.blocked_until = Some(now + Duration::from_secs(self.window_secs));
        }
        suspicious
    }

    pub fn is_blocked(&self, now: Instant) -> bool {
        self.blocked_until.map(|u| now < u).unwrap_or(false)
    }
}

/// Multi-key detector. Keys are typically IP or IP+UA.
#[derive(Debug)]
pub struct ScrapeDetector {
    pub window_secs: u64,
    pub max_in_window: usize,
    states: HashMap<String, KeyState>,
}

impl ScrapeDetector {
    pub fn new(window_secs: u64, max_in_window: usize) -> Self {
        Self {
            window_secs,
            max_in_window,
            states: HashMap::new(),
        }
    }

    pub fn observe(&mut self, key: &str, obs: Observation) -> bool {
        let state = self
            .states
            .entry(key.to_string())
            .or_insert_with(|| KeyState::new(self.window_secs, self.max_in_window));
        state.observe(obs)
    }

    pub fn is_blocked(&self, key: &str, now: Instant) -> bool {
        self.states
            .get(key)
            .map(|s| s.is_blocked(now))
            .unwrap_or(false)
    }

    pub fn prune(&mut self, now: Instant) {
        self.states.retain(|_, s| {
            s.blocked_until
                .map(|u| now < u)
                .unwrap_or(!s.window.is_empty())
        });
    }
}

/// Normalize a user-agent for allowlisting (lowercase, trimmed).
pub fn normalize_ua(ua: &str) -> String {
    ua.trim().to_lowercase()
}

/// True if the UA looks like a known automated scraper (no browser tokens).
pub fn looks_like_scraper(ua: &str) -> bool {
    let u = normalize_ua(ua);
    if u.is_empty() {
        return true;
    }
    // Common scraper signatures.
    let scraper_tokens = [
        "curl",
        "wget",
        "python-requests",
        "scrapy",
        "httpclient",
        "go-http-client",
        "java/",
        "okhttp",
        "axios",
        "node-fetch",
        "postman",
    ];
    scraper_tokens.iter().any(|t| u.contains(t))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn volume_spike_flags_scraping() {
        let mut d = ScrapeDetector::new(60, 3);
        let now = Instant::now();
        for i in 0..3 {
            let susp = d.observe(
                "ip1",
                Observation {
                    at: now + Duration::from_millis(i * 10),
                    path: format!("/api/v1/users/{i}"),
                    user_agent: "Mozilla/5.0".into(),
                },
            );
            if i == 2 {
                assert!(susp, "third request in window should flag");
            }
        }
        assert!(d.is_blocked("ip1", now + Duration::from_secs(1)));
    }

    #[test]
    fn empty_ua_is_suspicious() {
        let mut d = ScrapeDetector::new(60, 100);
        let now = Instant::now();
        let susp = d.observe(
            "ip2",
            Observation {
                at: now,
                path: "/api/v1/users/".into(),
                user_agent: "".into(),
            },
        );
        assert!(susp);
    }

    #[test]
    fn looks_like_scraper_detects_common_tools() {
        assert!(looks_like_scraper("curl/7.81.0"));
        assert!(looks_like_scraper("python-requests/2.28"));
        assert!(looks_like_scraper(""));
        assert!(!looks_like_scraper(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64)"
        ));
    }

    #[test]
    fn normalize_ua_trims_and_lowercases() {
        assert_eq!(normalize_ua("  Mozilla/5.0  "), "mozilla/5.0");
    }

    #[test]
    fn blocked_key_stays_blocked_until_expiry() {
        let mut d = ScrapeDetector::new(60, 2);
        let now = Instant::now();
        d.observe(
            "ip3",
            Observation {
                at: now,
                path: "/a".into(),
                user_agent: "Mozilla/5.0".into(),
            },
        );
        d.observe(
            "ip3",
            Observation {
                at: now + Duration::from_millis(1),
                path: "/b".into(),
                user_agent: "Mozilla/5.0".into(),
            },
        );
        assert!(d.is_blocked("ip3", now + Duration::from_secs(1)));
        assert!(!d.is_blocked("ip3", now + Duration::from_secs(120)));
    }
}
