//! Store: the shared in-memory aggregate (demo/local; Postgres-backed in prod).
//!
//! Split into per-domain sub-stores (`users`, `friendships`, `locations`)
//! so each service's data lives with its owner. `Store` is the facade the
//! handlers lock; domain modules (`users.rs`, `friendships.rs`,
//! `locations.rs`) hold the per-table load/persist logic. Serde shape is
//! unchanged (flat `users`/`friendships`/… keys) so the JSONB rollback
//! snapshot in `pinpoint_state` stays compatible.

use serde::{Deserialize, Serialize};

use crate::models::{
    AdminAuditEntry, CheckIn, DiagnosticReport, EmergencyAlert, FriendLocation, Friendship,
    LocationShare, User,
};

/// Per-domain slices of the shared aggregate. `Store` keeps them as
/// separate fields (not one blob) so handlers and persistence already
/// work domain-by-domain; `mod.rs` orchestrates across them.
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
    pub diagnostic_reports: Vec<DiagnosticReport>,
    #[serde(default)]
    pub admin_audit: Vec<AdminAuditEntry>,
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
    #[serde(default)]
    pub next_report_id: i64,
    #[serde(default)]
    pub next_audit_id: i64,
}

impl Store {
    pub fn new() -> Self {
        let mut s = Self {
            next_user_id: 1,
            next_friendship_id: 1,
            next_share_id: 1,
            next_check_in_id: 1,
            next_emergency_id: 1,
            next_report_id: 1,
            next_audit_id: 1,
            ..Default::default()
        };
        s.seed_demo();
        s
    }

    /// Owned-field copy for the persist tick (lock released before I/O).
    pub(super) fn snapshot(&self) -> Self {
        Self {
            users: self.users.clone(),
            friendships: self.friendships.clone(),
            shares: self.shares.clone(),
            locations: self.locations.clone(),
            check_ins: self.check_ins.clone(),
            emergencies: self.emergencies.clone(),
            diagnostic_reports: self.diagnostic_reports.clone(),
            admin_audit: self.admin_audit.clone(),
            next_user_id: self.next_user_id,
            next_friendship_id: self.next_friendship_id,
            next_share_id: self.next_share_id,
            next_check_in_id: self.next_check_in_id,
            next_emergency_id: self.next_emergency_id,
            next_report_id: self.next_report_id,
            next_audit_id: self.next_audit_id,
        }
    }

    /// Seed demo accounts (idempotent): alice, bob, admin.
    pub fn seed_demo(&mut self) {
        use crate::security::password::hash_password;
        if !self.users.is_empty() {
            return;
        }
        let admin = User {
            id: self.next_user_id,
            username: String::new(),
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
            failed_login_attempts: 0,
            locked_until: None,
            auth_method: "email".into(),
            public_key: None,
            recovery_code_hash: None,
            identity_subject: None,
        };
        self.next_user_id += 1;
        let alice = User {
            id: self.next_user_id,
            username: String::new(),
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
            failed_login_attempts: 0,
            locked_until: None,
            auth_method: "email".into(),
            public_key: None,
            recovery_code_hash: None,
            identity_subject: None,
        };
        self.next_user_id += 1;
        let bob = User {
            id: self.next_user_id,
            username: String::new(),
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
            failed_login_attempts: 0,
            locked_until: None,
            auth_method: "email".into(),
            public_key: None,
            recovery_code_hash: None,
            identity_subject: None,
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
        // Demo locations (Chișinău, Moldova)
        self.locations = vec![
            FriendLocation {
                user_id: self.users[1].id,
                user_name: "Alice Example".into(),
                latitude: 47.0105,
                longitude: 28.8638,
                precision_level: "exact".into(),
                updated_at: "2026-11-08T12:00:00Z".into(),
                accuracy: Some(8.0),
            },
            FriendLocation {
                user_id: self.users[2].id,
                user_name: "Bob Example".into(),
                latitude: 47.0056,
                longitude: 28.8575,
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
