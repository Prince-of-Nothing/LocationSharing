# 0012. Opaque Server-Side Sessions Instead of JWT + Rotating Refresh Tokens

Date: 2026-11-13

## Status

Accepted. Supersedes the token-transport portion of
[ADR-0003](./password-auth-with-rotating-refresh-tokens.md); the password
authentication decision itself (Argon2id, server-enforced) is unchanged.

## Context

ADR-0003 chose **session-bound JWTs (15 min) plus rotating refresh tokens**
back when the backend was the Go service. The Rust rewrite (ADR-0007) now
issues every session from one Axum service family, and the web client is a
same-origin page served by the same process (ADR-0004).

The JWT + refresh design carries costs that the current architecture does not
need to pay:

1. **Two credentials to protect.** A bearer JWT must be kept out of
   `localStorage` (XSS) and a refresh token needs `HttpOnly`, rotation, reuse
   detection and a second table (`pinpoint_refresh_tokens`).
2. **A refresh endpoint and client retry loop.** The web client would need
   refresh-on-401 logic for every call — more code, more failure modes.
3. **Revocation is already server-side.** ADR-0003's own rationale is that a
   session row must be checked so logout and password change take effect
   immediately. Once that lookup happens on every request, the JWT adds
   signing/verification work but no security property.
4. **CSRF is already required.** State-changing requests use the
   `X-CSRF-Token` double-submit pattern, so a cookie-only credential is not
   exposed to cross-site writes.

The threat model is a thesis demo plus a realistic deployment: same-origin
web, a Rust desktop client that already stores cookies and the CSRF token,
and Postgres for multi-replica deployments.

## Decision

1. **One opaque credential.** Login, registration, anonymous creation and
   recovery create a 256-bit random session token, delivered as an
   `HttpOnly; SameSite=Lax` cookie (`Secure` when served over HTTPS) and
   stored in `pinpoint_sessions` **only as its SHA-256** — a stolen database
   dump cannot be replayed as a cookie. The cookie *is* the session; no JWT is
   issued and no token is ever readable by JavaScript.
2. **Per-request validation.** `session_auth` loads the session row (memory
   cache, Postgres fallback), rejects missing/expired sessions, and requires
   `X-CSRF-Token` to equal the session's CSRF token on
   POST/PUT/PATCH/DELETE. Handlers receive `CurrentUser` from the session,
   never from a request body.
3. **Sliding idle timeout, immediate revocation.** `last_activity` is
   refreshed at most every five minutes; a session older than
   `PINPOINT_SESSION_IDLE_MINUTES` (default 720 in the demo, 30 recommended
   for production) is deleted on next use. Logout deletes one session,
   `logout-all` and password change delete every session for the user.
4. **No refresh tokens.** There is nothing to rotate: the cookie is
   long-lived relative to a JWT, but it is revocable on the server at any
   time and is re-issued on every login. Migration 15 is therefore not
   needed and is marked superseded rather than pending.
5. **Device management is real.** `GET /auth/sessions` lists active sessions
   for the signed-in user (8-hex prefix only — the full id is a bearer
   secret), `DELETE /auth/sessions/:id` revokes one by prefix, and
   `POST /auth/logout-all` revokes all. The prefix is matched only within
   the caller's own sessions.

## Consequences

* One indexed lookup per request (shared with the split services through
  Postgres). This was already required for revocation; it is now the only
  auth cost.
* XSS cannot exfiltrate the session cookie (`HttpOnly`), and CSRF is still
  blocked by the double-submit header. The residual risk is a compromised
  browser profile, which no token design prevents.
* Native clients must persist a cookie jar + CSRF token instead of a bearer
  token. `src/api.rs` already does exactly that, so the contract tests stay
  valid.
* Cookies require `SameSite` and `Secure` discipline in deployment. The demo
  runs on loopback HTTP without `Secure`; production must terminate TLS.
* Anonymous accounts use the same machinery: `POST /auth/anonymous` creates
  the account and returns a one-time recovery code, and
  `POST /auth/anonymous/recover` rotates it. Device challenge-response
  (`/auth/anonymous/verify` with a stored public key) remains a follow-up;
  the `public_key` column is in place for it.
* If a future native app needs offline-tolerant long sessions, a refresh
  token can be added *on top of* this design without changing the session
  contract; ADR-0003 stays on record for that case.

## Alternatives considered

* **JWT access + refresh (ADR-0003 as written).** Rejected for this
  architecture: two credentials, a refresh table, rotation/reuse logic and a
  client retry loop, with no property that the server-side session lookup
  does not already provide.
* **Stateless JWT only.** Rejected in ADR-0003 and still rejected: logout and
  password change would not take effect until expiry.
* **Server-side sessions in Redis only.** Rejected: Postgres is already the
  shared store for the split services, and a second stateful dependency
  increases the demo/deployment surface without a measured need.
