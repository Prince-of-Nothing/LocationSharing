# 05 — User Stories and Use Cases

## User stories (acceptance-criteria-backed)

- **US-01** As a sharer, I grant each friend a precision (exact/approximate/
  city) and pause/revoke anytime. *→ FR-03/FR-04.*
- **US-02** As a recipient, I see only consented friends with freshness +
  accuracy. *→ FR-05.*
- **US-03** As a user, I pan/zoom the map with mouse, touch or keyboard and
  switch base layers. *→ FR-06.*
- **US-04** As a user, I search “café” or “Chișinău” and jump to the result.
  *→ FR-07.*
- **US-05** As a user, I tap any point to learn what/where it is. *→ FR-08.*
- **US-06** As a user, I route from my location to a friend/place by car,
  foot or bike and follow turn-by-turn steps (or hand off to OsmAnd /
  Organic Maps). *→ FR-09/FR-12.*
- **US-07** As a user, I browse food/fuel/pharmacy/etc. around me without
  typing. *→ FR-10.*
- **US-08** As a user, I star places and reuse them offline / export them.
  *→ FR-11.*
- **US-09** As a mobile user, I download an offline map pack and navigate
  offline. *→ FR-13/FR-16.*
- **US-10** As a desktop user, I open the same OSM destination in OSM,
  OSM directions or Organic Maps. *→ FR-18.*

## Key use cases

### UC-01 — Share and view (consent flow)
1. User A sends friendship → B accepts → A creates share (precision).
2. A publishes location → server fans out only to B.
3. B opens map → sees A at authorised precision + freshness.
Acceptance: C (non-friend) never sees A; paused/revoked stops immediately.

### UC-02 — Search and route
1. User types query → debounced Nominatim search (viewbox-biased).
2. Picks place → map flies to it; panel offers save/route/nav.
3. Enters from/to + mode → OSRM route renders + steps → hand off via `geo:`.
Acceptance: friend coordinates are never in the Nominatim/OSRM request logs.

### UC-03 — Browse POI and save
1. User taps “Load nearby” for a category → Overpass returns ≤60 POIs.
2. POIs render as markers + list; user opens detail (phone/web) or stars it.
3. Bookmark appears in saved list; export GeoJSON works offline.
Acceptance: no bulk Overpass calls; bookmarks are device-local.

### UC-04 — Offline (mobile)
1. User downloads current-area pack (progress shown).
2. Goes offline → map still renders the pack; `geo:` hand-off to Organic
   Maps/OsmAnd works.
Acceptance: attribution remains visible offline.

### UC-05 — Session restore (web)
1. User signs in (cookie + CSRF) → reloads → `POST /auth/session`
   revalidates → app shell re-enters without re-login.
Acceptance: 401 → clean login screen; no location cache shown.

## Actors

`Resident (sharer/recipient)`, `Administrator`, `OSM service (external)`,
`Nav app (external)`. System boundary: anything touching `/api/v1` is
inside; OSM services + nav apps are outside and untrusted for privacy.
