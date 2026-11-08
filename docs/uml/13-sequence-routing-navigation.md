# 13 — Routing & Navigation (Sequence + State)

**Traceability:** FR-09, FR-12, FR-16 · NFR-03, NFR-06

## Sequence: compute route → hand off

```mermaid
sequenceDiagram
  actor U as User
  participant M as Map client
  participant R as OSRM (routing)
  participant X as Nav app (geo:)
  U->>M: Enter from/to + mode (driving/foot/bike)
  M->>R: GET /route/v1/{profile}/{lon,lat};… (steps=true)
  R-->>M: Route + polyline + steps
  M->>M: Render line + turn-by-turn list
  U->>M: Tap "Navigate (OSM)" / "Navigate (geo:)"
  M->>X: geo:lat,lon?from=…&q=label (or OSM directions URL)
  X-->>U: External nav (works offline if app has packs)
```

## Routing fallback states (mobile)

```mermaid
stateDiagram-v2
  [*] --> OnlineEngine: OSRM available
  OnlineEngine --> Stub: offline / 5xx
  Stub --> OnlineEngine: back online (retry on tap)
  OnlineEngine --> HandOff: user taps navigate
  Stub --> HandOff
  HandOff --> [*]
```

## Class sketch (domain)

```mermaid
classDiagram
  class GeoPoint { +latitude +longitude }
  class TravelMode { DRIVE FOOT BIKE }
  class Route { +points +distanceMeters +durationSeconds +steps +summary +mode }
  class RouteStep { +text +distanceMeters +durationSeconds }
  class RoutingEngine { +name +isOfflineCapable +route(from,to) }
  class OsrmRoutingEngine { +route(from,to,mode) }
  class DefaultRoutingEngine { +route(from,to) }
  RoutingEngine <|-- OsrmRoutingEngine
  RoutingEngine <|-- DefaultRoutingEngine
  Route --> GeoPoint : contains
  Route --> RouteStep : contains
```

## Notes

- OSRM demo is not production; Valhalla/GraphHopper plugs into
  `RoutingEngine` without touching features.
- GPX export (`navigation.js routeToGpx`) targets OsmAnd/Organic Maps.
