//! Pure-Rust API server for Pinpoint (replaces the archived Go backend).
//!
//! Implements the `/api/v1` wire contract consumed by the Rust client
//! (`src/api.rs`) and the archived web/mobile clients:
//! - `POST /api/v1/auth/login` → `{access_token, csrf_token, user}` + session cookie
//! - `POST /api/v1/auth/register`, `/auth/logout`, `/auth/session`
//! - `POST /api/v1/auth/password/change`, `/auth/password/force-change`
//! - `GET /api/v1/auth/me`
//! - `GET /api/v1/users/`, `GET /api/v1/users/{id}`, `PATCH /api/v1/users/{id}`, admin `POST /users/`
//! - `POST /api/v1/friendships/`, `GET /api/v1/friendships/`, `POST /friendships/{id}/respond`
//! - `POST /locations/shares`, `GET /locations/shares`, `PATCH /locations/shares/{id}`
//! - `POST /locations/me`, `GET /locations/friends`
//! - `GET /health`
//!
//! In-memory store (demo/tests/local). For production, swap `Store` for
//! Postgres behind the same handlers. All validation/consent/precision
//! rules mirror the Go handlers exactly.
//!
//! Run: `cargo run --bin pinpoint-server` (listens on 127.0.0.1:8000).

mod auth_handlers;
mod handlers;
mod locations_handlers;
mod migrations;
mod state;
mod users_handlers;

pub use handlers::sync_status;
pub use state::{AppState, ServiceKind, Store};

pub use crate::models::{FriendLocation, Friendship, LocationShare, User};

use axum::{
    extract::State,
    middleware::Next,
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde_json::json;

/// Per-request convergence for multi-replica determinism.
///
/// With 2 replicas, reading only the in-memory `Store` (5s tick) serves
/// stale data and breaks read-after-write (the e2e saw 29/38 at scale 2).
/// This middleware makes every request converge: refresh from Postgres
/// before the handler, and persist after mutating requests. In-memory
/// mode (no `DATABASE_URL`) and all unit/router tests are a no-op (both
/// calls early-return), so the suite stays green.
/// State is read from request extensions (not a `State` extractor) so this
/// layer composes on `Router<AppState>` before `with_state`.
pub async fn converge(
    State(state): State<AppState>,
    req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> axum::response::Response {
    let method = req.method().clone();
    eprintln!("CONVERGE run {} {}", method, req.uri());
    state.refresh_from_db().await;
    let res = next.run(req).await;
    if method != axum::http::Method::GET {
        state.persist().await;
        eprintln!("CONVERGE persisted");
    }
    res
}

/// Build the full Axum router with all `/api/v1` routes + health.
pub fn router(state: AppState) -> Router {
    let app_state = state.clone();
    auth_router()
        .merge(users_router())
        .merge(locations_router())
        .merge(map_router())
        .route("/health", get(handlers::health))
        .route("/api/v1/sync-status", get(handlers::sync_status))
        .with_state(state)
        .layer(axum::middleware::from_fn_with_state(app_state, converge))
}

/// Auth service routes (`/api/v1/auth/*`). Independently scalable.
/// Stateless: the app state is supplied once by the binary's `with_state`.
pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/api/v1/auth/login", post(auth_handlers::login))
        .route("/api/v1/auth/register", post(auth_handlers::register))
        .route("/api/v1/auth/logout", post(auth_handlers::logout))
        .route("/api/v1/auth/session", post(auth_handlers::session))
        .route("/api/v1/auth/me", get(auth_handlers::me))
        .route(
            "/api/v1/auth/password/change",
            post(auth_handlers::change_password),
        )
        .route(
            "/api/v1/auth/password/force-change",
            post(auth_handlers::force_change_password),
        )
        .route("/api/v1/auth/mfa/enroll", post(auth_handlers::mfa_enroll))
        .route("/api/v1/auth/mfa/verify", post(auth_handlers::mfa_verify))
        .route("/api/v1/auth/mfa/status", get(auth_handlers::mfa_status))
        .route("/api/v1/auth/mfa", delete(auth_handlers::mfa_disable))
}

/// Users + friendships service routes. Independently scalable.
/// NOTE: matchit 0.7 (axum 0.7) uses `:param` syntax, NOT `{param}` (that is axum 0.8).
pub fn users_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health))
        .route(
            "/api/v1/users/",
            get(users_handlers::list_users).post(users_handlers::create_user),
        )
        .route(
            "/api/v1/users/:id",
            get(users_handlers::get_user)
                .patch(users_handlers::update_user)
                .delete(users_handlers::delete_user),
        )
        .route("/api/v1/users/search", get(users_handlers::search_users))
        .route(
            "/api/v1/friendships/",
            get(users_handlers::list_friendships).post(users_handlers::create_friendship),
        )
        .route(
            "/api/v1/friendships/:id/respond",
            post(users_handlers::respond_friendship),
        )
        .route(
            "/api/v1/friendships/:id",
            delete(users_handlers::delete_friendship),
        )
}

/// Locations + shares service routes. Independently scalable.
pub fn locations_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health))
        .route(
            "/api/v1/locations/shares",
            get(locations_handlers::list_shares).post(locations_handlers::create_share),
        )
        .route(
            "/api/v1/locations/shares/:id",
            patch(locations_handlers::update_share).delete(locations_handlers::delete_share),
        )
        .route(
            "/api/v1/locations/me",
            post(locations_handlers::publish_location).get(locations_handlers::get_my_location),
        )
        .route(
            "/api/v1/locations/friends",
            get(locations_handlers::list_friend_locations),
        )
        .route(
            "/api/v1/locations/check-ins",
            get(locations_handlers::list_check_ins).post(locations_handlers::create_check_in),
        )
        .route(
            "/api/v1/locations/check-ins/:id",
            delete(locations_handlers::delete_check_in),
        )
        .route(
            "/api/v1/locations/emergency",
            get(locations_handlers::list_emergencies).post(locations_handlers::trigger_emergency),
        )
        .route(
            "/api/v1/locations/emergency/:id",
            patch(locations_handlers::resolve_emergency),
        )
}

/// Map service routes (`/map/*`): tile proxy + attribution + health.
/// Isolated so map load (tiles/search/routing) never starves auth or locations.
pub fn map_router() -> Router<AppState> {
    Router::new()
        .route("/map/tiles/:z/:x/:y", get(map_handlers::tile))
        .route("/map/attribution", get(map_handlers::attribution))
        .route("/map/health", get(map_handlers::health))
}

pub mod map_handlers {
    use axum::{
        http::{header, HeaderMap, StatusCode},
        response::IntoResponse,
        Json,
    };
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    use std::time::{Duration, Instant};

    type TileKey = (u8, u8, u8);
    type TileCacheMap = HashMap<TileKey, (Vec<u8>, Instant)>;
    static TILE_CACHE: OnceLock<Mutex<TileCacheMap>> = OnceLock::new();
    const TILE_TTL: Duration = Duration::from_secs(3600);
    const TILE_CACHE_MAX: usize = 1024;

    fn tile_cache() -> &'static Mutex<TileCacheMap> {
        TILE_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
    }

    pub fn tile_cache_len() -> usize {
        tile_cache().lock().unwrap().len()
    }

    pub async fn health() -> Json<serde_json::Value> {
        Json(json!({"service": "map", "status": "healthy", "cache_entries": tile_cache_len()}))
    }

    pub async fn attribution() -> Json<serde_json::Value> {
        Json(json!({
            "data": "© OpenStreetMap contributors",
            "license": "https://opendatacommons.org/licenses/odbl/1-0/",
            "tiles": "https://a.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}.png",
        }))
    }

    /// Proxies a single raster tile from CARTO (policy-compliant, OSM ODbL).
    /// Never uses tile.openstreetmap.org. Cached in-memory (TTL 1h, 1024 entries).
    pub async fn tile(
        axum::extract::Path((z, x, y)): axum::extract::Path<(u32, u32, u32)>,
    ) -> impl IntoResponse {
        if z > 19 {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": {"code": "INVALID_ZOOM", "message": "Zoom must be 0-19"}})),
            )
                .into_response();
        }
        let key = (z as u8, x as u8, y as u8);
        if let Some((bytes, inserted)) = tile_cache().lock().unwrap().get(&key) {
            if inserted.elapsed() < TILE_TTL {
                let mut headers = HeaderMap::new();
                headers.insert(header::CONTENT_TYPE, "image/png".parse().unwrap());
                headers.insert(
                    header::CACHE_CONTROL,
                    "public, max-age=3600".parse().unwrap(),
                );
                headers.insert(
                    "X-Data-Attribution",
                    "© OpenStreetMap contributors".parse().unwrap(),
                );
                return (StatusCode::OK, headers, bytes.clone()).into_response();
            }
        }
        // CARTO Basemaps now requires a (free) API key — see
        // https://carto.com/basemaps/apikey. Read CARTO_API_KEY at request
        // time so operators can inject it without rebuilding.
        let key_param = match std::env::var("CARTO_API_KEY") {
            Ok(k) if !k.trim().is_empty() => format!("?apikey={}", k.trim()),
            _ => String::new(),
        };
        let url = format!("https://a.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}.png{key_param}");
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("PinpointMap/0.2.0 (ODbL tile proxy)")
            .build()
        {
            Ok(c) => c,
            Err(_) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({"error": {"code": "TILE_FETCH_FAILED", "message": "Tile service unavailable"}})),
                )
                    .into_response()
            }
        };
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => match resp.bytes().await {
                Ok(body) => {
                    let bytes = body.to_vec();
                    {
                        let mut cache = tile_cache().lock().unwrap();
                        if cache.len() >= TILE_CACHE_MAX {
                            cache.clear();
                        }
                        cache.insert(key, (bytes.clone(), Instant::now()));
                    }
                    let mut headers = HeaderMap::new();
                    headers.insert(header::CONTENT_TYPE, "image/png".parse().unwrap());
                    headers.insert(header::CACHE_CONTROL, "public, max-age=3600".parse().unwrap());
                    headers.insert("X-Data-Attribution", "© OpenStreetMap contributors".parse().unwrap());
                    (StatusCode::OK, headers, bytes).into_response()
                }
                Err(_) => (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({"error": {"code": "TILE_FETCH_FAILED", "message": "Tile read failed"}})),
                )
                    .into_response(),
            },
            Ok(resp) => {
                let upstream = resp.status().as_u16();
                (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({"error": {"code": "TILE_FETCH_FAILED", "message": format!("Upstream tile service returned {upstream}. If upstream is CARTO, set CARTO_API_KEY (free at https://carto.com/basemaps/apikey)."), "upstream_status": upstream}})),
                )
                    .into_response()
            }
            Err(_) => (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": {"code": "TILE_FETCH_FAILED", "message": "Tile service unavailable"}})),
            )
                .into_response(),
        }
    }
}

/// Error envelope matching the Go API: `{"error": {"code": ..., "message": ...}}`.
pub fn error_envelope(code: &str, message: &str) -> axum::response::Response {
    (
        axum::http::StatusCode::BAD_REQUEST,
        Json(json!({"error": {"code": code, "message": message}})),
    )
        .into_response()
}

pub use axum::response::IntoResponse;
