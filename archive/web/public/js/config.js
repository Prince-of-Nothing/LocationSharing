// ---------------------------------------------------------------------------
// Pinpoint — web client configuration.
//
// Tile source policy (see THIRD_PARTY_NOTICES.md):
// The default is the Wikimedia Foundation tile endpoint, which explicitly
// welcomes third-party applications (unlike tile.openstreetmap.org, whose
// OSMF Tile Usage Policy prohibits app backends). Data is still ©
// OpenStreetMap contributors (ODbL) — attribution is rendered permanently in
// the map panel and enforced here via `attribution`.
//
// Self-hosted deployments (recommended for production): point `tiles.url`
// at your own server (Martin / TileServer GL serving Geofabrik extracts).
//
// OSM services (Organic-Maps / Magic-Earth parity):
//  - Tiles: CARTO raster (dark/light/voyager) + Wikimedia OSM standard.
//    Never tile.openstreetmap.org (bulk/app use is prohibited). Canonical
//    OSM wiki: https://wiki.openstreetmap.org/wiki/Main_Page
//  - Search: Nominatim (https://nominatim.openstreetmap.org) — must be
//    called sparingly (1 req/s), with viewbox, cached, no bulk.
//  - Routing: OSRM public demo (router.project-osrm.org) — demo only;
//    production must self-host OSRM/Valhalla/GraphHopper over OSM data.
//  - POI: Overpass API (overpass-api.de) — queries are user-initiated,
//    low frequency, never bulk.
//  - Offline: web keeps shell offline only; tile/route caching offline
//    is a mobile feature (offline packs). See THIRD_PARTY_NOTICES.md.
// ---------------------------------------------------------------------------

export const API_BASE = "/api/v1";

// --- Raster tile layers (world-wide OSM-derived raster) ---------------------
// {z}/{x}/{y} are substituted client-side. No API key anywhere: every layer
// below is keyless on purpose — keys imply accounts and third-party signup.
export const tileLayers = {
  dark: {
    label: "Dark (CARTO)",
    url: "https://a.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}.png",
    attribution:
      '© <a href="https://www.openstreetmap.org/copyright" target="_blank" rel="noopener">OpenStreetMap</a> contributors, <a href="https://carto.com/attributions" target="_blank" rel="noopener">CARTO</a>',
    maxZoom: 19,
  },
  light: {
    label: "Light (CARTO)",
    url: "https://a.basemaps.cartocdn.com/light_all/{z}/{x}/{y}.png",
    attribution:
      '© <a href="https://www.openstreetmap.org/copyright" target="_blank" rel="noopener">OpenStreetMap</a> contributors, <a href="https://carto.com/attributions" target="_blank" rel="noopener">CARTO</a>',
    maxZoom: 19,
  },
  voyager: {
    label: "Voyager (CARTO)",
    url: "https://a.basemaps.cartocdn.com/rastertiles/voyager/{z}/{x}/{y}.png",
    attribution:
      '© <a href="https://www.openstreetmap.org/copyright" target="_blank" rel="noopener">OpenStreetMap</a> contributors, <a href="https://carto.com/attributions" target="_blank" rel="noopener">CARTO</a>',
    maxZoom: 19,
  },
  osm: {
    label: "OSM Standard (Wikimedia)",
    // Wikimedia Foundation raster of OSM data — welcomes app use, keyless.
    url: "https://maps.wikimedia.org/osm-intl/{z}/{x}/{y}.png",
    attribution:
      '© <a href="https://www.openstreetmap.org/copyright" target="_blank" rel="noopener">OpenStreetMap</a> contributors, <a href="https://foundation.wikimedia.org/wiki/Maps" target="_blank" rel="noopener">Wikimedia Foundation</a>',
    maxZoom: 19,
  },
};

export const tiles = tileLayers.dark;

// --- OSM web services -------------------------------------------------------
// All three are optional and user-initiated. Each has a strict usage policy;
// see docs/sdlc/07-openstreetmap-integration.md for rate limits, caching and
// self-hosting guidance. Never send Pinpoint friend/shared locations to any
// of them — queries only contain user-typed search text or route endpoints
// the user explicitly chose to search/route to.
export const osm = {
  nominatim: {
    base: "https://nominatim.openstreetmap.org",
    attribution:
      '© <a href="https://www.openstreetmap.org/copyright" target="_blank" rel="noopener">OpenStreetMap</a> contributors (Nominatim / OpenStreetMap Foundation)',
    // Nominatim usage policy: max 1 req/s, valid referer, cache results,
    // no bulk. UI debounces and throttles accordingly.
    minIntervalMs: 1000,
    maxResults: 8,
  },
  osrm: {
    // Public demo server (driving/foot/bike). Demo only — operators must
    // self-host for production. Data: OSM planet extracts.
    base: "https://router.project-osrm.org",
    attribution:
      '© <a href="https://www.openstreetmap.org/copyright" target="_blank" rel="noopener">OpenStreetMap</a> contributors, routing by OSRM (route.openstreetmap.org)',
    profiles: { driving: "driving", foot: "foot", bike: "bike" },
  },
  overpass: {
    base: "https://overpass-api.de/api/interpreter",
    alternative: "https://overpass.kumi.systems/api/interpreter",
    attribution:
      '© <a href="https://www.openstreetmap.org/copyright" target="_blank" rel="noopener">OpenStreetMap</a> contributors via Overpass API',
    // Overpass policy: user-initiated, low volume, generous timeout.
    timeoutSeconds: 25,
    maxResults: 60,
  },
};

// --- POI categories (Organic-Maps style browse) ----------------------------
// Each maps to Overpass QL fragments. Labels/icons are UI-only; selection
// never sends friend locations to Overpass — only the visible map centre
// and the chosen category.
export const poiCategories = [
  { id: "food", label: "Food", icon: "🍽️", ql: 'node["amenity"="restaurant"](around:{r},{lat},{lon});node["amenity"="cafe"](around:{r},{lat},{lon});node["amenity"="fast_food"](around:{r},{lat},{lon});' },
  { id: "fuel", label: "Fuel", icon: "⛽", ql: 'node["amenity"="fuel"](around:{r},{lat},{lon});' },
  { id: "pharmacy", label: "Pharmacy", icon: "💊", ql: 'node["amenity"="pharmacy"](around:{r},{lat},{lon});' },
  { id: "hospital", label: "Health", icon: "🏥", ql: 'node["amenity"="hospital"](around:{r},{lat},{lon});node["amenity"="doctors"](around:{r},{lat},{lon});' },
  { id: "hotel", label: "Stay", icon: "🏨", ql: 'node["tourism"="hotel"](around:{r},{lat},{lon});node["tourism"="hostel"](around:{r},{lat},{lon});node["tourism"="guest_house"](around:{r},{lat},{lon});' },
  { id: "tourism", label: "Sights", icon: "🏛️", ql: 'node["tourism"="museum"](around:{r},{lat},{lon});node["tourism"="attraction"](around:{r},{lat},{lon});node["tourism"="viewpoint"](around:{r},{lat},{lon});' },
  { id: "water", label: "Water", icon: "💧", ql: 'node["amenity"="drinking_water"](around:{r},{lat},{lon});node["amenity"="toilets"](around:{r},{lat},{lon});' },
  { id: "parking", label: "Parking", icon: "🅿️", ql: 'node["amenity"="parking"](around:{r},{lat},{lon});' },
  { id: "park", label: "Parks", icon: "🌳", ql: 'node["leisure"="park"](around:{r},{lat},{lon});node["leisure"="playground"](around:{r},{lat},{lon});' },
];

// --- Travel modes (routing + navigation) -----------------------------------
export const travelModes = [
  { id: "driving", label: "🚗 Driving", profile: "driving", speedKmh: 40 },
  { id: "foot", label: "🚶 Walking", profile: "foot", speedKmh: 5 },
  { id: "bike", label: "🚲 Cycling", profile: "bike", speedKmh: 15 },
];

// Location freshness thresholds (minutes) — mirrored from the web UI states.
export const freshness = {
  warnMinutes: 10,
  staleMinutes: 30,
};

// Service worker cache version — bump when any static asset changes.
export const SW_VERSION = "v3";

// Default map centre until friends/geolocation position it (Chișinău).
export const defaultCenter = { latitude: 47.0105, longitude: 28.8638, zoom: 12 };
