# Self-Hosted OpenStreetMap Services

Companion to [ADR-0011](../adr/0011-self-hosted-osm-services.md). Everything
below is OSM data (ODbL) served from **your** infrastructure; the public
endpoints are demo fallbacks only.

## 1. Why self-host

- **Privacy:** user queries (typed text, coordinates, POI browse) stay on your
  infrastructure. Friend/shared coordinates never leave Pinpoint either way.
- **Policy:** Nominatim is ≤1 req/s and forbids bulk; the OSRM demo server is
  not for production; Overpass is fair-use. Self-hosting removes those limits.
- **Reliability:** no third-party rate limits, outages or term changes.
- **Control:** pin data versions, cache aggressively, log/retain on your terms.

**ODbL still requires attribution:** "© OpenStreetMap contributors" + the
licence link must remain visible. Self-hosting does not remove it.

## 2. Components

| Need | Recommended software | Data source |
|---|---|---|
| Raster/vector tiles | `maptiler/tileserver-gl`, `martin`, `planetiler` | Geofabrik `.osm.pbf` |
| Search / reverse | `mediagis/nominatim` | Geofabrik `.osm.pbf` |
| Routing | `osrm/osrm-backend` (or Valhalla/GraphHopper) | Geofabrik `.osm.pbf` |
| POI browse | `wiktorn/overpass-api` | Geofabrik `.osm.pbf` + `osm3s` DB |
| Transit (future) | OpenTripPlanner | GTFS + OSM |

Pick the **country/region extract** first (e.g. Moldova) — the planet is an
order of magnitude heavier.

### Approximate sizing

| Service | Country extract | Planet |
|---|---|---|
| Tiles | 2 vCPU / 4 GB RAM / 20 GB disk | 16 vCPU / 64 GB RAM / 1 TB disk |
| Nominatim | 2 vCPU / 4 GB RAM / 20 GB disk | 32 vCPU / 128 GB RAM / 2 TB disk |
| OSRM | 2 vCPU / 4 GB RAM / 10 GB disk | 16 vCPU / 64 GB RAM / 500 GB disk |
| Overpass | 2 vCPU / 4 GB RAM / 20 GB disk | 32 vCPU / 128 GB RAM / 2 TB disk |

## 3. Compose sketch

```yaml
services:
  nominatim:
    image: mediagis/nominatim:4.4
    environment:
      PBF_URL: https://download.geofabrik.de/europe/moldova-latest.osm.pbf
      REPLICATION_URL: https://download.geofabrik.de/europe/moldova-updates/
    ports: ["8081:8080"]
    volumes: ["nominatim-data:/var/lib/postgresql/14/main"]

  osrm:
    image: osrm/osrm-backend
    # prepare once: osrm-extract -p /opt/car.lua /data/moldova.osm.pbf
    #               osrm-partition /data/moldova.osrm
    #               osrm-customize /data/moldova.osrm
    command: osrm-routed --algorithm mld /data/moldova.osrm
    volumes: ["./data:/data"]
    ports: ["5000:5000"]

  tiles:
    image: maptiler/tileserver-gl
    volumes: ["./data:/data"]
    ports: ["8082:8080"]

  overpass:
    image: wiktorn/overpass-api
    volumes: ["./data:/db"]
    ports: ["8083:80"]

volumes:
  nominatim-data:
```

## 4. Point Pinpoint at it

Set these in the server/client environment (see `.env.example`):

```bash
PINPOINT_NOMINATIM_URL=http://nominatim:8080
PINPOINT_OSRM_URL=http://osrm:5000
PINPOINT_OVERPASS_URL=http://overpass:80/api/interpreter
PINPOINT_TILE_URL=http://tiles:8080/data/moldova/{z}/{x}/{y}.png
```

- The Rust client reads them in `src/osm.rs` (`nominatim_base`, `osrm_base`,
  `overpass_url`) and `src/tilepacks.rs` (`tile_candidate_urls`).
- The server tile proxy reads `PINPOINT_TILE_URL` first; public providers are
  demo fallbacks.
- Unset ⇒ the public demo defaults (local development only).

## 5. Attribution (mandatory)

- Every map view keeps "Map data © OpenStreetMap contributors (ODbL)" + the
  `https://www.openstreetmap.org/copyright` link.
- The tile proxy returns `X-Data-Attribution` on every response.
- `THIRD_PARTY_NOTICES.md` lists OSM/ODbL and the tile/search/routing software.
- QA (`scripts/qa.sh` / `map-qa.yml`) greps for attribution and forbids
  `tile.openstreetmap.org`.

## 6. Update cadence

| Data | Cadence | Command |
|---|---|---|
| Geofabrik extract | weekly | download new `.osm.pbf` |
| Nominatim | daily/weekly | replication (`REPLICATION_URL`) |
| OSRM | weekly | `osrm-extract` → `osrm-partition` → `osrm-customize` |
| Tiles | weekly/monthly | regenerate with `planetiler`/`tilemaker` |
| Overpass | weekly | `update_database` against the new extract |

## 7. Monitoring & backups

- Health: each service's `/health` (Nominatim `/status`, OSRM `nearest`,
  Overpass `status`, tiles health endpoint).
- Metrics: query latency, 4xx/5xx, disk/RAM, last successful import.
- Backups: the `.osm.pbf` + generated indexes; keep one previous version for
  rollback.
- Alerts: import failure, disk > 80 %, query latency p95.

## 8. Fallback policy

- **Local development:** public demo endpoints are acceptable.
- **Production:** set all four env vars; if a self-hosted service is down, the
  UI shows a graceful error — it must **not** silently fall back to public
  endpoints (privacy + policy).
- **Never** `tile.openstreetmap.org`, even as a fallback.
