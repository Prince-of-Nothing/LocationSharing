//! Map visualization module for friend locations

use crate::models::{AppData, FriendLocation};
use eframe::egui::{self, Color32, RichText, Ui};

/// Render a simple map display showing friend locations
pub fn render_map(ui: &mut Ui, state: &mut AppData) {
    // Collect all locations to display
    let mut all_locations: Vec<FriendLocation> = Vec::new();
    all_locations.extend(state.locations.iter().cloned());

    if all_locations.is_empty() {
        ui.group(|ui| {
            ui.label(RichText::new("No locations shared yet").size(16.0));
            ui.label(
                RichText::new("Map visualization will be rendered here")
                    .size(14.0)
                    .color(Color32::from_rgb(148, 163, 184)),
            );
        });
        return;
    }

    // Draw the map visualization
    ui.group(|ui| {
        ui.label(RichText::new("Friend locations map").size(16.0).strong());
        ui.add_space(10.0);

        // Calculate bounds
        let lats: Vec<f64> = all_locations.iter().map(|l| l.latitude).collect();
        let lons: Vec<f64> = all_locations.iter().map(|l| l.longitude).collect();
        let min_lat = lats.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_lat = lats.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_lon = lons.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_lon = lons.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Draw a simple map representation
        let painter = ui.painter();
        let rect = ui.available_rect_before_wrap();
        let map_rect = egui::Rect::from_min_size(
            rect.min + egui::vec2(10.0, 10.0),
            egui::vec2(rect.width() - 20.0, 300.0),
        );

        // Draw map background
        painter.rect_filled(map_rect, 4.0, Color32::from_rgb(15, 23, 42));
        painter.rect_stroke(
            map_rect,
            4.0_f32,
            egui::Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)),
            egui::StrokeKind::Inside,
        );

        // Draw pins for each location
        for loc in &all_locations {
            let x = map_rect.left()
                + ((loc.longitude - min_lon) / (max_lon - min_lon + 0.001)
                    * map_rect.width() as f64) as f32;
            let y = map_rect.top()
                + ((max_lat - loc.latitude) / (max_lat - min_lat + 0.001)
                    * map_rect.height() as f64) as f32;

            let pin_pos = egui::pos2(x, y);
            painter.circle_filled(pin_pos, 8.0, Color32::from_rgb(248, 113, 113));
            painter.circle_stroke(pin_pos, 8.0, egui::Stroke::new(2.0_f32, Color32::BLACK));

            // Name label
            painter.text(
                pin_pos + egui::vec2(0.0, -20.0),
                egui::Align2::CENTER_BOTTOM,
                &loc.user_name,
                egui::FontId::proportional(12.0),
                Color32::WHITE,
            );
        }

        ui.allocate_rect(map_rect, egui::Sense::hover());

        ui.add_space(320.0);

        // Legend
        ui.label(RichText::new("Legend").size(12.0).strong());
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Fresh")
                    .size(12.0)
                    .color(Color32::from_rgb(52, 211, 153)),
            );
            ui.label(
                RichText::new("Approximate")
                    .size(12.0)
                    .color(Color32::from_rgb(239, 68, 68)),
            );
            ui.label(
                RichText::new("Exact")
                    .size(12.0)
                    .color(Color32::from_rgb(250, 204, 21)),
            );
        });

        // Map provider buttons (OSM data — ODbL; see docs/sdlc/07-openstreetmap-integration.md)
        // Project reference: https://wiki.openstreetmap.org/wiki/Main_Page
        ui.horizontal(|ui| {
            if ui.button("OpenStreetMap").clicked() {
                open_map_in_browser(ui.ctx(), &all_locations[0], "osm");
            }
            if ui.button("Directions (OSM)").clicked() {
                open_map_in_browser(ui.ctx(), &all_locations[0], "directions");
            }
            if ui.button("Organic Maps").clicked() {
                open_map_in_browser(ui.ctx(), &all_locations[0], "organic");
            }
            if ui.button("Google Maps").clicked() {
                open_map_in_browser(ui.ctx(), &all_locations[0], "google");
            }
        });
        ui.label(
            RichText::new("Map data © OpenStreetMap contributors (ODbL)")
                .size(10.0)
                .color(Color32::from_rgb(148, 163, 184)),
        );
    });

    ui.add_space(10.0);

    // Show location details list
    draw_location_list(ui, &all_locations);
}

/// Open a map in the default browser for the given location and provider.
/// All URLs are OSM-data based (ODbL): OSM standard, OSM directions, and
/// Organic Maps (https://organicmaps.app). Reference:
/// https://wiki.openstreetmap.org/wiki/Main_Page — Magic Earth
/// (https://wiki.openstreetmap.org/wiki/Magic_Earth) is an OSM-data
/// consumer too; we link to OSM-native targets instead of proprietary
/// deep links so the desktop client stays OSM-only by default.
fn open_map_in_browser(ctx: &egui::Context, loc: &FriendLocation, provider: &str) {
    let url = match provider {
        "osm" => format!(
            "https://www.openstreetmap.org/#map=10/{:.4}/{:.4}",
            loc.latitude, loc.longitude
        ),
        "directions" => format!(
            "https://www.openstreetmap.org/directions?to={:.4}%2C{:.4}",
            loc.latitude, loc.longitude
        ),
        "organic" => {
            // Organic Maps web export format (OSM data, offline-first).
            format!(
                "https://organicmaps.app/?ll={:.4}%2C{:.4}&z=14",
                loc.latitude, loc.longitude
            )
        }
        "google" => format!(
            "https://www.google.com/maps/dir/?api=1&destination={:.4},{:.4}",
            loc.latitude, loc.longitude
        ),
        _ => return,
    };

    if let Ok(path) = which::which("xdg-open") {
        let _ = std::process::Command::new(path).arg(&url).spawn();
    } else if let Ok(path) = which::which("open") {
        let _ = std::process::Command::new(path).arg(&url).spawn();
    } else {
        ctx.open_url(egui::OpenUrl::new_tab(url));
    }
}

fn draw_location_list(ui: &mut Ui, locations: &[FriendLocation]) {
    ui.group(|ui| {
        ui.label(RichText::new(format!("Locations: {}", locations.len())).size(14.0));

        for loc in locations {
            let precision_color = match loc.precision_level.as_str() {
                "exact" => Color32::from_rgb(250, 204, 21),
                "approximate" => Color32::from_rgb(239, 68, 68),
                _ => Color32::from_rgb(148, 163, 184),
            };

            ui.horizontal(|ui| {
                ui.label(RichText::new(&loc.user_name).size(13.0));
                ui.add_space(50.0);
                ui.label(
                    RichText::new(format!("{:.4}, {:.4}", loc.latitude, loc.longitude)).size(12.0),
                );
                ui.add_space(20.0);
                ui.label(
                    RichText::new(&loc.precision_level)
                        .size(10.0)
                        .color(precision_color),
                );
            });
            // Distance from first demo point (proxy for "near me" when no GPS).
            let anchor = (47.0105, 28.8638);
            let d = crate::models::haversine_m(anchor.0, anchor.1, loc.latitude, loc.longitude);
            let b = crate::models::bearing_deg(anchor.0, anchor.1, loc.latitude, loc.longitude);
            // Freshness badge (honest: stale/dead never shown as live).
            let now = chrono::Utc::now().timestamp();
            let fresh = crate::models::freshness_state(&loc.updated_at, now);
            let fresh_color = match fresh {
                "live" => Color32::from_rgb(52, 211, 153),
                "fresh" => Color32::from_rgb(250, 204, 21),
                "stale" => Color32::from_rgb(249, 115, 22),
                "dead" => Color32::from_rgb(239, 68, 68),
                _ => Color32::from_rgb(148, 163, 184),
            };
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} {} · updated {}",
                        crate::models::format_distance(d, true),
                        crate::models::compass_label(b),
                        loc.updated_at
                    ))
                    .size(10.0)
                    .color(Color32::from_rgb(148, 163, 184)),
                );
                ui.colored_label(fresh_color, RichText::new(format!("● {fresh}")).size(10.0));
                if ui.small_button("★ Save").clicked() {
                    let store = crate::bookmarks::BookmarkStore::default_location();
                    store.add(&loc.user_name, loc.latitude, loc.longitude);
                }
                if ui.small_button("Address").clicked() {
                    // Handled by Places screen reverse lookup; here just notice.
                    ui.ctx().request_repaint();
                }
            });
            ui.add_space(5.0);
        }
    });
}
