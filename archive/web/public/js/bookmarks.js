// ---------------------------------------------------------------------------
// Bookmarks: user-saved places (Organic-Maps style favourites).
// Stored ONLY in this browser's localStorage, on this device. Friend/shared
// locations are never written here — bookmarks are manually saved places
// (search results, route endpoints) the user explicitly star.
//
// Privacy: bookmarks are not synced, not sent to any server. Export
// produces a standard GeoJSON download for use in OsmAnd/Organic Maps/
// Magic Earth.
// ---------------------------------------------------------------------------
import { esc } from "./ui.js";

const KEY = "pinpoint.bookmarks.v1";

function uid() { return `bm_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 8)}`; }

export function loadBookmarks() {
  try {
    const raw = localStorage.getItem(KEY);
    const arr = raw ? JSON.parse(raw) : [];
    return Array.isArray(arr) ? arr : [];
  } catch { return []; }
}

function persist(list) { localStorage.setItem(KEY, JSON.stringify(list)); }

export function addBookmark({ name, latitude, longitude, note = "", category = "" }) {
  const list = loadBookmarks();
  const bm = { id: uid(), name: String(name || "Saved place"), latitude: Number(latitude), longitude: Number(longitude), note: String(note || ""), category: String(category || ""), createdAt: new Date().toISOString() };
  if (!Number.isFinite(bm.latitude) || !Number.isFinite(bm.longitude)) throw new Error("Invalid bookmark coordinates.");
  list.unshift(bm);
  persist(list);
  return bm;
}

export function removeBookmark(id) { persist(loadBookmarks().filter((b) => b.id !== id)); }

export function clearBookmarks() { localStorage.removeItem(KEY); }

export function exportGeoJSON() {
  const features = loadBookmarks().map((b) => ({
    type: "Feature",
    geometry: { type: "Point", coordinates: [b.longitude, b.latitude] },
    properties: { name: b.name, note: b.note, category: b.category },
  }));
  return JSON.stringify({ type: "FeatureCollection", features }, null, 2);
}

export function downloadGeoJSON() {
  const blob = new Blob([exportGeoJSON()], { type: "application/geo+json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url; a.download = "pinpoint-bookmarks.geojson";
  document.body.appendChild(a); a.click(); a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

/** Render the bookmarks list (escaped; actions via callback). */
export function renderBookmarksList(container, list, { onOpen, onDelete } = {}) {
  const ul = typeof container === "string" ? document.querySelector(container) : container;
  if (!ul) return;
  ul.innerHTML = "";
  if (!list.length) { ul.innerHTML = '<li class="muted">No saved places yet — star a search result or route point.</li>'; return; }
  list.forEach((b) => {
    const li = document.createElement("li");
    li.innerHTML =
      `<strong>★ ${esc(b.name)}</strong>` +
      `<div class="muted">${Number(b.latitude).toFixed(5)}, ${Number(b.longitude).toFixed(5)}${b.category ? ` · ${esc(b.category)}` : ""}${b.note ? ` — ${esc(b.note)}` : ""}</div>`;
    const open = document.createElement("button");
    open.className = "btn"; open.textContent = "Open";
    open.addEventListener("click", () => onOpen?.(b));
    const del = document.createElement("button");
    del.className = "btn danger"; del.textContent = "Delete";
    del.addEventListener("click", () => onDelete?.(b));
    li.appendChild(open); li.appendChild(del);
    ul.appendChild(li);
  });
}
