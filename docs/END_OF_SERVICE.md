# Pinpoint — End of Service & Data Reuse Guide

This document covers what happens when the Pinpoint service ends, how to
export your data, and how to reuse the Pinpoint client code and design for
your own projects.

---

## 1. What Happens When the Service Ends

If the Pinpoint backend server is shut down:

### What continues to work

| Feature | Offline behavior |
|---------|-----------------|
| Map view | ✅ Works — demo locations and OSM tiles load |
| Search places | ⚠️ Requires internet (Nominatim) |
| Navigate to location | ⚠️ Requires internet (OpenStreetMap.org) |
| Publish location | ❌ Requires server |
| Friend requests | ❌ Requires server |
| Location sharing | ❌ Requires server |
| Admin panel | ❌ Requires server |
| Change password | ❌ Requires server |

The app degrades gracefully: the map opens, demo locations are visible, and
navigation links open in your browser. Only server-dependent features
(publishing, sharing, friends) stop working.

### What data is lost

- **Friendships** — Stored server-side only. Lost when the server ends.
- **Location shares** — Stored server-side only. Lost when the server ends.
- **Published locations** — Stored server-side (latest-only). Lost when the
  server ends.
- **Bookmarks** — Stored on your device only. NOT lost — see "Exporting
  Bookmarks" below.
- **Session tokens** — Stored on your device only. Become invalid when the
  server ends.

Pinpoint follows a **latest-location-only** storage policy: the server stores
only your most recent location publish, not a history. There is no location
history to lose.

---

## 2. Exporting Your Data

### Bookmarks (On-Device Data)

Bookmarks are stored locally on your device, not on the server. They survive
service termination.

**Desktop:** `~/.pinpoint/bookmarks.json`

**Mobile:** App DataStore (cleared on app uninstall)

Bookmarks are stored as JSON with this format:

```json
[
  {
    "id": "bm_1234567890",
    "name": "Home",
    "latitude": 41.8902,
    "longitude": -87.6323,
    "note": "",
    "category": "",
    "created_at": "2026-11-08T12:00:00Z"
  }
]
```

You can copy this file to any device and import it into a future version of
Pinpoint or any other application that supports GeoJSON/bookmark imports.

### Friend Locations (Server-Side)

Friend locations are stored server-side only. Before the service ends:

1. Open the map screen.
2. Note down the locations of friends you want to remember (latitude,
   longitude, name).
3. These are display-only — there is no export endpoint because locations
   are latest-only and consent-based.

### Account Information

Account data (email, friendships, shares) is stored server-side. Contact the
server administrator before shutdown to request:
- A copy of your account data (GDPR data export)
- Account deletion

---

## 3. Reusing the Pinpoint Code

Pinpoint is open source (MIT License). You can reuse the entire codebase or
individual components for your own projects.

### What's Included

| Component | License | Reuse Potential |
|-----------|---------|-----------------|
| Rust client (`src/`) | MIT | Full application, library, or components |
| OSM services (`src/osm.rs`) | MIT | Nominatim/OSRM/Overpass client for any Rust project |
| Bookmarks (`src/bookmarks.rs`) | MIT | On-device JSON storage pattern |
| Platform abstraction (`src/platform.rs`) | MIT | Cross-platform Rust app template |
| API client (`src/api.rs`) | MIT | Session+CSRF client pattern for any API |
| Docker/Podman files | MIT | Container build templates |
| Documentation (`docs/`) | MIT | Documentation templates |

### Reusing the Map-First UX

The Pinpoint UI is built with `eframe`/`egui` (immediate-mode GUI). Key
patterns you can reuse:

1. **Map-first landing** — Show the map immediately, sign-in optional.
2. **Offline-first** — Demo data works without a server.
3. **Progressive disclosure** — Sign-in unlocks features gradually.
4. **OSM integration** — OpenStreetMap directions, search, and POI browse.

### Reusing the OSM Client

The `OsmClient` in `src/osm.rs` is a standalone, dependency-light client
for OpenStreetMap services:

```rust
use pinpoint_desktop::osm::OsmClient;

let client = OsmClient::new().unwrap();
let places = client.search("café Chicago").await.unwrap();
let route = client.route((41.8902, -87.6323), (41.8781, -87.6298),
    pinpoint_desktop::osm::TravelMode::Walking).await?;
```

Dependencies: only `reqwest` and `serde`. No GUI dependencies.

### Reusing the Bookmarks Store

The `BookmarkStore` in `src/bookmarks.rs` provides on-device JSON storage:

```rust
use pinpoint_desktop::bookmarks::BookmarkStore;

let store = BookmarkStore::default_location();
store.add("Home", 41.8902, -87.6323).unwrap();
let bookmarks = store.list().unwrap();
store.remove("bm_1234567890").unwrap();
```

Dependencies: only `serde` and `serde_json`.

---

## 4. Building Your Own Instance

### Backend

Pinpoint's backend was archived in the Rust-multiplatform pivot
(`archive/backend/`). You can:

1. **Revive the Go backend** from `archive/backend/` — it includes the full
   `/api/v1` contract that the Pinpoint client expects.
2. **Build your own backend** using the Pinpoint wire contract documented in
   `src/api.rs` and `tests/contract.rs`.
3. **Use the mock API** (`docker/mock-api/mock_api.py`) as a template for
   testing.

### Frontend (Client)

The Rust client is ready to use as-is:

```bash
git clone https://github.com/your-org/pinpoint.git
cd pinpoint
cargo build --release --locked
cargo run -- --backend https://your-api.example.com
```

### Container Deployment

```bash
docker build -t your-pinpoint:latest -f Dockerfile --target runtime .
docker run -e PINPOINT_BACKEND=https://your-api.example.com your-pinpoint:latest
```

---

## 5. Data Portability

Pinpoint is designed for data portability:

| Data Type | Storage Location | Portable Format |
|-----------|-----------------|-----------------|
| Bookmarks | Device-local JSON | JSON / GeoJSON-compatible |
| Account | Server-side | Request from administrator |
| Friendships | Server-side | Request from administrator |
| Shares | Server-side | Request from administrator |
| Locations | Server-side (latest-only) | Request from administrator |

There is no vendor lock-in. Your bookmarks are yours to keep; everything
else is stored server-side and can be requested before service termination.

---

## 6. Open Source License

Pinpoint is released under the **MIT License**. You are free to:

- Use the code for any purpose (commercial or personal)
- Modify the code
- Distribute modified copies
- Use parts of the code in other projects

The only requirement is to include the original copyright and license notice
in distributions.

---

## 7. Dependencies & Licenses

Pinpoint's dependencies are all open source:

| Crate | License | Purpose |
|-------|---------|---------|
| eframe/egui | MIT | GUI framework |
| reqwest | MIT | HTTP client |
| serde/serde_json | MIT | JSON serialization |
| tokio | MIT | Async runtime |
| which | MIT | Finding system executables |
| webbrowser | MIT | Opening URLs in browser |

All dependencies are compatible with the MIT license and can be reused in
commercial projects.

---

## 8. Contact

For questions about end-of-service data export, code reuse, or licensing:

- Check `docs/USER_GUIDE.md` for feature usage.
- Check `docs/TROUBLESHOOTING.md` for common issues.
- Review the source code in `src/` — it is self-documenting.
- The MIT License in `LICENSE` governs all reuse.

---
