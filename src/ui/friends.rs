//! Friends screen
use super::{FriendshipRef, PendingAction, PinpointAppUI};
use eframe::egui::{self, Direction, RichText};

impl PinpointAppUI {
    pub fn draw_friends_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
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
                egui::TextEdit::singleline(&mut self.friend_handle)
                    .hint_text("Username (calm-otter-42) or email")
                    .desired_width(220.0),
            );
            ui.label("Usernames are exact-match; badges shown here are handles.");
            ui.add_space(8.0);
            if ui.button("Send Request").clicked() && !self.friend_handle.trim().is_empty() {
                self.spawn_action(
                    PendingAction::SendFriendRequest {
                        handle: self.friend_handle.clone(),
                    },
                    ctx,
                );
            }
        });
    }
}
