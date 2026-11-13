//! Shared HTTP handler helpers for the Rust API server.
//!
//! Service-specific handlers live in `auth_handlers`, `users_handlers` and
//! `locations_handlers` so each deployable compiles only what it serves.

use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};
use std::time::SystemTime;

use super::state::AppState;
use super::CurrentUser;

/// Extract the session user or return a 401 response.
#[allow(clippy::result_large_err)] // the error is an axum Response; boxing buys nothing here
pub(crate) fn require_current_user(
    cu: Option<axum::extract::Extension<CurrentUser>>,
) -> Result<CurrentUser, axum::response::Response> {
    use axum::response::IntoResponse;
    cu.map(|e| e.0).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": {"code": "UNAUTHENTICATED", "message": "Sign in required."}})),
        )
            .into_response()
    })
}

pub(crate) fn is_expired(expires_at: &Option<String>) -> bool {
    match expires_at
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
    {
        Some(at) => at.with_timezone(&chrono::Utc) < chrono::Utc::now(),
        None => false,
    }
}

pub(crate) fn not_found(code: &str, message: &str) -> axum::response::Response {
    use axum::response::IntoResponse;
    (
        StatusCode::NOT_FOUND,
        Json(json!({"error": {"code": code, "message": message}})),
    )
        .into_response()
}

pub(crate) fn user_json(u: &crate::models::User) -> Value {
    json!({
        "id": u.id,
        "username": super::usernames::username_for(u.id),
        "email": u.email,
        "full_name": u.full_name,
        "role": u.role,
        "is_active": u.is_active,
        "must_change_password": u.must_change_password,
    })
}

pub(crate) fn share_json(s: &crate::models::LocationShare) -> Value {
    json!({
        "id": s.id,
        "friendship_id": s.friendship_id,
        "owner_id": s.owner_id,
        "direction": s.direction,
        "precision_level": s.precision_level,
        "is_active": s.is_active && !is_expired(&s.expires_at),
        "last_location_update_at": s.last_location_update_at,
        "expires_at": s.expires_at,
        "label": s.label,
    })
}

/// Health check - verifies database connectivity when Postgres is configured,
/// otherwise reports in-memory demo mode.
pub async fn health(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let mut checks = serde_json::Map::new();
    checks.insert("status".to_string(), json!("healthy"));
    checks.insert("version".to_string(), json!(env!("CARGO_PKG_VERSION")));
    checks.insert(
        "mode".to_string(),
        json!(if state.pg.is_some() {
            "postgres"
        } else {
            "in-memory"
        }),
    );

    if let Some(pool) = &state.pg {
        match sqlx::query("SELECT 1").fetch_one(pool).await {
            Ok(_) => {
                checks.insert("database".to_string(), json!("connected"));
            }
            Err(e) => {
                checks.insert("database".to_string(), json!(format!("error: {e}")));
                checks.insert("status".to_string(), json!("degraded"));
                return (StatusCode::SERVICE_UNAVAILABLE, Json(Value::Object(checks)));
            }
        }
    } else {
        checks.insert("database".to_string(), json!("not configured"));
    }

    (StatusCode::OK, Json(Value::Object(checks)))
}

/// Sync status - reports replication freshness for monitoring replica health.
/// `last_persist_at` is the last successful `persist()` timestamp; clients and
/// load balancers can use this to detect stale replicas.
pub async fn sync_status(State(state): State<AppState>) -> Json<Value> {
    let last_persist = state.last_persist_at.lock().unwrap();
    let age_secs = last_persist
        .as_ref()
        .and_then(|t| t.elapsed().ok())
        .map(|d| d.as_secs())
        .unwrap_or(u64::MAX);
    let db_connected = state.pg.is_some();
    let mode = if state.pg.is_some() {
        "postgres"
    } else {
        "in-memory"
    };
    Json(json!({
        "mode": mode,
        "database_connected": db_connected,
        "last_persist_at": last_persist.map(|t| t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs()),
        "last_persist_age_secs": age_secs,
        "fresh": age_secs <= 10,
    }))
}

#[cfg(test)]
mod expiry_tests {
    use super::is_expired;

    #[test]
    fn past_expiry_is_expired_even_after_seed_date() {
        assert!(is_expired(&Some("2026-11-12T00:00:00Z".into())));
    }

    #[test]
    fn future_and_missing_expiry_are_not_expired() {
        assert!(!is_expired(&Some("2099-01-01T00:00:00Z".into())));
        assert!(!is_expired(&None));
    }
}
