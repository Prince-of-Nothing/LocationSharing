# web/ — Rust/WASM clients (modular)

Browser clients share the `pinpoint_desktop` crate and its modular backend
abstraction (`crate::backend::{Backend, BackendConfig, DynBackend}`), so
backend architecture changes (endpoints/auth/protocol version) are handled in
adapters/config rather than UI code.

Build with [Trunk](https://trunk.rs/):

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked

# General web client
trunk serve --config web/Trunk.toml --release        # http://localhost:8080

# Admin console (weba)
trunk serve --config web/weba/Trunk.toml --release

# Kiosk/check-in (webk)
trunk serve --config web/webk/Trunk.toml --release
```

## What's here

- `index.html` + `Trunk.toml` — general web shell / `src/bin/pinpoint-web.rs`.
- `weba/` — admin console SPA shell + Trunk config / `src/bin/pinpoint-weba.rs`.
- `webk/` — kiosk/check-in shell + Trunk config / `src/bin/pinpoint-webk.rs`.
- `src/webapp/` — shared portable webapp modules (config, view models, admin,
  kiosk adapter points).

## Modularity / backend changes

- Webapps resolve backend config through `webapp::config::WebAppConfig` and the
  `backend` module. Add a new `BackendKind`/adapter when the backend
  architecture changes; keep UI/view-model code stable.
- `webk` kiosk token/share-link endpoints are isolated behind
  `webapp::kiosk::KioskBackend` so evolving kiosk contracts don't leak into the
  general client.
- `weba` uses the shared `Backend` admin/user operations; server-side
  authorization remains authoritative (UI role checks are fail-closed hints).

## Status (honest)

Desktop GUI modules (`src/ui`, `map`, `tilepacks`) are `#[cfg(not(wasm32))]`.
The wasm bins currently provide modular wiring/stubs plus HTML shells. A full
egui web UI reusing desktop screens requires the `tokio` → wasm-runtime swap
(`wasm_bindgen_futures`/async spawn), browser storage for bookmarks, and no
filesystem tilepacks — tracked in [`docs/client-platforms.md`](../docs/client-platforms.md).

Backend URL: `PINPOINT_BACKEND` / host config > per-platform default. In
production, use HTTPS (ADR-0006); plain HTTP is only valid for loopback
development contexts as enforced by backend adapters.
