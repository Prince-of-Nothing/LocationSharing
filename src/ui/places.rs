//! Places screen (routes / POIs / reverse)
use super::{PendingAction, PinpointAppUI};
use eframe::egui::{self, RichText};

impl PinpointAppUI {
    pub fn draw_places_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.draw_main_toolbar(ui);
        ui.heading("Places — route, nearby POIs & address lookup");
        ui.add_space(8.0);

        // Route form
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

        // POI form
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

        // Reverse form
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

        // Nearby POI results (name, hours, phone, website, save, OSM link)
        let pois = { self.app.data().lock().unwrap().poi_results.clone() };
        if !pois.is_empty() {
            ui.add_space(8.0);
            ui.group(|ui| {
                ui.label(
                    RichText::new(format!("Nearby places ({})", pois.len()))
                        .size(14.0)
                        .strong(),
                );
                for poi in &pois {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} {}", poi.icon, poi.name));
                        if !poi.opening_hours.is_empty() {
                            ui.label(RichText::new(&poi.opening_hours).size(10.0));
                        }
                        if !poi.phone.is_empty() {
                            ui.label(RichText::new(&poi.phone).size(10.0));
                        }
                        if ui.small_button("Save").clicked() {
                            let store = crate::bookmarks::BookmarkStore::default_location();
                            store.add(&poi.name, poi.latitude, poi.longitude);
                            self.app.set_notice(Some(format!("Saved {}", poi.name)));
                        }
                        if ui.small_button("OSM").clicked() {
                            let url = format!(
                                "https://www.openstreetmap.org/?mlat={:.4}&mlon={:.4}#map=16/{:.4}/{:.4}",
                                poi.latitude, poi.longitude, poi.latitude, poi.longitude
                            );
                            let _ = webbrowser::open(&url);
                        }
                    });
                    if !poi.website.is_empty() {
                        ui.hyperlink_to(RichText::new(&poi.website).size(10.0), &poi.website);
                    }
                    ui.add_space(2.0);
                }
            });
        }

        // Route results (summary, steps, GPX export)
        let route = { self.app.data().lock().unwrap().last_route.clone() };
        if let Some(route) = route {
            ui.add_space(8.0);
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "Route — {:.2} km, ~{} min",
                            route.distance_m / 1000.0,
                            (route.duration_s / 60.0) as i64
                        ))
                        .size(14.0)
                        .strong(),
                    );
                    if ui.small_button("Export GPX").clicked() {
                        self.spawn_action(
                            PendingAction::ExportGpx {
                                name: "Pinpoint route".to_string(),
                            },
                            ctx,
                        );
                    }
                });
                if !route.summary.is_empty() {
                    ui.label(RichText::new(&route.summary).size(11.0));
                }
                for step in &route.steps {
                    ui.label(format!("• {}", step.text));
                }
            });
        }
    }
}
