# 0011. Self-Hosted OpenStreetMap Services (Tiles, Search, Routing, POI)

Date: 2026-11-13

## Status

Accepted (config + docs landed; deployment is operator work).

## Context

Pinpoint's map stack currently calls **public demo endpoints**:

- tiles via CARTO / Wikimedia / OpenFreeMap (third-party, ODbL-derived),
- search via `nominatim.openstreetmap.org`,
- routing via `router.project-osrm.org`,
- POI via `overpass-api.de`.

Those endpoints are fine for a local demo but not for a real deployment:

- **Usage policies:** Nominatim is ≤1 req/s and forbids bulk use; the OSRM
  demo server is not for production; Overpass is fair-use only.
- **Privacy:** every user query (typed text, coordinates, POI browse) leaves
  for OSMF/third-party infrastructure. Even though friend locations are never
  sent, the queries themselves are metadata.
- **Reliability:** public endpoints rate-limit, change, or go down.
- **Control:** self-hosting lets us pin data versions, cache aggressively,
  and keep the service inside our trust boundary.

ODbL **requires attribution** regardless of who hosts the data: self-hosting
does not remove "© OpenStreetMap contributors" or the licence link.

## Decision

We will make every OSM service endpoint **configurable** and treat
**self-hosted services as the production default**:

| Env var | Service | Default (demo only) |
|---|---|---|
| `PINPOINT_NOMINATIM_URL` | Nominatim search/reverse | `https://nominatim.openstreetmap.org` |
| `PINPOINT_OSRM_URL` | OSRM routing | `https://router.project-osrm.org` |
| `PINPOINT_OVERPASS_URL` | Overpass POI | `https://overpass-api.de/api/interpreter` |
| `PINPOINT_TILE_URL` | Raster tile template (`…/{z}/{x}/{y}.png`) | CARTO/Wikimedia/OpenFreeMap |

Rules:

1. **Production sets all four** to self-hosted URLs; the public defaults are
   for local development/demo only.
2. **Never** use `tile.openstreetmap.org` (OSMF policy; QA grep is a blocker).
3. **ODbL attribution stays** on every map view and tile response
   (`X-Data-Attribution`), plus the `openstreetmap.org/copyright` link.
4. **Privacy envelope unchanged:** only user-initiated queries; friend/shared
   coordinates never leave Pinpoint.
5. The self-hosted stack over Geofabrik extracts is documented in
   [`docs/maps/self-hosted-osm.md`](../maps/self-hosted-osm.md).

Alternatives rejected:

- **Keep public endpoints in production** — violates usage policies and leaks
  query metadata; rejected.
- **Third-party managed tiles only** — convenient, but keeps a third party in
  the trust path and can change pricing/terms; kept only as demo fallback.
- **No attribution because "we host it"** — ODbL does not work that way;
  attribution is mandatory.

## Consequences

Easier:

- Queries stay on our infrastructure; no OSMF/third-party rate limits.
- Predictable latency and data version pinning.
- Full control over caching, logging, and retention.
- The same client code works for demo and production (env-only change).

Harder / risks:

- **Ops cost:** disk (planet vs country extract), RAM, CPU; Nominatim and
  Overpass are the heavy ones.
- **Update cadence:** weekly/monthly Geofabrik imports; OSRM needs
  re-extract/re-contract.
- **Monitoring/backups** for each service.
- **Attribution discipline** must survive the migration (QA greps +
  `THIRD_PARTY_NOTICES.md`).

Follow-ups:

- Add the env vars to `.env.example` and the deployment guide.
- Provide a Compose profile for the self-hosted stack (tiles/search/routing).
- Replace demo endpoints in CI/e2e with the self-hosted ones once available.
- Keep the QA tile-policy grep and attribution checks green.
