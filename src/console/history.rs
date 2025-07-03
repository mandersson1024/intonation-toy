// Console History Management
// Maintains command history with navigation support for the development console

/// Maximum number of commands to store in history to prevent memory issues
const MAX_HISTORY_SIZE: usize = 100;

/// Console command history manager
/// 
/// Maintains a list of previously executed commands with navigation support.
/// Provides up/down arrow navigation functionality and memory management.
#[derive(Debug, Clone)]
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

    /// Create a new console history manager with custom size limit
    #[cfg(test)]
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            commands: Vec::new(),
            current_position: None,
            max_size: max_size.max(1), // Ensure at least 1 command can be stored
        }
    }

    /// Add a command to history
    /// 
    /// Commands are stored in chronological order with most recent first.
    /// Duplicate consecutive commands are not stored.
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
    /// 
    /// Returns the command at the previous position in history, or None if at the end.
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
    /// 
    /// Returns the command at the next position in history, or None if at the beginning.
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

    /// Get current command at navigation position
    #[cfg(test)]
    pub fn current_command(&self) -> Option<&str> {
        match self.current_position {
            None => None,
            Some(pos) => self.commands.get(pos).map(|s| s.as_str()),
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

    /// Get all commands in history (most recent first)
    #[cfg(test)]
    pub fn commands(&self) -> &[String] {
        &self.commands
    }

    /// Get maximum history size
    #[cfg(test)]
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Serialize history to JSON string for persistence
    /// 
    /// This can be used with browser local storage to persist command history
    /// across sessions. Returns None if serialization fails.
    pub fn to_json(&self) -> Option<String> {
        // Simple JSON serialization of commands array
        let commands_json: Vec<String> = self.commands.iter()
            .map(|cmd| format!("\"{}\"", cmd.replace('"', "\\\"")))
            .collect();
        
        Some(format!("[{}]", commands_json.join(",")))
    }

    /// Deserialize history from JSON string
    /// 
    /// This can be used to restore command history from browser local storage.
    /// Invalid JSON or malformed data will be ignored silently to maintain stability.
    pub fn from_json(&mut self, json: &str) {
        // Simple JSON parsing for commands array
        let json = json.trim();
        if !json.starts_with('[') || !json.ends_with(']') {
            return; // Not a valid array
        }

        let inner = json[1..json.len()-1].trim();
        if inner.is_empty() {
            // Empty array - clear history
            self.commands.clear();
            self.current_position = None;
            return;
        }

        // Parse simple string array
        let mut commands = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut escaped = false;
        
        for ch in inner.chars() {
            match ch {
                '"' if !escaped => {
                    in_quotes = !in_quotes;
                    if !in_quotes {
                        // End of string, add to commands
                        if !current.trim().is_empty() {
                            commands.push(current.clone());
                        }
                        current.clear();
                    }
                }
                '\\' if in_quotes && !escaped => {
                    escaped = true;
                }
                ',' if !in_quotes => {
                    // Command separator (already handled above)
                }
                _ if in_quotes => {
                    if escaped {
                        // Handle escaped characters
                        match ch {
                            '"' => current.push('"'),
                            '\\' => current.push('\\'),
                            _ => {
                                current.push('\\');
                                current.push(ch);
                            }
                        }
                        escaped = false;
                    } else {
                        current.push(ch);
                    }
                }
                _ => {
                    // Ignore whitespace outside quotes
                }
            }
        }

        // Restore commands (JSON array is already in most recent first order)
        self.commands.clear();
        self.current_position = None;
        
        for command in commands.into_iter() {
            if self.commands.len() < self.max_size {
                self.commands.push(command);
            }
        }
    }
}

impl Default for ConsoleHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_history() {
        let history = ConsoleHistory::new();
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
        assert_eq!(history.max_size(), MAX_HISTORY_SIZE);
    }

    #[test]
    fn test_with_max_size() {
        let history = ConsoleHistory::with_max_size(5);
        assert_eq!(history.max_size(), 5);
        
        // Test minimum size enforcement
        let history = ConsoleHistory::with_max_size(0);
        assert_eq!(history.max_size(), 1);
    }

    #[test]
    fn test_add_command() {
        let mut history = ConsoleHistory::new();
        
        history.add_command("help".to_string());
        assert_eq!(history.len(), 1);
        assert_eq!(history.commands()[0], "help");
        
        history.add_command("clear".to_string());
        assert_eq!(history.len(), 2);
        assert_eq!(history.commands()[0], "clear"); // Most recent first
        assert_eq!(history.commands()[1], "help");
    }

    #[test]
    fn test_duplicate_commands() {
        let mut history = ConsoleHistory::new();
        
        history.add_command("help".to_string());
        history.add_command("help".to_string()); // Duplicate
        
        assert_eq!(history.len(), 1); // Only one command stored
    }

    #[test]
    fn test_empty_commands() {
        let mut history = ConsoleHistory::new();
        
        history.add_command("".to_string());
        history.add_command("   ".to_string()); // Whitespace only
        
        assert_eq!(history.len(), 0); // No commands stored
    }

    #[test]
    fn test_size_limit() {
        let mut history = ConsoleHistory::with_max_size(2);
        
        history.add_command("first".to_string());
        history.add_command("second".to_string());
        history.add_command("third".to_string()); // Should remove "first"
        
        assert_eq!(history.len(), 2);
        assert_eq!(history.commands()[0], "third");
        assert_eq!(history.commands()[1], "second");
    }

    #[test]
    fn test_navigation_empty_history() {
        let mut history = ConsoleHistory::new();
        
        assert_eq!(history.navigate_previous(), None);
        assert_eq!(history.navigate_next(), None);
    }

    #[test]
    fn test_navigation_single_command() {
        let mut history = ConsoleHistory::new();
        history.add_command("help".to_string());
        
        assert_eq!(history.navigate_previous(), Some("help"));
        assert_eq!(history.navigate_previous(), None); // At end
        assert_eq!(history.navigate_next(), Some("")); // Back to beginning
        assert_eq!(history.navigate_next(), None); // Not navigating
    }

    #[test]
    fn test_navigation_multiple_commands() {
        let mut history = ConsoleHistory::new();
        history.add_command("first".to_string());
        history.add_command("second".to_string());
        history.add_command("third".to_string());
        
        // Navigate backwards
        assert_eq!(history.navigate_previous(), Some("third"));
        assert_eq!(history.navigate_previous(), Some("second"));
        assert_eq!(history.navigate_previous(), Some("first"));
        assert_eq!(history.navigate_previous(), None); // At end
        
        // Navigate forwards
        assert_eq!(history.navigate_next(), Some("second"));
        assert_eq!(history.navigate_next(), Some("third"));
        assert_eq!(history.navigate_next(), Some("")); // Back to beginning
        assert_eq!(history.navigate_next(), None); // Not navigating
    }

    #[test]
    fn test_navigation_reset() {
        let mut history = ConsoleHistory::new();
        history.add_command("help".to_string());
        
        assert_eq!(history.navigate_previous(), Some("help"));
        history.reset_navigation();
        assert_eq!(history.current_command(), None);
    }

    #[test]
    fn test_add_command_resets_navigation() {
        let mut history = ConsoleHistory::new();
        history.add_command("first".to_string());
        history.add_command("second".to_string());
        
        // Start navigating
        assert_eq!(history.navigate_previous(), Some("second"));
        assert_eq!(history.current_command(), Some("second"));
        
        // Add new command should reset navigation
        history.add_command("third".to_string());
        assert_eq!(history.current_command(), None);
    }

    #[test]
    fn test_json_serialization() {
        let mut history = ConsoleHistory::new();
        history.add_command("help".to_string());
        history.add_command("clear".to_string());
        history.add_command("api-status".to_string());
        
        let json = history.to_json().unwrap();
        // Commands are stored most recent first
        assert_eq!(json, r#"["api-status","clear","help"]"#);
    }

    #[test]
    fn test_json_serialization_empty() {
        let history = ConsoleHistory::new();
        let json = history.to_json().unwrap();
        assert_eq!(json, "[]");
    }

    #[test]
    fn test_json_serialization_with_quotes() {
        let mut history = ConsoleHistory::new();
        history.add_command(r#"echo "hello world""#.to_string());
        
        let json = history.to_json().unwrap();
        assert_eq!(json, r#"["echo \"hello world\""]"#);
    }

    #[test]
    fn test_json_deserialization() {
        let mut history = ConsoleHistory::new();
        history.from_json(r#"["api-status","clear","help"]"#);
        
        // Commands should be restored in original order (most recent first)
        assert_eq!(history.len(), 3);
        assert_eq!(history.commands()[0], "api-status");
        assert_eq!(history.commands()[1], "clear");
        assert_eq!(history.commands()[2], "help");
    }

    #[test]
    fn test_json_deserialization_empty() {
        let mut history = ConsoleHistory::new();
        history.add_command("test".to_string());
        
        history.from_json("[]");
        assert!(history.is_empty());
    }

    #[test]
    fn test_json_deserialization_invalid() {
        let mut history = ConsoleHistory::new();
        history.add_command("original".to_string());
        
        // Invalid JSON should be ignored silently
        history.from_json("not json");
        history.from_json("{invalid}");
        history.from_json("[unclosed");
        
        // Original command should remain
        assert_eq!(history.len(), 1);
        assert_eq!(history.commands()[0], "original");
    }

    #[test]
    fn test_json_deserialization_with_quotes() {
        let mut history = ConsoleHistory::new();
        history.from_json(r#"["echo \"hello world\""]"#);
        
        assert_eq!(history.len(), 1);
        assert_eq!(history.commands()[0], r#"echo "hello world""#);
    }

    #[test]
    fn test_json_roundtrip() {
        let mut history1 = ConsoleHistory::new();
        history1.add_command("help".to_string());
        history1.add_command("clear".to_string());
        history1.add_command(r#"echo "test""#.to_string());
        
        let json = history1.to_json().unwrap();
        
        let mut history2 = ConsoleHistory::new();
        history2.from_json(&json);
        
        assert_eq!(history1.len(), history2.len());
        assert_eq!(history1.commands(), history2.commands());
    }
}