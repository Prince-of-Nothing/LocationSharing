# 01 — Business Requirements Document (BRD)

## 1. Problem

Existing location-sharing products conflate social relationships with data
access. Being “friends” usually means blanket visibility, enabling
accidental over-sharing, stalking and forgotten grants. For vulnerable
users the harm is physical. At the same time, the tools people reach for
when they need to *find* and *navigate to* a shared position (Organic
Maps, Magic Earth, OsmAnd, Google Maps) are either proprietary or lack
consent enforcement.

Pinpoint exists to close that gap: a consent-based sharing layer on top of
open OSM maps with Organic-Maps-grade usability.

## 2. Business goals

| ID | Goal | Measure |
|----|------|---------|
| BG-1 | Make consent the default, not the exception | 100% of location reads authorized server-side; zero IDOR leakage in tests |
| BG-2 | Make OSM maps genuinely useful (parity) | Interactive web map + Android map with search/routing/POI/bookmarks/offline |
| BG-3 | Keep the privacy stance credible | 0 trackers; CSP-enforced; no friend data to third parties |
| BG-4 | Keep deployment reproducible for thesis review | One-command `docker compose up --build` + documented SDLC traceability |
| BG-5 | Control OSM provider risk | No `tile.openstreetmap.org` app use; self-hosting path documented |

## 3. Stakeholders

| Stakeholder | Interest | In scope |
|-------------|----------|----------|
| End users (sharers/recipients) | Control who sees them, at what precision, how long; navigate to them | Consent UX, map, freshness |
| Thesis examiners / mentors | Reproducible, secure, documented work | This SDLC set, ADRs, RTM, tests |
| OSM community / OSMF | Correct ODbL attribution, policy compliance | Notices, CSP, rate limits, self-host path |
| Operators | Cheap, humane deployment | Docker stack, runbooks, monitoring |

## 4. Business constraints

- No advertising, no data monetization — ever (non-goal).
- No covert monitoring features: the product protects the sharer.
- No bulk OSM scraping; public OSM services are user-initiated only.
- All new features must preserve server-side enforcement and ODbL duties.

## 5. Success criteria (release exit)

- All FR-01…FR-22 implemented on web + Android (desktop: map links + credit).
- NFR security/privacy evidence green in CI (`scripts/qa.sh` + `ci.yml`).
- Tile policy verified: no `tile.openstreetmap.org` anywhere in clients.
- Attribution visible on every map view (web `#map-attribution`, Android
  MapLibre logo+attribution, Rust map panel credit).
- One-command demo boots: `docker compose up --build` → http://localhost:8080.

## 6. Out-of-business-scope (deferred, not rejected)

Movement trails, visited-region heatmaps, friend mosaics, messaging/calls,
push notifications, OAuth/OIDC, E2E encryption — each needs its own threat
analysis (see 14-risk-register). They are **not** acceptance criteria here.
