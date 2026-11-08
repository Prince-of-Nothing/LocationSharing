# 10 — Security and Privacy

## Threat model (summary)

| ID | Threat / abuse | Mitigation | Where |
|----|----------------|------------|-------|
| T-01 | IDOR: read another user's location by ID | Server-side ownership/consent checks; generic 404; audit | Go/Python handlers + tests |
| T-02 | Over-sharing / forgotten grants | Directional shares + expiry + pause/revoke; freshness honesty | Consent model |
| T-03 | Session hijack / CSRF | HttpOnly cookies, SameSite, CSRF double-submit, rotation | `security` + middleware |
| T-04 | Tracker / third-party SDK leak | 0-tracker stance; CSP allow-list; no analytics SDKs | nginx CSP, mobile deps |
| T-05 | Friend data to OSM services | OSM queries carry only user input; friend coords never sent | `geocode`/`routing`/`places` + review |
| T-06 | Offline data exposure on shared device | Never cache `/api` or location in SW/localStorage/IndexedDB; bookmarks exclude friend data | `sw.js`, `bookmarks.js` |
| T-07 | OSM ToS violation (tiles/bulk) | No `tile.openstreetmap.org`; throttles; self-host path | QA policy greps |
| T-08 | XSS via server data | `esc()` everywhere; no unescaped `innerHTML` | web `ui.js` discipline |
| T-09 | Token/log leakage | BASIC logging only; never log headers/body (tokens/coords) | mobile + Go `slog` |
| T-10 | Supply chain (npm/CDN) | Zero JS deps; vendored Leaflet pinned (Python client only) | web/README |

## Controls (selected)

- **Auth:** bcrypt/Argon2 passwords; session store; rate limits; forced
  change; lockout; secure cookie flags (`Secure`, `SameSite=Lax`, `HttpOnly`).
- **Transport:** HTTPS-only in prod; cleartext only for loopback dev
  (Rust fail-closed, Android network config).
- **Privacy:** pseudonymous accounts; minimal PII; ODbL attribution (legal);
  no location history server-side (latest-only); bookmarks deletable.
- **OSM privacy envelope:** user-initiated only, no background queries, no
  analytics around OSM calls, referrer suppressed where appropriate.

## Data handling

| Data | Stored | Sent to OSM services? | Retention |
|------|--------|------------------------|-----------|
| Account credentials | hashed (server) / Keystore (mobile tokens) | No | until deletion |
| Friend locations | latest-only (server) | **No** | overwritten on publish |
| Bookmarks | device localStorage/DataStore only | No | until user deletes |
| Search/route/POI queries | memory/session cache | Yes (user input only) | session only |
| Tiles | browser/OS cache (no SW) | n/a | provider-controlled |

## Incident response

See `docs/architecture/incident-response-plan.md` + `docs/SECURITY.md`.
OSM outage playbook: clients degrade to offline shell + bookmarks + cached
packs (mobile); sharing core stays available.

## Assurance evidence

- `scripts/qa.sh`: policy/attribution/CSP greps, syntax, Mermaid, links.
- `ci.yml`: backend/mobile/rust jobs + `map-qa` job.
- Security tests: Go `internal/security` + handler suites (CSRF, rate,
  authz). Mobile: Keystore + network-config review.
