//! HTTP client for the Pinpoint API.
//!
//! Contract notes, verified against `backend/internal/handlers`:
//!
//! * Session cookie auth (`session_id`) plus the double-submit CSRF header
//!   (`X-CSRF-Token`) are required for every unsafe request (POST/PUT/PATCH/
//!   DELETE) once a session exists; GETs authenticate with the cookie alone.
//! * `POST /api/v1/auth/login` and `/register` answer with
//!   `{access_token, csrf_token, user}` and set both cookies.
//! * Errors use the `{error: {code, message, fields}}` envelope.
//! * Plain HTTP is accepted only for loopback development URLs; anything
//!   else must be HTTPS (fail closed — see ADR-0005).

use crate::models::{FriendLocation, Friendship, LocationShare, LoginResponse, User};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use url::Url;

/// Validates the backend URL before any request can be issued.
///
/// HTTP is allowed only for loopback hosts (`localhost`, `127.0.0.1`,
/// `::1`) so development keeps working while releases are forced onto TLS.
pub fn validate_backend_url(raw: &str) -> anyhow::Result<Url> {
    let url = Url::parse(raw.trim()).map_err(|e| anyhow::anyhow!("invalid backend URL: {e}"))?;
    match url.scheme() {
        "https" => Ok(url),
        "http" => {
            let loopback = match url.host() {
                Some(url::Host::Domain(d)) => d.eq_ignore_ascii_case("localhost"),
                Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
                Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
                None => false,
            };
            if loopback {
                Ok(url)
            } else {
                Err(anyhow::anyhow!(
                    "refusing plain HTTP to non-loopback backend \"{}\": use HTTPS outside local development",
                    url.as_str()
                ))
            }
        }
        other => Err(anyhow::anyhow!(
            "unsupported backend URL scheme \"{}\" (expected http or https)",
            other
        )),
    }
}

#[derive(Debug, Clone)]
struct Session {
    /// `name=value` pairs harvested from `Set-Cookie` response headers.
    cookies: Vec<String>,
    /// Double-submit token from the login body or `X-CSRF-Token` header.
    csrf_token: Option<String>,
}

pub struct ApiClient {
    backend_url: String,
    http: reqwest::Client,
    session: Arc<Mutex<Option<Session>>>,
}

impl ApiClient {
    /// Builds a client for `backend_url`, rejecting transports that do not
    /// meet the minimum transport policy.
    pub fn new(backend_url: impl Into<String>) -> anyhow::Result<Self> {
        let raw = backend_url.into();
        let url = validate_backend_url(&raw)?;
        // Normalize to the parsed form (no trailing slash surprises).
        let backend_url = url.as_str().trim_end_matches('/').to_string();
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()?;
        Ok(Self {
            backend_url,
            http,
            session: Arc::new(Mutex::new(None)),
        })
    }

    pub fn backend_url(&self) -> &str {
        &self.backend_url
    }

    /// True while a session cookie (and therefore an authenticated client
    /// state) is held locally. Cleared only after a confirmed logout or a
    /// password change that revokes server-side sessions.
    pub fn session_active(&self) -> bool {
        self.session.lock().unwrap().is_some()
    }

    fn store_session(&self, session: Option<Session>) {
        *self.session.lock().unwrap() = session;
    }

    fn session_snapshot(&self) -> Option<Session> {
        self.session.lock().unwrap().clone()
    }

    /// Issues a request, attaching the session cookie on every request and
    /// the CSRF header on unsafe methods. The session lock is never held
    /// across an `.await`.
    async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> anyhow::Result<reqwest::Response> {
        let url = format!("{}{}", self.backend_url, path);
        let mut request = self.http.request(method.clone(), &url);

        if let Some(session) = self.session_snapshot() {
            if !session.cookies.is_empty() {
                request = request.header("Cookie", session.cookies.join("; "));
            }
            let unsafe_method = matches!(
                method,
                reqwest::Method::POST
                    | reqwest::Method::PUT
                    | reqwest::Method::PATCH
                    | reqwest::Method::DELETE
            );
            if unsafe_method {
                if let Some(csrf) = session.csrf_token {
                    request = request.header("X-CSRF-Token", csrf);
                }
            }
        }

        if let Some(b) = body {
            request = request.json(&b);
        }

        Ok(request.send().await?)
    }

    /// Converts a non-success response into a readable error built from the
    /// API's error envelope (never leaks internals beyond code+message).
    async fn check(
        response: reqwest::Response,
        fallback: &str,
    ) -> anyhow::Result<reqwest::Response> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let text = response.text().await.unwrap_or_default();
        Err(anyhow::anyhow!(format_api_error(status, &text, fallback)))
    }

    /// Reads session material from a successful login/register response.
    fn capture_session(
        &self,
        headers: &reqwest::header::HeaderMap,
        body: &LoginResponse,
    ) -> anyhow::Result<()> {
        let cookies: Vec<String> = headers
            .get_all(reqwest::header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .filter_map(|s| s.split(';').next())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();

        // Fail closed: a 200 without a session cookie means the local state
        // would silently be unauthenticated on the next request.
        if cookies.is_empty() {
            return Err(anyhow::anyhow!(
                "server accepted the credentials but issued no session cookie"
            ));
        }

        let header_csrf = headers
            .get("X-CSRF-Token")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let csrf_token = body.csrf_token.clone().or(header_csrf);

        self.store_session(Some(Session {
            cookies,
            csrf_token,
        }));
        Ok(())
    }

    pub async fn login(&self, email: &str, password: &str) -> anyhow::Result<User> {
        let response = self
            .request(
                reqwest::Method::POST,
                "/api/v1/auth/login",
                Some(serde_json::json!({ "email": email, "password": password })),
            )
            .await?;
        let response = Self::check(response, "Login failed").await?;

        let headers = response.headers().clone();
        let body: LoginResponse = response
            .json()
            .await
            .map_err(|_| anyhow::anyhow!("Login failed: unexpected response body"))?;
        self.capture_session(&headers, &body)?;
        Ok(body.user)
    }

    pub async fn register(
        &self,
        email: &str,
        password: &str,
        full_name: &str,
    ) -> anyhow::Result<User> {
        let response = self
            .request(
                reqwest::Method::POST,
                "/api/v1/auth/register",
                Some(serde_json::json!({
                    "email": email,
                    "password": password,
                    "full_name": full_name
                })),
            )
            .await?;
        let response = Self::check(response, "Registration failed").await?;

        let headers = response.headers().clone();
        let body: LoginResponse = response
            .json()
            .await
            .map_err(|_| anyhow::anyhow!("Registration failed: unexpected response body"))?;
        self.capture_session(&headers, &body)?;
        Ok(body.user)
    }

    /// Confirms logout with the server before dropping local session state;
    /// a failed revocation keeps the session so the client never claims a
    /// logout that did not happen.
    pub async fn logout(&self) -> anyhow::Result<()> {
        let response = self
            .request(reqwest::Method::POST, "/api/v1/auth/logout", None)
            .await?;
        let response = Self::check(response, "Logout failed").await?;
        let _ = response;
        self.store_session(None);
        Ok(())
    }

    /// `POST /api/v1/auth/password/change` (revokes all sessions server-side,
    /// so local state is dropped) or `/password/force-change` when the server
    /// forced the rotation (`current_password` omitted).
    pub async fn change_password(
        &self,
        current_password: Option<&str>,
        new_password: &str,
        confirm_password: &str,
    ) -> anyhow::Result<bool> {
        let (path, body) = match current_password {
            Some(current) => (
                "/api/v1/auth/password/change",
                serde_json::json!({
                    "current_password": current,
                    "new_password": new_password,
                    "confirm_password": confirm_password
                }),
            ),
            None => (
                "/api/v1/auth/password/force-change",
                serde_json::json!({
                    "new_password": new_password,
                    "confirm_password": confirm_password
                }),
            ),
        };

        let response = self
            .request(reqwest::Method::POST, path, Some(body))
            .await?;
        let response = Self::check(response, "Password change failed").await?;
        let _ = response;

        // The regular change endpoint revokes every session for the user;
        // drop local state so the UI returns to the login screen. The
        // forced change keeps the current session alive.
        let revoked = current_password.is_some();
        if revoked {
            self.store_session(None);
        }
        Ok(revoked)
    }

    pub async fn get_friendships(&self) -> anyhow::Result<Vec<Friendship>> {
        let response = self
            .request(reqwest::Method::GET, "/api/v1/friendships/", None)
            .await?;
        let response = Self::check(response, "Failed to fetch friendships").await?;
        Ok(response.json().await?)
    }

    pub async fn send_friend_request(&self, email: &str) -> anyhow::Result<()> {
        let response = self
            .request(
                reqwest::Method::POST,
                "/api/v1/friendships/",
                Some(serde_json::json!({ "receiver_email": email })),
            )
            .await?;
        Self::check(response, "Failed to send friend request").await?;
        Ok(())
    }

    pub async fn respond_to_friendship(&self, id: i64, action: &str) -> anyhow::Result<()> {
        let response = self
            .request(
                reqwest::Method::POST,
                &format!("/api/v1/friendships/{id}/respond"),
                Some(serde_json::json!({ "action": action })),
            )
            .await?;
        Self::check(response, "Failed to respond to friendship").await?;
        Ok(())
    }

    pub async fn get_location_shares(&self) -> anyhow::Result<Vec<LocationShare>> {
        let response = self
            .request(reqwest::Method::GET, "/api/v1/locations/shares", None)
            .await?;
        let response = Self::check(response, "Failed to fetch location shares").await?;
        Ok(response.json().await?)
    }

    pub async fn create_location_share(
        &self,
        friendship_id: i64,
        direction: &str,
        precision_level: &str,
    ) -> anyhow::Result<()> {
        let response = self
            .request(
                reqwest::Method::POST,
                "/api/v1/locations/shares",
                Some(serde_json::json!({
                    "friendship_id": friendship_id,
                    "direction": direction,
                    "precision_level": precision_level
                })),
            )
            .await?;
        Self::check(response, "Failed to create location share").await?;
        Ok(())
    }

    pub async fn update_location_share(
        &self,
        share_id: i64,
        is_active: Option<bool>,
        precision: Option<String>,
    ) -> anyhow::Result<()> {
        let mut body = serde_json::json!({});
        if let Some(active) = is_active {
            body["is_active"] = serde_json::Value::Bool(active);
        }
        if let Some(p) = precision {
            body["precision_level"] = serde_json::Value::String(p);
        }

        let response = self
            .request(
                reqwest::Method::PATCH,
                &format!("/api/v1/locations/shares/{share_id}"),
                Some(body),
            )
            .await?;
        Self::check(response, "Failed to update location share").await?;
        Ok(())
    }

    pub async fn get_friend_locations(&self) -> anyhow::Result<Vec<FriendLocation>> {
        let response = self
            .request(reqwest::Method::GET, "/api/v1/locations/friends", None)
            .await?;
        let response = Self::check(response, "Failed to fetch friend locations").await?;
        Ok(response.json().await?)
    }

    pub async fn publish_location(
        &self,
        latitude: f64,
        longitude: f64,
        accuracy: f64,
    ) -> anyhow::Result<()> {
        let response = self
            .request(
                reqwest::Method::POST,
                "/api/v1/locations/me",
                Some(serde_json::json!({
                    "latitude": latitude,
                    "longitude": longitude,
                    "accuracy": accuracy
                })),
            )
            .await?;
        Self::check(response, "Failed to publish location").await?;
        Ok(())
    }
}

/// Renders the API's `{error: {code, message, fields}}` envelope as a
/// single readable line; falls back to a truncated body snippet so the UI
/// never shows a full HTML/JSON dump.
fn format_api_error(status: reqwest::StatusCode, body: &str, fallback: &str) -> String {
    #[derive(serde::Deserialize)]
    struct ErrorEnvelope {
        error: ErrorDetail,
    }
    #[derive(serde::Deserialize)]
    struct ErrorDetail {
        #[serde(default)]
        code: Option<String>,
        #[serde(default)]
        message: Option<String>,
        #[serde(default)]
        fields: Option<serde_json::Map<String, serde_json::Value>>,
    }

    if let Ok(envelope) = serde_json::from_str::<ErrorEnvelope>(body) {
        let code = envelope.error.code.unwrap_or_default();
        let message = envelope
            .error
            .message
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| fallback.to_string());
        let mut line = if code.is_empty() {
            message
        } else {
            format!("{code}: {message}")
        };
        if let Some(fields) = envelope.error.fields {
            let mut parts: Vec<String> = Vec::new();
            for (field, errors) in fields {
                match errors {
                    serde_json::Value::Array(list) => {
                        let msgs: Vec<String> = list
                            .iter()
                            .filter_map(|e| e.as_str().map(str::to_string))
                            .collect();
                        if !msgs.is_empty() {
                            parts.push(format!("{field}: {}", msgs.join(", ")));
                        }
                    }
                    serde_json::Value::String(s) => parts.push(format!("{field}: {s}")),
                    _ => {}
                }
            }
            if !parts.is_empty() {
                line = format!("{line} ({})", parts.join("; "));
            }
        }
        return line;
    }

    let snippet: String = body.chars().take(160).collect();
    if snippet.trim().is_empty() {
        format!("{} {fallback}", status.as_u16())
    } else {
        format!("{status}: {snippet}")
    }
}

#[cfg(test)]
mod tests {
    use super::validate_backend_url;

    #[test]
    fn accepts_loopback_http_development_urls() {
        for url in [
            "http://localhost:8000",
            "http://127.0.0.1:8000",
            "http://[::1]:8000",
            "http://LOCALHOST:8000",
        ] {
            assert!(
                validate_backend_url(url).is_ok(),
                "{url} should be accepted"
            );
        }
    }

    #[test]
    fn accepts_https_everywhere() {
        for url in ["https://api.example.com", "https://10.0.0.5:8443"] {
            assert!(
                validate_backend_url(url).is_ok(),
                "{url} should be accepted"
            );
        }
    }

    // Fail closed: non-loopback plain HTTP is the exact regression this
    // policy exists to prevent (ADR-0005).
    #[test]
    fn rejects_plaintext_http_to_non_loopback_hosts() {
        for url in [
            "http://example.com",
            "http://api.example.com:8000",
            "http://192.168.1.10:8000",
            "http://10.0.2.2:8000",
        ] {
            assert!(validate_backend_url(url).is_err(), "{url} must be rejected");
        }
    }

    #[test]
    fn rejects_unsupported_schemes_and_garbage() {
        for url in ["ftp://example.com", "ws://localhost:8000", "not a url", ""] {
            assert!(validate_backend_url(url).is_err(), "{url} must be rejected");
        }
    }
}
