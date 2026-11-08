// ---------------------------------------------------------------------------
// Routing client: OSRM over OSM data (driving / walking / cycling).
//
// OSRM demo server (router.project-osrm.org) is demo-only. Production
// deployments must self-host OSRM over Geofabrik extracts (or use
// Valhalla/GraphHopper). Route requests are user-initiated only:
// endpoints are the user's own position/friend visible position or a
// searched place — never background tracking.
// ---------------------------------------------------------------------------
import { osm, travelModes } from "./config.js";

/** Decode Google polyline (used by OSRM `overview=full` geometry). */
export function decodePolyline(str) {
  const points = [];
  let index = 0, lat = 0, lon = 0;
  while (index < str.length) {
    let shift = 0, result = 0, byte;
    do { byte = str.charCodeAt(index++) - 63; result |= (byte & 0x1f) << shift; shift += 5; } while (byte >= 0x20);
    const dLat = result & 1 ? ~(result >> 1) : result >> 1; lat += dLat;
    shift = 0; result = 0;
    do { byte = str.charCodeAt(index++) - 63; result |= (byte & 0x1f) << shift; shift += 5; } while (byte >= 0x20);
    const dLon = result & 1 ? ~(result >> 1) : result >> 1; lon += dLon;
    points.push({ latitude: lat / 1e5, longitude: lon / 1e5 });
  }
  return points;
}

export function formatDistance(meters) {
  if (meters == null || !Number.isFinite(meters)) return "—";
  if (meters < 1000) return `${Math.round(meters)} m`;
  return `${(meters / 1000).toFixed(1)} km`;
}

export function formatDuration(seconds) {
  if (seconds == null || !Number.isFinite(seconds)) return "—";
  const min = Math.round(seconds / 60);
  if (min < 60) return `${min} min`;
  const h = Math.floor(min / 60), m = min % 60;
  return m ? `${h} h ${m} min` : `${h} h`;
}

/** Humanise an OSRM step into a turn-by-turn instruction. */
export function stepText(step) {
  const maneuvers = {
    turn: "Turn", depart: "Depart", arrive: "Arrive", continue: "Continue",
    newname: "Continue", merge: "Merge", onramp: "Take the ramp", offramp: "Exit",
    fork: "Keep", endofroad: "At the end of the road, turn", roundabout: "At the roundabout, take",
    rotary: "At the roundabout, take", roundaboutturn: "At the roundabout, take", notification: "Continue",
  };
  const m = step?.maneuver || {};
  const base = maneuvers[m.type] || "Continue";
  const name = step?.name ? ` onto ${step.name}` : (step?.ref ? ` (${step.ref})` : "");
  const exit = m.exit ? ` (exit ${m.exit})` : "";
  const dist = step?.distance ? ` — ${formatDistance(step.distance)}` : "";
  return `${base}${name}${exit}${dist}`;
}

/**
 * Compute a route.
 * @param {{latitude:number,longitude:number}} from
 * @param {{latitude:number,longitude:number}} to
 * @param {{mode?:string, alternatives?:number}} opts
 */
export async function computeRoute(from, to, { mode = "driving", alternatives = 0 } = {}) {
  const profile = travelModes.find((m) => m.id === mode)?.profile || "driving";
  const coords = `${from.longitude},${from.latitude};${to.longitude},${to.latitude}`;
  const params = new URLSearchParams({
    overview: "full", geometries: "polyline", steps: "true",
    annotations: "false", alternatives: String(alternatives),
  });
  const url = `${osm.osrm.base}/route/v1/${profile}/${coords}?${params}`;
  const res = await fetch(url, { headers: { "Accept": "application/json" } });
  if (!res.ok) throw new Error(`Routing service error (HTTP ${res.status})`);
  const data = await res.json();
  if (data?.code !== "Ok" || !data.routes?.length) throw new Error("No route found between these points.");
  const r = data.routes[0];
  const points = r.geometry ? decodePolyline(r.geometry) : [];
  const steps = [];
  for (const leg of r.legs || []) for (const s of leg.steps || []) {
    steps.push({ text: stepText(s), distance: s.distance ?? null, duration: s.duration ?? null, name: s.name || "" });
  }
  return {
    profile, mode,
    points,
    distanceMeters: r.distance ?? null,
    durationSeconds: r.duration ?? null,
    summary: r.legs?.[0]?.summary || "",
    steps,
    alternatives: data.routes.slice(1).map((alt) => ({
      points: alt.geometry ? decodePolyline(alt.geometry) : [],
      distanceMeters: alt.distance ?? null,
      durationSeconds: alt.duration ?? null,
    })),
  };
}

/** Keep current coordinates string for copy/share. */
export function coordsString(lat, lon) { return `${Number(lat).toFixed(5)}, ${Number(lon).toFixed(5)}`; }
