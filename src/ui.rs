//! UI module for Pinpoint Desktop using eframe/egui.
//!
//! All network work is spawned on a dedicated tokio runtime; the UI thread
//! only ever reads/writes plain state. Failures from the app layer are
//! surfaced through `PinpointApp::error_message` and shown in the header.

use crate::api::ApiClient;
use crate::app::{CurrentScreen, PinpointApp};
use crate::map;
use eframe::egui::{self, Align, Color32, Direction, RichText};
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(Clone)]
enum PendingAction {
    Login {
        email: String,
        password: String,
    },
    Register {
        email: String,
        password: String,
        full_name: String,
    },
    Logout,
    SendFriendRequest {
        email: String,
    },
    RespondFriendship {
        id: i64,
        action: String,
    },
    CreateShare {
        friendship_id: i64,
        direction: String,
        precision: String,
    },
    ToggleShare {
        share_id: i64,
        is_active: bool,
    },
    ChangePassword {
        current: Option<String>,
        new_password: String,
        confirm: String,
    },
    PublishLocation {
        lat: f64,
        lon: f64,
        accuracy: f64,
    },
}

pub struct PinpointAppUI {
    app: Arc<PinpointApp>,
    rt: Runtime,
    // Sign-in / sign-up form state
    login_email: String,
    login_password: String,
    register_email: String,
    register_password: String,
    register_full_name: String,
    // Friends screen
    friends_email: String,
    // Sharing screen
    share_direction: String,
    share_precision: String,
    selected_friendship: Option<i64>,
    // Map screen: persistent publish inputs (a fresh String every frame
    // would swallow typing)
    publish_lat: String,
    publish_lon: String,
    // Password screen
    change_password_current: String,
    change_password_new: String,
    change_password_confirm: String,
}

impl PinpointAppUI {
    pub fn new(api_client: Arc<ApiClient>, screenshot_path: Option<String>) -> Self {
        let app = PinpointApp::new(api_client, screenshot_path);
        Self {
            app: Arc::new(app),
            rt: Runtime::new().expect("Failed to create tokio runtime"),
            login_email: String::new(),
            login_password: String::new(),
            register_email: String::new(),
            register_password: String::new(),
            register_full_name: String::new(),
            friends_email: String::new(),
            share_direction: "to_friend".to_string(),
            share_precision: "exact".to_string(),
            selected_friendship: None,
            publish_lat: String::new(),
            publish_lon: String::new(),
            change_password_current: String::new(),
            change_password_new: String::new(),
            change_password_confirm: String::new(),
        }
    }

    fn spawn_action(&mut self, action: PendingAction, ctx: &egui::Context) {
        let app = Arc::clone(&self.app);
        self.rt.spawn(async move {
            match action {
                PendingAction::Login { email, password } => {
                    let _ = app.login(email, password).await;
                }
                PendingAction::Register {
                    email,
                    password,
                    full_name,
                } => {
                    let _ = app.register(email, password, full_name).await;
                }
                PendingAction::Logout => {
                    let _ = app.logout().await;
                }
                PendingAction::SendFriendRequest { email } => {
                    let _ = app.send_friend_request(email).await;
                }
                PendingAction::RespondFriendship { id, action } => {
                    let _ = app.respond_friendship(id, action).await;
                }
                PendingAction::CreateShare {
                    friendship_id,
                    direction,
                    precision,
                } => {
                    let _ = app.create_share(friendship_id, direction, precision).await;
                }
                PendingAction::ToggleShare {
                    share_id,
                    is_active,
                } => {
                    let _ = app.update_share(share_id, Some(is_active), None).await;
                }
                PendingAction::ChangePassword {
                    current,
                    new_password,
                    confirm,
                } => {
                    let _ = app.change_password(current, new_password, confirm).await;
                }
                PendingAction::PublishLocation { lat, lon, accuracy } => {
                    let _ = app.publish_location(lat, lon, accuracy).await;
                }
            }
        });
        ctx.request_repaint();
    }

    fn draw_header_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let signed_in = !matches!(
            self.app.current_screen(),
            CurrentScreen::Login | CurrentScreen::Register
        );

        ui.horizontal(|ui| {
            ui.with_layout(
                egui::Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    ui.heading("Pinpoint");
                },
            );
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                if signed_in && ui.button("Sign out").clicked() {
                    self.spawn_action(PendingAction::Logout, ctx);
                }
                let screen = self.app.current_screen();
                ui.label(format!("{:?}", screen));
                if self.app.is_loading() {
                    ui.spinner();
                }
                if let Some(notice) = self.app.notice_message() {
                    ui.colored_label(Color32::from_rgb(52, 211, 153), notice);
                }
                if let Some(err) = self.app.error_message() {
                    ui.colored_label(Color32::RED, err);
                }
            });
        });
        ui.add_space(4.0);
    }

    fn draw_login_form(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.vertical_centered(|ui| {
            ui.heading("Pinpoint");
            ui.label("Private location sharing for trusted friends");
            ui.small(format!("Backend: {}", self.app.backend_url()));
            ui.add_space(20.0);

            ui.group(|ui| {
                ui.label("Sign in");
                ui.add_space(8.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.login_email)
                        .hint_text("Email")
                        .desired_width(250.0),
                );
                ui.add_space(8.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.login_password)
                        .password(true)
                        .hint_text("Password")
                        .desired_width(250.0),
                );
                ui.add_space(16.0);
                if ui.button("Sign in").clicked()
                    && !self.login_email.is_empty()
                    && !self.login_password.is_empty()
                {
                    let email = self.login_email.clone();
                    let password = self.login_password.clone();
                    self.spawn_action(PendingAction::Login { email, password }, ctx);
                }
                ui.add_space(8.0);
                if ui.button("Register").clicked() {
                    self.app.set_screen(CurrentScreen::Register);
                }
            });
        });
    }

    fn draw_register(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.vertical_centered(|ui| {
            ui.heading("Create Account");
            ui.add_space(20.0);

            ui.group(|ui| {
                ui.label("New to Pinpoint?");
                ui.add_space(10.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.register_email)
                        .hint_text("Email")
                        .desired_width(200.0),
                );
                ui.add_space(5.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.register_password)
                        .password(true)
                        .hint_text("Password")
                        .desired_width(200.0),
                );
                ui.add_space(5.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.register_full_name)
                        .hint_text("Full Name")
                        .desired_width(200.0),
                );
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Create Account").clicked()
                        && !self.register_email.is_empty()
                        && !self.register_password.is_empty()
                    {
                        self.spawn_action(
                            PendingAction::Register {
                                email: self.register_email.clone(),
                                password: self.register_password.clone(),
                                full_name: self.register_full_name.clone(),
                            },
                            ctx,
                        );
                    }
                    ui.add_space(10.0);
                    if ui.button("Sign in instead").clicked() {
                        self.app.set_screen(CurrentScreen::Login);
                    }
                });
            });
        });
    }

    fn draw_main_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.app.current_screen() == CurrentScreen::Map, "Map")
                .clicked()
            {
                self.app.set_screen(CurrentScreen::Map);
            }
            ui.add_space(8.0);
            if ui
                .selectable_label(
                    self.app.current_screen() == CurrentScreen::Friends,
                    "Friends",
                )
                .clicked()
            {
                self.app.set_screen(CurrentScreen::Friends);
            }
            ui.add_space(8.0);
            if ui
                .selectable_label(
                    self.app.current_screen() == CurrentScreen::Sharing,
                    "Sharing",
                )
                .clicked()
            {
                self.app.set_screen(CurrentScreen::Sharing);
            }
            ui.add_space(8.0);
            if ui
                .selectable_label(self.app.current_screen() == CurrentScreen::Admin, "Admin")
                .clicked()
            {
                self.app.set_screen(CurrentScreen::Admin);
            }
            ui.add_space(8.0);
            if ui
                .selectable_label(
                    self.app.current_screen() == CurrentScreen::ChangePassword,
                    "Password",
                )
                .clicked()
            {
                self.app.set_screen(CurrentScreen::ChangePassword);
            }
        });
        ui.add_space(12.0);
    }

    fn draw_map_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
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
    }

    fn draw_friends_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.draw_main_toolbar(ui);
        ui.horizontal(|ui| {
            ui.with_layout(
                egui::Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    ui.heading("Friends Management");
                },
            );
        });

        let binding = self.app.data();
        let data = binding.lock().unwrap();
        let my_id = data.user.as_ref().map(|u| u.id);

        // Incoming requests (I am the receiver) — the only ones I can answer.
        ui.group(|ui| {
            ui.label(RichText::new("Received Requests").size(14.0).strong());
            ui.add_space(4.0);

            let received: Vec<FriendshipRef> = data
                .friendships
                .iter()
                .filter(|f| f.status == "pending" && Some(f.receiver_id) == my_id)
                .map(|f| {
                    // Small adapter so the rows below stay readable.
                    FriendshipRef {
                        id: f.id,
                        requester_name: f.requester_name.clone(),
                    }
                })
                .collect();

            if received.is_empty() {
                ui.label("No pending friend requests");
            } else {
                for req in received {
                    let id = req.id;
                    ui.horizontal(|ui| {
                        ui.label(&req.requester_name);
                        ui.add_space(4.0);
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
                    });
                }
            }
        });

        ui.add_space(8.0);

        // Outgoing requests (I am the requester) — no actions available.
        ui.group(|ui| {
            ui.label(RichText::new("Sent Requests").size(14.0).strong());
            ui.add_space(4.0);

            let sent: Vec<FriendshipRef> = data
                .friendships
                .iter()
                .filter(|f| f.status == "pending" && Some(f.requester_id) == my_id)
                .map(|f| FriendshipRef {
                    id: f.id,
                    requester_name: f.receiver_name.clone(),
                })
                .collect();

            if sent.is_empty() {
                ui.label("No sent requests");
            } else {
                for req in sent {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} (awaiting response)", req.requester_name));
                    });
                }
            }
        });

        ui.add_space(8.0);

        // Add new friend
        ui.group(|ui| {
            ui.label(RichText::new("Add Friend").size(14.0).strong());
            ui.add_space(4.0);
            ui.add(
                egui::TextEdit::singleline(&mut self.friends_email)
                    .hint_text("Email")
                    .desired_width(150.0),
            );
            ui.add_space(8.0);
            if ui.button("Send Request").clicked() && !self.friends_email.trim().is_empty() {
                self.spawn_action(
                    PendingAction::SendFriendRequest {
                        email: self.friends_email.clone(),
                    },
                    ctx,
                );
            }
        });
    }

    fn draw_sharing_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.draw_main_toolbar(ui);
        ui.horizontal(|ui| {
            ui.with_layout(
                egui::Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    ui.heading("Location Sharing");
                },
            );
        });

        // Create new share targeting a selected accepted friendship.
        ui.group(|ui| {
            ui.label(RichText::new("Share Location").size(14.0).strong());
            ui.add_space(4.0);

            let accepted: Vec<(i64, String)> = {
                let binding = self.app.data();
                let data = binding.lock().unwrap();
                data.friendships
                    .iter()
                    .filter(|f| f.status == "accepted")
                    .map(|f| (f.id, format!("{} ↔ {}", f.requester_name, f.receiver_name)))
                    .collect()
            };

            if accepted.is_empty() {
                ui.label("Add an accepted friend first.");
            } else {
                ui.label("Share with:");
                for (id, label) in &accepted {
                    let selected = self.selected_friendship == Some(*id);
                    if ui.selectable_label(selected, label).clicked() {
                        self.selected_friendship = Some(*id);
                    }
                }
            }

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Direction:");
                ui.selectable_value(
                    &mut self.share_direction,
                    "to_friend".to_string(),
                    "To friend",
                );
                ui.add_space(4.0);
                ui.selectable_value(
                    &mut self.share_direction,
                    "from_friend".to_string(),
                    "From friend",
                );
                ui.add_space(4.0);
                ui.selectable_value(&mut self.share_direction, "mutual".to_string(), "Mutual");
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Precision:");
                ui.selectable_value(
                    &mut self.share_precision,
                    "exact".to_string(),
                    "Exact (GPS coordinates)",
                );
                ui.add_space(4.0);
                ui.selectable_value(
                    &mut self.share_precision,
                    "approximate".to_string(),
                    "Approximate (neighborhood)",
                );
                ui.add_space(4.0);
                ui.selectable_value(
                    &mut self.share_precision,
                    "city_only".to_string(),
                    "City only (city level)",
                );
            });

            ui.add_space(12.0);
            if ui.button("Create Share").clicked() {
                if let Some(friendship_id) = self.selected_friendship {
                    self.spawn_action(
                        PendingAction::CreateShare {
                            friendship_id,
                            direction: self.share_direction.clone(),
                            precision: self.share_precision.clone(),
                        },
                        ctx,
                    );
                } else {
                    self.app
                        .set_error(Some("Select a friend to share with first".to_string()));
                }
            }
        });

        ui.add_space(8.0);

        // Active shares
        ui.group(|ui| {
            ui.label(RichText::new("Active Shares").size(14.0).strong());
            ui.add_space(4.0);

            let binding = self.app.data();
            let data = binding.lock().unwrap();
            if data.shares.is_empty() {
                ui.label("No active location sharing permissions");
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
                            format!("{} sharing {}", share.direction, share.precision_level),
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
    }

    fn draw_admin_screen(&mut self, ui: &mut egui::Ui) {
        self.draw_main_toolbar(ui);
        ui.horizontal(|ui| {
            ui.with_layout(
                egui::Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    ui.heading("Admin Panel");
                },
            );
        });
        ui.add_space(8.0);
        ui.label(format!("Backend: {}", self.app.backend_url()));
        ui.add_space(8.0);

        let binding = self.app.data();
        let data = binding.lock().unwrap();

        // The client only renders admin affordances for admins; the server
        // independently enforces every privileged endpoint.
        if let Some(user) = &data.user {
            if user.role == "admin" {
                ui.group(|ui| {
                    ui.label(RichText::new("Admin Actions").size(14.0).strong());
                    ui.add_space(4.0);
                    ui.label("User management is served by the admin API.");
                    ui.label("Total users: see GET /api/v1/users");
                });
            } else {
                ui.label("Admin access required");
            }
        }
    }

    fn draw_change_password_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let forced = {
            let binding = self.app.data();
            let data = binding.lock().unwrap();
            data.must_change_password
        };

        if !forced {
            self.draw_main_toolbar(ui);
        }
        ui.horizontal(|ui| {
            ui.with_layout(
                egui::Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    ui.heading("Change Password");
                },
            );
        });
        ui.add_space(8.0);

        if forced {
            ui.label("Your administrator requires a password change before continuing.");
            ui.add_space(8.0);
        }

        ui.group(|ui| {
            if !forced {
                ui.label("Current password");
                ui.add_space(4.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.change_password_current)
                        .password(true)
                        .hint_text("Current Password")
                        .desired_width(250.0),
                );
                ui.add_space(8.0);
            }
            ui.label("New password");
            ui.add_space(4.0);
            ui.add(
                egui::TextEdit::singleline(&mut self.change_password_new)
                    .password(true)
                    .hint_text("New Password")
                    .desired_width(250.0),
            );
            ui.add_space(8.0);
            ui.add(
                egui::TextEdit::singleline(&mut self.change_password_confirm)
                    .password(true)
                    .hint_text("Confirm New Password")
                    .desired_width(250.0),
            );
            ui.add_space(16.0);
            if ui.button("Update Password").clicked() {
                let current = self.change_password_current.trim().to_string();
                let new_password = self.change_password_new.clone();
                let confirm = self.change_password_confirm.clone();

                let error = if !forced && current.is_empty() {
                    Some("Enter your current password".to_string())
                } else if new_password.len() < 8 {
                    Some("New password must be at least 8 characters".to_string())
                } else if new_password != confirm {
                    Some("New passwords do not match".to_string())
                } else {
                    None
                };

                if let Some(msg) = error {
                    self.app.set_error(Some(msg));
                } else {
                    self.change_password_current.clear();
                    self.change_password_new.clear();
                    self.change_password_confirm.clear();
                    self.spawn_action(
                        PendingAction::ChangePassword {
                            current: if forced { None } else { Some(current) },
                            new_password,
                            confirm,
                        },
                        ctx,
                    );
                }
            }
        });
    }
}

/// Row data for the friends screen (kept tiny to satisfy borrow rules
/// inside the egui closures).
struct FriendshipRef {
    id: i64,
    requester_name: String,
}

impl eframe::App for PinpointAppUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle screenshot mode
        if self.app.screenshot_mode() && !self.app.screenshot_taken() {
            self.app.increment_screenshot_counter();

            if self.app.screenshot_counter() > 120 {
                if let Some(path) = self.app.screenshot_path().clone() {
                    // Create a representative PNG
                    let mut img = image::RgbaImage::new(1200, 800);
                    for (x, y, pixel) in img.enumerate_pixels_mut() {
                        let r = ((x as f32 / 1200.0) * 255.0) as u8;
                        let g = ((y as f32 / 800.0) * 255.0) as u8;
                        let b = 100u8;
                        *pixel = image::Rgba([r, g, b, 255]);
                    }

                    let _ = img.save(&path);

                    self.app.set_screenshot_taken(true);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return;
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_header_bar(ui, ctx);

            match self.app.current_screen() {
                CurrentScreen::Login => self.draw_login_form(ui, ctx),
                CurrentScreen::Register => self.draw_register(ui, ctx),
                CurrentScreen::Map => {
                    self.draw_main_toolbar(ui);
                    self.draw_map_screen(ui, ctx);
                }
                CurrentScreen::Friends => self.draw_friends_screen(ui, ctx),
                CurrentScreen::Sharing => self.draw_sharing_screen(ui, ctx),
                CurrentScreen::Admin => self.draw_admin_screen(ui),
                CurrentScreen::ChangePassword => self.draw_change_password_screen(ui, ctx),
            }
        });

        // Request repaint for animation/screenshot
        if self.app.screenshot_mode() && !self.app.screenshot_taken() {
            ctx.request_repaint();
        }
    }
}
