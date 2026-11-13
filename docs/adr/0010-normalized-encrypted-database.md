# 0010. Normalized, Encrypted, Modular Database Schema

Date: 2026-11-13

## Status

Accepted (design target). Migration plan in
[`docs/database/schema.md`](../database/schema.md); the current runtime still
uses the older normalized tables + a `pinpoint_state` JSONB snapshot and
plaintext sensitive columns.

## Context

The current persistence layer has three problems:

1. **A single JSONB blob is the rollback source of truth.** `pinpoint_state`
   stores the whole store as one JSON document, alongside `pinpoint_counters`
   (a single-row counter table). That is convenient for demo persistence but
   is redundant with the normalized tables, hard to query/evolve, and
   contradicts the "proper relational schema" requirement.
2. **Sensitive data is plaintext at rest.** `pinpoint_users.email/full_name/
   totp_secret`, `pinpoint_locations` coordinates, `pinpoint_checkins.notes`
   and `pinpoint_shares.label` are readable in a `pg_dump`/stolen volume.
3. **Schema growth is ad-hoc.** New features risk adding columns to unrelated
   tables or stuffing more keys into the snapshot.

The requirement: a **normalized** schema (no redundant single-table blob),
**not one table** (domain-owned tables), **stolen-dump safe** (a dump alone
does not reveal PII/locations, even without the schema), and **modular**
(additive migrations for new features).

## Decision

Adopt the target schema in [`docs/database/schema.md`](../database/schema.md)
and the reference DDL in [`db/schema/00-target.sql`](../../db/schema/00-target.sql):

- **Domain-owned tables:** `users`, `sessions`, `refresh_tokens`,
  `mfa_secrets`, `oauth_identities`, `passkeys`, `recovery_codes`,
  `user_settings` (auth); `friendships`, `blocks` (social); `shares`,
  `locations`, `check_ins`, `emergencies` (location); `audit_events`
  (append-only).
- **No single-table source of truth:** identity columns
  (`GENERATED ALWAYS AS IDENTITY`) replace `pinpoint_counters`; the
  `pinpoint_state` JSONB snapshot is retired (kept only as a read-only
  rollback artifact during migration, then dropped).
- **Application-level encryption** for sensitive columns (`*_enc`, AES-256-GCM,
  `v1:<nonce>:<ciphertext>`, AAD = table:column:row id). The KEK
  (`DATA_ENCRYPTION_KEY`) lives in the environment/KMS, **never in the DB or
  its backups**.
- **Blind indexes** for equality lookups on encrypted fields
  (`users.email_hash`, `oauth_identities.subject_hash`).
- **One-way hashes** for passwords, CSRF tokens, refresh tokens, recovery
  codes.
- **Least privilege:** per-service roles (`pinpoint_auth/users/locations/audit`),
  no runtime superuser; `audit_events` is INSERT/SELECT only.
- **Additive modularity:** new entity → new table + FK; new preference → a key
  in `user_settings.settings` (JSONB) + version bump; new audit detail → a key
  in `audit_events.metadata_enc`. JSONB is only for schemaless feature
  payloads, never for core entities.

Alternatives rejected:

- **Postgres `pgcrypto`** — the key would be supplied to the DB (query logs,
  connection state) and a DB-level compromise could decrypt; application-level
  keeps plaintext out of the DB entirely.
- **Full-disk/volume encryption only** — protects the volume at rest, but a
  `pg_dump`, replica, or misconfigured backup still exposes plaintext.
- **Keep the JSONB snapshot as the source of truth** — redundant, unqueryable,
  and violates the normalized-schema requirement.
- **A single `users` mega-table** — every feature would widen it; violates the
  "not one table" and modularity goals.

## Consequences

Easier:

- A stolen dump reveals only ciphertext, blind-index hashes and opaque ids;
  PII/coordinates are unreadable without the KEK.
- Queries are typed and indexed (no JSON scanning); constraints enforce
  invariants (`direction`, `precision_level`, unique friendships).
- New features are additive (new table + FK) without touching core tables.
- Per-service ownership maps cleanly to the existing service split.

Harder / risks:

- **Key management is now critical:** losing `DATA_ENCRYPTION_KEY` loses the
  data; leaking it defeats the encryption. Keys must live in a KMS/secret
  manager, never with the dump.
- **More code:** encrypt/decrypt on write/read, blind-index maintenance,
  key-rotation jobs.
- **No plaintext analytics** on encrypted columns (counts/timestamps only).
- **Migration work:** backfill + encrypt existing rows, dual-read during the
  transition, retire the snapshot/counters.
- **Precision enforcement:** the server must decrypt coordinates to apply
  `exact/approximate/city_only`; that is compatible with app-level encryption
  (the server holds the KEK) but must be reviewed against the E2E threat model.

Follow-ups:

1. Add the target tables as idempotent migrations (v15+), then backfill.
2. Implement `EncryptedColumn` helpers (AES-256-GCM, AAD, `v1:` format) and
   blind-index helpers.
3. Encrypt `users`/`locations`/`check_ins`/`emergencies`/`shares` sensitive
   columns; dual-read during the transition.
4. Retire `pinpoint_state` + `pinpoint_counters`; switch the runtime to the
   normalized tables.
5. Enforce per-service grants; remove runtime superuser.
6. Add key-rotation + backup-key separation to `docs/DEPLOYMENT.md`.
