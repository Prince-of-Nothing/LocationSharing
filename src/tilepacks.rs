//! Offline tile packs — user-initiated, throttled downloads of map tiles to
//! the on-device app dir so Pinpoint works offline.
//!
//! OSM policy: never bulk-scrapes `tile.openstreetmap.org`; packs fetch from
//! CARTO (same provider as the tile proxy) at ≤1 req/s, only for an explicit
//! bounding box the user chose, capped to 4096 tiles per pack.

use std::path::PathBuf;
use std::time::Duration;

use crate::platform;

/// Slippy tile for `(lat, lon)` at zoom `z` (Web Mercator, OSM standard).
pub fn latlon_to_tile(lat: f64, lon: f64, z: u8) -> (u32, u32) {
    let n = 1u32 << z;
    let lat_r = lat.to_radians();
    let x = n as f64 * (lon + 180.0) / 360.0;
    let y = n as f64 * (1.0 - (lat_r.tan() + 1.0 / lat_r.cos()).ln() / std::f64::consts::PI) / 2.0;
    let max = n.saturating_sub(1);
    ((x.floor() as u32).min(max), (y.floor() as u32).min(max))
}

/// Inclusive tile range covering `[(min_lat, min_lon), (max_lat, max_lon)]` at `z`.
pub fn bbox_tiles(
    min_lat: f64,
    min_lon: f64,
    max_lat: f64,
    max_lon: f64,
    z: u8,
) -> Vec<(u32, u32)> {
    let (x0, y0) = latlon_to_tile(max_lat, min_lon, z);
    let (x1, y1) = latlon_to_tile(min_lat, max_lon, z);
    let (x_lo, x_hi) = (x0.min(x1), x0.max(x1));
    let (y_lo, y_hi) = (y0.min(y1), y0.max(y1));
    let mut out = Vec::new();
    for y in y_lo..=y_hi {
        for x in x_lo..=x_hi {
            out.push((x, y));
        }
    }
    out
}

pub const PACK_TILE_CAP: usize = 4096;
pub const CARTO_BASE: &str = "https://a.basemaps.cartocdn.com/dark_all";

/// CARTO now requires an API key (free: https://carto.com/basemaps/apikey).
/// Operators set `CARTO_API_KEY`; otherwise keyless requests are attempted
/// (upstream decides). Test-time packs use the no-key path.
/// Download `tiles` (z/x/y triples, from [`bbox_tiles`] or a bookmark) into
/// `tiles/{z}/{x}/{y}.png` under the app data dir. Skips files already
/// present (resume-safe). Returns (downloaded, skipped, failed).
pub async fn download_pack(
    tiles: &[(u32, u32, u32)],
    client: Option<reqwest::Client>,
) -> (usize, usize, usize) {
    let Some(base) = platform::app_data_dir() else {
        return (0, 0, tiles.len());
    };
    let root = base.join("tiles");
    let client = match client {
        Some(c) => c,
        None => match reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("PinpointTiles/0.2.0 (offline packs, user-initiated)")
            .build()
        {
            Ok(c) => c,
            Err(_) => return (0, 0, tiles.len()),
        },
    };
    let (mut ok, mut skipped, mut failed) = (0, 0, 0);
    for (i, (z, x, y)) in tiles.iter().take(PACK_TILE_CAP).enumerate() {
        let dir = root.join(z.to_string()).join(x.to_string());
        let file = dir.join(format!("{y}.png"));
        if file.exists() {
            skipped += 1;
            continue;
        }
        // OSM courtesy throttle: ≤1 req/s.
        if i > 0 {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        let key_param = match std::env::var("CARTO_API_KEY") {
            Ok(k) if !k.trim().is_empty() => format!("?apikey={}", k.trim()),
            _ => String::new(),
        };
        let url = format!("{CARTO_BASE}/{z}/{x}/{y}.png{key_param}");
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => match resp.bytes().await {
                Ok(body) => {
                    if std::fs::create_dir_all(&dir).is_err() {
                        failed += 1;
                        continue;
                    }
                    match std::fs::write(&file, body) {
                        Ok(()) => ok += 1,
                        Err(_) => failed += 1,
                    }
                }
                Err(_) => failed += 1,
            },
            _ => failed += 1,
        }
    }
    (ok, skipped, failed)
}

/// Bytes currently occupied by offline tile packs (0 when unavailable).
pub fn packs_size() -> u64 {
    let Some(base) = platform::app_data_dir() else {
        return 0;
    };
    let root = base.join("tiles");
    let mut total = 0u64;
    if let Ok(entries) = walk_files(&root) {
        for f in entries {
            total += std::fs::metadata(f).map(|m| m.len()).unwrap_or(0);
        }
    }
    total
}

fn walk_files(dir: &std::path::Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{bbox_tiles, latlon_to_tile};

    #[test]
    fn slippy_origin() {
        // (0,0) at z=0 is tile (0,0); at z=1 the SE quadrant is (1,1).
        assert_eq!(latlon_to_tile(0.0, 0.0, 0), (0, 0));
        assert_eq!(latlon_to_tile(10.0, 90.0, 1), (1, 0));
        assert_eq!(latlon_to_tile(-10.0, -90.0, 1), (0, 1));
    }

    #[test]
    fn bbox_small_area_single_tile() {
        let tiles = bbox_tiles(41.88, -87.63, 41.89, -87.62, 14);
        assert!(!tiles.is_empty() && tiles.len() <= 9, "got {}", tiles.len());
    }

    #[test]
    fn bbox_inverted_corners_normalized() {
        let a = bbox_tiles(41.88, -87.63, 41.89, -87.62, 12);
        let b = bbox_tiles(41.89, -87.62, 41.88, -87.63, 12);
        assert_eq!(a.len(), b.len());
    }
}
