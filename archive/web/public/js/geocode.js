// ---------------------------------------------------------------------------
// Geocoding client: Nominatim search + reverse (Organic-Maps style search).
//
// OSMF Nominatim usage policy (https://operations.osmfoundation.org/policies/nominatim/):
//  - Max 1 request/second per user, valid referer, descriptive behaviour.
//  - Cache results aggressively; never bulk or scrape.
//  - Privacy: queries are user-typed only. Friend/shared locations are
//    NEVER sent here. Viewbox bias (visible map) is fine; exact friend
//    coordinates are not.
// ---------------------------------------------------------------------------
import { osm } from "./config.js";

const cache = new Map();
let lastRequestAt = 0;

async function throttledFetch(url, { signal } = {}) {
  const wait = Math.max(0, osm.nominatim.minIntervalMs - (Date.now() - lastRequestAt));
  if (wait > 0) await new Promise((r) => setTimeout(r, wait));
  lastRequestAt = Date.now();
  const res = await fetch(url, {
    signal,
    headers: { "Accept": "application/json", "Accept-Language": navigator.language || "en" },
  });
  if (!res.ok) throw new Error(`Search service error (HTTP ${res.status})`);
  return res.json();
}

function normalize(item) {
  return {
    id: `${item.osm_type || "N"}/${item.osm_id || item.place_id || ""}`,
    displayName: item.display_name || item.name || "Unnamed place",
    shortName: (item.display_name || "").split(",")[0] || item.name || "Unnamed place",
    latitude: parseFloat(item.lat),
    longitude: parseFloat(item.lon),
    type: item.type || "",
    category: item.class || item.category || item.type || "",
    osmId: item.osm_id ? Number(item.osm_id) : null,
    osmType: item.osm_type || null,
    address: item.address || null,
  };
}

/** Debounced multi-field search (addresses + named POIs). */
export async function searchPlaces(query, { viewbox = null, limit = osm.nominatim.maxResults } = {}) {
  const q = String(query || "").trim();
  if (q.length < 3) throw new Error("Type at least 3 characters to search.");
  if (!Number.isFinite(q.length)) throw new Error("Invalid query.");
  const key = `search:${q.toLowerCase()}:${viewbox?.join(",") || ""}:${limit}`;
  if (cache.has(key)) return cache.get(key);
  const params = new URLSearchParams({
    q, format: "jsonv2", limit: String(limit), addressdetails: "1", extratags: "1",
  });
  if (viewbox && viewbox.length === 4) {
    params.set("viewbox", viewbox.join(","));
    params.set("bounded", "0"); // prefer nearby results, don't hard-clip
  }
  const data = await throttledFetch(`${osm.nominatim.base}/search?${params}`);
  const out = (Array.isArray(data) ? data : [])
    .map(normalize)
    .filter((p) => Number.isFinite(p.latitude) && Number.isFinite(p.longitude));
  if (cache.size > 120) cache.clear();
  cache.set(key, out);
  return out;
}

/** Reverse geocode a point to a human-readable place name. */
export async function reverseGeocode(latitude, longitude) {
  const lat = Number(latitude), lon = Number(longitude);
  if (!Number.isFinite(lat) || !Number.isFinite(lon)) throw new Error("Invalid coordinates.");
  const key = `rev:${lat.toFixed(5)},${lon.toFixed(5)}`;
  if (cache.has(key)) return cache.get(key);
  const params = new URLSearchParams({ lat: String(lat), lon: String(lon), format: "jsonv2", zoom: "16", addressdetails: "1" });
  const data = await throttledFetch(`${osm.nominatim.base}/reverse?${params}`);
  const out = data ? normalize(data) : null;
  cache.set(key, out);
  return out;
}

/** Place details / "more info" for a known OSM node/way/relation. */
export async function lookupDetails(osmType, osmId) {
  if (!osmType || !osmId) throw new Error("Unknown OSM id.");
  const params = new URLSearchParams({ osm_type: osmType, osm_id: String(osmId), format: "jsonv2", addressdetails: "1", extratags: "1" });
  const data = await throttledFetch(`${osm.nominatim.base}/lookup?${params}`);
  const item = Array.isArray(data) ? data[0] : data;
  return item ? normalize(item) : null;
}

/** Clear in-memory geocode cache (privacy: e.g. on logout). */
export function clearGeocodeCache() { cache.clear(); }
