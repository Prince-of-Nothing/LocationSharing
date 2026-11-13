# 04 — Non-Functional Requirements

| ID | Category | Requirement | Target / measure |
|----|----------|-------------|------------------|
| NFR-01 | Performance (map) | Tile render keeps 60 fps pan on mid-range laptop/phone; tile grid ≤ ~30 imgs | Manual + Lighthouse (no blocker) |
| NFR-02 | Performance (search) | Debounce 350 ms; ≤1 Nominatim req/s; results cached per session | Code review + QA grep |
| NFR-03 | Performance (routing/POI) | Single user-initiated request; 30 s timeout; no polling/looping | Code review |
| NFR-04 | Availability | Demo boots from `docker compose up --build` with zero config | Manual QA step |
| NFR-05 | Security | TLS in transit; HttpOnly cookies; CSRF double-submit; Argon2id; rate limits; generic errors | Security test suite |
| NFR-06 | Privacy | 0 trackers; CSP-enforced origins; no friend data to OSM services; bookmarks on-device only | CSP + QA policy checks |
| NFR-07 | Licensing | ODbL attribution permanent on every map view; notices updated | QA attribution grep |
| NFR-08 | Offline | Web shell + bookmarks offline; live data shows banner; mobile packs offline | Manual QA |
| NFR-09 | Usability / a11y | Keyboard map pan/zoom; ARIA labels; tap targets ≥38 px; color + text freshness (not color-only) | Manual review |
| NFR-10 | Maintainability | Modular vanilla JS (no bundler); Clean-Architecture mobile modules; Rust fmt/clippy clean | CI |
| NFR-11 | Portability | Web: evergreen browsers, no npm; Mobile: API 26+ (minSdk 26), JDK 17 / SDK 35; Rust: Windows/Linux/macOS via cargo | CI matrices |
| NFR-12 | Observability | Request IDs, structured logs (`slog` Go side), offline banner, error toasts | Manual review |
| NFR-13 | Compatibility | Same `/api/v1` for Go + Python backends; no per-client endpoints | API contract CI |
| NFR-14 | Data retention | Friend locations: latest-only, never cached client-side; bookmarks: user-deletable | Audit + QA |
| NFR-15 | Legal | No scraping; respect Nominatim/OSRM/Overpass/tile ToS; recommend self-host for production | Notices + 07 guide |
| NFR-16 | Battery | No background location work without explicit publish; geolocation is one-shot on demand | Permissions audit |
| NFR-17 | Scalability path | Stateless API → Postgres/Redis; tile/routing self-host path | 12-deployment |
| NFR-18 | QA automation | `scripts/qa.sh` exit 0 on clean tree; CI `map-qa` job green | Evidence in 11 |

## Key trade-offs

- **Zero-dependency web** vs richer map libs: we keep vanilla JS (no Leaflet/
  MapLibre GL bundle) for supply-chain reasons; the Slippy engine in
  `js/map.js` is ~300 lines and fully owned.
- **Public OSM services** vs self-hosted: public for demo (zero ops);
  self-host is the documented production path (07).
- **Offline scope**: web is shell-only offline by design (tile caching would
  violate ToS + leak data). Mobile carries the heavy offline story.
