//! Places screen (routes / POIs / reverse)
use super::{PendingAction, PinpointAppUI};
use eframe::egui::{self, RichText};

impl PinpointAppUI {
    pub fn draw_places_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.draw_main_toolbar(ui);
        ui.heading("Places — route, nearby POIs & address lookup");
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.label(RichText::new("Route (OSRM)").size(14.0).strong());
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.route_from_lat)
                        .hint_text("From lat")
                        .desired_width(90.0),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.route_from_lon)
                        .hint_text("From lon")
                        .desired_width(90.0),
                );
                for m in ["foot", "bike", "driving"] {
                    ui.selectable_value(&mut self.route_mode, m.to_string(), m);
                }
            });
            if ui
                .button("Route to selected friend / first demo point")
                .clicked()
            {
                let flat = self.route_from_lat.trim().parse::<f64>().unwrap_or(41.8902);
                let flon = self
                    .route_from_lon
                    .trim()
                    .parse::<f64>()
                    .unwrap_or(-87.6323);
                let to = (41.8781, -87.6298);
                self.spawn_action(
                    PendingAction::Route {
                        from: (flat, flon),
                        to,
                        mode: self.route_mode.clone(),
                    },
                    ctx,
                );
            }
        });
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.label(RichText::new("Nearby (Overpass)").size(14.0).strong());
            for cat in [
                "food", "fuel", "pharmacy", "hospital", "hotel", "tourism", "water", "parking",
                "park",
            ] {
                if ui.selectable_label(self.poi_category == cat, cat).clicked() {
                    self.poi_category = cat.to_string();
                }
            }
            if ui.button("Find nearby").clicked() {
                self.spawn_action(
                    PendingAction::Poi {
                        lat: 41.8902,
                        lon: -87.6323,
                        category: self.poi_category.clone(),
                    },
                    ctx,
                );
            }
        });
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.label(RichText::new("Reverse geocode").size(14.0).strong());
            ui.add(
                egui::TextEdit::singleline(&mut self.reverse_label)
                    .hint_text("lat,lon e.g. 41.89,-87.63")
                    .desired_width(220.0),
            );
            if ui.button("Lookup address").clicked() {
                let s = self.reverse_label.clone();
                {
                    let parts: Vec<&str> = s.split(',').collect();
                    if parts.len() == 2 {
                        if let (Ok(lat), Ok(lon)) = (
                            parts[0].trim().parse::<f64>(),
                            parts[1].trim().parse::<f64>(),
                        ) {
                            self.spawn_action(PendingAction::Reverse { lat, lon }, ctx);
                        }
                    }
                }
            }
        });
    }
}
