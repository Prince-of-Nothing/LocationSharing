//! Admin + password screens.
use super::{PendingAction, PinpointAppUI};
use eframe::egui::{self, Direction, RichText};

impl PinpointAppUI {
    pub fn draw_admin_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.draw_main_toolbar(ui);
        ui.horizontal(|ui| {
            ui.with_layout(
                egui::Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    ui.heading("Admin Console");
                },
            );
        });
        ui.add_space(8.0);
        ui.label(format!("Backend: {}", self.app.backend_url()));
        ui.add_space(8.0);

        let binding = self.app.data();
        let data = binding.lock().unwrap();

        // Server-side authorization is authoritative; UI checks are fail-closed hints.
        let is_admin = data
            .user
            .as_ref()
            .map(|u| u.role.eq_ignore_ascii_case("admin") && u.is_active)
            .unwrap_or(false);

        if !is_admin {
            ui.group(|ui| {
                ui.colored_label(
                    egui::Color32::RED,
                    "Access denied: Admin role required (server-side authorization is authoritative).",
                );
            });
            return;
        }

        ui.group(|ui| {
            ui.label(RichText::new("User Management").size(14.0).strong());
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.admin_search_query)
                        .hint_text("Search users by name/email/username")
                        .desired_width(280.0),
                );
                if ui.button("Search Users").clicked() {
                    let query = self.admin_search_query.clone();
                    self.spawn_action(PendingAction::AdminLoadUsers { query }, ctx);
                }
                if ui.button("Refresh List").clicked() {
                    self.admin_search_query.clear();
                    self.spawn_action(
                        PendingAction::AdminLoadUsers {
                            query: String::new(),
                        },
                        ctx,
                    );
                }
            });
        });

        ui.add_space(12.0);

        let users = data.users.clone();
        drop(data);

        ui.group(|ui| {
            ui.label(
                RichText::new(format!("User Accounts ({})", users.len()))
                    .size(14.0)
                    .strong(),
            );
            ui.add_space(8.0);

            if users.is_empty() {
                ui.label(
                    "No users loaded. Click 'Search Users' or 'Refresh List' to retrieve records.",
                );
            } else {
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for user in users {
                            ui.horizontal(|ui| {
                                ui.label(format!("#{} {}", user.id, user.username));
                                ui.label(format!("({})", user.email));
                                ui.label(format!("Role: {}", user.role));
                                let status_text =
                                    if user.is_active { "Active" } else { "Disabled" };
                                let color = if user.is_active {
                                    egui::Color32::from_rgb(52, 211, 153)
                                } else {
                                    egui::Color32::RED
                                };
                                ui.colored_label(color, status_text);

                                if user.is_active {
                                    if ui.button("Disable").clicked() {
                                        self.spawn_action(
                                            PendingAction::AdminSetUserActive {
                                                user_id: user.id,
                                                is_active: false,
                                            },
                                            ctx,
                                        );
                                    }
                                } else {
                                    if ui.button("Enable").clicked() {
                                        self.spawn_action(
                                            PendingAction::AdminSetUserActive {
                                                user_id: user.id,
                                                is_active: true,
                                            },
                                            ctx,
                                        );
                                    }
                                }
                            });
                            ui.separator();
                        }
                    });
            }
        });
    }

    pub fn draw_change_password_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
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
