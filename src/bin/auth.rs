//! Auth service binary (pure Rust).
//! Run: `cargo run --bin pinpoint-auth` (BIND_ADDR default 0.0.0.0:8001).
use pinpoint_desktop::server::{auth_router, AppState, ServiceKind};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8001".to_string());
    let state = AppState::from_env_for(ServiceKind::Auth).await;
    let persist_state = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            persist_state.persist().await;
            persist_state.refresh_from_db().await;
        }
    });
    println!("pinpoint-auth listening on http://{addr}/");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, auth_router().with_state(state)).await?;
    Ok(())
}
