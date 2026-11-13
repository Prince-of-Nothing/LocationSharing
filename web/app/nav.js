// Search, place sheet and route panel wiring.
// All requests are same-origin; no user identity or friend data is sent to OSM.

const $ = (id) => document.getElementById(id);
const MIN_QUERY = 3;

export function initNavigation({ mapApi, setStatus, getUserId }) {
  const form = $("search-form");
  const input = $("search-input");
  const results = $("search-results");
  const chips = $("category-chips");
  const placeSheet = $("place-sheet");
  const routePanel = $("route-panel");
  const savedList = $("saved-list");
  const recentList = $("recent-list");

  const capabilities = { autocomplete: false, modes: ["driving"] };
  let searchController = null;
  let debounceTimer = null;
  let selected = null;
  let route = {
    origin: null,
    destination: null,
    mode: "driving",
    routes: [],
    index: 0,
    controller: null,
    watchId: null,
  };

  // --- per-account on-device storage (never leaves the browser) ------------
  function accountKey(kind) {
    const id = typeof getUserId === "function" ? getUserId() : "anon";
    return `pinpoint.v1.${id}.${kind}`;
  }

  function loadList(kind) {
    try {
      const raw = localStorage.getItem(accountKey(kind));
      const parsed = raw ? JSON.parse(raw) : [];
      return Array.isArray(parsed) ? parsed.filter(isPlaceRecord) : [];
    } catch (_) {
      return [];
    }
  }

  function saveList(kind, list) {
    try {
      localStorage.setItem(accountKey(kind), JSON.stringify(list.slice(0, 50)));
    } catch (_) {
      // quota/private mode: keep the app usable, just don't persist
    }
  }

  function isPlaceRecord(value) {
    return (
      value &&
      typeof value === "object" &&
      typeof value.display_name === "string" &&
      Number.isFinite(Number(value.latitude)) &&
      Number.isFinite(Number(value.longitude))
    );
  }

  function placeRecord(place) {
    return {
      display_name: String(place.display_name || ""),
      short_name: String(place.short_name || place.display_name || "Place"),
      latitude: Number(place.latitude),
      longitude: Number(place.longitude),
      category: String(place.category || ""),
    };
  }

  function samePlace(a, b) {
    return (
      a.display_name === b.display_name &&
      Math.abs(Number(a.latitude) - Number(b.latitude)) < 1e-6 &&
      Math.abs(Number(a.longitude) - Number(b.longitude)) < 1e-6
    );
  }

  function addRecent(place) {
    const record = placeRecord(place);
    if (!isPlaceRecord(record)) return;
    const recents = loadList("recents").filter((item) => !samePlace(item, record));
    recents.unshift(record);
    saveList("recents", recents);
  }

  function updateSaveButton() {
    const btn = $("place-save");
    if (!selected) return;
    const record = placeRecord(selected);
    const saved = loadList("savedPlaces").some((item) => samePlace(item, record));
    btn.textContent = saved ? "Saved ✓" : "Save";
    btn.setAttribute("aria-pressed", String(saved));
  }

  function renderStorage() {
    const saved = loadList("savedPlaces");
    const recents = loadList("recents");
    savedList.innerHTML = saved.length ? "" : '<li class="muted">No saved places.</li>';
    recentList.innerHTML = recents.length ? "" : '<li class="muted">No recent places.</li>';

    for (const place of saved) {
      savedList.appendChild(storageRow(place, "savedPlaces"));
    }
    for (const place of recents) {
      recentList.appendChild(storageRow(place, "recents"));
    }
    updateSaveButton();
  }

  function storageRow(place, kind) {
    const li = document.createElement("li");
    const show = document.createElement("button");
    show.type = "button";
    show.className = "link";
    show.textContent = place.short_name || place.display_name || "Place";
    show.addEventListener("click", () => selectPlace(place));
    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "mini";
    remove.setAttribute("aria-label", `Remove ${show.textContent}`);
    remove.textContent = "×";
    remove.addEventListener("click", () => {
      saveList(
        kind,
        loadList(kind).filter((item) => !samePlace(item, place)),
      );
      renderStorage();
    });
    li.append(show, remove);
    return li;
  }

  fetch("/map/capabilities")
    .then((r) => (r.ok ? r.json() : null))
    .then((caps) => {
      if (!caps) return;
      capabilities.autocomplete = Boolean(caps.autocomplete);
      capabilities.modes = Array.isArray(caps.modes) ? caps.modes : ["driving"];
      applyModeAvailability();
    })
    .catch(() => {});

  function applyModeAvailability() {
    document.querySelectorAll(".mode").forEach((btn) => {
      const mode = btn.dataset.mode;
      const enabled = capabilities.modes.includes(mode);
      btn.disabled = !enabled;
      btn.title = enabled
        ? ""
        : "Self-host foot/bike routing to enable this mode.";
    });
  }

  function showResults(open) {
    results.classList.toggle("hidden", !open);
    chips.classList.toggle("hidden", !open || input.value.trim().length > 0);
  }

  function hidePlace() {
    placeSheet.classList.add("hidden");
    mapApi.setSelectedPlace(null);
    selected = null;
  }

  function showPlace(place) {
    selected = place;
    $("place-name").textContent = place.short_name || place.display_name || "Place";
    $("place-address").textContent = place.display_name || "";
    $("place-distance").textContent = Number.isFinite(place.distance_m)
      ? `${formatDistance(place.distance_m)} from map centre`
      : "";
    placeSheet.classList.remove("hidden");
    mapApi.setSelectedPlace(place);
  }

  async function runSearch(query) {
    const q = query.trim();
    if (q.length < MIN_QUERY) {
      setStatus(`Type at least ${MIN_QUERY} characters.`, true);
      return;
    }
    if (searchController) searchController.abort();
    searchController = new AbortController();
    const center = mapApi.map.getCenter();
    try {
      const url = `/map/search?q=${encodeURIComponent(q)}&lat=${center.lat.toFixed(5)}&lon=${center.lng.toFixed(5)}&limit=8`;
      const res = await fetch(url, { signal: searchController.signal, credentials: "include" });
      const body = await res.json().catch(() => ({}));
      if (!res.ok) throw new Error(body?.error?.message || "Search failed.");
      renderResults(Array.isArray(body.results) ? body.results : []);
    } catch (err) {
      if (err.name === "AbortError") return;
      setStatus(err.message, true);
      renderResults([]);
    }
  }

  function renderResults(items) {
    results.innerHTML = "";
    if (!items.length) {
      results.innerHTML = '<div class="result muted">No places found near here.</div>';
    } else {
      for (const place of items) {
        const btn = document.createElement("button");
        btn.type = "button";
        btn.className = "result";
        const name = document.createElement("span");
        name.className = "name";
        name.textContent = place.short_name || place.display_name || "Place";
        const meta = document.createElement("span");
        meta.className = "meta";
        const dist = Number.isFinite(place.distance_m) ? ` · ${formatDistance(place.distance_m)}` : "";
        meta.textContent = `${place.display_name || ""}${dist}`;
        btn.append(name, meta);
        btn.addEventListener("click", () => {
          showResults(false);
          selectPlace(place);
        });
        results.appendChild(btn);
      }
    }
    showResults(true);
  }

  function selectPlace(place, fly = true) {
    showPlace(place);
    showResults(false);
    addRecent(place);
    renderStorage();
    const lat = Number(place.latitude);
    const lon = Number(place.longitude);
    if (fly && Number.isFinite(lat) && Number.isFinite(lon)) {
      mapApi.map.flyTo([lat, lon], Math.max(mapApi.map.getZoom(), 15), { duration: 0.5 });
    }
  }

  async function reverseAt(lat, lon) {
    setStatus("Looking up this place…");
    try {
      const res = await fetch(
        `/map/reverse?lat=${lat.toFixed(5)}&lon=${lon.toFixed(5)}`,
        { credentials: "include" },
      );
      const body = await res.json().catch(() => ({}));
      if (!res.ok) throw new Error(body?.error?.message || "Could not look up this place.");
      selectPlace(body, false);
      setStatus("");
    } catch (err) {
      setStatus(err.message, true);
    }
  }

  function openRouteFor(place) {
    route.origin = route.origin || centerOrigin();
    route.destination = {
      lat: Number(place.latitude),
      lon: Number(place.longitude),
      label: place.short_name || place.display_name || "Destination",
    };
    renderRouteFields();
    routePanel.classList.remove("hidden");
    placeSheet.classList.add("hidden");
    planRoute();
  }

  function centerOrigin() {
    const c = mapApi.map.getCenter();
    return { lat: c.lat, lon: c.lng, label: `Map centre (${c.lat.toFixed(4)}, ${c.lng.toFixed(4)})` };
  }

  function renderRouteFields() {
    $("route-from").value = route.origin?.label || "Choose origin";
    $("route-to").value = route.destination?.label || "Choose destination";
  }

  async function planRoute() {
    if (!route.origin || !route.destination) return;
    if (route.controller) route.controller.abort();
    route.controller = new AbortController();
    $("route-summary").textContent = "Routing…";
    $("route-steps").innerHTML = "";
    $("route-alternatives").innerHTML = "";
    try {
      const params = new URLSearchParams({
        from_lat: route.origin.lat.toFixed(6),
        from_lon: route.origin.lon.toFixed(6),
        to_lat: route.destination.lat.toFixed(6),
        to_lon: route.destination.lon.toFixed(6),
        mode: route.mode,
        alternatives: "true",
      });
      const res = await fetch(`/map/route?${params}`, {
        signal: route.controller.signal,
        credentials: "include",
      });
      const body = await res.json().catch(() => ({}));
      if (!res.ok) throw new Error(body?.error?.message || "Routing failed.");
      route.routes = Array.isArray(body.routes) ? body.routes : [];
      if (!route.routes.length) throw new Error("No route found between these points.");
      route.index = 0;
      renderAlternatives();
      applyRoute(0);
    } catch (err) {
      if (err.name === "AbortError") return;
      $("route-summary").textContent = err.message;
      setStatus(err.message, true);
    }
  }

  function applyRoute(index) {
    const chosen = route.routes[index];
    if (!chosen) return;
    route.index = index;
    mapApi.drawRoute(chosen);
    mapApi.fitRoute(chosen);
    $("route-summary").textContent = `${formatDistance(chosen.distance_m)} · ${formatDuration(chosen.duration_s)} · ${route.mode}`;
    const steps = $("route-steps");
    steps.innerHTML = "";
    for (const step of chosen.steps || []) {
      const li = document.createElement("li");
      const distance = Number.isFinite(step.distance_m) ? ` (${formatDistance(step.distance_m)})` : "";
      li.textContent = `${step.text}${distance}`;
      steps.appendChild(li);
    }
    document
      .querySelectorAll("#route-alternatives button")
      .forEach((btn, i) => btn.classList.toggle("active", i === index));
  }

  function renderAlternatives() {
    const wrap = $("route-alternatives");
    wrap.innerHTML = "";
    if (route.routes.length < 2) return;
    route.routes.forEach((candidate, i) => {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.textContent = `${formatDistance(candidate.distance_m)} · ${formatDuration(candidate.duration_s)}`;
      btn.addEventListener("click", () => applyRoute(i));
      wrap.appendChild(btn);
    });
  }

  function formatDuration(seconds) {
    const s = Number(seconds);
    if (!Number.isFinite(s)) return "";
    if (s < 3600) return `${Math.max(1, Math.round(s / 60))} min`;
    const h = Math.floor(s / 3600);
    const m = Math.round((s % 3600) / 60);
    return `${h} h ${m} min`;
  }

  function setMode(mode) {
    if (!capabilities.modes.includes(mode)) return;
    route.mode = mode;
    document
      .querySelectorAll(".mode")
      .forEach((btn) => btn.classList.toggle("active", btn.dataset.mode === mode));
    if (route.origin && route.destination) planRoute();
  }

  function swapEndpoints() {
    const tmp = route.origin;
    route.origin = route.destination;
    route.destination = tmp;
    renderRouteFields();
    if (route.origin && route.destination) planRoute();
  }

  function stopFollowing() {
    if (route.watchId != null) {
      navigator.geolocation.clearWatch(route.watchId);
      route.watchId = null;
    }
  }

  function showNavBanner(step, meta, { retry = false } = {}) {
    $("nav-step").textContent = step;
    $("nav-meta").textContent = meta || "";
    $("nav-retry").classList.toggle("hidden", !retry);
    $("nav-banner").classList.remove("hidden");
  }

  function hideNavBanner() {
    $("nav-banner").classList.add("hidden");
    $("nav-retry").classList.add("hidden");
  }

  function exitNavigation() {
    stopFollowing();
    mapApi.clearSelfLocation();
    hideNavBanner();
  }

  function clearRoutePanel() {
    exitNavigation();
    route.routes = [];
    route.destination = null;
    mapApi.clearRoute();
    routePanel.classList.add("hidden");
    $("route-steps").innerHTML = "";
    $("route-alternatives").innerHTML = "";
    $("route-summary").textContent = "";
  }

  function geolocationErrorMessage(err) {
    if (err?.code === 1) {
      return "Location permission is blocked for this site.";
    }
    if (err?.code === 3) {
      return "Location request timed out.";
    }
    return "Your location is currently unavailable.";
  }

  function enterNavigationPreview() {
    const chosen = route.routes[route.index];
    const step = chosen?.steps?.[0]?.text || chosen?.summary || "Route preview";
    const meta = chosen
      ? `${formatDistance(chosen.distance_m)} · ${formatDuration(chosen.duration_s)} · ${route.mode}`
      : "Route preview";
    showNavBanner(step, meta);
  }

  function startFollowing() {
    if (!route.origin || !route.destination) {
      setStatus("Plan a route first.", true);
      return;
    }
    // Navigation always starts in preview mode; GPS is an enhancement, not a
    // prerequisite. A denied permission must never make Start a dead end.
    enterNavigationPreview();
    if (!navigator.geolocation) {
      showNavBanner("Location unavailable", "Route preview only — this browser has no geolocation.", {
        retry: false,
      });
      return;
    }
    stopFollowing();
    let firstFix = true;
    route.watchId = navigator.geolocation.watchPosition(
      (pos) => {
        const fix = {
          lat: pos.coords.latitude,
          lon: pos.coords.longitude,
          accuracy: pos.coords.accuracy,
        };
        mapApi.setSelfLocation(fix);
        route.origin = { lat: fix.lat, lon: fix.lon, label: "My location" };
        renderRouteFields();
        const chosen = route.routes[route.index];
        const step = chosen?.steps?.[0]?.text || "Following route";
        const meta = chosen
          ? `Following · ${formatDistance(chosen.distance_m)} · ${formatDuration(chosen.duration_s)}`
          : "Following";
        showNavBanner(step, meta);
        if (firstFix) {
          mapApi.map.panTo([fix.lat, fix.lon], { animate: true });
          firstFix = false;
        }
        setStatus("Following your location. Route preview only — no voice guidance.");
      },
      (err) => {
        const message = geolocationErrorMessage(err);
        showNavBanner("Location unavailable", `${message} Route preview is still active.`, {
          retry: true,
        });
        setStatus(`${message} Allow location and press Enable location.`, true);
      },
      { enableHighAccuracy: true, maximumAge: 10000, timeout: 12000 },
    );
  }

  function onMapClick({ lat, lon }) {
    if (!routePanel.classList.contains("hidden")) {
      route.destination = {
        lat,
        lon,
        label: `Dropped pin (${lat.toFixed(4)}, ${lon.toFixed(4)})`,
      };
      renderRouteFields();
      planRoute();
      return;
    }
    reverseAt(lat, lon);
  }

  // Search form: submit is always allowed; live suggestions only when the
  // deployment is self-hosted (Nominatim policy forbids public autocomplete).
  form.addEventListener("submit", (e) => {
    e.preventDefault();
    runSearch(input.value);
  });
  input.addEventListener("input", () => {
    if (!capabilities.autocomplete) return;
    clearTimeout(debounceTimer);
    const q = input.value.trim();
    if (q.length < MIN_QUERY) {
      showResults(false);
      return;
    }
    debounceTimer = setTimeout(() => runSearch(q), 300);
  });
  input.addEventListener("focus", () => {
    if (input.value.trim().length >= MIN_QUERY || !capabilities.autocomplete) {
      showResults(true);
    }
  });

  chips.querySelectorAll("button[data-q]").forEach((btn) => {
    btn.addEventListener("click", () => {
      input.value = btn.dataset.q;
      runSearch(btn.dataset.q);
    });
  });

  $("place-close").addEventListener("click", hidePlace);
  $("place-directions").addEventListener("click", () => {
    if (selected) openRouteFor(selected);
  });
  $("place-origin").addEventListener("click", () => {
    if (!selected) return;
    route.origin = {
      lat: Number(selected.latitude),
      lon: Number(selected.longitude),
      label: selected.short_name || selected.display_name || "Origin",
    };
    renderRouteFields();
    if (route.destination) {
      routePanel.classList.remove("hidden");
      planRoute();
    } else {
      setStatus("Origin set — choose a destination from search or the map.");
    }
  });
  $("place-copy").addEventListener("click", async () => {
    if (!selected) return;
    const text = `${selected.display_name || ""}\n${selected.latitude}, ${selected.longitude}`;
    try {
      await navigator.clipboard.writeText(text);
      setStatus("Place copied.");
    } catch (_) {
      setStatus("Select the address and copy it manually.", true);
    }
  });
  $("place-osm").addEventListener("click", () => {
    if (!selected) return;
    const url = `https://www.openstreetmap.org/?mlat=${selected.latitude}&mlon=${selected.longitude}#map=16/${selected.latitude}/${selected.longitude}`;
    window.open(url, "_blank", "noopener");
  });
  $("place-save").addEventListener("click", () => {
    if (!selected) return;
    const record = placeRecord(selected);
    const saved = loadList("savedPlaces");
    const index = saved.findIndex((item) => samePlace(item, record));
    if (index >= 0) {
      saved.splice(index, 1);
      setStatus("Removed from saved places.");
    } else {
      saved.unshift(record);
      setStatus("Saved on this device.");
    }
    saveList("savedPlaces", saved);
    renderStorage();
  });
  $("clear-saved").addEventListener("click", () => {
    saveList("savedPlaces", []);
    renderStorage();
    setStatus("Saved places cleared for this account.");
  });
  $("clear-recents").addEventListener("click", () => {
    saveList("recents", []);
    renderStorage();
    setStatus("Recent places cleared for this account.");
  });

  $("route-close").addEventListener("click", () => {
    clearRoutePanel();
  });
  $("nav-exit").addEventListener("click", exitNavigation);
  $("nav-retry").addEventListener("click", startFollowing);

  document.querySelectorAll(".mode").forEach((btn) => {
    btn.addEventListener("click", () => setMode(btn.dataset.mode));
  });
  $("route-swap").addEventListener("click", swapEndpoints);
  $("route-start").addEventListener("click", startFollowing);
  $("route-clear").addEventListener("click", clearRoutePanel);

  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      showResults(false);
      hidePlace();
    }
    if (e.key === "/" && document.activeElement !== input) {
      e.preventDefault();
      input.focus();
    }
  });

  renderStorage();

  return {
    onMapClick,
    route: () => route,
    selected: () => selected,
    selectPlace,
    openRouteFor,
    refreshStorage: renderStorage,
  };
}

export function formatDistance(meters) {
  const m = Number(meters);
  if (!Number.isFinite(m)) return "";
  return m < 1000 ? `${Math.round(m)} m` : `${(m / 1000).toFixed(1)} km`;
}
