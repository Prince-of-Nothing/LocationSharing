//! In-process router probe: proves which routes match (no network needed).

use axum::body::Body;
use axum::http::{Request, StatusCode};
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
