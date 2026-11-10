//! Locations + shares service handlers.
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};

use super::handlers::{current_user, share_json};
use super::state::AppState;
use super::{error_envelope, IntoResponse};

/// `POST /locations/shares`.
pub async fn create_share(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let friendship_id = body
        .get("friendship_id")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let direction = body
        .get("direction")
        .and_then(Value::as_str)
        .unwrap_or("to_friend")
        .to_string();
    let precision = body
        .get("precision_level")
        .and_then(Value::as_str)
        .unwrap_or("exact")
        .to_string();
    // Optional expiry: duration_hours (int) or explicit expires_at ISO-8601.
    let expires_at: Option<String> =
        if let Some(h) = body.get("duration_hours").and_then(Value::as_i64) {
            if h <= 0 {
                return error_envelope("VALIDATION_FAILED", "duration_hours must be > 0");
            }
            Some(format!("2026-11-08T{}:00:00Z", 12 + h))
        } else {
            body.get("expires_at")
                .and_then(Value::as_str)
                .map(str::to_string)
        };
    let label = body
        .get("label")
        .and_then(Value::as_str)
        .map(str::to_string);
    if friendship_id <= 0 {
        return error_envelope("VALIDATION_FAILED", "friendship_id is required");
    }
    let mut store = state.store.lock().unwrap();
    if !store
        .friendships
        .iter()
        .any(|f| f.id == friendship_id && f.status == "accepted")
    {
        return error_envelope(
            "FRIENDSHIP_NOT_FOUND",
            "Friendship not found or not accepted.",
        );
    }
    let id = store.next_share_id;
    store.next_share_id += 1;
    let owner = store
        .users
        .iter()
        .find(|u| u.role == "resident")
        .unwrap_or(&store.users[0])
        .clone();
    let share = crate::models::LocationShare {
        id,
        friendship_id,
        owner_id: owner.id,
        direction: direction.clone(),
        precision_level: precision.clone(),
        is_active: true,
        last_location_update_at: None,
        expires_at,
        label,
    };
    store.shares.push(share.clone());
    (StatusCode::CREATED, Json(share_json(&share))).into_response()
}

/// `GET /locations/shares`.
pub async fn list_shares(State(state): State<AppState>) -> impl IntoResponse {
    let store = state.store.lock().unwrap();
    let shares: Vec<Value> = store.shares.iter().map(share_json).collect();
    (StatusCode::OK, Json(shares))
}

/// `PATCH /locations/shares/{id}`.
pub async fn update_share(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let mut store = state.store.lock().unwrap();
    let share = match store.shares.iter_mut().find(|s| s.id == id) {
        Some(s) => s,
        None => return error_envelope("SHARE_NOT_FOUND", "Share not found."),
    };
    if let Some(active) = body.get("is_active").and_then(Value::as_bool) {
        share.is_active = active;
    }
    if let Some(p) = body.get("precision_level").and_then(Value::as_str) {
        if !["exact", "approximate", "city_only"].contains(&p) {
            return error_envelope(
                "VALIDATION_FAILED",
                "precision_level must be exact, approximate, or city_only",
            );
        }
        share.precision_level = p.to_string();
    }
    if let Some(d) = body.get("direction").and_then(Value::as_str) {
        if !["to_friend", "from_friend", "mutual"].contains(&d) {
            return error_envelope(
                "VALIDATION_FAILED",
                "direction must be to_friend, from_friend, or mutual",
            );
        }
        share.direction = d.to_string();
    }
    if let Some(l) = body.get("label").and_then(Value::as_str) {
        share.label = Some(l.to_string());
    }
    if body.get("expires_at").is_some() {
        share.expires_at = body
            .get("expires_at")
            .and_then(Value::as_str)
            .map(str::to_string);
    }
    (StatusCode::OK, Json(share_json(share))).into_response()
}

/// `DELETE /locations/shares/{id}` — revoke a share permanently.
pub async fn delete_share(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    {
        let mut store = state.store.lock().unwrap();
        let pos = match store.shares.iter().position(|s| s.id == id) {
            Some(p) => p,
            None => return error_envelope("SHARE_NOT_FOUND", "Share not found."),
        };
        store.shares.remove(pos);
    }
    state.sql_delete_share(id).await;
    (StatusCode::OK, Json(json!({"message": "Share deleted."}))).into_response()
}

/// `POST /locations/me` — publish current location.
pub async fn publish_location(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let lat = body.get("latitude").and_then(Value::as_f64).unwrap_or(0.0);
    let lon = body.get("longitude").and_then(Value::as_f64).unwrap_or(0.0);
    let accuracy = body.get("accuracy").and_then(Value::as_f64);
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
        return error_envelope(
            "VALIDATION_FAILED",
            "latitude must be -90..90, longitude -180..180",
        );
    }
    let mut store = state.store.lock().unwrap();
    // Upsert: update or create location for the current demo user.
    let user_id = store
        .users
        .iter()
        .find(|u| u.role == "resident")
        .map(|u| u.id)
        .unwrap_or(1);
    let user_name = store
        .find_user_by_id(user_id)
        .map(|u| u.full_name.clone())
        .unwrap_or_default();
    if let Some(loc) = store.locations.iter_mut().find(|l| l.user_id == user_id) {
        loc.latitude = lat;
        loc.longitude = lon;
        loc.accuracy = accuracy;
        loc.updated_at = "2026-11-08T12:00:00Z".into();
    } else {
        store.locations.push(crate::models::FriendLocation {
            user_id,
            user_name,
            latitude: lat,
            longitude: lon,
            precision_level: "exact".into(),
            updated_at: "2026-11-08T12:00:00Z".into(),
            accuracy,
        });
    }
    let shares_updated = store.shares.iter().filter(|s| s.is_active).count();
    (
        StatusCode::OK,
        Json(json!({"message": "Location updated.", "shares_updated": shares_updated})),
    )
        .into_response()
}

/// `GET /locations/friends` — locations visible to the caller.
/// Consent model: returns only locations for users with an accepted
/// friendship + active share (or all demo users in the demo store).
pub async fn list_friend_locations(State(state): State<AppState>) -> impl IntoResponse {
    let store = state.store.lock().unwrap();
    // Demo store: return all locations (seeded demo + published).
    // Production would filter by accepted friendships + active shares.
    let locations: Vec<Value> = store
        .locations
        .iter()
        .filter(|l| {
            let me = current_user(&store).map(|u| u.id).unwrap_or(-1);
            // Hide my own dot from "friends" only if an explicit row exists;
            // demo keeps all rows visible for map parity.
            l.user_id != me || store.locations.len() <= 2
        })
        .map(|l| {
            let (lat, lon) = match l.precision_level.as_str() {
                "exact" => (l.latitude, l.longitude),
                "approximate" => (
                    (l.latitude * 100.0).round() / 100.0,
                    (l.longitude * 100.0).round() / 100.0,
                ),
                _ => (
                    (l.latitude * 10.0).round() / 10.0,
                    (l.longitude * 10.0).round() / 10.0,
                ),
            };
            json!({
                "user_id": l.user_id,
                "user_name": l.user_name,
                "latitude": lat,
                "longitude": lon,
                "accuracy": l.accuracy,
                "updated_at": l.updated_at,
                "precision_level": l.precision_level,
            })
        })
        .collect();
    (StatusCode::OK, Json(locations))
}

/// `GET /locations/me` — my own latest published location (or 404).
pub async fn get_my_location(State(state): State<AppState>) -> impl IntoResponse {
    let store = state.store.lock().unwrap();
    let me = match current_user(&store) {
        Some(u) => u,
        None => return error_envelope("USER_NOT_FOUND", "User not found."),
    };
    match store.locations.iter().find(|l| l.user_id == me.id) {
        Some(l) => (
            StatusCode::OK,
            Json(json!({
                "user_id": l.user_id, "user_name": l.user_name,
                "latitude": l.latitude, "longitude": l.longitude,
                "accuracy": l.accuracy, "updated_at": l.updated_at,
                "precision_level": l.precision_level,
            })),
        )
            .into_response(),
        None => error_envelope("LOCATION_NOT_FOUND", "No location published yet."),
    }
}

/// `POST /locations/check-ins` — create a check-in (manual/scheduled/trip).
pub async fn create_check_in(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let lat = body.get("latitude").and_then(Value::as_f64).unwrap_or(0.0);
    let lon = body.get("longitude").and_then(Value::as_f64).unwrap_or(0.0);
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
        return error_envelope(
            "VALIDATION_FAILED",
            "latitude must be -90..90, longitude -180..180",
        );
    }
    let note = body
        .get("note")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let kind = body
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("manual")
        .to_string();
    if !["manual", "scheduled", "trip"].contains(&kind.as_str()) {
        return error_envelope(
            "VALIDATION_FAILED",
            "kind must be manual, scheduled, or trip",
        );
    }
    let mut store = state.store.lock().unwrap();
    let me = match current_user(&store) {
        Some(u) => u,
        None => return error_envelope("USER_NOT_FOUND", "User not found."),
    };
    let id = store.next_check_in_id;
    store.next_check_in_id += 1;
    let ci = crate::models::CheckIn {
        id,
        user_id: me.id,
        user_name: me.full_name.clone(),
        latitude: lat,
        longitude: lon,
        note,
        kind,
        created_at: "2026-11-08T12:00:00Z".into(),
    };
    store.check_ins.push(ci.clone());
    (
        StatusCode::CREATED,
        Json(json!({
            "id": ci.id, "user_id": ci.user_id, "user_name": ci.user_name,
            "latitude": ci.latitude, "longitude": ci.longitude,
            "note": ci.note, "kind": ci.kind, "created_at": ci.created_at,
        })),
    )
        .into_response()
}

/// `GET /locations/check-ins` — all check-ins (newest first).
pub async fn list_check_ins(State(state): State<AppState>) -> impl IntoResponse {
    let store = state.store.lock().unwrap();
    let mut items: Vec<Value> = store
        .check_ins
        .iter()
        .map(|c| {
            json!({
                "id": c.id, "user_id": c.user_id, "user_name": c.user_name,
                "latitude": c.latitude, "longitude": c.longitude,
                "note": c.note, "kind": c.kind, "created_at": c.created_at,
            })
        })
        .collect();
    items.reverse();
    (StatusCode::OK, Json(items)).into_response()
}

/// `DELETE /locations/check-ins/{id}`.
pub async fn delete_check_in(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    {
        let mut store = state.store.lock().unwrap();
        let pos = match store.check_ins.iter().position(|c| c.id == id) {
            Some(p) => p,
            None => return error_envelope("CHECK_IN_NOT_FOUND", "Check-in not found."),
        };
        store.check_ins.remove(pos);
    }
    state.sql_delete_check_in(id).await;
    (
        StatusCode::OK,
        Json(json!({"message": "Check-in deleted."})),
    )
        .into_response()
}

/// `POST /locations/emergency` — panic share: records an alert + activates all shares.
pub async fn trigger_emergency(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let lat = body.get("latitude").and_then(Value::as_f64).unwrap_or(0.0);
    let lon = body.get("longitude").and_then(Value::as_f64).unwrap_or(0.0);
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
        return error_envelope(
            "VALIDATION_FAILED",
            "latitude must be -90..90, longitude -180..180",
        );
    }
    let mut store = state.store.lock().unwrap();
    let me = match current_user(&store) {
        Some(u) => u,
        None => return error_envelope("USER_NOT_FOUND", "User not found."),
    };
    // Publish the emergency point as the latest location + activate shares.
    if let Some(loc) = store.locations.iter_mut().find(|l| l.user_id == me.id) {
        loc.latitude = lat;
        loc.longitude = lon;
        loc.updated_at = "2026-11-08T12:00:00Z".into();
    } else {
        store.locations.push(crate::models::FriendLocation {
            user_id: me.id,
            user_name: me.full_name.clone(),
            latitude: lat,
            longitude: lon,
            precision_level: "exact".into(),
            updated_at: "2026-11-08T12:00:00Z".into(),
            accuracy: None,
        });
    }
    for s in store.shares.iter_mut() {
        if s.owner_id == me.id {
            s.is_active = true;
        }
    }
    let id = store.next_emergency_id;
    store.next_emergency_id += 1;
    let alert = crate::models::EmergencyAlert {
        id,
        user_id: me.id,
        user_name: me.full_name.clone(),
        latitude: lat,
        longitude: lon,
        created_at: "2026-11-08T12:00:00Z".into(),
        resolved: false,
    };
    store.emergencies.push(alert.clone());
    (
        StatusCode::CREATED,
        Json(json!({
            "id": alert.id, "user_id": alert.user_id, "user_name": alert.user_name,
            "latitude": alert.latitude, "longitude": alert.longitude,
            "created_at": alert.created_at, "resolved": alert.resolved,
        })),
    )
        .into_response()
}

/// `GET /locations/emergency` — all emergency alerts (newest first).
pub async fn list_emergencies(State(state): State<AppState>) -> impl IntoResponse {
    let store = state.store.lock().unwrap();
    let mut items: Vec<Value> = store
        .emergencies
        .iter()
        .map(|e| {
            json!({
                "id": e.id, "user_id": e.user_id, "user_name": e.user_name,
                "latitude": e.latitude, "longitude": e.longitude,
                "created_at": e.created_at, "resolved": e.resolved,
            })
        })
        .collect();
    items.reverse();
    (StatusCode::OK, Json(items)).into_response()
}

/// `PATCH /locations/emergency/{id}` — resolve / re-open an alert.
pub async fn resolve_emergency(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let resolved = body
        .get("resolved")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let mut store = state.store.lock().unwrap();
    let alert = match store.emergencies.iter_mut().find(|e| e.id == id) {
        Some(e) => e,
        None => return error_envelope("EMERGENCY_NOT_FOUND", "Emergency alert not found."),
    };
    alert.resolved = resolved;
    (
        StatusCode::OK,
        Json(json!({
            "id": alert.id, "user_id": alert.user_id, "user_name": alert.user_name,
            "latitude": alert.latitude, "longitude": alert.longitude,
            "created_at": alert.created_at, "resolved": alert.resolved,
        })),
    )
        .into_response()
}
