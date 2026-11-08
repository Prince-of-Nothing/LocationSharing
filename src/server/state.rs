//! In-memory store for the Rust API server.
//!
#![allow(clippy::type_complexity, clippy::field_reassign_with_default)]
//!
//! Demo/local only. Production uses Postgres behind the same handler logic.

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::models::{CheckIn, EmergencyAlert, FriendLocation, Friendship, LocationShare, User};

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
    /// never the persist rewrite — see `state.rs::sql_*` helpers.
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
        #[allow(clippy::type_complexity)]
        let Some(pool) = &self.pg
        else {
            return false;
        };
        let users: Vec<(
            i64,
            String,
            String,
            String,
            bool,
            bool,
            Option<String>,
            Option<String>,
            bool,
            i64,
        )> = sqlx::query_as(
            "SELECT id, email, full_name, role, is_active, must_change_password, password_hash, totp_secret, mfa_enabled, mfa_last_step FROM pinpoint_users ORDER BY id",
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        if users.is_empty() {
            return false;
        }
        let mut store = Store::default();
        store.next_user_id = 1;
        store.next_friendship_id = 1;
        store.next_share_id = 1;
        store.next_check_in_id = 1;
        store.next_emergency_id = 1;
        for (
            id,
            email,
            full_name,
            role,
            is_active,
            must_change_password,
            password_hash,
            totp_secret,
            mfa_enabled,
            mfa_last_step,
        ) in users
        {
            store.next_user_id = store.next_user_id.max(id + 1);
            store.users.push(User {
                id,
                email,
                full_name,
                role,
                is_active,
                must_change_password,
                password_hash,
                totp_secret,
                mfa_enabled,
                mfa_last_step: mfa_last_step.max(0) as u64,
            });
        }
        let fr: Vec<(i64, i64, i64, Option<String>, Option<String>, String, Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT id, requester_id, receiver_id, requester_name, receiver_name, status, created_at, responded_at FROM pinpoint_friendships ORDER BY id",
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        for (id, requester_id, receiver_id, rn, dn, status, created_at, responded_at) in fr {
            store.next_friendship_id = store.next_friendship_id.max(id + 1);
            store.friendships.push(Friendship {
                id,
                requester_id,
                receiver_id,
                requester_name: rn.unwrap_or_default(),
                receiver_name: dn.unwrap_or_default(),
                status,
                created_at,
                responded_at,
            });
        }
        let sh: Vec<(i64, i64, i64, String, String, bool, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT id, friendship_id, owner_id, direction, precision_level, is_active, last_location_update_at, expires_at, label FROM pinpoint_shares ORDER BY id",
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        for (
            id,
            friendship_id,
            owner_id,
            direction,
            precision_level,
            is_active,
            last_location_update_at,
            expires_at,
            label,
        ) in sh
        {
            store.next_share_id = store.next_share_id.max(id + 1);
            store.shares.push(LocationShare {
                id,
                friendship_id,
                owner_id,
                direction,
                precision_level,
                is_active,
                last_location_update_at,
                expires_at,
                label,
            });
        }
        let locs: Vec<(i64, Option<String>, f64, f64, Option<String>, Option<String>, Option<f64>)> = sqlx::query_as(
            "SELECT user_id, user_name, latitude, longitude, precision_level, updated_at, accuracy FROM pinpoint_locations ORDER BY user_id",
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        for (user_id, user_name, latitude, longitude, precision_level, updated_at, accuracy) in locs
        {
            store.locations.push(FriendLocation {
                user_id,
                user_name: user_name.unwrap_or_default(),
                latitude,
                longitude,
                precision_level: precision_level.unwrap_or_else(|| "exact".to_string()),
                updated_at: updated_at.unwrap_or_default(),
                accuracy,
            });
        }
        let cis: Vec<(i64, i64, Option<String>, f64, f64, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT id, user_id, user_name, latitude, longitude, note, kind, created_at FROM pinpoint_checkins ORDER BY id",
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        for (id, user_id, user_name, latitude, longitude, note, kind, created_at) in cis {
            store.next_check_in_id = store.next_check_in_id.max(id + 1);
            store.check_ins.push(CheckIn {
                id,
                user_id,
                user_name: user_name.unwrap_or_default(),
                latitude,
                longitude,
                note: note.unwrap_or_default(),
                kind: kind.unwrap_or_else(|| "manual".to_string()),
                created_at: created_at.unwrap_or_default(),
            });
        }
        let ems: Vec<(i64, i64, Option<String>, f64, f64, Option<String>, bool)> = sqlx::query_as(
            "SELECT id, user_id, user_name, latitude, longitude, created_at, resolved FROM pinpoint_emergencies ORDER BY id",
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        for (id, user_id, user_name, latitude, longitude, created_at, resolved) in ems {
            store.next_emergency_id = store.next_emergency_id.max(id + 1);
            store.emergencies.push(EmergencyAlert {
                id,
                user_id,
                user_name: user_name.unwrap_or_default(),
                latitude,
                longitude,
                created_at: created_at.unwrap_or_default(),
                resolved,
            });
        }
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
        let store = {
            let guard = self.store.lock().unwrap();
            Store {
                users: guard.users.clone(),
                friendships: guard.friendships.clone(),
                shares: guard.shares.clone(),
                locations: guard.locations.clone(),
                check_ins: guard.check_ins.clone(),
                emergencies: guard.emergencies.clone(),
                next_user_id: guard.next_user_id,
                next_friendship_id: guard.next_friendship_id,
                next_share_id: guard.next_share_id,
                next_check_in_id: guard.next_check_in_id,
                next_emergency_id: guard.next_emergency_id,
            }
        };
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
        // --- Upserts (parents-first) ---
        if self.owner.owns_users() {
            for u in &store.users {
                let _ = sqlx::query("INSERT INTO pinpoint_users (id, email, full_name, role, is_active, must_change_password, password_hash, totp_secret, mfa_enabled, mfa_last_step) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) ON CONFLICT (id) DO UPDATE SET email = EXCLUDED.email, full_name = EXCLUDED.full_name, role = EXCLUDED.role, is_active = EXCLUDED.is_active, must_change_password = EXCLUDED.must_change_password, password_hash = EXCLUDED.password_hash, totp_secret = EXCLUDED.totp_secret, mfa_enabled = EXCLUDED.mfa_enabled, mfa_last_step = EXCLUDED.mfa_last_step")
                    .bind(u.id).bind(&u.email).bind(&u.full_name).bind(&u.role).bind(u.is_active).bind(u.must_change_password).bind(&u.password_hash).bind(&u.totp_secret).bind(u.mfa_enabled).bind(u.mfa_last_step as i64)
                    .execute(&mut *tx).await;
            }
        }
        if self.owner.owns_friendships() {
            for f in &store.friendships {
                let _ = sqlx::query("INSERT INTO pinpoint_friendships (id, requester_id, receiver_id, requester_name, receiver_name, status, created_at, responded_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT (id) DO UPDATE SET requester_id = EXCLUDED.requester_id, receiver_id = EXCLUDED.receiver_id, requester_name = EXCLUDED.requester_name, receiver_name = EXCLUDED.receiver_name, status = EXCLUDED.status, created_at = EXCLUDED.created_at, responded_at = EXCLUDED.responded_at")
                    .bind(f.id).bind(f.requester_id).bind(f.receiver_id).bind(&f.requester_name).bind(&f.receiver_name).bind(&f.status).bind(&f.created_at).bind(&f.responded_at)
                    .execute(&mut *tx).await;
            }
        }
        if self.owner.owns_shares() {
            for s in &store.shares {
                let _ = sqlx::query("INSERT INTO pinpoint_shares (id, friendship_id, owner_id, direction, precision_level, is_active, last_location_update_at, expires_at, label) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT (id) DO UPDATE SET friendship_id = EXCLUDED.friendship_id, owner_id = EXCLUDED.owner_id, direction = EXCLUDED.direction, precision_level = EXCLUDED.precision_level, is_active = EXCLUDED.is_active, last_location_update_at = EXCLUDED.last_location_update_at, expires_at = EXCLUDED.expires_at, label = EXCLUDED.label")
                    .bind(s.id).bind(s.friendship_id).bind(s.owner_id).bind(&s.direction).bind(&s.precision_level).bind(s.is_active).bind(&s.last_location_update_at).bind(&s.expires_at).bind(&s.label)
                    .execute(&mut *tx).await;
            }
        }
        if self.owner.owns_locations() {
            for l in &store.locations {
                let _ = sqlx::query("INSERT INTO pinpoint_locations (user_id, user_name, latitude, longitude, precision_level, updated_at, accuracy) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT (user_id) DO UPDATE SET user_name = EXCLUDED.user_name, latitude = EXCLUDED.latitude, longitude = EXCLUDED.longitude, precision_level = EXCLUDED.precision_level, updated_at = EXCLUDED.updated_at, accuracy = EXCLUDED.accuracy")
                    .bind(l.user_id).bind(&l.user_name).bind(l.latitude).bind(l.longitude).bind(&l.precision_level).bind(&l.updated_at).bind(l.accuracy)
                    .execute(&mut *tx).await;
            }
        }
        if self.owner.owns_checkins() {
            for c in &store.check_ins {
                let _ = sqlx::query("INSERT INTO pinpoint_checkins (id, user_id, user_name, latitude, longitude, note, kind, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT (id) DO UPDATE SET user_id = EXCLUDED.user_id, user_name = EXCLUDED.user_name, latitude = EXCLUDED.latitude, longitude = EXCLUDED.longitude, note = EXCLUDED.note, kind = EXCLUDED.kind, created_at = EXCLUDED.created_at")
                    .bind(c.id).bind(c.user_id).bind(&c.user_name).bind(c.latitude).bind(c.longitude).bind(&c.note).bind(&c.kind).bind(&c.created_at)
                    .execute(&mut *tx).await;
            }
        }
        if self.owner.owns_emergencies() {
            for e in &store.emergencies {
                let _ = sqlx::query("INSERT INTO pinpoint_emergencies (id, user_id, user_name, latitude, longitude, created_at, resolved) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT (id) DO UPDATE SET user_id = EXCLUDED.user_id, user_name = EXCLUDED.user_name, latitude = EXCLUDED.latitude, longitude = EXCLUDED.longitude, created_at = EXCLUDED.created_at, resolved = EXCLUDED.resolved")
                    .bind(e.id).bind(e.user_id).bind(&e.user_name).bind(e.latitude).bind(e.longitude).bind(&e.created_at).bind(e.resolved)
                    .execute(&mut *tx).await;
            }
        }
        // --- Delete rows removed from this snapshot (children-first) ---
        // `<> ALL($1)` with an empty list deletes all rows (legitimate: the
        // owner removed every row); with a non-empty list only the missing
        // keys go away. Rows written by another replica after our snapshot
        // would also be removed if absent here — the advisory lock keeps
        // that window to one tick and `refresh_from_db()` re-converges us.
        if self.owner.owns_emergencies() {
            let ids: Vec<i64> = store.emergencies.iter().map(|e| e.id).collect();
            let _ = sqlx::query("DELETE FROM pinpoint_emergencies WHERE id <> ALL($1::bigint[])")
                .bind(&ids)
                .execute(&mut *tx)
                .await;
        }
        if self.owner.owns_checkins() {
            let ids: Vec<i64> = store.check_ins.iter().map(|c| c.id).collect();
            let _ = sqlx::query("DELETE FROM pinpoint_checkins WHERE id <> ALL($1::bigint[])")
                .bind(&ids)
                .execute(&mut *tx)
                .await;
        }
        if self.owner.owns_locations() {
            let ids: Vec<i64> = store.locations.iter().map(|l| l.user_id).collect();
            let _ =
                sqlx::query("DELETE FROM pinpoint_locations WHERE user_id <> ALL($1::bigint[])")
                    .bind(&ids)
                    .execute(&mut *tx)
                    .await;
        }
        if self.owner.owns_shares() {
            let ids: Vec<i64> = store.shares.iter().map(|s| s.id).collect();
            let _ = sqlx::query("DELETE FROM pinpoint_shares WHERE id <> ALL($1::bigint[])")
                .bind(&ids)
                .execute(&mut *tx)
                .await;
        }
        if self.owner.owns_friendships() {
            let ids: Vec<i64> = store.friendships.iter().map(|f| f.id).collect();
            let _ = sqlx::query("DELETE FROM pinpoint_friendships WHERE id <> ALL($1::bigint[])")
                .bind(&ids)
                .execute(&mut *tx)
                .await;
        }
        if self.owner.owns_users() {
            let ids: Vec<i64> = store.users.iter().map(|u| u.id).collect();
            let _ = sqlx::query("DELETE FROM pinpoint_users WHERE id <> ALL($1::bigint[])")
                .bind(&ids)
                .execute(&mut *tx)
                .await;
        }
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

    /// Direct user insert for the users service (Auth owns `pinpoint_users`
    /// for persist upserts, so the users service must upsert single rows —
    /// never delete+rewrite — to avoid clobbering credentials written by Auth).
    /// Returns the assigned id. Falls back to the in-memory counter when the
    /// DB is unreachable (demo mode: in-memory store is authoritative).
    pub async fn sql_insert_user(
        &self,
        email: &str,
        full_name: &str,
        role: &str,
        password_hash: Option<&str>,
    ) -> i64 {
        if let Some(pool) = &self.pg {
            let row: Option<(i64,)> = sqlx::query_as(
                "INSERT INTO pinpoint_users (id, email, full_name, role, is_active, must_change_password, password_hash) SELECT COALESCE((SELECT max(id) FROM pinpoint_users), 0) + 1, $1, $2, $3, true, false, $4 RETURNING id",
            )
            .bind(email)
            .bind(full_name)
            .bind(role)
            .bind(password_hash)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
            if let Some((id,)) = row {
                return id;
            }
        }
        let mut store = self.store.lock().unwrap();
        let id = store.next_user_id;
        store.next_user_id += 1;
        id
    }

    /// Direct user update; no-op when DB unreachable/in-memory mode.
    pub async fn sql_update_user(
        &self,
        id: i64,
        full_name: Option<&str>,
        role: Option<&str>,
        is_active: Option<bool>,
    ) {
        let Some(pool) = &self.pg else { return };
        if let Some(n) = full_name {
            let _ = sqlx::query("UPDATE pinpoint_users SET full_name=$1 WHERE id=$2")
                .bind(n)
                .bind(id)
                .execute(pool)
                .await;
        }
        if let Some(r) = role {
            let _ = sqlx::query("UPDATE pinpoint_users SET role=$1 WHERE id=$2")
                .bind(r)
                .bind(id)
                .execute(pool)
                .await;
        }
        if let Some(a) = is_active {
            let _ = sqlx::query("UPDATE pinpoint_users SET is_active=$1 WHERE id=$2")
                .bind(a)
                .bind(id)
                .execute(pool)
                .await;
        }
    }

    /// Direct user delete; no-op in in-memory mode (store handles it).
    pub async fn sql_delete_user(&self, id: i64) {
        let Some(pool) = &self.pg else { return };
        let _ = sqlx::query("DELETE FROM pinpoint_users WHERE id=$1")
            .bind(id)
            .execute(pool)
            .await;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// In-memory data store (demo/local; Postgres snapshot persistence when configured).
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Store {
    #[serde(default)]
    pub users: Vec<User>,
    #[serde(default)]
    pub friendships: Vec<Friendship>,
    #[serde(default)]
    pub shares: Vec<LocationShare>,
    #[serde(default)]
    pub locations: Vec<FriendLocation>,
    #[serde(default)]
    pub check_ins: Vec<CheckIn>,
    #[serde(default)]
    pub emergencies: Vec<EmergencyAlert>,
    #[serde(default)]
    pub next_user_id: i64,
    #[serde(default)]
    pub next_friendship_id: i64,
    #[serde(default)]
    pub next_share_id: i64,
    #[serde(default)]
    pub next_check_in_id: i64,
    #[serde(default)]
    pub next_emergency_id: i64,
}

impl Store {
    pub fn new() -> Self {
        let mut s = Self {
            next_user_id: 1,
            next_friendship_id: 1,
            next_share_id: 1,
            next_check_in_id: 1,
            next_emergency_id: 1,
            ..Default::default()
        };
        s.seed_demo();
        s
    }

    /// Seed demo accounts (idempotent): alice, bob, admin.
    pub fn seed_demo(&mut self) {
        use crate::security::password::hash_password;
        if !self.users.is_empty() {
            return;
        }
        let admin = User {
            id: self.next_user_id,
            email: "admin@example.com".into(),
            full_name: "Administrator".into(),
            role: "administrator".into(),
            is_active: true,
            must_change_password: true,
            password_hash: Some(hash_password(
                "ChangeMe123!",
                crate::security::password::DEFAULT_ITERS,
            )),
            totp_secret: None,
            mfa_enabled: false,
            mfa_last_step: 0,
        };
        self.next_user_id += 1;
        let alice = User {
            id: self.next_user_id,
            email: "alice@example.com".into(),
            full_name: "Alice Example".into(),
            role: "resident".into(),
            is_active: true,
            must_change_password: false,
            password_hash: Some(hash_password(
                "DemoPass123!",
                crate::security::password::DEFAULT_ITERS,
            )),
            totp_secret: None,
            mfa_enabled: false,
            mfa_last_step: 0,
        };
        self.next_user_id += 1;
        let bob = User {
            id: self.next_user_id,
            email: "bob@example.com".into(),
            full_name: "Bob Example".into(),
            role: "resident".into(),
            is_active: true,
            must_change_password: false,
            password_hash: Some(hash_password(
                "DemoPass456!",
                crate::security::password::DEFAULT_ITERS,
            )),
            totp_secret: None,
            mfa_enabled: false,
            mfa_last_step: 0,
        };
        self.next_user_id += 1;

        let friendship = Friendship {
            id: self.next_friendship_id,
            requester_id: alice.id,
            receiver_id: bob.id,
            requester_name: alice.full_name.clone(),
            receiver_name: bob.full_name.clone(),
            status: "accepted".into(),
            created_at: Some("2026-01-01T00:00:00Z".into()),
            responded_at: Some("2026-01-01T00:00:00Z".into()),
        };
        self.next_friendship_id += 1;

        self.users = vec![admin, alice, bob];
        self.friendships = vec![friendship];
        // Demo locations (Chicago)
        self.locations = vec![
            FriendLocation {
                user_id: self.users[1].id,
                user_name: "Alice Example".into(),
                latitude: 41.8902,
                longitude: -87.6323,
                precision_level: "exact".into(),
                updated_at: "2026-11-08T12:00:00Z".into(),
                accuracy: Some(8.0),
            },
            FriendLocation {
                user_id: self.users[2].id,
                user_name: "Bob Example".into(),
                latitude: 41.8781,
                longitude: -87.6298,
                precision_level: "exact".into(),
                updated_at: "2026-11-08T12:00:00Z".into(),
                accuracy: Some(10.0),
            },
        ];
    }

    pub fn find_user_by_email(&self, email: &str) -> Option<&User> {
        self.users.iter().find(|u| u.email == email)
    }

    pub fn find_user_by_id(&self, id: i64) -> Option<&User> {
        self.users.iter().find(|u| u.id == id)
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
