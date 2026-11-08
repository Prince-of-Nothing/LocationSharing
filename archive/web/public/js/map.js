// ---------------------------------------------------------------------------
// Map engine: interactive Slippy map (Organic-Maps style) + friend pins.
//
// Zero-dependency vanilla implementation: raster tiles from config.js
// `tiles.url` / `tileLayers` (CARTO + Wikimedia, OSM data ODbL).
// Compliance (see THIRD_PARTY_NOTICES.md):
//  - Tiles come ONLY from config.js allow-listed hosts (never
//    tile.openstreetmap.org — OSMF Tile Usage Policy prohibits apps).
//  - ODbL attribution rendered permanently below the map; must stay visible.
//  - Friend pins use ONLY authorised coordinates from /locations/friends.
//  - Search/route/POI overlays are user-initiated; endpoints are explicit.
// ---------------------------------------------------------------------------
import { tiles, tileLayers, freshness as freshnessConfig, defaultCenter } from "./config.js";
import { $, esc } from "./ui.js";

export function freshness(updatedAt) {
  const mins = (Date.now() - new Date(updatedAt).getTime()) / 60000;
  if (!Number.isFinite(mins) || mins > freshnessConfig.staleMinutes) {
    return { label: "STALE — do not rely on this position", cls: "fresh-stale" };
  }
  if (mins > freshnessConfig.warnMinutes) {
    return { label: `updated ${Math.round(mins)} min ago`, cls: "fresh-warn" };
  }
  return { label: `updated ${Math.max(0, Math.round(mins))} min ago`, cls: "fresh-ok" };
}

// --- Web Mercator helpers ---------------------------------------------------
const TILE = 256;
function lngLatToWorld(lat, lon, zoom) {
  const n = 2 ** zoom;
  const latRad = Math.log(Math.tan(Math.PI / 4 + (lat * Math.PI) / 360));
  return { x: ((lon + 180) / 360) * n * TILE, y: ((1 - latRad / Math.PI) / 2) * n * TILE };
}
function worldToLngLat(x, y, zoom) {
  const n = 2 ** zoom;
  const lon = (x / (n * TILE)) * 360 - 180;
  const lat = (Math.atan(Math.sinh(Math.PI * (1 - (2 * y) / (n * TILE)))) * 180) / Math.PI;
  return { latitude: Math.max(-85.0511, Math.min(85.0511, lat)), longitude: Math.max(-180, Math.min(180, lon)) };
}

// --- Map state --------------------------------------------------------------
const state = {
  ready: false,
  centre: { latitude: defaultCenter.latitude, longitude: defaultCenter.longitude },
  zoom: defaultCenter.zoom,
  layerId: "dark",
  friends: [],
  myLoc: null,
  searchPin: null,
  pois: [],
  route: null,
  selectedLoc: null,
  els: {},
};

function layer() { return tileLayers[state.layerId] || tiles; }

function ensureDom() {
  if (state.ready) return;
  const map = $("#map");
  if (!map) return;
  map.innerHTML = "";
  map.style.position = "relative";
  map.style.overflow = "hidden";
  map.style.touchAction = "none";
  map.setAttribute("role", "application");
  map.setAttribute("aria-label", "Interactive map. Drag to pan, scroll to zoom.");

  const tilesEl = document.createElement("div");
  tilesEl.className = "map-tiles";
  const overlay = document.createElement("svg");
  overlay.className = "map-overlay";
  overlay.setAttribute("preserveAspectRatio", "none");
  const markersEl = document.createElement("div");
  markersEl.className = "map-markers";
  map.append(tilesEl, overlay, markersEl);

  const controls = document.createElement("div");
  controls.className = "map-controls";
  controls.innerHTML =
    '<button class="map-btn" data-act="zoom-in" aria-label="Zoom in">+</button>' +
    '<button class="map-btn" data-act="zoom-out" aria-label="Zoom out">−</button>' +
    '<button class="map-btn" data-act="locate" aria-label="Center on my location">◎</button>' +
    '<button class="map-btn" data-act="north" aria-label="Reset north">N</button>';
  map.appendChild(controls);

  const scale = document.createElement("div");
  scale.className = "map-scale";
  scale.textContent = "";
  map.appendChild(scale);

  const coords = document.createElement("div");
  coords.className = "map-coords";
  coords.textContent = "";
  map.appendChild(coords);

  state.els = { map, tilesEl, overlay, markersEl, scale, coords };
  bindInteractions();
  state.ready = true;
}

function renderTiles() {
  const { map, tilesEl } = state.els;
  if (!state.ready) return;
  const W = map.clientWidth || 800, H = map.clientHeight || 420;
  const { x: cx, y: cy } = lngLatToWorld(state.centre.latitude, state.centre.longitude, state.zoom);
  const tx0 = Math.floor(cx / TILE), ty0 = Math.floor(cy / TILE);
  const ox = (cx / TILE - tx0) * TILE, oy = (cy / TILE - ty0) * TILE;
  const cols = Math.ceil(W / TILE) + 2, rows = Math.ceil(H / TILE) + 2;
  const maxTile = 2 ** state.zoom;
  const t = layer();
  tilesEl.innerHTML = "";
  for (let i = 0; i < cols; i++) {
    for (let j = 0; j < rows; j++) {
      const tx = ((tx0 - 1 + i) % maxTile + maxTile) % maxTile;
      const ty = ty0 - 1 + j;
      if (ty < 0 || ty >= maxTile) continue;
      const img = document.createElement("img");
      img.src = t.url.replace("{z}", state.zoom).replace("{x}", tx).replace("{y}", ty);
      img.alt = "";
      img.draggable = false;
      img.loading = "lazy";
      img.decoding = "async";
      img.className = "map-tile";
      img.style.left = `${i * TILE - ox - TILE}px`;
      img.style.top = `${j * TILE - oy - TILE}px`;
      tilesEl.appendChild(img);
    }
  }
  renderOverlay();
  renderMarkers();
  updateScale();
}

function latLonToScreen(lat, lon) {
  const { map } = state.els;
  const W = map.clientWidth || 800, H = map.clientHeight || 420;
  const c = lngLatToWorld(state.centre.latitude, state.centre.longitude, state.zoom);
  const p = lngLatToWorld(lat, lon, state.zoom);
  return { x: p.x - c.x + W / 2, y: p.y - c.y + H / 2 };
}

function screenToLatLon(px, py) {
  const { map } = state.els;
  const W = map.clientWidth || 800, H = map.clientHeight || 420;
  const c = lngLatToWorld(state.centre.latitude, state.centre.longitude, state.zoom);
  return worldToLngLat(c.x + (px - W / 2), c.y + (py - H / 2), state.zoom);
}

function renderOverlay() {
  const { overlay } = state.els;
  const { map } = state.els;
  const W = map.clientWidth || 800, H = map.clientHeight || 420;
  overlay.setAttribute("viewBox", `0 0 ${W} ${H}`);
  overlay.innerHTML = "";
  if (!state.route?.points?.length) return;
  const pts = state.route.points.map((p) => { const s = latLonToScreen(p.latitude, p.longitude); return `${s.x},${s.y}`; }).join(" ");
  const line = document.createElementNS("http://www.w3.org/2000/svg", "polyline");
  line.setAttribute("points", pts);
  line.setAttribute("class", "map-route-line");
  overlay.appendChild(line);
  // Endpoint dots
  [state.route.points[0], state.route.points[state.route.points.length - 1]].forEach((p) => {
    const s = latLonToScreen(p.latitude, p.longitude);
    const dot = document.createElementNS("http://www.w3.org/2000/svg", "circle");
    dot.setAttribute("cx", s.x); dot.setAttribute("cy", s.y); dot.setAttribute("r", "6");
    dot.setAttribute("class", "map-route-dot");
    overlay.appendChild(dot);
  });
}

function renderMarkers() {
  const { markersEl } = state.els;
  markersEl.innerHTML = "";
  const add = (lat, lon, html, cls, title) => {
    const s = latLonToScreen(lat, lon);
    const el = document.createElement("div");
    el.className = `map-marker ${cls || ""}`;
    el.innerHTML = html;
    if (title) el.title = title;
    el.style.left = `${s.x}px`;
    el.style.top = `${s.y}px`;
    markersEl.appendChild(el);
  };
  state.friends.forEach((loc) => {
    const digits = loc.precision_level === "exact" ? 4 : 2;
    add(loc.latitude, loc.longitude, `<span class="pin-emoji">📍</span><span class="pin-label">${esc(String(loc.user_name || "").split(" ")[0])}</span>`,
      `pin-friend pin-${loc.precision_level || ""}`, `${loc.user_name}: ${loc.latitude.toFixed(digits)}, ${loc.longitude.toFixed(digits)}`);
  });
  if (state.myLoc) add(state.myLoc.latitude, state.myLoc.longitude, '<span class="pin-emoji">🔵</span>', "pin-me", "My location");
  if (state.searchPin) add(state.searchPin.latitude, state.searchPin.longitude, '<span class="pin-emoji">📌</span>', "pin-search",
    state.searchPin.displayName || state.searchPin.shortName || "Search result");
  if (state.selectedLoc) add(state.selectedLoc.latitude, state.selectedLoc.longitude, '<span class="pin-emoji">⭐</span>', "pin-selected",
    state.selectedLoc.name || "Selected");
  state.pois.forEach((poi) => {
    add(poi.latitude, poi.longitude, `<span class="pin-emoji">${esc(poi.icon || "•")}</span>`, "pin-poi", poi.name);
  });
}

function updateScale() {
  const { scale } = state.els;
  if (!scale) return;
  // Approx metres per pixel at centre latitude.
  const mpp = (156543.03392 * Math.cos((state.centre.latitude * Math.PI) / 180)) / (2 ** state.zoom);
  const targets = [50, 100, 200, 500, 1000, 2000, 5000, 10000];
  const pick = targets.reduce((best, t) => (Math.abs(t / mpp - 80) < Math.abs(best / mpp - 80) ? t : best), 100);
  scale.style.width = `${Math.round(pick / mpp)}px`;
  scale.textContent = pick >= 1000 ? `${pick / 1000} km` : `${pick} m`;
}

// --- Interactions -----------------------------------------------------------
function setZoom(z, cx, cy) {
  const next = Math.max(2, Math.min(layer().maxZoom || 19, Math.round(z)));
  if (next === state.zoom) return;
  // Keep the anchor point stable across zoom (wheel/pinch centred).
  const { map } = state.els;
  const rect = map.getBoundingClientRect();
  const ax = cx ?? rect.width / 2, ay = cy ?? rect.height / 2;
  const before = screenToLatLon(ax, ay);
  state.zoom = next;
  const c = lngLatToWorld(state.centre.latitude, state.centre.longitude, state.zoom);
  const p = lngLatToWorld(before.latitude, before.longitude, state.zoom);
  const W = rect.width || 800, H = rect.height || 420;
  state.centre = worldToLngLat(c.x + (p.x - c.x) + (W / 2 - ax), c.y + (p.y - c.y) + (H / 2 - ay), state.zoom);
  renderTiles();
  onMove?.();
}

function bindInteractions() {
  const { map } = state.els;
  const pointers = new Map();
  let drag = null, moved = false, pinch = null;

  const onDown = (e) => {
    map.setPointerCapture(e.pointerId);
    pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
    if (pointers.size === 1) { drag = { x: e.clientX, y: e.clientY, cx: state.centre.latitude, cl: state.centre.longitude }; moved = false; }
    if (pointers.size === 2) {
      const [a, b] = [...pointers.values()];
      pinch = { dist: Math.hypot(a.x - b.x, a.y - b.y), zoom: state.zoom };
      drag = null;
    }
  };
  const onMovePtr = (e) => {
    if (!pointers.has(e.pointerId)) return;
    pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
    const rect = map.getBoundingClientRect();
    if (pinch && pointers.size === 2) {
      const [a, b] = [...pointers.values()];
      const d = Math.hypot(a.x - b.x, a.y - b.y);
      if (pinch.dist > 0) setZoom(pinch.zoom + Math.log2(d / pinch.dist), e.clientX - rect.left, e.clientY - rect.top);
      return;
    }
    if (drag) {
      const dx = e.clientX - drag.x, dy = e.clientY - drag.y;
      if (Math.abs(dx) + Math.abs(dy) > 4) moved = true;
      const c = lngLatToWorld(drag.cx, drag.cl, state.zoom);
      state.centre = worldToLngLat(c.x - dx, c.y - dy, state.zoom);
      renderTiles();
      onMove?.();
    }
    state.els.coords.textContent = `${state.centre.latitude.toFixed(5)}, ${state.centre.longitude.toFixed(5)} · z${state.zoom}`;
  };
  const onUp = (e) => {
    pointers.delete(e.pointerId);
    if (pointers.size < 2) pinch = null;
    if (pointers.size === 0) {
      if (!moved && e.target === map) {
        const rect = map.getBoundingClientRect();
        const ll = screenToLatLon(e.clientX - rect.left, e.clientY - rect.top);
        onTap?.(ll, e);
      }
      drag = null; moved = false;
    }
  };
  map.addEventListener("pointerdown", onDown);
  map.addEventListener("pointermove", onMovePtr);
  map.addEventListener("pointerup", onUp);
  map.addEventListener("pointercancel", onUp);
  map.addEventListener("wheel", (e) => {
    e.preventDefault();
    const rect = map.getBoundingClientRect();
    setZoom(state.zoom + (e.deltaY < 0 ? 1 : -1), e.clientX - rect.left, e.clientY - rect.top);
  }, { passive: false });
  map.addEventListener("dblclick", (e) => {
    const rect = map.getBoundingClientRect();
    setZoom(state.zoom + 1, e.clientX - rect.left, e.clientY - rect.top);
  });
  map.addEventListener("keydown", (e) => {
    const step = 0.02 * (2 ** (14 - state.zoom));
    if (e.key === "ArrowUp") state.centre.latitude += step;
    else if (e.key === "ArrowDown") state.centre.latitude -= step;
    else if (e.key === "ArrowLeft") state.centre.longitude -= step;
    else if (e.key === "ArrowRight") state.centre.longitude += step;
    else if (e.key === "+" || e.key === "=") setZoom(state.zoom + 1);
    else if (e.key === "-" || e.key === "_") setZoom(state.zoom - 1);
    else return;
    e.preventDefault();
    renderTiles();
  });
  map.tabIndex = 0;

  map.querySelector(".map-controls")?.addEventListener("click", (e) => {
    const act = e.target?.dataset?.act;
    if (!act) return;
    if (act === "zoom-in") setZoom(state.zoom + 1);
    if (act === "zoom-out") setZoom(state.zoom - 1);
    if (act === "locate") onLocate?.() ?? locateMe();
    if (act === "north") { state.zoom = defaultCenter.zoom; if (state.friends.length) fitBounds(state.friends); else state.centre = { ...defaultCenter }; renderTiles(); }
  });
  window.addEventListener("resize", () => state.ready && renderTiles());
}

let onMove = null, onTap = null, onLocate = null;
export function onMapMove(fn) { onMove = fn; }
export function onMapTap(fn) { onTap = fn; }
export function onLocateRequest(fn) { onLocate = fn; }

// --- Public controller ------------------------------------------------------
export function setMapLayer(layerId) {
  if (!tileLayers[layerId]) return;
  state.layerId = layerId;
  renderTiles();
  $("#map-attribution").innerHTML = layer().attribution;
}
export function getMapLayer() { return state.layerId; }

export function getMapCenter() { return { ...state.centre, zoom: state.zoom }; }
export function getMapViewbox() {
  const { map } = state.els;
  const W = map.clientWidth || 800, H = map.clientHeight || 420;
  const tl = screenToLatLon(0, 0), br = screenToLatLon(W, H);
  return [tl.longitude, tl.latitude, br.longitude, br.latitude];
}

export function setMapCenter(lat, lon, zoom) {
  ensureDom();
  state.centre = { latitude: Number(lat), longitude: Number(lon) };
  if (zoom != null) state.zoom = Math.max(2, Math.min(layer().maxZoom || 19, Math.round(zoom)));
  renderTiles();
}

export function setMyLocation(loc) { state.myLoc = loc; renderTiles(); }

export function showSearchResult(place) {
  ensureDom();
  state.searchPin = place ? { latitude: place.latitude ?? place.lat, longitude: place.longitude ?? place.lon, displayName: place.displayName || place.shortName || place.name } : null;
  state.selectedLoc = null;
  if (place) setMapCenter(state.searchPin.latitude, state.searchPin.longitude, Math.max(state.zoom, 14));
  renderTiles();
}

export function showSelectedLocation(loc) {
  ensureDom();
  state.selectedLoc = loc ? { latitude: loc.latitude, longitude: loc.longitude, name: loc.name } : null;
  if (loc) setMapCenter(loc.latitude, loc.longitude, Math.max(state.zoom, 14));
  renderTiles();
}

export function showRouteOnMap(route) {
  ensureDom();
  state.route = route;
  if (route?.points?.length) fitBounds(route.points.map((p) => ({ latitude: p.latitude, longitude: p.longitude })));
  renderTiles();
}
export function clearRouteOnMap() { state.route = null; renderTiles(); }

export function showPoisOnMap(pois) { ensureDom(); state.pois = Array.isArray(pois) ? pois : []; renderTiles(); }
export function clearPoisOnMap() { state.pois = []; renderTiles(); }

export function fitBounds(points) {
  ensureDom();
  if (!points?.length) return;
  const lats = points.map((p) => p.latitude), lons = points.map((p) => p.longitude);
  let minLat = Math.min(...lats), maxLat = Math.max(...lats);
  let minLon = Math.min(...lons), maxLon = Math.max(...lons);
  if (minLat === maxLat) { minLat -= 0.02; maxLat += 0.02; }
  if (minLon === maxLon) { minLon -= 0.02; maxLon += 0.02; }
  const cLat = (minLat + maxLat) / 2, cLon = (minLon + maxLon) / 2;
  const span = Math.max(maxLat - minLat, maxLon - minLon);
  const z = span > 1 ? 8 : span > 0.2 ? 10 : span > 0.05 ? 12 : span > 0.01 ? 14 : 16;
  setMapCenter(cLat, cLon, z);
}

export function locateMe() {
  if (!navigator.geolocation) return;
  navigator.geolocation.getCurrentPosition(
    (pos) => { setMyLocation({ latitude: pos.coords.latitude, longitude: pos.coords.longitude }); setMapCenter(pos.coords.latitude, pos.coords.longitude, 15); },
    () => {}, { timeout: 8000, enableHighAccuracy: true },
  );
}

/** Compat entry point (existing main.js): renders friends + keeps list. */
export function renderMap(locs) {
  ensureDom();
  state.friends = Array.isArray(locs) ? locs : [];
  if (state.friends.length && (!state.els.map || state.els.map.dataset.fitted !== "1")) {
    fitBounds(state.friends);
    state.els.map.dataset.fitted = "1";
  }
  renderTiles();
  $("#map-attribution").innerHTML = layer().attribution;

  const list = $("#friend-list");
  if (!list) return;
  list.innerHTML = "";
  state.friends.forEach((loc) => {
    const digits = loc.precision_level === "exact" ? 4 : 2;
    const fr = freshness(loc.updated_at);
    const li = document.createElement("li");
    li.innerHTML = `<strong>${esc(loc.user_name)}</strong>
      <span class="badge ${esc(loc.precision_level)}">${esc(loc.precision_level)}</span>
      <div class="${fr.cls}">${esc(fr.label)}</div>
      <div class="muted">${Number(loc.latitude).toFixed(digits)}, ${Number(loc.longitude).toFixed(digits)}${loc.accuracy != null ? ` ±${Math.round(loc.accuracy)} m` : ""}</div>
      <div class="row" style="margin-top:.4rem">
        <a class="btn" target="_blank" rel="noopener" href="geo:${loc.latitude},${loc.longitude}?q=${loc.latitude}%2C${loc.longitude}">Navigate here</a>
        <a class="btn" target="_blank" rel="noopener" href="https://www.openstreetmap.org/directions?to=${loc.latitude}%2C${loc.longitude}">Open OSM ↗</a>
      </div>`;
    const open = document.createElement("button");
    open.className = "btn"; open.textContent = "Center";
    open.addEventListener("click", () => setMapCenter(loc.latitude, loc.longitude, 15));
    li.appendChild(open);
    list.appendChild(li);
  });
}
