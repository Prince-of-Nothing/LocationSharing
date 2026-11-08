# macOS packaging (`pinpoint-desktop` .app bundle)

## .app bundle (cargo-bundle, optional)

```bash
cargo install cargo-bundle --locked
cargo bundle --release --format app # uses [package.metadata.bundle]
open target/release/bundle/osx/Pinpoint.app
```

## Run without bundling

```bash
xcode-select --install   # CLT (provides link.exe equivalent + SDK)
cargo run --release -- --backend http://localhost:8000
```

## iOS

See [`ios/README.md`](../../ios/README.md) (`cargo-xcodebuild`) — the
`.app` bundle and the iOS build share the same `src/`.

## Status

Scaffolding only — cannot be built on this Windows host (no Xcode/macOS).
Build on a Mac or in CI (`macos-latest`).
