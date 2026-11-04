// ---------------------------------------------------------------------------
// Pinpoint — interactive raster map viewer.
//
// Zero third-party JS trackers. Custom pure-ES-module Mercator viewer:
//  - Smooth mouse and touch drag-to-pan
//  - Scroll-wheel and button zoom (levels 3-18)
//  - Exact Web Mercator projection (tiles and pins align with mathematical precision)
//  - Interactive friend pin tooltips & GPS user location radar pulse
//  - ODbL attribution permanently visible
// ---------------------------------------------------------------------------
import { tiles, freshness as freshnessConfig } from "./config.js";
import { $, esc } from "./ui.js";

const TILE_SIZE = 256;

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

// Spherical Web Mercator projection (EPSG:3857)
export function project(lat, lon, z) {
  const scale = TILE_SIZE * (2 ** z);
  const x = ((lon + 180) / 360) * scale;
  const sin = Math.sin((lat * Math.PI) / 180);
  const clampedSin = Math.min(Math.max(sin, -0.9999), 0.9999);
  const y = (0.5 - Math.log((1 + clampedSin) / (1 - clampedSin)) / (4 * Math.PI)) * scale;
  return { x, y };
}

export function unproject(px, py, z) {
  const scale = TILE_SIZE * (2 ** z);
  const lon = (px / scale) * 360 - 180;
  const y2 = 180 - (py / scale) * 360;
  const lat = (360 / Math.PI) * Math.atan(Math.exp((y2 * Math.PI) / 180)) - 90;
  return { lat, lon };
}

// Map state
const state = {
  centerLat: 48.8566,
  centerLon: 2.3522,
  zoom: 11,
  minZoom: 3,
  maxZoom: 18,
  locs: [],
  myLoc: null, // { lat, lon, accuracy }
  activePopupId: null,
  isDragging: false,
  dragStartX: 0,
  dragStartY: 0,
  startCenterPx: { x: 0, y: 0 },
  hasAutoCentered: false,
  initializedListeners: false,
};

export function zoomIn() {
  if (state.zoom < state.maxZoom) {
    state.zoom += 1;
    draw();
  }
}

export function zoomOut() {
  if (state.zoom > state.minZoom) {
    state.zoom -= 1;
    draw();
  }
}

export function flyTo(lat, lon, zoom) {
  state.centerLat = lat;
  state.centerLon = lon;
  if (zoom != null) {
    state.zoom = Math.min(Math.max(zoom, state.minZoom), state.maxZoom);
  }
  draw();
}

export function setMyPosition(lat, lon, accuracy) {
  state.myLoc = { lat, lon, accuracy };
  draw();
}

export function fitAll() {
  if (state.locs.length === 0) {
    if (state.myLoc) {
      flyTo(state.myLoc.lat, state.myLoc.lon, 12);
    } else {
      flyTo(48.8566, 2.3522, 11);
    }
    return;
  }
  const avgLat = state.locs.reduce((sum, l) => sum + l.latitude, 0) / state.locs.length;
  const avgLon = state.locs.reduce((sum, l) => sum + l.longitude, 0) / state.locs.length;
  flyTo(avgLat, avgLon, 12);
}

function setupLayers(map) {
  map.innerHTML = `
    <div class="map-tile-layer"></div>
    <div class="map-pin-layer"></div>
    <div class="map-popup-layer"></div>
    <div class="map-controls">
      <button class="map-ctrl-btn" id="ctrl-zoom-in" title="Zoom in">+</button>
      <button class="map-ctrl-btn" id="ctrl-zoom-out" title="Zoom out">−</button>
      <button class="map-ctrl-btn" id="ctrl-fit-all" title="Fit all pins">🎯</button>
      <span class="map-zoom-badge">z${state.zoom}</span>
    </div>
  `;

  map.querySelector("#ctrl-zoom-in").addEventListener("click", (e) => {
    e.stopPropagation();
    zoomIn();
  });
  map.querySelector("#ctrl-zoom-out").addEventListener("click", (e) => {
    e.stopPropagation();
    zoomOut();
  });
  map.querySelector("#ctrl-fit-all").addEventListener("click", (e) => {
    e.stopPropagation();
    fitAll();
  });
}

function initEventListeners(map) {
  // Mouse drag
  map.addEventListener("mousedown", (e) => {
    if (e.target.closest(".map-controls") || e.target.closest(".map-popup")) return;
    state.isDragging = true;
    state.dragStartX = e.clientX;
    state.dragStartY = e.clientY;
    state.startCenterPx = project(state.centerLat, state.centerLon, state.zoom);
    map.style.cursor = "grabbing";
    const popupLayer = map.querySelector(".map-popup-layer");
    if (popupLayer) popupLayer.innerHTML = "";
    state.activePopupId = null;
  });

  window.addEventListener("mousemove", (e) => {
    if (!state.isDragging) return;
    const dx = e.clientX - state.dragStartX;
    const dy = e.clientY - state.dragStartY;
    const newPx = state.startCenterPx.x - dx;
    const newPy = state.startCenterPx.y - dy;
    const { lat, lon } = unproject(newPx, newPy, state.zoom);
    state.centerLat = lat;
    state.centerLon = lon;
    draw();
  });

  window.addEventListener("mouseup", () => {
    if (state.isDragging) {
      state.isDragging = false;
      map.style.cursor = "grab";
    }
  });

  // Touch drag
  map.addEventListener("touchstart", (e) => {
    if (e.touches.length !== 1) return;
    if (e.target.closest(".map-controls") || e.target.closest(".map-popup")) return;
    state.isDragging = true;
    state.dragStartX = e.touches[0].clientX;
    state.dragStartY = e.touches[0].clientY;
    state.startCenterPx = project(state.centerLat, state.centerLon, state.zoom);
    const popupLayer = map.querySelector(".map-popup-layer");
    if (popupLayer) popupLayer.innerHTML = "";
    state.activePopupId = null;
  }, { passive: true });

  window.addEventListener("touchmove", (e) => {
    if (!state.isDragging || e.touches.length !== 1) return;
    const dx = e.touches[0].clientX - state.dragStartX;
    const dy = e.touches[0].clientY - state.dragStartY;
    const newPx = state.startCenterPx.x - dx;
    const newPy = state.startCenterPx.y - dy;
    const { lat, lon } = unproject(newPx, newPy, state.zoom);
    state.centerLat = lat;
    state.centerLon = lon;
    draw();
  }, { passive: true });

  window.addEventListener("touchend", () => {
    state.isDragging = false;
  });

  // Wheel zoom
  map.addEventListener("wheel", (e) => {
    e.preventDefault();
    if (e.deltaY < 0) {
      zoomIn();
    } else if (e.deltaY > 0) {
      zoomOut();
    }
  }, { passive: false });
}

function showPopup(loc, x, y) {
  const map = $("#map");
  if (!map) return;
  const popupLayer = map.querySelector(".map-popup-layer");
  if (!popupLayer) return;

  const digits = loc.precision_level === "exact" ? 4 : 2;
  const fr = freshness(loc.updated_at);

  popupLayer.innerHTML = `
    <div class="map-popup" style="left: ${x}px; top: ${y - 14}px;">
      <button class="map-popup-close" id="popup-close-btn">&times;</button>
      <strong>${esc(loc.user_name)}</strong>
      <span class="badge ${esc(loc.precision_level)}">${esc(loc.precision_level)}</span>
      <div class="${fr.cls}" style="font-size:0.75rem;margin:.25rem 0;">${esc(fr.label)}</div>
      <div class="muted" style="font-size:0.75rem;">${loc.latitude.toFixed(digits)}, ${loc.longitude.toFixed(digits)}</div>
      <div class="popup-actions">
        <a class="btn small" target="_blank" rel="noopener" href="https://www.openstreetmap.org/directions?to=${loc.latitude}%2C${loc.longitude}">Open OSM ↗</a>
      </div>
    </div>
  `;

  popupLayer.querySelector("#popup-close-btn").addEventListener("click", (e) => {
    e.stopPropagation();
    state.activePopupId = null;
    popupLayer.innerHTML = "";
  });
}

function draw() {
  const map = $("#map");
  if (!map) return;
  const W = map.clientWidth || 800;
  const H = map.clientHeight || 420;
  const z = state.zoom;
  const maxTile = 2 ** z;

  const tileLayer = map.querySelector(".map-tile-layer");
  const pinLayer = map.querySelector(".map-pin-layer");
  const zoomBadge = map.querySelector(".map-zoom-badge");

  if (!tileLayer || !pinLayer) return;
  if (zoomBadge) zoomBadge.textContent = `z${z}`;

  const centerPx = project(state.centerLat, state.centerLon, z);
  const tlX = centerPx.x - W / 2;
  const tlY = centerPx.y - H / 2;

  // 1. Render map tiles
  const startTileX = Math.floor(tlX / TILE_SIZE);
  const endTileX = Math.floor((tlX + W) / TILE_SIZE);
  const startTileY = Math.floor(tlY / TILE_SIZE);
  const endTileY = Math.floor((tlY + H) / TILE_SIZE);

  tileLayer.innerHTML = "";
  for (let tx = startTileX; tx <= endTileX; tx++) {
    const wrappedTx = ((tx % maxTile) + maxTile) % maxTile;
    for (let ty = startTileY; ty <= endTileY; ty++) {
      if (ty < 0 || ty >= maxTile) continue;
      const img = document.createElement("img");
      img.src = tiles.url
        .replace("{z}", z)
        .replace("{x}", wrappedTx)
        .replace("{y}", ty);
      img.alt = "";
      img.draggable = false;
      img.loading = "lazy";
      img.className = "map-tile";
      img.style.left = `${tx * TILE_SIZE - tlX}px`;
      img.style.top = `${ty * TILE_SIZE - tlY}px`;
      tileLayer.appendChild(img);
    }
  }

  // 2. Render pins
  pinLayer.innerHTML = "";

  // GPS user position
  if (state.myLoc) {
    const myP = project(state.myLoc.lat, state.myLoc.lon, z);
    const mx = myP.x - tlX;
    const my = myP.y - tlY;
    const myMarker = document.createElement("div");
    myMarker.className = "my-loc-marker";
    myMarker.style.left = `${mx}px`;
    myMarker.style.top = `${my}px`;
    myMarker.title = `You: ${state.myLoc.lat.toFixed(4)}, ${state.myLoc.lon.toFixed(4)}`;
    myMarker.innerHTML = `<span class="my-loc-dot"></span><span class="my-loc-pulse"></span>`;
    pinLayer.appendChild(myMarker);
  }

  // Friend pins
  state.locs.forEach((loc) => {
    const p = project(loc.latitude, loc.longitude, z);
    const px = p.x - tlX;
    const py = p.y - tlY;
    const fr = freshness(loc.updated_at);

    const pin = document.createElement("div");
    pin.className = `map-pin ${fr.cls}`;
    pin.style.left = `${px}px`;
    pin.style.top = `${py}px`;
    pin.title = `${loc.user_name} (${fr.label})`;

    pin.innerHTML = `
      <div class="pin-icon">📍</div>
      <div class="pin-label">${esc(loc.user_name.split(" ")[0])}</div>
    `;

    pin.addEventListener("click", (e) => {
      e.stopPropagation();
      state.activePopupId = loc.user_id || loc.id;
      showPopup(loc, px, py);
    });

    pinLayer.appendChild(pin);

    // Keep active popup anchored to pin location
    if (state.activePopupId && (state.activePopupId === (loc.user_id || loc.id))) {
      showPopup(loc, px, py);
    }
  });

  // Attribution
  const attr = $("#map-attribution");
  if (attr) attr.innerHTML = tiles.attribution;
}

/** Renders friend locations as pins over interactive raster tiles; updates list. */
export function renderMap(locs) {
  state.locs = Array.isArray(locs) ? locs : [];
  const map = $("#map");
  if (!map) return;

  if (!map.querySelector(".map-tile-layer")) {
    setupLayers(map);
  }

  if (!state.initializedListeners) {
    initEventListeners(map);
    state.initializedListeners = true;
  }

  // Auto-center on first load if we haven't centered yet
  if (!state.hasAutoCentered && state.locs.length > 0) {
    const avgLat = state.locs.reduce((sum, l) => sum + l.latitude, 0) / state.locs.length;
    const avgLon = state.locs.reduce((sum, l) => sum + l.longitude, 0) / state.locs.length;
    state.centerLat = avgLat;
    state.centerLon = avgLon;
    state.hasAutoCentered = true;
  }

  draw();

  // Render friend list cards below map
  const list = $("#friend-list");
  if (!list) return;
  list.innerHTML = "";

  state.locs.forEach((loc) => {
    const digits = loc.precision_level === "exact" ? 4 : 2;
    const fr = freshness(loc.updated_at);
    const li = document.createElement("li");
    li.innerHTML = `
      <div class="row" style="justify-content:space-between;margin-bottom:.3rem">
        <strong>${esc(loc.user_name)}</strong>
        <span class="badge ${esc(loc.precision_level)}">${esc(loc.precision_level)}</span>
      </div>
      <div class="${fr.cls}">${esc(fr.label)}</div>
      <div class="muted">${loc.latitude.toFixed(digits)}, ${loc.longitude.toFixed(digits)}${loc.accuracy != null ? ` ±${Math.round(loc.accuracy)} m` : ""}</div>
      <div class="row" style="margin-top:.6rem">
        <button class="btn btn-center-friend" data-lat="${loc.latitude}" data-lon="${loc.longitude}">Center on map 🎯</button>
        <a class="btn" target="_blank" rel="noopener" href="https://www.openstreetmap.org/directions?to=${loc.latitude}%2C${loc.longitude}">Open OSM ↗</a>
      </div>`;

    li.querySelector(".btn-center-friend").addEventListener("click", () => {
      flyTo(loc.latitude, loc.longitude, 13);
      const W = map.clientWidth || 800;
      const H = map.clientHeight || 420;
      state.activePopupId = loc.user_id || loc.id;
      showPopup(loc, W / 2, H / 2);
    });

    list.appendChild(li);
  });
}
