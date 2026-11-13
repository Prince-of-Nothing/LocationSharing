//! Headless demo client for the Alice/Bob containers.
//!
//! Run: `cargo run --bin pinpoint-demo-client`
//! Env: `PINPOINT_BACKEND`, `PINPOINT_EMAIL`, `PINPOINT_PASSWORD`,
//! `PINPOINT_LAT`, `PINPOINT_LON`, `PINPOINT_ACCURACY`,
//! `PINPOINT_INTERVAL_SECS`, `PINPOINT_ONCE=1`.
//!
//! This is a real client, not a server-side seed: it signs in through
//! `/api/v1/auth/login`, publishes through `/api/v1/locations/me`, and
//! re-authenticates when its session expires. It prints only the public handle
//! and coordinates — never the password or email.

use pinpoint_desktop::api::ApiClient;
use std::time::Duration;

fn env_f64(name: &str, default: f64) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.trim().parse::<f64>().ok())
        .unwrap_or(default)
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(default)
}

fn env_flag(name: &str) -> bool {
    std::env::var(name)
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let backend =
        std::env::var("PINPOINT_BACKEND").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
    let email = std::env::var("PINPOINT_EMAIL")
        .map_err(|_| anyhow::anyhow!("PINPOINT_EMAIL is required"))?;
    let password = std::env::var("PINPOINT_PASSWORD")
        .map_err(|_| anyhow::anyhow!("PINPOINT_PASSWORD is required"))?;
    let lat = env_f64("PINPOINT_LAT", 47.0105);
    let lon = env_f64("PINPOINT_LON", 28.8638);
    let accuracy = env_f64("PINPOINT_ACCURACY", 12.0);
    let interval = Duration::from_secs(env_u64("PINPOINT_INTERVAL_SECS", 30));
    let once = env_flag("PINPOINT_ONCE");
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
        anyhow::bail!("PINPOINT_LAT/PINPOINT_LON are out of range");
    }
    if !(aux_valid(accuracy)) {
        anyhow::bail!("PINPOINT_ACCURACY must be positive");
    }

    let client = ApiClient::new(backend)?;
    let mut label = String::from("demo client");
    let mut logged_in = false;

    loop {
        if !logged_in {
            match client.login(&email, &password).await {
                Ok(user) => {
                    label = if user.username.is_empty() {
                        format!("user-{}", user.id)
                    } else {
                        format!("@{}", user.username)
                    };
                    println!("{label} connected");
                    logged_in = true;
                }
                Err(err) => {
                    eprintln!("{label} login failed: {err}");
                    if once {
                        return Err(err);
                    }
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            }
        }

        match client.publish_location(lat, lon, accuracy).await {
            Ok(()) => println!("{label} published {lat:.5},{lon:.5}"),
            Err(err) => {
                eprintln!("{label} publish failed: {err}");
                // Force a fresh login on the next tick: the session may have
                // expired or been revoked by an admin.
                logged_in = false;
            }
        }

        if once {
            return Ok(());
        }
        tokio::time::sleep(interval).await;
    }
}

fn aux_valid(accuracy: f64) -> bool {
    accuracy.is_finite() && accuracy > 0.0 && accuracy <= 100_000.0
}
