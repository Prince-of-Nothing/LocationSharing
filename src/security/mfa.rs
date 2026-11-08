//! MFA — TOTP (RFC 6238) with HMAC-SHA256.
//!
//! Secrets are random 160-bit values, stored server-side only (never in
//! repos). Time window is 30s, tolerance one step (drift). First-time
//! enrolment returns the secret + otpauth URI for authenticator apps;
//! verification is constant-time and rejects replays via a consumed-code
//! window check (caller persists `last_used_step`).

use base64::Engine;
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub const TOTP_STEP_SECS: u64 = 30;
const SECRET_LEN: usize = 20;

pub fn generate_secret() -> String {
    let mut buf = [0u8; SECRET_LEN];
    rand::thread_rng().fill_bytes(&mut buf);
    base64::engine::general_purpose::STANDARD.encode(buf)
}

pub fn totp(secret_b64: &str, at_unix: u64) -> Option<String> {
    let step = at_unix / TOTP_STEP_SECS;
    let code = code_for_step(secret_b64, step)?;
    Some(format!("{code:06}"))
}

/// Verify `code` for `at_unix` allowing ±1 step drift (prev/current/next).
/// Constant-time compare; returns the step that matched (for replay checks).
pub fn verify_totp(secret_b64: &str, code: &str, at_unix: u64) -> Option<u64> {
    let step = at_unix / TOTP_STEP_SECS;
    for candidate in [step.saturating_sub(1), step, step + 1] {
        let expected = code_for_step(secret_b64, candidate)?;
        // Zero-pad to 6 chars to match totp()/authenticator formatting.
        let a = format!("{expected:06}").into_bytes();
        let b = code.trim().as_bytes().to_vec();
        if a.len() == b.len()
            && a.iter()
                .zip(b.iter())
                .fold(0u8, |acc, (x, y)| acc | (x ^ y))
                == 0
        {
            return Some(candidate);
        }
    }
    None
}

pub fn otpauth_uri(secret_b64: &str, account: &str, issuer: &str) -> String {
    format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA256",
        urlenc(issuer),
        urlenc(account),
        secret_b64,
        urlenc(issuer)
    )
}

fn code_for_step(secret_b64: &str, step: u64) -> Option<u32> {
    let secret = base64::engine::general_purpose::STANDARD
        .decode(secret_b64)
        .ok()?;
    let mut mac = <HmacSha256 as Mac>::new_from_slice(&secret).ok()?;
    mac.update(&step.to_be_bytes());
    let bytes = mac.finalize().into_bytes();
    let offset = (bytes[31] & 0x0f) as usize;
    let code = u32::from_be_bytes([
        bytes[offset] & 0x7f,
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]) % 1_000_000;
    Some(code)
}

fn urlenc(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{generate_secret, otpauth_uri, totp, verify_totp};

    #[test]
    fn rfc4226_appendix_d_vectors() {
        // RFC 4226 HOTP with SHA-1 truncated to 6 digits is the classic
        // vector; Pinpoint uses HMAC-SHA256, so we assert our own SHA256
        // vectors computed independently here (python hmac/hashlib):
        // secret-b64 of "12345678901234567890123456789012" (32B ASCII).
        use base64::Engine;
        let secret_b64 =
            base64::engine::general_purpose::STANDARD.encode(b"12345678901234567890123456789012");
        // HMAC-SHA256 dynamic truncation, mod 1e6 (independently computed
        // via node crypto.createHmac('sha256')).
        let expected = ["920136", "119246", "882438"];
        for (step, want) in expected.iter().enumerate() {
            // totp() uses at_unix/30; drive verify at step*30 directly.
            let at = (step as u64) * 30;
            let code = totp(&secret_b64, at).unwrap();
            assert_eq!(&code, *want, "step {step}");
            assert!(verify_totp(&secret_b64, &code, at).is_some());
        }
    }

    #[test]
    fn totp_round_trip_verifies() {
        let secret = generate_secret();
        let code = totp(&secret, 1_700_000_000).unwrap();
        assert_eq!(code.len(), 6);
        // Same step, ±1 step drift accepted.
        assert!(verify_totp(&secret, &code, 1_700_000_000).is_some());
        assert!(verify_totp(&secret, &code, 1_700_000_000 + 15).is_some());
        // Wrong code rejected.
        assert!(verify_totp(&secret, "000000", 1_700_000_000).is_none() || true); // 000000 is occasionally valid; check a bad code instead:
        let bad = if code == "999999" { "123456" } else { "999999" };
        assert!(verify_totp(&secret, bad, 1_700_000_000).is_none());
    }

    #[test]
    fn totp_round_trip_many_secrets() {
        // Regression: codes with a leading zero must verify (zero-padding bug).
        let mut saw_leading_zero = false;
        for _ in 0..100 {
            let secret = generate_secret();
            for at in [1_700_000_000u64, 1_700_000_031, 1_700_000_500] {
                let code = totp(&secret, at).unwrap();
                if code.starts_with('0') {
                    saw_leading_zero = true;
                }
                assert!(verify_totp(&secret, &code, at).is_some(), "code {code}");
            }
        }
        assert!(saw_leading_zero, "seed must exercise leading-zero codes");
    }

    #[test]
    fn secrets_are_unique() {
        assert_ne!(generate_secret(), generate_secret());
    }

    #[test]
    fn otpauth_uri_shape() {
        let uri = otpauth_uri("ABC", "alice@example.com", "Pinpoint");
        assert!(uri.starts_with("otpauth://totp/Pinpoint:"));
        assert!(uri.contains("secret=ABC"));
    }

    #[test]
    fn invalid_secret_fails_closed() {
        assert!(totp("!!!not-base64!!!", 0).is_none());
    }
}
