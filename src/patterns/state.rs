//! State Pattern — Screen/flow state management.
//!
//! The State pattern encapsulates screen-specific behavior into separate
//! state objects, allowing the UI to delegate rendering and input handling
//! to the current state object.

use crate::app::CurrentScreen;

/// Trait for all screen states.
pub trait ScreenState: Send + Sync {
    /// Returns the screen identifier.
    fn screen_id(&self) -> CurrentScreen;

    /// Returns whether the screen requires authentication.
    fn requires_auth(&self) -> bool {
        false
    }

    /// Returns the screen title.
    fn title(&self) -> &'static str;

    /// Handles navigation to another screen.
    fn navigate_to(&self, target: CurrentScreen) -> bool {
        let _ = target;
        true
    }

    /// Returns a description of what the user can do on this screen.
    fn description(&self) -> &'static str;

    /// Clones the boxed trait object.
    fn clone_box(&self) -> Box<dyn ScreenState>;
}

impl Clone for Box<dyn ScreenState> {
    fn clone(&self) -> Box<dyn ScreenState> {
        (**self).clone_box()
    }
}

/// Map screen state — the default landing screen.
#[derive(Clone)]
pub struct MapState;

impl ScreenState for MapState {
    fn screen_id(&self) -> CurrentScreen {
        CurrentScreen::Map
    }

    fn title(&self) -> &'static str {
        "Map"
    }

    fn description(&self) -> &'static str {
        "View friend locations, search places, and navigate"
    }

    fn clone_box(&self) -> Box<dyn ScreenState> {
        Box::new(self.clone())
    }
}

/// Login screen state.
#[derive(Clone)]
pub struct LoginState;

impl ScreenState for LoginState {
    fn screen_id(&self) -> CurrentScreen {
        CurrentScreen::Login
    }

    fn title(&self) -> &'static str {
        "Sign In"
    }

    fn requires_auth(&self) -> bool {
        false
    }

    fn description(&self) -> &'static str {
        "Sign in to unlock friends, sharing, and admin features"
    }

    fn clone_box(&self) -> Box<dyn ScreenState> {
        Box::new(self.clone())
    }
}

/// Register screen state.
#[derive(Clone)]
pub struct RegisterState;

impl ScreenState for RegisterState {
    fn screen_id(&self) -> CurrentScreen {
        CurrentScreen::Register
    }

    fn title(&self) -> &'static str {
        "Register"
    }

    fn requires_auth(&self) -> bool {
        false
    }

    fn description(&self) -> &'static str {
        "Create a new account"
    }

    fn clone_box(&self) -> Box<dyn ScreenState> {
        Box::new(self.clone())
    }
}

/// Friends screen state.
#[derive(Clone)]
pub struct FriendsState;

impl ScreenState for FriendsState {
    fn screen_id(&self) -> CurrentScreen {
        CurrentScreen::Friends
    }

    fn title(&self) -> &'static str {
        "Friends"
    }

    fn requires_auth(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        "Manage friend requests and view your friends"
    }

    fn clone_box(&self) -> Box<dyn ScreenState> {
        Box::new(self.clone())
    }
}

/// Sharing screen state.
#[derive(Clone)]
pub struct SharingState;

impl ScreenState for SharingState {
    fn screen_id(&self) -> CurrentScreen {
        CurrentScreen::Sharing
    }

    fn title(&self) -> &'static str {
        "Sharing"
    }

    fn requires_auth(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        "Manage location sharing permissions"
    }

    fn clone_box(&self) -> Box<dyn ScreenState> {
        Box::new(self.clone())
    }
}

/// Admin screen state.
#[derive(Clone)]
pub struct AdminState;

impl ScreenState for AdminState {
    fn screen_id(&self) -> CurrentScreen {
        CurrentScreen::Admin
    }

    fn title(&self) -> &'static str {
        "Admin"
    }

    fn requires_auth(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        "Administrator tools (admin role only)"
    }

    fn clone_box(&self) -> Box<dyn ScreenState> {
        Box::new(self.clone())
    }
}

/// Change password screen state.
#[derive(Clone)]
pub struct ChangePasswordState;

impl ScreenState for ChangePasswordState {
    fn screen_id(&self) -> CurrentScreen {
        CurrentScreen::ChangePassword
    }

    fn title(&self) -> &'static str {
        "Change Password"
    }

    fn requires_auth(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        "Change your account password"
    }

    fn clone_box(&self) -> Box<dyn ScreenState> {
        Box::new(self.clone())
    }
}

/// State context that holds the current screen state and delegates to it.
pub struct StateContext {
    current: Box<dyn ScreenState>,
    states: std::collections::HashMap<CurrentScreen, Box<dyn ScreenState>>,
}

impl StateContext {
    pub fn new() -> Self {
        let mut states: std::collections::HashMap<CurrentScreen, Box<dyn ScreenState>> =
            std::collections::HashMap::new();
        states.insert(CurrentScreen::Map, Box::new(MapState));
        states.insert(CurrentScreen::Login, Box::new(LoginState));
        states.insert(CurrentScreen::Register, Box::new(RegisterState));
        states.insert(CurrentScreen::Friends, Box::new(FriendsState));
        states.insert(CurrentScreen::Sharing, Box::new(SharingState));
        states.insert(CurrentScreen::Admin, Box::new(AdminState));
        states.insert(CurrentScreen::ChangePassword, Box::new(ChangePasswordState));

        Self {
            current: Box::new(MapState),
            states,
        }
    }

    pub fn current(&self) -> &dyn ScreenState {
        self.current.as_ref()
    }

    pub fn current_id(&self) -> CurrentScreen {
        self.current.screen_id()
    }

    pub fn set_state(&mut self, screen: CurrentScreen) -> bool {
        if let Some(state) = self.states.get(&screen) {
            self.current = state.clone_box();
            true
        } else {
            false
        }
    }

    pub fn requires_auth_for_current(&self) -> bool {
        self.current.requires_auth()
    }

    pub fn get_state(&self, screen: CurrentScreen) -> Option<&dyn ScreenState> {
        self.states.get(&screen).map(|s| s.as_ref())
    }

    pub fn list_available_states(&self) -> Vec<CurrentScreen> {
        self.states.keys().cloned().collect()
    }
}

impl Default for StateContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_is_map() {
        let ctx = StateContext::new();
        assert_eq!(ctx.current_id(), CurrentScreen::Map);
        assert_eq!(ctx.current().title(), "Map");
        assert!(!ctx.requires_auth_for_current());
    }

    #[test]
    fn friends_requires_auth() {
        let ctx = StateContext::new();
        let friends_state = ctx.get_state(CurrentScreen::Friends).unwrap();
        assert!(friends_state.requires_auth());
    }

    #[test]
    fn map_does_not_require_auth() {
        let ctx = StateContext::new();
        let map_state = ctx.get_state(CurrentScreen::Map).unwrap();
        assert!(!map_state.requires_auth());
    }

    #[test]
    fn state_transition() {
        let mut ctx = StateContext::new();
        assert!(ctx.set_state(CurrentScreen::Friends));
        assert_eq!(ctx.current_id(), CurrentScreen::Friends);
        assert_eq!(ctx.current().title(), "Friends");
    }

    #[test]
    fn list_available_states() {
        let ctx = StateContext::new();
        let states = ctx.list_available_states();
        assert_eq!(states.len(), 7);
        assert!(states.contains(&CurrentScreen::Map));
        assert!(states.contains(&CurrentScreen::Login));
        assert!(states.contains(&CurrentScreen::Admin));
    }
}
