//! Portable view-model helpers shared by web variants.
//!
//! These types are UI-framework-agnostic so they can be used by egui (web/native)
//! shells or thin host renderers while keeping business-facing state separate
//! from transport details.

use crate::backend::DynBackend;
use crate::models::{FriendLocation, User};
use crate::webapp::WebAppKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionView {
    pub user: Option<User>,
    pub notice: Option<String>,
    pub error: Option<String>,
    pub loading: bool,
}

#[derive(Debug, Clone, Default)]
pub struct MapView {
    pub center_lat: f64,
    pub center_lon: f64,
    pub friend_locations: Vec<FriendLocation>,
}

/// Minimal portable app shell state for web clients.
pub struct WebAppShell {
    pub kind: WebAppKind,
    pub backend: DynBackend,
    pub session: SessionView,
    pub map: MapView,
}

impl WebAppShell {
    pub fn new(kind: WebAppKind, backend: DynBackend) -> Self {
        Self {
            kind,
            backend,
            session: SessionView::default(),
            map: MapView {
                center_lat: 0.0,
                center_lon: 0.0,
                friend_locations: Vec::new(),
            },
        }
    }

    pub fn backend_label(&self) -> String {
        self.backend.backend_url().to_string()
    }
}
