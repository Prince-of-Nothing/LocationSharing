//! Admin + password screens.
use super::{PendingAction, PinpointAppUI};
use eframe::egui::{self, Direction, RichText};

impl PinpointAppUI {
    pub fn draw_admin_screen(&mut self, ui: &mut egui::Ui) {
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
