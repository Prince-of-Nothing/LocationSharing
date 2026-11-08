// ---------------------------------------------------------------------------
// Navigation hand-off: geo: URIs + external deep links (OSM/Google) + GPX.
// No private friend payload is included beyond the coordinates the user
// chose to navigate to; precisions below what the share allows are never
// "un-rounded" here (renderMap already shows the authorised value only).
// ---------------------------------------------------------------------------

export function geoUri(lat, lon, { label = "", from = null, zoom = 16 } = {}) {
  const base = `geo:${lat},${lon}?z=${zoom}`;
  const parts = [base];
  if (from) parts.push(`from=${from.latitude},${from.longitude}`);
  if (label) parts.push(`q=${encodeURIComponent(label)}`);
  return parts.join("&").replace("?z=", "?z=").replace("&&", "&");
}

export function osmDirectionsUrl(to, { from = null } = {}) {
  const base = "https://www.openstreetmap.org/directions";
  const params = new URLSearchParams({ to: `${to.latitude},${to.longitude}` });
  if (from) params.set("from", `${from.latitude},${from.longitude}`);
  return `${base}?${params}`;
}

export function googleDirectionsUrl(to, { from = null } = {}) {
  const params = new URLSearchParams({ api: "1", destination: `${to.latitude},${to.longitude}` });
  if (from) params.set("origin", `${from.latitude},${from.longitude}`);
  return `https://www.google.com/maps/dir/?${params}`;
}

export function osmPlaceUrl(lat, lon, zoom = 16) {
  return `https://www.openstreetmap.org/#map=${zoom}/${lat}/${lon}`;
}

export function openExternal(url) {
  const a = document.createElement("a");
  a.href = url; a.target = "_blank"; a.rel = "noopener noreferrer";
  document.body.appendChild(a); a.click(); a.remove();
}

/** Minimal GPX track export for a route polyline (OsmAnd/Organic Maps import). */
export function routeToGpx(route, { name = "Pinpoint route" } = {}) {
  const trkpts = (route.points || []).map((p) => `    <trkpt lat="${p.latitude.toFixed(6)}" lon="${p.longitude.toFixed(6)}"/>`).join("\n");
  return `<?xml version="1.0" encoding="UTF-8"?>\n<gpx version="1.1" creator="Pinpoint" xmlns="http://www.topografix.com/GPX/1/1">\n  <trk><name>${name.replace(/[<>&"]/g, "")}</name><trkseg>\n${trkpts}\n  </trkseg></trk>\n</gpx>`;
}

export function downloadText(filename, text, mime = "application/gpx+xml") {
  const blob = new Blob([text], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url; a.download = filename;
  document.body.appendChild(a); a.click(); a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
