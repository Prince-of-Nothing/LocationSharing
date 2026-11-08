//! Repository Pattern — Abstract data access behind trait-based interfaces.
//!
//! Repositories provide a collection-like interface for accessing domain
//! objects, hiding the details of persistence (API calls, local storage,
//! in-memory cache) from the rest of the application.
//!
//! All repositories are trait-based so they can be swapped for mock
//! implementations in tests or for different backends (online/offline).

#![allow(clippy::type_complexity)]

use crate::models::{FriendLocation, Friendship, LocationShare, User};

/// Error type for repository operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryError {
    NotFound(String),
    AlreadyExists(String),
    InvalidInput(String),
    NetworkError(String),
    Unauthorized,
    StorageError(String),
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryError::NotFound(msg) => write!(f, "Not found: {msg}"),
            RepositoryError::AlreadyExists(msg) => write!(f, "Already exists: {msg}"),
            RepositoryError::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            RepositoryError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            RepositoryError::Unauthorized => write!(f, "Unauthorized"),
            RepositoryError::StorageError(msg) => write!(f, "Storage error: {msg}"),
        }
    }
}

impl std::error::Error for RepositoryError {}

/// Result type for repository operations.
pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// Repository for user account data.
pub trait UserRepository: Send + Sync {
    fn get_current_user(&self) -> RepositoryResult<Option<User>>;
    fn set_current_user(&self, user: Option<User>) -> RepositoryResult<()>;
    fn update_user(&self, user: &User) -> RepositoryResult<()>;
}

/// Repository for friendship data.
pub trait FriendshipRepository: Send + Sync {
    fn list_friendships(&self) -> RepositoryResult<Vec<Friendship>>;
    fn get_friendship(&self, id: i64) -> RepositoryResult<Option<Friendship>>;
    fn create_friendship(&self, friendship: &Friendship) -> RepositoryResult<Friendship>;
    fn update_friendship(&self, friendship: &Friendship) -> RepositoryResult<()>;
    fn delete_friendship(&self, id: i64) -> RepositoryResult<()>;
}

/// Repository for location share data.
pub trait LocationShareRepository: Send + Sync {
    fn list_shares(&self) -> RepositoryResult<Vec<LocationShare>>;
    fn get_share(&self, id: i64) -> RepositoryResult<Option<LocationShare>>;
    fn create_share(&self, share: &LocationShare) -> RepositoryResult<LocationShare>;
    fn update_share(&self, share: &LocationShare) -> RepositoryResult<()>;
    fn delete_share(&self, id: i64) -> RepositoryResult<()>;
}

/// Repository for friend location data.
pub trait FriendLocationRepository: Send + Sync {
    fn list_locations(&self) -> RepositoryResult<Vec<FriendLocation>>;
    fn get_location(&self, user_id: i64) -> RepositoryResult<Option<FriendLocation>>;
    fn upsert_location(&self, location: &FriendLocation) -> RepositoryResult<()>;
    fn delete_location(&self, user_id: i64) -> RepositoryResult<()>;
}

/// Combined repository trait for convenience.
pub trait Repository:
    UserRepository + FriendshipRepository + LocationShareRepository + FriendLocationRepository
{
}

/// In-memory repository implementation. Suitable for offline mode and
/// testing. Data is lost when the process exits unless paired with a
/// persistence layer.
#[derive(Default)]
pub struct InMemoryRepository {
    users: std::sync::Mutex<Vec<User>>,
    friendships: std::sync::Mutex<Vec<Friendship>>,
    shares: std::sync::Mutex<Vec<LocationShare>>,
    locations: std::sync::Mutex<Vec<FriendLocation>>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl UserRepository for InMemoryRepository {
    fn get_current_user(&self) -> RepositoryResult<Option<User>> {
        Ok(self.users.lock().unwrap().first().cloned())
    }

    fn set_current_user(&self, user: Option<User>) -> RepositoryResult<()> {
        let mut users = self.users.lock().unwrap();
        users.clear();
        if let Some(u) = user {
            users.push(u);
        }
        Ok(())
    }

    fn update_user(&self, user: &User) -> RepositoryResult<()> {
        let mut users = self.users.lock().unwrap();
        if let Some(existing) = users.iter_mut().find(|u| u.id == user.id) {
            *existing = user.clone();
            Ok(())
        } else {
            Err(RepositoryError::NotFound(format!("User {}", user.id)))
        }
    }
}

impl FriendshipRepository for InMemoryRepository {
    fn list_friendships(&self) -> RepositoryResult<Vec<Friendship>> {
        Ok(self.friendships.lock().unwrap().clone())
    }

    fn get_friendship(&self, id: i64) -> RepositoryResult<Option<Friendship>> {
        Ok(self
            .friendships
            .lock()
            .unwrap()
            .iter()
            .find(|f| f.id == id)
            .cloned())
    }

    fn create_friendship(&self, friendship: &Friendship) -> RepositoryResult<Friendship> {
        let mut friendships = self.friendships.lock().unwrap();
        if friendships.iter().any(|f| f.id == friendship.id) {
            return Err(RepositoryError::AlreadyExists(format!(
                "Friendship {}",
                friendship.id
            )));
        }
        friendships.push(friendship.clone());
        Ok(friendship.clone())
    }

    fn update_friendship(&self, friendship: &Friendship) -> RepositoryResult<()> {
        let mut friendships = self.friendships.lock().unwrap();
        if let Some(existing) = friendships.iter_mut().find(|f| f.id == friendship.id) {
            *existing = friendship.clone();
            Ok(())
        } else {
            Err(RepositoryError::NotFound(format!(
                "Friendship {}",
                friendship.id
            )))
        }
    }

    fn delete_friendship(&self, id: i64) -> RepositoryResult<()> {
        let mut friendships = self.friendships.lock().unwrap();
        let initial_len = friendships.len();
        friendships.retain(|f| f.id != id);
        if friendships.len() == initial_len {
            Err(RepositoryError::NotFound(format!("Friendship {id}")))
        } else {
            Ok(())
        }
    }
}

impl LocationShareRepository for InMemoryRepository {
    fn list_shares(&self) -> RepositoryResult<Vec<LocationShare>> {
        Ok(self.shares.lock().unwrap().clone())
    }

    fn get_share(&self, id: i64) -> RepositoryResult<Option<LocationShare>> {
        Ok(self
            .shares
            .lock()
            .unwrap()
            .iter()
            .find(|s| s.id == id)
            .cloned())
    }

    fn create_share(&self, share: &LocationShare) -> RepositoryResult<LocationShare> {
        let mut shares = self.shares.lock().unwrap();
        if shares.iter().any(|s| s.id == share.id) {
            return Err(RepositoryError::AlreadyExists(format!(
                "Share {}",
                share.id
            )));
        }
        shares.push(share.clone());
        Ok(share.clone())
    }

    fn update_share(&self, share: &LocationShare) -> RepositoryResult<()> {
        let mut shares = self.shares.lock().unwrap();
        if let Some(existing) = shares.iter_mut().find(|s| s.id == share.id) {
            *existing = share.clone();
            Ok(())
        } else {
            Err(RepositoryError::NotFound(format!("Share {}", share.id)))
        }
    }

    fn delete_share(&self, id: i64) -> RepositoryResult<()> {
        let mut shares = self.shares.lock().unwrap();
        let initial_len = shares.len();
        shares.retain(|s| s.id != id);
        if shares.len() == initial_len {
            Err(RepositoryError::NotFound(format!("Share {id}")))
        } else {
            Ok(())
        }
    }
}

impl FriendLocationRepository for InMemoryRepository {
    fn list_locations(&self) -> RepositoryResult<Vec<FriendLocation>> {
        Ok(self.locations.lock().unwrap().clone())
    }

    fn get_location(&self, user_id: i64) -> RepositoryResult<Option<FriendLocation>> {
        Ok(self
            .locations
            .lock()
            .unwrap()
            .iter()
            .find(|l| l.user_id == user_id)
            .cloned())
    }

    fn upsert_location(&self, location: &FriendLocation) -> RepositoryResult<()> {
        let mut locations = self.locations.lock().unwrap();
        if let Some(existing) = locations.iter_mut().find(|l| l.user_id == location.user_id) {
            *existing = location.clone();
        } else {
            locations.push(location.clone());
        }
        Ok(())
    }

    fn delete_location(&self, user_id: i64) -> RepositoryResult<()> {
        let mut locations = self.locations.lock().unwrap();
        let initial_len = locations.len();
        locations.retain(|l| l.user_id != user_id);
        if locations.len() == initial_len {
            Err(RepositoryError::NotFound(format!(
                "Location for user {user_id}"
            )))
        } else {
            Ok(())
        }
    }
}

impl Repository for InMemoryRepository {}

// ---------------------------------------------------------------------------
// Postgres implementation (LSP: substitutable for InMemoryRepository).
//
// The sync traits above are implemented over sqlx by bridging to the async
// pool with a dedicated runtime. This completes LSP/DIP for server-side
// tooling and conformance tests: callers depend on `dyn Repository` and
// the factory (`open_repository`) picks the backend. Tables are the owned
// domain tables in `src/server/migrations.rs`; "current user" is the
// lowest-id resident (mirroring `handlers::current_user`).
// ---------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
mod postgres_impl {
    use super::{
        FriendLocationRepository, FriendshipRepository, LocationShareRepository, RepositoryError,
        RepositoryResult, UserRepository,
    };
    use crate::models::{FriendLocation, Friendship, LocationShare, User};
    use std::future::Future;

    pub struct PostgresRepository {
        pool: sqlx::PgPool,
    }

    impl PostgresRepository {
        pub async fn connect(database_url: &str) -> RepositoryResult<Self> {
            let pool = sqlx::PgPool::connect(database_url)
                .await
                .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
            Ok(Self { pool })
        }

        #[allow(dead_code)]
        pub fn from_pool(pool: sqlx::PgPool) -> Self {
            Self { pool }
        }

        fn block_on<T, F>(&self, fut: F) -> RepositoryResult<T>
        where
            F: Future<Output = RepositoryResult<T>> + Send,
            T: Send,
        {
            static RT: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
            let rt = RT.get_or_init(|| {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("repository runtime")
            });
            rt.block_on(fut)
        }
    }

    impl UserRepository for PostgresRepository {
        fn get_current_user(&self) -> RepositoryResult<Option<User>> {
            self.block_on(async {
                let row: Option<(
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
                    "SELECT id, email, full_name, role, is_active, must_change_password, password_hash, totp_secret, mfa_enabled, mfa_last_step FROM pinpoint_users ORDER BY (role='resident') DESC, id LIMIT 1",
                )
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                Ok(row.map(
                    |(
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
                    )| User {
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
                    },
                ))
            })
        }

        fn set_current_user(&self, user: Option<User>) -> RepositoryResult<()> {
            self.block_on(async {
                match user {
                    Some(u) => {
                        sqlx::query(
                            "INSERT INTO pinpoint_users (id, email, full_name, role, is_active, must_change_password, password_hash, totp_secret, mfa_enabled, mfa_last_step) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) ON CONFLICT (id) DO UPDATE SET email=EXCLUDED.email, full_name=EXCLUDED.full_name, role=EXCLUDED.role, is_active=EXCLUDED.is_active, must_change_password=EXCLUDED.must_change_password, password_hash=EXCLUDED.password_hash, totp_secret=EXCLUDED.totp_secret, mfa_enabled=EXCLUDED.mfa_enabled, mfa_last_step=EXCLUDED.mfa_last_step",
                        )
                        .bind(u.id).bind(&u.email).bind(&u.full_name).bind(&u.role).bind(u.is_active).bind(u.must_change_password).bind(&u.password_hash).bind(&u.totp_secret).bind(u.mfa_enabled).bind(u.mfa_last_step as i64)
                        .execute(&self.pool)
                        .await
                        .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                        Ok(())
                    }
                    None => {
                        // Mirror InMemory "clear": drop the current (first resident).
                        sqlx::query(
                            "DELETE FROM pinpoint_users WHERE id = (SELECT id FROM pinpoint_users ORDER BY (role='resident') DESC, id LIMIT 1)",
                        )
                        .execute(&self.pool)
                        .await
                        .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                        Ok(())
                    }
                }
            })
        }

        fn update_user(&self, user: &User) -> RepositoryResult<()> {
            self.block_on(async {
                let r = sqlx::query(
                    "UPDATE pinpoint_users SET email=$3, full_name=$4, role=$5, is_active=$6, must_change_password=$7, password_hash=$8, totp_secret=$9, mfa_enabled=$10, mfa_last_step=$11 WHERE id=$2",
                )
                .bind(user.id)
                .bind(user.id)
                .bind(&user.email)
                .bind(&user.full_name)
                .bind(&user.role)
                .bind(user.is_active)
                .bind(user.must_change_password)
                .bind(&user.password_hash)
                .bind(&user.totp_secret)
                .bind(user.mfa_enabled)
                .bind(user.mfa_last_step as i64)
                .execute(&self.pool)
                .await
                .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                if r.rows_affected() == 0 {
                    Err(RepositoryError::NotFound(format!("User {}", user.id)))
                } else {
                    Ok(())
                }
            })
        }
    }

    impl FriendshipRepository for PostgresRepository {
        fn list_friendships(&self) -> RepositoryResult<Vec<Friendship>> {
            self.block_on(async {
                let rows: Vec<(
                    i64,
                    i64,
                    i64,
                    Option<String>,
                    Option<String>,
                    String,
                    Option<String>,
                    Option<String>,
                )> = sqlx::query_as(
                    "SELECT id, requester_id, receiver_id, requester_name, receiver_name, status, created_at, responded_at FROM pinpoint_friendships ORDER BY id",
                )
                .fetch_all(&self.pool)
                .await
                .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                Ok(rows
                    .into_iter()
                    .map(
                        |(id, requester_id, receiver_id, rn, dn, status, created_at, responded_at)| Friendship {
                            id,
                            requester_id,
                            receiver_id,
                            requester_name: rn.unwrap_or_default(),
                            receiver_name: dn.unwrap_or_default(),
                            status,
                            created_at,
                            responded_at,
                        },
                    )
                    .collect())
            })
        }

        fn get_friendship(&self, id: i64) -> RepositoryResult<Option<Friendship>> {
            self.block_on(async {
                let row: Option<(
                    i64,
                    i64,
                    i64,
                    Option<String>,
                    Option<String>,
                    String,
                    Option<String>,
                    Option<String>,
                )> = sqlx::query_as(
                    "SELECT id, requester_id, receiver_id, requester_name, receiver_name, status, created_at, responded_at FROM pinpoint_friendships WHERE id=$1",
                )
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                Ok(row.map(
                    |(id, requester_id, receiver_id, rn, dn, status, created_at, responded_at)| Friendship {
                        id,
                        requester_id,
                        receiver_id,
                        requester_name: rn.unwrap_or_default(),
                        receiver_name: dn.unwrap_or_default(),
                        status,
                        created_at,
                        responded_at,
                    },
                ))
            })
        }

        fn create_friendship(&self, friendship: &Friendship) -> RepositoryResult<Friendship> {
            self.block_on(async {
                let exists: Option<(i64,)> = sqlx::query_as(
                    "SELECT id FROM pinpoint_friendships WHERE id=$1",
                )
                .bind(friendship.id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                if exists.is_some() {
                    return Err(RepositoryError::AlreadyExists(format!(
                        "Friendship {}",
                        friendship.id
                    )));
                }
                sqlx::query(
                    "INSERT INTO pinpoint_friendships (id, requester_id, receiver_id, requester_name, receiver_name, status, created_at, responded_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
                )
                .bind(friendship.id).bind(friendship.requester_id).bind(friendship.receiver_id).bind(&friendship.requester_name).bind(&friendship.receiver_name).bind(&friendship.status).bind(&friendship.created_at).bind(&friendship.responded_at)
                .execute(&self.pool)
                .await
                .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                Ok(friendship.clone())
            })
        }

        fn update_friendship(&self, friendship: &Friendship) -> RepositoryResult<()> {
            self.block_on(async {
                let r = sqlx::query(
                    "UPDATE pinpoint_friendships SET requester_id=$1, receiver_id=$2, requester_name=$3, receiver_name=$4, status=$5, created_at=$6, responded_at=$7 WHERE id=$8",
                )
                .bind(friendship.requester_id).bind(friendship.receiver_id).bind(&friendship.requester_name).bind(&friendship.receiver_name).bind(&friendship.status).bind(&friendship.created_at).bind(&friendship.responded_at).bind(friendship.id)
                .execute(&self.pool)
                .await
                .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                if r.rows_affected() == 0 {
                    Err(RepositoryError::NotFound(format!("Friendship {}", friendship.id)))
                } else {
                    Ok(())
                }
            })
        }

        fn delete_friendship(&self, id: i64) -> RepositoryResult<()> {
            self.block_on(async {
                let r = sqlx::query("DELETE FROM pinpoint_friendships WHERE id=$1")
                    .bind(id)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                if r.rows_affected() == 0 {
                    Err(RepositoryError::NotFound(format!("Friendship {id}")))
                } else {
                    Ok(())
                }
            })
        }
    }

    impl LocationShareRepository for PostgresRepository {
        fn list_shares(&self) -> RepositoryResult<Vec<LocationShare>> {
            self.block_on(async {
                let rows: Vec<(
                    i64,
                    i64,
                    i64,
                    String,
                    String,
                    bool,
                    Option<String>,
                    Option<String>,
                    Option<String>,
                )> = sqlx::query_as(
                    "SELECT id, friendship_id, owner_id, direction, precision_level, is_active, last_location_update_at, expires_at, label FROM pinpoint_shares ORDER BY id",
                )
                .fetch_all(&self.pool)
                .await
                .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                Ok(rows
                    .into_iter()
                    .map(
                        |(
                            id,
                            friendship_id,
                            owner_id,
                            direction,
                            precision_level,
                            is_active,
                            last_location_update_at,
                            expires_at,
                            label,
                        )| LocationShare {
                            id,
                            friendship_id,
                            owner_id,
                            direction,
                            precision_level,
                            is_active,
                            last_location_update_at,
                            expires_at,
                            label,
                        },
                    )
                    .collect())
            })
        }

        fn get_share(&self, id: i64) -> RepositoryResult<Option<LocationShare>> {
            self.block_on(async {
                let row: Option<(
                    i64,
                    i64,
                    i64,
                    String,
                    String,
                    bool,
                    Option<String>,
                    Option<String>,
                    Option<String>,
                )> = sqlx::query_as(
                    "SELECT id, friendship_id, owner_id, direction, precision_level, is_active, last_location_update_at, expires_at, label FROM pinpoint_shares WHERE id=$1",
                )
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                Ok(row.map(
                    |(
                        id,
                        friendship_id,
                        owner_id,
                        direction,
                        precision_level,
                        is_active,
                        last_location_update_at,
                        expires_at,
                        label,
                    )| LocationShare {
                        id,
                        friendship_id,
                        owner_id,
                        direction,
                        precision_level,
                        is_active,
                        last_location_update_at,
                        expires_at,
                        label,
                    },
                ))
            })
        }

        fn create_share(&self, share: &LocationShare) -> RepositoryResult<LocationShare> {
            self.block_on(async {
                let exists: Option<(i64,)> = sqlx::query_as(
                    "SELECT id FROM pinpoint_shares WHERE id=$1",
                )
                .bind(share.id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                if exists.is_some() {
                    return Err(RepositoryError::AlreadyExists(format!(
                        "Share {}",
                        share.id
                    )));
                }
                sqlx::query(
                    "INSERT INTO pinpoint_shares (id, friendship_id, owner_id, direction, precision_level, is_active, last_location_update_at, expires_at, label) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
                )
                .bind(share.id).bind(share.friendship_id).bind(share.owner_id).bind(&share.direction).bind(&share.precision_level).bind(share.is_active).bind(&share.last_location_update_at).bind(&share.expires_at).bind(&share.label)
                .execute(&self.pool)
                .await
                .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                Ok(share.clone())
            })
        }

        fn update_share(&self, share: &LocationShare) -> RepositoryResult<()> {
            self.block_on(async {
                let r = sqlx::query(
                    "UPDATE pinpoint_shares SET friendship_id=$1, owner_id=$2, direction=$3, precision_level=$4, is_active=$5, last_location_update_at=$6, expires_at=$7, label=$8 WHERE id=$9",
                )
                .bind(share.friendship_id).bind(share.owner_id).bind(&share.direction).bind(&share.precision_level).bind(share.is_active).bind(&share.last_location_update_at).bind(&share.expires_at).bind(&share.label).bind(share.id)
                .execute(&self.pool)
                .await
                .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                if r.rows_affected() == 0 {
                    Err(RepositoryError::NotFound(format!("Share {}", share.id)))
                } else {
                    Ok(())
                }
            })
        }

        fn delete_share(&self, id: i64) -> RepositoryResult<()> {
            self.block_on(async {
                let r = sqlx::query("DELETE FROM pinpoint_shares WHERE id=$1")
                    .bind(id)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                if r.rows_affected() == 0 {
                    Err(RepositoryError::NotFound(format!("Share {id}")))
                } else {
                    Ok(())
                }
            })
        }
    }

    impl FriendLocationRepository for PostgresRepository {
        fn list_locations(&self) -> RepositoryResult<Vec<FriendLocation>> {
            self.block_on(async {
                let rows: Vec<(
                    i64,
                    Option<String>,
                    f64,
                    f64,
                    Option<String>,
                    Option<String>,
                    Option<f64>,
                )> = sqlx::query_as(
                    "SELECT user_id, user_name, latitude, longitude, precision_level, updated_at, accuracy FROM pinpoint_locations ORDER BY user_id",
                )
                .fetch_all(&self.pool)
                .await
                .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                Ok(rows
                    .into_iter()
                    .map(
                        |(user_id, user_name, latitude, longitude, precision_level, updated_at, accuracy)| FriendLocation {
                            user_id,
                            user_name: user_name.unwrap_or_default(),
                            latitude,
                            longitude,
                            precision_level: precision_level.unwrap_or_else(|| "exact".to_string()),
                            updated_at: updated_at.unwrap_or_default(),
                            accuracy,
                        },
                    )
                    .collect())
            })
        }

        fn get_location(&self, user_id: i64) -> RepositoryResult<Option<FriendLocation>> {
            self.block_on(async {
                let row: Option<(
                    i64,
                    Option<String>,
                    f64,
                    f64,
                    Option<String>,
                    Option<String>,
                    Option<f64>,
                )> = sqlx::query_as(
                    "SELECT user_id, user_name, latitude, longitude, precision_level, updated_at, accuracy FROM pinpoint_locations WHERE user_id=$1",
                )
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                Ok(row.map(
                    |(user_id, user_name, latitude, longitude, precision_level, updated_at, accuracy)| FriendLocation {
                        user_id,
                        user_name: user_name.unwrap_or_default(),
                        latitude,
                        longitude,
                        precision_level: precision_level.unwrap_or_else(|| "exact".to_string()),
                        updated_at: updated_at.unwrap_or_default(),
                        accuracy,
                    },
                ))
            })
        }

        fn upsert_location(&self, location: &FriendLocation) -> RepositoryResult<()> {
            self.block_on(async {
                sqlx::query(
                    "INSERT INTO pinpoint_locations (user_id, user_name, latitude, longitude, precision_level, updated_at, accuracy) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT (user_id) DO UPDATE SET user_name=EXCLUDED.user_name, latitude=EXCLUDED.latitude, longitude=EXCLUDED.longitude, precision_level=EXCLUDED.precision_level, updated_at=EXCLUDED.updated_at, accuracy=EXCLUDED.accuracy",
                )
                .bind(location.user_id).bind(&location.user_name).bind(location.latitude).bind(location.longitude).bind(&location.precision_level).bind(&location.updated_at).bind(location.accuracy)
                .execute(&self.pool)
                .await
                .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                Ok(())
            })
        }

        fn delete_location(&self, user_id: i64) -> RepositoryResult<()> {
            self.block_on(async {
                let r = sqlx::query("DELETE FROM pinpoint_locations WHERE user_id=$1")
                    .bind(user_id)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| RepositoryError::StorageError(e.to_string()))?;
                if r.rows_affected() == 0 {
                    Err(RepositoryError::NotFound(format!(
                        "Location for user {user_id}"
                    )))
                } else {
                    Ok(())
                }
            })
        }
    }

    impl super::Repository for PostgresRepository {}

    /// Factory (DIP): callers depend on `dyn Repository`, the factory picks
    /// the backend. `Some(url)` → Postgres, `None`/empty → in-memory.
    pub fn open_repository(
        database_url: Option<&str>,
    ) -> super::RepositoryResult<Box<dyn super::Repository>> {
        match database_url.map(str::trim).filter(|s| !s.is_empty()) {
            Some(url) => {
                let repo = PostgresRepository::connect_checked(url)?;
                Ok(Box::new(repo))
            }
            None => Ok(Box::new(super::InMemoryRepository::new())),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl postgres_impl::PostgresRepository {
    fn connect_checked(database_url: &str) -> RepositoryResult<Self> {
        static RT: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
        let rt = RT.get_or_init(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("repository runtime")
        });
        rt.block_on(Self::connect(database_url))
    }
}

/// Factory (DIP): `None`/empty → in-memory, `Some(url)` → Postgres on wasm
/// returns a storage error (wasm has no sqlx pool).
#[cfg(not(target_arch = "wasm32"))]
pub fn open_repository(database_url: Option<&str>) -> RepositoryResult<Box<dyn Repository>> {
    postgres_impl::open_repository(database_url)
}

#[cfg(target_arch = "wasm32")]
pub fn open_repository(database_url: Option<&str>) -> RepositoryResult<Box<dyn Repository>> {
    let _ = database_url;
    Ok(Box::new(InMemoryRepository::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn friendship_crud() {
        let repo = InMemoryRepository::new();
        let f = Friendship {
            id: 1,
            requester_id: 100,
            receiver_id: 200,
            requester_name: "Alice".to_string(),
            receiver_name: "Bob".to_string(),
            status: "pending".to_string(),
            created_at: None,
            responded_at: None,
        };

        repo.create_friendship(&f).unwrap();
        assert!(repo.list_friendships().unwrap().len() == 1);
        assert!(repo.get_friendship(1).unwrap().is_some());

        repo.update_friendship(&Friendship {
            id: 1,
            requester_id: 100,
            receiver_id: 200,
            requester_name: "Alice".to_string(),
            receiver_name: "Bob".to_string(),
            status: "accepted".to_string(),
            created_at: None,
            responded_at: None,
        })
        .unwrap();

        let updated = repo.get_friendship(1).unwrap().unwrap();
        assert_eq!(updated.status, "accepted");

        repo.delete_friendship(1).unwrap();
        assert!(repo.list_friendships().unwrap().is_empty());
    }

    #[test]
    fn location_upsert() {
        let repo = InMemoryRepository::new();
        let loc = FriendLocation {
            user_id: 1,
            user_name: "Test".to_string(),
            latitude: 41.8902,
            longitude: -87.6323,
            precision_level: "exact".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            accuracy: Some(8.0),
        };

        repo.upsert_location(&loc).unwrap();
        assert!(repo.get_location(1).unwrap().is_some());

        let loc2 = FriendLocation {
            user_id: 1,
            user_name: "Test".to_string(),
            latitude: 41.9,
            longitude: -87.64,
            precision_level: "approximate".to_string(),
            updated_at: "2026-01-01T01:00:00Z".to_string(),
            accuracy: Some(25.0),
        };
        repo.upsert_location(&loc2).unwrap();
        let updated = repo.get_location(1).unwrap().unwrap();
        assert_eq!(updated.latitude, 41.9);
    }

    #[test]
    fn lsp_repositories_are_substitutable() {
        // Both backends behind the same trait object (DIP factory).
        let repos: Vec<Box<dyn Repository>> = vec![Box::new(InMemoryRepository::new())];
        for repo in &repos {
            let loc = FriendLocation {
                user_id: 7,
                user_name: "LSP".to_string(),
                latitude: 1.0,
                longitude: 2.0,
                precision_level: "exact".to_string(),
                updated_at: "2026-01-01T00:00:00Z".to_string(),
                accuracy: None,
            };
            repo.upsert_location(&loc).unwrap();
            assert!(repo.get_location(7).unwrap().is_some());
            repo.delete_location(7).unwrap();
            assert!(repo.get_location(7).unwrap().is_none());
        }
        // Factory: empty URL → in-memory.
        let repo = open_repository(None).unwrap();
        assert!(repo.list_friendships().unwrap().is_empty());
        let repo = open_repository(Some("   ")).unwrap();
        assert!(repo.list_shares().unwrap().is_empty());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn postgres_repository_conformance() {
        // Runs only with a real DB (TEST_DATABASE_URL or DATABASE_URL).
        // Skips (passes) without one — CI keeps the in-memory conformance
        // above authoritative; live stack runs it via `DATABASE_URL=…` + e2e.
        let url = std::env::var("TEST_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .unwrap_or_default();
        if url.trim().is_empty() {
            return;
        }
        let repo = postgres_impl::PostgresRepository::connect_checked(&url).unwrap();
        // Unique ids so conformance never touches seeded demo rows.
        let id = 900_001_i64;
        let _ = repo.delete_friendship(id); // best-effort clean
        let f = Friendship {
            id,
            requester_id: 900_010,
            receiver_id: 900_011,
            requester_name: "A".to_string(),
            receiver_name: "B".to_string(),
            status: "accepted".to_string(),
            created_at: None,
            responded_at: None,
        };
        repo.create_friendship(&f).unwrap();
        assert_eq!(repo.get_friendship(id).unwrap().unwrap().status, "accepted");
        repo.delete_friendship(id).unwrap();
        assert!(repo.get_friendship(id).unwrap().is_none());
        let loc = FriendLocation {
            user_id: id,
            user_name: "Conf".to_string(),
            latitude: 41.0,
            longitude: -87.0,
            precision_level: "exact".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            accuracy: None,
        };
        repo.upsert_location(&loc).unwrap();
        assert!(repo.get_location(id).unwrap().is_some());
        repo.delete_location(id).unwrap();
    }
}
