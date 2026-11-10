//! `weba` — Admin console webapp logic (modular, backend-adaptable).
//!
//! Admin operations use the shared [`crate::backend::Backend`] surface (user
//! listing/search/update/delete, plus future admin endpoints). Role enforcement
//! is server-side; the UI gates on the authenticated user's role and handles
//! denied responses fail-closed.

use crate::backend::DynBackend;
use crate::models::User;
use crate::webapp::config::WebAppConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdminState {
    pub users: Vec<User>,
    pub query: String,
    pub selected_user_id: Option<i64>,
    pub notice: Option<String>,
    pub error: Option<String>,
    pub loading: bool,
}

impl AdminState {
    pub fn clear_messages(&mut self) {
        self.notice = None;
        self.error = None;
    }
}

/// High-level admin app controller (UI-framework agnostic).
#[derive(Debug, Clone)]
pub struct AdminApp {
    pub config: WebAppConfig,
    pub signed_in: Option<User>,
    pub state: AdminState,
}

impl AdminApp {
    pub fn new(config: WebAppConfig) -> Self {
        Self {
            config,
            signed_in: None,
            state: AdminState::default(),
        }
    }

    pub fn can_access(&self) -> bool {
        is_admin(self.signed_in.as_ref())
    }

    pub fn apply_login(&mut self, user: User) {
        self.signed_in = Some(user);
        if !self.can_access() {
            self.state.error = Some(
                "Access denied: admin role required (server-side authorization is authoritative)."
                    .into(),
            );
        } else {
            self.state.notice = Some("Signed in as admin.".into());
            self.state.error = None;
        }
    }

    pub fn snapshot_json(&self) -> String {
        serde_json::json!({
            "app": crate::webapp::WebAppKind::Admin.slug(),
            "backend_url": self.config.backend.base_url,
            "backend_kind": format!("{:?}", self.config.backend.kind),
            "signed_in": self.signed_in.is_some(),
            "can_access": self.can_access(),
            "user_count": self.state.users.len(),
            "loading": self.state.loading,
            "notice": self.state.notice,
            "error": self.state.error,
        })
        .to_string()
    }
}

/// Returns true only when the signed-in user has the admin role.
/// This is a UI convenience; the backend remains authoritative.
pub fn is_admin(user: Option<&User>) -> bool {
    matches!(user, Some(u) if u.role.eq_ignore_ascii_case("admin") && u.is_active)
}

pub async fn load_users(backend: &DynBackend, state: &mut AdminState) -> anyhow::Result<()> {
    state.loading = true;
    state.clear_messages();
    let query =
        crate::webapp::validation::validate_query(&state.query).map_err(|e| anyhow::anyhow!(e))?;
    state.query = query.clone();
    let res = if query.is_empty() {
        backend.list_users().await
    } else {
        backend.search_users(query.trim()).await
    };
    state.loading = false;
    match res {
        Ok(users) => {
            state.users = users;
            Ok(())
        }
        Err(e) => {
            state.error = Some(e.to_string());
            Err(e)
        }
    }
}

pub async fn set_user_active(
    backend: &DynBackend,
    state: &mut AdminState,
    user_id: i64,
    is_active: bool,
) -> anyhow::Result<()> {
    state.loading = true;
    state.clear_messages();
    let res = backend
        .update_user(user_id, serde_json::json!({ "is_active": is_active }))
        .await;
    state.loading = false;
    match res {
        Ok(()) => {
            state.notice = Some(format!(
                "User {user_id} {}.",
                if is_active { "enabled" } else { "disabled" }
            ));
            Ok(())
        }
        Err(e) => {
            state.error = Some(e.to_string());
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(role: &str, active: bool) -> User {
        User {
            id: 1,
            email: "admin@example.invalid".into(),
            full_name: "Admin".into(),
            role: role.into(),
            is_active: active,
            must_change_password: false,
            password_hash: None,
            totp_secret: None,
            mfa_enabled: false,
            mfa_last_step: 0,
        }
    }

    #[test]
    fn admin_gate_is_fail_closed() {
        assert!(is_admin(Some(&user("admin", true))));
        assert!(!is_admin(Some(&user("resident", true))));
        assert!(!is_admin(Some(&user("admin", false))));
        assert!(!is_admin(None));
    }

    #[test]
    fn admin_app_denies_non_admin() {
        let cfg = WebAppConfig::new(crate::webapp::WebAppKind::Admin);
        let mut app = AdminApp::new(cfg);
        app.apply_login(user("resident", true));
        assert!(!app.can_access());
        assert!(app.state.error.is_some());
    }
}
