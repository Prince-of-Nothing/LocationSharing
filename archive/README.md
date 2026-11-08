# Archive — legacy stacks (NOT built or shipped)

Everything in this directory was moved out of the active tree in the
Rust-multiplatform pivot (**ADR-0007**: `docs/adr/0007-rust-multiplatform.md`).
The primary application codebase that builds, tests or ships is the root Rust
crate (`Cargo.toml` + `src/` + `tests/`).

**Update (ADR-0008):** The Go backend (`backend/`), web client (`web/`), and
Android app (`mobile/`) were restored to the repository root so friends can
use the service end-to-end (Go API on `:8000` + web PWA + Android APK).
Copies under `archive/` are historical and must not be built. The Rust
desktop client remains the primary multiplatform client.

| Entry | What it was |
|-------|-------------|
| `backend/` | Go (`cmd/server` + `internal/`) and Python/FastAPI backends sharing `/api/v1` |
| `web/` | Vanilla-JS web client (`web/public/js/` + nginx) |
| `mobile/` | Kotlin/Jetpack Compose Android app (Gradle, MapLibre) |
| `app/` | `gidireads-rebuild` (unrelated Rust static-site builder) |
| `e2e/` | End-to-end harnesses for the Go/Python stacks |
| `docker/` + `docker-compose.yml` + `render.yaml` + `.dockerignore` | Container deployment for the archived stacks |
| `scripts/` | Deploy + QA scripts for the archived stacks |
| `shots/` | Screenshots / presentation captures |
| `investigation.cany/` | Investigation notes/artifacts |
| `build_errors.txt`, `test_screenshot.html`, `$null`, `rustup-init.exe` | Misc build logs, junk, installer binary |

## Reading the archive

- The live contract now is `src/api.rs` + `tests/` (the Rust client is the
  only consumer). Use archived code only to recover design intent, never as
  a live dependency.
- OSM tile/search/routing/POI compliance for the Rust client lives in
  `docs/sdlc/07-openstreetmap-integration.md`; archived `THIRD_PARTY_NOTICES.md`
  copies are historical.
- Restoring any archived stack means a new ADR + moving it back to the root.
