# 14 — Offline Packs & Bookmarks (State + Data)

**Traceability:** FR-11, FR-13 · NFR-08, NFR-14

## Sequence: download offline pack (mobile)

```mermaid
sequenceDiagram
  actor U as User
  participant M as MapScreen
  participant VM as MapViewModel
  participant OM as OfflineMapManager (MapLibre)
  U->>M: Tap "Download current area"
  M->>VM: downloadRegion(centre, 0.1°)
  VM->>OM: downloadRegion(style, bounds, z8-15)
  OM-->>VM: DownloadProgress…
  VM-->>M: progress bar + "Available offline"
  M-->>U: renders offline (no network)
```

## State: bookmark lifecycle (web + mobile)

```mermaid
stateDiagram-v2
  [*] --> None
  None --> Saved: ★ star (user gesture)
  Saved --> Exported: GeoJSON download
  Exported --> Saved
  Saved --> Deleted: Delete / Clear
  Deleted --> [*]
  Saved --> Saved: Open (fly-to)
```

## Storage rules

```mermaid
flowchart LR
  subgraph Device[On-device only]
    LS[localStorage bookmarks]
    DS[DataStore bookmarks]
    PK[Offline packs: user downloads]
  end
  subgraph Never[NEVER stored]
    FL[Friend locations]
    SR[Search/route history]
    API[API responses]
  end
  API -.->|network only, no SW cache| Never
```

## Notes

- Web offline = shell + bookmarks only. Mobile offline = shell of app +
  vector packs + bookmarks. Tile bulk-caching on web is forbidden (ToS).
- Bookmarks are not synced; any future sync needs a threat analysis.
