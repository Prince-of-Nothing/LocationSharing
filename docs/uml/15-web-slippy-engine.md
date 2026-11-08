# 15 — Web Slippy Engine (Components + Projection)

**Traceability:** FR-06 · NFR-01, NFR-09, NFR-10

## Component diagram (web)

```mermaid
flowchart LR
  config[config.js tileLayers + osm] --> map[map.js]
  api[api.js /api/v1 only] --> main[main.js wiring]
  session[session.js] --> main
  ui[ui.js esc/toast] --> map & main
  map --> tiles[tile grid img]
  map --> overlay[SVG route line]
  map --> markers[div pins]
  geocode --> N[(Nominatim)]
  routing --> R[(OSRM)]
  places --> O[(Overpass)]
  main --> map & geocode & routing & places & bookmarks & navigation
```

## Web Mercator projection (math)

```mermaid
flowchart TB
  A[lngLatToWorld lat lon z] --> B[x = lon+180/360 * n * 256]
  A --> C[y = 1 - ln tan pi/4+lat/2 / pi /2 * n * 256]
  D[worldToLngLat inverse] --> E[lon = x/n/256*360 - 180]
  D --> F[lat = atan sinh pi 1-2y/n/256 *180/pi]
  B & C --> G[tiles 256 x 256 at z/x/y]
```

## Pointer state machine (map input)

```mermaid
stateDiagram-v2
  [*] --> Idle
  Idle --> Dragging: pointerdown (1)
  Dragging --> Idle: pointerup (tap → reverse-geocode)
  Dragging --> Dragging: pointermove → pan
  Idle --> Pinching: pointerdown (2)
  Pinching --> Idle: pointerup (1 remains)
  Idle --> Zooming: wheel / dblclick / keys
  Zooming --> Idle
```

## Compliance gates (enforced by QA)

- Tiles only from `tileLayers` hosts (CARTO/Wikimedia); never
  `tile.openstreetmap.org`.
- `#map-attribution` always rendered (ODbL). CSP `img-src`/`connect-src`
  match `config.js`.
