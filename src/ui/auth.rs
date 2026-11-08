//! Auth screens (login / register)
use super::{PendingAction, PinpointAppUI};
use crate::app::CurrentScreen;
use eframe::egui::{self};

impl PinpointAppUI {
    pub fn draw_login_form(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
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

    pub fn draw_register(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
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
}
