# 0009. Dual Authentication Models — Anonymous Device-Bound and Identity-Linked Accounts

Date: 2026-11-13

## Status

Accepted (decision). Implementation is **partial**: the `User` schema fields
and migration 14 are landed; the session store, middleware, real auth handlers
and anonymous endpoints are pending. Session mechanics are specified in
[`docs/security/session-management.md`](../security/session-management.md).

## Context

Pinpoint's only authentication path today is **email + password**
(ADR-0003): `POST /api/v1/auth/register` + `/login`, Argon2id password hashing,
server-side sessions. That creates a tension with the product's privacy
positioning:

- **Privacy cost:** every account requires an email address (PII), and the
  server links all activity to that identity. A fully anonymous user cannot
  use the product at all.
- **Usability cost:** email/password adds friction, and users who lose access
  to their email lose their account.
- **Competitive landscape:** mainstream location-sharing apps (Google Maps,
  Apple Find My, Life360) require an account; privacy-focused messengers
  (Signal) require a phone number; only a few (Session, some Web3 apps) offer
  truly anonymous accounts, and only some (Apple Sign-in, passkeys) offer
  passwordless identity.

The request is to support **two sign-up/sign-in models** behind one account:

1. **Anonymous / device-bound** — no email, no password; the account is created
   automatically and managed on-device. Fully anonymous, but no server-side
   recovery (device loss = account loss unless the user backs up).
2. **Identity-linked** — the user brings an identity (email, Google/Apple
   sign-in, passkey). Less anonymous, but recoverable and usable across
   devices.

## Decision

We will support **two authentication models over a single user record**, with
an explicit **upgrade path** from anonymous to identity-linked:

- **Model A — Anonymous, device-bound.** On first launch the client generates
  a key pair (Ed25519), stores the private key in the OS keychain
  (`src/security/keychain.rs`), and registers the **public key** with the
  server. The server creates a user with a random ID and **no PII**, and
  issues a session. Authentication is a challenge-signature (private key
  proves ownership).
- **Model B — Identity-linked.** Email+password (existing), passwordless
  email magic link, OAuth/OIDC (Google/Apple), or passkeys (WebAuthn). The
  server stores the identity (email or IdP subject) and uses existing session
  machinery.
- **Upgrade path.** An anonymous session can **link** an identity later
  (`POST /api/v1/auth/link`), converting the account to Model B while
  preserving its history. The server treats identity as optional metadata on
  the same user row.
- **Recovery.** Model A: optional recovery code / backup export (user-held);
  without it, the account is unrecoverable by design. Model B: recovery via
  the identity (magic link, OAuth, passkey sync).
- **Abuse control.** Anonymous creation is rate-limited per IP/device and may
  require device attestation (App Attest / Play Integrity) or invite codes to
  mitigate Sybil accounts.

Alternatives rejected:

- **Anonymous-only.** Maximises privacy but makes cross-device use and
  recovery impossible for normal users; too narrow for the demo.
- **Identity-only (status quo).** Simplest, but contradicts the privacy-first
  positioning and excludes anonymous users.
- **One model with optional email.** Rejected: "optional email" still forces
  a recovery/identity concept and muddies the consent story; two explicit
  models are clearer and better testable.

## Consequences

Easier:

- **Stronger privacy story:** a true anonymous mode (no PII) differentiates
  Pinpoint from Google/Apple/Life360 and matches the 0-tracker positioning.
- **Lower friction onboarding:** "Continue anonymously" is one tap.
- **Flexible recovery:** users choose the privacy/recoverability trade-off.
- **Upgrade path:** anonymous users can add an identity later without losing
  data.

Harder / risks:

- **Auth complexity:** two (or more) auth flows, a challenge-signature scheme,
  and an upgrade path must all be implemented and tested.
- **Account loss:** anonymous users who lose the device and have no backup
  lose the account — must be communicated clearly and mitigated with a
  recovery code.
- **Sybil/abuse:** anonymous accounts are cheap to create; without attestation
  or rate limits they enable spam and ban evasion.
- **Schema/migration:** `User` gains `auth_method`, `identity_subject`,
  `public_key`, `recovery_code`; existing accounts default to Model B.
- **Dependency:** OAuth/passkeys add external IdP dependencies and testing
  surface.

Follow-ups:

- Add `auth_method` + identity/public-key columns to the user store and a
  migration (`src/server/migrations.rs`).
- Implement `/api/v1/auth/anonymous`, `/auth/link`, and (phase 2) OAuth /
  magic-link / passkey endpoints.
- Client: onboarding choice UI, key generation in the keychain, recovery-code
  export/import.
- Abuse controls: rate limits, device attestation, optional invite codes.
- Update `docs/PRIVACY.md`, `docs/SECURITY.md`, and the traceability matrix.
