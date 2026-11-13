# 00 — Overview: Pinpoint OSM Map-Parity Release

## Release in one sentence

Pinpoint is a privacy-first, consent-based location-sharing platform. This
release adds **Organic-Maps / Magic-Earth-grade OSM map parity** to all three
clients (web, Android, Rust desktop) without weakened privacy: interactive
maps, search, routing, POI browse, bookmarks and offline packs over
**OpenStreetMap data (ODbL)**, with server-side consent enforcement unchanged.

## What changed in this release

| Area | Before | This release |
|------|--------|--------------|
| Web map | static centroid raster render | interactive Slippy map: pan/zoom (pointer, wheel, pinch, keyboard), layer switcher (CARTO dark/light/voyager, Wikimedia OSM), scale, coords, compass-style controls |
| Search | `geo:` / OSM deep links only | Nominatim search + reverse + lookup (debounced, throttled, cached) |
| Routing | none | OSRM driving/foot/bike with turn-by-turn steps, route polyline, GPX export, `geo:` + OSM hand-off |
| POI | none | Overpass browse around visible centre (9 categories), POI detail with website/phone/Wikipedia |
| Bookmarks | none | on-device saved places (localStorage / DataStore), GeoJSON export |
| Android | MapLibre + offline packs + `geo:` hand-off only | + Nominatim search, OSRM routing, Overpass POI, bookmarks, 3 vector styles (Liberty/Bright/Positron), route planner UI |
| Rust desktop | wrong Magic Earth link (pointed to Google Earth) | fixed: OSM standard, OSM directions, Organic Maps web, ODbL credit on the map panel |
| Docs | scattered MVP docs | this `docs/sdlc/` set: BRD, FR/NFR/TRS, lifecycle, risks, traceability |
| QA | CI (go, pytest, web syntax, mobile, rust) | + automatic map/compliance QA (tile policy, attribution, CSP, Mermaid, links) |

## Product principles (unchanged, now extended)

1. **Friendship ≠ permission.** Directional consent per (sharer → recipient)
   with precision (`exact`/`approximate`/`city_only`), purpose, expiry,
   pause/resume/revoke — enforced server-side.
2. **Honest freshness.** Every friend position shows age/accuracy and a
   live/stale/dead state. The map never implies unwarranted certainty.
3. **Privacy by default.** No trackers, no analytics, no third-party JS SDKs.
   OSM services receive only user-initiated queries; friend locations never
   reach them.
4. **OSM-first, policy-compliant.** All map data is OSM (ODbL) via
   compliant providers. Never `tile.openstreetmap.org` for app data.
   Attribution is permanent and non-removable.
5. **Offline where it matters.** Web: app shell + bookmarks offline.
   Mobile: full offline map packs (vector tiles) + `geo:` hand-off to
   offline nav apps (OsmAnd / Organic Maps / Magic Earth).

## Release shape

```
┌─────────────┐   HTTPS /api/v1   ┌──────────────────────┐
│ Web (PWA)   │ ────────────────▶ │ Go API (default)     │
│ Android     │ ────────────────▶ │  + Python alt API    │
│ Rust desktop│ ────────────────▶ └──────────────────────┘
└─────────────┘                            │
         │ OSM services (user-initiated)   │ Postgres / memory store
         ├─ CARTO/Wikimedia tiles (raster)
         ├─ OpenFreeMap (vector, mobile)
         ├─ Nominatim (search)  OSRM (routing)  Overpass (POI)
         └─ OSM wiki: https://wiki.openstreetmap.org/wiki/Main_Page
```

## How to read this set

- New to the project: 01 → 03 → 07 → 08.
- Implementing: 06 → 07 → 09 → 11.
- Operating / reviewing: 10 → 12 → 14 → 15.

## SDLC model

Pinpoint is built with the **spiral model** — a type of iterative-incremental
model. Each turn of the spiral runs four stages (goal elaboration → risk
analysis & prototyping → development of an interim version → planning the next
turn) and delivers a minimal finished version of the product, so risk gets
constant attention and the customer sees the product throughout. The full
spiral plan (turns S0–S6), exit criteria, pros/cons and critical path live in
[17-spiral-execution-plan.md](17-spiral-execution-plan.md).
The `docs/sdlc/00–16` set is the requirement/design/verification backbone that
each turn consumes and extends; `15-traceability-matrix.md` is the
requirement→test contract.
