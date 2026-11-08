//! In-memory store for the Rust API server (modular: per-domain sub-stores).
//!
//! `store.rs` holds the aggregate shape, `users.rs` / `friendships.rs` /
//! `locations.rs` hold per-table load/persist, and this file orchestrates:
//! ownership (`ServiceKind`), boot (`from_env`/`init_postgres`), whole-store
//! load/refresh, and the persist tick (one transaction per tick).
//!
//! Demo/local only. Production uses Postgres behind the same handler logic.

#![allow(clippy::type_complexity, clippy::field_reassign_with_default)]

mod friendships;
mod locations;
mod store;
mod users;

pub use store::Store;

use std::sync::{Arc, Mutex};

/// Application state shared between handlers.
#[derive(Debug, Clone)]
pub struct AppState {
    pub store: Arc<Mutex<Store>>,
    pub pg: Option<sqlx::PgPool>,
    /// Which service owns this instance — scopes `persist()` to owned tables
    /// only, so services never clobber each other's rows.
    pub owner: ServiceKind,
}

/// Which deployable this `AppState` backs (controls persistence ownership).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceKind {
    /// Combined binary (`pinpoint-server`) — owns everything.
    All,
    Auth,
    Users,
    Locations,
    MapOnly,
}

impl ServiceKind {
    /// `pinpoint_users` is owned EXCLUSIVELY by Auth (credentials + profile
    /// columns). The users service touches user rows via direct SQL only,
    /// never the persist rewrite — see `users.rs::sql_*` helpers.
    pub fn owns_users(self) -> bool {
        matches!(self, Self::All | Self::Auth)
    }
    pub fn owns_friendships(self) -> bool {
        matches!(self, Self::All | Self::Users)
    }
    pub fn owns_shares(self) -> bool {
        matches!(self, Self::All | Self::Locations)
    }
    pub fn owns_locations(self) -> bool {
        matches!(self, Self::All | Self::Locations)
    }
    pub fn owns_checkins(self) -> bool {
        matches!(self, Self::All | Self::Locations)
    }
    pub fn owns_emergencies(self) -> bool {
        matches!(self, Self::All | Self::Locations)
    }
    pub fn owns_snapshot(self) -> bool {
        matches!(self, Self::All)
    }

    /// Stable advisory-lock key per service, serializing the persist tick
    /// across replicas of the same service (`pg_advisory_xact_lock` inside
    /// the per-tick transaction in `persist()`). Distinct keys let
    /// auth/users/locations persist concurrently without blocking each
    /// other; replicas of the same service take turns. `MapOnly` never
    /// persists, so its key is unused but still distinct for safety.
    pub fn lock_key(self) -> i64 {
        match self {
            Self::All => 1,
            Self::Auth => 2,
            Self::Users => 3,
            Self::Locations => 4,
            Self::MapOnly => 5,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::for_owner(ServiceKind::All)
    }

    pub fn for_owner(owner: ServiceKind) -> Self {
        Self {
            store: Arc::new(Mutex::new(Store::new())),
            pg: None,
            owner,
        }
    }

    /// Production constructor: connects to Postgres when `DATABASE_URL`
    /// is set, loads the persisted snapshot (or seeds demo data), and
    /// persists mutations back to Postgres. Falls back to in-memory
    /// demo mode when `DATABASE_URL` is empty (local/tests).
    /// Ownership scopes which tables this instance may write.
    pub async fn from_env() -> Self {
        Self::from_env_for(ServiceKind::All).await
    }

    pub async fn from_env_for(owner: ServiceKind) -> Self {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_default();
        if database_url.trim().is_empty() {
            return Self::for_owner(owner);
        }
        match sqlx::PgPool::connect(&database_url).await {
            Ok(pool) => {
                let app = AppState {
                    store: Arc::new(Mutex::new(Store::new())),
                    pg: Some(pool),
                    owner,
                };
                app.init_postgres().await;
                app
            }
            Err(e) => {
                eprintln!("warning: Postgres unavailable ({e}); running in-memory demo mode");
                Self::for_owner(owner)
            }
        }
    }

    async fn init_postgres(&self) {
        let Some(pool) = &self.pg else { return };
        // Typed migration history (v1..v10) applies at boot; old DBs converge
        // idempotently since every statement is CREATE/ALTER IF NOT EXISTS.
        let applied = crate::server::migrations::apply_migrations(pool).await;
        if !applied.is_empty() {
            eprintln!("applied migrations: {:?}", applied);
        }
        // Normalized is authoritative once it has rows.
        if self.load_normalized().await {
            return;
        }
        // Else migrate legacy JSONB snapshot (or seed fresh), then normalize.
        let loaded: Option<(serde_json::Value,)> =
            sqlx::query_as("SELECT data FROM pinpoint_state WHERE id = 1")
                .fetch_optional(pool)
                .await
                .ok()
                .flatten();
        if let Some((data,)) = loaded {
            if let Ok(store) = serde_json::from_value::<Store>(data) {
                *self.store.lock().unwrap() = store;
                let _ = self.persist().await;
                return;
            }
        }
        self.persist().await;
    }

    async fn load_normalized(&self) -> bool {
        let Some(_) = &self.pg else {
            return false;
        };
        let mut store = Store::default();
        store.next_user_id = 1;
        store.next_friendship_id = 1;
        store.next_share_id = 1;
        store.next_check_in_id = 1;
        store.next_emergency_id = 1;
        if !self.load_users(&mut store).await {
            return false;
        }
        self.load_friendships(&mut store).await;
        self.load_locations_domain(&mut store).await;
        *self.store.lock().unwrap() = store;
        true
    }

    /// Reload every table from Postgres into the in-memory Store (merge
    /// convergence). Callers run this right AFTER `persist()` on each tick so
    /// cross-service writes (e.g. users-service direct-SQL user rows visible
    /// to the auth service) converge within one interval. No-op without DB
    /// or when the DB has no rows yet.
    pub async fn refresh_from_db(&self) {
        let Some(_) = &self.pg else { return };
        // load_normalized replaces the whole Store from DB rows.
        self.load_normalized().await;
    }

    /// Persist owned tables to Postgres (no-op without DB).
    ///
    /// Modularity: each owned table is written per-row (upsert +
    /// delete-missing) inside one transaction serialized across replicas
    /// of the same service by `pg_advisory_xact_lock(owner.lock_key())`.
    /// A service only writes tables it owns, so parallel services never
    /// clobber each other's rows. Reads (load) are shared by all.
    /// The legacy JSONB snapshot is written only by `ServiceKind::All`.
    ///
    /// Order matters for the FK graph: upserts run parents-first
    /// (users → friendships → shares → locations → check-ins →
    /// emergencies), deletes run children-first. Unlike the old
    /// wipe-rewrite, a row written by another replica between ticks is
    /// preserved unless this snapshot legitimately removed it.
    pub async fn persist(&self) {
        let Some(pool) = &self.pg else { return };
        // Snapshot the in-memory store once (lock released before I/O).
        let store = self.store.lock().unwrap().snapshot();
        let mut tx = match pool.begin().await {
            Ok(tx) => tx,
            Err(_) => return,
        };
        // Replicas of the same service serialize their tick; different
        // services (different keys) proceed concurrently.
        let _ = sqlx::query("SELECT pg_advisory_xact_lock($1)")
            .bind(self.owner.lock_key())
            .execute(&mut *tx)
            .await;
        // Rollback snapshot (combined binary only).
        if self.owner.owns_snapshot() {
            let data = serde_json::to_value(&store).unwrap_or_default();
            let _ = sqlx::query(
                "INSERT INTO pinpoint_state(id, data) VALUES (1, $1) ON CONFLICT (id) DO UPDATE SET data = EXCLUDED.data",
            )
            .bind(data)
            .execute(&mut *tx)
            .await;
        }
        // Domain persists, parents-first (users → friendships → locations).
        self.persist_users(&mut tx, &store).await;
        self.persist_friendships(&mut tx, &store).await;
        self.persist_locations_domain(&mut tx, &store).await;
        // Counter columns are owned by the same services as their tables;
        // update only owned columns so writers don't overwrite each other.
        if self.owner.owns_users() {
            let _ = sqlx::query("UPDATE pinpoint_counters SET next_user_id=$1 WHERE id=1")
                .bind(store.next_user_id)
                .execute(&mut *tx)
                .await;
        }
        if self.owner.owns_friendships() {
            let _ = sqlx::query("UPDATE pinpoint_counters SET next_friendship_id=$1 WHERE id=1")
                .bind(store.next_friendship_id)
                .execute(&mut *tx)
                .await;
        }
        if self.owner.owns_shares() {
            let _ = sqlx::query("UPDATE pinpoint_counters SET next_share_id=$1, next_checkin_id=$2, next_emergency_id=$3 WHERE id=1")
                .bind(store.next_share_id)
                .bind(store.next_check_in_id)
                .bind(store.next_emergency_id)
                .execute(&mut *tx)
                .await;
        }
        let _ = tx.commit().await;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::ServiceKind;

    #[test]
    fn ownership_matrix_is_exclusive() {
        // Auth owns users only.
        assert!(ServiceKind::Auth.owns_users());
        assert!(!ServiceKind::Auth.owns_friendships());
        assert!(!ServiceKind::Auth.owns_shares());
        // Users owns friendships only (user rows go via direct SQL).
        assert!(ServiceKind::Users.owns_friendships());
        assert!(!ServiceKind::Users.owns_users());
        assert!(!ServiceKind::Users.owns_locations());
        // Locations owns its four tables only.
        assert!(ServiceKind::Locations.owns_shares());
        assert!(ServiceKind::Locations.owns_locations());
        assert!(ServiceKind::Locations.owns_checkins());
        assert!(ServiceKind::Locations.owns_emergencies());
        assert!(!ServiceKind::Locations.owns_users());
        // Map owns nothing.
        assert!(!ServiceKind::MapOnly.owns_users());
        assert!(!ServiceKind::MapOnly.owns_friendships());
        assert!(!ServiceKind::MapOnly.owns_locations());
        // Combined binary owns everything + the rollback snapshot.
        assert!(ServiceKind::All.owns_users());
        assert!(ServiceKind::All.owns_snapshot());
        assert!(!ServiceKind::Auth.owns_snapshot());
    }

    #[test]
    fn lock_keys_are_distinct_per_service() {
        let kinds = [
            ServiceKind::All,
            ServiceKind::Auth,
            ServiceKind::Users,
            ServiceKind::Locations,
            ServiceKind::MapOnly,
        ];
        let keys: Vec<i64> = kinds.iter().map(|k| k.lock_key()).collect();
        let mut deduped = keys.clone();
        deduped.sort_unstable();
        deduped.dedup();
        assert_eq!(deduped.len(), keys.len(), "lock keys must be distinct");
        assert!(keys.iter().all(|k| *k > 0));
    }
}
