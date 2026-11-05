use crate::api::ApiClient;
use crate::models::*;
use std::sync::{Arc, Mutex};

pub struct PinpointApp {
    data: Arc<Mutex<AppData>>,
    api_client: Arc<ApiClient>,
    is_loading: Arc<Mutex<bool>>,
    screenshot_path: Option<String>,
    screenshot_mode: bool,
    screenshot_counter: Arc<Mutex<i32>>,
    screenshot_taken: Arc<Mutex<bool>>,
    current_screen: Arc<Mutex<CurrentScreen>>,
    error_message: Arc<Mutex<Option<String>>>,
    notice_message: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum CurrentScreen {
    #[default]
    Login,
    Register,
    Map,
    Friends,
    Sharing,
    Admin,
    ChangePassword,
}

impl PinpointApp {
    pub fn new(api_client: Arc<ApiClient>, screenshot_path: Option<String>) -> Self {
        let screenshot_mode = screenshot_path.is_some();
        Self {
            data: Arc::new(Mutex::new(AppData::default())),
            api_client,
            is_loading: Arc::new(Mutex::new(false)),
            screenshot_path,
            screenshot_mode,
            screenshot_counter: Arc::new(Mutex::new(0)),
            screenshot_taken: Arc::new(Mutex::new(false)),
            current_screen: Arc::new(Mutex::new(CurrentScreen::Login)),
            error_message: Arc::new(Mutex::new(None)),
            notice_message: Arc::new(Mutex::new(None)),
        }
    }

    pub fn data(&self) -> Arc<Mutex<AppData>> {
        Arc::clone(&self.data)
    }

    pub fn backend_url(&self) -> String {
        self.api_client.backend_url().to_string()
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
        *self.current_screen.lock().unwrap() = screen;
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
        self.set_loading(true);
        self.set_error(None);

        let result = self.api_client.login(&email, &password).await;

        self.set_loading(false);

        match result {
            Ok(user) => {
                self.apply_signed_in(user);
                if self.current_screen() == CurrentScreen::Map {
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

    pub async fn register(
        &self,
        email: String,
        password: String,
        full_name: String,
    ) -> anyhow::Result<()> {
        self.set_loading(true);
        self.set_error(None);

        let result = self
            .api_client
            .register(&email, &password, &full_name)
            .await;

        self.set_loading(false);

        match result {
            Ok(user) => {
                self.apply_signed_in(user);
                if self.current_screen() == CurrentScreen::Map {
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

    /// Logs out only after the server confirms the revocation; a failed
    /// server-side logout keeps local state and surfaces the error so the
    /// UI never claims a logout that did not happen.
    pub async fn logout(&self) -> anyhow::Result<()> {
        self.set_loading(true);
        let result = self.api_client.logout().await;
        self.set_loading(false);

        match result {
            Ok(()) => {
                *self.data.lock().unwrap() = AppData::default();
                self.set_screen(CurrentScreen::Login);
                self.set_notice(Some("Signed out.".to_string()));
                Ok(())
            }
            Err(e) => {
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
            .api_client
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
                    *self.data.lock().unwrap() = AppData::default();
                    self.set_screen(CurrentScreen::Login);
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
        let friendships = self.api_client.get_friendships().await?;
        let mut data = self.data.lock().unwrap();
        data.friendships = friendships;
        Ok(())
    }

    pub async fn refresh_shares(&self) -> anyhow::Result<()> {
        let shares = self.api_client.get_location_shares().await?;
        let mut data = self.data.lock().unwrap();
        data.shares = shares;
        Ok(())
    }

    pub async fn refresh_friend_locations(&self) -> anyhow::Result<()> {
        let locations = self.api_client.get_friend_locations().await?;
        let mut data = self.data.lock().unwrap();
        data.locations = locations;
        Ok(())
    }

    pub async fn send_friend_request(&self, email: String) -> anyhow::Result<()> {
        self.set_loading(true);
        self.set_error(None);
        let result = self.api_client.send_friend_request(&email).await;
        self.set_loading(false);
        if result.is_ok() {
            if let Err(e) = self.refresh_friendships().await {
                self.set_error(Some(e.to_string()));
            }
        } else {
            self.set_error(result.as_ref().err().map(|e| e.to_string()));
        }
        result
    }

    pub async fn respond_friendship(&self, id: i64, action: String) -> anyhow::Result<()> {
        self.set_loading(true);
        let result = self.api_client.respond_to_friendship(id, &action).await;
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
            .api_client
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
            .api_client
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
        let result = self.api_client.publish_location(lat, lon, accuracy).await;
        if let Err(e) = &result {
            self.set_error(Some(e.to_string()));
        } else {
            self.set_notice(Some("Location published.".to_string()));
            if let Err(e) = self.refresh_friend_locations().await {
                self.set_error(Some(e.to_string()));
            }
        }
        result
    }
}
