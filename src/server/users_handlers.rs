//! Users + friendships service handlers.
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};

use super::handlers::{current_user, user_json};
use super::state::AppState;
use super::{error_envelope, IntoResponse};

/// `GET /users/` — list users (demo: all users).
pub async fn list_users(State(state): State<AppState>) -> impl IntoResponse {
    let store = state.store.lock().unwrap();
    let users: Vec<Value> = store
        .users
        .iter()
        .map(|u| {
            json!({
                "id": u.id,
                "email": u.email,
                "full_name": u.full_name,
                "role": u.role,
                "is_active": u.is_active,
                "must_change_password": u.must_change_password,
            })
        })
        .collect();
    (StatusCode::OK, Json(users))
}

/// `POST /users/` (admin) — create user.
pub async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let email = body
        .get("email")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let full_name = body
        .get("full_name")
        .and_then(Value::as_str)
        .unwrap_or("New User")
        .to_string();
    if email.is_empty() {
        return error_envelope("VALIDATION_FAILED", "email is required");
    }
    let password = body
        .get("password")
        .and_then(Value::as_str)
        .unwrap_or("ChangeMe123!");
    if !crate::security::validation::validate_password(password).is_valid() {
        return error_envelope(
            "VALIDATION_FAILED",
            "password must be at least 8 characters",
        );
    }
    let role: String = body
        .get("role")
        .and_then(Value::as_str)
        .unwrap_or("resident")
        .to_string();
    let hash = crate::security::password::hash_password(
        password,
        crate::security::password::DEFAULT_ITERS,
    );
    // Direct SQL insert (Auth owns wipe-rewrites of this table).
    let id = state
        .sql_insert_user(&email, &full_name, &role, Some(&hash))
        .await;
    let user = crate::models::User {
        id,
        email: email.clone(),
        full_name: full_name.clone(),
        role: role.clone(),
        is_active: true,
        must_change_password: false,
        password_hash: Some(hash),
        totp_secret: None,
        mfa_enabled: false,
        mfa_last_step: 0,
    };
    {
        let mut store = state.store.lock().unwrap();
        store.users.push(user.clone());
    }
    (
        StatusCode::CREATED,
        Json(json!({
            "id": id,
            "email": email,
            "full_name": full_name,
            "role": user.role,
        })),
    )
        .into_response()
}

/// `GET /users/{id}`.
pub async fn get_user(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let store = state.store.lock().unwrap();
    match store.find_user_by_id(id) {
        Some(u) => (
            StatusCode::OK,
            Json(json!({
                "id": u.id,
                "email": u.email,
                "full_name": u.full_name,
                "role": u.role,
                "is_active": u.is_active,
                "must_change_password": u.must_change_password,
            })),
        )
            .into_response(),
        None => error_envelope("USER_NOT_FOUND", "User not found."),
    }
}

/// `PATCH /users/{id}` — real partial update (profile + admin).
/// In-memory store is the view; direct SQL mirrors it (Auth owns the
/// wipe-rewrite, this service upserts single rows).
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let full_name = body
        .get("full_name")
        .and_then(Value::as_str)
        .map(str::to_string);
    if let Some(name) = &full_name {
        if name.trim().is_empty() {
            return error_envelope("VALIDATION_FAILED", "full_name cannot be empty");
        }
    }
    let role = body.get("role").and_then(Value::as_str).map(str::to_string);
    if let Some(r) = &role {
        if !["resident", "admin", "administrator"].contains(&r.as_str()) {
            return error_envelope(
                "VALIDATION_FAILED",
                "role must be resident, admin, or administrator",
            );
        }
    }
    let is_active = body.get("is_active").and_then(Value::as_bool);
    let must_change = body.get("must_change_password").and_then(Value::as_bool);
    {
        let mut store = state.store.lock().unwrap();
        let user = match store.users.iter_mut().find(|u| u.id == id) {
            Some(u) => u,
            None => return error_envelope("USER_NOT_FOUND", "User not found."),
        };
        if let Some(name) = full_name.as_ref() {
            user.full_name = name.clone();
        }
        if let Some(r) = role.as_ref() {
            user.role = r.clone();
        }
        if let Some(a) = is_active {
            user.is_active = a;
        }
        if let Some(flag) = must_change {
            user.must_change_password = flag;
        }
    }
    state
        .sql_update_user(id, full_name.as_deref(), role.as_deref(), is_active)
        .await;
    let store = state.store.lock().unwrap();
    let view = user_json(store.find_user_by_id(id).unwrap());
    (StatusCode::OK, Json(view)).into_response()
}

/// `DELETE /users/{id}` — admin removal (cascades friendships/shares/locations).
pub async fn delete_user(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    {
        let mut store = state.store.lock().unwrap();
        let pos = match store.users.iter().position(|u| u.id == id) {
            Some(p) => p,
            None => return error_envelope("USER_NOT_FOUND", "User not found."),
        };
        let me = current_user(&store).map(|u| u.id).unwrap_or(-1);
        if me == id {
            return error_envelope(
                "VALIDATION_FAILED",
                "Cannot delete your own account via admin endpoint.",
            );
        }
        store.users.remove(pos);
        store
            .friendships
            .retain(|f| f.requester_id != id && f.receiver_id != id);
        store.shares.retain(|s| s.owner_id != id);
        store.locations.retain(|l| l.user_id != id);
        store.check_ins.retain(|c| c.user_id != id);
        store.emergencies.retain(|e| e.user_id != id);
    }
    // DB mirror (Auth owns wipe-rewrites; delete the row directly).
    state.sql_delete_user(id).await;
    (StatusCode::OK, Json(json!({"message": "User deleted."}))).into_response()
}

/// `GET /users/search?q=` — substring match on email/full_name (add-friend helper).
pub async fn search_users(
    State(state): State<AppState>,
    q: axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let needle = q.get("q").cloned().unwrap_or_default().to_lowercase();
    let store = state.store.lock().unwrap();
    let hits: Vec<Value> = store
        .users
        .iter()
        .filter(|u| {
            needle.is_empty()
                || u.email.to_lowercase().contains(&needle)
                || u.full_name.to_lowercase().contains(&needle)
        })
        .map(user_json)
        .collect();
    (StatusCode::OK, Json(hits)).into_response()
}

/// `POST /friendships/` — send friend request.
pub async fn create_friendship(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let receiver_email = body
        .get("receiver_email")
        .and_then(Value::as_str)
        .unwrap_or("");
    if receiver_email.is_empty() {
        return error_envelope("VALIDATION_FAILED", "receiver_email is required");
    }
    let mut store = state.store.lock().unwrap();
    let receiver = match store.find_user_by_email(receiver_email) {
        Some(u) => u.clone(),
        None => return error_envelope("USER_NOT_FOUND", "User not found."),
    };
    let requester = store
        .users
        .iter()
        .find(|u| u.role == "resident")
        .unwrap_or(&store.users[0])
        .clone();
    if requester.id == receiver.id {
        return error_envelope(
            "VALIDATION_FAILED",
            "Cannot send a friend request to yourself.",
        );
    }
    let id = store.next_friendship_id;
    store.next_friendship_id += 1;
    let friendship = crate::models::Friendship {
        id,
        requester_id: requester.id,
        receiver_id: receiver.id,
        requester_name: requester.full_name.clone(),
        receiver_name: receiver.full_name.clone(),
        status: "pending".into(),
        created_at: Some("2026-11-08T12:00:00Z".into()),
        responded_at: None,
    };
    store.friendships.push(friendship.clone());
    (
        StatusCode::CREATED,
        Json(json!({
            "id": friendship.id,
            "requester_id": friendship.requester_id,
            "receiver_id": friendship.receiver_id,
            "requester_name": friendship.requester_name,
            "receiver_name": friendship.receiver_name,
            "status": friendship.status,
        })),
    )
        .into_response()
}

/// `GET /friendships/`.
pub async fn list_friendships(State(state): State<AppState>) -> impl IntoResponse {
    let store = state.store.lock().unwrap();
    let friendships: Vec<Value> = store
        .friendships
        .iter()
        .map(|f| {
            json!({
                "id": f.id,
                "requester_id": f.requester_id,
                "receiver_id": f.receiver_id,
                "requester_name": f.requester_name,
                "receiver_name": f.receiver_name,
                "status": f.status,
            })
        })
        .collect();
    (StatusCode::OK, Json(friendships))
}

/// `POST /friendships/{id}/respond`.
pub async fn respond_friendship(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let action = body.get("action").and_then(Value::as_str).unwrap_or("");
    let mut store = state.store.lock().unwrap();
    let friendship = match store.friendships.iter_mut().find(|f| f.id == id) {
        Some(f) => f,
        None => return error_envelope("FRIENDSHIP_NOT_FOUND", "Friendship not found."),
    };
    friendship.status = match action {
        "accept" => "accepted".into(),
        "reject" => "rejected".into(),
        _ => return error_envelope("VALIDATION_FAILED", "action must be accept or reject"),
    };
    friendship.responded_at = Some("2026-11-08T12:00:00Z".into());
    let resp = json!({
        "id": friendship.id,
        "requester_id": friendship.requester_id,
        "receiver_id": friendship.receiver_id,
        "requester_name": friendship.requester_name,
        "receiver_name": friendship.receiver_name,
        "status": friendship.status,
    });
    (StatusCode::OK, Json(resp)).into_response()
}

/// `DELETE /friendships/{id}` — unfriend / remove request (cascades shares).
pub async fn delete_friendship(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    {
        let mut store = state.store.lock().unwrap();
        let pos = match store.friendships.iter().position(|f| f.id == id) {
            Some(p) => p,
            None => return error_envelope("FRIENDSHIP_NOT_FOUND", "Friendship not found."),
        };
        store.friendships.remove(pos);
        store.shares.retain(|s| s.friendship_id != id);
    }
    state.sql_delete_friendship(id).await;
    (
        StatusCode::OK,
        Json(json!({"message": "Friendship removed."})),
    )
        .into_response()
}
