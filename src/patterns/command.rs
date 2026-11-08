//! Command Pattern — Encapsulate user actions as objects.
//!
//! Commands represent user actions (login, publish location, create share,
//! etc.) and can be executed, undone, and logged. This enables audit
//! logging, retry logic, and command history.

use serde::{Deserialize, Serialize};

/// Result of executing a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandResult {
    Success,
    Failure(String),
}

/// Trait for all commands.
pub trait Command: Send + Sync {
    /// Returns a human-readable name for the command.
    fn name(&self) -> &'static str;

    /// Executes the command, returning a result.
    fn execute(&mut self) -> CommandResult;

    /// Attempts to undo the command, if supported.
    fn undo(&mut self) -> CommandResult {
        CommandResult::Failure("Undo not supported".to_string())
    }

    /// Returns whether the command can be undone.
    fn is_undoable(&self) -> bool {
        false
    }

    /// Serializes the command to JSON for audit logging.
    fn to_json(&self) -> Option<String>;
}

/// Command for logging in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginCommand {
    pub email: String,
    pub password: String,
}

impl Command for LoginCommand {
    fn name(&self) -> &'static str {
        "login"
    }

    fn execute(&mut self) -> CommandResult {
        CommandResult::Success
    }

    fn to_json(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }
}

/// Command for publishing a location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishLocationCommand {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: Option<f64>,
}

impl Command for PublishLocationCommand {
    fn name(&self) -> &'static str {
        "publish_location"
    }

    fn execute(&mut self) -> CommandResult {
        CommandResult::Success
    }

    fn to_json(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }
}

/// Command for creating a location share.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShareCommand {
    pub friendship_id: i64,
    pub direction: String,
    pub precision_level: String,
}

impl Command for CreateShareCommand {
    fn name(&self) -> &'static str {
        "create_share"
    }

    fn execute(&mut self) -> CommandResult {
        CommandResult::Success
    }

    fn to_json(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }
}

/// Command for toggling a share (pause/resume).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleShareCommand {
    pub share_id: i64,
    pub is_active: bool,
}

impl Command for ToggleShareCommand {
    fn name(&self) -> &'static str {
        "toggle_share"
    }

    fn execute(&mut self) -> CommandResult {
        CommandResult::Success
    }

    fn is_undoable(&self) -> bool {
        true
    }

    fn undo(&mut self) -> CommandResult {
        self.is_active = !self.is_active;
        CommandResult::Success
    }

    fn to_json(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }
}

/// Command for sending a friend request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendFriendRequestCommand {
    pub email: String,
}

impl Command for SendFriendRequestCommand {
    fn name(&self) -> &'static str {
        "send_friend_request"
    }

    fn execute(&mut self) -> CommandResult {
        CommandResult::Success
    }

    fn to_json(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }
}

/// Command executor that maintains command history for undo/redo.
pub struct CommandExecutor {
    history: Vec<Box<dyn Command>>,
    position: usize,
    max_history: usize,
}

impl CommandExecutor {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: Vec::new(),
            position: 0,
            max_history,
        }
    }

    pub fn execute(&mut self, mut command: Box<dyn Command>) -> CommandResult {
        // Truncate history if we're not at the end (new command invalidates
        // future commands).
        self.history.truncate(self.position);

        // Enforce max history size.
        if self.history.len() >= self.max_history {
            self.history.remove(0);
            self.position -= 1;
        }

        let result = command.execute();
        if matches!(result, CommandResult::Success) && command.is_undoable() {
            self.history.push(command);
            self.position += 1;
        }
        result
    }

    pub fn undo(&mut self) -> CommandResult {
        if self.position == 0 {
            return CommandResult::Failure("Nothing to undo".to_string());
        }
        self.position -= 1;
        self.history[self.position].undo()
    }

    pub fn redo(&mut self) -> CommandResult {
        if self.position >= self.history.len() {
            return CommandResult::Failure("Nothing to redo".to_string());
        }
        let result = self.history[self.position].execute();
        if matches!(result, CommandResult::Success) {
            self.position += 1;
        }
        result
    }

    pub fn history(&self) -> Vec<String> {
        self.history
            .iter()
            .take(self.position)
            .map(|c| c.name().to_string())
            .collect()
    }

    pub fn can_undo(&self) -> bool {
        self.position > 0
    }

    pub fn can_redo(&self) -> bool {
        self.position < self.history.len()
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_and_undo() {
        let mut executor = CommandExecutor::new(10);
        let cmd = ToggleShareCommand {
            share_id: 1,
            is_active: true,
        };
        assert!(cmd.is_undoable());

        let result = executor.execute(Box::new(cmd));
        assert!(matches!(result, CommandResult::Success));
        assert!(executor.can_undo());
        assert!(!executor.can_redo());

        let undo_result = executor.undo();
        assert!(matches!(undo_result, CommandResult::Success));
        assert!(!executor.can_undo());
        assert!(executor.can_redo());
    }

    #[test]
    fn non_undoable_command() {
        let mut executor = CommandExecutor::new(10);
        let cmd = LoginCommand {
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };
        assert!(!cmd.is_undoable());

        let result = executor.execute(Box::new(cmd));
        assert!(matches!(result, CommandResult::Success));
        assert!(!executor.can_undo());
    }
}
