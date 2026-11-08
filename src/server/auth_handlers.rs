//! Auth service handlers (/api/v1/auth/*).
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde_json::{json, Value};

use super::handlers::current_user;
use super::state::AppState;
use super::{error_envelope, IntoResponse};

/// `POST /auth/login` — real password verification against salted PBKDF2 hashes.
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let email = body.get("email").and_then(Value::as_str).unwrap_or("");
    let password = body.get("password").and_then(Value::as_str).unwrap_or("");

    let invalid = || {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": {"code": "INVALID_CREDENTIALS", "message": "Invalid email or password."}})),
        )
            .into_response()
    };
    let store = state.store.lock().unwrap();
    let user = match store.find_user_by_email(email) {
        Some(u) => u.clone(),
        None => return invalid(),
    };
    if !user.is_active {
        return invalid();
    }
    let ok = match &user.password_hash {
        Some(h) => crate::security::password::verify_password(password, h),
        None => false,
    };
    if !ok {
        return invalid();
    }
    // Release the read guard before taking the mutable lock below.
    drop(store);
    // MFA gate: enrolled users must present a valid TOTP (demo "current user"
    // scope for verify/status; login binds the code to the account).
    if user.mfa_enabled {
        let now = chrono::Utc::now().timestamp() as u64;
        let code = body.get("totp").and_then(Value::as_str).unwrap_or("");
        let step = user
            .totp_secret
            .as_deref()
            .and_then(|s| crate::security::mfa::verify_totp(s, code, now));
        match step {
            Some(st) if st > user.mfa_last_step => {
                // Replay-safe: the same step cannot be reused.
                let mut store = state.store.lock().unwrap();
                if let Some(u) = store.users.iter_mut().find(|u| u.id == user.id) {
                    u.mfa_last_step = st;
                }
            }
            _ => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": {"code": "MFA_REQUIRED", "message": "A valid 6-digit authenticator code is required."}})),
                )
                    .into_response()
            }
        }
    }

    let mut resp = (
        StatusCode::OK,
        Json(json!({
            "access_token": "mock-access-token",
            "token_type": "bearer",
            "expires_in": 900,
            "csrf_token": "mock-csrf-token",
            "user": {
                "id": user.id,
                "email": user.email,
                "full_name": user.full_name,
                "role": user.role,
                "is_active": user.is_active,
                "must_change_password": user.must_change_password,
                "mfa_enabled": user.mfa_enabled,
            }
        })),
    )
        .into_response();
    resp.headers_mut()
        .insert("X-CSRF-Token", "mock-csrf-token".parse().unwrap());
    resp.headers_mut().insert(
        "Set-Cookie",
        "session_id=mock-session-123; Path=/; HttpOnly; SameSite=Lax"
            .parse()
            .unwrap(),
    );
    let _ = headers;
    resp
}

/// `POST /auth/register`.
pub async fn register(State(state): State<AppState>, Json(body): Json<Value>) -> impl IntoResponse {
    let email = body
        .get("email")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let password = body.get("password").and_then(Value::as_str).unwrap_or("");
    let full_name = body
        .get("full_name")
        .and_then(Value::as_str)
        .unwrap_or("New User")
        .to_string();
    if email.is_empty() {
        return error_envelope("VALIDATION_FAILED", "email is required");
    }
    if !crate::security::validation::validate_password(password).is_valid() {
        return error_envelope(
            "VALIDATION_FAILED",
            "password must be at least 8 characters",
        );
    }
    let mut store = state.store.lock().unwrap();
    if store.find_user_by_email(&email).is_some() {
        return error_envelope("VALIDATION_FAILED", "email already registered");
    }
    let id = store.next_user_id;
    store.next_user_id += 1;
    let user = crate::models::User {
        id,
        email: email.clone(),
        full_name: full_name.clone(),
        role: "resident".into(),
        is_active: true,
        must_change_password: false,
        password_hash: Some(crate::security::password::hash_password(
            password,
            crate::security::password::DEFAULT_ITERS,
        )),
        totp_secret: None,
        mfa_enabled: false,
        mfa_last_step: 0,
    };
    store.users.push(user.clone());
    let mut resp = (
        StatusCode::CREATED,
        Json(json!({
            "access_token": "mock-access-token",
            "token_type": "bearer",
            "expires_in": 900,
            "csrf_token": "mock-csrf-token",
            "user": {
                "id": id,
                "email": email,
                "full_name": full_name,
                "role": "resident",
                "is_active": true,
                "must_change_password": false,
            }
        })),
    )
        .into_response();
    resp.headers_mut()
        .insert("X-CSRF-Token", "mock-csrf-token".parse().unwrap());
    resp.headers_mut().insert(
        "Set-Cookie",
        "session_id=mock-session-123; Path=/; HttpOnly; SameSite=Lax"
            .parse()
            .unwrap(),
    );
    resp
}

/// `POST /auth/logout`.
pub async fn logout() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({"message": "Successfully logged out"})),
    )
}

/// `POST /auth/session` — validate session cookie and return user.
pub async fn session(State(state): State<AppState>) -> impl IntoResponse {
    let store = state.store.lock().unwrap();
    // Demo mode: return the first resident user as "current".
    let user = store
        .users
        .iter()
        .find(|u| u.role == "resident")
        .unwrap_or(&store.users[0]);
    let mut resp = (
        StatusCode::OK,
        Json(json!({
            "user": {
                "id": user.id,
                "email": user.email,
                "full_name": user.full_name,
                "role": user.role,
                "is_active": user.is_active,
                "must_change_password": user.must_change_password,
            }
        })),
    )
        .into_response();
    resp.headers_mut()
        .insert("X-CSRF-Token", "mock-csrf-token".parse().unwrap());
    resp
}

/// `GET /auth/me`.
pub async fn me(State(state): State<AppState>) -> impl IntoResponse {
    let store = state.store.lock().unwrap();
    let user = store
        .users
        .iter()
        .find(|u| u.role == "resident")
        .unwrap_or(&store.users[0]);
    (
        StatusCode::OK,
        Json(json!({
            "id": user.id,
            "email": user.email,
            "full_name": user.full_name,
            "role": user.role,
            "is_active": user.is_active,
            "must_change_password": user.must_change_password,
        })),
    )
}

/// `POST /auth/mfa/enroll` — generate a TOTP secret for the demo current
/// user. MFA is enabled only after a successful `/auth/mfa/verify`.
pub async fn mfa_enroll(State(state): State<AppState>) -> impl IntoResponse {
    let mut store = state.store.lock().unwrap();
    let Some(me) = current_user(&store) else {
        return error_envelope("USER_NOT_FOUND", "User not found.");
    };
    let id = me.id;
    let secret = crate::security::mfa::generate_secret();
    let uri = crate::security::mfa::otpauth_uri(&secret, &me.email, "Pinpoint");
    let user = store.users.iter_mut().find(|u| u.id == id).unwrap();
    user.totp_secret = Some(secret.clone());
    user.mfa_enabled = false;
    user.mfa_last_step = 0;
    (
        StatusCode::OK,
        Json(json!({ "secret": secret, "otpauth_uri": uri })),
    )
        .into_response()
}

/// `POST /auth/mfa/verify` {code} — confirm enrolment (enables MFA) or
/// re-verify (consumes the step, replay-protected).
pub async fn mfa_verify(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let code = body.get("code").and_then(Value::as_str).unwrap_or("");
    let now = chrono::Utc::now().timestamp() as u64;
    let mut store = state.store.lock().unwrap();
    let Some(me) = current_user(&store) else {
        return error_envelope("USER_NOT_FOUND", "User not found.");
    };
    let id = me.id;
    let user = store.users.iter_mut().find(|u| u.id == id).unwrap();
    let secret = user.totp_secret.clone();
    let Some(secret) = secret else {
        return error_envelope("MFA_NOT_ENROLLED", "No MFA secret to verify.");
    };
    match crate::security::mfa::verify_totp(&secret, code, now) {
        Some(step) if step > user.mfa_last_step => {
            user.mfa_enabled = true;
            user.mfa_last_step = step;
            (StatusCode::OK, Json(json!({ "mfa_enabled": true }))).into_response()
        }
        _ => (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": {"code": "INVALID_MFA_CODE", "message": "Invalid or already-used authenticator code."}})),
        )
            .into_response(),
    }
}

/// `GET /auth/mfa/status`.
pub async fn mfa_status(State(state): State<AppState>) -> impl IntoResponse {
    let store = state.store.lock().unwrap();
    let enabled = current_user(&store).map(|u| u.mfa_enabled).unwrap_or(false);
    (StatusCode::OK, Json(json!({ "mfa_enabled": enabled }))).into_response()
}

/// `DELETE /auth/mfa` — disable MFA (clears secret + flag).
pub async fn mfa_disable(State(state): State<AppState>) -> impl IntoResponse {
    let mut store = state.store.lock().unwrap();
    let Some(me) = current_user(&store) else {
        return error_envelope("USER_NOT_FOUND", "User not found.");
    };
    let id = me.id;
    let user = store.users.iter_mut().find(|u| u.id == id).unwrap();
    user.totp_secret = None;
    user.mfa_enabled = false;
    user.mfa_last_step = 0;
    (StatusCode::OK, Json(json!({ "mfa_enabled": false }))).into_response()
}

/// `POST /auth/password/change` and `/auth/password/force-change` — real rotation.
pub async fn change_password(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let new_password = body
        .get("new_password")
        .and_then(Value::as_str)
        .unwrap_or("");
    let confirm = body
        .get("confirm_password")
        .and_then(Value::as_str)
        .unwrap_or("");
    if new_password != confirm {
        return error_envelope("VALIDATION_FAILED", "new passwords do not match");
    }
    if !crate::security::validation::validate_password(new_password).is_valid() {
        return error_envelope(
            "VALIDATION_FAILED",
            "password must be at least 8 characters",
        );
    }
    let mut store = state.store.lock().unwrap();
    let Some(me) = current_user(&store) else {
        return error_envelope("USER_NOT_FOUND", "User not found.");
    };
    let id = me.id;
    let user = store.users.iter_mut().find(|u| u.id == id).unwrap();
    user.password_hash = Some(crate::security::password::hash_password(
        new_password,
        crate::security::password::DEFAULT_ITERS,
    ));
    user.must_change_password = false;
    (
        StatusCode::OK,
        Json(json!({"message": "Password changed."})),
    )
        .into_response()
}

pub async fn force_change_password(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    change_password(State(state), Json(body)).await
}
