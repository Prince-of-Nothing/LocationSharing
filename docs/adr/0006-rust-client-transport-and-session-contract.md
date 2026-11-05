# 0006. Rust Desktop Client Transport Policy and Session Contract

Date: 2026-11-05

## Status

Accepted

## Context

Rust is the project's chosen long-term primary implementation
([ADR-0002](./0002-adopt-pinpoint-product-direction.md) direction, recorded in
`docs/task1/november-project-progress-and-plan.md`). The first Rust deliverable is
the desktop client (root crate `pinpoint-desktop`), which talks to the existing
Go API (default stack) and, during the migration window, must remain compatible
with the Python API.

Two security properties of that client were previously unverified — both called
out as High risks in the November progress report:

1. **Transport selection was unchecked.** The client accepted any `--backend`
   value and defaulted to plain `http://localhost:8000`, with no rule preventing
   a production deployment from silently running over cleartext HTTP.
2. **Session/CSRF handling did not match the server contract.** The Go API
   authenticates cookie sessions and requires the double-submit `X-CSRF-Token`
   header on every unsafe (POST/PUT/PATCH/DELETE) request
   (`backend/internal/handlers/middleware.go`), returning `403 CSRF_FAILED`
   otherwise. The client captured only the first `Set-Cookie`, never sent the
   CSRF header, and parsed the login envelope as a bare user object, so no
   state-changing request could succeed against the real server.

Forces: the thesis security baseline requires server-enforced, fail-closed
behavior; the client must not weaken it by, e.g., reporting a logout that never
happened or treating a 200 without a session cookie as signed in. Migration
parity requires the Rust client to work against the *existing* APIs without
server changes.

## Decision

We will enforce the following in `pinpoint-desktop`, pinned by tests:

1. **Transport policy (fail closed).** `api::ApiClient::new` validates the
   backend URL before any request can be issued: `https` is always accepted;
   plain `http` is accepted only for loopback hosts (`localhost`, `127.0.0.1`,
   `::1`); any other scheme, host, or malformed URL is rejected at startup with
   a clear error. Alternatives considered: *trust-the-operator* (rejected — it
   is exactly the risk the report lists as High) and *warn-but-allow* (rejected
   — warnings do not fail closed).
2. **Session contract mirroring the Go handlers.** On login/register the client
   stores **all** `Set-Cookie` `name=value` pairs from the response and the
   double-submit token (login body `csrf_token`, falling back to the
   `X-CSRF-Token` response header). Every request carries the cookie; unsafe
   methods additionally carry `X-CSRF-Token`. GETs deliberately do not.
3. **Fail-closed session handling.** A `200` login that issued no session
   cookie is an error (the client would otherwise appear signed in while
   anonymous). `logout` clears local state only after the server confirms;
   a failed revocation keeps the session and surfaces the error. A regular
   password change (server revokes all sessions) drops local state, while a
   forced change keeps it. Login/logout failures and refresh failures are
   shown in the UI instead of being discarded.
4. **Structured error surfacing.** The `{error: {code, message, fields}}`
   envelope is rendered as `CODE: message (field: …)`; raw bodies are truncated
   and never dumped into the UI.

## Consequences

* Plain HTTP to non-loopback hosts (LAN IPs, DNS names) is impossible;
  remote deployments must terminate TLS (or tunnel), which matches the
  deployment gates in the progress report. Local development against
  `http://localhost:8000` is unchanged.
* The wire contract is now executable: `tests/contract.rs` (recorded HTTP
  server, no extra dependencies) pins cookie/CSRF placement, the login
  envelope, numeric IDs, error formatting, and the fail-closed cases. Any
  change to the Go handlers' auth surface must be mirrored here in the same
  change.
* The `rust` CI job (`fmt`, `clippy -D warnings`, locked build, tests,
  `cargo-audit`) makes this contract a required check.
* The Python API's login envelope is expected to match (`csrf_token` +
  `user`); parity against the Python stack during the migration window is
  verified by the same tests only if its field names diverge — flagged for
  the January parity pass.
* Follow-up: session bootstrap on startup (`POST /api/v1/auth/session`) and
  cookie persistence across restarts remain open for the operations window.
