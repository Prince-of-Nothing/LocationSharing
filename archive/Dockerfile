# Dockerfile — Pinpoint Rust multiplatform client (Linux target)

# Syntax: docker/dockerfile:1
# Build stage: compile the Rust client for x86_64-unknown-linux-gnu.
# The desktop GUI (eframe/egui) needs a display; in headless CI the binary
# still builds and the test suite runs. For an interactive desktop session
# map an X11 socket or use VNC/noVNC.

ARG RUST_VERSION=1.99.1
ARG DEBIAN_VERSION=bookworm-slim

FROM rust:${RUST_VERSION}-${DEBIAN_VERSION} AS builder

# System dependencies needed by eframe/egui (winit/glow) at build time.
RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential \
      pkg-config \
      libssl-dev \
      libxcb-render0-dev \
      libxcb-shape0-dev \
      libxcb-xfixes0-dev \
      libxkbcommon-dev \
      libxkbcommon-x11-dev \
      libgl1-mesa-dev \
      libfontconfig1-dev \
      libfreetype6-dev \
      libxcb1-dev \
      libxcb-dri3-dev \
      libxcb-present-dev \
      libxcb-render-util0-dev \
      libxcb-icccm4-dev \
      libxcb-keysyms1-dev \
      libxcb-xinerama0-dev \
      libxrandr-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Cache dependencies by copying lock files first.
COPY Cargo.toml Cargo.lock ./

# Create a dummy main to pre-build dependencies (caches the dependency graph).
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    echo 'pub mod api; pub mod app; pub mod bookmarks; pub mod models; pub mod osm; pub mod platform;' > src/lib.rs && \
    cargo build --locked --release 2>/dev/null || true && \
    rm -rf src target/release/deps/pinpoint* target/release/*.d target/release/build

# Copy the real source and build.
COPY src ./src
COPY tests ./tests

RUN cargo build --locked --release

# ---------------------------------------------------------------------------
# Runtime stage: minimal image with just the binary and shared libs.
# ---------------------------------------------------------------------------
FROM debian:${DEBIAN_VERSION} AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates \
      libssl3 \
      libxcb1 \
      libxkbcommon0 \
      libxcb-render0 \
      libxcb-shape0 \
      libxcb-xfixes0 \
      libgl1 \
      libfontconfig1 \
      libfreetype6 \
    && rm -rf /var/lib/apt/lists/*

# Non-root user for safety.
RUN groupadd -r pinpoint && useradd -r -g pinpoint -d /home/pinpoint -s /sbin/nologin pinpoint

WORKDIR /app
COPY --from=builder /build/target/release/pinpoint-desktop /app/pinpoint-desktop

# Default backend URL — override at runtime with --backend https://api.example.com
ENV PINPOINT_BACKEND=http://host.docker.internal:8000
ENV PINPOINT_DATA_DIR=/home/pinpoint/.pinpoint

USER pinpoint

ENTRYPOINT ["/app/pinpoint-desktop"]
CMD ["--backend", "http://host.docker.internal:8000"]

# ---------------------------------------------------------------------------
# Test stage: run the full CI gate inside the container.
# ---------------------------------------------------------------------------
FROM builder AS tester

WORKDIR /build
COPY . ./
COPY --from=builder /usr/local/cargo /usr/local/cargo
COPY --from=builder /usr/local/rustup /usr/local/rustup

ENV PATH=/usr/local/cargo/bin:$PATH

CMD ["sh", "-c", "cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo build --locked && cargo test --locked"]
