//! UI module for Pinpoint Desktop using eframe/egui.
//!
//! All network work is spawned on a dedicated tokio runtime; the UI thread
//! only ever reads/writes plain state. Failures from the app layer are
//! surfaced through `PinpointApp::error_message` and shown in the header.
//!
//! Per-screen rendering lives in `ui/` submodule files (one per screen);
//! this file keeps the app shell, pending actions, header/toolbar and the
//! `eframe::App` dispatcher.

pub(crate) mod admin;
pub(crate) mod auth;
pub(crate) mod bookmarks;
pub(crate) mod emergency;
pub(crate) mod friends;
pub(crate) mod kiosk;
pub(crate) mod map;
pub(crate) mod places;
pub(crate) mod profile;
pub(crate) mod sharing;
pub(crate) mod trips;

use crate::app::{CurrentScreen, PinpointApp};
use crate::backend::DynBackend;
use crate::i18n::{self, key, Lang};
use crate::osm::OsmClient;
use eframe::egui::{self, Align, Color32, Direction, RichText};
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(Clone)]
#[allow(dead_code)]
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
        handle: String,
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
    SearchPlaces {
        query: String,
    },
    DeleteShare {
        share_id: i64,
    },
    CreateShareEx {
        friendship_id: i64,
        direction: String,
        precision: String,
        duration_hours: Option<i64>,
    },
    DeleteFriendship {
        id: i64,
    },
    CreateCheckIn {
        lat: f64,
        lon: f64,
        note: String,
        kind: String,
    },
    TriggerEmergency {
        lat: f64,
        lon: f64,
    },
    Route {
        from: (f64, f64),
        to: (f64, f64),
        mode: String,
    },
    Poi {
        lat: f64,
        lon: f64,
        category: String,
    },
    Reverse {
        lat: f64,
        lon: f64,
    },
    UpdateProfile {
        full_name: String,
    },
    PublishGpsLocation,
    ExportGpx {
        name: String,
    },
    DownloadTilePack {
        center_lat: f64,
        center_lon: f64,
        zoom: u8,
    },
    RefreshAll,
    KioskResolveToken {
        token: String,
    },
    KioskSubmitCheckIn {
        request: crate::webapp::kiosk::KioskCheckInRequest,
    },
    AdminLoadUsers {
        query: String,
    },
    AdminSetUserActive {
        user_id: i64,
        is_active: bool,
    },
}

pub struct PinpointAppUI {
    app: Arc<PinpointApp>,
    rt: Runtime,
    /// Current UI language (auto-detected from the OS locale; overridable).
    lang: Lang,
    // Sign-in / sign-up form state
    login_email: String,
    login_password: String,
    register_email: String,
    register_password: String,
    register_full_name: String,
    // Friends screen
    friend_handle: String,
    // Sharing screen
    share_direction: String,
    share_precision: String,
    share_duration: String,
    selected_friendship: Option<i64>,
    // Map screen: persistent publish inputs (a fresh String every frame
    // would swallow typing)
    publish_lat: String,
    publish_lon: String,
    // Search query input (results live in AppData)
    search_query: String,
    // Places screen
    route_mode: String,
    route_from_lat: String,
    route_from_lon: String,
    poi_category: String,
    reverse_label: String,
    #[allow(dead_code)]
    route_label: String,
    // Trips / check-ins
    checkin_lat: String,
    checkin_lon: String,
    checkin_note: String,
    checkin_kind: String,
    // Bookmarks
    bookmark_name: String,
    bookmark_lat: String,
    bookmark_lon: String,
    // Profile
    profile_name: String,
    // MFA
    mfa_code: String,
    // Emergency
    emergency_lat: String,
    emergency_lon: String,
    // Password screen
    change_password_current: String,
    change_password_new: String,
    change_password_confirm: String,
    // Admin screen query input
    pub(crate) admin_search_query: String,
    // Kiosk screen state
    pub(crate) kiosk_token_input: String,
    pub(crate) kiosk_note_input: String,
    pub(crate) kiosk_consent_accepted: bool,
    // Mobile shell
    mobile_nav_open: bool,
    mobile_publish_open: bool,
}

impl PinpointAppUI {
    /// Create the UI with a modular backend (desktop/mobile). Pass any
    /// `DynBackend` to adapt to changed backend architectures.
    pub fn new(backend: DynBackend, screenshot_path: Option<String>) -> Self {
        let app = Arc::new(PinpointApp::new(backend, screenshot_path));
        PinpointApp::start_auto_refresh(Arc::clone(&app));
        Self {
            app,
            rt: Runtime::new().expect("Failed to create tokio runtime"),
            lang: Lang::system(),
            login_email: String::new(),
            login_password: String::new(),
            register_email: String::new(),
            register_password: String::new(),
            register_full_name: String::new(),
            friend_handle: String::new(),
            share_direction: "to_friend".to_string(),
            share_precision: "exact".to_string(),
            share_duration: String::new(),
            selected_friendship: None,
            publish_lat: String::new(),
            publish_lon: String::new(),
            search_query: String::new(),
            route_mode: "foot".to_string(),
            route_from_lat: String::new(),
            route_from_lon: String::new(),
            poi_category: "food".to_string(),
            reverse_label: String::new(),
            route_label: String::new(),
            checkin_lat: String::new(),
            checkin_lon: String::new(),
            checkin_note: String::new(),
            checkin_kind: "manual".to_string(),
            bookmark_name: String::new(),
            bookmark_lat: String::new(),
            bookmark_lon: String::new(),
            profile_name: String::new(),
            mfa_code: String::new(),
            emergency_lat: String::new(),
            emergency_lon: String::new(),
            change_password_current: String::new(),
            change_password_new: String::new(),
            change_password_confirm: String::new(),
            admin_search_query: String::new(),
            kiosk_token_input: String::new(),
            kiosk_note_input: String::new(),
            kiosk_consent_accepted: false,
            mobile_nav_open: false,
            mobile_publish_open: false,
        }
    }

    /// Compatibility constructor for the concrete REST client.
    pub fn new_rest(
        api_client: Arc<crate::api::ApiClient>,
        screenshot_path: Option<String>,
    ) -> Self {
        Self::new(api_client as DynBackend, screenshot_path)
    }

    pub fn set_offline_mode(&mut self, offline: bool) {
        self.app.set_offline_mode(offline);
    }

    pub fn set_screen(&mut self, screen: CurrentScreen) {
        self.app.set_screen(screen);
    }

    /// Touch/narrow layout: below ~700 pt the desktop tab strip cannot fit.
    fn is_compact(ctx: &egui::Context) -> bool {
        ctx.screen_rect().width() < 700.0
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
                PendingAction::SendFriendRequest { handle } => {
                    let _ = app.send_friend_request(handle).await;
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
                PendingAction::SearchPlaces { query } => {
                    let v = crate::security::validation::validate_search_query(&query);
                    if !v.is_valid() {
                        app.set_error(Some(v.errors.join("; ")));
                        return;
                    }
                    if !app
                        .rate_limiter()
                        .lock()
                        .unwrap()
                        .check("nominatim")
                        .is_allowed()
                    {
                        app.set_error(Some("Search rate limited — try again shortly.".to_string()));
                        return;
                    }
                    let client = OsmClient::new().ok();
                    if let Some(client) = client {
                        match client.search(&query).await {
                            Ok(results) => {
                                app.data().lock().unwrap().search_results = results;
                            }
                            Err(e) => {
                                app.data().lock().unwrap().search_error = Some(e.to_string());
                            }
                        }
                    }
                }
                PendingAction::DeleteShare { share_id } => {
                    let _ = app.delete_share(share_id).await;
                }
                PendingAction::CreateShareEx {
                    friendship_id,
                    direction,
                    precision,
                    duration_hours,
                } => {
                    let _ = app
                        .create_share_with_expiry(
                            friendship_id,
                            direction,
                            precision,
                            duration_hours,
                            None,
                        )
                        .await;
                }
                PendingAction::DeleteFriendship { id } => {
                    let _ = app.delete_friendship(id).await;
                }
                PendingAction::CreateCheckIn {
                    lat,
                    lon,
                    note,
                    kind,
                } => {
                    let _ = app.create_check_in(lat, lon, note, kind).await;
                }
                PendingAction::TriggerEmergency { lat, lon } => {
                    let _ = app.trigger_emergency(lat, lon).await;
                }
                PendingAction::Route { from, to, mode } => {
                    let travel_mode = match mode.as_str() {
                        "driving" => crate::osm::TravelMode::Driving,
                        "bike" => crate::osm::TravelMode::Bike,
                        _ => crate::osm::TravelMode::Foot,
                    };
                    if let Ok(client) = OsmClient::new() {
                        match client.route(from, to, travel_mode).await {
                            Ok(r) => {
                                let mins = (r.duration_s / 60.0) as i64;
                                app.set_notice(Some(format!(
                                    "Route: {:.2} km, ~{} min.",
                                    r.distance_m / 1000.0,
                                    mins
                                )));
                                app.data().lock().unwrap().last_route = Some(r);
                            }
                            Err(e) => app.set_error(Some(e.to_string())),
                        }
                    }
                }
                PendingAction::Poi { lat, lon, category } => {
                    if let Ok(client) = OsmClient::new() {
                        match client.pois(lat, lon, &category).await {
                            Ok(pois) => {
                                let count = pois.len();
                                app.data().lock().unwrap().poi_results = pois;
                                app.set_notice(Some(format!(
                                    "Found {} {} place(s) nearby",
                                    count, category
                                )));
                            }
                            Err(e) => app.set_error(Some(e.to_string())),
                        }
                    }
                }
                PendingAction::Reverse { lat, lon } => {
                    if let Ok(client) = OsmClient::new() {
                        match client.reverse(lat, lon).await {
                            Ok(place) => {
                                app.set_notice(Some(format!("Address: {}", place.display_name)));
                            }
                            Err(e) => app.set_error(Some(e.to_string())),
                        }
                    }
                }
                PendingAction::UpdateProfile { full_name } => {
                    let _ = app.update_profile(full_name).await;
                }
                PendingAction::PublishGpsLocation => {
                    let _ = app.publish_my_location().await;
                }
                PendingAction::ExportGpx { name } => {
                    let route = app.data().lock().unwrap().last_route.clone();
                    match route {
                        Some(route) => match crate::osm::export_route_gpx(&route, &name) {
                            Ok(path) => app.set_notice(Some(format!(
                                "GPX exported to {}",
                                path.display()
                            ))),
                            Err(e) => app.set_error(Some(format!("GPX export failed: {e}"))),
                        },
                        None => app.set_error(Some("No route to export.".to_string())),
                    }
                }
                PendingAction::DownloadTilePack {
                    center_lat,
                    center_lon,
                    zoom,
                } => {
                    let delta = 0.02;
                    let z = zoom.clamp(10, 15);
                    let xy = crate::tilepacks::bbox_tiles(
                        center_lat - delta,
                        center_lon - delta,
                        center_lat + delta,
                        center_lon + delta,
                        z,
                    );
                    let triples: Vec<(u32, u32, u32)> =
                        xy.into_iter().map(|(x, y)| (z as u32, x, y)).collect();
                    let backend_url = app.backend().backend_url().to_string();
                    let (ok, skipped, failed) = crate::tilepacks::download_pack(
                        &triples,
                        None,
                        Some(&backend_url),
                    )
                    .await;
                    app.set_notice(Some(format!(
                        "Offline tile pack (z{z}): {ok} downloaded, {skipped} cached, {failed} failed."
                    )));
                }
                PendingAction::RefreshAll => {
                    let _ = app.refresh_all().await;
                    let _ = app.refresh_my_location().await;
                    let _ = app.refresh_check_ins().await;
                    let _ = app.refresh_emergencies().await;
                    let _ = app.refresh_users().await;
                }
                PendingAction::KioskResolveToken { token } => {
                    let backend_url = app.backend().backend_url().to_string();
                    let routes = crate::webapp::kiosk::KioskRestRoutes::default();
                    if let Ok(adapter) =
                        crate::webapp::kiosk_rest::RestKioskAdapter::new(&backend_url, routes)
                    {
                        use crate::webapp::kiosk::KioskBackend;
                        match adapter.resolve_share_token(&token).await {
                            Ok(view) => {
                                app.set_notice(Some("Share token resolved.".into()));
                                app.data().lock().unwrap().kiosk_resolved = Some(view);
                            }
                            Err(e) => app.set_error(Some(format!("Failed to resolve token: {e}"))),
                        }
                    }
                }
                PendingAction::KioskSubmitCheckIn { request } => {
                    let backend_url = app.backend().backend_url().to_string();
                    let routes = crate::webapp::kiosk::KioskRestRoutes::default();
                    if let Ok(adapter) =
                        crate::webapp::kiosk_rest::RestKioskAdapter::new(&backend_url, routes)
                    {
                        use crate::webapp::kiosk::KioskBackend;
                        match adapter.submit_check_in(request).await {
                            Ok(()) => {
                                app.set_notice(Some("Check-in submitted successfully.".into()));
                            }
                            Err(e) => app.set_error(Some(format!("Check-in failed: {e}"))),
                        }
                    }
                }
                PendingAction::AdminLoadUsers { query } => {
                    let mut admin_state = crate::webapp::admin::AdminState {
                        query,
                        ..Default::default()
                    };
                    let backend = app.backend();
                    match crate::webapp::admin::load_users(&backend, &mut admin_state).await {
                        Ok(()) => {
                            app.data().lock().unwrap().users = admin_state.users;
                            if let Some(n) = admin_state.notice {
                                app.set_notice(Some(n));
                            }
                        }
                        Err(e) => app.set_error(Some(format!("Admin load users failed: {e}"))),
                    }
                }
                PendingAction::AdminSetUserActive { user_id, is_active } => {
                    let mut admin_state = crate::webapp::admin::AdminState::default();
                    let backend = app.backend();
                    match crate::webapp::admin::set_user_active(
                        &backend,
                        &mut admin_state,
                        user_id,
                        is_active,
                    )
                    .await
                    {
                        Ok(()) => {
                            if let Some(n) = admin_state.notice {
                                app.set_notice(Some(n));
                            }
                            let mut reload_state = crate::webapp::admin::AdminState::default();
                            if let Ok(()) =
                                crate::webapp::admin::load_users(&backend, &mut reload_state).await
                            {
                                app.data().lock().unwrap().users = reload_state.users;
                            }
                        }
                        Err(e) => app.set_error(Some(format!("Admin user update failed: {e}"))),
                    }
                }
            }
        });
        ctx.request_repaint();
    }

    fn draw_header_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let signed_in = self.app.is_signed_in();
        let lang = self.lang;

        if Self::is_compact(ctx) {
            // Android/iOS status bar sits over the window; leave a safe inset.
            ui.add_space(28.0);
            let auth_screen = matches!(
                self.app.current_screen(),
                CurrentScreen::Login | CurrentScreen::Register
            );
            ui.horizontal(|ui| {
                if auth_screen {
                    if ui.button("←").clicked() {
                        self.app.set_screen(CurrentScreen::Map);
                    }
                } else if ui.button("☰").clicked() {
                    self.mobile_nav_open = !self.mobile_nav_open;
                }
                ui.label(
                    RichText::new(i18n::t(lang, key::APP_TITLE))
                        .size(20.0)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    if !auth_screen {
                        if signed_in {
                            if ui.button(i18n::t(lang, key::SIGN_OUT)).clicked() {
                                self.spawn_action(PendingAction::Logout, ctx);
                            }
                        } else if ui.button(i18n::t(lang, key::SIGN_IN)).clicked() {
                            self.app.set_screen(CurrentScreen::Login);
                        }
                    }
                    if self.app.is_loading() {
                        ui.spinner();
                    }
                });
            });
            if let Some(err) = self.app.error_message() {
                ui.colored_label(Color32::RED, err);
            } else if let Some(notice) = self.app.notice_message() {
                ui.colored_label(Color32::from_rgb(52, 211, 153), notice);
            }
            ui.add_space(4.0);
            return;
        }

        ui.horizontal(|ui| {
            ui.with_layout(
                egui::Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    ui.heading(i18n::t(lang, key::APP_TITLE));
                },
            );
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                // Language picker (works for every registered language).
                egui::ComboBox::from_id_salt("lang")
                    .selected_text(self.lang.native_name())
                    .show_ui(ui, |ui| {
                        for l in Lang::all() {
                            if ui
                                .selectable_label(self.lang == *l, l.native_name())
                                .clicked()
                            {
                                self.lang = *l;
                            }
                        }
                    });
                ui.add_space(4.0);
                if signed_in {
                    if ui.button(i18n::t(lang, key::SIGN_OUT)).clicked() {
                        self.spawn_action(PendingAction::Logout, ctx);
                    }
                    if let Some(user) = &self.app.data().lock().unwrap().user {
                        ui.label(user.email.to_string());
                    }
                } else {
                    if ui.button(i18n::t(lang, key::SIGN_IN)).clicked() {
                        self.app.set_screen(CurrentScreen::Login);
                    }
                }
                ui.add_space(4.0);
                if self.app.is_offline_mode() {
                    ui.label(i18n::t(lang, key::OFFLINE_MODE));
                }
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

    fn draw_main_toolbar(&mut self, ui: &mut egui::Ui) {
        if Self::is_compact(ui.ctx()) {
            return;
        }
        let lang = self.lang;
        let screens = [
            (CurrentScreen::Map, key::MAP),
            (CurrentScreen::Friends, key::FRIENDS),
            (CurrentScreen::Sharing, key::SHARING),
            (CurrentScreen::Places, key::PLACES),
            (CurrentScreen::Bookmarks, key::SAVED),
            (CurrentScreen::Trips, key::TRIPS),
            (CurrentScreen::Emergency, key::EMERGENCY),
            (CurrentScreen::Profile, key::PROFILE),
            (CurrentScreen::Admin, key::ADMIN),
            (CurrentScreen::Kiosk, key::KIOSK),
            (CurrentScreen::ChangePassword, key::PASSWORD),
        ];
        ui.horizontal(|ui| {
            for (screen, key_str) in screens {
                if ui
                    .selectable_label(self.app.current_screen() == screen, i18n::t(lang, key_str))
                    .clicked()
                {
                    self.app.set_screen(screen);
                }
                ui.add_space(4.0);
            }
            if ui.button(i18n::t(lang, key::REFRESH)).clicked() {
                let app = self.app.clone();
                let rt = Arc::new(Runtime::new().expect("tokio"));
                // Fire-and-forget refresh on a throwaway runtime thread.
                std::thread::spawn(move || {
                    rt.block_on(async move {
                        let _ = app.refresh_all().await;
                        let _ = app.refresh_my_location().await;
                        let _ = app.refresh_check_ins().await;
                        let _ = app.refresh_emergencies().await;
                    });
                });
            }
        });
        ui.add_space(12.0);
    }

    fn draw_sign_in_required(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, feature: &str) {
        let lang = self.lang;
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.heading(i18n::t(lang, key::SIGN_IN_REQUIRED));
            ui.add_space(16.0);
            ui.label(i18n::t(lang, key::SIGN_IN_TO_USE).replace("{feature}", feature));
            ui.add_space(24.0);
            ui.group(|ui| {
                ui.label(i18n::t(lang, key::SIGN_IN));
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
                if ui.button(i18n::t(lang, key::SIGN_IN)).clicked()
                    && !self.login_email.is_empty()
                    && !self.login_password.is_empty()
                {
                    let email = self.login_email.clone();
                    let password = self.login_password.clone();
                    self.spawn_action(PendingAction::Login { email, password }, ctx);
                }
                ui.add_space(8.0);
                ui.label(i18n::t(lang, key::OFFLINE_MAP_HINT));
                ui.add_space(8.0);
                if ui.button(i18n::t(lang, key::GO_TO_MAP)).clicked() {
                    self.app.set_screen(CurrentScreen::Map);
                }
            });
            ui.add_space(24.0);
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

        let compact = Self::is_compact(ctx);
        if compact && self.mobile_nav_open {
            let lang = self.lang;
            let current = self.app.current_screen();
            let mut chosen: Option<CurrentScreen> = None;
            let mut chosen_lang: Option<Lang> = None;
            egui::SidePanel::left("mobile_nav")
                .resizable(false)
                .default_width(280.0)
                .show(ctx, |ui| {
                    ui.add_space(28.0);
                    ui.heading(i18n::t(lang, key::APP_TITLE));
                    ui.separator();
                    for (screen, key_str) in [
                        (CurrentScreen::Map, key::MAP),
                        (CurrentScreen::Friends, key::FRIENDS),
                        (CurrentScreen::Sharing, key::SHARING),
                        (CurrentScreen::Places, key::PLACES),
                        (CurrentScreen::Bookmarks, key::SAVED),
                        (CurrentScreen::Trips, key::TRIPS),
                        (CurrentScreen::Emergency, key::EMERGENCY),
                        (CurrentScreen::Profile, key::PROFILE),
                        (CurrentScreen::Admin, key::ADMIN),
                        (CurrentScreen::Kiosk, key::KIOSK),
                        (CurrentScreen::ChangePassword, key::PASSWORD),
                    ] {
                        if ui
                            .selectable_label(current == screen, i18n::t(lang, key_str))
                            .clicked()
                        {
                            chosen = Some(screen);
                        }
                    }
                    ui.separator();
                    ui.label("Language");
                    egui::ComboBox::from_id_salt("mobile_lang")
                        .selected_text(lang.native_name())
                        .show_ui(ui, |ui| {
                            for l in Lang::all() {
                                if ui.selectable_label(lang == *l, l.native_name()).clicked() {
                                    chosen_lang = Some(*l);
                                }
                            }
                        });
                });
            if let Some(screen) = chosen {
                self.app.set_screen(screen);
                self.mobile_nav_open = false;
            }
            if let Some(l) = chosen_lang {
                self.lang = l;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_header_bar(ui, ctx);

            match self.app.current_screen() {
                CurrentScreen::Map => {
                    self.draw_main_toolbar(ui);
                    self.draw_map_screen(ui, ctx);
                }
                CurrentScreen::Login => self.draw_login_form(ui, ctx),
                CurrentScreen::Register => self.draw_register(ui, ctx),
                CurrentScreen::Places => self.draw_places_screen(ui, ctx),
                CurrentScreen::Bookmarks => self.draw_bookmarks_screen(ui, ctx),
                CurrentScreen::Trips => {
                    if self.app.is_signed_in() {
                        self.draw_trips_screen(ui, ctx);
                    } else {
                        self.draw_sign_in_required(ui, ctx, "Trips");
                    }
                }
                CurrentScreen::Emergency => {
                    if self.app.is_signed_in() {
                        self.draw_emergency_screen(ui, ctx);
                    } else {
                        self.draw_sign_in_required(ui, ctx, "Emergency");
                    }
                }
                CurrentScreen::Profile => {
                    if self.app.is_signed_in() {
                        self.draw_profile_screen(ui, ctx);
                    } else {
                        self.draw_sign_in_required(ui, ctx, "Profile");
                    }
                }
                CurrentScreen::Friends => {
                    if self.app.is_signed_in() {
                        self.draw_main_toolbar(ui);
                        self.draw_friends_screen(ui, ctx);
                    } else {
                        self.draw_sign_in_required(ui, ctx, "Friends");
                    }
                }
                CurrentScreen::Sharing => {
                    if self.app.is_signed_in() {
                        self.draw_main_toolbar(ui);
                        self.draw_sharing_screen(ui, ctx);
                    } else {
                        self.draw_sign_in_required(ui, ctx, "Sharing");
                    }
                }
                CurrentScreen::Admin => {
                    if self.app.is_signed_in() {
                        self.draw_admin_screen(ui, ctx);
                    } else {
                        self.draw_sign_in_required(ui, ctx, "Admin");
                    }
                }
                CurrentScreen::Kiosk => {
                    self.draw_main_toolbar(ui);
                    self.draw_kiosk_screen(ui, ctx);
                }
                CurrentScreen::ChangePassword => {
                    if self.app.is_signed_in() {
                        self.draw_main_toolbar(ui);
                        self.draw_change_password_screen(ui, ctx);
                    } else {
                        self.draw_sign_in_required(ui, ctx, "Change Password");
                    }
                }
            }
        });

        // Request repaint for animation/screenshot
        if self.app.screenshot_mode() && !self.app.screenshot_taken() {
            ctx.request_repaint();
        }
    }
}
