# Pinpoint v1.1 — Navigation & Map-First Web UI (Rust only)

Date: 2026-11-13 · Status: **Proposed** (awaiting approval) ·
Scope owner: Pinpoint thesis demo

Related: ADR-0007 (Rust multiplatform), ADR-0011 (self-hosted OSM),
ADR-0012 (opaque sessions), `docs/security/session-management.md`,
`docs/maps/self-hosted-osm.md`, `RELEASE_NOTES.md`.

Product references for interaction patterns and publicly documented
features: Organic Maps, OsmAnd, Magic Earth, Photon, Valhalla.

---

## 1. Intent and why

1.0.0 made the web client functional (auth, map, friends, shares, sessions)
but it cannot answer the two questions a map app exists for: *where is X?*
and *how do I get there?*. The Rust OSM engine (`src/osm.rs`) already does
search, reverse geocoding, routing and POIs — only the desktop UI uses it.
The web UI also looks like a dashboard with a map card, not a map.

v1.1 turns the web client into a map-first navigation surface and exposes the
existing OSM engine through same-origin endpoints. Kotlin is archived; all
features are built once in Rust (ADR-0007).

## 2. Goals

- **Place search**: type-ahead, location-biased, category shortcuts.
- **Place sheet**: details and actions (Directions, Save, Copy, OSM,
  What's here?).
- **A→B routing**: drive/walk/bike, up to 3 alternatives, ETA/distance,
  turn list, route drawn and fit to bounds.
- **Map interaction**: click to set a point, explicit locate-me, scale,
  readable dark basemap.
- **Saved places + recent searches**: on-device only, clearable.
- **UI**: map-first layout, floating panels, responsive bottom sheet,
  keyboard accessible, local-only assets.
- Keep the whole flow same-origin: browser → Pinpoint server → OSM services.

## 3. Non-goals (v1.1)

- Turn-by-turn voice guidance, lane guidance, speed-limit warnings.
- Offline routing / offline search (offline tiles already exist in desktop).
- Traffic, public transport, 3D/vector maps, satellite imagery.
- Multi-stop optimisation, elevation profiles, isochrones.
- Android/iOS builds (scaffolds stay), Kotlin app (archived).

## 4. Anti-goals

- Copying proprietary code, assets, icons or pixel-identical screens from
  any commercial app.
- Google Maps/Mapbox or any keyed third-party SDK.
- Sending friend coordinates or user identity to OSM services.
- Tracking, analytics, raw IP storage, IP/WebRTC geolocation.
- A build step or CDN: vendored Leaflet only, CSP stays `'self'`.

## 5. Constraints

- Browser assets must be local (`tests/assets.rs`, strict CSP, no inline
  styles/scripts, `font-src 'self'`).
- OSM calls are user-initiated only; Nominatim/OSRM public endpoints are demo
  fallbacks, self-hosted URLs win (`PINPOINT_NOMINATIM_URL`,
  `PINPOINT_OSRM_URL`).
- Courtesy limits: debounce search, cache, one route request at a time,
  identify with `Pinpoint/<version>` User-Agent.
- ODbL attribution visible on every map view.
- No schema changes; no new database table.

## 6. Approaches considered

| Option | Why not / why |
|---|---|
| Extend the hand-rolled 5×3 tile grid | Re-implements pan/pinch/popups/fit-bounds; current drag already rebuilds 15 images per pointer move. Rejected. |
| **Vendor Leaflet locally (chosen)** | Mature interactions, polylines, markers, popups, fit-bounds; BSD-2; ~150 KB; same-origin; CSP-compatible; no build step. |
| MapLibre GL + vector tiles | Best-looking and smooth, but needs a self-hosted vector style/tile pipeline we do not have; ~600 KB+; deferred to v1.2+. |

## 7. UX

### 7.1 Layout (desktop)

```
┌──────────────────────────────────────────────────────────┐
│ Pinpoint   [ Search a place…                    ]  ⌖  ⊕ │
│ ┌────────────────┐                                       │
│ │ Route          │                                       │
│ │ ● My location  │            MAP                        │
│ │ ○ Destination  │      ━━━ mint route                   │
│ │ ⇄  🚗 🚶 🚲    │      ● A     ○ B                      │
│ │ 4.2 km · 9 min │                                       │
│ │ Steps ▾        │                                       │
│ └────────────────┘                     © OSM · z13  ▭   │
└──────────────────────────────────────────────────────────┘
```

Mobile (<720 px): full-screen map, search bar on top, route/place panels as a
draggable bottom sheet; controls above the sheet.

### 7.2 States

1. **Idle map** — search bar, locate button, attribution, scale.
2. **Searching** — results sheet under the bar (max 8), each row: name,
   category icon, address, distance; arrow keys + Enter; Esc closes.
3. **Place selected** — place sheet with actions.
4. **Route planning** — A/B fields, swap, mode tabs, alternatives chips,
   summary, collapsible steps.
5. **Route preview** — route fit to bounds, start/end markers, step list;
   "Start" enters a lightweight follow mode (watchPosition + next-step card),
   no voice.
6. **Empty/error** — "No places found", "Routing unavailable — try again",
   geolocation denied; never a blank map.

### 7.3 Interaction rules

- Search: 300 ms debounce, ≥3 chars, `AbortController` cancels stale queries,
  bias by current map centre, max 8 results.
- Map click: sets destination when the route panel is open; otherwise opens
  "What's here?" (reverse geocode) place sheet.
- Long-press/right-click: same as click.
- Locate: only after an explicit button press; no automatic geolocation.
- Swap A/B, clear route, Esc closes the top sheet.
- Keyboard: `/` focuses search, ↑/↓ navigate results, Enter selects, Esc
  closes; visible focus rings.
- Reduced motion: no fly animation when `prefers-reduced-motion`.

### 7.4 Visual language (frontend-design)

- Deep navy base `#0B1020`, panel `#0F172A`, hairline `#1F2937`, text
  `#E5E7EB`, muted `#9CA3AF`.
- Mint `#34D399` for route/primary, amber `#FBBF24` for cautions, red
  `#F87171` for destructive/live.
- System UI font; coordinates and distances in `ui-monospace`.
- Markers: initial-letter discs with a soft pulse for self, outlined pin for
  destination, numbered small discs for alternatives.
- Tiles: OSM raster with a tuned dark treatment (CSS filter) until a
  self-hosted vector style exists; ODbL credit always readable.
- No icon font, no CDN; inline SVG sprite (local) for search, locate, swap,
  car/walk/bike, save, copy, close.

## 8. Backend API (new, same origin)

All under the existing map router; JSON; no auth required (public map data),
but rate-limited and privacy-preserving (no user id, no IP in keys).

### `GET /map/search?q=&lat=&lon=&limit=`

- `q` 2..=120 chars; `lat/lon` optional bias; `limit` 1..=10 (default 8).
- → `{ "results": [ { "display_name", "short_name", "latitude",
  "longitude", "category", "place_type", "distance_m" } ] }`
- 400 on invalid query/coords; 502 `OSM_UNAVAILABLE` on upstream failure.

### `GET /map/reverse?lat=&lon=`

- → one place object (same shape), 404 `NOT_FOUND` if nothing.

### `GET /map/route?from_lat=&from_lon=&to_lat=&to_lon=&mode=&alternatives=`

- `mode` ∈ `driving|foot|bike`; `alternatives` bool (default true).
- → `{ "routes": [ { "distance_m", "duration_s", "summary",
  "points": [[lat,lon],…], "steps": [{ "text", "distance_m",
  "duration_s" }] } ] }`
- 400 on invalid coords (range, NaN) or mode; 502 upstream; 404 no route.

Implementation: thin wrappers call `osm::OsmClient` (`search`, `reverse`,
`route`); extend `route` to request `alternatives=true` and return a Vec.
Cache: in-process LRU with TTL (search 60 s, reverse 10 min, route 5 min),
keyed only by query/rounded coords/mode; cap 512 entries. Serialise route
requests with a semaphore (max 2 concurrent) and return 429 `BUSY` if the
queue is full.

## 9. Frontend architecture

- Keep the no-build plain-JS app, but split into ES modules served
  same-origin: `api.js` (fetch + CSRF), `map.js` (Leaflet setup, dark
  treatment, markers, route layer), `nav.js` (search/route state),
  `ui.js` (sheets, toasts, a11y), `app.js` (bootstrap + auth screens).
- Vendor Leaflet 1.9.x (`leaflet.js`, `leaflet.css`, LICENSE) under
  `web/app/vendor/`; use `L.divIcon` markers so no marker images are needed.
- Server gains routes for the new modules and vendor files; `include_str!`
  keeps the single-binary deployment.
- State: `{ map, search, selectedPlace, route, saved[], recent[] }`;
  localStorage keys `pinpoint.savedPlaces.v1`, `pinpoint.recentSearches.v1`.
- Failure UX: tile error overlay, OSM error toast with retry, geolocation
  denial hint; never remove the map.
- Attribution: Leaflet attribution control pinned with
  `© OpenStreetMap contributors` linking to the copyright page.

## 10. Privacy & security

- Search/route requests contain only the user's query/coords; no session id,
  no friend data. Server strips nothing because it never receives identity
  on these routes.
- Saved/recent data never leaves the browser; a "Clear history" action.
- No IP logging; existing middleware unchanged.
- CSP unchanged (`connect-src 'self'`), so the browser cannot call Nominatim
  or OSRM directly.
- Leaflet vendored and license recorded; `tests/assets.rs` extended to assert
  no CDN URL and that the vendor license file exists.

## 11. Testing & acceptance

Automatable:
- Router probes: `/map/search` and `/map/route` reject bad input (400) without
  network; valid requests return 502 (not panic) when upstream is unreachable.
- Unit: route JSON mapping, alternatives, coordinate validation, cache keys.
- Assets: no third-party hosts; Leaflet files local; CSP still strict.
- Full gate: fmt, clippy -D warnings, build, tests.

Manual/browser:
- Search "pharmacy" near Chișinău returns results; selecting one flies there.
- Route My location → result: polyline, distance/time, mode switch, steps.
- Map click sets B; swap works; alternatives selectable.
- 390 px layout uses the bottom sheet; keyboard flow works.
- Network log shows only `127.0.0.1` requests.

## 12. Delivery slices

1. **Endpoints + tests** (`/map/search`, `/reverse`, `/route`, cache,
   alternatives) — no UI.
2. **Leaflet shell** — vendor + dark map + markers + attribution; remove the
   custom grid renderer.
3. **Search + place sheet** — type-ahead, categories, reverse "What's here?".
4. **Routing** — A/B, modes, alternatives, steps, fit-bounds.
5. **Saved/recent/locate + polish** — localStorage, keyboard, mobile sheet.
6. **Docs/release** — CHANGELOG, ROADMAP, `RELEASE_NOTES` 1.1, browser
   verification; version 1.1.0.

Each slice ends green; no slice leaves the UI worse than before.

## 13. Risks & mitigations

- **Public OSRM/Nominatim throttling** → cache, debounce, semaphore,
  self-host env; clear error copy.
- **Leaflet under strict CSP** → CSS-only stylesheet, `divIcon`, no inline
  script; verify in browser before merging.
- **Dark raster treatment looks muddy** → tune filter per zoom, keep
  attribution readable; vector style deferred.
- **Scope creep** → non-goals list; saved/recents capped at 50 entries.
- **Third-party inspiration** → original assets and interaction design; no
  third-party names or logos in the UI.

## 14. Decisions left to the builder

- Exact Leaflet patch version and whether to strip unused control code.
- Precise cache eviction policy beyond LRU+TTL.
- Whether to keep a compatibility shim for the old tile grid during slice 2.
- Icon shapes within the stated visual language.

## 16. Builder-review resolutions (2026-11-13)

The eight questions below were raised against the first draft; these are the
binding decisions. Where they conflict with §7–§12, this section wins.

1. **Keep the 1.0 surfaces.** Friends/requests, shares, friend-location list,
   publish-location and active sessions move into a right-hand drawer; friend
   markers stay on the Leaflet map (`setFriendMarkers`). No 1.0 feature is
   dropped. The drawer is part of slice 2, not slice 6, so the app is never
   worse between slices.
2. **No public-Nominatim autocomplete.** Nominatim's usage policy forbids
   client-side autocomplete. Search is **submit-on-Enter/button by default**;
   type-ahead (300 ms debounce) is enabled only when
   `PINPOINT_SEARCH_AUTOCOMPLETE=1` is set for a self-hosted Nominatim.
   `GET /map/capabilities` returns `{"autocomplete":bool,"modes":["driving"]}`
   so the UI can adapt. The server still throttles and validates every call.
3. **Walk/bike modes are real only if configured.** The public OSRM demo has
   only a car profile. Add optional `PINPOINT_OSRM_FOOT_URL` and
   `PINPOINT_OSRM_BIKE_URL`; `/map/capabilities.modes` lists the modes whose
   URL is configured (`driving` always). Unconfigured tabs are disabled with
   “Self-host foot/bike routing to enable this mode.” No cosmetic modes.
4. **A is selectable.** Origin picker: My location (session’s published
   location), Map centre, or a searched/clicked place. Default = published
   location if available, else map centre. If geolocation is denied, routing
   still works from map centre and the UI says so. Map clicks set B while the
   route panel is open; an explicit “Set as origin” action sets A.
5. **Category chips are searches, not POI pins.** A chip runs
   `/map/search?q=pharmacy&lat=…&lon=…` and shows the normal results list.
   Overpass POIs remain desktop-only in v1.1 (no fourth endpoint).
6. **Saved places have a surface.** Drawer section “Saved places”: list, click
   to show on map, “Directions” to set B, delete per item. “Clear history”
   clears recents only; “Clear saved” is separate. Saved records store name,
   coordinates, category and address.
7. **Storage is per account.** Keys are
   `pinpoint.v1.<userId>.savedPlaces` and `pinpoint.v1.<userId>.recents`;
   anonymous accounts use their numeric user id too. Logout leaves the data on
   the device but the next account does not see it; “Clear saved/history”
   affects the current account only.
8. **Redact query strings in server logs.** `converge` logs
   `METHOD path` only; for `/map/*` requests the query string is dropped
   before `eprintln!` so coordinates/queries never reach stderr. Browser
   history is unaffected because fetches do not change the address bar.
   A unit test pins the redaction helper.

`/map/capabilities` is added to the open (unauthenticated) map router.


