# Pinpoint — E2E Encryption + MFA Design Note

**Status:** MFA implemented (client verify + unit tests). E2E encryption: design only — NOT implemented (server sees plaintext locations; accepted MVP trade-off).

## 1. MFA (TOTP, implemented)

- `src/security/mfa.rs` — RFC 6238 TOTP with HMAC-SHA256, 30s step, ±1 step tolerance, constant-time compare.
- `generate_secret()` (160-bit random), `totp(secret, unix)`, `verify_totp(...) -> Option<step>`, `otpauth_uri(...)`.
- Tests: round-trip + drift, unique secrets, uri shape, fail-closed on bad secret.
- Remaining integration: server-side `totp_secret_enc` column + `/auth/mfa/*` endpoints + UI enrolment screen (DB + API work, not yet wired to client). Client verify + server verify must agree on step boundaries (use `verify_totp` on both sides).

## 2. E2E encryption (design, post-MVP)

Goal: server stores only ciphertext; friends decrypt with per-friendship keys.

- Key exchange: X25519 ECDH per friendship (first connect), wrapped under each party's device key.
- Payload: AES-256-GCM over `{lat, lon, accuracy, ts}` per publish; server stores opaque blob + `precision_level` **outside** the ciphertext (needed for server-side precision enforcement).
- Conflict: server-side precision downgrade (T-I1 defence) requires plaintext. E2E therefore needs client-side coarsening + server still enforces a **max-precision cap** agreed at share creation. This weakens T-E2 unless the client is trusted — acceptable only with attestation.
- Key storage: OS keychain (Windows Credential Manager / macOS Keychain / Android Keystore), never in `bookmarks.json` or repo.
- Migration: dual-read path (plaintext `FriendLocation` + ciphertext blob) behind feature flag; revocation rotates friendship keys.

## 3. OS keychain for bookmark keys

- **Implemented.** `src/security/keychain.rs` provides a unified `Keychain`
  trait with `OsKeychain` (Windows Credential Manager / macOS Keychain /
  Linux Secret Service via the `keyring` crate) and a `FileKeychain`
  fallback (AES-256-GCM, file-backed master key). `KeyManager` now
  persists its active key through `Keychain::load_or_generate`, and
  `PinpointApp::new` opens the best-available keychain via
  `keychain::open_default`. See `src/security/encryption.rs`.
- Fallback: derived key from user password (PBKDF2) with salt in
  `bookmarks.json` header — still pending for platforms without any
  keychain (currently in-memory).

## 4. Threat-model deltas

- MFA closes T-S1 (credential stuffing → account takeover) for enrolled users; replay protection via consumed-step tracking.
- E2E closes T-I3 (DB dump) but reopens T-E2 (precision) — must ship with cap + attestation before removing server downgrade.
