# Vendored Leaflet

Leaflet **1.9.4**, vendored so the web client makes no third-party requests
and the CSP can stay `default-src 'self'`.

- Upstream: https://github.com/Leaflet/Leaflet
- License: BSD-2-Clause — see `LICENSE` in this directory.
- Files: `leaflet.js`, `leaflet.css`, `images/*` (layers/marker sprites).
- Downloaded from `https://unpkg.com/leaflet@1.9.4/dist/` and the upstream
  `v1.9.4` tag. Do not replace with a CDN URL.

`tests/assets.rs` asserts these files are served locally and that the CSS has
no remote host.
