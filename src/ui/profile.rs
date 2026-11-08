//! Profile and settings screen (incl. MFA)
use super::{PendingAction, PinpointAppUI};
use eframe::egui::{self, RichText};
use std::sync::Arc;

impl PinpointAppUI {
    pub fn draw_profile_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.draw_main_toolbar(ui);
        ui.heading("Profile & settings");
        ui.add_space(8.0);
        let binding = self.app.data();
        let data = binding.lock().unwrap();
        let (email, name) = match &data.user {
            Some(u) => (u.email.clone(), u.full_name.clone()),
            None => ("—".to_string(), String::new()),
        };
        let metric = data.settings.metric;
        drop(data);
        ui.label(format!("Signed in as: {email}"));
        ui.group(|ui| {
            ui.label(RichText::new("Display name").size(14.0).strong());
            ui.add(
                egui::TextEdit::singleline(&mut self.profile_name)
                    .hint_text(&name)
                    .desired_width(220.0),
            );
            if ui.button("Save name").clicked() && !self.profile_name.trim().is_empty() {
                self.spawn_action(
                    PendingAction::UpdateProfile {
                        full_name: self.profile_name.clone(),
                    },
                    ctx,
                );
            }
        });
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.label(RichText::new("Units").size(14.0).strong());
            ui.selectable_value(
                &mut self.app.data().lock().unwrap().settings.metric,
                true,
                "Metric (km/m)",
            );
            ui.selectable_value(
                &mut self.app.data().lock().unwrap().settings.metric,
                false,
                "Imperial (mi/ft)",
            );
            let _ = metric;
        });
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.label(RichText::new("Safety").size(14.0).strong());
            ui.label("Share expiry, pause, and delete live under Sharing. Emergency alerts under Emergency.");
            ui.label("Location is latest-only; no history is stored.");
        });
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.label(
                RichText::new("Two-factor authentication (TOTP)")
                    .size(14.0)
                    .strong(),
            );
            let (enabled, pending) = {
                let d = self.app.data();
                let d = d.lock().unwrap();
                (d.mfa_enabled, d.totp_pending_secret.clone())
            };
            if enabled {
                ui.label("MFA is ON for this account.");
                if ui.button("Disable MFA").clicked() {
                    let app = Arc::clone(&self.app);
                    let rt_ctx = ctx.clone();
                    self.rt.spawn(async move {
                        let _ = app.mfa_disable().await;
                    });
                    let _ = rt_ctx;
                }
            } else if let Some(secret) = pending {
                ui.label(
                    "Scan this secret in your authenticator app, then enter the 6-digit code:",
                );
                ui.monospace(&secret);
                ui.add(
                    egui::TextEdit::singleline(&mut self.mfa_code)
                        .hint_text("6-digit code")
                        .desired_width(140.0),
                );
                if ui.button("Verify & enable").clicked() && !self.mfa_code.trim().is_empty() {
                    let code = self.mfa_code.clone();
                    let app = Arc::clone(&self.app);
                    self.rt.spawn(async move {
                        let _ = app.mfa_verify(code).await;
                    });
                }
            } else {
                ui.label("MFA is OFF. Enrol to require an authenticator code at sign-in.");
                if ui.button("Enrol MFA").clicked() {
                    let app = Arc::clone(&self.app);
                    self.rt.spawn(async move {
                        let _ = app.mfa_enroll().await;
                    });
                }
            }
        });
    }
}
