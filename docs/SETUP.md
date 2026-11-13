# Pinpoint — Setup & Installation Guide

This document covers installing and running Pinpoint on all supported
platforms: Windows, Linux, macOS, Android, and iOS, plus container
deployment with Docker and Podman.

---

## 1. System Requirements

### Desktop (Windows / Linux / macOS)

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| Rust | 1.70+ | 1.99+ |
| RAM | 2 GB | 4 GB+ |
| Disk | 1 GB | 5 GB+ |
| Display | Any | 1280x720+ |

**Windows-specific:** Visual Studio Build Tools 2022 with "Desktop
development with C++" workload (provides `link.exe` and Windows SDK).

**Linux-specific:** `build-essential`, `pkg-config`, `libssl-dev`,
`libxcb-render0-dev`, `libxcb-shape0-dev`, `libxcb-xfixes0-dev`,
`libxkbcommon-dev`, `libxkbcommon-x11-dev`, `libgl1-mesa-dev`,
`libfontconfig1-dev`, `libfreetype6-dev`.

**macOS-specific:** Xcode Command Line Tools.

### Mobile (Android / iOS)

**Android:** Android Studio, SDK 35, JDK 17, Rust target
`aarch64-linux-android`, `cargo-apk`.

**iOS:** Xcode, Rust target `aarch64-apple-ios`, `cargo-xcodebuild`.

---

## 2. Desktop Setup

### Step 1: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Choose the default stable installation. Restart your terminal after
installation.

### Step 2: Clone the Repository

```bash
git clone https://github.com/your-org/pinpoint.git
cd pinpoint
```

### Step 3: Install System Dependencies

**Windows:**
- Install Visual Studio Build Tools 2022 with "Desktop development with C++".
- This provides `link.exe` and Windows SDK `.lib` files.

**Linux (Debian/Ubuntu):**
```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev libxkbcommon-x11-dev libgl1-mesa-dev \
  libfontconfig1-dev libfreetype6-dev
```

**macOS:**
```bash
xcode-select --install
```

### Step 4: Build

```bash
cargo build --locked
```

### Step 5: Run

```bash
cargo run -- --backend http://localhost:8000
```

The app opens with the map as the first page. Sign in is optional — the map
works offline with demo locations.

---

## 3. Mobile Setup

### Android

```bash
# Add the Android target
rustup target add aarch64-linux-android

# Install cargo-apk
cargo install cargo-apk

# Build the APK
cargo apk build --locked
```

The debug APK will be at `target/aarch64-linux-android/debug/pinpoint-debug.apk`.

### iOS

```bash
# Add the iOS target
rustup target add aarch64-apple-ios

# Build with cargo-xcodebuild
cargo install cargo-xcodebuild
cargo xcodebuild --locked
```

---

## 4. Container Setup (Docker)

### Build the Image

```bash
docker build -t pinpoint-desktop:latest -f Dockerfile --target runtime .
```

### Run the Client (with X11 forwarding, Linux only)

```bash
docker run --rm \
  -e DISPLAY=$DISPLAY \
  -v /tmp/.X11-unix:/tmp/.X11-unix:rw \
  -v $XAUTHORITY:$XAUTHORITY:ro \
  pinpoint-desktop:latest
```

### Run the CI Gate (fmt, clippy, build, test)

```bash
docker build -t pinpoint-desktop:tester -f Dockerfile --target tester .
docker run --rm pinpoint-desktop:tester
```

### Docker Compose

```bash
# Start the mock API (offline development)
docker compose up -d mock-api

# Run the client pointing at the mock API
docker compose run --rm pinpoint --backend http://mock-api:8000

# Run the full CI gate
docker compose run --rm tester

# Stop everything
docker compose down
```

The compose file includes three services:
- **pinpoint** — The desktop client (runtime image)
- **tester** — CI gate (fmt, clippy, build, test)
- **mock-api** — Flask stand-in for the Pinpoint API

---

## 5. Container Setup (Podman)

### Build the Image

```bash
podman build -t pinpoint-desktop:latest -f Containerfile.podman --target runtime .
```

### Run the CI Gate

```bash
podman build -t pinpoint-desktop:tester -f Containerfile.podman --target tester .
podman run --rm pinpoint-desktop:tester
```

### Rootless Execution

Podman runs rootless by default on desktop systems. No `--privilege` or
Docker socket access is required.

```bash
podman run --rm -e PINPOINT_BACKEND=http://host.containers.internal:8000 \
  pinpoint-desktop:latest
```

---

## 6. Backend Setup

Pinpoint is a client-side application that talks to a Pinpoint API backend
over `/api/v1`. The backend is not included in this repository (it was
archived in the Rust-multiplatform pivot).

### Using the Mock API

For offline development, use the included Flask mock API:

```bash
# With Docker Compose
docker compose up -d mock-api

# Then run the client
cargo run -- --backend http://localhost:8000
```

The mock API implements login, register, friendships, location shares, and
location publishing with mock data. It is not a production backend.

### Connecting to a Real Backend

```bash
cargo run -- --backend https://api.example.com
```

Production backends require HTTPS. Plain HTTP is only allowed for loopback
development URLs.

---

## 7. Verifying the Installation

After setup, run the full quality gate:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo build --locked
cargo test --locked
```

All four commands should complete without errors. If any fail, see
`docs/TROUBLESHOOTING.md`.

---

## 8. Uninstalling

### Desktop

Simply delete the `pinpoint-desktop` binary and the `target/` directory:

```bash
rm -rf target/
```

Bookmarks are stored in `~/.pinpoint/bookmarks.json` (desktop) or app
DataStore (mobile). Delete these files to remove all local data.

### Mobile

Uninstall the app from your device. All local data (bookmarks, encrypted
tokens) is removed with the app.

---

## 9. Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.2.0 | 2026-11-08 | Rust multiplatform pivot; map-first UX; OSM search/routing/POI; bookmarks |
| 1.0.0 | 2026-11-13 | Real sessions, anonymous accounts, web UI, local-only assets, release packaging |
| 0.1.0 | — | Initial Rust desktop client |

---
