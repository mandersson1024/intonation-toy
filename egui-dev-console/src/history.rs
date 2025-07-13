// Console History Management
// Maintains command history with navigation support for the development console

use serde::{Serialize, Deserialize};

/// Maximum number of commands to store in history to prevent memory issues
const MAX_HISTORY_SIZE: usize = 100;

/// Console command history manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleHistory {
    /// Storage for command history
    commands: Vec<String>,
    /// Current position in history for navigation (0 = most recent)
    current_position: Option<usize>,
    /// Maximum number of commands to store
    max_size: usize,
}

impl ConsoleHistory {
    /// Create a new console history manager
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            current_position: None,
            max_size: MAX_HISTORY_SIZE,
        }
    }

    /// Add a command to history
    pub fn add_command(&mut self, command: String) {
        // Don't add empty commands
        if command.trim().is_empty() {
            return;
        }

        // Don't add duplicate consecutive commands
        if let Some(last_command) = self.commands.first() {
            if last_command == &command {
                return;
            }
        }

        // Add command to the front of the history
        self.commands.insert(0, command);

        // Maintain size limit
        if self.commands.len() > self.max_size {
            self.commands.truncate(self.max_size);
        }

        // Reset navigation position
        self.current_position = None;
    }

    /// Navigate to previous command (up arrow)
    pub fn navigate_previous(&mut self) -> Option<&str> {
        if self.commands.is_empty() {
            return None;
        }

        match self.current_position {
            None => {
                // First navigation, go to most recent command
                self.current_position = Some(0);
                Some(&self.commands[0])
            }
            Some(pos) => {
                // Move backwards in history (towards older commands)
                let new_pos = pos + 1;
                if new_pos < self.commands.len() {
                    self.current_position = Some(new_pos);
                    Some(&self.commands[new_pos])
                } else {
                    // Already at oldest command
                    None
                }
            }
        }
    }

    /// Navigate to next command (down arrow)
    pub fn navigate_next(&mut self) -> Option<&str> {
        match self.current_position {
            None => None, // Not navigating
            Some(0) => {
                // At most recent, reset navigation
                self.current_position = None;
                Some("") // Return empty string to clear input
            }
            Some(pos) => {
                // Move forwards in history (towards newer commands)
                let new_pos = pos - 1;
                self.current_position = Some(new_pos);
                Some(&self.commands[new_pos])
            }
        }
    }

    /// Reset navigation position
    pub fn reset_navigation(&mut self) {
        self.current_position = None;
    }

    /// Get number of commands in history
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Default for ConsoleHistory {
    fn default() -> Self {
        Self::new()
    }
}