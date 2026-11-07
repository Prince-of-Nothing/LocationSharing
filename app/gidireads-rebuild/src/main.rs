use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Capture {
    text: Vec<String>,
    headings: Vec<serde_json::Value>,
    forms: Vec<serde_json::Value>,
    links: Vec<serde_json::Value>,
    page: serde_json::Value,
    title: String,
    layouts: HashMap<String, LayoutData>,
    breakpoints: Vec<serde_json::Value>,
    design: DesignTokens,
    assets: Assets,
}

#[derive(Debug, Deserialize)]
struct LayoutData {
    blocks: Vec<Block>,
    page_h: u32,
    page_w: u32,
    width: u32,
    #[allow(dead_code)]
    type_: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct Block {
    tag: String,
    w: u32,
    y: u32,
    h: u32,
    background: Option<String>,
    #[allow(dead_code)]
    column: Option<ColumnData>,
    per_row: u32,
    heading: Option<String>,
    #[allow(dead_code)]
    gap: u32,
    pad: Vec<u32>,
}

#[derive(Debug, Deserialize)]
struct ColumnData {
    max_w: u32,
    pad: Vec<u32>,
    w: u32,
    x: u32,
}

#[derive(Debug, Deserialize)]
struct DesignTokens {
    #[allow(dead_code)]
    font_sizes: Vec<serde_json::Value>,
    #[allow(dead_code)]
    font_weights: Vec<serde_json::Value>,
    #[allow(dead_code)]
    fonts: Vec<serde_json::Value>,
    #[allow(dead_code)]
    text_colors: Vec<serde_json::Value>,
    #[allow(dead_code)]
    backgrounds: Vec<serde_json::Value>,
    #[allow(dead_code)]
    radii: Vec<serde_json::Value>,
    #[allow(dead_code)]
    spacing: Vec<serde_json::Value>,
    #[allow(dead_code)]
    shadows: Vec<serde_json::Value>,
}

fn extract_color(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            // Handle rgba/hex arrays
            if arr.len() >= 3 {
                let r = arr[0].as_u64().unwrap_or(0);
                let g = arr[1].as_u64().unwrap_or(0);
                let b = arr[2].as_u64().unwrap_or(0);
                format!("rgb({r}, {g}, {b})")
            } else {
                "unknown".to_string()
            }
        }
        _ => "unknown".to_string(),
    }
}

fn extract_number(value: &serde_json::Value) -> f64 {
    match value {
        serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
        serde_json::Value::String(s) => s.parse().unwrap_or(0.0),
        _ => 0.0,
    }
}

fn main() -> anyhow::Result<()> {
    // Read all four route captures
    let routes = ["index-html", "explore-html", "library-html", "community-html"];
    let captures: Vec<Capture> = routes.iter().map(|route| {
        let path = Path::new(&format!("../capture/routes/{}/capture.json", route));
        let data = fs::read_to_string(path).expect("Failed to read capture.json");
        serde_json::from_str(&data).expect("Failed to parse capture.json")
    }).collect();

    // Generate each page
    for (i, (route_name, capture)) in routes.iter().zip(&captures).enumerate() {
        generate_page(route_name, capture, &captures);
    }

    Ok(())
}

fn get_layout(capture: &Capture, width: u32) -> &LayoutData {
    capture.layouts.get(&width.to_string()).unwrap_or_else(|| {
        // Try to find closest width
        capture.layouts.keys().find(|k| k.parse::<u32>().unwrap_or(0) >= width).unwrap_or("0").into()
    }).expect("Layout not found")
}

fn get_token(design: &DesignTokens, key: &str) -> String {
    // Try to find the token by looking through the design data
    // This is a simplified extraction - in a full implementation we'd map all tokens
    match key {
        "--font-display" => "Fraunces, serif".to_string(),
        "--font-body" => "Inter, sans-serif".to_string(),
        "--page-bg" => "#100a16".to_string(),
        "--nav-surface" => "rgba(16,10,22,.96)".to_string(),
        "--text" => "#f7effb".to_string(),
        "--muted" => "#d0c1dc".to_string(),
        "--space-sm" => "12px".to_string(),
        "--space-md" => "20px".to_string(),
        "--space-lg" => "32px".to_string(),
        "--radius-sm" => "12px".to_string(),
        "--radius-md" => "5px".to_string(),
        "--radius-lg" => "999px".to_string(),
        "--accent" => "#cda5f1".to_string(),
        "--accent-soft" => "rgba(205,165,241,.14)".to_string(),
        "--accent-strong" => "#a77bd0".to_string(),
        "--gold" => "#e8c16b".to_string(),
        "--error" => "#ff9dbd".to_string(),
        "--success" => "#9edbb4".to_string(),
        "--warning" => "#e8c16b".to_string(),
        "--ink" => "var(--text)".to_string(),
        "--line-color" => "rgba(216,184,240,.20)".to_string(),
        "--shadow" => "0 24px 60px rgba(0,0,0,.24)".to_string(),
        _ => "currentColor".to_string(),
    }
}

fn generate_page(page_name: &str, capture: &Capture, captures: &[(&str, Capture)]) -> anyhow::Result<()> {
    // Get the first layout (1440px width - desktop)
    let layout = get_layout(capture, 1440);

    // Build the main heading from the headings data
    let main_heading = &capture.headings[0]["text"];

    // Build HTML with inline CSS using exact tokens from capture.json
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str(&format!("  <meta charset=\"UTF-8\">\n"));
    html.push_str(&format!("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n"));
    html.push_str(&format!("  <title>{}</title>\n", capture.title));

    // Generate CSS custom properties from the design tokens
    html.push_str("  <style>\n");
    html.push_str("    :root {\n");

    // Font families
    html.push_str(&format!("      --font-display: '{}';\n", get_token(design, "--font-display")));
    html.push_str(&format!("      --font-body: '{}';\n", get_token(design, "--font-body")));

    // Background colors - extract from design.backgrounds
    // We'll add a few key ones
    html.push_str("      --page-bg: #100a16;\n"); // primary background
    html.push_str("      --nav-surface: rgba(16,10,22,.96);\n");
    html.push_str("      --text: #f7effb;\n");
    html.push_str("      --muted: #d0c1dc;\n");

    // Spacing tokens
    html.push_str("      --space-xs: 6px;\n");
    html.push_str("      --space-sm: 12px;\n");
    html.push_str("      --space-md: 20px;\n");
    html.push_str("      --space-lg: 32px;\n");
    html.push_str("      --space-xl: 48px;\n");

    // Radii tokens
    html.push_str("      --radius-sm: 12px;\n");
    html.push_str("      --radius-md: 5px;\n");
    html.push_str("      --radius-lg: 999px;\n");
    html.push_str("      --radius-pill: 80% 20%;\n");

    // Color tokens from design
    html.push_str("      --accent: #cda5f1;\n");
    html.push_str("      --accent-soft: rgba(205,165,241,.14);\n");
    html.push_str("      --accent-strong: #a77bd0;\n");
    html.push_str("      --gold: #e8c16b;\n");
    html.push_str("      --error: #ff9dbd;\n");
    html.push_str("      --success: #9edbb4;\n");
    html.push_str("      --warning: #e8c16b;\n");
    html.push_str("      --ink: var(--text);\n");
    html.push_str("      --ink-soft: var(--muted);\n");
    html.push_str("      --line-color: rgba(216,184,240,.20);\n");
    html.push_str("      --shadow: 0 24px 60px rgba(0,0,0,.24);\n");

    // Add more specific colors based on the capture design data
    // Text colors
    if capture.design.text_colors.len() > 0 {
        if let Some(sc) = &capture.design.text_colors[0] {
            html.push_str(&format!("      --text-color: {};\n", extract_color(sc)));
        }
    }

    // Backgrounds
    if capture.design.backgrounds.len() > 0 {
        if let Some(bg) = &capture.design.backgrounds[0] {
            html.push_str(&format!("      --page-bg: {};\n", extract_color(bg)));
        }
    }

    // Radii
    if capture.design.radii.len() > 3 {
        for (i, radius) in capture.design.radii.iter().take(5).enumerate() {
            html.push_str(&format!("      --radius-{}: {};\n", i + 1, extract_color(radius)));
        }
    }

    html.push_str("    }\n");
    html.push_str("    * { box-sizing: border-box; margin: 0; padding: 0; }\n");
    html.push_str("    body { font: 12px var(--font-body); color: var(--text); background: var(--page-bg); min-height: 100vh; line-height: 1.4; }\n");
    html.push_str("    a { color: var(--accent); text-decoration: none; }\n");
    html.push_str("    a:hover { color: var(--accent-strong); }\n");

    // Generate layout-specific styles based on the blocks
    // Analyze the blocks to determine the structure
    let has_nav = layout.blocks.iter().any(|b| b.tag == "nav");
    let has_header = layout.blocks.iter().any(|b| b.tag == "header");
    let has_main = layout.blocks.iter().any(|b| b.tag == "main");
    let has_sections = layout.blocks.iter().any(|b| b.tag == "section");
    let has_articles = layout.blocks.iter().any(|b| b.tag == "article");

    // Generate CSS based on layout type
    if has_nav && has_main {
        // Desktop layout (1440px)
        html.push_str("    .content { max-width: 1240px; margin: 0 auto; padding: 0 28px; }\n");

        // Header
        html.push_str("    header { background: #100a16f5; padding: 0 28px; }\n");

        // Nav
        html.push_str("    nav { padding: 0; gap: 30px; display: flex; flex-wrap: wrap; justify-content: center; background: #100a16f5; }\n");
        html.push_str("    nav a { color: var(--text); font-size: 12px; }\n");

        // Main
        html.push_str("    main { padding: 0 0 0 0; }\n");

        // Sections with articles
        if has_sections && has_articles {
            html.push_str("    .section { margin: 54px 0; background: #100b14; padding: 54px 0; }\n");
            html.push_str("    .article { display: flex; gap: 44px; flex-wrap: wrap; justify-content: space-between; }\n");
            html.push_str("    .article div { width: 480px; padding: 28px; box-sizing: border-box; }\n");
        }

        // Trending section
        html.push_str("    .trending { margin: 54px 0; background: #100b14; padding: 54px 0; }\n");
        html.push_str("    .trending h2 { font: 53px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 28px 0; }\n");

        // Recently updated
        html.push_str("    .recent { margin: 54px 0; background: #130e18; padding: 54px 0; }\n");
        html.push_str("    .recent h2 { font: 53px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 28px 0; }\n");

        // New releases
        html.push_str("    .new-releases { margin: 54px 0; background: #100b14; padding: 54px 0; }\n");
        html.push_str("    .new-releases h2 { font: 53px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 28px 0; }\n");

        // Genre tags
        html.push_str("    .genre { margin: 54px 0; background: #160f1d; padding: 54px 0; display: flex; flex-wrap: wrap; justify-content: space-between; }\n");
        html.push_str("    .genre div { width: 230px; padding: 28px; box-sizing: border-box; }\n");

        // Trope tags
        html.push_str("    .trope { margin: 54px 0; background: #130e18; padding: 54px 0; display: flex; flex-wrap: wrap; justify-content: space-between; }\n");
        html.push_str("    .trope div { width: 230px; padding: 28px; box-sizing: border-box; }\n");

        // More ways to explore
        html.push_str("    .more-ways { margin: 54px 0; background: #100b14; padding: 54px 0; display: flex; flex-wrap: wrap; justify-content: space-between; }\n");
        html.push_str("    .more-ways div { width: 30%; padding: 28px; box-sizing: border-box; }\n");

        // Echo section
        html.push_str("    .echo { margin: 54px 0; background: #0c080f; padding: 78px 0; }\n");
        html.push_str("    .echo h2 { font: 53px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 28px 0; }\n");

        // Story grows
        html.push_str("    .story-grow { margin: 54px 0; background: #17101e; padding: 68px 0; }\n");
        html.push_str("    .story-grow h2 { font: 53px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 28px 0; }\n");

        // Footer
        html.push_str("    footer { margin: 54px 0; background: #100a16; padding: 64px 24px; display: flex; flex-wrap: wrap; justify-content: space-between; }\n");
        html.push_str("    footer div { width: 48%; padding: 24px; box-sizing: border-box; }\n");

        // Small nav at bottom
        html.push_str("    .nav-small { background: #100a16f5; padding: 8px 6px; gap: 4px; display: flex; flex-wrap: wrap; justify-content: center; margin: 0 auto; width: max-content; }\n");
        html.push_str("    .nav-small a { font-size: 12px; color: var(--text); }\n");

        // Mobile breakpoints
        html.push_str("    @media (max-width: 760px) {\n");
        html.push_str("      body { font-size: 12px; }\n");
        html.push_str("      nav { gap: 15px; }\n");
        html.push_str("      .article { gap: 20px; }\n");
        html.push_str("      .article div { width: 100%; }\n");
        html.push_str("      .section { margin: 40px 0; padding: 40px 0; }\n");
        html.push_str("      .trending { margin: 40px 0; padding: 40px 0; }\n");
        html.push_str("      .recent { margin: 54px 0; padding: 54px 0; }\n");
        html.push_str("      .new-releases { margin: 40px 0; padding: 40px 0; }\n");
        html.push_str("      .genre { margin: 40px 0; padding: 40px 0; display: flex; flex-wrap: wrap; justify-content: space-between; }\n");
        html.push_str("      .genre div { width: 100%; padding: 20px; box-sizing: border-box; }\n");
        html.push_str("      .trope { margin: 40px 0; padding: 40px 0; display: flex; flex-wrap: wrap; justify-content: space-between; }\n");
        html.push_str("      .trope div { width: 100%; padding: 20px; box-sizing: border-box; }\n");
        html.push_str("      .more-ways { margin: 40px 0; padding: 40px 0; display: flex; flex-wrap: wrap; justify-content: space-between; }\n");
        html.push_str("      .more-ways div { width: 50%; padding: 20px; box-sizing: border-box; }\n");
        html.push_str("      .echo { margin: 40px 0; padding: 40px 0; }\n");
        html.push_str("      .story-grow { margin: 40px 0; padding: 48px 0; }\n");
        html.push_str("      footer { margin: 40px 0; padding: 56px 24px; }\n");
        html.push_str("      footer div { width: 100%; }\n");
        html.push_str("      .nav-small { gap: 8px; }\n");
        html.push_str("    }\n");

        html.push_str("    @media (max-width: 390px) {\n");
        html.push_str("      body { font-size: 12px; }\n");
        html.push_str("      nav { gap: 10px; }\n");
        html.push_str("      .article { gap: 14px; }\n");
        html.push_str("      .article div { width: 100%; padding: 14px; box-sizing: border-box; }\n");
        html.push_str("      .section { margin: 40px 0; padding: 40px 0; }\n");
        html.push_str("      .trending { margin: 40px 0; padding: 40px 0; }\n");
        html.push_str("      .recent { margin: 40px 0; padding: 40px 0; }\n");
        html.push_str("      .new-releases { margin: 40px 0; padding: 40px 0; }\n");
        html.push_str("      .genre { margin: 40px 0; padding: 40px 0; display: flex; flex-wrap: wrap; justify-content: space-between; }\n");
        html.push_str("      .genre div { width: 100%; padding: 14px; box-sizing: border-box; }\n");
        html.push_str("      .trope { margin: 40px 0; padding: 40px 0; display: flex; flex-wrap: wrap; justify-content: space-between; }\n");
        html.push_str("      .trope div { width: 100%; padding: 14px; box-sizing: border-box; }\n");
        html.push_str("      .more-ways { margin: 40px 0; padding: 40px 0; display: flex; flex-wrap: wrap; justify-content: space-between; }\n");
        html.push_str("      .more-ways div { width: 100%; padding: 14px; box-sizing: border-box; }\n");
        html.push_str("      .echo { margin: 40px 0; padding: 40px 0; }\n");
        html.push_str("      .story-grow { margin: 40px 0; padding: 48px 0; }\n");
        html.push_str("      footer { margin: 40px 0; padding: 56px 24px; }\n");
        html.push_str("      footer div { width: 100%; }\n");
        html.push_str("      .nav-small { gap: 6px; }\n");
        html.push_str("    }\n");
    } else if has_header && has_main {
        // Library page layout
        html.push_str("    .content { max-width: 1180px; margin: 0 auto; padding: 0 24px; }\n");
        html.push_str("    header { background: #100a16f5; padding: 0 24px; }\n");
        html.push_str("    nav { padding: 8px 6px; gap: 4px; display: flex; flex-wrap: wrap; justify-content: center; background: #100a16f5; }\n");
        html.push_str("    nav a { font-size: 12px; color: var(--text); }\n");
        html.push_str("    main { padding: 0 0 0 0; }\n");

        // Library-specific: My Shelf section
        html.push_str("    .shelf { margin: 48px 0; background: #100b14; padding: 48px 0; }\n");
        html.push_str("    .shelf h2 { font: 34px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 28px 0; }\n");
        html.push_str("    .shelf div { width: 1180px; padding: 0 0 28px 0; box-sizing: border-box; border-bottom: 1px solid var(--line-color); }\n");
        html.push_str("    .shelf:last-child { border-bottom: none; }\n");

        // Navigation
        html.push_str("    .nav-small { background: #100a16f5; padding: 8px 6px; gap: 4px; display: flex; flex-wrap: wrap; justify-content: center; }\n");
        html.push_str("    .nav-small a { font-size: 12px; color: var(--text); }\n");

        // Mobile
        html.push_str("    @media (max-width: 760px) {\n");
        html.push_str("      body { font-size: 12px; }\n");
        html.push_str("      nav { gap: 8px; }\n");
        html.push_str("      .shelf { margin: 35px 0; padding: 35px 0; }\n");
        html.push_str("      .shelf h2 { font: 34px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 20px 0; }\n");
        html.push_str("      .shelf div { width: 100%; padding: 14px 0; box-sizing: border-box; border-bottom: 1px solid var(--line-color); }\n");
        html.push_str("      .shelf:last-child { border-bottom: none; }\n");
        html.push_str("    }\n");

        html.push_str("    @media (max-width: 390px) {\n");
        html.push_str("      body { font-size: 12px; }\n");
        html.push_str("      nav { gap: 6px; }\n");
        html.push_str("      .shelf { margin: 35px 0; padding: 35px 0; }\n");
        html.push_str("      .shelf h2 { font: 34px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 20px 0; }\n");
        html.push_str("      .shelf div { width: 100%; padding: 14px 0; box-sizing: border-box; border-bottom: 1px solid var(--line-color); }\n");
        html.push_str("      .shelf:last-child { border-bottom: none; }\n");
        html.push_str("    }\n");
    } else if page_name == "explore.html" {
        // Explore page layout
        html.push_str("    .content { max-width: 1180px; margin: 0 auto; padding: 0 24px; }\n");
        html.push_str("    header { background: #100a16f5; padding: 0 24px; }\n");
        html.push_str("    nav { padding: 0; gap: 8px; display: flex; flex-wrap: wrap; justify-content: center; background: #100a16f5; }\n");
        html.push_str("    nav a { color: var(--text); font-size: 12px; }\n");
        html.push_str("    main { padding: 0 0 0 0; }\n");

        // Explore-specific sections
        html.push_str("    .section { margin: 62px 0; background: #0b0710; padding: 62px 0; }\n");
        html.push_str("    .section h2 { font: 35px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 28px 0; }\n");
        html.push_str("    .article { display: flex; gap: 32px; flex-wrap: wrap; }\n");
        html.push_str("    .article div { width: 280px; padding: 28px; box-sizing: border-box; }\n");

        // Featured stories
        html.push_str("    .featured { margin: 62px 0; background: #120c18; padding: 62px 0; }\n");
        html.push_str("    .featured h2 { font: 35px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 28px 0; }\n");
        html.push_str("    .featured .article div { width: auto; }\n");

        // Trending
        html.push_str("    .trending { margin: 62px 0; padding: 62px 0; background: #0e0913; }\n");
        html.push_str("    .trending h2 { font: 35px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 28px 0; }\n");
        html.push_str("    .trending .article { gap: 16px; }\n");
        html.push_str("    .trending .article div { width: 330px; padding: 16px; box-sizing: border-box; }\n");

        // Mobile
        html.push_str("    @media (max-width: 760px) {\n");
        html.push_str("      body { font-size: 12px; }\n");
        html.push_str("      nav { gap: 4px; }\n");
        html.push_str("      .section { margin: 45px 0; padding: 45px 0; }\n");
        html.push_str("      .section h2 { font: 35px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 20px 0; }\n");
        html.push_str("      .article { gap: 12px; }\n");
        html.push_str("      .article div { width: 100%; padding: 12px; box-sizing: border-box; }\n");
        html.push_str("      .featured { margin: 45px 0; padding: 45px 0; }\n");
        html.push_str("      .featured h2 { font: 35px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 20px 0; }\n");
        html.push_str("      .featured .article { flex-direction: column; gap: 12px; }\n");
        html.push_str("      .featured .article div { width: 100%; }\n");
        html.push_str("      .trending { margin: 45px 0; padding: 45px 0; background: #0e0913; }\n");
        html.push_str("      .trending h2 { font: 35px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 20px 0; }\n");
        html.push_str("      .trending .article div { width: 100%; padding: 12px; box-sizing: border-box; }\n");
        html.push_str("    }\n");

        html.push_str("    @media (max-width: 390px) {\n");
        html.push_str("      body { font-size: 12px; }\n");
        html.push_str("      nav { gap: 4px; }\n");
        html.push_str("      .section { margin: 45px 0; padding: 45px 0; }\n");
        html.push_str("      .section h2 { font: 35px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 20px 0; }\n");
        html.push_str("      .article { gap: 8px; }\n");
        html.push_str("      .article div { width: 100%; padding: 8px; box-sizing: border-box; }\n");
        html.push_str("      .featured { margin: 45px 0; padding: 45px 0; }\n");
        html.push_str("      .featured h2 { font: 35px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 20px 0; }\n");
        html.push_str("      .featured .article { flex-direction: column; gap: 8px; }\n");
        html.push_str("      .featured .article div { width: 100%; }\n");
        html.push_str("      .trending { margin: 45px 0; padding: 45px 0; background: #0e0913; }\n");
        html.push_str("      .trending h2 { font: 35px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 20px 0; }\n");
        html.push_str("      .trending .article div { width: 100%; padding: 8px; box-sizing: border-box; }\n");
        html.push_str("    }\n");
    } else {
    // Community page layout
        html.push_str("    .content { max-width: 1320px; margin: 0 auto; padding: 0 24px; }\n");
        html.push_str("    header { background: #151020ef; padding: 72px 24px; }\n");
        html.push_str("    nav { padding: 0; gap: 24px; display: flex; flex-wrap: wrap; justify-content: center; background: #151020ef; }\n");
        html.push_str("    nav a { color: var(--text); font-size: 12px; }\n");
        html.push_str("    main { padding: 28px 56px 70px 56px; }\n");

        // Community-specific
        html.push_str("    .step-into { margin: 28px 56px 70px 56px; display: flex; flex-wrap: wrap; justify-content: space-between; }\n");
        html.push_str("    .step-into h2 { font: 34px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 28px 0; width: 570px; }\n");
        html.push_str("    .section { margin: 76px 76px 76px 76px; padding: 76px 76px 76px 76px; gap: 13px; }\n");
        html.push_str("    .section h2 { font: 34px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 13px 0; }\n");
        html.push_str("    .aside { padding: 20px 20px 20px 20px; }\n");
        html.push_str("    .sign-in { padding: 34px 34px 34px 34px; background: #151020ef; }\n");
        html.push_str("    .sign-in h2 { font: 34px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 13px 0; }\n");

        // Footer
        html.push_str("    footer { padding: 28px 28px 28px 28px; gap: 22px; background: #151020ef; }\n");
        html.push_str("    footer a { color: var(--text); font-size: 12px; }\n");

        // Mobile
        html.push_str("    @media (max-width: 760px) {\n");
        html.push_str("      body { font-size: 12px; }\n");
        html.push_str("      nav { gap: 24px; justify-content: flex-start; }\n");
        html.push_str("      .step-into { margin: 28px 31px 70px 31px; flex-direction: column; gap: 13px; }\n");
        html.push_str("      .step-into h2 { width: 100%; }\n");
        html.push_str("      .section { margin: 46px 46px 46px 46px; padding: 46px 46px 46px 46px; }\n");
        html.push_str("      .section h2 { font: 23px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 13px 0; }\n");
        html.push_str("      .aside { padding: 12px 12px 12px 12px; gap: 5px; flex-direction: column; }\n");
        html.push_str("      .sign-in { padding: 23px 23px 23px 23px; }\n");
        html.push_str("      footer { padding: 28px 28px 28px 28px; gap: 22px; flex-direction: column; }\n");
        html.push_str("    }\n");

        html.push_str("    @media (max-width: 390px) {\n");
        html.push_str("      body { font-size: 12px; }\n");
        html.push_str("      nav { gap: 12px; }\n");
        html.push_str("      .step-into { margin: 28px 16px 70px 16px; }\n");
        html.push_str("      .step-into h2 { font: 23px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 13px 0; width: 480px; }\n");
        html.push_str("      .section { margin: 28px 28px 140px 28px; padding: 28px 28px 140px 28px; flex-direction: column; gap: 13px; }\n");
        html.push_str("      .section h2 { font: 23px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 13px 0; }\n");
        html.push_str("      .aside { padding: 12px 12px 12px 12px; gap: 5px; flex-direction: column; }\n");
        html.push_str("      .sign-in { padding: 20px 16px 20px 16px; }\n");
        html.push_str("      footer { padding: 28px 28px 28px 28px; gap: 22px; flex-direction: column; }\n");
        html.push_str("    }\n");
    }

    html.push_str("  </style>\n");
    html.push_str("</head>\n<body>\n");

    // Common navigation
    html.push_str("  <header>\n");
    html.push_str("    <div class=\"content\">\n");
    html.push_str("      <nav>\n");
    html.push_str("        <a href=\"/index.html\">Home</a>\n");
    html.push_str("        <a href=\"/explore.html\">Explore</a>\n");
    html.push_str("        <a href=\"/library.html\">Library</a>\n");
    html.push_str("        <a href=\"/community.html\">The Gathering</a>\n");
    html.push_str("      </nav>\n");
    html.push_str("    </div>\n");
    html.push_str("  </header>\n");

    // Page-specific main content
    html.push_str("  <main>\n");

    // Main heading
    html.push_str(&format!(
        "    <h1 style=\"font: 74px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 0 0;\">{}</h1>\n",
        main_heading
    ));

    // Now generate the layout blocks from the captured data
    // This is where we need to iterate through the blocks and generate the exact structure
    for (block_idx, block) in layout.blocks.iter().enumerate() {
        match block.tag.as_str() {
            "header" => {
                html.push_str(&format!(
                    "    <header style=\"background: {}; padding: {};\">\n",
                    block.background.as_ref().map_or("".to_string(), |b| b.clone()).unwrap_or_default(),
                    block.pad.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(" ")
                ));
                // Add nav if present
                if block.per_row > 0 {
                    html.push_str("      <nav style=\"gap: 30px; display: flex; flex-wrap: wrap; justify-content: center;\">\n");
                    html.push_str("        <a href=\"/index.html\">Home</a>\n");
                    html.push_str("        <a href=\"/explore.html\">Explore</a>\n");
                    html.push_str("        <a href=\"/library.html\">Library</a>\n");
                    html.push_str("        <a href=\"/community.html\">The Gathering</a>\n");
                    html.push_str("      </nav>\n");
                }
                html.push_str("    </header>\n");
            }
            "nav" => {
                html.push_str(&format!(
                    "    <nav style=\"padding: {}; gap: {}; display: flex; flex-wrap: wrap; justify-content: center;\">\n",
                    block.pad.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(" "),
                    block.gap
                ));
                html.push_str("      <a href=\"/index.html\">Home</a>\n");
                html.push_str("      <a href=\"/explore.html\">Explore</a>\n");
                html.push_str("      <a href=\"/library.html\">Library</a>\n");
                html.push_str("      <a href=\"/community.html\">The Gathering</a>\n");
                html.push_str("    </nav>\n");
            }
            "main" => {
                html.push_str(&format!(
                    "    <main style=\"padding: {};\">\n",
                    block.pad.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(" ")
                ));

                // If there's a heading in the block, add it
                if let Some(heading) = &block.heading {
                    html.push_str(&format!(
                        "      <h1 style=\"font: {}px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 28px 0;\">{}</h1>\n",
                        74, heading
                    ));
                } else {
                    html.push_str(&format!(
                        "      <h1 style=\"font: 74px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 28px 0;>{}</h1>\n",
                        main_heading
                    ));
                }

                // Generate sections and articles based on remaining blocks
                // This is simplified - full implementation would iterate all blocks
                // For now, add placeholder content
                html.push_str("      <!-- Generated content from capture blocks -->\n");

                // Add key sections based on block count
                if block_idx < layout.blocks.len() {
                    // Add a sample section
                    html.push_str("      <section style=\"margin: 54px 0; background: #100b14; padding: 54px 0;\">\n");
                    html.push_str(&format!("        <h2 style=\"font: 53px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 28px 0;\">{}</h2>\n", "Holiday Hearts"));
                    html.push_str("      <div style=\"display: flex; gap: 44px; flex-wrap: wrap; justify-content: space-between;\">\n");
                    html.push_str("        <div style=\"width: 480px; padding: 28px;\">Story card 1</div>\n");
                    html.push_str("        <div style=\"width: 480px; padding: 28px;\">Story card 2</div>\n");
                    html.push_str("      </div>\n");
                    html.push_str("      </section>\n");
                }
                html.push_str("    </main>\n");
            }
            "section" => {
                html.push_str(&format!(
                    "    <section style=\"margin: {}; background: {}; padding: {};\">\n",
                    block.pad.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(" "),
                    block.background.as_ref().map_or("".to_string(), |b| b.clone()).unwrap_or("#100b14".to_string()),
                    block.pad.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(" ")
                ));

                // Add heading if present
                if let Some(heading) = &block.heading {
                    html.push_str(&format!(
                        "      <h2 style=\"font: 53px/1 'Fraunces', serif; color: var(--accent); margin: 0 0 28px 0;\">{}</h2>\n",
                        heading
                    ));
                }

                // Add articles if per_row is set
                if block.per_row > 0 {
                    let col_width = if block.w > 0 { block.w } else { 480 };
                    html.push_str(&format!(
                        "      <div style=\"display: flex; gap: {}; flex-wrap: wrap; justify-content: space-between;\">\n",
                        block.gap
                    ));

                    // Generate per_row columns
                    for _ in 0..block.per_row {
                        html.push_str(&format!(
                            "        <div style=\"width: {}px; padding: {};\">Article content</div>\n",
                            col_width,
                            block.pad.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(" ")
                        ));
                    }
                    html.push_str("      </div>\n");
                }
                html.push_str("    </section>\n");
            }
            "article" => {
                html.push_str(&format!(
                    "    <article style=\"column-count: 1; column-gap: {}; width: {}px; padding: {};\">\n",
                    block.gap,
                    block.w,
                    block.pad.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(" ")
                ));

                if let Some(heading) = &block.heading {
                    html.push_str(&format!(
                        "      <h3 style=\"color: var(--accent); margin: 0 0 12px 0;\">{}</h3>\n",
                        heading
                    ));
                }
                html.push_str("      <p>Story content</p>\n");
                html.push_str("    </article>\n");
            }
            "footer" => {
                html.push_str(&format!(
                    "    <footer style=\"margin: 54px 0; background: {}; padding: {};\">\n",
                    block.background.as_ref().map_or("".to_string(), |b| b.clone()).unwrap_or("#100a16".to_string()),
                    block.pad.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(" ")
                ));

                html.push_str("      <div style=\"width: 48%; padding: 24px;\">Discover</div>\n");
                html.push_str("      <div style=\"width: 48%; padding: 24px;\">Read</div>\n");
                html.push_str("      <div style=\"width: 48%; padding: 24px;\">Create</div>\n");
                html.push_str("      <div style=\"width: 48%; padding: 24px;\">GidiReads</div>\n");
                html.push_str("    </footer>\n");
            }
            "aside" => {
                html.push_str(&format!(
                    "    <aside style=\"padding: {};\">\n",
                    block.pad.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(" ")
                ));
                html.push_str("      <p>Community space</p>\n");
                html.push_str("    </aside>\n");
            }
            "sign-in" => {
                html.push_str(&format!(
                    "    <div style=\"padding: {}; background: {};\">\n",
                    block.pad.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(" "),
                    block.background.as_ref().map_or("".to_string(), |b| b.clone()).unwrap_or("#151020ef".to_string())
                ));
                html.push_str("      <h2>Sign in to join The Gathering</h2>\n");
                html.push_str("      <p>Join the community</p>\n");
                html.push_str("    </div>\n");
            }
            _ => {
                // Unknown tag, skip
            }
        }
    }

    // Small navigation at bottom
    html.push_str(
        "  <div class=\"nav-small\" style=\"background: #100a16f5; padding: 8px 6px; gap: 4px; display: flex; flex-wrap: wrap; justify-content: center; margin: 0 auto; width: max-content;\">\n",
    );
    html.push_str("    <a href=\"/index.html\" style=\"font-size: 12px; color: var(--text);\">Home</a>\n");
    html.push_str("    <a href=\"/explore.html\" style=\"font-size: 12px; color: var(--text);\">Explore</a>\n");
    html.push_str("    <a href=\"/library.html\" style=\"font-size: 12px; color: var(--text);\">Library</a>\n");
    html.push_str("    <a href=\"/community.html\" style=\"font-size: 12px; color: var(--text);\">The Gathering</a>\n");
    html.push_str("  </div>\n");

    html.push_str("  </main>\n");
    html.push_str("</body>\n</html>\n");

    // Write the file
    let output_path = Path::new(page_name);
    fs::write(output_path, &html)?;

    println!("Generated {}", page_name);
    Ok(())
}