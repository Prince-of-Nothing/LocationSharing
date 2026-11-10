//! Command-pattern helpers for webapp actions.
//!
//! Commands encapsulate user-intended operations with validation and structured
//! outcomes, making actions auditable, testable, and easy to extend when backend
//! capabilities/architecture change.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommandOutcome {
    Success { message: String },
    Denied { reason: String },
    Invalid { reason: String },
    Failed { reason: String },
}

impl CommandOutcome {
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Success { .. })
    }
}

/// A minimal command descriptor for audit/UI logs (never include secrets).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAudit {
    pub action: String,
    pub target: Option<String>,
    pub outcome: String,
}

impl CommandAudit {
    pub fn from_outcome(
        action: impl Into<String>,
        target: Option<String>,
        outcome: &CommandOutcome,
    ) -> Self {
        let outcome = match outcome {
            CommandOutcome::Success { .. } => "success",
            CommandOutcome::Denied { .. } => "denied",
            CommandOutcome::Invalid { .. } => "invalid",
            CommandOutcome::Failed { .. } => "failed",
        }
        .to_string();
        Self {
            action: action.into(),
            target,
            outcome,
        }
    }
}

/// Admin command requests (validated before execution).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdminCommand {
    RefreshUsers { query: Option<String> },
    SetUserActive { user_id: i64, is_active: bool },
}

impl AdminCommand {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            AdminCommand::RefreshUsers { query } => {
                if let Some(q) = query {
                    crate::webapp::validation::validate_query(q)?;
                }
                Ok(())
            }
            AdminCommand::SetUserActive { user_id, .. } => {
                if *user_id <= 0 {
                    return Err("Invalid user id.".into());
                }
                Ok(())
            }
        }
    }

    pub fn audit_target(&self) -> Option<String> {
        match self {
            AdminCommand::RefreshUsers { .. } => None,
            AdminCommand::SetUserActive { user_id, .. } => Some(format!("user:{user_id}")),
        }
    }

    pub fn action_name(&self) -> &'static str {
        match self {
            AdminCommand::RefreshUsers { .. } => "admin.refresh_users",
            AdminCommand::SetUserActive {
                is_active: true, ..
            } => "admin.enable_user",
            AdminCommand::SetUserActive {
                is_active: false, ..
            } => "admin.disable_user",
        }
    }
}

/// Kiosk command requests (validated before execution).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KioskCommand {
    ResolveToken {
        token: String,
    },
    SubmitCheckIn {
        token: String,
        note: Option<String>,
        consent_accepted: bool,
    },
}

impl KioskCommand {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            KioskCommand::ResolveToken { token } => {
                crate::webapp::kiosk::KioskApp::validate_token(token)
            }
            KioskCommand::SubmitCheckIn {
                token,
                note,
                consent_accepted,
            } => {
                crate::webapp::kiosk::KioskApp::validate_token(token)?;
                crate::webapp::validation::validate_note(note.as_deref())?;
                if !*consent_accepted {
                    // Not always required, but explicit command can carry consent intent;
                    // controller enforces when required.
                }
                Ok(())
            }
        }
    }

    pub fn action_name(&self) -> &'static str {
        match self {
            KioskCommand::ResolveToken { .. } => "kiosk.resolve_token",
            KioskCommand::SubmitCheckIn { .. } => "kiosk.submit_check_in",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_command_validation() {
        assert!(AdminCommand::SetUserActive {
            user_id: 0,
            is_active: true
        }
        .validate()
        .is_err());
        assert!(AdminCommand::SetUserActive {
            user_id: 1,
            is_active: true
        }
        .validate()
        .is_ok());
    }

    #[test]
    fn audit_records_outcome() {
        let out = CommandOutcome::Denied {
            reason: "nope".into(),
        };
        let a = CommandAudit::from_outcome("x", None, &out);
        assert_eq!(a.outcome, "denied");
    }
}
