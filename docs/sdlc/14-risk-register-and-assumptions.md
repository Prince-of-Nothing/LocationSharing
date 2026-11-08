# 14 — Risk Register and Assumptions

| ID | Risk / issue | Likelihood | Impact | Mitigation / owner |
|----|--------------|------------|--------|--------------------|
| R-01 | Public Nominatim/OSRM/Overpass rate-limit or block | Medium | Search/route/POI fail live | Throttles + cache; self-host path (07); graceful errors |
| R-02 | Tile host ToS change / outage | Low | Web tiles blank | 4-layer switcher; self-host Martin |
| R-03 | Attribution removed accidentally | Medium | ODbL violation | QA grep (blocker); notices; code review |
| R-04 | Friend data leaks to OSM services | Low | Privacy breach | User-input-only queries; review + tests |
| R-05 | Offline tile cache violates ToS | Low | Legal | Web never caches tiles; mobile user-downloads only |
| R-06 | New JS dep adds tracker/CDN | Low | Privacy + supply chain | Zero-dep rule; CSP structural block |
| R-07 | Mobile MapLibre API drift breaks build | Medium | Build failure | Isolated in `:core:map` (2 files); CI compile |
| R-08 | OSRM demo discontinued | Medium | Routing down | Hand-off `geo:` fallback; Valhalla/GraphHopper prep |
| R-09 | Store/algorithm drift (Go vs Python) | Medium | Auth/location bugs | Mirror deliberately; conformance suite |
| R-10 | Location precision misread as exact | Medium | Safety overconfidence | Freshness + accuracy always shown; stale banner |
| R-11 | Demo credentials in wrong env | Low | Account takeover | Local-only seeds; rotate before prod |
| R-12 | Bookmark sync pressure (future) | Low | Privacy scope creep | Keep on-device; any sync needs threat analysis |

## Assumptions (from 02)

A-1 public OSM demo availability · A-2 keyless providers stay friendly ·
A-3 ODbL unchanged · A-4 HTTPS+GPS devices · A-5 thesis-scale review.
Revisit each assumption at every release.

## Open issues / decisions needed

- Production OSM hosting budget (tiles/search/routing/POI): donate,
  self-host on VPS, or managed (MapTiler etc.)?
- Offline routing engine: GraphHopper vs Valhalla vs hand-off-only?
- Realtime (WebSocket) priority vs polling?
