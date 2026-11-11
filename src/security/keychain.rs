//! OS keychain integration for secure secret storage.
//!
//! Provides a unified [`Keychain`] trait with platform-specific backends:
//! - Windows: Credential Manager (future)
//! - macOS: Keychain Services (future)
//! - Linux: Secret Service API (future)
//! - Fallback: [`FileKeychain`] — AES-256-GCM encrypted file in the app data
//!   directory, works on all platforms.
//!
//! Used to store E2E encryption keys, OAuth tokens, and other sensitive
//! material so users don't need to re-enter passwords every session.

use anyhow::{bail, Result};
use aead::{Aead, KeyInit};
use rand::RngCore;
use std::path::PathBuf;

/// Unified keychain trait for storing and retrieving secrets.
///
/// Implementations must be thread-safe (`Send + Sync`). All operations
/// fail closed: on any error, no secret material is exposed.
pub trait Keychain: Send + Sync {
    /// Store `value` under `key_name`. If an item already exists and
    /// `overwrite` is false, returns an error.
    fn set(&self, key_name: &str, value: &[u8], overwrite: bool) -> Result<()>;
    /// Retrieve the secret value stored under `key_name`.
    fn get(&self, key_name: &str) -> Result<Vec<u8>>;
    /// Delete the secret stored under `key_name`. No error if missing.
    fn delete(&self, key_name: &str) -> Result<()>;
    /// Return `true` if an item exists under `key_name`.
    fn contains(&self, key_name: &str) -> Result<bool>;
}

/// Cross-platform file-based keychain using AES-256-GCM encryption.
///
/// Each entry is stored as a separate file in `dir`:
/// `<dir>/<key_name>.keychain` containing nonce || ciphertext.
///
/// `master_key` is used to encrypt all stored values. It should be
/// derived from a user password or generated randomly and stored
/// in the OS keychain (see [`OsKeychain`]).
pub struct FileKeychain {
    dir: PathBuf,
    master_key: [u8; 32],
}

impl FileKeychain {
    /// Create a new file keychain. `dir` is created if missing.
    pub fn new(dir: PathBuf, master_key: [u8; 32]) -> Result<Self> {
        std::fs::create_dir_all(&dir).map_err(|e| anyhow::anyhow!("create dir: {e}"))?;
        Ok(Self { dir, master_key })
    }

    fn entry_path(&self, key_name: &str) -> PathBuf {
        self.dir.join(format!("{}.keychain", key_name))
    }

    fn encrypt_value(&self, value: &[u8]) -> Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce: hybrid_array::Array<u8, typenum::U12> = nonce_bytes.into();
        let key_arr: hybrid_array::Array<u8, typenum::U32> = self.master_key.into();
        let cipher = aes_gcm::Aes256Gcm::new(&key_arr);
        let ciphertext = cipher
            .encrypt(&nonce, value)
            .map_err(|e| anyhow::anyhow!("encrypt: {e}"))?;
        let mut out = nonce_bytes.to_vec();
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    fn decrypt_value(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        if encrypted.len() < 12 {
            bail!("ciphertext too short");
        }
        let (nonce_bytes, ciphertext) = encrypted.split_at(12);
        let mut nonce_arr = [0u8; 12];
        nonce_arr.copy_from_slice(nonce_bytes);
        let nonce: hybrid_array::Array<u8, typenum::U12> = nonce_arr.into();
        let key_arr: hybrid_array::Array<u8, typenum::U32> = self.master_key.into();
        let cipher = aes_gcm::Aes256Gcm::new(&key_arr);
        cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("decrypt: {e}"))
    }
}

impl Keychain for FileKeychain {
    fn set(&self, key_name: &str, value: &[u8], overwrite: bool) -> Result<()> {
        let path = self.entry_path(key_name);
        if path.exists() && !overwrite {
            bail!("keychain item already exists: {key_name}");
        }
        let encrypted = self.encrypt_value(value)?;
        std::fs::write(&path, encrypted).map_err(|e| anyhow::anyhow!("write: {e}"))?;
        Ok(())
    }

    fn get(&self, key_name: &str) -> Result<Vec<u8>> {
        let path = self.entry_path(key_name);
        if !path.exists() {
            bail!("keychain item not found: {key_name}");
        }
        let encrypted = std::fs::read(&path).map_err(|e| anyhow::anyhow!("read: {e}"))?;
        self.decrypt_value(&encrypted)
    }

    fn delete(&self, key_name: &str) -> Result<()> {
        let path = self.entry_path(key_name);
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| anyhow::anyhow!("delete: {e}"))?;
        }
        Ok(())
    }

    fn contains(&self, key_name: &str) -> Result<bool> {
        Ok(self.entry_path(key_name).exists())
    }
}

/// OS-native keychain backend.
///
/// On Windows this wraps Credential Manager; on macOS, Keychain Services;
/// on Linux, the Secret Service API. On platforms where a native backend
/// is not yet implemented, constructing an `OsKeychain` returns an error
/// and callers should fall back to [`FileKeychain`].
pub enum OsKeychain {}

impl OsKeychain {
    /// Attempt to open the OS-native keychain. Returns an error on
    /// platforms where no native backend is implemented yet.
    pub fn open() -> Result<Box<dyn Keychain>> {
        bail!("OS-native keychain not implemented on this platform; use FileKeychain");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_keychain_round_trip() {
        let dir = std::env::temp_dir().join(format!("pinpoint-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let master_key = [42u8; 32];
        let kc = FileKeychain::new(dir.clone(), master_key).unwrap();

        // Not found initially.
        assert!(kc.get("nonexistent").is_err());

        // Set and get.
        kc.set("location_key", b"secret-value-123", false).unwrap();
        assert!(kc.contains("location_key").unwrap());
        let val = kc.get("location_key").unwrap();
        assert_eq!(val, b"secret-value-123");

        // Overwrite without flag fails.
        assert!(kc.set("location_key", b"new-value", false).is_err());
        // Overwrite with flag succeeds.
        kc.set("location_key", b"new-value", true).unwrap();
        let val = kc.get("location_key").unwrap();
        assert_eq!(val, b"new-value");

        // Delete.
        kc.delete("location_key").unwrap();
        assert!(!kc.contains("location_key").unwrap());
        assert!(kc.get("location_key").is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_keychain_encrypts_at_rest() {
        let dir = std::env::temp_dir().join(format!("pinpoint-test-enc-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let master_key = [99u8; 32];
        let kc = FileKeychain::new(dir.clone(), master_key).unwrap();

        kc.set("sensitive", b"top-secret-plaintext", false).unwrap();

        // Raw file should not contain plaintext.
        let path = kc.entry_path("sensitive");
        let raw = std::fs::read(&path).unwrap();
        assert!(!raw.contains(b"top-secret-plaintext"));

        // But decryption recovers it.
        let val = kc.get("sensitive").unwrap();
        assert_eq!(val, b"top-secret-plaintext");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
