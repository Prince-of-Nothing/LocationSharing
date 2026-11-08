# Third-Party Notices & Licensing — Pinpoint Web Client

Companion to [mobile/THIRD_PARTY_NOTICES.md](../mobile/THIRD_PARTY_NOTICES.md)
(same OSM obligations; SDK tables differ per platform).

## Privacy stance (0 trackers)

The client ships **zero third-party JavaScript** — no npm dependencies, no
bundler, no CDN scripts, no fonts, no analytics. The browser only ever
contacts:

1. **Our own origin** (API via same-origin `/api/` proxy; session cookies
   are first-party, HttpOnly, SameSite=Lax).
2. **Configured raster-tile hosts** (CARTO basemaps + Wikimedia — see
   below). Tiles are `<img>` requests; no user identifiers are attached.
3. **User-initiated OSM services** (only on explicit tap/type):
   - `nominatim.openstreetmap.org` — search / reverse / lookup.
   - `router.project-osrm.org` — driving/foot/bike routing (demo).
   - `overpass-api.de` — POI browse around the visible centre.
   Queries contain only what the user typed (search text) or coordinates
   the user explicitly chose (route endpoints, POI centre). Friend/shared
   locations are **never** sent to any of them. There is no background
   tracking, no autoplay, no telemetry.

The nginx CSP makes this structural: `default-src 'self'` with only the
tile hosts in `img-src` and the three OSM services in `connect-src`. A
future patch that tried to load a tracker would be blocked by the browser,
not just by review.

## Map data: OpenStreetMap (ODbL)

- **Data**: © OpenStreetMap contributors, [ODbL v1.0](https://opendatacommons.org/licenses/odbl/1-0/).
- **Attribution**: rendered permanently under the map (`#map-attribution`,
  populated from `config.js` `tileLayers.*.attribution`). Removing it
  violates the license; the string must stay visible on every map view.
- **Share-alike**: applies if we redistribute the database (e.g. bundle
  tiles). End-user downloads from a provider are governed by that
  provider's terms.
- **Reference**: OSM wiki home is
  https://wiki.openstreetmap.org/wiki/Main_Page (project reference for
  data model, policies and community conventions).

## Tile sources (raster layers)

Default (dark): CARTO dark_all `https://a.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}.png` —
keyless, OSM-derived, welcomes app use. Alternatives (layer switcher):
CARTO light_all / voyager, Wikimedia OSM Standard
`https://maps.wikimedia.org/osm-intl/{z}/{x}/{y}.png`.

- **Never** `tile.openstreetmap.org`: OSMF
  [Tile Usage Policy](https://osmfoundation.org/wiki/Tile_Usage_Policy)
  prohibits serving apps from that infrastructure.
- **Production**: self-host from Geofabrik extracts (Martin / TileServer GL)
  — one line in `js/config.js` (`tileLayers.*.url`) + the CSP `img-src`
  allow-list. See `docs/sdlc/07-openstreetmap-integration.md`.
- **Python-served client**: `backend/web/app.js` (Leaflet) uses the same
  endpoint, same attribution; its CSP is in
  `backend/app/core/middleware.py`.

## Search: Nominatim (OSMF)

- **Endpoint**: `https://nominatim.openstreetmap.org/search|reverse|lookup`
  — keyless OSM geocoding (GPLv2 software; data ODbL).
- **Usage policy** (https://operations.osmfoundation.org/policies/nominatim/):
  max 1 req/s per user, valid referer, aggressive caching, no bulk.
  The client debounces input (350 ms), throttles to 1 req/s in-memory,
  caches results per session, and never scrapes. Production must run a
  self-hosted Nominatim (e.g. `nominatim-docker`) over OSM extracts.
- **Attribution**: “© OpenStreetMap contributors (Nominatim / OSMF)” —
  shown alongside search results / in the map UI.

## Routing: OSRM (demo)

- **Endpoint**: `https://router.project-osrm.org/route/v1/{driving,foot,bike}/…`
  — OSM-derived routing, **demo server only**, not for production load.
- **Production**: self-host OSRM (or Valhalla/GraphHopper, both OSM-data
  compatible) over Geofabrik planet extracts. Routes are user-initiated
  only; no history is stored or sent anywhere.
- **Attribution**: “© OpenStreetMap contributors, routing by OSRM”.

## POI: Overpass API

- **Endpoint**: `https://overpass-api.de/api/interpreter` — read-only
  OSM database access (AGPLv3 software; data ODbL). One query per tap,
  1.5 km radius (max 5 km), 60 results max, 25 s timeout. The client never
  bulk-downloads; production should self-host Overpass or mirror OSM data
  to the Pinpoint API instead of querying the public instance.
- **Attribution**: “© OpenStreetMap contributors via Overpass API”.

## JS libraries and assets

| Component | Version | License | Notes |
|---|---|---|---|
| (none) | — | — | The Go-stack web client has **no JS dependencies**; all code is original, covered by the repo license. |
| Leaflet (Python-served client only, `backend/web/vendor/leaflet/`) | 1.9.4 | **BSD-2-Clause** | Vendored with hash-pinned provenance in `backend/web/vendor/README.md`; attribution control kept enabled (ODbL). |
| PWA manifest / icons | — | — | Data-URI emoji icon, no binary assets. |

## Offline scope (honest limits)

The service worker caches the **app shell only** (HTML/CSS/JS). Map tiles,
search, routing and POI responses are *not* cached offline — bulk tile
caching would violate provider terms of service and store location data on
disk. Live locations always require connectivity; the UI shows an explicit
offline banner instead of stale positions.

Exception: **bookmarks** (user-saved places, manually starred) persist in
`localStorage` on this device only so they survive offline. They are not
synced, not sent anywhere, and can be exported as GeoJSON for
OsmAnd/Organic Maps/Magic Earth. Friend/shared locations are never written
to localStorage, Cache Storage or IndexedDB.

(Offline map packs are a mobile-only feature — see the Android app.)

## Data protection posture

- Session: HttpOnly cookie + double-submit CSRF (token readable from the
  non-HttpOnly companion cookie by design, rotated via login/session
  bootstrap).
- No friend/shared location ever written to Cache Storage, localStorage,
  or IndexedDB — state is in-memory only and dies with the tab.
- Bookmarks only store manually-saved places (name + coordinates + note);
  cleared on demand.
- CSP + `Referrer-Policy: no-referrer` + `X-Frame-Options: DENY` +
  `Permissions-Policy` (geolocation self-only; camera/mic/payment blocked)
  set in `web/nginx.conf` (and the Python middleware for the alternate
  stack).
