# 02 — Scope and Assumptions

## In scope (this release)

- **Sharing core (unchanged):** accounts, sessions, friendships
  (request/accept/reject), directional shares (precision/purpose/expiry,
  pause/resume), publish location, friend locations, freshness, admin.
- **Web map parity:** interactive Slippy map, 4 raster layers, search,
  routing, POI browse, bookmarks, navigation hand-off, GPX/GeoJSON export.
- **Mobile parity:** MapLibre vector map, offline packs, search, routing,
  POI browse, bookmarks, 3 styles, `geo:` hand-off.
- **Desktop parity:** correct OSM links (standard/directions/Organic Maps),
  ODbL credit, no proprietary deep links.
- **Documentation:** full SDLC set (`docs/sdlc/`), UML additions
  (`docs/uml/12-15`), OSM integration guide, QA pipeline docs.
- **Automatic QA:** `scripts/qa.sh` (+ `.ps1`), CI map/compliance jobs.

## Out of scope

- Backend API changes to consent semantics (none needed; map features are
  client-side + OSM services).
- Self-hosted tile/routing/search infrastructure (documented as the
  production path, not shipped in this release).
- Background location tracking, live trails, geofences, check-ins
  (scheduled/trip), push notifications, iOS client, native offline routing
  engine (GraphHopper/Valhalla plugin point only).
- Load testing, penetration testing, external security audit.

## Assumptions

| ID | Assumption | If wrong |
|----|------------|----------|
| A-1 | Public OSM services (Nominatim/OSRM demo/Overpass) remain available for demo-scale use | Self-hosting path (07) becomes mandatory before any public traffic |
| A-2 | CARTO + Wikimedia + OpenFreeMap remain keyless and app-friendly | Switch `config.js`/`MapConfig` URLs + CSP; no code redesign |
| A-3 | OSM data licenses (ODbL) remain as published; OSM wiki is canonical | Re-check notices + attribution strings |
| A-4 | Users have HTTPS + GPS-capable devices for web/mobile | Offline shell/bookmarks still work; live data degrades to banner |
| A-5 | Release review is thesis-scale (not commercial GA) | Scaling path (12) applies before growth |

## Dependencies

| Dependency | Owner | Fallback |
|------------|-------|----------|
| Pinpoint Go API (`/api/v1`) | Pinpoint | Python alt API (same contract) |
| CARTO / Wikimedia / OpenFreeMap | respective projects | Self-hosted Martin/TileServer GL over Geofabrik extracts |
| Nominatim / OSRM / Overpass (public) | OSMF / OSRM / Overpass | Self-hosted Nominatim/OSRM/Overpass or proxy |
| Docker Compose + CI runners | project | Single-binary (`WEB_ROOT`) mode |

## Boundaries

- **Client is untrusted.** All consent decisions re-checked server-side.
- **OSM services are untrusted for privacy.** They never see friend data.
- **Provider ToS are hard constraints.** Violation = release blocker.
