//! Live end-to-end smoke against the real Go API (default stack).
//!
//! Ignored by default so CI stays hermetic; run it against a seeded local
//! server to prove the client's wire contract matches the real handlers:
//!
//! ```text
//! cd backend && SEED_DEMO_DATA=true go run ./cmd/server
//! cargo test --test live_api -- --ignored
//! ```
//!
//! Uses only the repository's local demo seed accounts (never real data).

use pinpoint_desktop::api::ApiClient;

#[tokio::test]
#[ignore = "requires a seeded Go API on http://127.0.0.1:8000"]
async fn demo_account_round_trip_against_go_api() {
    let client = ApiClient::new("http://127.0.0.1:8000").expect("client");

    // 1. Login: envelope, Set-Cookie capture and CSRF token extraction.
    let user = client
        .login("alice@example.com", "DemoPass123!")
        .await
        .expect("login against the real /api/v1/auth/login");
    assert_eq!(user.email, "alice@example.com");
    assert!(client.session_active());

    // 2. Reads: the real response shapes must parse into our models.
    let friendships = client.get_friendships().await.expect("friendships");
    assert!(!friendships.is_empty(), "demo seed creates a friendship");
    let shares = client.get_location_shares().await.expect("shares");
    let locations = client
        .get_friend_locations()
        .await
        .expect("friend locations");
    assert!(!locations.is_empty(), "demo seed shares locations");
    let _ = shares;

    // 3. State-changing request: must pass the server's CSRF double-submit
    //    middleware with the captured token.
    client
        .publish_location(48.8566, 2.3522, 12.0)
        .await
        .expect("publish must pass requireAuth + CSRF checks");

    // 4. Logout: server-confirmed revocation clears local state...
    client.logout().await.expect("logout");
    assert!(!client.session_active());

    // 5. ...and the revoked session must no longer authenticate (fail closed).
    assert!(
        client.get_friendships().await.is_err(),
        "requests after logout must be rejected by the server"
    );
}
