// ---------------------------------------------------------------------------
// Pinpoint — service worker (offline app shell).
//
// Privacy stance: this worker caches ONLY the static app shell (HTML/CSS/JS).
// API responses — especially anything containing locations — are NEVER put
// in Cache Storage: a shared or stolen device must not expose location data
// from an offline cache. Tiles, search, routing and POI responses are also
// NEVER cached (bulk tile caching violates provider ToS; route/POI responses
// are session data). Live data requires connectivity, by design.
// Bookmarks (localStorage) are the only offline user data; the SW never
// touches localStorage.
//
// Bump SW_VERSION in js/config.js when static assets change.
// ---------------------------------------------------------------------------

const VERSION = "pinpoint-shell-v3";
const SHELL = [
  "./",
  "./index.html",
  "./app.css",
  "./manifest.webmanifest",
  "./js/config.js",
  "./js/api.js",
  "./js/session.js",
  "./js/ui.js",
  "./js/map.js",
  "./js/geocode.js",
  "./js/routing.js",
  "./js/places.js",
  "./js/bookmarks.js",
  "./js/navigation.js",
  "./js/main.js",
];

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(VERSION).then((cache) => cache.addAll(SHELL)).then(() => self.skipWaiting()),
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches.keys()
      .then((keys) => Promise.all(keys.filter((k) => k !== VERSION).map((k) => caches.delete(k))))
      .then(() => self.clients.claim()),
  );
});

self.addEventListener("fetch", (event) => {
  const url = new URL(event.request.url);

  // Same-origin API and health: network only, never cached (location data).
  if (url.pathname.startsWith("/api/") || url.pathname === "/health") {
    event.respondWith(fetch(event.request));
    return;
  }

  // Cross-origin (map tiles, Nominatim, OSRM, Overpass): let the browser
  // handle it — we deliberately never cache these responses (provider ToS
  // + they are not app-shell assets). The browser's own HTTP cache still
  // applies within the session (tiles are in-memory/OS cache only).
  if (url.origin !== self.location.origin) {
    return;
  }

  // Static shell: cache-first (instant loads, works fully offline).
  event.respondWith(
    caches.match(event.request).then((cached) => {
      if (cached) return cached;
      return fetch(event.request).then((res) => {
        if (res.ok && event.request.method === "GET") {
          const copy = res.clone();
          caches.open(VERSION).then((cache) => cache.put(event.request, copy));
        }
        return res;
      });
    }),
  );
});
