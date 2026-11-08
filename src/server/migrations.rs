//! Typed migration history for the modular Postgres schema.
//!
//! Each entry is an ordered (version, name, statements) triple applied once
//! and recorded in `pinpoint_schema_migrations`. All statements are
//! idempotent (`IF NOT EXISTS`), so old databases (created by the legacy
//! CREATE-block) converge by simply recording versions. New tables/columns
//! go here — never ad-hoc DDL elsewhere.

pub const MIGRATIONS: &[(u32, &str, &[&str])] = &[
    (
        1,
        "users",
        &["CREATE TABLE IF NOT EXISTS pinpoint_users (id BIGINT PRIMARY KEY, email TEXT UNIQUE NOT NULL, full_name TEXT NOT NULL, role TEXT NOT NULL, is_active BOOLEAN NOT NULL DEFAULT true, must_change_password BOOLEAN NOT NULL DEFAULT false, password_hash TEXT, totp_secret TEXT, mfa_enabled BOOLEAN NOT NULL DEFAULT false, mfa_last_step BIGINT NOT NULL DEFAULT 0)"],
    ),
    (
        2,
        "friendships",
        &["CREATE TABLE IF NOT EXISTS pinpoint_friendships (id BIGINT PRIMARY KEY, requester_id BIGINT NOT NULL, receiver_id BIGINT NOT NULL, requester_name TEXT, receiver_name TEXT, status TEXT NOT NULL, created_at TEXT, responded_at TEXT)"],
    ),
    (
        3,
        "shares",
        &["CREATE TABLE IF NOT EXISTS pinpoint_shares (id BIGINT PRIMARY KEY, friendship_id BIGINT NOT NULL, owner_id BIGINT NOT NULL, direction TEXT NOT NULL, precision_level TEXT NOT NULL, is_active BOOLEAN NOT NULL DEFAULT true, last_location_update_at TEXT, expires_at TEXT, label TEXT)"],
    ),
    (
        4,
        "locations",
        &["CREATE TABLE IF NOT EXISTS pinpoint_locations (user_id BIGINT PRIMARY KEY, user_name TEXT, latitude DOUBLE PRECISION NOT NULL, longitude DOUBLE PRECISION NOT NULL, precision_level TEXT, updated_at TEXT, accuracy DOUBLE PRECISION)"],
    ),
    (
        5,
        "checkins",
        &["CREATE TABLE IF NOT EXISTS pinpoint_checkins (id BIGINT PRIMARY KEY, user_id BIGINT NOT NULL, user_name TEXT, latitude DOUBLE PRECISION, longitude DOUBLE PRECISION, note TEXT, kind TEXT, created_at TEXT)"],
    ),
    (
        6,
        "emergencies",
        &["CREATE TABLE IF NOT EXISTS pinpoint_emergencies (id BIGINT PRIMARY KEY, user_id BIGINT NOT NULL, user_name TEXT, latitude DOUBLE PRECISION, longitude DOUBLE PRECISION, created_at TEXT, resolved BOOLEAN NOT NULL DEFAULT false)"],
    ),
    (
        7,
        "counters",
        &["CREATE TABLE IF NOT EXISTS pinpoint_counters (id INT PRIMARY KEY CHECK (id = 1), next_user_id BIGINT, next_friendship_id BIGINT, next_share_id BIGINT, next_checkin_id BIGINT, next_emergency_id BIGINT)"],
    ),
    (
        8,
        "state_rollback",
        &["CREATE TABLE IF NOT EXISTS pinpoint_state (id INT PRIMARY KEY CHECK (id = 1), data JSONB NOT NULL)"],
    ),
    (
        9,
        "counters_seed",
        &["INSERT INTO pinpoint_counters (id, next_user_id, next_friendship_id, next_share_id, next_checkin_id, next_emergency_id) VALUES (1, 1, 1, 1, 1, 1) ON CONFLICT (id) DO NOTHING"],
    ),
    (
        10,
        "mfa_columns",
        &[
            "ALTER TABLE pinpoint_users ADD COLUMN IF NOT EXISTS totp_secret TEXT",
            "ALTER TABLE pinpoint_users ADD COLUMN IF NOT EXISTS mfa_enabled BOOLEAN NOT NULL DEFAULT false",
            "ALTER TABLE pinpoint_users ADD COLUMN IF NOT EXISTS mfa_last_step BIGINT NOT NULL DEFAULT 0",
        ],
    ),
];

/// Apply all pending migrations (idempotent, recorded in version table).
pub async fn apply_migrations(pool: &sqlx::PgPool) -> Vec<u32> {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS pinpoint_schema_migrations (version INT PRIMARY KEY, name TEXT NOT NULL, applied_at TIMESTAMPTZ NOT NULL DEFAULT now())",
    )
    .execute(pool)
    .await;
    let applied: Vec<(i32,)> = sqlx::query_as("SELECT version FROM pinpoint_schema_migrations")
        .fetch_all(pool)
        .await
        .unwrap_or_default();
    let done: std::collections::HashSet<i32> = applied.into_iter().map(|(v,)| v).collect();
    let mut newly = Vec::new();
    for (version, name, statements) in MIGRATIONS {
        if done.contains(&(*version as i32)) {
            continue;
        }
        let mut tx = match pool.begin().await {
            Ok(tx) => tx,
            Err(_) => return newly,
        };
        let mut ok = true;
        for stmt in *statements {
            if sqlx::query(stmt).execute(&mut *tx).await.is_err() {
                ok = false;
                break;
            }
        }
        // End-state fallback: older DBs already carry e.g. the MFA columns
        // (created by legacy DDL); ALTER then fails only due to table
        // ownership, but the desired state holds — record as applied.
        if !ok && *version == 10 {
            let cols: Vec<(i64,)> = sqlx::query_as(
                "SELECT count(*) FROM information_schema.columns WHERE table_name = 'pinpoint_users' AND column_name IN ('totp_secret','mfa_enabled','mfa_last_step')",
            )
            .fetch_all(&mut *tx)
            .await
            .unwrap_or_default();
            ok = cols.first().map(|(c,)| *c == 3).unwrap_or(false);
        }
        if ok
            && sqlx::query(
                "INSERT INTO pinpoint_schema_migrations (version, name) VALUES ($1, $2) ON CONFLICT (version) DO NOTHING",
            )
            .bind(*version as i32)
            .bind(*name)
            .execute(&mut *tx)
            .await
            .is_ok()
        {
            newly.push(*version);
        }
        let _ = if ok {
            tx.commit().await
        } else {
            tx.rollback().await
        };
    }
    newly
}
