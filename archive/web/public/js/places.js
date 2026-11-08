// ---------------------------------------------------------------------------
// POI explorer: Overpass API browsing around the visible map centre.
//
// Overpass policy: user-initiated, low-frequency queries (one per tap),
// moderate radius (default 1.5 km, max 5 km), generous timeout. Results
// are never bulk-downloaded or cached to disk. Centre + radius only —
// friend locations are never sent to Overpass.
// ---------------------------------------------------------------------------
import { osm, poiCategories } from "./config.js";

export { poiCategories };

function normalizePoi(el, category) {
  const tags = el.tags || {};
  const name = tags.name || tags.brand || tags.operator || category.icon + " (unnamed)";
  return {
    osmId: el.id,
    osmType: el.type,
    name,
    latitude: el.lat,
    longitude: el.lon,
    category: category.id,
    categoryLabel: category.label,
    icon: category.icon,
    tags,
    phone: tags.phone || tags["contact:phone"] || "",
    website: tags.website || tags["contact:website"] || "",
    openingHours: tags.opening_hours || "",
    cuisine: tags.cuisine || "",
  };
}

/**
 * Fetch POIs of one category around a centre.
 * @param {{latitude:number,longitude:number}} centre
 * @param {string} categoryId
 * @param {{radiusMeters?:number}} opts
 */
export async function fetchPois(centre, categoryId, { radiusMeters = 1500 } = {}) {
  const cat = poiCategories.find((c) => c.id === categoryId);
  if (!cat) throw new Error("Unknown POI category.");
  const radius = Math.min(5000, Math.max(200, Number(radiusMeters) || 1500));
  const body =
    `[out:json][timeout:${osm.overpass.timeoutSeconds}];` +
    `(${cat.ql.replaceAll("{r}", String(radius)).replaceAll("{lat}", centre.latitude.toFixed(6)).replaceAll("{lon}", centre.longitude.toFixed(6))});` +
    `out center 60;`;
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), (osm.overpass.timeoutSeconds + 5) * 1000);
  try {
    const res = await fetch(osm.overpass.base, {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded", "Accept": "application/json" },
      body: new URLSearchParams({ data: body }).toString(),
      signal: controller.signal,
    });
    if (!res.ok) throw new Error(`Places service error (HTTP ${res.status})`);
    const data = await res.json();
    return (data.elements || [])
      .filter((e) => Number.isFinite(e.lat) && Number.isFinite(e.lon))
      .map((e) => normalizePoi(e, cat));
  } finally {
    clearTimeout(timer);
  }
}

/** Wikipedia / website / phone links for a POI detail panel. */
export function poiLinks(poi) {
  const links = [];
  if (poi.website) links.push({ label: "Website ↗", href: poi.website });
  if (poi.tags?.wikipedia) {
    const [lang, title] = String(poi.tags.wikipedia).split(":");
    if (title) links.push({ label: "Wikipedia ↗", href: `https://${lang || "en"}.wikipedia.org/wiki/${encodeURIComponent(title.replaceAll(" ", "_"))}` });
  }
  if (poi.phone) links.push({ label: `📞 ${poi.phone}`, href: `tel:${poi.phone.replaceAll(/\s/g, "")}` });
  return links;
}
