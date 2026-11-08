# AGENTS.md — Pinpoint (Rust multiplatform)

## Repo shape (active — Rust only)

- **Rust multiplatform client** (root crate `pinpoint-desktop` v0.2.0):
  `src/lib.rs` (portable core) + `src/main.rs` (desktop), eframe/egui UI +
  `reqwest`. One codebase for desktop / Android (`cargo-apk`) / iOS
  (`cargo-xcodebuild`) / wasm stub — see `docs/adr/0007-rust-multiplatform.md`.
  Modules: `api` (session cookie + `X-CSRF-Token` double-submit, plain HTTP
  only for loopback, ADR-0006), `app`/`ui` (state/screens), `map` (pins +
  OSM hand-off + ODbL credit), `osm` (Nominatim/OSRM/Overpass in Rust),
  `platform` (all `cfg(target_os)`), `bookmarks` (on-device JSON).
- `tests/contract.rs` pins the wire contract. CI `rust` job runs
  fmt/clippy/build/test/audit.
- Docs in `docs/` (arc42 + Diátaxis + ADRs; SDLC set `docs/sdlc/00–16`;
  UML `docs/uml/01–15`). Contribution rules in `CONTRIBUTING.md`.

## Archive (`archive/` — NOT built)

Everything else was archived in the Rust-multiplatform pivot:
`backend/` (Go + Python), `web/` (vanilla JS), `mobile/` (Kotlin),
`app/` (gidireads Rust static site), `e2e/`, `docker/`, `scripts/`,
`shots/`, plus compose/render/binaries and junk files. See
`archive/README.md`. Do not build, lint or import from `archive/`; consult
it only as historical reference. OSM maps policy lives in
`docs/sdlc/07-openstreetmap-integration.md`; never use
`tile.openstreetmap.org` (OSMF policy).

## Commands (Rust — must pass before pushing)

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo build --locked && cargo test --locked
cargo run -- --backend http://localhost:8000   # desktop dev (loopback only)
```

Android/iOS need their Rust targets + NDK/Xcode (see `README.md`
multiplatform section); they share `src/` via `cfg(target_os)`.

## Gotchas

- Security is thesis-mandated: server-side enforcement only, no secrets or
  real user data anywhere including tests (`CONTRIBUTING.md`).
- Password/session/token wire contract lives in `src/api.rs` + `tests/` —
  change it only with a contract test update and an ADR if semantic.
- Platform code goes in `src/platform.rs`; OSM network calls in `src/osm.rs`;
  nothing platform- or network-specific inside `src/ui.rs`.

## Commits

Every commit is co-authored by the human team members (not bots). Add trailers on each commit:

```
Co-authored-by: Mordret <blacksnake@noreply.codeberg.org>
Co-authored-by: Andrei Cliucovschi <cliucovschiandrei@gmail.com>
Co-authored-by: Pleșu Dinu <dinu.plesu@isa.utm.md>
Co-authored-by: Nedelea Dragoș <63261775+dragos-nedelea@users.noreply.github.com>
Co-authored-by: dana-r19 <200202105+dana-r19@users.noreply.github.com>
Co-authored-by: Tcacenco Nichita <nichita.tcacenco@isa.utm.md>
```

Exclude bots: `qwen.ai[bot]`, copilot, dependabot.

## Docs conventions

- Diagrams are Mermaid text, never binary images; keep the table in
  `docs/uml/README.md` in sync with files that exist.
- ADRs are immutable once accepted, named `NNNN-kebab-title.md`, indexed in
  `docs/adr/README.md`; architecture-significant changes need an ADR in the same PR.
