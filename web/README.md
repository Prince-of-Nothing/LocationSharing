# web/ — Rust/WASM client (webapp)

Same `pinpoint_desktop` crate as desktop/mobile. Build the web client with
[Trunk](https://trunk.rs/):

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
trunk serve --release        # http://localhost:8080
# or: trunk build --release && python -m http.server --directory dist
```

## What's here

- `index.html` — static shell (map-first landing, ODbL credit, build links).
- `Trunk.toml` — points trunk at `src/bin/pinpoint-web.rs`.
- `src/bin/pinpoint-web.rs` — WASM entry (`#[wasm_bindgen::start]`) +
  native stub so desktop CI (`cargo build/test`) stays green without the
  wasm toolchain.

## Status (honest)

The desktop GUI (`src/ui`, `map`, `tilepacks`) is `#[cfg(not(wasm32))]`:
it needs `tokio` and the filesystem, which the wasm target doesn't have.
The web client therefore ships today as the static shell above plus the
WASM entry hook. A full egui Web UI reusing the desktop screens requires
the `tokio` → wasm-runtime swap (spawn tasks via `wasm_bindgen_futures`,
bookmarks in `localStorage`, no tilepacks on disk) — tracked as the next
web step in [`docs/client-platforms.md`](../docs/client-platforms.md).

Backend URL: `PINPOINT_BACKEND` env at runtime > per-platform default.
In the browser, loopback `http://localhost:8000` only works when the page
is served from loopback; production deploys must point at the gateway's
HTTPS URL (ADR-0006 — plain HTTP off-loopback is rejected).
