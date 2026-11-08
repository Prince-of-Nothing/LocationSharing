# 12 — Map Search & Places (Sequence + Components)

**Traceability:** FR-07, FR-08, FR-10, FR-15, FR-17 · NFR-02, NFR-06, NFR-07

## Sequence: search → pick → save/route

```mermaid
sequenceDiagram
  actor U as User
  participant M as Map client (web/mobile)
  participant N as Nominatim (search)
  participant O as Overpass (POI)
  participant L as Bookmarks (device)
  U->>M: Type "café" (debounced 350 ms)
  M->>N: GET /search?q=café&viewbox=… (user text only)
  N-->>M: [SearchPlace…]
  M->>U: Results list
  U->>M: Pick place
  M->>M: Fly-to + detail panel
  M->>L: ★ Save (on-device only)
  U->>M: Load nearby (Food)
  M->>O: POST /interpreter (centre + category)
  O-->>M: [Poi…] (≤60)
  M->>M: Render markers + list
```

## Component diagram (web)

```mermaid
flowchart LR
  main[main.js wiring] --> geocode[geocode.js Nominatim]
  main --> places[places.js Overpass]
  main --> bookmarks[bookmarks.js localStorage]
  main --> navigation[navigation.js geo:/GPX]
  map[map.js Slippy engine] --> config[config.js layers+endpoints]
  geocode --> N[(Nominatim)]
  places --> O[(Overpass)]
```

## Activity: POI browse guardrails

```mermaid
stateDiagram-v2
  [*] --> Idle
  Idle --> Debounced: user types (≥3 chars)
  Debounced --> Search: Nominatim OK
  Search --> Idle: results cached
  Idle --> Browse: tap "Load nearby"
  Browse --> Overpass: one query (1.5 km)
  Overpass --> Idle: render ≤60
  Overpass --> Idle: error toast (429/5xx)
```

## Notes

- Friend/shared coordinates **never** enter Nominatim/Overpass requests.
- All results carry ODbL attribution; removal is a QA blocker.
