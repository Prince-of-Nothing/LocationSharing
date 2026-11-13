//! Asset-locality guard: the web UI must never pull fonts, scripts, styles or
//! images from a third-party origin, and the CSP must enforce same-origin.
//!
//! This is deliberately a source-level test: it fails the build if someone
//! pastes a Google Fonts/CDN URL into the shipped HTML/CSS/JS, before a
//! browser ever gets a chance to request it.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

const HTML: &str = include_str!("../web/app/index.html");
const CSS: &str = include_str!("../web/app/app.css");
const JS: &str = include_str!("../web/app/app.js");

const BANNED_HOSTS: &[&str] = &[
    "fonts.googleapis",
    "fonts.gstatic",
    "gstatic.com",
    "unpkg.com",
    "jsdelivr.net",
    "bootstrapcdn",
    "fontawesome",
    "cdnjs.",
    "google-analytics",
    "googletagmanager",
];

#[test]
fn shipped_web_assets_have_no_third_party_hosts() {
    for (name, text) in [("index.html", HTML), ("app.css", CSS), ("app.js", JS)] {
        let lower = text.to_ascii_lowercase();
        for host in BANNED_HOSTS {
            assert!(
                !lower.contains(host),
                "{name} references third-party host `{host}` — assets must be local"
            );
        }
    }
}

#[test]
fn css_and_js_make_no_absolute_requests() {
    assert!(!CSS.contains("http://") && !CSS.contains("https://"));
    assert!(!CSS.contains("@import"), "no imported stylesheets");
    assert!(!CSS.contains("url(http"), "no remote url() assets");
    assert!(!JS.contains("http://") && !JS.contains("https://"));
    assert!(!JS.contains("fetch(\"http"), "all fetches stay same-origin");
    assert!(!JS.contains("fetch('http"));
}

#[test]
fn html_only_links_out_for_attribution() {
    // Subresources (script/link/img src|href) must be relative.
    for line in HTML.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("<script")
            || trimmed.starts_with("<link")
            || trimmed.starts_with("<img")
        {
            assert!(
                !trimmed.contains("http://") && !trimmed.contains("https://"),
                "subresource must be local: {trimmed}"
            );
        }
    }
    // Absolute URLs are allowed only for the ODbL attribution (now rendered
    // by the Leaflet control in map.js); the HTML itself may contain none.
    let absolute = HTML.matches("https://").count();
    assert!(
        absolute <= 1,
        "only the ODbL attribution link may be absolute"
    );
    if absolute == 1 {
        assert!(HTML.contains("https://www.openstreetmap.org/copyright"));
    }
}

#[tokio::test]
async fn csp_is_same_origin_and_blocks_remote_fonts() {
    let app = pinpoint_desktop::server::router(pinpoint_desktop::server::AppState::new());
    let resp = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let csp = resp
        .headers()
        .get("content-security-policy")
        .expect("CSP header")
        .to_str()
        .unwrap();
    assert!(csp.contains("default-src 'self'"), "CSP: {csp}");
    assert!(csp.contains("font-src 'self'"), "CSP must pin fonts: {csp}");
    assert!(csp.contains("script-src 'self'"), "CSP: {csp}");
    assert!(csp.contains("connect-src 'self'"), "CSP: {csp}");
    assert!(
        !csp.contains("https:") && !csp.contains('*'),
        "CSP must not allow remote origins: {csp}"
    );
}

#[tokio::test]
async fn favicon_is_served_locally() {
    let app = pinpoint_desktop::server::router(pinpoint_desktop::server::AppState::new());
    for path in ["/favicon.svg", "/favicon.ico"] {
        let resp = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "{path}");
        assert_eq!(
            resp.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok()),
            Some("image/svg+xml"),
            "{path}"
        );
    }
}

#[tokio::test]
async fn admin_assets_are_served() {
    let app = pinpoint_desktop::server::router(pinpoint_desktop::server::AppState::new());
    for path in [
        "/admin.js",
        "/admin/overview.js",
        "/admin/users.js",
        "/admin/placeholder.js",
        "/sharing.js",
    ] {
        let resp = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "{path}");
    }
}

#[tokio::test]
async fn leaflet_is_vendored_and_served() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let css = std::fs::read_to_string(root.join("web/app/vendor/leaflet.css"))
        .expect("web/app/vendor/leaflet.css must be vendored");
    let css_lower = css.to_ascii_lowercase();
    for host in ["cdn.", "unpkg.com", "jsdelivr.net"] {
        assert!(
            !css_lower.contains(host),
            "vendored Leaflet CSS must not reference {host}"
        );
    }
    assert!(
        !css_lower.contains("url(http"),
        "vendored Leaflet CSS must not load remote url() assets"
    );

    let app = pinpoint_desktop::server::router(pinpoint_desktop::server::AppState::new());
    for path in [
        "/vendor/leaflet.js",
        "/vendor/leaflet.css",
        "/vendor/leaflet/LICENSE",
    ] {
        let resp = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "{path}");
    }
}
