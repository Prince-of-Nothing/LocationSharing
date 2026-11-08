# 03 — Functional Requirements

Format: **ID — statement** · *Acceptance.* Traceability in 15.

## Sharing core (unchanged)

- **FR-01 — Accounts & sessions.** Users register/login with email+password;
  session cookie + CSRF double-submit (web) / bearer + CSRF (mobile/Rust);
  forced password change; logout revokes server-side.
  *Acceptance:* session survives reload via bootstrap; revoked sessions fail.
- **FR-02 — Friendships.** Send/accept/reject friendships; friendship alone
  grants no location access.
  *Acceptance:* non-friends see empty profiles; IDOR returns 403/404.
- **FR-03 — Directional shares.** Create/pause/resume shares with
  `direction` (mutual/to_friend/from_friend) + `precision`
  (exact/approximate/city_only) + optional purpose/expiry.
  *Acceptance:* only authorized recipients read; revoked/expired fail closed.
- **FR-04 — Publish location.** Publish current lat/lon (+accuracy);
  server fans out only to active consented shares; precision downgrade
  applied at read time.
  *Acceptance:* recipient sees downgraded precision; sharer sees own data.
- **FR-05 — Freshness.** Every friend position shows age + accuracy +
  live/stale/dead state (web 10/30 min; mobile/Rust equivalents).
  *Acceptance:* stale never rendered as live.

## Web map parity (new)

- **FR-06 — Interactive Slippy map.** Pan (drag/keys), zoom (wheel/pinch/
  buttons/dblclick), layer switch (dark/light/voyager/OSM), scale bar,
  coords readout, my-location centering. Centered on friends on first load.
  *Acceptance:* all input modes work; attribution stays visible; tiles load
  from allow-listed hosts only.
- **FR-07 — Search (Nominatim).** Debounced multi-field search with viewbox
  bias; results list; pick → fly-to + detail panel (center/save/route/nav).
  *Acceptance:* ≥3 chars required; ≤1 req/s; cached per session.
- **FR-08 — Reverse geocode.** Tap/click any point → human-readable place
  (Nominatim reverse). *Acceptance:* invalid coords fail with toast, not crash.
- **FR-09 — Routing (OSRM).** From/to as `lat,lon` or search text; 3 modes
  (driving/foot/bike); distance/duration/summary + ≤15 steps; polyline on
  map; GPX export; `geo:`/OSM hand-off; clear.
  *Acceptance:* no-route case shows error; steps are escaped text.
- **FR-10 — POI browse (Overpass).** 9 categories; query around visible
  centre (1.5 km); POI markers on map; list with name/coords/cuisine/hours;
  detail with website/Wikipedia/phone; save/route/nav actions.
  *Acceptance:* one query per tap; radius capped at 5 km; errors shown inline.
- **FR-11 — Bookmarks.** Star search/POI/route points; list/open/delete;
  GeoJSON export; localStorage-only, no sync.
  *Acceptance:* bookmarks survive reload offline; friend data never stored.
- **FR-12 — Navigation hand-off.** `geo:` URI, OSM directions, OSM place
  links; no extra private payload beyond chosen coordinates.
  *Acceptance:* links open correctly; no friend precision “un-rounding”.

## Mobile parity (new)

- **FR-13 — Vector map + offline packs.** MapLibre over OpenFreeMap;
  download/list/delete bbox packs (z 8–15); renders offline.
  *Acceptance:* attribution/logo enabled; packs delete cleanly.
- **FR-14 — Styles.** Liberty/Bright/Positron switch via `MapLayers`.
  *Acceptance:* switch changes style without data loss.
- **FR-15 — Mobile search (Nominatim).** Debounced search screen.
  *Acceptance:* same privacy rules as FR-07 (no friend coords).
- **FR-16 — Mobile routing (OSRM) + fallback.** Planner with 3 modes;
  straight-line engine remains as offline fallback; `geo:` hand-off.
  *Acceptance:* online route + steps; offline stub still works.
- **FR-17 — Mobile POI + bookmarks.** Overpass browse + DataStore bookmarks.
  *Acceptance:* same rules as FR-10/FR-11, on-device only.

## Desktop (Rust) parity

- **FR-18 — Correct OSM hand-off links.** OSM standard, OSM directions,
  Organic Maps web; ODbL credit on map panel. No Google-Earth link.
  *Acceptance:* all three open the right target at z14.

## Documentation & QA

- **FR-19 — SDLC docs.** This set + `docs/uml/12-15` + OSM integration guide.
- **FR-20 — Automatic QA.** `scripts/qa.sh`/`.ps1` + CI jobs (syntax,
  policy, attribution, CSP, Mermaid, links) green on push.
- **FR-21 — Licensing notices.** Both `THIRD_PARTY_NOTICES.md` updated for
  Nominatim/OSRM/Overpass + layer providers.
- **FR-22 — Traceability.** Every FR above has a row in 15-traceability.

## Non-goals (explicit)

Trails, heatmaps, mosaics, messaging, push, OAuth, E2E — see 02-scope.
