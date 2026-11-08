//! Trips and check-ins screen
use super::{PendingAction, PinpointAppUI};
use eframe::egui::{self, RichText};

impl PinpointAppUI {
    pub fn draw_trips_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.draw_main_toolbar(ui);
        ui.heading("Trips & check-ins");
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.label(RichText::new("New check-in").size(14.0).strong());
            ui.add(
                egui::TextEdit::singleline(&mut self.checkin_lat)
                    .hint_text("Latitude")
                    .desired_width(110.0),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.checkin_lon)
                    .hint_text("Longitude")
                    .desired_width(110.0),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.checkin_note)
                    .hint_text("Note")
                    .desired_width(220.0),
            );
            for k in ["manual", "scheduled", "trip"] {
                ui.selectable_value(&mut self.checkin_kind, k.to_string(), k);
            }
            if ui.button("Check in").clicked() {
                if let (Ok(lat), Ok(lon)) = (
                    self.checkin_lat.trim().parse::<f64>(),
                    self.checkin_lon.trim().parse::<f64>(),
                ) {
                    self.spawn_action(
                        PendingAction::CreateCheckIn {
                            lat,
                            lon,
                            note: self.checkin_note.clone(),
                            kind: self.checkin_kind.clone(),
                        },
                        ctx,
                    );
                } else {
                    self.app
                        .set_error(Some("Enter valid latitude/longitude".to_string()));
                }
            }
        });
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.label(RichText::new("Recent check-ins").size(14.0).strong());
            let data = self.app.data();
            let d = data.lock().unwrap();
            if d.check_ins.is_empty() {
                ui.label("No check-ins yet.");
            }
            for c in &d.check_ins {
                ui.label(format!(
                    "[{}] {} ({:.4}, {:.4}) — {} @ {}",
                    c.kind, c.user_name, c.latitude, c.longitude, c.note, c.created_at
                ));
            }
        });
    }
}
