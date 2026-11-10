//! REST adapter for kiosk (`webk`) backend operations.
//!
//! Isolates kiosk endpoint/method/payload details so a backend architecture
//! change only requires updating this adapter (or adding a new one), not the
//! kiosk UI/controller logic.

use crate::backend::BackendFuture;
use crate::webapp::kiosk::{KioskBackend, KioskCheckInRequest, KioskRestRoutes, KioskShareView};
use std::time::Duration;

/// Configurable REST adapter for kiosk endpoints.
///
/// Endpoint paths/methods/payload mapping are centralized here. When backend
/// architecture changes, update `KioskRestRoutes` and/or the request/response
/// mapping in this file (or add a new adapter).
#[derive(Debug, Clone)]
pub struct RestKioskAdapter {
    pub base_url: String,
    pub routes: KioskRestRoutes,
    http: reqwest::Client,
}

impl RestKioskAdapter {
    pub fn new(base_url: impl Into<String>, routes: KioskRestRoutes) -> anyhow::Result<Self> {
        let base_url = base_url.into().trim_end_matches('/').to_string();
        // Reuse the same transport policy mindset as the main API client.
        crate::api::validate_backend_url(&base_url)?;
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()?;
        Ok(Self {
            base_url,
            routes,
            http,
        })
    }

    pub fn resolve_url(&self, token: &str) -> String {
        format!("{}{}", self.base_url, self.routes.resolve_share_path(token))
    }

    pub fn check_in_url(&self) -> String {
        format!("{}{}", self.base_url, self.routes.submit_check_in)
    }

    async fn parse_error(response: reqwest::Response, fallback: &str) -> anyhow::Error {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        if text.trim().is_empty() {
            anyhow::anyhow!("{} {fallback}", status.as_u16())
        } else {
            // Best-effort: surface compact JSON/text without dumping internals.
            let snippet: String = text.chars().take(200).collect();
            anyhow::anyhow!("{status}: {snippet}")
        }
    }
}

impl KioskBackend for RestKioskAdapter {
    fn resolve_share_token<'a>(&'a self, token: &'a str) -> BackendFuture<'a, KioskShareView> {
        let url = self.resolve_url(token);
        let token_owned = token.to_string();
        Box::pin(async move {
            let resp = self.http.get(&url).send().await?;
            if !resp.status().is_success() {
                return Err(Self::parse_error(resp, "Failed to resolve share token").await);
            }
            let value: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
            // Flexible mapping: accept either a direct view shape or nested/common fields.
            let view = if let Ok(v) = serde_json::from_value::<KioskShareView>(value.clone()) {
                KioskShareView {
                    token: token_owned,
                    ..v
                }
            } else {
                let label = value
                    .get("label")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let expires_at = value
                    .get("expires_at")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let consent_required = value
                    .get("consent_required")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let location = value.get("location").and_then(|v| {
                    serde_json::from_value::<crate::models::FriendLocation>(v.clone()).ok()
                });
                KioskShareView {
                    token: token_owned,
                    label,
                    expires_at,
                    location,
                    consent_required,
                }
            };
            Ok(view)
        })
    }

    fn submit_check_in<'a>(&'a self, request: KioskCheckInRequest) -> BackendFuture<'a, ()> {
        let url = self.check_in_url();
        Box::pin(async move {
            let resp = self.http.post(&url).json(&request).send().await?;
            if !resp.status().is_success() {
                return Err(Self::parse_error(resp, "Failed to submit kiosk check-in").await);
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_urls_from_routes() {
        let a =
            RestKioskAdapter::new("http://localhost:8000/", KioskRestRoutes::default()).unwrap();
        assert!(a.resolve_url("abc").starts_with("http://localhost:8000/"));
        assert_eq!(
            a.check_in_url(),
            "http://localhost:8000/api/v1/kiosk/check-ins"
        );
    }

    #[test]
    fn rejects_plaintext_non_loopback() {
        assert!(RestKioskAdapter::new("http://example.com", KioskRestRoutes::default()).is_err());
    }
}
