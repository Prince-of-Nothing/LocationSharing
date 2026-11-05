//! Domain models mirroring the Go API's JSON contract
//! (`backend/internal/handlers`: `userResponse`, `loginResponse`,
//! `friendshipResponse`, `locationShareResponse`, `locationResponse`).

use serde::{Deserialize, Serialize};

/// Authenticated user as returned in the `user` field of login/register.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub is_active: bool,
    pub must_change_password: bool,
}

/// Envelope of `POST /api/v1/auth/login` and `POST /api/v1/auth/register`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
    pub csrf_token: Option<String>,
    pub user: User,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Friendship {
    pub id: i64,
    pub requester_id: i64,
    pub receiver_id: i64,
    pub requester_name: String,
    pub receiver_name: String,
    /// `pending` | `accepted` | `rejected`
    pub status: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub responded_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationShare {
    pub id: i64,
    pub friendship_id: i64,
    pub owner_id: i64,
    /// `to_friend` | `from_friend` | `mutual`
    pub direction: String,
    /// `exact` | `approximate` | `city_only`
    pub precision_level: String,
    pub is_active: bool,
    #[serde(default)]
    pub last_location_update_at: Option<String>,
}

/// A friend's location as visible to the caller, already rounded by the
/// server according to the active share's precision level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FriendLocation {
    pub user_id: i64,
    pub user_name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub precision_level: String,
    pub updated_at: String,
    #[serde(default)]
    pub accuracy: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum AppState {
    #[default]
    Auth,
    Main,
    Admin,
}

#[derive(Debug, Clone, Default)]
pub struct AppData {
    pub user: Option<User>,
    pub friendships: Vec<Friendship>,
    pub shares: Vec<LocationShare>,
    pub locations: Vec<FriendLocation>,
    pub state: AppState,
    pub must_change_password: bool,
}
