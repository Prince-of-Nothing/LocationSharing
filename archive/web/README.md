# Pinpoint — Web Client

Browser client for the Pinpoint API (Go stack; the Python stack has a
separate Leaflet-based client under `backend/web/`). Privacy-first: **zero
third-party JavaScript, zero trackers**, modular ES modules with **no build
step**, offline app shell, OSM-licensed maps with Organic-Maps-style search,
routing and POI browse.

Full lifecycle docs: [docs/sdlc/](../docs/sdlc/) · OSM integration:
[docs/sdlc/07-openstreetmap-integration.md](../docs/sdlc/07-openstreetmap-integration.md).

## Architecture

Plain ES2022 modules loaded natively by the browser — no npm, no bundler, no
supply chain. The dependency tree is: nothing.

```
public/
├── index.html            # static shell (auth + map/friends/shares/admin)
├── app.css               # dark theme, Slippy map, search/route/POI UI
├── manifest.webmanifest  # PWA install metadata (no remote icons)
├── sw.js                 # service worker: offline shell (never caches data)
└── js/
    ├── config.js         # tile layers + OSM endpoints (Nominatim/OSRM/Overpass)
    ├── api.js            # ONLY module talking to /api/v1 (CSRF, errors)
    ├── session.js        # login/logout/session-restore (cookie bootstrap)
    ├── ui.js             # DOM helpers, toast, offline banner
    ├── map.js            # interactive Slippy engine + overlays + ODbL credit
    ├── geocode.js        # Nominatim search/reverse/lookup (throttled+cached)
    ├── routing.js        # OSRM routing + polyline + steps
    ├── places.js         # Overpass POI browse (9 categories)
    ├── bookmarks.js      # on-device saved places (localStorage, GeoJSON export)
    ├── navigation.js     # geo:/OSM hand-off + GPX export
    └── main.js           # tab wiring, search/route/POI/bookmarks, boot
```

Module boundaries (by convention, enforced in review):

- `api.js` owns all `fetch` to `/api/v1` — features never build URLs or headers.
- `map.js` owns all tile/attribution/overlay logic — the compliance-sensitive part.
- `geocode.js`/`routing.js`/`places.js` own OSM-service calls
  (user-initiated only; friend locations never sent).
- Everything that renders server data goes through `esc()`.

## Running

Served by nginx in the full stack:

```bash
docker compose up --build   # web on http://localhost:8080
```

Local dev without Docker: serve `public/` with any static server and proxy
`/api/` to a running Go backend (`cd backend && go run ./cmd/server`).

## Features

- **Login + session restore**: HttpOnly session cookie survives reloads;
  `POST /auth/session` re-validates it on boot and rotates CSRF.
- **Interactive map**: Slippy pan/zoom (pointer/wheel/pinch/keys), 4 layers
  (CARTO dark/light/voyager, Wikimedia OSM), scale/coords, my-location.
- **Search**: Nominatim (debounced, cached, viewbox-biased) + tap-to-reverse.
- **Routing**: OSRM driving/foot/bike, steps, polyline, GPX export.
- **Places**: Overpass browse around centre (9 cats), detail + save/route.
- **Bookmarks**: local-only saved places + GeoJSON export (OsmAnd/Organic Maps).
- **Friends / Sharing / Admin / Password change**: as before.
- **Offline shell**: service worker caches the app UI only; live data
  requires connectivity (banner). Bookmarks work offline.

## Map tiles & licensing

One line per layer in `js/config.js` (`tileLayers.*.url`). Never point them
at `tile.openstreetmap.org` (OSMF Tile Usage Policy prohibits apps).
Production should self-host tiles (Martin / TileServer GL). ODbL attribution
under the map is a legal requirement. Full obligations:
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Security headers

`nginx.conf` CSP: `default-src 'self'` with tile hosts in `img-src` and
Nominatim/OSRM/Overpass in `connect-src`. If you change `tiles.url`, update
`img-src` to match — the browser will silently blank the map otherwise.

## Automatic QA

```bash
./scripts/qa.sh        # tile policy, attribution, CSP, SHELL, syntax, Mermaid, notices
./scripts/qa.ps1       # Windows mirror
```

Also runs as CI job `map-qa` (`.github/workflows/map-qa.yml`).
