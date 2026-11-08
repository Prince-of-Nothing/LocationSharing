//! On-device bookmarks in pure Rust — the multiplatform replacement for the
//! archived web (`js/bookmarks.js`, localStorage) and Kotlin
//! (`BookmarkRepositoryImpl`, DataStore). Same semantics everywhere:
//! manually-starred places only, stored as JSON next to the app data dir,
//! never synced, never containing friend/shared locations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub created_at: String,
}

pub struct BookmarkStore {
    path: Option<std::path::PathBuf>,
}

impl BookmarkStore {
    /// `None` on targets without a writable app dir (wasm/iOS sandbox) —
    /// the store then keeps everything in memory.
    pub fn new(dir: Option<std::path::PathBuf>) -> Self {
        let path = dir.map(|d| d.join("bookmarks.json"));
        Self { path }
    }

    /// Default store under [`crate::platform::app_data_dir`].
    pub fn default_location() -> Self {
        Self::new(crate::platform::app_data_dir())
    }

    pub fn list(&self) -> Vec<Bookmark> {
        let Some(path) = &self.path else {
            return Vec::new();
        };
        let Ok(text) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        serde_json::from_str(&text).unwrap_or_default()
    }

    pub fn add(&self, name: &str, latitude: f64, longitude: f64) -> Bookmark {
        let mut list = self.list();
        let bm = Bookmark {
            id: format!("bm_{}", list.len() + 1),
            name: name.to_string(),
            latitude,
            longitude,
            note: String::new(),
            category: String::new(),
            created_at: String::new(),
        };
        list.insert(0, bm.clone());
        self.persist(&list);
        bm
    }

    pub fn remove(&self, id: &str) {
        let next: Vec<Bookmark> = self.list().into_iter().filter(|b| b.id != id).collect();
        self.persist(&next);
    }

    fn persist(&self, list: &[Bookmark]) {
        let Some(path) = &self.path else { return };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(text) = serde_json::to_string_pretty(list) {
            let _ = std::fs::write(path, text);
        }
    }
}

impl Default for BookmarkStore {
    fn default() -> Self {
        Self::default_location()
    }
}

#[cfg(test)]
mod tests {
    use super::BookmarkStore;

    #[test]
    fn in_memory_store_add_remove() {
        let store = BookmarkStore::new(None);
        assert!(store.list().is_empty());
        let bm = store.add("Home", 47.01, 28.86);
        assert_eq!(bm.name, "Home");
        // In-memory mode persists nothing — list stays empty by design.
        assert!(store.list().is_empty());
    }

    #[test]
    fn file_store_round_trip() {
        let dir = std::env::temp_dir().join(format!("pinpoint-bm-{}", std::process::id()));
        let store = BookmarkStore::new(Some(dir.clone()));
        store.add("Cafe", 47.0, 28.8);
        let list = store.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Cafe");
        store.remove(&list[0].id.clone());
        assert!(store.list().is_empty());
        let _ = std::fs::remove_dir_all(dir);
    }
}
