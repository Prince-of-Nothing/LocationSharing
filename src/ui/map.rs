//! Map screen
use super::{PendingAction, PinpointAppUI};
use crate::map;
use crate::models::FriendLocation;
use eframe::egui::{self, Color32, Direction, RichText};

impl PinpointAppUI {
    pub fn draw_map_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.with_layout(
                egui::Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    ui.heading("Friend locations");
                },
            );
        });
        ui.add_space(8.0);

        {
            let binding = self.app.data();
            let data = binding.lock().unwrap();
            let unknown = "Unknown".to_string();
            let user_email = data.user.as_ref().map(|u| &u.email).unwrap_or(&unknown);
            ui.label(format!("Signed in as: {}", user_email));
        }

        // Quick location publish (persistent inputs so typing survives frames)
        ui.group(|ui| {
            ui.label(RichText::new("Publish my location").size(12.0).strong());
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.publish_lat)
                        .hint_text("Latitude")
                        .desired_width(100.0),
                );
                ui.add_space(8.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.publish_lon)
                        .hint_text("Longitude")
                        .desired_width(100.0),
                );
            });
            ui.add_space(4.0);
            if ui.button("Publish").clicked() {
                match (
                    self.publish_lat.trim().parse::<f64>(),
                    self.publish_lon.trim().parse::<f64>(),
                ) {
                    (Ok(lat), Ok(lon))
                        if (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lon) =>
                    {
                        self.spawn_action(
                            PendingAction::PublishLocation {
                                lat,
                                lon,
                                accuracy: 10.0,
                            },
                            ctx,
                        );
                    }
                    _ => {
                        self.app.set_error(Some(
                            "Enter a valid latitude (-90..90) and longitude (-180..180)"
                                .to_string(),
                        ));
                    }
                }
            }
        });

        ui.add_space(8.0);

        // Friendships with inline accept/reject for incoming requests
        ui.group(|ui| {
            ui.label(RichText::new("Friends").size(14.0).strong());
            ui.add_space(4.0);

            let binding = self.app.data();
            let data = binding.lock().unwrap();
            if data.friendships.is_empty() {
                ui.label("No friends yet");
            } else {
                for friendship in &data.friendships {
                    let color = match friendship.status.as_str() {
                        "pending" => Color32::YELLOW,
                        "accepted" => Color32::GREEN,
                        _ => Color32::GRAY,
                    };
                    let label = format!(
                        "{} → {}",
                        friendship.requester_name, friendship.receiver_name
                    );
                    let id = friendship.id;
                    let is_incoming = data
                        .user
                        .as_ref()
                        .map(|u| u.id == friendship.receiver_id)
                        .unwrap_or(false);
                    ui.horizontal(|ui| {
                        ui.colored_label(color, &label);
                        if friendship.status == "pending" && is_incoming {
                            if ui.button("Accept").clicked() {
                                self.spawn_action(
                                    PendingAction::RespondFriendship {
                                        id,
                                        action: "accept".to_string(),
                                    },
                                    ctx,
                                );
                            }
                            if ui.button("Reject").clicked() {
                                self.spawn_action(
                                    PendingAction::RespondFriendship {
                                        id,
                                        action: "reject".to_string(),
                                    },
                                    ctx,
                                );
                            }
                        } else if friendship.status == "pending" {
                            ui.small("awaiting response");
                        }
                    });
                }
            }
        });

        ui.add_space(8.0);

        // Location shares
        ui.group(|ui| {
            ui.label(RichText::new("Location Sharing").size(14.0).strong());
            ui.add_space(4.0);

            let binding = self.app.data();
            let data = binding.lock().unwrap();
            if data.shares.is_empty() {
                ui.label("No location sharing permissions");
            } else {
                for share in &data.shares {
                    let precision_color = match share.precision_level.as_str() {
                        "exact" => Color32::GREEN,
                        "approximate" => Color32::YELLOW,
                        "city_only" => Color32::RED,
                        _ => Color32::GRAY,
                    };
                    let is_active = share.is_active;
                    let share_id = share.id;
                    ui.horizontal(|ui| {
                        ui.colored_label(
                            precision_color,
                            format!("{} - {}", share.direction, share.precision_level),
                        );
                        let label = if is_active { "Pause" } else { "Resume" };
                        if ui.button(label).clicked() {
                            self.spawn_action(
                                PendingAction::ToggleShare {
                                    share_id,
                                    is_active: !is_active,
                                },
                                ctx,
                            );
                        }
                    });
                }
            }
        });

        ui.add_space(8.0);

        // Map visualization + location list
        let binding = self.app.data();
        let mut data = binding.lock().unwrap();
        map::render_map(ui, &mut data);

        ui.add_space(8.0);

        // Navigation to friend locations
        ui.group(|ui| {
            ui.label(RichText::new("Navigate to a friend").size(14.0).strong());
            ui.add_space(4.0);
            let locations: Vec<FriendLocation> = data.locations.clone();
            drop(data);
            if locations.is_empty() {
                ui.label("No locations to navigate to.");
            } else {
                for loc in &locations {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{} ({:.4}, {:.4})",
                            loc.user_name, loc.latitude, loc.longitude
                        ));
                        ui.add_space(4.0);
                        if ui.button("Navigate").clicked() {
                            let url = format!(
                                "https://www.openstreetmap.org/directions?to={:.4}%2C{:.4}",
                                loc.latitude, loc.longitude
                            );
                            let _ = webbrowser::open(&url);
                            self.app.set_notice(Some(format!(
                                "Opening OSM directions to {}",
                                loc.user_name
                            )));
                        }
                        if ui.button("OSM Map").clicked() {
                            let url = format!(
                                "https://www.openstreetmap.org/#map=14/{:.4}/{:.4}",
                                loc.latitude, loc.longitude
                            );
                            let _ = webbrowser::open(&url);
                        }
                    });
                }
            }
        });

        ui.add_space(8.0);

        // Search for places (Nominatim)
        ui.group(|ui| {
            ui.label(
                RichText::new("Search places (OpenStreetMap)")
                    .size(14.0)
                    .strong(),
            );
            ui.add_space(4.0);
            let binding = self.app.data();
            let data = binding.lock().unwrap();
            let results = data.search_results.clone();
            let error = data.search_error.clone();
            drop(data);
            ui.add(
                egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("Search for a place (e.g. café, pharmacy, Chicago)")
                    .desired_width(300.0),
            );
            ui.add_space(4.0);
            if ui.button("Search").clicked() && !self.search_query.trim().is_empty() {
                let query = self.search_query.clone();
                self.spawn_action(PendingAction::SearchPlaces { query }, ctx);
            }
            if !results.is_empty() {
                ui.add_space(4.0);
                ui.label(format!("Results ({}):", results.len()));
                for place in results {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{} ({:.4}, {:.4})",
                            place.display_name, place.latitude, place.longitude
                        ));
                        ui.add_space(4.0);
                        if ui.button("Navigate").clicked() {
                            let url = format!(
                                "https://www.openstreetmap.org/directions?to={:.4}%2C{:.4}",
                                place.latitude, place.longitude
                            );
                            let _ = webbrowser::open(&url);
                            self.app.set_notice(Some(format!(
                                "Opening OSM directions to {}",
                                place.display_name
                            )));
                        }
                        if ui.button("OSM Map").clicked() {
                            let url = format!(
                                "https://www.openstreetmap.org/#map=14/{:.4}/{:.4}",
                                place.latitude, place.longitude
                            );
                            let _ = webbrowser::open(&url);
                        }
                    });
                }
            }
            if let Some(err) = error {
                ui.colored_label(Color32::RED, err);
            }
        });
    }
}
