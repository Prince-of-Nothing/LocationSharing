# Pinpoint — Postgres HA (streaming replica prototype)

Default stays a single primary (`docker-compose.yml` → `db`). This doc is
the production step the redundancy doc points to: a hot-standby that you
promote when the primary dies, so reads keep flowing with near-zero
data loss (WAL replay, not periodic `pg_dump`).

## Prototype (docker-compose.ha.yml — opt-in)

```bash
docker compose -f docker-compose.yml -f docker-compose.ha.yml up -d db-replica
# verify replication:
docker exec pinpoint-db psql -U pinpoint -d pinpoint -c "SELECT client_addr, state, replay_lag FROM pg_stat_replication"
```

`docker-compose.ha.yml` (in repo root) adds `db-replica`:
- waits for `db` healthy, then `pg_basebackup -h db -U replicator -D . -Ft -z -Xs -P -v`
- starts with `standby.signal` + `primary_conninfo='host=db …'` (WAL streaming)
- own volume `pinpoint-db-replica-data` so promote never loses the primary's disk

Primary needs a replication slot + role (already created by the overlay's
init SQL when the primary is fresh; for an existing volume run it once
manually — see the file header comments).

## Failover

```bash
# 1. promote the replica (read-write):
docker exec pinpoint-replica pg_ctl promote -D /var/lib/postgresql/data
# 2. repoint services (primary DNS name unchanged — stop old primary first):
docker compose stop db
docker compose run --rm --no-deps -e DATABASE_URL=postgresql://…@db-replica:5432/pinpoint auth true  # smoke
docker compose up -d auth users locations   # pick up db-replica via new DATABASE_URL
```

Promote is manual (one command) — automated promotion (Patroni/Stolon,
or a managed Postgres) is the cloud step. Old primary, when it returns,
must NOT rejoin as primary; rebuild it as a fresh replica from the
promoted primary before re-adding it.

## Why not in the default compose?

Streaming replication adds a second PG instance, WAL archiving bandwidth,
and a promotion decision to a demo stack that currently recovers via
`pinpoint-backups` (`pg_dump` every 300s, keep 24). The overlay keeps
that trade-off explicit and keeps `docker compose up` (default) a
one-primary demo. See `docs/redundancy.md` for the service-layer HA
that is already default (`scale: 2` + gateway retries).
