# 9. Architecture Decisions

This section is an index into the [Architecture Decision Records](../adr) (ADRs). Do
**not** duplicate decision rationale here — link to the ADRs, which are the durable
source of truth for *why* a decision was made.

| ADR | Title | Status |
|-----|-------|--------|
| [0001](../adr/record-architecture-decisions.md) | Record architecture decisions | Accepted |
| [0002](../adr/0002-adopt-pinpoint-product-direction.md) | Adopt Pinpoint as the product direction | Accepted |
| [0003](../adr/password-auth-with-rotating-refresh-tokens.md) | Password authentication with session-bound JWTs and rotating refresh tokens for the MVP | Accepted |
| [0004](../adr/web-client-for-mvp-demonstration.md) | Web client for the MVP demonstration; Flutter app deferred | Accepted |
| [0005](../adr/store-latest-location-only.md) | Store only the latest location; derive freshness and check-in states on read | Accepted |
| [0006](../adr/0006-rust-client-transport-and-session-contract.md) | Rust desktop client transport policy (HTTPS off-loopback) and session/CSRF contract | Accepted |

Add a row here whenever a new ADR is created. See [`adr/template.md`](../adr/template.md)
for the format used to write new ADRs, and [`adr/README.md`](../adr/README.md) for the
process.

---
*Part of the [architecture documentation](./README.md) (arc42 §9).*
