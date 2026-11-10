//! Webapp configuration resolved in a backend-architecture-friendly way.
//!
//! Resolution order (browser/wasm builds should inject via host config; native
//! stubs use env): explicit value > `PINPOINT_BACKEND` > platform default.
//! Adapters may also read `PINPOINT_BACKEND_KIND` in the future.

use crate::backend::{BackendConfig, BackendKind};
use crate::webapp::WebAppKind;

#[derive(Debug, Clone)]
pub struct WebAppConfig {
    pub kind: WebAppKind,
    pub backend: BackendConfig,
    /// Optional share/check-in token for kiosk flows (`webk`).
    pub share_token: Option<String>,
    /// Feature flags/toggles (non-secret). Unknown keys are ignored.
    pub features: std::collections::BTreeMap<String, String>,
}

impl WebAppConfig {
    pub fn new(kind: WebAppKind) -> Self {
        Self {
            kind,
            backend: BackendConfig::from_env(),
            share_token: None,
            features: std::collections::BTreeMap::new(),
        }
    }

    pub fn with_backend_url(mut self, url: impl Into<String>) -> Self {
        self.backend.base_url = url.into();
        self
    }

    pub fn with_backend_kind(mut self, kind: BackendKind) -> Self {
        self.backend.kind = kind;
        self
    }

    pub fn with_share_token(mut self, token: impl Into<String>) -> Self {
        let t = token.into();
        self.share_token = if t.trim().is_empty() { None } else { Some(t) };
        self
    }
}

/// Parse a simple `key=value` query fragment/token source without leaking
/// values into structured logs. Intended for kiosk share links.
pub fn parse_query_param(query: &str, key: &str) -> Option<String> {
    for pair in query.trim_start_matches('?').split('&') {
        let mut it = pair.splitn(2, '=');
        let k = it.next().unwrap_or("");
        let v = it.next().unwrap_or("");
        if k == key && !v.is_empty() {
            return Some(percent_decode(v));
        }
    }
    None
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Ok(h), Ok(l)) = (
                std::str::from_utf8(&bytes[i + 1..i + 2]),
                std::str::from_utf8(&bytes[i + 2..i + 3]),
            ) {
                if let (Some(h), Some(l)) = (
                    u8::from_str_radix(h, 16).ok(),
                    u8::from_str_radix(l, 16).ok(),
                ) {
                    out.push((h << 4) | l);
                    i += 3;
                    continue;
                }
            }
        } else if bytes[i] == b'+' {
            out.push(b' ');
            i += 1;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_share_token_from_query() {
        assert_eq!(
            parse_query_param("?token=abc123&x=1", "token"),
            Some("abc123".into())
        );
        assert_eq!(
            parse_query_param("x=1&token=hello%20world", "token"),
            Some("hello world".into())
        );
        assert_eq!(parse_query_param("?x=1", "token"), None);
    }
}
