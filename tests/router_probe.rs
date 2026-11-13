//! In-process router probe: proves which routes match (no network needed).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use tower::ServiceExt;

#[tokio::test]
async fn map_tile_route_matches() {
    let app = pinpoint_desktop::server::map_router()
        .with_state(pinpoint_desktop::server::AppState::new());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/map/tiles/10/1/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // Tile fetch may fail offline (502) but must NOT be 404 (route must match).
    assert_ne!(resp.status(), StatusCode::NOT_FOUND, "tile route missing");
}

#[tokio::test]
async fn minimal_param_route() {
    use axum::routing::get;
    let app = axum::Router::new().route("/:id", get(|| async { "hi" }));
    let resp = app
        .oneshot(Request::builder().uri("/42").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "minimal param route must match"
    );
}

#[tokio::test]
async fn users_by_id_route_matches() {
    let app = pinpoint_desktop::server::users_router()
        .with_state(pinpoint_desktop::server::AppState::new());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/users/2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_ne!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "users by-id route missing"
    );
}

#[tokio::test]
async fn health_route_matches_all_services() {
    // Redundancy: every service replica must serve /health for Docker
    // healthchecks and gateway failover (docker-compose.yml). The routers
    // themselves no longer embed /health (the combined router would panic on
    // duplicate routes); each service binary adds it, as this test does.
    use pinpoint_desktop::server::health;
    for app in [
        pinpoint_desktop::server::auth_router()
            .route("/health", get(health))
            .with_state(pinpoint_desktop::server::AppState::new()),
        pinpoint_desktop::server::users_router()
            .route("/health", get(health))
            .with_state(pinpoint_desktop::server::AppState::new()),
        pinpoint_desktop::server::locations_router()
            .route("/health", get(health))
            .with_state(pinpoint_desktop::server::AppState::new()),
    ] {
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "/health missing");
    }
}

#[tokio::test]
async fn sync_status_route_matches() {
    // Redundancy: /api/v1/sync-status must be reachable for replica
    // freshness monitoring by clients and load balancers.
    let app = axum::Router::new()
        .route(
            "/api/v1/sync-status",
            get(pinpoint_desktop::server::sync_status),
        )
        .with_state(pinpoint_desktop::server::AppState::new());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/sync-status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_ne!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "sync-status route missing"
    );
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["mode"], "in-memory");
    assert_eq!(json["database_connected"], false);
}

#[tokio::test]
async fn session_list_and_revoke_flow() {
    use pinpoint_desktop::server::{router, AppState};

    let state = AppState::new();
    let (sid, csrf) = state.create_session(42, "email", false).await;
    let app = router(state.clone());
    let cookie = format!("session_id={sid}");

    // Unauthenticated: fail closed.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/sessions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Authenticated: one session, marked current.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/sessions")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["sessions"][0]["current"], true);
    let prefix = json["sessions"][0]["id"].as_str().unwrap().to_string();
    assert_eq!(prefix.len(), 8, "only a short prefix may be exposed");
    assert!(!prefix.contains(&sid), "full session id must stay secret");

    // Revoking without the CSRF token is rejected (double-submit).
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/auth/sessions/{prefix}"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // With the token the session is revoked and the cookie cleared.
    let resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/auth/sessions/{prefix}"))
                .header("cookie", &cookie)
                .header("x-csrf-token", &csrf)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(state.get_session(&sid).await.is_none());
}

#[tokio::test]
async fn anonymous_account_creation_and_recovery() {
    use pinpoint_desktop::server::{router, AppState};

    let app = router(AppState::new());
    let body = r#"{"device_name":"Test device"}"#;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/anonymous")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let code = json["recovery_code"].as_str().unwrap().to_string();
    assert!(code.len() >= 32, "recovery code must be high-entropy");
    assert_eq!(json["user"]["auth_method"], "anonymous");

    // Recover with the code: a fresh session + a rotated code.
    let body = serde_json::json!({ "recovery_code": code }).to_string();
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/anonymous/recover")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let rotated = json["recovery_code"].as_str().unwrap().to_string();
    assert_ne!(rotated, code, "recovery code must rotate on use");

    // The old code is now dead.
    let body = serde_json::json!({ "recovery_code": code }).to_string();
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/anonymous/recover")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn map_capabilities_lists_driving() {
    let app = pinpoint_desktop::server::map_router()
        .with_state(pinpoint_desktop::server::AppState::new());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/map/capabilities")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["modes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m == "driving"));
    assert!(json["autocomplete"].is_boolean());
}

#[tokio::test]
async fn map_search_rejects_short_query() {
    let app = pinpoint_desktop::server::map_router()
        .with_state(pinpoint_desktop::server::AppState::new());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/map/search?q=ab")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn map_route_rejects_invalid_coordinates() {
    let app = pinpoint_desktop::server::map_router()
        .with_state(pinpoint_desktop::server::AppState::new());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/map/route?from_lat=999&from_lon=28.8&to_lat=47.0&to_lon=28.9&mode=driving")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
