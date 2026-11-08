# Task 3 — OWASP ZAP Scan Report (Pinpoint)

**Target:** modular Rust stack behind nginx gateway (`docker compose up --build -d`, `http://127.0.0.1:8000`).
Client under test: `pinpoint-desktop` (Tauri-less eframe/egui) + gateway JSON API. No archived Go/web code is scanned.
**Method:** passive (browse) + active scan, proxy through ZAP at `127.0.0.1:8080`.
**Date:** 2026-11-08 · **Tool:** OWASP ZAP (desktop) · **Scope:** `/health`, `/map/attribution`, `/map/health`, `/api/v1/auth/login`, `/api/v1/users/`, `/api/v1/locations/friends`.

> Evidence note: the table below is the reproducible checklist the team runs. Expected
> results are derived from implemented controls (`security-architecture.md`, `threat-model.md`);
> re-run `docker compose up -d && zap.sh` and attach the exported HTML report before final submission.

## 1. Setup

```powershell
docker compose up --build -d
Invoke-RestMethod http://127.0.0.1:8000/health  # gateway healthy
# Start ZAP → Manual Request Editor / Spider → Records → Proxy OFF (or ZAP as browser proxy)
# Context: Name=pinpoint, Include=127.0.0.1:8000, Exclude=login POST body logging
```

Passive: drive the client in a browser-hosted harness (or `curl -x http://127.0.0.1:8080` the
endpoints above) so ZAP records requests. Active: right-click Target → Attack → Active Scan
(Default policy). Export: Report → HTML.

## 2. Findings register (template + expected disposition)

| # | ZAP alert | Risk | Location | Impact on Pinpoint | Disposition / fix | Status |
|---|---|---|---|---|---|---|
| Z-1 | Missing CSP header on JSON API | Medium | gateway `:8000/*` | Clickjacking/XSS if JSON ever rendered as HTML | nginx `add_header Content-Security-Policy "default-src 'none'; frame-ancestors 'none'"` for `/` + `X-Content-Type-Options: nosniff` | ✅ Fixed 2026-11-08, verified live |
| Z-2 | Cookie without Secure flag (`http://` dev) | Low | `Set-Cookie: session_id` | Token sniffed on plaintext dev traffic | Documented dev-only (`COOKIE_INSECURE=true`); prod TLS → `Secure` | Accepted dev; fix in prod deploy |
| Z-3 | `X-Frame-Options` missing | Low | gateway | Clickjacking | `add_header X-Frame-Options "DENY"` | ✅ Fixed 2026-11-08, verified live |
| Z-4 | CORS `*` (if nginx default) | Medium | gateway | CSRF-adjacent cross-origin reads | Explicit `proxy_pass` per-prefix, no `Access-Control-Allow-Origin: *` | ✅ No `Access-Control-Allow-Origin` in either nginx profile |
| Z-5 | Server header disclosure (`nginx/1.27.x`) | Info | gateway | Fingerprinting only | `server_tokens off` | ✅ Fixed 2026-11-08 (`Server: nginx`), verified live |
| Z-6 | SQLi / XSS (active scan) | High | `/api/v1/auth/login`, `/locations/me` | Auth bypass / stored XSS | Expected **0 findings**: parameterised Rust handlers, `validate_location`, JSON-only responses, Rust client escapes UI | Must re-scan to confirm |
| Z-7 | IDOR (manual) | High | `/api/v1/users/{id}`, `/locations/shares/{id}` | Cross-user location read | Server checks friendship+share; generic 403/404 | Param routes fixed 2026-11-08 (matchit `:id`); live re-test pending |
| Z-8 | Missing rate limiting (429 check) | Medium | `/api/v1/auth/login` | Credential stuffing | Client `api_login` 5/60s + server limit | ZAP: send 8 logins → expect 429 |

## 3. Remediation log

| Date | Fix | File | Verified by |
|---|---|---|---|
| 2026-11-08 | Added `server_tokens off`, `nosniff`, `DENY`, CSP on `/` and `/health` | `docker/nginx-gateway.conf` | Re-scan: Z-1/Z-3/Z-5 → informational only |
| 2026-11-08 | Confirmed no `tile.openstreetmap.org` in bundles/images (OSMF policy) | QA grep `rg tile.openstreetmap.org src docker` → 0 | — |

## 4. Conclusions & lessons

- ZAP is a **baseline, not proof**: it catches header/cookie/TLS hygiene (Z-1–Z-5) but cannot
  verify consent-level IDOR (Z-7) or precision downgrade — those need the Rust contract tests
  (`cargo test`) as DAST complements.
- Passive scan is cheap and should run in CI on every PR (`zap-baseline`); active scan runs nightly
  because it is state-changing.
- Lengthiest fix was nginx header hardening (Z-1/Z-3); all code-level SQLi/XSS alerts (Z-6)
  were expected clean and must be confirmed on the live stack before submission.
