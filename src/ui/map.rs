//! Map screen
use super::{PendingAction, PinpointAppUI};
use crate::i18n::{self, key};
use crate::map;
use crate::models::FriendLocation;
use eframe::egui::{self, Color32, Direction, RichText};

impl PinpointAppUI {
    pub fn draw_map_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if Self::is_compact(ctx) {
            self.draw_mobile_map_screen(ui, ctx);
            return;
        }
        let lang = self.lang;
        ui.horizontal(|ui| {
            ui.with_layout(
                egui::Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    ui.heading(i18n::t(lang, key::FRIEND_LOCATIONS));
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
            ui.label(
                RichText::new(i18n::t(lang, key::PUBLISH_MY_LOCATION))
                    .size(12.0)
                    .strong(),
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.publish_lat)
                        .hint_text(i18n::t(lang, key::LATITUDE))
                        .desired_width(100.0),
                );
                ui.add_space(8.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.publish_lon)
                        .hint_text(i18n::t(lang, key::LONGITUDE))
                        .desired_width(100.0),
                );
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button(i18n::t(lang, key::PUBLISH)).clicked() {
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
                if ui.button("📍 Use GPS").clicked() {
                    self.spawn_action(PendingAction::PublishGpsLocation, ctx);
                }
            });
        });

        ui.add_space(8.0);

        // Friendships with inline accept/reject for incoming requests
        ui.group(|ui| {
            ui.label(
                RichText::new(i18n::t(lang, key::FRIENDS))
                    .size(14.0)
                    .strong(),
            );
            ui.add_space(4.0);

            let binding = self.app.data();
            let data = binding.lock().unwrap();
            if data.friendships.is_empty() {
                ui.label(i18n::t(lang, key::NO_FRIENDS));
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
                            if ui.button(i18n::t(lang, key::ACCEPT)).clicked() {
                                self.spawn_action(
                                    PendingAction::RespondFriendship {
                                        id,
                                        action: "accept".to_string(),
                                    },
                                    ctx,
                                );
                            }
                            if ui.button(i18n::t(lang, key::REJECT)).clicked() {
                                self.spawn_action(
                                    PendingAction::RespondFriendship {
                                        id,
                                        action: "reject".to_string(),
                                    },
                                    ctx,
                                );
                            }
                        } else if friendship.status == "pending" {
                            ui.small(i18n::t(lang, key::AWAITING_RESPONSE));
                        }
                    });
                }
            }
        });

        ui.add_space(8.0);

        // Location shares
        ui.group(|ui| {
            ui.label(
                RichText::new(i18n::t(lang, key::LOCATION_SHARING))
                    .size(14.0)
                    .strong(),
            );
            ui.add_space(4.0);

            let binding = self.app.data();
            let data = binding.lock().unwrap();
            if data.shares.is_empty() {
                ui.label(i18n::t(lang, key::NO_SHARING));
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
                        let label = if is_active {
                            i18n::t(lang, key::PAUSE)
                        } else {
                            i18n::t(lang, key::RESUME)
                        };
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
        let backend_url = self.app.backend_url();
        let online = !self.app.is_offline_mode();
        let binding = self.app.data();
        let mut data = binding.lock().unwrap();
        map::render_map_with(ui, &mut data, 320.0, Some(&backend_url), online);
        let (center_lat, center_lon, zoom) =
            (data.map_center_lat, data.map_center_lon, data.map_zoom);
        let locations: Vec<FriendLocation> = data.locations.clone();
        drop(data);

        ui.add_space(6.0);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                let bytes = crate::tilepacks::packs_size();
                ui.label(
                    RichText::new(format!(
                        "Offline map packs: {:.1} KB cached",
                        bytes as f64 / 1024.0
                    ))
                    .size(12.0),
                );
                if ui.button("⬇ Download current area pack").clicked() {
                    self.spawn_action(
                        PendingAction::DownloadTilePack {
                            center_lat,
                            center_lon,
                            zoom,
                        },
                        ctx,
                    );
                }
            });
        });

        ui.add_space(8.0);

        // Navigation to friend locations
        ui.group(|ui| {
            ui.label(
                RichText::new(i18n::t(lang, key::NAVIGATE_TO_FRIEND))
                    .size(14.0)
                    .strong(),
            );
            ui.add_space(4.0);
            if locations.is_empty() {
                ui.label(i18n::t(lang, key::NO_LOCATIONS));
            } else {
                for loc in &locations {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{} ({:.4}, {:.4})",
                            loc.user_name, loc.latitude, loc.longitude
                        ));
                        ui.add_space(4.0);
                        if ui.button(i18n::t(lang, key::NAVIGATE)).clicked() {
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
                        if ui.button(i18n::t(lang, key::OSM_MAP)).clicked() {
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
                RichText::new(i18n::t(lang, key::SEARCH_PLACES))
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
                    .hint_text(i18n::t(lang, key::SEARCH_PLACES))
                    .desired_width(300.0),
            );
            ui.add_space(4.0);
            if ui.button(i18n::t(lang, key::SEARCH)).clicked()
                && !self.search_query.trim().is_empty()
            {
                let query = self.search_query.clone();
                self.spawn_action(PendingAction::SearchPlaces { query }, ctx);
            }
            if !results.is_empty() {
                ui.add_space(4.0);
                ui.label(format!(
                    "{} ({}):",
                    i18n::t(lang, key::RESULTS),
                    results.len()
                ));
                for place in results {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{} ({:.4}, {:.4})",
                            place.display_name, place.latitude, place.longitude
                        ));
                        ui.add_space(4.0);
                        if ui.small_button("★ Save").clicked() {
                            let store = crate::bookmarks::BookmarkStore::default_location();
                            store.add(&place.short_name, place.latitude, place.longitude);
                            self.app
                                .set_notice(Some(format!("Saved {}", place.short_name)));
                        }
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
                        if ui.button(i18n::t(lang, key::OSM_MAP)).clicked() {
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

impl PinpointAppUI {
    /// Map-first touch layout: full-width map, compact actions, publish sheet.
    fn draw_mobile_map_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let signed_in = self.app.is_signed_in();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(if signed_in { "Live map" } else { "Demo map" })
                    .size(16.0)
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("📍 Locate").clicked() {
                    self.spawn_action(PendingAction::PublishGpsLocation, ctx);
                }
            });
        });
        ui.add_space(4.0);

        let height = (ui.available_height() - 150.0).clamp(260.0, 620.0);
        let backend_url = self.app.backend_url();
        let online = !self.app.is_offline_mode();
        {
            let binding = self.app.data();
            let mut data = binding.lock().unwrap();
            crate::map::render_map_with(ui, &mut data, height, Some(&backend_url), online);
        }
        ui.add_space(6.0);

        ui.horizontal_wrapped(|ui| {
            if ui.button("Publish").clicked() {
                self.mobile_publish_open = !self.mobile_publish_open;
            }
            if ui.button(i18n::t(self.lang, key::FRIENDS)).clicked() {
                self.app.set_screen(crate::app::CurrentScreen::Friends);
            }
            if ui.button(i18n::t(self.lang, key::SHARING)).clicked() {
                self.app.set_screen(crate::app::CurrentScreen::Sharing);
            }
            if ui.button(i18n::t(self.lang, key::PLACES)).clicked() {
                self.app.set_screen(crate::app::CurrentScreen::Places);
            }
            if ui.button(i18n::t(self.lang, key::SAVED)).clicked() {
                self.app.set_screen(crate::app::CurrentScreen::Bookmarks);
            }
        });

        if self.mobile_publish_open {
            ui.add_space(6.0);
            ui.group(|ui| {
                ui.label(
                    RichText::new(i18n::t(self.lang, key::PUBLISH_MY_LOCATION))
                        .size(14.0)
                        .strong(),
                );
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.publish_lat)
                            .hint_text(i18n::t(self.lang, key::LATITUDE))
                            .desired_width(110.0),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut self.publish_lon)
                            .hint_text(i18n::t(self.lang, key::LONGITUDE))
                            .desired_width(110.0),
                    );
                });
                ui.horizontal(|ui| {
                    if ui.button(i18n::t(self.lang, key::PUBLISH)).clicked() {
                        match (
                            self.publish_lat.trim().parse::<f64>(),
                            self.publish_lon.trim().parse::<f64>(),
                        ) {
                            (Ok(lat), Ok(lon))
                                if (-90.0..=90.0).contains(&lat)
                                    && (-180.0..=180.0).contains(&lon) =>
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
                            _ => self.app.set_error(Some(
                                "Enter a valid latitude (-90..90) and longitude (-180..180)"
                                    .to_string(),
                            )),
                        }
                    }
                    if ui.button("📍 Use GPS").clicked() {
                        self.spawn_action(PendingAction::PublishGpsLocation, ctx);
                    }
                });
            });
        }
    }
}
