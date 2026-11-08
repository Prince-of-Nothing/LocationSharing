# Pinpoint — Docker, Podman, and Full Lifecycle Documentation

This document is the **complete operations and development reference** for running
Pinpoint in containers (Docker Compose and Podman) and natively. It is written to
be long and exhaustive: every build flag, environment variable, network setting,
security control, and failure mode is documented here.

It is a living document. When a build argument, port, or environment variable
changes, this file must be updated in the same PR.

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Repository Layout](#2-repository-layout)
3. [Architecture Decision Summary](#3-architecture-decision-summary)
4. [Build Systems Comparison](#4-build-systems-comparison)
5. [Docker — Container Images and Compose](#5-docker-container-images-and-compose)
   - 5.1 Dockerfile stages
   - 5.2 docker-compose.yml services
   - 5.3 Building the images
   - 5.4 Running the client (GUI) with X11 forwarding
   - 5.5 Running the CI gate (tester service)
   - 5.6 Mock API service
   - 5.7 Environment variables reference
   - 5.8 Volumes and networking
   - 5.9 Docker security posture
6. [Podman — Containerfile and Rootless Execution](#6-podman-containerfile-and-rootless-execution)
   - 6.1 Containerfile.podman stages
   - 6.2 Building and running with Podman
   - 6.3 Rootless vs rootful mode
   - 6.4 Podman networking (slirp4netns / pnetns)
   - 6.5 Podman on Windows (WSL2 backend)
   - 6.6 Podman security posture
7. [Native (Non-Container) Development](#7-native-non-container-development)
   - 7.1 Prerequisites
   - 7.2 Windows (MSVC linker) setup
   - 7.3 Linux/macOS setup
   - 7.4 Cargo commands (fmt, clippy, build, test)
   - 7.5 Running the desktop client
   - 7.6 Android and iOS targets
8. [Rust Multiplatform Client Internals](#8-rust-multiplatform-client-internals)
   - 8.1 Module map
   - 8.2 API client (session + CSRF)
   - 8.3 OSM services (Nominatim / OSRM / Overpass)
   - 8.4 Bookmarks (on-device JSON)
   - 8.5 Platform abstraction
   - 8.6 GUI (eframe/egui)
9. [OpenStreetMap Integration](#9-openstreetmap-integration)
   - 9.1 Data license (ODbL) and attribution
   - 9.2 Tiles (never tile.openstreetmap.org)
   - 9.3 Nominatim search / reverse
   - 9.4 OSRM routing (demo vs self-hosted)
   - 9.5 Overpass POI browse
   - 9.6 Hand-off links (geo:, OSM, Organic Maps)
   - 9.7 Privacy envelope
10. [Security Model](#10-security-model)
    - 10.1 Transport policy (ADR-0006)
    - 10.2 Session cookie + CSRF double-submit
    - 10.3 Fail-closed session handling
    - 10.4 Container security (non-root, capabilities, seccomp)
    - 10.5 Secrets and credentials
11. [Automatic QA and CI](#11-automatic-qa-and-ci)
    - 11.1 CI jobs
    - 11.2 Local QA script
    - 11.3 Contract tests
    - 11.4 Exit criteria
12. [Deployment and Operations](#12-deployment-and-operations)
    - 12.1 Local development
    - 12.2 Production deployment
    - 12.3 Health checks
    - 12.4 Rollback and roll-forward
    - 12.5 Monitoring and logging
13. [Troubleshooting](#13-troubleshooting)
    - 13.1 Docker engine 500 errors on Windows
    - 13.2 Missing MSVC linker (link.exe)
    - 13.3 X11 forwarding failures
    - 13.4 Mock API connection refused
    - 13.5 Podman rootless networking
14. [Glossary](#14-glossary)
15. [Appendix: Command Cheat Sheet](#15-appendix-command-cheat-sheet)

---

## 1. Project Overview

Pinpoint is a **privacy-first, consent-based location-sharing platform** for
trusted friends. The core invariant is: **friendship does not grant location
access**. Every share of your location is explicit, granular (precision level),
time-boxed (expiry), and revocable, and every consent decision is enforced
server-side.

This repository's **active codebase is a single Rust crate** (`pinpoint-desktop`,
v0.2.0) that serves as a multiplatform client:

- **Desktop** (Windows / Linux / macOS) via `eframe`/`egui` + `run_native`
- **Android** via `cargo-apk` + `eframe`/`winit` (`aarch64-linux-android`)
- **iOS** via `cargo-xcodebuild` (`aarch64-apple-ios`)
- **WebAssembly stub** — the portable core compiles to `wasm32`; the GUI modules
  are compiled out (`#[cfg(not(target_arch = "wasm32))]`)

The previous polyglot stacks (Go/Python backends, vanilla-JS web client,
Kotlin/Jetpack Compose Android app) were **archived** under `archive/` in the
Rust-multiplatform pivot (ADR-0007). They are historical reference only and are
not built, tested, or shipped.

The client talks to a Pinpoint API over `/api/v1` (session-cookie auth with
`X-CSRF-Token` double-submit, plain HTTP allowed only for loopback dev URLs,
fail closed). It also talks to OpenStreetMap-derived public services
(Nominatim search/reverse, OSRM routing, Overpass POI browse) for map
functionality. All map data is © OpenStreetMap contributors under ODbL v1.0;
attribution is permanent and non-removable.

---

## 2. Repository Layout

```
C:\Users\Dinusauuuur\Desktop\git\Pinpoint\          (repository root)
├── Cargo.toml                                       # pinpoint-desktop v0.2.0
├── Cargo.lock                                       # locked dependency tree
├── Dockerfile                                       # multi-stage: builder / runtime / tester
├── Containerfile.podman                             # podman-equivalent build
├── docker-compose.yml                               # pinpoint / tester / mock-api
├── .cargo/config.toml                               # MSVC linker + LIBPATH (Windows)
├── src/
│   ├── lib.rs                                       # public module declarations
│   ├── main.rs                                      # desktop entry (eframe::run_native)
│   ├── api.rs                                       # /api/v1 client, session + CSRF, URL validation
│   ├── app.rs                                       # state machine + screens
│   ├── ui.rs                                        # egui widgets + rendering
│   ├── map.rs                                       # friend pins + OSM hand-off + ODbL credit
│   ├── models.rs                                    # domain models (User, Friendship, Share, Location)
│   ├── osm.rs                                       # Nominatim / OSRM / Overpass + polyline decode
│   ├── platform.rs                                  # OS detection, backend defaults, app data dir
│   └── bookmarks.rs                                 # on-device JSON bookmarks
├── tests/
│   ├── contract.rs                                  # 10 wire-contract tests (mock server)
│   └── common/
│       └── mod.rs                                   # dependency-free recorded HTTP server
├── archive/                                         # LEGACY — NOT built (see archive/README.md)
│   ├── backend/                                     # Go + Python API (archived)
│   ├── web/                                         # vanilla-JS web client (archived)
│   ├── mobile/                                      # Kotlin Android app (archived)
│   ├── app/                                         # gidireads Rust static site (archived)
│   ├── docker/, scripts/, shots/, e2e/, ...
├── docker/
│   └── mock-api/
│       ├── mock_api.py                              # Flask stand-in for /api/v1
│       └── README.md
├── docs/
│   ├── sdlc/00-16.md                                # full SDLC documentation set
│   ├── uml/01-15.md                                 # Mermaid diagrams
│   └── adr/0001-0007.md                             # architecture decision records
├── .github/workflows/
│   ├── ci.yml                                       # rust + release jobs
│   └── map-qa.yml                                   # OSM compliance job
├── README.md                                        # project + multiplatform docs
├── AGENTS.md                                        # contributor instructions
├── CHANGELOG.md                                     # release notes
├── LICENSE                                          # MIT
└── CONTRIBUTING.md                                  # contribution rules
```

---

## 3. Architecture Decision Summary

| ADR | Title | Status | Relevance to containers |
|-----|-------|--------|-------------------------|
| 0001 | Record Architecture Decisions | Accepted | This document follows ADR conventions |
| 0002 | Adopt Pinpoint Product Direction | Accepted | Privacy-first, consent-based |
| 0003 | Password Auth with Session-Bound JWTs | Accepted | Session cookie + CSRF in `src/api.rs` |
| 0004 | Web Client for MVP; Flutter Deferred | Accepted | Superseded by 0007 |
| 0005 | Store Only Latest Location | Accepted | No location history in client |
| 0006 | Rust Desktop Client Transport Policy | Accepted | Plain HTTP only for loopback |
| 0007 | **Rust Multiplatform as Sole Codebase** | **Accepted** | Single crate builds desktop/mobile |

The container images exist to (a) build and test the Rust crate in a clean,
reproducible environment, and (b) optionally run an offline mock API for client
development when the real Go/Python backend is unavailable.

---

## 4. Build Systems Comparison

| System | File | Purpose | Best for |
|--------|------|---------|----------|
| **Cargo** | `Cargo.toml`, `Cargo.lock` | Native Rust build | Daily development, CI |
| **Docker** | `Dockerfile`, `docker-compose.yml` | OCI images + multi-service | Cross-platform builds, CI, mock API |
| **Podman** | `Containerfile.podman` | OCI images (daemonless, rootless) | Rootless hosts, OpenShift, Windows WSL2 |

Docker and Podman produce **OCI-compatible images** from nearly identical
instructions. The only differences are the daemon model (Docker daemon vs
Podman daemonless) and the `host.docker.internal` vs `host.containers.internal`
alias for reaching the host from a container.

---

## 5. Docker — Container Images and Compose

### 5.1 Dockerfile stages

`Dockerfile` defines three stages, targeted by `--target`:

#### `builder` (default)

- Base: `rust:1.99.1-bookworm-slim`
- Installs system libraries required by `eframe`/`egui`/`winit` at compile
  time: `build-essential`, `pkg-config`, `libssl-dev`, `libxcb-*`,
  `libxkbcommon-dev`, `libgl1-mesa-dev`, `libfontconfig1-dev`,
  `libfreetype6-dev`, etc.
- Copies `Cargo.toml`/`Cargo.lock` first and builds a dummy binary to cache
  the dependency graph (layer caching).
- Copies real `src/` and `tests/`, runs `cargo build --locked --release`.

#### `runtime`

- Base: `debian:bookworm-slim`
- Minimal runtime libraries: `ca-certificates`, `libssl3`, `libxcb1`,
  `libxkbcommon0`, `libxcb-render0`, `libxcb-shape0`, `libxcb-xfixes0`,
  `libgl1`, `libfontconfig1`, `libfreetype6`.
- Creates non-root user `pinpoint` (uid/gid 1000 by default in compose).
- Copies only the release binary from `builder`.
- Entry point: `/app/pinpoint-desktop --backend http://host.docker.internal:8000`.

#### `tester`

- Reuses `builder` plus the full repository.
- Default command runs the complete CI gate:
  `cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo build --locked && cargo test --locked`.
- Exits with the cargo exit code (useful for `docker compose up tester`).

### 5.2 docker-compose.yml services

| Service | Image | Role |
|---------|-------|------|
| `pinpoint` | `pinpoint-desktop:latest` (runtime stage) | Desktop GUI client |
| `tester` | `pinpoint-desktop:tester` (tester stage) | CI gate (fmt/clippy/build/test) |
| `mock-api` | `python:3.11-slim` + Flask | Stand-in `/api/v1` for offline dev |

Network: `pinpoint-net` (bridge). All services attach to it, so `mock-api` is
reachable from `pinpoint` at `http://mock-api:8000`.

### 5.3 Building the images

From the repository root:

```powershell
# Build all images (runtime + tester)
docker compose build

# Build only the client runtime image
docker compose build pinpoint

# Build only the tester image
docker compose build tester

# Build with no cache (e.g. after dependency changes)
docker compose build --no-cache
```

Build output is cached by Docker layer caching; the `Cargo.lock` copy step
ensures dependency compilation is reused unless dependencies change.

### 5.4 Running the client (GUI) with X11 forwarding

`pinpoint-desktop` is an `eframe`/`egui` GUI application. It needs a display
server. On a **Linux host**, forward the X11 socket:

```powershell
docker compose run --rm `
  -e DISPLAY=$DISPLAY `
  -v /tmp/.X11-unix:/tmp/.X11-unix:rw `
  -v $XAUTHORITY:$XAUTHORITY:ro `
  pinpoint
```

On **Windows/macOS Docker Desktop**, native X11 forwarding is not supported
out of the box. Options:

1. **Run natively** on Windows (see §7) — recommended for GUI development.
2. **Run the tester service** for CI validation (headless, no GUI needed).
3. **Use VNC/noVNC** inside the container (not configured by default; would
   require adding a VNC server to the runtime stage).

If you only need to validate that the client builds and tests pass, use the
`tester` service instead of the GUI `pinpoint` service.

### 5.5 Running the CI gate (tester service)

```powershell
# One-off run of the full gate (exits with cargo status)
docker compose run --rm tester

# Or bring up and watch logs
docker compose up tester
```

The tester runs, in order:

1. `cargo fmt --all -- --check` — formatting must be clean
2. `cargo clippy --all-targets -- -D warnings` — lints, warnings are errors
3. `cargo build --locked` — locked dependency tree, offline-friendly
4. `cargo test --locked` — unit + contract tests

If any step fails, the container exits non-zero and `docker compose` reports
failure.

### 5.6 Mock API service

The `mock-api` service is a minimal Flask application (`docker/mock-api/
mock_api.py`) that implements the subset of `/api/v1` the Rust client needs
for offline development:

- `POST /api/v1/auth/login|register` — returns session cookie + CSRF token
- `POST /api/v1/auth/logout|session` — session management
- `GET|POST /api/v1/friendships/` — CSRF enforced on POST
- `GET|POST /api/v1/locations/shares` — CSRF enforced on POST
- `POST /api/v1/locations/me` — publish location (CSRF enforced)
- `GET /api/v1/locations/friends` — friend locations (empty by default)
- `GET /api/v1/users/` — user list (empty by default)
- `GET /health` — health check

Usage:

```powershell
# Start the mock API in the background
docker compose up -d mock-api

# Run the client pointing at the mock API
docker compose run --rm pinpoint --backend http://mock-api:8000

# Stop when done
docker compose down
```

The mock API is **not** a production backend. It has no persistence, no
authentication beyond echoing cookies, and no consent logic. It exists solely
so the Rust client can be exercised without a running Go/Python server.

### 5.7 Environment variables reference

| Variable | Default | Service | Meaning |
|----------|---------|---------|---------|
| `PINPOINT_BACKEND` | `http://host.docker.internal:8000` | pinpoint | Backend API URL passed to `--backend` |
| `PINPOINT_DATA_DIR` | `/home/pinpoint/.pinpoint` | pinpoint | Directory for on-device bookmarks (JSON) |
| `CARGO_NET_OFFLINE` | `false` | tester | Whether Cargo avoids network access |
| `DISPLAY` | (unset) | pinpoint | X11 display for GUI forwarding |
| `XAUTHORITY` | (unset) | pinpoint | X11 authority file for GUI forwarding |

Override at runtime:

```powershell
docker compose run --rm -e PINPOINT_BACKEND=https://api.example.com pinpoint
```

Or edit `docker-compose.yml` and redeploy.

### 5.8 Volumes and networking

| Volume | Mount point | Purpose |
|--------|-------------|---------|
| `pinpoint-data` | (declared, not mounted by default) | Persistent bookmark data |
| `/tmp/.X11-unix` | `/tmp/.X11-unix` (optional) | X11 socket for GUI forwarding |
| `$XAUTHORITY` | `$XAUTHORITY` (optional) | X11 authentication |
| `~/.pinpoint` | `/home/pinpoint/.pinpoint` (optional) | Host-side bookmark persistence |

To persist bookmarks across container recreations:

```yaml
# In docker-compose.yml pinpoint service, add:
volumes:
  - pinpoint-data:/home/pinpoint/.pinpoint
```

### 5.9 Docker security posture

- **Non-root user**: The runtime image runs as `pinpoint` (non-root).
- **No added capabilities**: The container runs with the default Docker
  capability set (no `NET_ADMIN`, `SYS_ADMIN`, etc.).
- **Read-only root filesystem**: Not enabled by default (the binary needs to
  write bookmarks to `PINPOINT_DATA_DIR`), but the writable path is limited
  to the data directory.
- **No secrets in image**: No API keys, passwords, or tokens are baked into
  the image. The mock API uses hardcoded mock tokens only.
- **Health check**: The `mock-api` service defines a health check hitting
  `GET /health`. The `pinpoint` and `tester` services do not define health
  checks (they are short-lived or GUI processes).
- **Image scanning**: The CI `rust` job runs `cargo-audit` (via
  `rustsec/audit-check`) for dependency vulnerabilities. Container image
  scanning (e.g. Trivy) is recommended before production deployment.

---

## 6. Podman — Containerfile and Rootless Execution

### 6.1 Containerfile.podman stages

`Containerfile.podman` mirrors `Dockerfile` almost exactly, with two
intentional differences:

- Base images use `docker.io/rust:...` and `docker.io/debian:...` (explicit
  registry, which Podman resolves without Docker Hub rate-limit quirks).
- The runtime stage uses `host.containers.internal` instead of
  `host.docker.internal` as the default backend alias (Podman's host alias
  on rootless networks).

Stages: `builder`, `runtime`, `tester` — identical semantics to Docker.

### 6.2 Building and running with Podman

Prerequisites: Podman installed and (for rootless) the `subuid`/`subgid`
ranges configured.

```bash
# Build the runtime image
podman build -t pinpoint-desktop:latest -f Containerfile.podman --target runtime .

# Build the tester image
podman build -t pinpoint-desktop:tester -f Containerfile.podman --target tester .

# Run the tester (CI gate)
podman run --rm pinpoint-desktop:tester

# Run the client (headless; GUI needs X11 or VNC)
podman run --rm -e PINPOINT_BACKEND=http://host.containers.internal:8000 \
  pinpoint-desktop:latest
```

Podman is daemonless: `podman build` and `podman run` spawn short-lived
processes directly, which works well on rootless hosts without a background
daemon.

### 6.3 Rootless vs rootful mode

| Mode | Command | Privileges | Notes |
|------|---------|------------|-------|
| Rootless | `podman` (default on desktop) | User namespaces, `slirp4netns` | Safer; no root on host. May need `sysctl` tweaks for large numbers of user namespaces. |
| Rootful | `sudo podman` | Full root inside container | Required if rootless user namespace limits are hit. Not recommended for multi-user hosts. |

Pinpoint's Containerfile does **not** require `--privilege`, `--cap-add`,
or Docker socket access. It is fully compatible with rootless execution.

### 6.4 Podman networking

Rootless Podman uses `slirp4netns` (or `pnetns` on newer versions) to give
containers a private network namespace. The host is reachable via
`host.containers.internal`. Port publishing (`-p 8000:8000`) works via
slirp4netns port forwarding.

If you need the `mock-api` container reachable from the `pinpoint` container
under Podman, use a podman network:

```bash
podman network create pinpoint-net
podman run -d --name mock-api --network pinpoint-net ...
podman run --rm --network pinpoint-net -e PINPOINT_BACKEND=http://mock-api:8000 pinpoint-desktop:latest
```

### 6.5 Podman on Windows (WSL2 backend)

Podman Desktop or the standalone Podman installer on Windows typically runs
a WSL2-based Linux VM. Inside that VM:

```bash
podman build -t pinpoint-desktop:latest -f Containerfile.podman .
podman run --rm pinpoint-desktop:tester
```

GUI forwarding from the WSL2 VM to Windows is not supported by default;
use the `tester` service for CI validation, or run the Rust client natively
on Windows (see §7).

### 6.6 Podman security posture

- **Daemonless**: No long-running daemon to attack; each command is its own
  process.
- **Rootless by default**: Containers run as an unprivileged user on the
  host (mapped to root inside the container via user namespaces).
- **No new privileges**: Consider adding `--security-opt no-new-privileges`
  for extra hardening.
- **Seccomp/AppArmor**: Podman applies the default profile unless overridden.

---

## 7. Native (Non-Container) Development

This is the **recommended daily workflow** on Windows, since the Rust client
is a GUI application and Docker Desktop on Windows does not forward X11
automatically.

### 7.1 Prerequisites

- **Rust stable 1.99.1** (MSVC toolchain): `rustc 1.99.1`, `cargo 1.99.1`
- **Visual Studio Build Tools** with "Desktop development with C++"
  workload — provides `link.exe` and Windows SDK `.lib` files
  (`kernel32.lib`, `userenv.lib`, `ws2_32.lib`, `dbghelp.lib`, etc.)
- **Windows SDK** (installed with Build Tools)
- **Node.js** — only if you later reintroduce the archived web client; not
  needed for the Rust client
- **Android SDK 35 + JDK 17** — only if building the Android target
  (`cargo apk build`)
- **Xcode** — only if building the iOS target (`cargo xcodebuild`)

### 7.2 Windows (MSVC linker) setup

On a stock Windows Rust install, `cargo build` fails with:

```
error: linker 'link.exe' not found
```

Fix by installing Visual Studio Build Tools 2022 (Desktop development with
C++). Once installed, `link.exe` lives at:

```
C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\<version>\bin\Hostx64\x64\link.exe
```

and Windows SDK `.lib` files at:

```
C:\Program Files (x86)\Windows Kits\10\lib\<version>\ucrt\x64
C:\Program Files (x86)\Windows Kits\10\lib\<version>\um\x64
```

To let Rust find them automatically, create `.cargo/config.toml` at the
repository root (already present in this repo):

```toml
[target.x86_64-pc-windows-msvc]
linker = "C:\\...\\link.exe"
rustflags = [
  "-Clink-arg=/LIBPATH:C:\\...\\lib\\x64",
  ...
]
```

The exact paths depend on your installed VS Build Tools version.

### 7.3 Linux/macOS setup

On Linux:

```bash
sudo apt-get install build-essential pkg-config libssl-dev \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev libxkbcommon-x11-dev libgl1-mesa-dev
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo build --locked && cargo test --locked
```

On macOS:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo build --locked && cargo test --locked
```

### 7.4 Cargo commands

| Command | Purpose |
|---------|---------|
| `cargo fmt --all -- --check` | Formatting check (must be clean) |
| `cargo fmt --all` | Apply formatting |
| `cargo clippy --all-targets -- -D warnings` | Lints; warnings are errors |
| `cargo build --locked` | Compile with locked dependency tree |
| `cargo test --locked` | Run unit + contract tests |
| `cargo run -- --backend http://localhost:8000` | Run desktop client against local API |
| `cargo run --release -- --backend https://api.example.com` | Release build against production API |

### 7.5 Running the desktop client

```powershell
cargo run -- --backend http://localhost:8000
```

The client opens an egui window. Log in with a seeded account (if the real
backend is running with demo data), or point `--backend` at the mock API
container for offline development.

### 7.6 Android and iOS targets

Android requires `cargo-apk`, the NDK, and the `aarch64-linux-android`
Rust target:

```bash
rustup target add aarch64-linux-android
cargo apk build --locked
```

iOS requires Xcode and the `aarch64-apple-ios` target:

```bash
rustup target add aarch64-apple-ios
cargo xcodebuild --locked
```

Both targets share `src/` via `cfg(target_os)` in `src/platform.rs`.

---

## 8. Rust Multiplatform Client Internals

### 8.1 Module map

| Module | File | Contents | Platform gates |
|--------|------|----------|----------------|
| `api` | `src/api.rs` | `/api/v1` client: session cookie + `X-CSRF-Token` double-submit, URL validation (ADR-0006), error envelope parsing | All targets |
| `app` | `src/app.rs` | State machine (`CurrentScreen`), refresh logic, CRUD operations | All targets |
| `ui` | `src/ui.rs` | egui widgets, screens, rendering | `not(wasm32)` |
| `map` | `src/map.rs` | Friend pins, OSM hand-off links, ODbL credit | `not(wasm32)` |
| `models` | `src/models.rs` | `User`, `Friendship`, `LocationShare`, `FriendLocation`, `AppData` | All targets |
| `osm` | `src/osm.rs` | Nominatim/OSRM/Overpass clients, polyline decode, POI fragments | All targets |
| `platform` | `src/platform.rs` | OS detection, backend URL defaults, app data dir | All targets |
| `bookmarks` | `src/bookmarks.rs` | On-device JSON bookmarks | All targets |

### 8.2 API client (`src/api.rs`)

- `validate_backend_url()` — accepts `https://...` anywhere, `http://...`
  only for loopback hosts (`localhost`, `127.0.0.1`, `::1`). Fail closed.
- `Session` — stores `Set-Cookie` pairs and the CSRF token (from body or
  `X-CSRF-Token` response header).
- `request()` — attaches `Cookie` on every authenticated request; adds
  `X-CSRF-Token` on `POST`/`PUT`/`PATCH`/`DELETE`.
- `capture_session()` — fail-closed: a `200` without `Set-Cookie` is an
  error (prevents silently-unauthenticated state).
- Contract tests in `tests/contract.rs` pin this behavior against a mock
  HTTP server.

### 8.3 OSM services (`src/osm.rs`)

Zero new dependencies beyond `reqwest` + `serde`:

- `OsmClient::search()` — Nominatim `/search` (debounced ≥3 chars by UI,
  1 req/s throttle).
- `OsmClient::reverse()` — Nominatim `/reverse` for tap-to-geocode.
- `OsmClient::route()` — OSRM `/route/v1/{profile}/{lon,lat};{lon,lat}`
  for driving/foot/bike; decodes Google polyline geometry into steps.
- `OsmClient::pois()` — Overpass `/api/interpreter` with QL fragments for
  9 categories (food, fuel, pharmacy, hospital, hotel, tourism, water,
  parking, park).

Friend/shared coordinates are **never** sent to Nominatim/OSRM/Overpass.

### 8.4 Bookmarks (`src/bookmarks.rs`)

- `BookmarkStore` persists `Bookmark` records as JSON to
  `<app_data_dir>/bookmarks.json` (desktop: `~/.pinpoint/bookmarks.json`).
- `None` data dir (wasm/iOS sandbox) → in-memory only, no persistence.
- Operations: `list`, `add`, `remove`. No sync, no server, no friend data.

### 8.5 Platform abstraction (`src/platform.rs`)

- `Platform::current()` — compile-time OS detection via `std::env::consts::OS`
  plus `cfg(target_os)` for Android/iOS/wasm.
- `default_backend_url()` — `http://10.0.2.2:8000` on Android emulator,
  `http://localhost:8000` elsewhere.
- `app_data_dir()` — `~/.pinpoint` on desktop; `None` on sandboxed targets.
- `build_identity()` — human-readable version/OS/arch string.

### 8.6 GUI (`src/ui.rs`, `src/map.rs`)

- `eframe`/`egui` immediate-mode GUI.
- Screens: Login, Register, Map, Friends, Sharing, Admin, ChangePassword.
- Map renders friend pins at server-authorized precision, with freshness
  coloring (ok/warn/stale) and ODbL attribution text.
- OSM hand-off buttons: OpenStreetMap, OSM Directions, Organic Maps
  (no Google-Earth/Magic-Earth proprietary deep links).

---

## 9. OpenStreetMap Integration

### 9.1 Data license (ODbL) and attribution

All map data is © OpenStreetMap contributors under the **Open Database
License v1.0 (ODbL)**. Obligations fulfilled by Pinpoint:

- **Attribution**: permanent, visible on every map view:
  - Web (archived): `#map-attribution` from `js/config.js`
  - Rust desktop (`src/map.rs`): "Map data © OpenStreetMap contributors (ODbL)"
  - Rust mobile (archived): MapLibre logo/attribution + text in `MapScreen.kt`
- **Share-alike**: applies only if Pinpoint redistributes the database
  (e.g. bundles tiles). End-user tile downloads are governed by provider
  terms.
- **Reference**: https://wiki.openstreetmap.org/wiki/Main_Page

### 9.2 Tiles (never tile.openstreetmap.org)

Pinpoint never defaults to `tile.openstreetmap.org`. The OSMF Tile Usage
Policy prohibits app/bulk use of that infrastructure. Compliant sources:

- Web (archived): CARTO raster (dark/light/voyager) + Wikimedia OSM Standard
- Rust mobile (archived): OpenFreeMap vector styles (liberty/bright/positron)
- Rust desktop: OSM hand-off links (no in-app tile rendering in v0.2.0)

Production deployments should self-host Martin/TileServer GL over Geofabrik
extracts.

### 9.3 Nominatim search / reverse

- Endpoint: `https://nominatim.openstreetmap.org/search|reverse|lookup`
- Policy: max 1 req/s per user, valid referer, cache aggressively, no bulk.
- Client: debounced 350 ms, ≥3 chars, viewbox bias, 8 results max,
  per-session cache.
- Attribution: "© OpenStreetMap contributors (Nominatim / OSMF)".

### 9.4 OSRM routing (demo vs self-hosted)

- Demo: `https://router.project-osrm.org/route/v1/{driving,foot,bike}/...`
- Policy: demo server only, user-initiated single routes, no history.
- Production: self-host OSRM (or Valhalla/GraphHopper) over OSM extracts.

### 9.5 Overpass POI browse

- Endpoint: `https://overpass-api.de/api/interpreter`
- Policy: one QL query per tap, 1.5 km radius (cap 5 km), 60 results max,
  25 s timeout. No bulk.

### 9.6 Hand-off links

- `geo:lat,lon?q=label` — Android/iOS, handled by OsmAnd/Organic Maps/Magic Earth
- `https://www.openstreetmap.org/directions?to=...` — OSM directions
- `https://www.openstreetmap.org/#map=z/lat/lon` — OSM place view
- `https://organicmaps.app/?ll=lat,lon&z=14` — Organic Maps web

### 9.7 Privacy envelope

OSM services receive **only** user-typed search text or user-chosen
coordinates. Friend/shared locations never reach Nominatim/OSRM/Overpass.
Results live in memory/session cache only.

---

## 10. Security Model

### 10.1 Transport policy (ADR-0006)

Plain HTTP is allowed **only** for loopback backend URLs
(`localhost`, `127.0.0.1`, `::1`, and Android emulator `10.0.2.2`).
Everything else requires HTTPS. `validate_backend_url()` enforces this at
client startup; non-conforming URLs cause a hard failure before any request.

### 10.2 Session cookie + CSRF double-submit

- Session cookie: `Set-Cookie` from login/register, `HttpOnly`,
  `SameSite=Lax`, attached to every authenticated request.
- CSRF token: returned in login body or `X-CSRF-Token` response header,
  sent as `X-CSRF-Token` header on unsafe methods.
- GETs (safe methods) never send the CSRF header.

### 10.3 Fail-closed session handling

- Rejected login → no session state; subsequent requests unauthenticated.
- `200` without `Set-Cookie` → login fails ("no session cookie").
- Failed logout → session retained; client never claims success.
- Confirmed logout → session cleared; no cookie on subsequent requests.

### 10.4 Container security

- Non-root user (`pinpoint`) in runtime image.
- No added Linux capabilities.
- No secrets baked into images.
- Rootless Podman compatible.
- `cargo-audit` in CI for dependency vulnerabilities.

### 10.5 Secrets and credentials

No secrets are stored in the repository. The mock API uses hardcoded mock
tokens (`mock-session-123`, `mock-csrf-token`) clearly marked as non-production.
Real deployments inject backend URLs via `PINPOINT_BACKEND` or `--backend`
at runtime, never hardcoded.

---

## 11. Automatic QA and CI

### 11.1 CI jobs (`.github/workflows/ci.yml`)

| Job | Runs on | Steps |
|-----|---------|-------|
| `rust` | ubuntu-latest | fmt check, clippy `-D warnings`, locked build, locked test, cargo-audit |
| `release` | windows-latest | `cargo build --release --locked`, zip artifact, upload |

The `rust` job is the sole application gate (ADR-0007 removed Go/Python/web/
mobile jobs). `map-qa.yml` is a separate fast OSM-compliance job.

### 11.2 Local QA script

The archived `scripts/qa.sh`/`qa.ps1` covered tile policy, attribution, CSP,
and SW SHELL sync for the web client. Under the Rust-only codebase, the
equivalent gate is `cargo clippy --all-targets -- -D warnings` +
`cargo test --locked`, run in CI and locally before pushing.

### 11.3 Contract tests

`tests/contract.rs` (10 tests) exercises `src/api.rs` against a dependency-
free mock HTTP server (`tests/common/mod.rs`). Coverage:

- Login parses user + stores session material
- Unsafe requests send CSRF double-submit
- CSRF falls back to response header
- Rejected login leaves client unauthenticated (fail closed)
- Login without session cookie fails closed
- Logout failure keeps session (fail closed)
- Confirmed logout clears session
- Share creation sends numeric `friendship_id`
- Error envelope surfaced readably
- Publish location sends float accuracy

### 11.4 Exit criteria

A PR is mergeable only when:

- `cargo fmt --all -- --check` exits 0
- `cargo clippy --all-targets -- -D warnings` exits 0
- `cargo build --locked` exits 0
- `cargo test --locked` exits 0
- Security changes include a regression test **and** a fail-closed test
- Docs updated in the same PR (this file, SDLC, or ADR as applicable)

---

## 12. Deployment and Operations

### 12.1 Local development

```powershell
# Native (recommended on Windows for GUI)
cargo run -- --backend http://localhost:8000

# Docker CI gate
docker compose run --rm tester

# Docker client + mock API
docker compose up -d mock-api
docker compose run --rm pinpoint --backend http://mock-api:8000
```

### 12.2 Production deployment

1. Build the release binary: `cargo build --release --locked`
2. Containerize (optional): `docker build -t pinpoint-desktop --target runtime .`
3. Deploy the binary or image to target hosts
4. Set `PINPOINT_BACKEND=https://api.example.com` (HTTPS required off-loopback)
5. Run the binary; it opens an egui window (desktop) or installs as APK
   (Android, via `cargo apk`)

The client is **stateless** except for on-device bookmarks. No server-side
state is kept in the client image.

### 12.3 Health checks

- `mock-api` service: `GET /health` → `{"status":"ok",...}`
- Rust client: no HTTP health endpoint (GUI application); verify by window
  opening and successful login.
- Backend health: `GET /health` on the Go/Python API (archived; future
  Rust backend TBD).

### 12.4 Rollback and roll-forward

- Container image rollback: redeploy previous `pinpoint-desktop:<tag>`.
- Binary rollback: replace the executable with the previous release build.
- No database migrations exist in the client (no server-side schema).
- Bookmarks are local JSON; rolling back does not affect them.

### 12.5 Monitoring and logging

- `tracing-subscriber` initialized in `src/main.rs`; set
  `RUST_LOG=debug` for verbose output.
- No telemetry, analytics, or crash reporting SDKs (0-tracker policy).
- Errors are shown to the user via egui toasts; details are **not** sent
  anywhere.

---

## 13. Troubleshooting

### 13.1 Docker engine 500 errors on Windows

Symptom: `docker ps` returns `500 Internal Server Error` on
`npipe://./pipe/docker_engine` or `dockerDesktopLinuxEngine`.

Cause: Docker Desktop's Linux VM backend is unhealthy or WSL2 is not
installed/running.

Workarounds:

1. Start Docker Desktop from the Start menu and wait for the whale icon to
   stabilize (can take 30–90 s).
2. Run `docker context use default` then retry.
3. Downgrade API version: `$env:DOCKER_API_VERSION='1.44'; docker ps`.
4. If WSL2 is not installed, run `wsl --install` (requires admin) and
   restart Docker Desktop.
5. If Docker remains unusable, use **native Rust builds** (§7) — the full
   CI gate runs natively without Docker.

### 13.2 Missing MSVC linker (link.exe)

Symptom: `cargo build` fails with `linker 'link.exe' not found`.

Fix: Install Visual Studio Build Tools 2022 with "Desktop development with
C++" (requires admin). Then create `.cargo/config.toml`:

```toml
[target.x86_64-pc-windows-msvc]
linker = "C:\\Program Files\\Microsoft Visual Studio\\2022\\BuildTools\\VC\\Tools\\MSVC\\<ver>\\bin\\Hostx64\\x64\\link.exe"
rustflags = [
  "-Clink-arg=/LIBPATH:C:\\...\\VC\\Tools\\MSVC\\<ver>\\lib\\x64",
  "-Clink-arg=/LIBPATH:C:\\...\\Windows Kits\\10\\Lib\\<ver>\\ucrt\\x64",
  "-Clink-arg=/LIBPATH:C:\\...\\Windows Kits\\10\\Lib\\<ver>\\um\\x64",
]
```

Use `/LIBPATH:` (colon), not `-LIBPATH=` (equals) — MSVC `link.exe` rejects
the equals form.

### 13.3 X11 forwarding failures

Symptom: `pinpoint` container exits immediately or shows "cannot open
display".

Cause: GUI forwarding not configured; Docker Desktop on Windows/macOS does
not forward X11 by default.

Fix: On Linux hosts, add `-e DISPLAY=$DISPLAY -v /tmp/.X11-unix:/tmp/.X11-
unix:rw -v $XAUTHORITY:$XAUTHORITY:ro`. On Windows/macOS, run the client
natively (§7.5) or use the `tester` service for headless validation.

### 13.4 Mock API connection refused

Symptom: Client cannot reach `http://mock-api:8000`.

Fix: Ensure `mock-api` is running (`docker compose ps`), and that both
services are on the same `pinpoint-net` network. Use the service name
`mock-api` (Docker DNS) rather than `localhost` from inside the client
container.

### 13.5 Podman rootless networking

Symptom: Container cannot reach the host or other containers.

Fix: Use `host.containers.internal` for the host; create a `podman network`
and attach both containers for inter-container communication. Rootless
networking uses `slirp4netns`; some corporate networks block its user-mode
stack. Switch to rootful (`sudo podman`) if needed.

---

## 14. Glossary

| Term | Definition |
|------|------------|
| ADR | Architecture Decision Record — immutable, numbered design decision |
| ODbL | Open Database License v1.0 — OSM data license |
| OSMF | OpenStreetMap Foundation |
| Nominatim | OSM geocoding service (search/reverse/lookup) |
| OSRM | Open Source Routing Machine (driving/foot/bike routing) |
| Overpass | OSM database query API (POI browse) |
| `geo:` URI | Standard deep link handled by OsmAnd/Organic Maps/Magic Earth |
| eframe/egui | Rust immediate-mode GUI framework |
| wasm32 | WebAssembly target (stub; GUI compiled out) |
| Slirp4netns | User-mode networking for rootless Podman |
| Directional consent | Per-pair share record (precision/purpose/expiry/state) |
| Precision downgrade | Server-side rounding of coordinates at read time |
| Stale/dead | Freshness states: warn (10–30 min) / stale (>30 min) |

---

## 15. Appendix: Command Cheat Sheet

### Native development (Windows, recommended for GUI)

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo build --locked
cargo test --locked
cargo run -- --backend http://localhost:8000
```

### Docker

```powershell
docker compose build
docker compose up -d mock-api
docker compose run --rm pinpoint --backend http://mock-api:8000
docker compose run --rm tester
docker compose down
```

### Linux-host Docker with X11

```bash
docker compose run --rm -e DISPLAY=$DISPLAY -v /tmp/.X11-unix:/tmp/.X11-unix:rw pinpoint
```

### Podman

```bash
podman build -t pinpoint-desktop:latest -f Containerfile.podman --target runtime .
podman run --rm pinpoint-desktop:tester
podman run --rm -e PINPOINT_BACKEND=http://host.containers.internal:8000 pinpoint-desktop:latest
```

### Android

```bash
rustup target add aarch64-linux-android
cargo apk build --locked
```

### iOS

```bash
rustup target add aarch64-apple-ios
cargo xcodebuild --locked
```

---

**End of documentation.** This file should be treated as the single source of
truth for building, containerizing, and operating the Pinpoint Rust
multiplatform client. When in doubt, prefer running the commands here over
memory; they are the ones validated by CI.
