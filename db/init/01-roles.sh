#!/bin/sh
# Create per-service least-privilege DB roles (run once at DB init).
# Tables are created lazily by services at first boot; run db/02-grants.sql
# afterwards for per-table grants (see docs/DEPLOYMENT.md).
# Local default uses identical dev passwords; production sets AUTH_DB_PWD etc.
# via environment (never commit real secrets).
set -eu
AUTH_PWD="${AUTH_DB_PWD:-pinpoint-auth-dev}"
USERS_PWD="${USERS_DB_PWD:-pinpoint-users-dev}"
LOC_PWD="${LOCATIONS_DB_PWD:-pinpoint-locations-dev}"
psql -v ON_ERROR_STOP=1 -U "$POSTGRES_USER" -d "$POSTGRES_DB" <<SQL
DO \$\$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'pinpoint_auth') THEN
    CREATE ROLE pinpoint_auth LOGIN PASSWORD '$AUTH_PWD';
  ELSE
    ALTER ROLE pinpoint_auth WITH LOGIN PASSWORD '$AUTH_PWD';
  END IF;
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'pinpoint_users') THEN
    CREATE ROLE pinpoint_users LOGIN PASSWORD '$USERS_PWD';
  ELSE
    ALTER ROLE pinpoint_users WITH LOGIN PASSWORD '$USERS_PWD';
  END IF;
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'pinpoint_locations') THEN
    CREATE ROLE pinpoint_locations LOGIN PASSWORD '$LOC_PWD';
  ELSE
    ALTER ROLE pinpoint_locations WITH LOGIN PASSWORD '$LOC_PWD';
  END IF;
  ALTER ROLE pinpoint_auth WITH NOCREATEDB NOCREATEROLE NOSUPERUSER;
  ALTER ROLE pinpoint_users WITH NOCREATEDB NOCREATEROLE NOSUPERUSER;
  ALTER ROLE pinpoint_locations WITH NOCREATEDB NOCREATEROLE NOSUPERUSER;
END \$\$;
-- PG15+ removed CREATE from PUBLIC on the public schema; services create
-- tables lazily (CREATE TABLE IF NOT EXISTS) as their own roles.
GRANT CREATE ON SCHEMA public TO pinpoint_auth, pinpoint_users, pinpoint_locations;
SQL
