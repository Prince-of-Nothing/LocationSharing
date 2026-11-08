//! Repository Pattern — Abstract data access behind trait-based interfaces.
//!
//! Repositories provide a collection-like interface for accessing domain
//! objects, hiding the details of persistence (API calls, local storage,
//! in-memory cache) from the rest of the application.
//!
//! All repositories are trait-based so they can be swapped for mock
//! implementations in tests or for different backends (online/offline).

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
}
