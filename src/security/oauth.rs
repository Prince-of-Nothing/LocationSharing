//! OAuth2 (authorization code + PKCE) helpers.
//!
//! Secrets never ship in the binary: client IDs come from env, client
//! secrets (if any) are exchanged server-side only. PKCE (S256) is
//! mandatory for public clients. State is opaque + CSRF-bound.

use base64::Engine;
use sha2::{Digest, Sha256};

/// A configured OAuth2 provider.
#[derive(Debug, Clone)]
pub struct OAuthProvider {
    pub name: &'static str,
    pub authorize_url: &'static str,
    pub token_url: &'static str,
    pub scopes: &'static [&'static str],
    pub client_id: String,
}

impl OAuthProvider {
    pub fn google(client_id: String) -> Self {
        Self {
            name: "google",
            authorize_url: "https://accounts.google.com/o/oauth2/v2/auth",
            token_url: "https://oauth2.googleapis.com/token",
            scopes: &["openid", "email", "profile"],
            client_id,
        }
    }
    pub fn github(client_id: String) -> Self {
        Self {
            name: "github",
            authorize_url: "https://github.com/login/oauth/authorize",
            token_url: "https://github.com/login/oauth/access_token",
            scopes: &["read:user", "user:email"],
            client_id,
        }
    }
}

/// Generate a PKCE code verifier (43-128 chars, URL-safe).
pub fn generate_verifier() -> String {
    use rand::RngCore;
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf)
}

/// S256 code challenge: BASE64URL(SHA256(verifier)).
pub fn challenge_s256(verifier: &str) -> String {
    let hash = Sha256::digest(verifier.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash)
}

/// Validate a redirect URI against an allowlist (exact match, no open redirect).
pub fn validate_redirect_uri(uri: &str, allowlist: &[&str]) -> bool {
    !uri.is_empty() && allowlist.contains(&uri)
}

/// Build the authorization URL.
pub fn authorize_url(
    provider: &OAuthProvider,
    redirect_uri: &str,
    state: &str,
    verifier: &str,
) -> String {
    let challenge = challenge_s256(verifier);
    let scope = provider.scopes.join(" ");
    format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
        provider.authorize_url,
        urlenc(&provider.client_id),
        urlenc(redirect_uri),
        urlenc(&scope),
        urlenc(state),
        urlenc(&challenge),
    )
}

fn urlenc(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Opaque state token (CSRF-bound).
pub fn generate_state() -> String {
    use rand::RngCore;
    let mut buf = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut buf);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_is_s256() {
        let v = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let c = challenge_s256(v);
        // RFC 7636 appendix B expected value.
        assert_eq!(c, "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
    }

    #[test]
    fn verifier_is_url_safe() {
        let v = generate_verifier();
        assert!(v.len() >= 43);
        assert!(!v.contains('+') && !v.contains('/'));
    }

    #[test]
    fn state_is_unique() {
        assert_ne!(generate_state(), generate_state());
    }

    #[test]
    fn redirect_uri_allowlist_is_exact() {
        let allow = ["https://app.example.com/cb"];
        assert!(validate_redirect_uri("https://app.example.com/cb", &allow));
        assert!(!validate_redirect_uri(
            "https://evil.example.com/cb",
            &allow
        ));
        assert!(!validate_redirect_uri("", &allow));
        // Open-redirect attempt via suffix.
        assert!(!validate_redirect_uri(
            "https://app.example.com/cb.evil",
            &allow
        ));
    }

    #[test]
    fn authorize_url_contains_pkce_and_state() {
        let p = OAuthProvider::google("client123".into());
        let u = authorize_url(&p, "https://app/cb", "STATE", "VERIFIER");
        assert!(u.contains("code_challenge="));
        assert!(u.contains("code_challenge_method=S256"));
        assert!(u.contains("state=STATE"));
        assert!(u.contains("client_id=client123"));
    }

    #[test]
    fn provider_scopes_non_empty() {
        assert!(!OAuthProvider::google("x".into()).scopes.is_empty());
        assert!(!OAuthProvider::github("x".into()).scopes.is_empty());
    }
}
