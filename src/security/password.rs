//! Password hashing — **Argon2id** (RFC 9106, OWASP first choice).
//!
//! New hashes are PHC strings: `$argon2id$v=19$m=19456,t=2,p=1$<salt>$<hash>`.
//! Argon2id is memory-hard (resists GPU/ASIC cracking), has no 72-byte input
//! limit, and is the modern recommendation over bcrypt/PBKDF2.
//!
//! **Legacy compatibility:** existing `pbkdf2$<iters>$<salt>$<key>` hashes are
//! still verifiable so accounts can log in and be re-hashed to Argon2id on
//! the next successful login (`needs_rehash`). The `pbkdf2` primitive is also
//! kept for E2E key derivation (`src/security/e2e.rs`).

use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use argon2::Argon2;
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Legacy PBKDF2 iteration count (only used for verifying old hashes).
pub const DEFAULT_ITERS: u32 = 50_000;

/// Hash a password with Argon2id (PHC string).
///
/// `legacy_iters` is accepted for call-site compatibility and ignored; Argon2id
/// parameters come from [`Argon2::default`] (OWASP-aligned: m=19456 KiB,
/// t=2, p=1).
pub fn hash_password(password: &str, legacy_iters: u32) -> String {
    let _ = legacy_iters;
    let hash: PasswordHash = Argon2::default()
        .hash_password(password.as_bytes())
        .expect("argon2id hashing");
    hash.to_string()
}

/// Verify a password against an Argon2id PHC hash or a legacy PBKDF2 hash.
/// Fails closed on malformed input (never panics on attacker-controlled hashes).
pub fn verify_password(password: &str, encoded: &str) -> bool {
    if encoded.starts_with("$argon2") {
        let Ok(parsed) = PasswordHash::new(encoded) else {
            return false;
        };
        return Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok();
    }
    verify_legacy_pbkdf2(password, encoded)
}

/// True when a stored hash is not Argon2id (legacy PBKDF2, argon2i/argon2d).
/// Call after a successful login to transparently upgrade the hash.
pub fn needs_rehash(encoded: &str) -> bool {
    !encoded.starts_with("$argon2id$")
}

/// Legacy PBKDF2 verification (format: `pbkdf2$<iters>$<base64-salt>$<base64-key>`).
fn verify_legacy_pbkdf2(password: &str, encoded: &str) -> bool {
    let parts: Vec<&str> = encoded.split('$').collect();
    if parts.len() != 4 || parts[0] != "pbkdf2" {
        return false;
    }
    let Ok(iters) = parts[1].parse::<u32>() else {
        return false;
    };
    let b64 = base64::engine::general_purpose::STANDARD;
    let (Ok(salt), Ok(expected)) = (b64.decode(parts[2]), b64.decode(parts[3])) else {
        return false;
    };
    let actual = pbkdf2(password.as_bytes(), &salt, iters);
    actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected.iter())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0
}

/// PBKDF2-HMAC-SHA256 (single block, 32-byte output). Used for legacy hash
/// verification and E2E key derivation; **not** used for new passwords.
pub fn pbkdf2(password: &[u8], salt: &[u8], iters: u32) -> [u8; 32] {
    // Single-block PBKDF2 (dkLen = hLen = 32).
    let mut block = Vec::with_capacity(salt.len() + 4);
    block.extend_from_slice(salt);
    block.extend_from_slice(&1u32.to_be_bytes());
    let mut mac = <HmacSha256 as Mac>::new_from_slice(password).expect("hmac key");
    mac.update(&block);
    let mut u = mac.finalize().into_bytes();
    let mut t = u.to_vec();
    for _ in 1..iters {
        let mut mac = <HmacSha256 as Mac>::new_from_slice(password).expect("hmac key");
        mac.update(&u);
        u = mac.finalize().into_bytes();
        for (x, y) in t.iter_mut().zip(u.iter()) {
            *x ^= y;
        }
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&t);
    out
}

#[cfg(test)]
mod tests {
    use super::{hash_password, needs_rehash, verify_password, DEFAULT_ITERS};

    #[test]
    fn argon2id_hash_and_verify_round_trip() {
        let h = hash_password("DemoPass123!", DEFAULT_ITERS);
        assert!(h.starts_with("$argon2id$"), "hash was {h}");
        assert!(verify_password("DemoPass123!", &h));
        assert!(!verify_password("wrong-password", &h));
        assert!(!needs_rehash(&h));
    }

    #[test]
    fn salts_are_unique() {
        let a = hash_password("same-password", DEFAULT_ITERS);
        let b = hash_password("same-password", DEFAULT_ITERS);
        assert_ne!(a, b);
        assert!(verify_password("same-password", &a));
        assert!(verify_password("same-password", &b));
    }

    #[test]
    fn legacy_pbkdf2_hash_still_verifies_and_needs_rehash() {
        use super::{pbkdf2, DEFAULT_ITERS};
        use base64::Engine;
        // Build a legacy hash the way the old implementation did.
        let salt = [7u8; 16];
        let key = pbkdf2(b"LegacyPass123!", &salt, DEFAULT_ITERS);
        let b64 = base64::engine::general_purpose::STANDARD;
        let legacy = format!(
            "pbkdf2${}${}${}",
            DEFAULT_ITERS,
            b64.encode(salt),
            b64.encode(key)
        );
        assert!(needs_rehash(&legacy));
        assert!(verify_password("LegacyPass123!", &legacy));
        assert!(!verify_password("wrong", &legacy));
    }

    #[test]
    fn malformed_hash_fails_closed() {
        assert!(!verify_password("x", "garbage"));
        assert!(!verify_password("x", "$argon2id$not-a-valid-phc"));
        assert!(!verify_password("x", "pbkdf2$abc$salt$hash"));
        assert!(!verify_password("x", ""));
        // Non-ASCII input must not panic.
        assert!(!verify_password("x", "$argon2id$v=19$m=1,t=1,p=1$é$é"));
    }
}
