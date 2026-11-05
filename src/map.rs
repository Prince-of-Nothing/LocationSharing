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

        // Map provider buttons
        ui.horizontal(|ui| {
            if ui.button("OpenStreetMap").clicked() {
                open_map_in_browser(ui.ctx(), &all_locations[0], "osm");
            }
            if ui.button("Google Maps").clicked() {
                open_map_in_browser(ui.ctx(), &all_locations[0], "google");
            }
            if ui.button("Magic Earth").clicked() {
                open_map_in_browser(ui.ctx(), &all_locations[0], "magic");
            }
        });
    });

    ui.add_space(10.0);

    // Show location details list
    draw_location_list(ui, &all_locations);
}

/// Open a map in the default browser for the given location and provider
fn open_map_in_browser(ctx: &egui::Context, loc: &FriendLocation, provider: &str) {
    let url = match provider {
        "osm" => format!(
            "https://www.openstreetmap.org/#map=10/{:.4}/{:.4}",
            loc.latitude, loc.longitude
        ),
        "google" => format!(
            "https://www.google.com/maps/@{:.4},{:.4},.zm",
            loc.latitude, loc.longitude
        ),
        "magic" => format!(
            "https://www.earth.google.com/web/@{:.4},{:.4},.zm",
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
            ui.add_space(5.0);
        }
    });
}
