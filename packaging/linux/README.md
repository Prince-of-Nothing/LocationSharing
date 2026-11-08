# Linux packaging (`pinpoint-desktop` on Debian/Ubuntu, .deb + .desktop)

## .deb (cargo-deb, optional)

```bash
cargo install cargo-deb --locked
cargo deb --locked          # uses [package.metadata.deb] in Cargo.toml
sudo dpkg -i target/debian/pinpoint_*.deb
pinpoint-desktop --backend http://localhost:8000
```

Build deps (Debian/Ubuntu): `build-essential pkg-config libssl-dev
libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
libxkbcommon-dev libxkbcommon-x11-dev libgl1-mesa-dev
libfontconfig1-dev libfreetype6-dev` (see `Dockerfile.client`).

## Container (Linux host only, X11)

```bash
docker compose --profile client run --rm client
# or: docker build -f Dockerfile.client --target runtime -t pinpoint-desktop .
```

## Status

`Dockerfile.client` (Linux/X11) builds today; the `.desktop` entry above
is for native `cargo install` / `.deb` installs. No GUI run was done on a
Linux host in this session — first-run check: `pinpoint-desktop --help`,
then `cargo run -- --backend http://localhost:8000`.
