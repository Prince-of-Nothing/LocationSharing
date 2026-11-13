# Admin console — modular architecture

The admin console is a role-gated tab in the web client. It is deliberately a
**registry of modules**, so new admin domains do not touch the map, social or
auth screens.

## Layers

| Layer | Location | Responsibility |
|---|---|---|
| API | `src/server/admin_handlers.rs`, `src/server/diagnostic_handlers.rs` | `/api/v1/admin/*` and report triage; every handler calls `require_admin` |
| Router | `src/server/mod.rs` (`admin_router`, `diagnostic_router`) | combined server + dedicated binaries |
| Dedicated ports | `src/bin/admin.rs` (8006), `src/bin/diag.rs` (8005) | split deployment; same role gates |
| Shell | `web/app/admin.js` | registry, sidebar, module lifecycle |
| Modules | `web/app/admin/*.js` | one file per domain (`overview`, `users`, `diagnostics`, `audit`) |
| Static assets | `src/server/web_handlers.rs` | `include_str!` + explicit routes |

All endpoints are fail-closed: no session → `401`, non-admin → `403`.
Self-demotion, self-deactivation and self-deletion are refused. Deletion
revokes every session and removes friendships, shares, locations, check-ins
and emergencies before deleting the user row. Role changes, active-state
changes, deletions and report triage append to `pinpoint_admin_audit` (ids and
public handles only — no email or coordinates).

## Add a module

1. **Backend:** add handlers in `admin_handlers.rs` (or a new
   `admin_<domain>.rs`) that call `require_admin`; add routes in
   `admin_router()`.
2. **Frontend:** create `web/app/admin/<domain>.js` exporting a module:
   ```js
   export const diagnosticsModule = {
     id: "diagnostics",
     title: "Diagnostics",
     order: 30,
     async mount(el, ctx) {
       const data = await ctx.api("/api/v1/admin/diagnostics");
       el.innerHTML = "…";
     },
   };
   ```
3. **Register:** import it in `admin.js` and add it to `BUILTIN`.
4. **Serve:** add an `include_str!` constant + handler in `web_handlers.rs`
   and a route in `router()`.
5. **Test:** extend `tests/router_probe.rs` for role denial/acceptance and
   `tests/assets.rs` for the new module asset.

Module context (`ctx`) provides `api`, `setStatus`, `getUser`, `openModule`
and the ordered module list. `mount` may return a `destroy` function for
cleanup.

## Current modules

- **Overview** — users, active users, sessions, friendships, shares, open
  diagnostic reports, audit events.
- **Users** — private listing, filter, role change, active toggle, purge with
  confirmation.
- **Diagnostics** — PII-free reports from `POST /api/v1/diagnostics/report`,
  newest first, with status/triage filters.
- **Audit** — append-only log of role changes, active-state changes, account
  deletions and report triage, with actor handle and timestamp.

## Diagnostics service

`pinpoint-diag` listens on **8005** and mounts only the ingestion router
(`/api/v1/diagnostics/report`). Reports contain a category, message, app
version/platform and — when signed in — the public handle. The service never
reads the IP, user agent or coordinates. `pinpoint-admin` listens on **8006**
and mounts the role-gated admin API, including diagnostics listing/triage and
the audit trail. Both can also be reached through the gateway on
`/api/v1/diagnostics/` and `/api/v1/admin/`; the combined `pinpoint-server`
serves the web console and the same routes on its own origin.

Postgres migration 15 creates `pinpoint_diagnostic_reports`,
`pinpoint_admin_audit` and `users.created_at`; audit rows are append-only
(INSERT/SELECT grants, no UPDATE/DELETE).
