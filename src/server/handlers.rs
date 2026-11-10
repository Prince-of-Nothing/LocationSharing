//! Shared HTTP handler helpers for the Rust API server.
//!
//! Service-specific handlers live in `auth_handlers`, `users_handlers` and
//! `locations_handlers` so each deployable compiles only what it serves.

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};

use super::state::AppState;

pub(crate) fn current_user(store: &crate::server::state::Store) -> Option<crate::models::User> {
    // Demo auth: first resident is "current" for all self-scoped endpoints.
    store
        .users
        .iter()
        .find(|u| u.role == "resident")
        .or(store.users.first())
        .cloned()
}

pub(crate) fn is_expired(expires_at: &Option<String>) -> bool {
    match expires_at {
        None => false,
        Some(s) => s.as_str() < "2026-11-08T12:00:00Z", // demo clock: seed date is the floor
    }
}

pub(crate) fn user_json(u: &crate::models::User) -> Value {
    json!({
        "id": u.id,
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

/// Health check — verifies database connectivity when Postgres is configured,
/// otherwise reports in-memory demo mode.
pub async fn health(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let mut checks = serde_json::Map::new();
    checks.insert("status".to_string(), json!("healthy"));
    checks.insert("version".to_string(), json!("0.2.0"));
    checks.insert("mode".to_string(), json!(if state.pg.is_some() { "postgres" } else { "in-memory" }));

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
