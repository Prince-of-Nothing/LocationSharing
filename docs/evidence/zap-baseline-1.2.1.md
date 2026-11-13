# ZAP baseline scan — Pinpoint 1.2.1

**Date:** 2026-11-13 · **Target:** `http://host.docker.internal:8080/`
(combined `pinpoint-server`, in-memory demo) · **Tool:** OWASP ZAP 2.17.0
(`ghcr.io/zaproxy/zaproxy:stable`) · **Mode:** unauthenticated baseline
(`zap-baseline.py -m 2 -I`).

## Result

| Risk | Count |
|---|---|
| High | **0** |
| Medium | **0** |
| Low | **0** |
| Informational | 4 |

The first baseline run (2026-11-13, before the header fix) reported four Low
warnings:

| Alert | Fix |
|---|---|
| Cross-Origin-Embedder-Policy missing | `Cross-Origin-Embedder-Policy: require-corp` |
| Cross-Origin-Opener-Policy missing | `Cross-Origin-Opener-Policy: same-origin` |
| Cross-Origin-Resource-Policy missing | `Cross-Origin-Resource-Policy: same-origin` |
| Permissions-Policy missing | `Permissions-Policy: geolocation=(self), camera=(), microphone=(), payment=(), usb=()` |

All four are now set by `src/server/web_handlers.rs::secure()` and asserted in
`tests/assets.rs`. Static JS/CSS use `Cache-Control: no-cache` (revalidate);
HTML deliberately keeps `no-store`, which is the remaining "Non-Storable
Content" informational note. "Modern Web Application" is informational and
needs no change.

## Scope and follow-up

This is the **unauthenticated** baseline (login page + static assets). An
authenticated active scan of the map/social/admin routes is still a follow-up;
it needs a seeded session and a ZAP context with the session cookie.

Raw reports: [`zap-baseline.html`](./zap-baseline.html),
[`zap-baseline.json`](./zap-baseline.json),
[`zap-baseline.md`](./zap-baseline.md).

Reproduce:

```bash
docker run --rm -v "$PWD/docs/evidence:/zap/wrk:rw" \
  ghcr.io/zaproxy/zaproxy:stable \
  zap-baseline.py -t http://host.docker.internal:8080/ \
  -r zap-baseline.html -J zap-baseline.json -w zap-baseline.md -m 2 -I
```
