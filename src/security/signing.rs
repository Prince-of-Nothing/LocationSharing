//! Security — Request signing.
//!
//! Signs API requests to ensure integrity and authenticity. Used for
//! sensitive operations like location publishing and share creation.

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Signing key for request signing.
#[derive(Clone)]
pub struct SigningKey {
    key: [u8; 32],
}

impl SigningKey {
    /// Creates a new random signing key.
    pub fn new() -> Self {
        use rand::RngCore;
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        Self { key }
    }

    /// Creates from raw bytes (32 bytes).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != 32 {
            return Err("Signing key must be 32 bytes".to_string());
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(bytes);
        Ok(Self { key })
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }
}

impl Default for SigningKey {
    fn default() -> Self {
        Self::new()
    }
}

/// Request signature.
#[derive(Debug, Clone)]
pub struct RequestSignature {
    pub signature: String,
    pub timestamp: u64,
    pub nonce: String,
}

/// Signs API requests.
pub struct RequestSigner {
    key: Mutex<Option<SigningKey>>,
}

impl RequestSigner {
    pub fn new() -> Self {
        Self {
            key: Mutex::new(None),
        }
    }

    pub fn with_key(key: SigningKey) -> Self {
        Self {
            key: Mutex::new(Some(key)),
        }
    }

    /// Sets the signing key.
    pub fn set_key(&self, key: SigningKey) {
        *self.key.lock().unwrap() = Some(key);
    }

    /// Returns the current key if set.
    pub fn current_key(&self) -> Option<SigningKey> {
        self.key.lock().unwrap().clone()
    }

    /// Signs a request with the given method, path, and body.
    pub fn sign_request(
        &self,
        method: &str,
        path: &str,
        body: &[u8],
    ) -> Result<RequestSignature, String> {
        let key = self
            .key
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| "No signing key configured".to_string())?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();

        let nonce = generate_nonce();

        // Build the canonical string: method\npath\ntimestamp\nnonce\nbody_hash
        let body_hash = sha256_hex(body);
        let canonical = format!("{method}\n{path}\n{timestamp}\n{nonce}\n{body_hash}");

        let signature = hmac_sha256_hex(&key.as_bytes()[..], canonical.as_bytes());

        Ok(RequestSignature {
            signature,
            timestamp,
            nonce,
        })
    }

    /// Verifies a request signature.
    pub fn verify_signature(
        &self,
        method: &str,
        path: &str,
        body: &[u8],
        signature: &RequestSignature,
        tolerance_seconds: u64,
    ) -> Result<(), String> {
        let key = self
            .key
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| "No signing key configured".to_string())?;

        // Check timestamp tolerance
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();

        if signature.timestamp > now {
            if signature.timestamp - now > tolerance_seconds {
                return Err("Signature timestamp in the future".to_string());
            }
        } else {
            if now - signature.timestamp > tolerance_seconds {
                return Err("Signature timestamp too old".to_string());
            }
        }

        // Recompute signature
        let body_hash = sha256_hex(body);
        let canonical = format!(
            "{method}\n{path}\n{}\n{}\n{body_hash}",
            signature.timestamp, signature.nonce
        );

        let expected = hmac_sha256_hex(&key.as_bytes()[..], canonical.as_bytes());

        if expected == signature.signature {
            Ok(())
        } else {
            Err("Invalid signature".to_string())
        }
    }
}

impl Default for RequestSigner {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a random nonce.
fn generate_nonce() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// SHA-256 hex digest.
fn sha256_hex(data: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

/// HMAC-SHA256 hex digest.
fn hmac_sha256_hex(key: &[u8], data: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key length");
    mac.update(data);
    let result = mac.finalize();
    result
        .into_bytes()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify_request() {
        let signer = RequestSigner::with_key(SigningKey::new());

        let signature = signer
            .sign_request(
                "POST",
                "/api/v1/locations/me",
                b"{\"lat\":41.89,\"lon\":-87.63}",
            )
            .unwrap();

        assert!(!signature.signature.is_empty());
        assert!(signature.timestamp > 0);
        assert!(!signature.nonce.is_empty());

        // Verify should succeed
        assert!(signer
            .verify_signature(
                "POST",
                "/api/v1/locations/me",
                b"{\"lat\":41.89,\"lon\":-87.63}",
                &signature,
                300,
            )
            .is_ok());
    }

    #[test]
    fn wrong_body_fails_verification() {
        let signer = RequestSigner::with_key(SigningKey::new());

        let signature = signer
            .sign_request("POST", "/api/v1/locations/me", b"{\"lat\":41.89}")
            .unwrap();

        // Different body should fail
        assert!(signer
            .verify_signature(
                "POST",
                "/api/v1/locations/me",
                b"{\"lat\":41.90}",
                &signature,
                300,
            )
            .is_err());
    }

    #[test]
    fn wrong_method_fails_verification() {
        let signer = RequestSigner::with_key(SigningKey::new());

        let signature = signer
            .sign_request("POST", "/api/v1/locations/me", b"{}")
            .unwrap();

        assert!(signer
            .verify_signature("GET", "/api/v1/locations/me", b"{}", &signature, 300)
            .is_err());
    }

    #[test]
    fn expired_signature_fails() {
        let signer = RequestSigner::with_key(SigningKey::new());

        // Create a signature with an old timestamp
        let signature = RequestSignature {
            signature: "fake".to_string(),
            timestamp: 1,
            nonce: "test".to_string(),
        };

        assert!(signer
            .verify_signature("POST", "/api/v1/test", b"{}", &signature, 300)
            .is_err());
    }

    #[test]
    fn no_key_returns_error() {
        let signer = RequestSigner::new();
        assert!(signer.sign_request("POST", "/api/v1/test", b"{}").is_err());
    }
}
