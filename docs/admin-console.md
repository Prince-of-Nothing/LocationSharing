# Admin console — modular architecture

The admin console is a role-gated tab in the web client. It is deliberately a
**registry of modules**, so new admin domains do not touch the map, social or
auth screens.

## Layers

| Layer | Location | Responsibility |
|---|---|---|
| API | `src/server/admin_handlers.rs` | `/api/v1/admin/*`, every handler calls `require_admin` |
| Router | `src/server/mod.rs` (`admin_router`) | mounts admin routes; can later run on a dedicated admin binary/port |
| Shell | `web/app/admin.js` | registry, sidebar, module lifecycle |
| Modules | `web/app/admin/*.js` | one file per domain (`overview`, `users`, `placeholder`) |
| Static assets | `src/server/web_handlers.rs` | `include_str!` + explicit routes |

All endpoints are fail-closed: no session → `401`, non-admin → `403`.
Self-demotion, self-deactivation and self-deletion are refused. Deletion
revokes every session and removes friendships, shares, locations, check-ins
and emergencies before deleting the user row.

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

- **Overview** — users, active users, sessions, friendships, shares,
  diagnostics counter.
- **Users** — private listing, filter, role change, active toggle, purge with
  confirmation.
- **Diagnostics** — placeholder until the dedicated ingestion port/table land.
- **Audit** — placeholder until the admin audit table lands.
