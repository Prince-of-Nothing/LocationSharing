# 06 — Vanilla-JS Web Client (archived)

> **Status:** 🗄️ **Archived** (ADR-0007). Not built or shipped. Kept for
> design intent. Dir: `archive/web/`.

## Overview

The original Pinpoint web client: a zero-build, same-origin PWA written in
vanilla JavaScript with a vendored Leaflet map. It authenticates with the
session cookie + CSRF double-submit flow, renders friend locations on an OSM
map, and supports search, routing, POI, bookmarks and navigation hand-off.
Superseded by the Rust/WASM web stubs in `web/`.

## Stack

| Concern | Choice |
|---|---|
| Language | Vanilla JavaScript (ES modules) |
| Map | Leaflet (vendored under `web/vendor/leaflet`) |
| Tiles | CARTO / Wikimedia (ODbL-compliant) |
| PWA | `manifest.webmanifest` + `sw.js` service worker |
| Serving | nginx (`nginx.conf`) reverse proxy to `/api` |
| Build | none (static files) |

## Module map

```
archive/web/
├── public/
│   ├── index.html            # app shell
│   ├── app.css               # styling
│   ├── manifest.webmanifest  # PWA manifest
│   ├── sw.js                 # service worker (offline shell + SHELL list)
│   ├── vercel.json           # deployment config
│   └── js/
│       ├── main.js           # bootstrap + screen router
│       ├── api.js            # fetch wrapper: cookie + X-CSRF-Token
│       ├── session.js        # login/logout/session restore
│       ├── map.js            # Leaflet map + friend pins + layers
│       ├── geocode.js        # Nominatim search / reverse
│       ├── routing.js        # OSRM routing + GPX export
│       ├── places.js         # Overpass POI browse
│       ├── navigation.js     # geo: / OSM hand-off
│       ├── bookmarks.js      # localStorage bookmarks + GeoJSON export
│       ├── config.js         # tile layers, CSP-safe endpoints
│       └── ui.js             # shared DOM helpers
└── vendor/leaflet/           # vendored Leaflet assets
```

## Features

- Login / logout / session restore with CSRF double-submit.
- Leaflet map with friend pins, freshness badges, precision badges.
- Layer switcher (CARTO dark/light/voyager, Wikimedia OSM).
- Nominatim search + reverse geocode.
- OSRM routing (driving/foot/bike) with GPX export.
- Overpass POI browse (9 categories).
- Bookmarks in `localStorage` with GeoJSON export.
- Navigation hand-off (`geo:` + OSM links).
- Offline app shell via service worker.

## Security controls

- **CSRF:** double-submit token on state-changing requests (`api.js`).
- **Cookies:** session cookie `HttpOnly`/`Secure`/`SameSite` (set by backend).
- **CSP:** `nginx.conf` restricts `img-src`/`connect-src` to allow-listed OSM
  providers; no third-party JS SDKs.
- **Privacy:** friend coordinates never sent to OSM services; only
  user-initiated queries.
- **XSS:** output encoding in `ui.js`; no innerHTML with user data.

## Storage

- **Bookmarks:** `localStorage` (on-device only, no sync).
- **Session:** cookie (server-side session); no token in JS storage.

## Build / run

```bash
# static files only — serve with any static server
cd archive/web/public
python -m http.server 8080
# or via the archived nginx container (archive/docker-compose.yml)
```

## Deployment

- nginx config: `archive/web/nginx.conf` (SPA + `/api` reverse proxy).
- `vercel.json` for static hosting.
- Compose: `archive/docker-compose.yml` (web + Go/Python API + Postgres).

## Relationship to other components

- **Consumed:** component 1 (Go) / component 2 (Python) backends.
- **Superseded by:** component 4 (Rust client) and the `web/` wasm stubs.
- **Design intent reused:** Slippy map interactions, layer switcher, OSM
  hand-off, bookmarks GeoJSON — all preserved in the Rust client.
