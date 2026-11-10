use crate::backend::DynBackend;
use crate::models::*;
use crate::patterns::state::StateContext;
use crate::security::{AuditLogger, KeyManager, RateLimiter};
use std::sync::{Arc, Mutex};

/// Core application state/logic shared by desktop/mobile clients.
///
/// Depends on [`DynBackend`] (the modular backend abstraction) rather than a
/// concrete HTTP client, so changed backend architectures only require a new
/// adapter implementation.
pub struct PinpointApp {
    data: Arc<Mutex<AppData>>,
    backend: DynBackend,
    is_loading: Arc<Mutex<bool>>,
    screenshot_path: Option<String>,
    screenshot_mode: bool,
    screenshot_counter: Arc<Mutex<i32>>,
    screenshot_taken: Arc<Mutex<bool>>,
    current_screen: Arc<Mutex<CurrentScreen>>,
    error_message: Arc<Mutex<Option<String>>>,
    notice_message: Arc<Mutex<Option<String>>>,
    offline_mode: Arc<Mutex<bool>>,
    state_context: Arc<Mutex<StateContext>>,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    audit_logger: Arc<AuditLogger>,
    key_manager: Arc<KeyManager>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum CurrentScreen {
    #[default]
    Map,
    Register,
    Login,
    Friends,
    Sharing,
    Places,
    Bookmarks,
    Trips,
    Emergency,
    Profile,
    Admin,
    ChangePassword,
}

impl PinpointApp {
    fn demo_locations() -> Vec<FriendLocation> {
        let now = "2026-11-08T12:00:00Z".to_string();
        vec![
            FriendLocation {
                user_id: 1001,
                user_name: "Demo: Ana".to_string(),
                latitude: 47.0105,
                longitude: 28.8638,
                precision_level: "exact".to_string(),
                updated_at: now.clone(),
                accuracy: Some(8.0),
            },
            FriendLocation {
                user_id: 1002,
                user_name: "Demo: Mihai".to_string(),
                latitude: 47.0056,
                longitude: 28.8575,
                precision_level: "approximate".to_string(),
                updated_at: now.clone(),
                accuracy: Some(250.0),
            },
            FriendLocation {
                user_id: 1003,
                user_name: "Demo: Elena".to_string(),
                latitude: 47.0240,
                longitude: 28.8320,
                precision_level: "city_only".to_string(),
                updated_at: now,
                accuracy: None,
            },
        ]
    }

    /// Create the app with any [`DynBackend`] implementation (REST v1 by default,
    /// swappable for changed backend architectures).
    pub fn new(backend: DynBackend, screenshot_path: Option<String>) -> Self {
        let screenshot_mode = screenshot_path.is_some();
        // Seed demo friend locations so the map is functional offline
        // before any server connection. These are example locations
        // around Chicago for demonstration purposes only.
        let data = AppData {
            locations: Self::demo_locations(),
            ..Default::default()
        };

        let key_manager = KeyManager::new();
        let _ = key_manager.generate_key();

        Self {
            data: Arc::new(Mutex::new(data)),
            backend,
            is_loading: Arc::new(Mutex::new(false)),
            screenshot_path,
            screenshot_mode,
            screenshot_counter: Arc::new(Mutex::new(0)),
            screenshot_taken: Arc::new(Mutex::new(false)),
            current_screen: Arc::new(Mutex::new(CurrentScreen::Map)),
            error_message: Arc::new(Mutex::new(None)),
            notice_message: Arc::new(Mutex::new(None)),
            offline_mode: Arc::new(Mutex::new(true)),
            state_context: Arc::new(Mutex::new(StateContext::new())),
            rate_limiter: Arc::new(Mutex::new(RateLimiter::with_defaults())),
            audit_logger: Arc::new(AuditLogger::with_console()),
            key_manager: Arc::new(key_manager),
        }
    }

    /// Convenience constructor from the concrete REST client (kept for
    /// compatibility with existing call sites).
    pub fn new_rest(
        api_client: Arc<crate::api::ApiClient>,
        screenshot_path: Option<String>,
    ) -> Self {
        Self::new(api_client as DynBackend, screenshot_path)
    }

    pub fn data(&self) -> Arc<Mutex<AppData>> {
        Arc::clone(&self.data)
    }

    pub fn backend(&self) -> DynBackend {
        Arc::clone(&self.backend)
    }

    pub fn backend_url(&self) -> String {
        self.backend.backend_url().to_string()
    }

    /// Spawn the realtime polling loop (default ≤30s, settings-configurable,
    /// 0 disables via refresh_interval_secs — here clamped to ≥5s when on).
    /// One thread, never blocks the UI thread; only runs while signed in.
    pub fn start_auto_refresh(app: Arc<Self>) {
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(_) => return,
            };
            rt.block_on(async move {
                loop {
                    let interval = app
                        .data()
                        .lock()
                        .map(|d| d.settings.refresh_interval_secs)
                        .unwrap_or(30);
                    if interval == 0 {
                        // Polling disabled — idle.
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        continue;
                    }
                    let secs = interval.clamp(5, 3600);
                    tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
                    if app.is_signed_in() && !app.is_offline_mode() {
                        let _ = app.refresh_all().await;
                    }
                }
            });
        });
    }

    pub fn set_loading(&self, loading: bool) {
        *self.is_loading.lock().unwrap() = loading;
    }

    pub fn is_loading(&self) -> bool {
        *self.is_loading.lock().unwrap()
    }

    pub fn screenshot_path(&self) -> Option<String> {
        self.screenshot_path.clone()
    }

    pub fn screenshot_mode(&self) -> bool {
        self.screenshot_mode
    }

    pub fn screenshot_counter(&self) -> i32 {
        *self.screenshot_counter.lock().unwrap()
    }

    pub fn increment_screenshot_counter(&self) {
        *self.screenshot_counter.lock().unwrap() += 1;
    }

    pub fn screenshot_taken(&self) -> bool {
        *self.screenshot_taken.lock().unwrap()
    }

    pub fn set_screenshot_taken(&self, taken: bool) {
        *self.screenshot_taken.lock().unwrap() = taken;
    }

    pub fn current_screen(&self) -> CurrentScreen {
        self.current_screen.lock().unwrap().clone()
    }

    pub fn set_screen(&self, screen: CurrentScreen) {
        *self.current_screen.lock().unwrap() = screen.clone();
        let _ = self.state_context.lock().unwrap().set_state(screen);
    }

    pub fn state_context(&self) -> Arc<Mutex<StateContext>> {
        Arc::clone(&self.state_context)
    }

    pub fn rate_limiter(&self) -> Arc<Mutex<RateLimiter>> {
        Arc::clone(&self.rate_limiter)
    }

    pub fn audit_logger(&self) -> Arc<AuditLogger> {
        Arc::clone(&self.audit_logger)
    }

    pub fn key_manager(&self) -> Arc<KeyManager> {
        Arc::clone(&self.key_manager)
    }

    pub fn is_signed_in(&self) -> bool {
        self.data.lock().unwrap().user.is_some()
    }

    pub fn is_offline_mode(&self) -> bool {
        *self.offline_mode.lock().unwrap()
    }

    pub fn set_offline_mode(&self, offline: bool) {
        *self.offline_mode.lock().unwrap() = offline;
    }

    pub fn requires_sign_in(&self, screen: CurrentScreen) -> bool {
        self.state_context
            .lock()
            .unwrap()
            .get_state(screen.clone())
            .map(|s| s.requires_auth())
            .unwrap_or(matches!(
                screen,
                CurrentScreen::Friends
                    | CurrentScreen::Sharing
                    | CurrentScreen::Admin
                    | CurrentScreen::ChangePassword
            ))
    }

    pub fn error_message(&self) -> Option<String> {
        self.error_message.lock().unwrap().clone()
    }

    pub fn set_error(&self, msg: Option<String>) {
        if msg.is_some() {
            *self.notice_message.lock().unwrap() = None;
        }
        *self.error_message.lock().unwrap() = msg;
    }

    pub fn notice_message(&self) -> Option<String> {
        self.notice_message.lock().unwrap().clone()
    }

    pub fn set_notice(&self, msg: Option<String>) {
        if msg.is_some() {
            *self.error_message.lock().unwrap() = None;
        }
        *self.notice_message.lock().unwrap() = msg;
    }

    /// Applies a successful login/register: records the user, chooses the
    /// initial screen (forced password change first when the server
    /// demands it) and refreshes the visible data.
    fn apply_signed_in(&self, user: User) {
        let forced_change = user.must_change_password;
        {
            let mut data = self.data.lock().unwrap();
            data.state = if user.role == "admin" {
                AppState::Admin
            } else {
                AppState::Main
            };
            data.must_change_password = forced_change;
            data.user = Some(user);
        }
        if forced_change {
            self.set_screen(CurrentScreen::ChangePassword);
        } else {
            self.set_screen(CurrentScreen::Map);
        }
    }

    pub async fn login(&self, email: String, password: String) -> anyhow::Result<()> {
        let email_valid = crate::security::validation::validate_email(&email);
        if !email_valid.is_valid() {
            self.set_error(Some(email_valid.errors.join("; ")));
            return Err(anyhow::anyhow!(email_valid.errors.join("; ")));
        }
        if !self
            .rate_limiter
            .lock()
            .unwrap()
            .check("api_login")
            .is_allowed()
        {
            self.set_error(Some("Rate limited — try again shortly.".to_string()));
            return Err(anyhow::anyhow!("Rate limited — try again shortly."));
        }
        self.set_loading(true);
        self.set_error(None);

        let result = self.backend.login(&email, &password).await;

        self.set_loading(false);

        match result {
            Ok(user) => {
                self.audit_logger
                    .log_event("auth", "login", Some(&email), "success");
                self.set_offline_mode(false);
                self.apply_signed_in(user);
                if self.current_screen() == CurrentScreen::Map {
                    self.refresh_all().await;
                }
                Ok(())
            }
            Err(e) => {
                self.audit_logger
                    .log_event("auth", "login", Some(&email), "failure");
                self.set_error(Some(e.to_string()));
                Err(e)
            }
        }
    }

    pub async fn register(
        &self,
        email: String,
        password: String,
        full_name: String,
    ) -> anyhow::Result<()> {
        for v in [
            crate::security::validation::validate_email(&email),
            crate::security::validation::validate_password(&password),
            crate::security::validation::validate_name(&full_name),
        ] {
            if !v.is_valid() {
                self.set_error(Some(v.errors.join("; ")));
                return Err(anyhow::anyhow!(v.errors.join("; ")));
            }
        }
        self.set_loading(true);
        self.set_error(None);

        let result = self.backend.register(&email, &password, &full_name).await;

        self.set_loading(false);

        match result {
            Ok(user) => {
                self.audit_logger
                    .log_event("auth", "register", Some(&email), "success");
                self.set_offline_mode(false);
                self.apply_signed_in(user);
                if self.current_screen() == CurrentScreen::Map {
                    self.refresh_all().await;
                }
                Ok(())
            }
            Err(e) => {
                self.audit_logger
                    .log_event("auth", "register", Some(&email), "failure");
                self.set_error(Some(e.to_string()));
                Err(e)
            }
        }
    }

    /// Logs out only after the server confirms the revocation; a failed
    /// server-side logout keeps local state and surfaces the error so the
    /// UI never claims a logout that did not happen.
    pub async fn logout(&self) -> anyhow::Result<()> {
        self.set_loading(true);
        let result = self.backend.logout().await;
        self.set_loading(false);

        match result {
            Ok(()) => {
                self.audit_logger
                    .log_event("auth", "logout", None, "success");
                let fresh = AppData {
                    locations: Self::demo_locations(),
                    ..Default::default()
                };
                *self.data.lock().unwrap() = fresh;
                self.set_screen(CurrentScreen::Map);
                self.set_offline_mode(true);
                self.set_notice(Some("Signed out.".to_string()));
                Ok(())
            }
            Err(e) => {
                self.audit_logger
                    .log_event("auth", "logout", None, "failure");
                self.set_error(Some(format!("Logout failed: {e}")));
                Err(e)
            }
        }
    }

    /// Changes the password; returns the user to the login screen when the
    /// server revoked sessions (regular change), otherwise keeps the
    /// session (forced change).
    pub async fn change_password(
        &self,
        current_password: Option<String>,
        new_password: String,
        confirm_password: String,
    ) -> anyhow::Result<()> {
        self.set_loading(true);
        self.set_error(None);

        let result = self
            .backend
            .change_password(
                current_password.as_deref(),
                &new_password,
                &confirm_password,
            )
            .await;

        self.set_loading(false);

        match result {
            Ok(sessions_revoked) => {
                if sessions_revoked {
                    let fresh = AppData {
                        locations: Self::demo_locations(),
                        ..Default::default()
                    };
                    *self.data.lock().unwrap() = fresh;
                    self.set_screen(CurrentScreen::Map);
                    self.set_offline_mode(true);
                    self.set_notice(Some(
                        "Password changed. Your other sessions were signed out; please sign in again."
                            .to_string(),
                    ));
                } else {
                    {
                        let mut data = self.data.lock().unwrap();
                        data.must_change_password = false;
                    }
                    self.set_screen(CurrentScreen::Map);
                    self.set_notice(Some("Password changed.".to_string()));
                    self.refresh_all().await;
                }
                Ok(())
            }
            Err(e) => {
                self.set_error(Some(e.to_string()));
                Err(e)
            }
        }
    }

    pub async fn refresh_all(&self) {
        let screen = self.current_screen.lock().unwrap().clone();
        if screen == CurrentScreen::Login || screen == CurrentScreen::Register {
            return;
        }
        // Surface the first refresh failure instead of silently dropping it.
        if let Err(e) = self.refresh_friendships().await {
            self.set_error(Some(e.to_string()));
            return;
        }
        if let Err(e) = self.refresh_shares().await {
            self.set_error(Some(e.to_string()));
            return;
        }
        if let Err(e) = self.refresh_friend_locations().await {
            self.set_error(Some(e.to_string()));
        }
    }

    pub async fn refresh_friendships(&self) -> anyhow::Result<()> {
        let friendships = self.backend.get_friendships().await?;
        let mut data = self.data.lock().unwrap();
        data.friendships = friendships;
        Ok(())
    }

    pub async fn refresh_shares(&self) -> anyhow::Result<()> {
        let shares = self.backend.get_location_shares().await?;
        let mut data = self.data.lock().unwrap();
        data.shares = shares;
        Ok(())
    }

    pub async fn refresh_friend_locations(&self) -> anyhow::Result<()> {
        let locations = self.backend.get_friend_locations().await?;
        let mut data = self.data.lock().unwrap();
        data.locations = locations;
        Ok(())
    }

    pub async fn send_friend_request(&self, email: String) -> anyhow::Result<()> {
        let v = crate::security::validation::validate_email(&email);
        if !v.is_valid() {
            self.set_error(Some(v.errors.join("; ")));
            return Err(anyhow::anyhow!(v.errors.join("; ")));
        }
        self.set_loading(true);
        self.set_error(None);
        let result = self.backend.send_friend_request(&email).await;
        self.set_loading(false);
        if result.is_ok() {
            self.audit_logger
                .log_event("friends", "send_request", Some(&email), "success");
            if let Err(e) = self.refresh_friendships().await {
                self.set_error(Some(e.to_string()));
            }
        } else {
            self.audit_logger
                .log_event("friends", "send_request", Some(&email), "failure");
            self.set_error(result.as_ref().err().map(|e| e.to_string()));
        }
        result
    }

    pub async fn respond_friendship(&self, id: i64, action: String) -> anyhow::Result<()> {
        self.set_loading(true);
        let result = self.backend.respond_to_friendship(id, &action).await;
        self.set_loading(false);
        match &result {
            Ok(()) => {
                if let Err(e) = self.refresh_friendships().await {
                    self.set_error(Some(e.to_string()));
                } else if let Err(e) = self.refresh_shares().await {
                    self.set_error(Some(e.to_string()));
                }
            }
            Err(e) => self.set_error(Some(e.to_string())),
        }
        result
    }

    pub async fn create_share(
        &self,
        friendship_id: i64,
        direction: String,
        precision: String,
    ) -> anyhow::Result<()> {
        self.set_loading(true);
        let result = self
            .backend
            .create_location_share(friendship_id, &direction, &precision)
            .await;
        self.set_loading(false);
        match &result {
            Ok(()) => {
                if let Err(e) = self.refresh_shares().await {
                    self.set_error(Some(e.to_string()));
                }
            }
            Err(e) => self.set_error(Some(e.to_string())),
        }
        result
    }

    pub async fn update_share(
        &self,
        share_id: i64,
        is_active: Option<bool>,
        precision: Option<String>,
    ) -> anyhow::Result<()> {
        self.set_loading(true);
        let result = self
            .backend
            .update_location_share(share_id, is_active, precision)
            .await;
        self.set_loading(false);
        match &result {
            Ok(()) => {
                if let Err(e) = self.refresh_shares().await {
                    self.set_error(Some(e.to_string()));
                }
            }
            Err(e) => self.set_error(Some(e.to_string())),
        }
        result
    }

    pub async fn publish_location(&self, lat: f64, lon: f64, accuracy: f64) -> anyhow::Result<()> {
        let v = crate::security::validation::validate_location(lat, lon, Some(accuracy));
        if !v.is_valid() {
            self.set_error(Some(v.errors.join("; ")));
            return Err(anyhow::anyhow!(v.errors.join("; ")));
        }
        if !self
            .rate_limiter
            .lock()
            .unwrap()
            .check("api_general")
            .is_allowed()
        {
            self.set_error(Some("Rate limited — try again shortly.".to_string()));
            return Err(anyhow::anyhow!("Rate limited — try again shortly."));
        }
        let result = self.backend.publish_location(lat, lon, accuracy).await;
        if let Err(e) = &result {
            self.set_error(Some(e.to_string()));
        } else {
            self.audit_logger
                .log_event("location", "publish", None, "success");
            self.set_notice(Some("Location published.".to_string()));
            if let Err(e) = self.refresh_friend_locations().await {
                self.set_error(Some(e.to_string()));
            }
            if let Err(e) = self.refresh_my_location().await {
                self.set_error(Some(e.to_string()));
            }
        }
        result
    }

    // ---- Maximum-feature additions ----
    pub async fn refresh_users(&self) -> anyhow::Result<()> {
        let users = self.backend.list_users().await?;
        self.data.lock().unwrap().users = users;
        Ok(())
    }

    pub async fn refresh_my_location(&self) -> anyhow::Result<()> {
        match self.backend.get_my_location().await {
            Ok(loc) => {
                self.data.lock().unwrap().my_location = Some(loc);
                Ok(())
            }
            Err(_) => Ok(()), // 404 when never published — not an error
        }
    }

    pub async fn refresh_check_ins(&self) -> anyhow::Result<()> {
        self.data.lock().unwrap().check_ins = self.backend.list_check_ins().await?;
        Ok(())
    }

    pub async fn refresh_emergencies(&self) -> anyhow::Result<()> {
        self.data.lock().unwrap().emergencies = self.backend.list_emergencies().await?;
        Ok(())
    }

    pub async fn delete_share(&self, share_id: i64) -> anyhow::Result<()> {
        let result = self.backend.delete_share(share_id).await;
        match result {
            Ok(()) => {
                self.audit_logger
                    .log_event("shares", "delete", None, "success");
                self.refresh_shares().await
            }
            Err(e) => {
                self.set_error(Some(e.to_string()));
                Err(e)
            }
        }
    }

    pub async fn create_share_with_expiry(
        &self,
        friendship_id: i64,
        direction: String,
        precision: String,
        duration_hours: Option<i64>,
        label: Option<String>,
    ) -> anyhow::Result<()> {
        let result = self
            .backend
            .create_share_with_expiry(
                friendship_id,
                &direction,
                &precision,
                duration_hours,
                label.as_deref(),
            )
            .await;
        match result {
            Ok(()) => self.refresh_shares().await,
            Err(e) => {
                self.set_error(Some(e.to_string()));
                Err(e)
            }
        }
    }

    pub async fn delete_friendship(&self, id: i64) -> anyhow::Result<()> {
        let result = self.backend.delete_friendship(id).await;
        match result {
            Ok(()) => {
                self.refresh_friendships().await?;
                self.refresh_shares().await
            }
            Err(e) => {
                self.set_error(Some(e.to_string()));
                Err(e)
            }
        }
    }

    pub async fn create_check_in(
        &self,
        lat: f64,
        lon: f64,
        note: String,
        kind: String,
    ) -> anyhow::Result<()> {
        let result = self.backend.create_check_in(lat, lon, &note, &kind).await;
        match result {
            Ok(()) => {
                self.audit_logger
                    .log_event("checkins", "create", None, "success");
                self.refresh_check_ins().await
            }
            Err(e) => {
                self.set_error(Some(e.to_string()));
                Err(e)
            }
        }
    }

    pub async fn trigger_emergency(&self, lat: f64, lon: f64) -> anyhow::Result<()> {
        let result = self.backend.trigger_emergency(lat, lon).await;
        match result {
            Ok(()) => {
                self.audit_logger
                    .log_event("emergency", "trigger", None, "success");
                self.set_notice(Some(
                    "Emergency alert sent — all shares activated.".to_string(),
                ));
                self.refresh_emergencies().await?;
                self.refresh_shares().await
            }
            Err(e) => {
                self.set_error(Some(e.to_string()));
                Err(e)
            }
        }
    }

    pub async fn update_profile(&self, full_name: String) -> anyhow::Result<()> {
        let v = crate::security::validation::validate_name(&full_name);
        if !v.is_valid() {
            self.set_error(Some(v.errors.join("; ")));
            return Err(anyhow::anyhow!(v.errors.join("; ")));
        }
        let my_id = self.data.lock().unwrap().user.as_ref().map(|u| u.id);
        let Some(id) = my_id else {
            self.set_error(Some("Sign in required".to_string()));
            return Err(anyhow::anyhow!("Sign in required"));
        };
        let result = self
            .backend
            .update_user(id, serde_json::json!({ "full_name": full_name }))
            .await;
        match result {
            Ok(()) => self.refresh_users().await,
            Err(e) => {
                self.set_error(Some(e.to_string()));
                Err(e)
            }
        }
    }

    pub async fn mfa_enroll(&self) -> anyhow::Result<()> {
        let result = self.backend.mfa_enroll().await;
        match result {
            Ok((secret, _uri)) => {
                self.data.lock().unwrap().totp_pending_secret = Some(secret);
                Ok(())
            }
            Err(e) => {
                self.set_error(Some(e.to_string()));
                Err(e)
            }
        }
    }

    pub async fn mfa_verify(&self, code: String) -> anyhow::Result<()> {
        let result = self.backend.mfa_verify(code.trim()).await;
        match result {
            Ok(()) => {
                let mut data = self.data.lock().unwrap();
                data.mfa_enabled = true;
                data.totp_pending_secret = None;
                Ok(())
            }
            Err(e) => {
                self.set_error(Some(e.to_string()));
                Err(e)
            }
        }
    }

    pub async fn mfa_disable(&self) -> anyhow::Result<()> {
        let result = self.backend.mfa_disable().await;
        match result {
            Ok(()) => {
                let mut data = self.data.lock().unwrap();
                data.mfa_enabled = false;
                data.totp_pending_secret = None;
                Ok(())
            }
            Err(e) => {
                self.set_error(Some(e.to_string()));
                Err(e)
            }
        }
    }

    pub async fn refresh_mfa_status(&self) -> anyhow::Result<()> {
        match self.backend.mfa_status().await {
            Ok(enabled) => {
                self.data.lock().unwrap().mfa_enabled = enabled;
                Ok(())
            }
            Err(e) => {
                self.set_error(Some(e.to_string()));
                Err(e)
            }
        }
    }
}
