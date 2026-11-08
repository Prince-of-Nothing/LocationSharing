//! Emergency screen
use super::{PendingAction, PinpointAppUI};
use eframe::egui::{self, Color32, RichText};

impl PinpointAppUI {
    pub fn draw_emergency_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.draw_main_toolbar(ui);
        ui.heading("Emergency");
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.colored_label(
                Color32::RED,
                RichText::new("PANIC — notifies all accepted friends & activates all shares")
                    .size(13.0),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.emergency_lat)
                    .hint_text("Latitude")
                    .desired_width(110.0),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.emergency_lon)
                    .hint_text("Longitude")
                    .desired_width(110.0),
            );
            if ui
                .add(egui::Button::new(
                    RichText::new("SEND EMERGENCY ALERT").color(Color32::WHITE),
                ))
                .clicked()
            {
                if let (Ok(lat), Ok(lon)) = (
                    self.emergency_lat.trim().parse::<f64>(),
                    self.emergency_lon.trim().parse::<f64>(),
                ) {
                    self.spawn_action(PendingAction::TriggerEmergency { lat, lon }, ctx);
                } else {
                    self.app
                        .set_error(Some("Enter valid latitude/longitude".to_string()));
                }
            }
        });
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.label(RichText::new("Alerts").size(14.0).strong());
            let data = self.app.data();
            let d = data.lock().unwrap();
            if d.emergencies.is_empty() {
                ui.label("No alerts.");
            }
            for e in &d.emergencies {
                ui.label(format!(
                    "{} ({:.4}, {:.4}) @ {} resolved={}",
                    e.user_name, e.latitude, e.longitude, e.created_at, e.resolved
                ));
            }
        });
    }
}
