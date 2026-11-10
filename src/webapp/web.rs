//! General web client (`web`) app logic (modular, backend-adaptable).

use crate::webapp::config::WebAppConfig;
use crate::webapp::view_model::SessionView;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WebState {
    pub session: SessionView,
    pub map_ready: bool,
}

#[derive(Debug, Clone)]
pub struct WebApp {
    pub config: WebAppConfig,
    pub state: WebState,
}

impl WebApp {
    pub fn new(config: WebAppConfig) -> Self {
        Self {
            config,
            state: WebState::default(),
        }
    }

    pub fn snapshot_json(&self) -> String {
        serde_json::json!({
            "app": crate::webapp::WebAppKind::Web.slug(),
            "backend_url": self.config.backend.base_url,
            "backend_kind": format!("{:?}", self.config.backend.kind),
            "signed_in": self.state.session.user.is_some(),
            "loading": self.state.session.loading,
            "map_ready": self.state.map_ready,
            "notice": self.state.session.notice,
            "error": self.state.session.error,
        })
        .to_string()
    }
}
