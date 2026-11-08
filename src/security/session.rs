//! Security — Session management with timeout.
//!
//! Manages authenticated sessions with automatic timeout, idle detection,
//! and secure session lifecycle.

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Session state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionState {
    /// No session established.
    Unauthenticated,
    /// Session is active.
    Authenticated,
    /// Session has timed out due to inactivity.
    TimedOut,
    /// Session was explicitly terminated.
    Terminated,
}

/// Session configuration.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Maximum session lifetime in seconds.
    pub max_lifetime_seconds: u64,
    /// Idle timeout in seconds.
    pub idle_timeout_seconds: u64,
    /// Require re-authentication after this many seconds.
    pub reauth_after_seconds: Option<u64>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_lifetime_seconds: 86400, // 24 hours
            idle_timeout_seconds: 1800,  // 30 minutes
            reauth_after_seconds: None,
        }
    }
}

/// Session information.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub state: SessionState,
    pub user_id: Option<i64>,
    pub email: Option<String>,
}

/// Manages session lifecycle with timeout enforcement.
pub struct SessionManager {
    config: SessionConfig,
    session: Mutex<Option<SessionInfo>>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new(SessionConfig::default())
    }
}

impl SessionManager {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config,
            session: Mutex::new(None),
        }
    }

    /// Creates a new session after successful authentication.
    pub fn create_session(&self, user_id: i64, email: &str) -> Result<String, SessionError> {
        let session_id = generate_session_id();
        let now = Instant::now();
        let mut session = self.session.lock().unwrap();

        // Terminate any existing session
        if let Some(ref s) = *session {
            if s.state == SessionState::Authenticated {
                // Previous session terminated due to new login
            }
        }

        *session = Some(SessionInfo {
            session_id: session_id.clone(),
            created_at: now,
            last_activity: now,
            state: SessionState::Authenticated,
            user_id: Some(user_id),
            email: Some(email.to_string()),
        });

        Ok(session_id)
    }

    /// Terminates the current session.
    pub fn terminate_session(&self) -> Result<(), SessionError> {
        let mut session = self.session.lock().unwrap();
        if let Some(ref mut s) = *session {
            s.state = SessionState::Terminated;
            Ok(())
        } else {
            Err(SessionError::NoActiveSession)
        }
    }

    /// Gets the current session info.
    pub fn get_session(&self) -> Option<SessionInfo> {
        self.session.lock().unwrap().clone()
    }

    /// Checks if the session is currently active.
    pub fn is_authenticated(&self) -> bool {
        let session = self.session.lock().unwrap();
        matches!(
            session.as_ref(),
            Some(s)
                if s.state == SessionState::Authenticated && !self.is_session_expired(s)
        )
    }

    /// Records activity on the session (updates last_activity).
    pub fn record_activity(&self) -> Result<(), SessionError> {
        let mut session = self.session.lock().unwrap();

        match session.as_mut() {
            Some(s) => {
                // Check if session has expired
                if self.is_session_expired(s) {
                    s.state = SessionState::TimedOut;
                    return Err(SessionError::SessionExpired);
                }

                // Update last activity
                s.last_activity = Instant::now();
                Ok(())
            }
            None => Err(SessionError::NoActiveSession),
        }
    }

    /// Checks if the session has expired based on config.
    fn is_session_expired(&self, session: &SessionInfo) -> bool {
        let now = Instant::now();

        // Check max lifetime
        if now.duration_since(session.created_at)
            > Duration::from_secs(self.config.max_lifetime_seconds)
        {
            return true;
        }

        // Check idle timeout
        if now.duration_since(session.last_activity)
            > Duration::from_secs(self.config.idle_timeout_seconds)
        {
            return true;
        }

        // Check reauth requirement
        if let Some(reauth_after) = self.config.reauth_after_seconds {
            if now.duration_since(session.created_at) > Duration::from_secs(reauth_after) {
                return true;
            }
        }

        false
    }

    /// Returns the time until the session expires (idle timeout).
    pub fn time_until_idle_timeout(&self) -> Option<Duration> {
        let session = self.session.lock().unwrap();
        match session.as_ref() {
            Some(s) => {
                let elapsed = s.last_activity.elapsed();
                let timeout = Duration::from_secs(self.config.idle_timeout_seconds);
                if elapsed < timeout {
                    Some(timeout - elapsed)
                } else {
                    Some(Duration::ZERO)
                }
            }
            None => None,
        }
    }

    /// Returns the time until the session max lifetime expires.
    pub fn time_until_max_lifetime(&self) -> Option<Duration> {
        let session = self.session.lock().unwrap();
        match session.as_ref() {
            Some(s) => {
                let elapsed = s.created_at.elapsed();
                let lifetime = Duration::from_secs(self.config.max_lifetime_seconds);
                if elapsed < lifetime {
                    Some(lifetime - elapsed)
                } else {
                    Some(Duration::ZERO)
                }
            }
            None => None,
        }
    }

    /// Returns remaining idle time in seconds.
    pub fn remaining_idle_seconds(&self) -> Option<u64> {
        self.time_until_idle_timeout().map(|d| d.as_secs())
    }
}

/// Session errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionError {
    NoActiveSession,
    SessionExpired,
    SessionAlreadyTerminated,
    InvalidSession,
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::NoActiveSession => write!(f, "No active session"),
            SessionError::SessionExpired => write!(f, "Session expired"),
            SessionError::SessionAlreadyTerminated => {
                write!(f, "Session already terminated")
            }
            SessionError::InvalidSession => write!(f, "Invalid session"),
        }
    }
}

impl std::error::Error for SessionError {}

/// Generates a random session ID.
fn generate_session_id() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_validate_session() {
        let mgr = SessionManager::default();
        assert!(!mgr.is_authenticated());

        let session_id = mgr.create_session(1, "alice@example.com").unwrap();
        assert!(!session_id.is_empty());
        assert!(mgr.is_authenticated());

        let session = mgr.get_session().unwrap();
        assert_eq!(session.session_id, session_id);
        assert_eq!(session.user_id, Some(1));
        assert_eq!(session.email, Some("alice@example.com".to_string()));
        assert_eq!(session.state, SessionState::Authenticated);
    }

    #[test]
    fn session_timeout_on_activity() {
        let config = SessionConfig {
            idle_timeout_seconds: 1,
            max_lifetime_seconds: 3600,
            reauth_after_seconds: None,
        };
        let mgr = SessionManager::new(config);

        mgr.create_session(1, "alice@example.com").unwrap();
        assert!(mgr.is_authenticated());

        // Record activity immediately - should work
        assert!(mgr.record_activity().is_ok());

        // Wait for idle timeout
        std::thread::sleep(Duration::from_secs(2));

        // Should now be expired
        assert!(mgr.record_activity().is_err());
        assert!(!mgr.is_authenticated());
        let session = mgr.get_session().unwrap();
        assert_eq!(session.state, SessionState::TimedOut);
    }

    #[test]
    fn terminate_session() {
        let mgr = SessionManager::default();
        mgr.create_session(1, "alice@example.com").unwrap();
        assert!(mgr.is_authenticated());

        assert!(mgr.terminate_session().is_ok());
        assert!(!mgr.is_authenticated());
    }

    #[test]
    fn remaining_idle_time() {
        let mgr = SessionManager::default();
        mgr.create_session(1, "alice@example.com").unwrap();

        let remaining = mgr.remaining_idle_seconds().unwrap();
        assert!(remaining > 0);
        assert!(remaining <= 1800); // default 30 min
    }

    #[test]
    fn custom_config() {
        let config = SessionConfig {
            idle_timeout_seconds: 1,
            max_lifetime_seconds: 3600,
            reauth_after_seconds: None,
        };
        let mgr = SessionManager::new(config);
        mgr.create_session(1, "alice@example.com").unwrap();

        std::thread::sleep(Duration::from_secs(2));
        assert!(!mgr.is_authenticated());
    }
}
