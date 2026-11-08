# 16 — Glossary and References

## Glossary (selected)

| Term | Meaning |
|------|---------|
| OSM / ODbL | OpenStreetMap / Open Database License v1.0 (data share-alike) |
| OSMF | OpenStreetMap Foundation (policies, tile/search infrastructure governance) |
| Slippy map | 2D web map with 256 px tiles, Web Mercator, z/x/y addressing |
| Nominatim | OSM geocoding (search/reverse/lookup), GPLv2 |
| OSRM | Open Source Routing Machine (driving/foot/bike over OSM data) |
| Overpass | OSM database query language + API for POI browse |
| Tailor | Slippy tile server (Martin/TileServer GL) serving Geofabrik extracts |
| Geofabrik | OSM extracts (planet/country) for self-hosting |
| `geo:` URI | Standard deep link handled by OsmAnd/Organic Maps/Magic Earth |
| GPX / GeoJSON | Track / point exchange formats for nav-app import |
| Directional consent | Per-pair share record (precision/purpose/expiry/state) — friendship ≠ permission |
| Precision downgrade | Server-side rounding (exact→approximate→city_only) at read time |
| Stale/dead | Freshness states: warn (10–30 min) / stale (>30 min) |

## OSM references

- https://wiki.openstreetmap.org/wiki/Main_Page — project home (data model,
  policies, community).
- https://wiki.openstreetmap.org/wiki/Magic_Earth — Magic Earth (OSM-data
  consumer; routing/nav over OSM).
- https://organicmaps.app/ — Organic Maps (offline-first, OSM, no trackers).
- https://osmfoundation.org/wiki/Tile_Usage_Policy — why we never use
  `tile.openstreetmap.org`.
- https://operations.osmfoundation.org/policies/nominatim/ — Nominatim limits.
- https://opendatacommons.org/licenses/odbl/1-0/ — ODbL text.

## Internal references

- ADRs: `docs/adr/` (0001–0003, 0006 + consent/transport ADRs).
- UML: `docs/uml/` (01–15). Architecture: `docs/sdlc/08`. API:
  `docs/reference/api-contract.md` + `docs/sdlc/09`.
- Privacy: `docs/PRIVACY.md` + `docs/sdlc/10`. Deployment:
  `docs/DEPLOYMENT.md` + `docs/sdlc/12`. Notices: `web/` + `mobile/`
  `THIRD_PARTY_NOTICES.md`.
