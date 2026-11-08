# 15 — Traceability Matrix (SDLC release)

| Req | Business goal | Design | Implementation | Evidence |
|-----|---------------|--------|---------------|----------|
| FR-01 | BG-1 | api/session | `web/js/session.js`, mobile `SessionStore`, Rust `api.rs` | Go `TestAuth*`; manual login QA |
| FR-02 | BG-1 | friendships | `friendships.go`, web/mobile friends | `TestFriendship*` |
| FR-03/04 | BG-1 | shares/locations | `locations.go`, `store/*` | `TestShare*`, `TestLocationPublish*` |
| FR-05 | BG-1 | freshness | web `map.js`, mobile `FriendLocation.isStale` | `TestFreshness*` |
| FR-06 | BG-2 | Slippy engine | web `js/map.js`, `index.html`, `app.css` | manual map QA + `scripts/qa.sh` |
| FR-07/08 | BG-2 | Nominatim | web `js/geocode.js`, mobile `OsmServices.kt` | manual search QA; policy grep |
| FR-09 | BG-2 | OSRM | web `js/routing.js`, mobile `OsrmRoutingEngine` | manual route QA |
| FR-10 | BG-2 | Overpass | web `js/places.js`, mobile `PoiRepositoryImpl` | manual POI QA |
| FR-11 | BG-2/3 | bookmarks | web `js/bookmarks.js`, mobile `BookmarkRepositoryImpl` | offline bookmark QA |
| FR-12 | BG-2 | hand-off | web `js/navigation.js`, mobile `GeoNavigation.kt` | manual `geo:` QA |
| FR-13/14 | BG-2 | offline+styles | mobile `OfflineMapManager`, `MapLayers.kt` | APK offline QA |
| FR-15/16/17 | BG-2 | mobile OSM UI | `SearchScreen`, `RoutePlannerScreen`, `PlacesScreen` | manual mobile QA |
| FR-18 | BG-2 | desktop links | `src/map.rs` (OSM/directions/Organic) | manual Rust QA |
| FR-19 | BG-4 | docs | `docs/sdlc/`, `docs/uml/12-15` | docs review |
| FR-20 | BG-4 | auto QA | `scripts/qa.sh/.ps1`, `map-qa.yml` | CI `map-qa` green |
| FR-21 | BG-5 | licensing | both `THIRD_PARTY_NOTICES.md` | `scripts/qa.sh` notices check |
| FR-22 | BG-4 | this file | this matrix | reviewer sign-off |
| NFR-01…18 | BG-3/4/5 | see 04 | see 06/07/08 | see 11 |

Deferred features (trails, heatmaps, mosaics, messaging, push, OAuth, E2E)
intentionally have **no** rows — they are out of scope (02) until their
own threat analysis lands.
