-- Per-table least-privilege grants (run AFTER services have created tables:
--   docker exec pinpoint-db psql -U pinpoint -d pinpoint -f /grants.sql
-- or via the bootstrap target). Owner-mapping mirrors ServiceKind in
-- src/server/state.rs: each binary writes only its own tables.
--
-- auth service (pinpoint-auth): credentials + MFA fields.
GRANT ALL ON pinpoint_users, pinpoint_state TO pinpoint_auth;
GRANT ALL ON pinpoint_counters TO pinpoint_auth;
--
-- users service (pinpoint-users): profile + friendships (+ own counters).
GRANT ALL ON pinpoint_users, pinpoint_friendships, pinpoint_state TO pinpoint_users;
GRANT ALL ON pinpoint_counters TO pinpoint_users;
--
-- locations service (pinpoint-locations): shares/locations/check-ins/emergencies;
-- read-only users+friendships for current_user/consent checks.
GRANT ALL ON pinpoint_shares, pinpoint_locations, pinpoint_checkins, pinpoint_emergencies TO pinpoint_locations;
GRANT SELECT ON pinpoint_users, pinpoint_friendships TO pinpoint_locations;
GRANT UPDATE (next_share_id, next_checkin_id, next_emergency_id) ON pinpoint_counters TO pinpoint_locations;
--
-- map service: no database access (no DATABASE_URL).
--
-- Deny-by-default: revoke from PUBLIC and don't grant anything else.
REVOKE ALL ON ALL TABLES IN SCHEMA public FROM PUBLIC;
