//! GidiReads Static Site Generator
//! Rebuilds the site exactly matching capture.json design tokens and layout

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

mod theme;
mod layout;
mod generate_index;
mod generate_explore;
mod generate_library;
mod generate_community;

use theme::*;
use layout::*;
use generate_index::*;
use generate_explore::*;
use generate_library::*;
use generate_community::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dist = PathBuf::from("dist");
    if dist.exists() {
        fs::remove_dir_all(&dist)?;
    }
    fs::create_dir_all(&dist)?;
    fs::create_dir_all(dist.join("assets"))?;

    // Copy logo placeholder
    copy_logo_placeholder(&dist.join("assets"))?;

    // Generate CSS
    let css = generate_theme_css();
    fs::write(dist.join("style.css"), css)?;

    // Generate four pages
    let index_html = generate_index_html();
    fs::write(dist.join("index.html"), index_html)?;

    let explore_html = generate_explore_html();
    fs::write(dist.join("explore.html"), explore_html)?;

    let library_html = generate_library_html();
    fs::write(dist.join("library.html"), library_html)?;

    let community_html = generate_community_html();
    fs::write(dist.join("community.html"), community_html)?;

    // Generate manifest.webmanifest
    let manifest = generate_manifest();
    fs::write(dist.join("manifest.webmanifest"), manifest)?;

    // Generate fonts.css (inline font-face)
    let fonts_css = generate_fonts_css();
    fs::write(dist.join("fonts.css"), fonts_css)?;

    // Generate other CSS files referenced by original
    generate_auxiliary_css(&dist)?;

    println!("Build complete! Output in app/dist/");
    Ok(())
}

fn copy_logo_placeholder(assets: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple SVG placeholder for the logo
    let logo_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><defs><linearGradient id="g" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" stop-color="#cda5f1"/><stop offset="100%" stop-color="#a77bd0"/></linearGradient></defs><rect width="512" height="512" fill="#100a16"/><text x="256" y="320" font-family="Fraunces, serif" font-size="120" font-weight="700" fill="url(#g)" text-anchor="middle">G</text></svg>"#;
    fs::write(assets.join("gidireads-logo.svg"), logo_svg)?;
    // Also create PNG placeholder as 1x1 transparent (we can't generate real PNG without deps)
    fs::write(assets.join("gidireads-logo.png"), vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])?; // minimal PNG header
    Ok(())
}

fn generate_fonts_css() -> String {
    r#"
/* fonts.css - Font faces for GidiReads */
@font-face {
    font-family: 'Inter';
    font-style: normal;
    font-weight: 400 700;
    font-display: swap;
    src: local('Inter'), url('https://fonts.gstatic.com/s/inter/v18/UcCO3FwrK3iLTeHuS_nVMrMxCp50SjIw2boKoduKmMEVuLyfAZ9hjp-Ek-_EeA.woff2') format('woff2');
}
@font-face {
    font-family: 'Fraunces';
    font-style: normal;
    font-weight: 400 700;
    font-display: swap;
    src: local('Fraunces'), url('https://fonts.gstatic.com/s/fraunces/v26/LDIzapSCOBgTFS-QU7dcjzE2nFyS.woff2') format('woff2');
}
@font-face {
    font-family: 'Fraunces';
    font-style: italic;
    font-weight: 400 700;
    font-display: swap;
    src: local('Fraunces Italic'), url('https://fonts.gstatic.com/s/fraunces/v26/LDI1apSCOBgTFS-QU7dcjzE2nFySm5s.woff2') format('woff2');
}
"#.to_string()
}

fn generate_manifest() -> String {
    r#"{
  "name": "GidiReads — Stories begin here",
  "short_name": "GidiReads",
  "start_url": "/index.html",
  "display": "standalone",
  "background_color": "#100a16",
  "theme_color": "#100a16",
  "icons": [
    {
      "src": "assets/gidireads-logo.png",
      "sizes": "512x512",
      "type": "image/png"
    }
  ]
}"#.to_string()
}

fn generate_auxiliary_css(dist: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // These are the CSS files the original references
    // We'll include their content in style.css but create empty files for 404 prevention
    let files = [
        "taxonomy.css",
        "global-dark-plum.css", 
        "homepage-collections.css",
        "holiday-rail.css",
        "home-redesign.css",
        "community.css",
        "explore.css",
        "library.css",
        "profile-redesign.css",
        "my-gidireads.css",
        "economy-design-system.css"
    ];
    for f in files {
        fs::write(dist.join(f), "/* generated - included in style.css */")?;
    }
    Ok(())
}