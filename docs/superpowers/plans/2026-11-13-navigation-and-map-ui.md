# Pinpoint v1.1 Navigation & Map-First Web UI — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add place search, reverse geocoding and A→B routing to the web client, and rebuild it as a map-first UI with locally vendored Leaflet.

**Architecture:** Three same-origin JSON endpoints wrap the existing `src/osm.rs` engine (Nominatim + OSRM), with validation, caching and a route semaphore. The web client keeps its no-build plain-JS approach but splits into ES modules; Leaflet 1.9.4 is vendored under `web/app/vendor/` and restyled. No database, auth or consent behaviour changes.

**Tech Stack:** Rust/Axum 0.7, reqwest, serde_json, Tokio (`sync` feature), Leaflet 1.9.4 (BSD-2), plain ES modules, CSS.

**Spec:** `docs/superpowers/specs/2026-11-13-navigation-and-map-ui-design.md`

## Global Constraints

- Rust is the only client codebase (ADR-0007). Do not touch `mobile/` or add Kotlin.
- Browser assets are local only: no CDN, no Google Fonts, no remote `@import`/`url()`; CSP stays `'self'` (`tests/assets.rs` must stay green).
- OSM services are user-initiated only; self-hosted env URLs (`PINPOINT_NOMINATIM_URL`, `PINPOINT_OSRM_URL`) win; public endpoints are demo fallbacks. ODbL attribution always visible.
- Never send session/user/friend data to OSM endpoints; map endpoints are unauthenticated and cache keys contain no identity.
- Leaflet is vendored at version 1.9.4 with its BSD-2 license file; use `L.divIcon` so no marker images are needed.
- No inline scripts/styles (CSP), no build step, no new database migration.
- `git` is not installed on this machine: skip commit steps and leave the tree ready for the owner to commit.
- Full gate after each task: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`, `cargo test --locked`.

## Review Focus

1. **OSM upstream unavailable/timeout** — search/route must return a clear `502 OSM_UNAVAILABLE` with a retry message, never an empty map or a panic.
2. **Invalid input** (`lat=NaN/999`, bad `mode`, short query, non-numeric limit) — `400` before any upstream call.
3. **Fast typing** — stale responses must not overwrite newer results; aborting in-flight requests is required.
4. **Map click with an open route panel** sets destination B; with no route it opens “What’s here?” — never both.
5. **Corrupt/blocked localStorage or denied geolocation** — app still boots and the map still works.

## Builder-review resolutions (binding)

`§16` of the spec overrides the task text where they differ:

1. Keep all 1.0 surfaces in a drawer + friend markers on the Leaflet map from slice 2.
2. Search is submit-on-Enter by default; type-ahead only with `PINPOINT_SEARCH_AUTOCOMPLETE=1`. Add `GET /map/capabilities`.
3. Walk/bike are shown only when `PINPOINT_OSRM_FOOT_URL` / `PINPOINT_OSRM_BIKE_URL` are configured.
4. Origin A is selectable (my location / map centre / searched place); geolocation denial falls back to map centre.
5. Category chips run text search; no Overpass endpoint in v1.1.
6. Saved places get a drawer panel (show, directions, delete); clear-saved and clear-recents are separate.
7. Saved/recent keys are per account: `pinpoint.v1.<userId>.savedPlaces|recents`.
8. Server logs drop query strings for `/map/*`; unit-test the redaction.

---

### Task 1: OSM proxy endpoints (`/map/search`, `/map/reverse`, `/map/route`)

**Files:**
- Modify: `src/osm.rs` (add `route_alternatives`)
- Modify: `src/server/mod.rs` (`map_router`, `map_handlers`)
- Modify: `Cargo.toml` (Tokio `sync` feature)
- Test: `tests/router_probe.rs`, unit tests in `map_handlers`

**Interfaces:**
- Consumes: `osm::{OsmClient, SearchPlace, Route, TravelMode, nominatim_base, osrm_base}`.
- Produces:
  - `pub async fn route_alternatives(&self, from: (f64, f64), to: (f64, f64), mode: TravelMode) -> anyhow::Result<Vec<Route>>` (first route equals `route()`; empty vec → `anyhow::bail!("No route found")`).
  - `map_handlers::{search, reverse, route}` Axum handlers mounted at `/map/search`, `/map/reverse`, `/map/route`.

- [ ] **Step 1: Write the failing tests**

In `tests/router_probe.rs` add:

```rust
#[tokio::test]
async fn map_search_rejects_short_query() {
    let app = pinpoint_desktop::server::map_router()
        .with_state(pinpoint_desktop::server::AppState::new());
    let resp = app.oneshot(Request::builder().uri("/map/search?q=ab").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn map_route_rejects_invalid_coordinates() {
    let app = pinpoint_desktop::server::map_router()
        .with_state(pinpoint_desktop::server::AppState::new());
    let resp = app.oneshot(Request::builder()
        .uri("/map/route?from_lat=999&from_lon=28.8&to_lat=47.0&to_lon=28.9&mode=driving")
        .body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --locked map_search_rejects_short_query map_route_rejects_invalid_coordinates`
Expected: FAIL (404 / route missing).

- [ ] **Step 3: Extend OSRM routing with alternatives**

In `src/osm.rs` add `route_alternatives`; change the OSRM URL to
`?overview=full&geometries=polyline&steps=true&alternatives=true`, parse
`resp.routes` into a Vec, and keep `route()` delegating to
`route_alternatives(...).await?.into_iter().next()` (bail if empty).
Add a unit test `route_alternatives_builds_url` if the URL builder is factored
into `fn osrm_route_url(...) -> String`.

- [ ] **Step 4: Implement handlers, validation and cache**

In `src/server/mod.rs`:

```rust
#[derive(serde::Deserialize)]
pub struct SearchParams { q: String, lat: Option<f64>, lon: Option<f64>, limit: Option<usize> }
#[derive(serde::Deserialize)]
pub struct ReverseParams { lat: f64, lon: f64 }
#[derive(serde::Deserialize)]
pub struct RouteParams {
    from_lat: f64, from_lon: f64, to_lat: f64, to_lon: f64,
    mode: Option<String>, alternatives: Option<bool>,
}
```

Rules: query trimmed `3..=120`; coordinates finite and in range (`lat -90..=90`,
`lon -180..=180`); limit `1..=10` default 8; mode `driving|foot|bike`
(default `driving`). Invalid input → `400` with
`{"error":{"code":"VALIDATION_FAILED","message":"…"}}`.
Upstream error → `502` `{"error":{"code":"OSM_UNAVAILABLE","message":"Search is unavailable right now. Try again."}}`.

Response shapes (exact keys):

```json
{"results":[{"display_name":"…","short_name":"…","latitude":47.0,"longitude":28.8,"category":"pharmacy","place_type":"…","distance_m":123.4}]}
{"routes":[{"distance_m":4200.0,"duration_s":540.0,"summary":"…","points":[[47.0,28.8],[47.1,28.9]],"steps":[{"text":"Turn right","distance_m":120.0,"duration_s":18.0}]}]}
```

Cache: `static MAP_CACHE: OnceLock<Mutex<HashMap<String,(serde_json::Value,Instant)>>>`,
cap 512, TTL search 60 s / reverse 10 min / route 5 min; key is the normalized
query or rounded coords + mode, never identity. Route concurrency:
`static ROUTE_PERMITS: OnceLock<tokio::sync::Semaphore>` with 2 permits;
`try_acquire` failure → `429 {"error":{"code":"BUSY","message":"Routing is busy, try again."}}`.
Add `"sync"` to the Tokio feature list in `Cargo.toml`.

Mount the three routes in `map_router()`.

- [ ] **Step 5: Add unit tests for pure helpers**

In `map_handlers` tests: `parse_mode("walk")` rejects, `parse_mode("foot")` is
`TravelMode::Foot`; `valid_coords(47.0, 28.8)` is true, `valid_coords(f64::NAN, 0.0)` is false;
`place_json()` contains all seven keys; `upstream_error()` returns status `502`
and code `OSM_UNAVAILABLE`. Run `cargo test --locked map_`.

- [ ] **Step 6: Run the full gate**

Run: `cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo build --locked && cargo test --locked`
Expected: all green; router probes now pass.

---

### Task 2: Vendor Leaflet locally + serve its files

**Files:**
- Create: `web/app/vendor/leaflet.js`, `web/app/vendor/leaflet.css`, `web/app/vendor/images/*.png`, `web/app/vendor/LICENSE`
- Modify: `src/server/web_handlers.rs`, `src/server/mod.rs`
- Test: `tests/assets.rs`

**Interfaces:**
- Produces: `GET /vendor/leaflet.js` (`text/javascript`), `/vendor/leaflet.css` (`text/css`), `/vendor/leaflet/LICENSE` (`text/plain`), `/vendor/images/*` (`image/png`).

- [ ] **Step 1: Download Leaflet 1.9.4 and its license**

Download `leaflet.js`, `leaflet.css`, `images/{layers.png,layers-2x.png,marker-icon.png,marker-icon-2x.png,marker-shadow.png}` and `LICENSE` from
`https://unpkg.com/leaflet@1.9.4/dist/...` into `web/app/vendor/`.
Record the exact version and license in `web/app/vendor/README.md`.

- [ ] **Step 2: Write the failing asset test**

In `tests/assets.rs` add:

```rust
#[tokio::test]
async fn leaflet_is_vendored_and_served() {
    let app = pinpoint_desktop::server::router(pinpoint_desktop::server::AppState::new());
    for path in ["/vendor/leaflet.js", "/vendor/leaflet.css", "/vendor/leaflet/LICENSE"] {
        let resp = app.clone().oneshot(Request::builder().uri(path).body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "{path}");
    }
    let css = include_str!("../web/app/vendor/leaflet.css").to_ascii_lowercase();
    for host in ["cdn.", "unpkg.com", "jsdelivr.net", "https://"] {
        assert!(!css.contains(host), "vendored CSS must not reference {host}");
    }
}
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `cargo test --locked leaflet_is_vendored_and_served`
Expected: FAIL (404).

- [ ] **Step 4: Serve the files**

Add `include_str!`/`include_bytes!` constants and handlers in `web_handlers.rs`
(one helper for JS/CSS with the existing `secure()` headers, a plain-text
license handler, and an image handler setting `image/png` + cache). Mount
`/vendor/leaflet.js`, `/vendor/leaflet.css`, `/vendor/leaflet/LICENSE`,
`/vendor/images/:file` in `router()`. Reject unknown image names with 404.

- [ ] **Step 5: Run the full gate**

Run: `cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo build --locked && cargo test --locked`
Expected: green (asset tests now 6).

---

### Task 3: Map-first shell with Leaflet

**Files:**
- Create: `web/app/map.js`
- Modify: `web/app/index.html`, `web/app/app.css`, `web/app/app.js`, `src/server/web_handlers.rs`, `src/server/mod.rs`
- Test: browser verification (no JS runner)

**Interfaces:**
- Produces:
  - `export function createMap({ element, center, zoom, onMapClick })` returning
    `{ map, setFriendMarkers(locations), setSelectedPlace(place), drawRoute(route), clearRoute(), fitRoute(route), locate() }`.
  - `web/app/app.js` becomes `<script type="module">` and imports `createMap`.

- [ ] **Step 1: Restructure the markup**

In `index.html`: replace the map card with a full-height map region:

```html
<section id="app-panel" class="hidden app-shell">
  <div id="map" class="map" aria-label="Map"></div>
  <header class="map-topbar">…brand, #search-form, #locate…</header>
  <aside id="side-panel" class="panel">…place/route panels…</aside>
  <div id="social-drawer" class="drawer">…publish/friends/shares/sessions…</div>
  <div id="attribution">…ODbL…</div>
</section>
<script src="/vendor/leaflet.js"></script>
<script type="module" src="/app.js"></script>
```

`createMap` initialises `L.map('map', { zoomControl: false, attributionControl: false })`,
adds the existing same-origin tile layer `/map/tiles/{z}/{x}/{y}?v=4`, a
`L.control.attribution` with the ODbL link, `L.control.zoom({ position: 'bottomright' })`
and `L.control.scale({ imperial: false, position: 'bottomleft' })`.
Apply `className: 'tiles-dark'` and the CSS filter from the spec.
Name the returned object `mapApi`.

- [ ] **Step 2: Serve `map.js`**

Add `MAP_JS` include + `/map.js` route in `web_handlers.rs`/`router()` with
`text/javascript` and `secure()` headers.

- [ ] **Step 3: Replace the custom renderer**

Delete the old `renderMap`, tile-grid constants, drag handlers and zoom
handlers from `app.js`; call `createMap` after a successful session check and
pass `setFriendMarkers(state.locations)`. Keep marker labels; use `L.divIcon`
with the initial letter and `marker-self`/`marker-friend` classes.

- [ ] **Step 4: Style the shell**

`app.css`: `.map { position: fixed; inset: 0; }`, overlay topbar with
`backdrop-filter`, panel cards, `.tiles-dark img { filter: … }`, drawer on the
right, responsive bottom sheet under 720 px. Keep the existing palette
variables; add `--min` mint and `--warn` amber if missing.

- [ ] **Step 5: Verify in the browser**

Open `http://127.0.0.1:8080/`, log in as alice, confirm: map fills the
viewport, pan/zoom smooth, 15 tiles 200, friend markers at Alice/Bob, ODbL
visible, only `127.0.0.1` in the network log, zero console errors. Screenshot.

- [ ] **Step 6: Gate**

Run the full gate. Expected: green.

---

### Task 4: Search, results and place sheet

**Files:**
- Create: `web/app/nav.js`
- Modify: `web/app/index.html`, `web/app/app.css`, `web/app/app.js`, `src/server/mod.rs`, `src/server/web_handlers.rs`

**Interfaces:**
- Consumes: `createMap(...)`, `GET /map/search`, `GET /map/reverse`.
- Produces:
  - `export function setupSearch({ mapApi, onSelect, onDirections })` wiring
    the form, 300 ms debounce, `AbortController`, categories chips, results list.
  - Place sheet actions: Directions, Save (Task 6), Copy, Open in OSM, What’s here?.

- [ ] **Step 1: Markup**

Add inside the topbar: `<form id="search-form"><input id="search-input" …></form>`,
`#search-results` list, `#place-sheet` with name/address/distance and action
buttons using inline SVG icons. Add category chips (food, cafe, fuel,
pharmacy, hospital, parking, atm, hotel, park).

- [ ] **Step 2: Implement search client**

`nav.js`: `searchPlaces(query, {lat, lon})` fetches
`/map/search?q=…&lat=…&lon=…&limit=8`; abort previous request; render rows with
`display_name`, distance; click → `onSelect(place)`; Enter selects first;
Escape closes; `/` focuses input.

- [ ] **Step 3: Map click / reverse**

When no route panel is open, `createMap` `onMapClick` calls
`/map/reverse?lat=&lon=` and opens the place sheet; when the route panel is
open, it sets destination B instead. Add a “What’s here?” action that calls
reverse for the current centre.

- [ ] **Step 4: Place sheet actions**

Directions sets B and opens the route panel; Copy uses `navigator.clipboard`
with a textarea fallback; Open in OSM opens the copyright/OSM link in a new
tab; Save is wired in Task 6.

- [ ] **Step 5: Error/empty states**

No results → “No places found near here.”; 502 → toast with Retry; abort errors
are ignored. Never clear the map.

- [ ] **Step 6: Browser verification + gate**

Search “pharmacy”, verify results and fly-to; map click opens the sheet;
network log only same-origin. Run the full gate.

---

### Task 5: Route planning, alternatives, steps and follow mode

**Files:**
- Modify: `web/app/nav.js`, `web/app/map.js`, `web/app/index.html`, `web/app/app.css`

**Interfaces:**
- `GET /map/route?from_lat=&from_lon=&to_lat=&to_lon=&mode=&alternatives=true`.
- `mapApi.drawRoute(route)` renders `points` as a mint polyline with a dark
  casing; `fitRoute(route)` fits bounds; `clearRoute()` removes both.

- [ ] **Step 1: Route panel markup**

A/B rows with swap button, mode tabs `driving|foot|bike`, alternatives chips,
summary (`distance · duration`), collapsible step list, Start/Clear buttons.

- [ ] **Step 2: Implement route client**

`planRoute({from, to, mode})` sends one request at a time; renders the first
route, alternatives as selectable chips (distance/time), steps as formatted
text, errors as `OSM_UNAVAILABLE` toast. Distance formatting: `<1000 m` in m,
else km with one decimal; duration in min/h.

- [ ] **Step 3: Interactions**

Swap A/B; map click sets B when panel open; selecting an alternative redraws
and refits; Clear removes route and resets the panel; Escape closes.

- [ ] **Step 4: Follow mode**

Start requests `navigator.geolocation.watchPosition` only after an explicit
click, centres the map on the fix, and shows the next step card; Stop clears
the watch. Denied permission → hint, route stays.

- [ ] **Step 5: Browser verification**

Route My location → searched pharmacy in all three modes; verify polyline,
ETA, steps, alternatives, swap, clear; network only same-origin. Run the gate.

---

### Task 6: Saved places, recents, locate and responsive polish

**Files:**
- Modify: `web/app/nav.js`, `web/app/app.js`, `web/app/index.html`, `web/app/app.css`

**Interfaces:**
- localStorage keys `pinpoint.savedPlaces.v1`, `pinpoint.recentSearches.v1`
  (arrays, max 50); corrupt JSON → reset to `[]`, never throw.
- `mapApi.locate()` resolves on the first fix, rejects on denial/timeout.

- [ ] **Step 1: Storage helpers**

`loadList(key)`/`saveList(key, list)` with try/catch and validation; recent
search added on select; saved place toggled from the place sheet; “Clear
history” button in the drawer.

- [ ] **Step 2: Locate button**

Topbar button calls `mapApi.locate()`, flies to the fix, drops the self
marker; denied → toast “Location permission denied — search instead.”

- [ ] **Step 3: Social drawer + responsive**

Move publish/friends/shares/sessions into a drawer toggled by a header button;
under 720 px it becomes a bottom sheet, search stays on top, route panel
collapsible. Focus trap + `aria-expanded`, Escape closes.

- [ ] **Step 4: Accessibility pass**

Labels on all icon buttons, visible focus rings, `aria-live="polite"` on
results/status, `prefers-reduced-motion` disables fly animations, contrast
check for mint/amber on navy.

- [ ] **Step 5: Browser verification + gate**

Test desktop 1280×800 and mobile 390×844 (devtools emulation): search, route,
place sheet, drawer, keyboard flow, corrupt localStorage. Run the gate.

---

### Task 7: Docs, version 1.1.0 and release artifact

**Files:**
- Modify: `Cargo.toml`, `CHANGELOG.md`, `ROADMAP.md`, `RELEASE_NOTES.md`, `README.md`, `docs/reference/api-contract.md`, `docs/ROADMAP.md`

- [ ] **Step 1: Update docs**

Document the three endpoints, Leaflet vendoring/license, keyboard shortcuts,
saved/recent storage and the new UI. Changelog entries flagged `[FEATURE]`,
`[FIX]`, `[SECURITY]`, `[DOCS]`, `[INFRA]` as in 1.0.0.

- [ ] **Step 2: Bump the version**

`Cargo.toml` → `1.1.0`; `/health` and User-Agent already use
`CARGO_PKG_VERSION`. Update `Cargo.lock` via `cargo build`.

- [ ] **Step 3: Full gate + release build**

Run the full gate, then `cargo build --release --locked`; verify
`/health` reports `1.1.0` from the release binary.

- [ ] **Step 4: Package**

Create `dist/pinpoint-1.1.0-windows-x64/` with the binaries, `web/app`
(including vendor), docs and `RUN.md`; write `SHA256SUMS.txt` and zip it.
Update `dist/SHA256SUMS.txt` and `RELEASE_NOTES.md` (1.1 section).

- [ ] **Step 5: Final browser proof**

Log in, search, route, switch modes, save a place, reload (saved survives),
clear history; capture screenshots and network log. Report counts and flag any
known gaps.
