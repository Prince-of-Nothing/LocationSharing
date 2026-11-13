# 09 — Data Model and API

> **Persistence target:** the normalized, application-encrypted schema
> (ADR-0010) is specified in
> [`docs/database/schema.md`](../database/schema.md) with reference DDL in
> [`db/schema/00-target.sql`](../../db/schema/00-target.sql). This document
> keeps the domain/API view; the schema doc is the storage source of truth.

## Domain entities (consent core — unchanged)

```text
User(id, email*, display_name, password_hash, role, must_change_password, locked, timestamps)
Friendship(id, requester_id, addressee_id, status: pending|accepted|rejected|removed)
SharePermission(id, sharer_id, recipient_id, precision: exact|approximate|city_only,
                purpose?, expires_at?, state: active|paused|revoked|expired, timestamps)
Location(user_id PK, lat, lon, accuracy_m, updated_at)  -- latest publish only
```

`*` unique; email→id index maintained by store. Full ER in
`docs/uml/09-er-diagram.md`; reference model in
`docs/reference/domain-data-model.md`.

## Map entities (new, client-local)

```text
SearchPlace(display_name, short_name, lat, lon, category, type, osm_id, osm_type)  -- Nominatim, in-memory
Route(points[GeoPoint], distance_m, duration_s, steps[RouteStep], summary, mode)    -- OSRM, in-memory
Poi(osm_id, name, lat, lon, category, icon, phone, website, opening_hours, cuisine) -- Overpass, in-memory
Bookmark(id, name, lat, lon, note, category, created_at)                            -- on-device only
OfflinePack(name, bounds, zmin, zmax, bytes, complete)                              -- mobile only
```

No new server tables for maps. Bookmarks are **not** server-synced.

## API contract (`/api/v1`, Go + Python mirror)

| Group | Endpoints (auth unless noted) |
|-------|-------------------------------|
| Auth | `POST /auth/login|register|logout|session`, `GET /auth/me`, `POST /auth/password/change|force-change` |
| Users | `POST /users/` (admin), `GET /users/`, `GET /users/{id}`, `PATCH /users/{id}` |
| Friendships | `POST /friendships/`, `GET /friendships/`, `POST /friendships/{id}/respond` |
| Locations | `POST /locations/shares`, `GET /locations/shares`, `PATCH /locations/shares/{id}`, `POST /locations/me`, `GET /locations/friends` |

Conventions: JSON; unknown fields rejected (Python) / strictly validated
(Go); timestamps ISO-8601 UTC; errors `{"error":{"code","message"}}` (Go)
or RFC 9457 (Python); 404 for not-found **or not-yours**; 429 with
`Retry-After`; 401 with session bootstrap.

## Precision & freshness semantics

| Precision | Server read-time handling |
|-----------|---------------------------|
| `exact` | full coords + accuracy |
| `approximate` | rounded to 2 decimals (~1.1 km), no accuracy leak |
| `city_only` | rounded to 1 decimal (~11 km), centroid only |

Freshness: `fresh` ≤10 min, `warn` 10–30 min, `stale` >30 min or unknown
(web `js/map.js` + `config.js`). Recipients never get raw coords for
downgraded shares — not in body, not in headers, not in logs.

## OSM external contracts (not Pinpoint API)

- Nominatim: `GET /search?q=&format=jsonv2&limit=8&addressdetails=1`
- OSRM: `GET /route/v1/{profile}/{lon,lat};…?overview=full&geometries=polyline&steps=true`
- Overpass: `POST /api/interpreter` with `data=<QL>` (form-urlencoded)

See 07 for policies and self-hosting. External contracts are versioned by
their owners; Pinpoint normalizes them to domain models so provider swaps
do not ripple to features.
