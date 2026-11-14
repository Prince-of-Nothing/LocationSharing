//! Auth screens (login / register)
use super::{PendingAction, PinpointAppUI};
use crate::app::CurrentScreen;
use eframe::egui::{self};

impl PinpointAppUI {
    pub fn draw_login_form(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let compact = Self::is_compact(ctx);
        ui.vertical_centered(|ui| {
            if !compact {
                ui.heading("Pinpoint");
                ui.label("Private location sharing for trusted friends");
            }
            ui.small(format!("Backend: {}", self.app.backend_url()));
            ui.add_space(if compact { 10.0 } else { 20.0 });

            ui.group(|ui| {
                ui.label("Sign in");
                ui.add_space(8.0);
                let field_width = if compact { 300.0 } else { 250.0 };
                ui.add(
                    egui::TextEdit::singleline(&mut self.login_email)
                        .hint_text("Email")
                        .desired_width(field_width),
                );
                ui.add_space(8.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.login_password)
                        .password(true)
                        .hint_text("Password")
                        .desired_width(field_width),
                );
                ui.add_space(8.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.login_totp)
                        .hint_text("Authenticator code (if MFA is on)")
                        .desired_width(field_width),
                );
                ui.add_space(16.0);
                if ui.button("Sign in").clicked()
                    && !self.login_email.is_empty()
                    && !self.login_password.is_empty()
                {
                    let email = self.login_email.clone();
                    let password = self.login_password.clone();
                    let totp = {
                        let code = self.login_totp.trim();
                        if code.is_empty() {
                            None
                        } else {
                            Some(code.to_string())
                        }
                    };
                    self.spawn_action(
                        PendingAction::Login {
                            email,
                            password,
                            totp,
                        },
                        ctx,
                    );
                }
                ui.add_space(8.0);
                if ui.button("Register").clicked() {
                    self.app.set_screen(CurrentScreen::Register);
                }
            });
        });
    }

    pub fn draw_register(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let compact = Self::is_compact(ctx);
        ui.vertical_centered(|ui| {
            if !compact {
                ui.heading("Create Account");
            }
            ui.add_space(if compact { 10.0 } else { 20.0 });

            ui.group(|ui| {
                ui.label("New to Pinpoint?");
                ui.add_space(10.0);
                let field_width = if compact { 300.0 } else { 200.0 };
                ui.add(
                    egui::TextEdit::singleline(&mut self.register_email)
                        .hint_text("Email")
                        .desired_width(field_width),
                );
                ui.add_space(5.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.register_password)
                        .password(true)
                        .hint_text("Password")
                        .desired_width(field_width),
                );
                ui.add_space(5.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.register_full_name)
                        .hint_text("Full Name")
                        .desired_width(field_width),
                );
                ui.add_space(10.0);

                ui.horizontal_wrapped(|ui| {
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
}
