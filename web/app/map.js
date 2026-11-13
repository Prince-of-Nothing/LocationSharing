// Leaflet map shell — locally vendored, same-origin tiles, no CDN.
// Exposes a small API used by app.js/nav.js so map concerns stay here.

export function createMap({ element, center, zoom, onClick }) {
  const map = L.map(element, {
    zoomControl: false,
    attributionControl: false,
    minZoom: 2,
    maxZoom: 19,
    worldCopyJump: true,
  }).setView(center, zoom);

  L.tileLayer("/map/tiles/{z}/{x}/{y}?v=4", {
    maxZoom: 19,
    className: "tiles-dark",
  }).addTo(map);

  L.control
    .attribution({ position: "bottomright", prefix: false })
    .addAttribution(
      '© <a href="https://www.openstreetmap.org/copyright" target="_blank" rel="noopener">OpenStreetMap contributors</a>',
    )
    .addTo(map);
  L.control.zoom({ position: "bottomright" }).addTo(map);
  L.control.scale({ imperial: false, position: "bottomleft", maxWidth: 110 }).addTo(map);

  map.on("click", (e) => {
    if (typeof onClick === "function") {
      onClick({ lat: e.latlng.lat, lon: e.latlng.lng });
    }
  });

  const friendLayer = L.layerGroup().addTo(map);
  let selectedMarker = null;
  let selfMarker = null;
  let selfAccuracy = null;
  let routeLayer = null;

  function initial(name) {
    const text = String(name || "?").trim();
    return text ? text[0].toUpperCase() : "?";
  }

  function setFriendMarkers(locations) {
    friendLayer.clearLayers();
    for (const loc of Array.isArray(locations) ? locations : []) {
      const lat = Number(loc.latitude);
      const lon = Number(loc.longitude);
      if (!Number.isFinite(lat) || !Number.isFinite(lon)) continue;
      const name = loc.user_name || "Friend";
      const icon = L.divIcon({
        className: "pin-friend",
        html: `<span class="pin-dot">${initial(name)}</span><span class="pin-label">${escapeHtml(name)}</span>`,
        iconSize: [0, 0],
      });
      L.marker([lat, lon], { icon, keyboard: false })
        .bindTooltip(escapeHtml(name), { direction: "top", offset: [0, -14] })
        .addTo(friendLayer);
    }
  }

  function setSelectedPlace(place) {
    if (selectedMarker) {
      map.removeLayer(selectedMarker);
      selectedMarker = null;
    }
    if (!place) return;
    const lat = Number(place.latitude ?? place.lat);
    const lon = Number(place.longitude ?? place.lon);
    if (!Number.isFinite(lat) || !Number.isFinite(lon)) return;
    const icon = L.divIcon({
      className: "pin-selected",
      html: `<span class="pin-dot"></span><span class="pin-label">${escapeHtml(place.short_name || place.display_name || "Selected")}</span>`,
      iconSize: [0, 0],
    });
    selectedMarker = L.marker([lat, lon], { icon }).addTo(map);
  }

  function clearRoute() {
    if (routeLayer) {
      map.removeLayer(routeLayer);
      routeLayer = null;
    }
  }

  function drawRoute(route) {
    clearRoute();
    const points = (route?.points || [])
      .map(([lat, lon]) => [Number(lat), Number(lon)])
      .filter(([lat, lon]) => Number.isFinite(lat) && Number.isFinite(lon));
    if (points.length < 2) return;
    const casing = L.polyline(points, { color: "#020617", weight: 9, opacity: 0.85 });
    const line = L.polyline(points, { color: "#34d399", weight: 5, opacity: 1 });
    routeLayer = L.layerGroup([casing, line]).addTo(map);

    const start = L.divIcon({
      className: "pin-route-end",
      html: '<span class="pin-dot pin-a">A</span>',
      iconSize: [0, 0],
    });
    const end = L.divIcon({
      className: "pin-route-end",
      html: '<span class="pin-dot pin-b">B</span>',
      iconSize: [0, 0],
    });
    L.marker(points[0], { icon: start, keyboard: false }).addTo(routeLayer);
    L.marker(points[points.length - 1], { icon: end, keyboard: false }).addTo(routeLayer);
  }

  function fitRoute(route) {
    const points = (route?.points || [])
      .map(([lat, lon]) => [Number(lat), Number(lon)])
      .filter(([lat, lon]) => Number.isFinite(lat) && Number.isFinite(lon));
    if (points.length < 2) return;
    map.fitBounds(L.latLngBounds(points), { padding: [48, 48], maxZoom: 16 });
  }

  function setSelfLocation(fix, { fly = false } = {}) {
    const lat = Number(fix?.lat ?? fix?.latitude);
    const lon = Number(fix?.lon ?? fix?.longitude);
    if (!Number.isFinite(lat) || !Number.isFinite(lon)) return;
    const accuracy = Number(fix?.accuracy ?? fix?.accuracy_m ?? 0);

    if (selfMarker) map.removeLayer(selfMarker);
    if (selfAccuracy) map.removeLayer(selfAccuracy);

    if (Number.isFinite(accuracy) && accuracy > 0 && accuracy < 5000) {
      selfAccuracy = L.circle([lat, lon], {
        radius: accuracy,
        color: "#34d399",
        weight: 1,
        opacity: 0.5,
        fillColor: "#34d399",
        fillOpacity: 0.08,
        interactive: false,
        className: "self-accuracy",
      }).addTo(map);
    }

    const selfIcon = L.divIcon({
      className: "pin-self",
      html: '<span class="pin-dot"></span><span class="pin-label">You</span>',
      iconSize: [0, 0],
    });
    selfMarker = L.marker([lat, lon], {
      icon: selfIcon,
      keyboard: false,
      zIndexOffset: 1000,
    }).addTo(map);

    if (fly) {
      map.flyTo([lat, lon], Math.max(map.getZoom(), 15), { duration: 0.6 });
    }
  }

  function clearSelfLocation() {
    if (selfMarker) {
      map.removeLayer(selfMarker);
      selfMarker = null;
    }
    if (selfAccuracy) {
      map.removeLayer(selfAccuracy);
      selfAccuracy = null;
    }
  }

  function locate() {
    return new Promise((resolve, reject) => {
      if (!navigator.geolocation) {
        reject(new Error("Geolocation is not available in this browser."));
        return;
      }
      navigator.geolocation.getCurrentPosition(
        (pos) => {
          const fix = {
            lat: pos.coords.latitude,
            lon: pos.coords.longitude,
            accuracy: pos.coords.accuracy,
          };
          setSelfLocation(fix, { fly: true });
          resolve(fix);
        },
        () => reject(new Error("Location permission denied — search instead.")),
        { enableHighAccuracy: true, timeout: 8000, maximumAge: 30000 },
      );
    });
  }

  return {
    map,
    setFriendMarkers,
    setSelectedPlace,
    setSelfLocation,
    clearSelfLocation,
    drawRoute,
    clearRoute,
    fitRoute,
    locate,
  };
}

function escapeHtml(value) {
  return String(value ?? "").replace(/[&<>"']/g, (c) => ({
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#39;",
  }[c]));
}
