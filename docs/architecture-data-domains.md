# Pinpoint — Data architecture: modular domains, one connected database

Modular software does **not** need three disconnected databases. Pinpoint
keeps one Postgres database (`pinpoint`) so user data, location data, and
saved/map data stay joinable (who shared with whom, at what precision, to
which friend), while services own separate **domains** of it.

## Domain map

| Domain | Tables | Service (owner writes) | FK glue |
|---|---|---|---|
| User data | `pinpoint_users`, `pinpoint_friendships`, `pinpoint_shares`, `pinpoint_counters` | auth (users), users (friendships), locations (shares) | friendships.requester/receiver → `pinpoint_users` |
| Location data | `pinpoint_locations`, `pinpoint_checkins`, `pinpoint_emergencies` | locations | `user_id` → `pinpoint_users`; shares.friendship_id → friendships |
| Saved/map data | `pinpoint_state` (JSONB rollback only), tiles in `~/.pinpoint/tiles/` (device only) | map (tiles, stateless) | attribution header on every tile response |
| Meta | `pinpoint_schema_migrations` (v1–v10) | all services (read) | — |

Constraints: `db/init/03-fks.sql` (superuser bootstrap, idempotent) adds
`FOREIGN KEY` constraints across the user↔location boundary; services keep
NO ACTION referential integrity (no cascades across ownership borders).

## Why not three separate databases?

- You can reasonably want user data, map data, and prompt/save data in
  separate stores — Pinpoint effectively does that for **writes** (ownership
  scopes in code, least-privilege roles in the DB).
- Splitting to **disconnected** physical DBs would lose joins that the
  product needs (share → friend → user; location → sharer), add distributed
  transactions, and is unnecessary at this scale. The modular seam is the
  **service + table-owner** boundary, not the wire.

## Write ownership (who may INSERT/UPDATE/DELETE)

- `pinpoint_users`: Auth only (per-row upsert + delete-missing); users
  service upserts single rows via direct SQL (see `state.rs::sql_*`).
- `pinpoint_friendships`: Users service only.
- `pinpoint_shares`, `pinpoint_locations`, `pinpoint_checkins`,
  `pinpoint_emergencies`: Locations service only.
- `pinpoint_counters`: routed per service (own columns only).
- `pinpoint_state`: combined binary only (`ServiceKind::All`), rollback.

`persist()` writes each owned table per-row (upsert + delete-missing)
inside one transaction serialized across replicas of the same service
by `pg_advisory_xact_lock(ServiceKind::lock_key())`. Different
services (different keys) persist concurrently; same-service replicas
take turns — no table is ever wipe-rewritten.

## Convergence

Every service runs `persist()` then `refresh_from_db()` on a 5s tick, so a
service picks up other services' rows (e.g. a user registered via auth,
visible to users without a restart) within seconds. Reads are shared;
writes are owned. Lossless ordering: deletes children-first, inserts
parents-first (matches the FK graph).
