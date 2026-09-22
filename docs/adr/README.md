# Architecture Decision Records (ADR)

This directory contains the project's **Architecture Decision Records** — a numbered,
append-only log of significant architectural decisions and the reasoning behind them.

ADRs are the primary safeguard against losing architectural knowledge: even after a
decision is superseded or the code is rewritten, the *why* remains here permanently.

## When to write an ADR

Write one whenever a decision is:
- Hard or costly to reverse,
- Affects multiple components or the system's overall shape,
- Involved weighing meaningful trade-offs/alternatives, or
- Would otherwise only live in someone's memory or a chat thread.

Examples for this project: choice of backend framework/language, real-time transport
(WebSockets vs. MQTT vs. polling), database technology, authentication approach,
location data retention/privacy policy, mobile framework, hosting/cloud provider.

## Process

1. Copy [`template.md`](./template.md) to `NNNN-short-title.md` (next sequential
   number, kebab-case title).
2. Fill it in — status starts as `Proposed`.
3. Discuss/review; once agreed, set status to `Accepted` (or `Rejected`).
4. Add a row to the index in
   [`architecture/09-architecture-decisions.md`](../architecture/architecture-decisions-link.md).
5. **Never edit or delete an old ADR's decision after acceptance.** If circumstances
   change, write a *new* ADR that supersedes it, and mark the old one
   `Superseded by ADR-NNNN`.

## Index

See [`architecture/09-architecture-decisions.md`](../architecture/architecture-decisions-link.md)
for the full, up-to-date list.
