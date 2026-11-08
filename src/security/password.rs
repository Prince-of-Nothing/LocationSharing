//! Password hashing — PBKDF2-HMAC-SHA256 (pure Rust, no new crates).
//!
//! Replaces demo "any password" acceptance with salted, iterated hashing.
//! Format: `pbkdf2$<iters>$<base64-salt>$<base64-key>` (key 32 bytes).
//! Production guidance: raise iterations to ≥600k and migrate to argon2id/bcrypt
//! when a dedicated crate is allowed (wire contract unchanged).

use base64::Engine;
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub const DEFAULT_ITERS: u32 = 50_000;
const SALT_LEN: usize = 16;

pub fn hash_password(password: &str, iters: u32) -> String {
    let mut salt = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    let key = pbkdf2(password.as_bytes(), &salt, iters);
    let b64 = base64::engine::general_purpose::STANDARD;
    format!("pbkdf2${iters}${}${}", b64.encode(salt), b64.encode(key))
}

pub fn verify_password(password: &str, encoded: &str) -> bool {
    // Format: pbkdf2$<iters>$<base64-salt>$<base64-key>
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

fn pbkdf2(password: &[u8], salt: &[u8], iters: u32) -> [u8; 32] {
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
    use super::{hash_password, verify_password, DEFAULT_ITERS};

    #[test]
    fn hash_and_verify_round_trip() {
        let h = hash_password("DemoPass123!", DEFAULT_ITERS);
        assert!(h.starts_with("pbkdf2$"));
        assert!(verify_password("DemoPass123!", &h));
        assert!(!verify_password("wrong-password", &h));
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
    fn malformed_hash_fails_closed() {
        assert!(!verify_password("x", "garbage"));
        assert!(!verify_password("x", "pbkdf2$abc$salt$hash"));
        assert!(!verify_password("x", ""));
    }
}
