# 07 — OpenStreetMap Integration Guide

Reference: https://wiki.openstreetmap.org/wiki/Main_Page (project home),
Magic Earth https://wiki.openstreetmap.org/wiki/Magic_Earth,
Organic Maps https://organicmaps.app/.

## 7.1 Data license (ODbL) — non-negotiable

- Data © OpenStreetMap contributors, ODbL v1.0.
- **Attribution must be permanent & visible** on every map view:
  - Web: `#map-attribution` from `js/config.js` `tileLayers.*.attribution`.
  - Android: MapLibre logo+attribution enabled + “© OpenStreetMap contributors”
    text in `MapScreen.kt`.
  - Rust: “Map data © OpenStreetMap contributors (ODbL)” in `src/map.rs`.
- Share-alike applies to redistributed databases (bundled tiles). End-user
  downloads are governed by provider terms.

## 7.2 Tiles (never `tile.openstreetmap.org`)

| Client | Provider | URL | Notes |
|--------|----------|-----|-------|
| Web (default) | CARTO dark_all | `https://a.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}.png` | keyless, app-friendly |
| Web (layers) | CARTO light_all, voyager; Wikimedia OSM | `config.js tileLayers` | CSP `img-src` allow-listed |
| Mobile | OpenFreeMap vector | `https://tiles.openfreemap.org/styles/{liberty,bright,positron}` | no keys/telemetry |

- **Why not `tile.openstreetmap.org`?** OSMF Tile Usage Policy prohibits
  app/bulk use of that infrastructure. QA greps for it (blocker).
- **Production:** self-host Martin/TileServer GL over Geofabrik extracts
  (ODbL-preserving). Change `config.js` / `MapConfig` + CSP in one PR.

## 7.3 Search — Nominatim

- **Endpoint:** `https://nominatim.openstreetmap.org/search|reverse|lookup`
- **Policy:** ≤1 req/s/user, valid referer, cache aggressively, no bulk.
- **Client rules:** debounce 350 ms, ≥3 chars, viewbox bias, 8 results max,
  per-session cache, never send friend coordinates.
- **Production:** self-host `nominatim-docker` over OSM extracts; point
  `config.js osm.nominatim.base` / `OsmEndpoints.NOMINATIM` at it.
- **Attribution:** “© OpenStreetMap contributors (Nominatim / OSMF)”.

## 7.4 Routing — OSRM (demo) → Valhalla/GraphHopper (prod)

- **Demo:** `https://router.project-osrm.org/route/v1/{driving,foot,bike}/{lon,lat};…`
  with `overview=full&geometries=polyline&steps=true`.
- **Policy:** demo server only; user-initiated single routes; no history.
- **Production:** self-host OSRM (or Valhalla BSD-3 / GraphHopper Apache-2.0,
  both OSM-compatible) over Geofabrik extracts.
- **Client:** `js/routing.js` (polyline decode, steps) / `OsmServices.kt`
  (`OsrmRoutingEngine`); mobile keeps straight-line stub as offline fallback.
  Navigation hand-off is `geo:` intents (OsmAnd/Organic Maps/Magic Earth).

## 7.5 POI — Overpass API

- **Endpoint:** `https://overpass-api.de/api/interpreter`
- **Policy:** one QL query per tap, 1.5 km radius (cap 5 km), 60 max,
  25 s timeout. No bulk, no background.
- **Categories:** food, fuel, pharmacy, health, stay, sights, water,
  parking, parks (mirrored web/mobile).
- **Production:** self-host Overpass or mirror OSM data to Pinpoint API.

## 7.6 Hand-off links

- `geo:lat,lon?q=label` (Android), `geo:` (web `href`), OSM directions
  `https://www.openstreetmap.org/directions?to=…`, OSM place
  `https://www.openstreetmap.org/#map=…`, Organic Maps
  `https://organicmaps.app/?ll=…`, GPX/GPX-Lite + GeoJSON export for
  OsmAnd/Organic Maps/Magic Earth import.

## 7.7 Privacy envelope (all three services)

- Queries contain **only** user-typed text or user-chosen coordinates.
- Friend/shared locations, precision-downgraded or not, **never** leave the
  Pinpoint API except to authorized recipients.
- Results live in memory/session cache only (web) / not persisted as location
  history (mobile bookmarks store only manually-starred places).

## 7.8 Rate-limit & quota checklist (pre-launch)

- [ ] Tile host allows demo load; CSP matches `config.js` layers.
- [ ] Nominatim: referer + 1 req/s verified; cache size bounded.
- [ ] OSRM demo replaced by self-host if traffic > demo tolerance.
- [ ] Overpass: single-query UX verified; timeout/abort handled.
- [ ] Attribution strings reviewed against notices + QA greps green.
- [ ] Production checklist in 12-deployment signed off.

## 7.9 Self-hosted services (production default — ADR-0011)

The public endpoints above are **demo defaults only**. Production must point
these env vars at self-hosted services (guide:
[`docs/maps/self-hosted-osm.md`](../maps/self-hosted-osm.md)):

| Env var | Service | Code |
|---|---|---|
| `PINPOINT_NOMINATIM_URL` | Nominatim search/reverse | `src/osm.rs` (`nominatim_base`) |
| `PINPOINT_OSRM_URL` | OSRM routing | `src/osm.rs` (`osrm_base`) |
| `PINPOINT_OVERPASS_URL` | Overpass POI | `src/osm.rs` (`overpass_url`) |
| `PINPOINT_TILE_URL` | Tile template (`…/{z}/{x}/{y}.png`) | `src/tilepacks.rs`, `src/server/mod.rs` |

Rules:

- Self-hosted first; public providers are fallbacks for local development.
- **ODbL attribution stays** on every map view and tile response, even when we
  host the data (`openstreetmap.org/copyright`).
- Never `tile.openstreetmap.org`; never silently fall back to public endpoints
  in production.
- Self-hosting removes OSMF/demo rate limits and keeps query metadata on our
  infrastructure; friend/shared coordinates still never leave Pinpoint.

The stack: `mediagis/nominatim`, `osrm/osrm-backend`, `maptiler/tileserver-gl`
(or `martin`/`planetiler`), `wiktorn/overpass-api`, all over Geofabrik
extracts. See `docs/maps/self-hosted-osm.md` for sizing, cadence and Compose.
