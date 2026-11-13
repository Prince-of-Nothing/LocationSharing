# OSM Map Parity — What to Borrow from Organic Maps, Magic Earth & Co.

> Status: analysis + prioritised plan. Pinpoint already has a solid OSM core
> (Slippy canvas, cached tiles, offline raster packs, Nominatim/OSRM/Overpass,
> bookmarks GeoJSON, ODbL attribution). This document maps the remaining gaps
> against the best OSM apps and decides what to build next.

## 1. What the reference apps do

| Feature | Organic Maps | Magic Earth | OsmAnd | Guru Maps | Komoot | Pinpoint today |
|---|---|---|---|---|---|---|
| Offline vector maps | ✅ per region | ✅ | ✅ | ✅ | ✅ | 🟡 raster tile packs |
| Offline search | ✅ | ✅ | ✅ | ✅ | 🟡 | ❌ (online Nominatim) |
| Offline routing | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ (online OSRM) |
| Turn-by-turn voice | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| GPX track record/export | ✅ | 🟡 | ✅ | ✅ | ✅ | 🟡 export (bookmarks only) |
| Bookmarks import/export | ✅ KML/KMZ/GPX/GeoJSON | 🟡 | ✅ GPX | ✅ GPX/KML | 🟡 | ✅ GeoJSON |
| POI details (phone/web/hours) | ✅ | ✅ | ✅ | ✅ | ✅ | 🟡 fetched, not shown |
| Contour lines / elevation | ✅ | ✅ | ✅ (plugin) | 🟡 | ✅ | ❌ |
| 3D terrain | 🟡 | ✅ | ✅ | ❌ | 🟡 | ❌ |
| Public transport | ✅ | ✅ | ✅ | 🟡 | ✅ | ❌ |
| Wikipedia / Wikivoyage | ✅ | 🟡 | ✅ | 🟡 | 🟡 | ❌ |
| OSM editing (POI/notes) | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ |
| Weather layers | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ |
| Live traffic / speed limits | ❌ | ✅ | 🟡 | ❌ | ❌ | ❌ |
| Parking / fuel prices | ❌ | ✅ | 🟡 | ❌ | ❌ | ❌ |
| Dark mode | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ (egui dark) |
| No trackers | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ |
| Android Auto / CarPlay | ✅ | ✅ | ✅ | 🟡 | ✅ | ❌ |
| Self-hosted tiles/routing | 🟡 | ❌ | ✅ | ❌ | ❌ | 🟡 (tile proxy) |

## 2. What Pinpoint already does well

- **Consent-first location sharing** (not a map-only app) — nothing to borrow,
  it's the differentiator.
- **Interactive Slippy canvas** with drag/zoom, cached tiles, precision rings,
  ODbL attribution.
- **Offline raster packs** (CARTO → Wikimedia → CARTO Voyager) with resume-safe
  downloads.
- **Nominatim / OSRM / Overpass** with throttling and privacy (friend coords
  never sent).
- **Bookmarks** with GeoJSON export/import.
- **E2E location encryption + OS keychain** (none of the map apps have this).
- **0 trackers / 0 ads / OSM-only**, matching Organic Maps/OsmAnd.

## 3. Gaps, ordered by value × effort

### P0 — quick wins (3-day sprint, desktop + Kotlin)

These use data Pinpoint already fetches; only the UI/export is missing.

1. **Show POI details** — the Overpass client already returns `phone`,
   `website`, `opening_hours`, `cuisine`; render them in the Places screen
   with "Save" (bookmark) and "Navigate" (OSM link) buttons.
2. **Show route steps** — OSRM already returns turn-by-turn steps; render the
   full list (distance/duration per step) instead of only the first step.
3. **GPX export** — `Route.points` already holds the decoded polyline; write a
   standards-compliant GPX 1.1 file (track + route points) to
   `~/.pinpoint/exports/`.
4. **Search result → bookmark** — one "Save" button next to each Nominatim
   result.
5. **Copy coordinates / open in OSM** — one click for any POI/result.
6. **Bookmark categories/notes** — the `Bookmark` model already has `note` and
   `category`; expose them in the add form.

### P1 — post-MVP (high value)

7. **Offline routing engine** — replace online-only OSRM with a local engine
   behind the existing `TravelMode`/route seam (Valhalla/GraphHopper
   integration or a Rust routing crate). Biggest UX win for travellers.
8. **Voice guidance** — TTS over the OSRM steps; platform-specific
   (`tts`/Android `TextToSpeech`/iOS `AVSpeechSynthesizer`).
9. **Contour lines / elevation profile** — add an elevation tile layer
   (OpenTopoMap/OSM-derived, ODbL) and a profile chart for routes.
10. **Wikipedia/Wikivoyage articles** — fetch by POI name/coords for the
    selected place (user-initiated only).
11. **Public transport routing** — OSRM doesn't do transit; needs
    OpenTripPlanner/Valhalla or a transit API.
12. **OSM editing** — add POI / create OSM notes from the map (OAuth to OSM,
    changeset API). Nice thesis demo, but a large surface.

### P2 — long-term / differentiators

13. **3D terrain** — vector tiles + MapLibre (Kotlin already has MapLibre; the
    egui desktop canvas is 2D-only).
14. **Live traffic / speed limits / parking / fuel** — Magic Earth's
    differentiators; require data feeds and licensing.
15. **Android Auto / CarPlay** — platform integrations.
16. **Self-hosted tiles/search/routing** — replace public demo endpoints
    before public traffic (see `docs/sdlc/07-openstreetmap-integration.md`).
17. **Vector tiles on desktop** — switch the raster canvas to vector tiles for
    sharper zoom and smaller packs.

## 4. What to borrow specifically

- **Organic Maps:** offline-first defaults, bookmark import/export breadth
  (KML/GPX/GeoJSON), no-tracker pledge, Wikipedia integration, OSM editing.
- **Magic Earth:** 3D terrain, public transport, weather/traffic layers
  (but those need data feeds; keep privacy-first by making them opt-in and
  user-initiated).
- **OsmAnd:** plugin model for contour/nautical/ski, GPX track recording,
  OsmAnd Cloud (optional E2E sync — aligns with our E2E keychain).
- **Komoot:** elevation profiles and route planning UX for hiking/cycling.
- **StreetComplete/Vespucci:** lightweight OSM contribution flow (notes first).

## 5. Privacy guardrails (do not lose these)

- Friend/shared coordinates are **never** sent to Nominatim/OSRM/Overpass.
- Only **user-initiated** queries (typed search, tapped POI, requested route).
- ODbL attribution is permanent and non-removable.
- Never `tile.openstreetmap.org`; use compliant providers or self-hosted tiles.
- New layers (weather/traffic) must be opt-in and documented in
  `THIRD_PARTY_NOTICES.md` if a new provider is added.

## 6. Recommendation for the 3-day sprint

Ship **P0 (1–6)** on the Rust desktop client (and mirror 1–4 in Kotlin if
time): it makes the map feel complete without new providers or licences.
Keep **P1** as the report's "next steps" and **P2** as the long-term roadmap.
