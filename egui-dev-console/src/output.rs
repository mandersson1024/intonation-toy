// Console Output System
// Handles formatting and display of console command results and messages

use std::fmt;

/// Console output message types with different styling and behavior
#[derive(Debug, Clone, PartialEq)]
pub enum ConsoleOutput {
    /// Informational message (default styling)
    Info(String),
    /// Success message (green styling)
    Success(String),
    /// Warning message (yellow styling)
    Warning(String),
    /// Error message (red styling)
    Error(String),
    /// Command echo (shows the executed command)
    Echo(String),
    /// Empty output (used for spacing or clearing)
    Empty,
}

impl ConsoleOutput {
    /// Create an info message
    pub fn info(message: impl Into<String>) -> Self {
        Self::Info(message.into())
    }

    /// Create a success message
    pub fn success(message: impl Into<String>) -> Self {
        Self::Success(message.into())
    }

    /// Create a warning message
    pub fn warning(message: impl Into<String>) -> Self {
        Self::Warning(message.into())
    }

    /// Create an error message
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error(message.into())
    }


    /// Create a command echo message
    pub fn echo(message: impl Into<String>) -> Self {
        Self::Echo(message.into())
    }

    /// Create an empty output
    pub fn empty() -> Self {
        Self::Empty
    }


    /// Get the output type as a string (for CSS classes)
    pub fn output_type(&self) -> &'static str {
        match self {
            Self::Info(_) => "info",
            Self::Success(_) => "success",
            Self::Warning(_) => "warning",
            Self::Error(_) => "error",
            Self::Echo(_) => "command",
            Self::Empty => "empty",
        }
    }


    /// Get the message content as a string
    pub fn message(&self) -> &str {
        match self {
            Self::Info(msg) | Self::Success(msg) | Self::Warning(msg) 
            | Self::Error(msg) | Self::Echo(msg) => msg,
            Self::Empty => "",
        }
    }
}

impl fmt::Display for ConsoleOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info(msg) => write!(f, "[INFO] {}", msg),
            Self::Success(msg) => write!(f, "[SUCCESS] {}", msg),
            Self::Warning(msg) => write!(f, "[WARNING] {}", msg),
            Self::Error(msg) => write!(f, "[ERROR] {}", msg),
            Self::Echo(msg) => write!(f, "> {}", msg),
            Self::Empty => write!(f, ""),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConsoleEntry {
    /// The output message
    pub output: ConsoleOutput,
}

impl ConsoleEntry {
    pub fn new(output: ConsoleOutput) -> Self {
        Self {
            output,
        }
    }
}

/// Console output manager that handles multiple output entries
#[derive(Debug, Clone)]
pub struct ConsoleOutputManager {
    /// List of console entries (most recent first)
    entries: Vec<ConsoleEntry>,
    /// Maximum number of entries to store
    max_entries: usize,
}

impl ConsoleOutputManager {
    /// Create a new output manager
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 1000, // Same as history limit
        }
    }

    #[cfg(test)]
    /// Create a new output manager with custom settings
    pub fn with_settings(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries: max_entries.max(1),
        }
    }

    /// Add a new output entry
    pub fn add_output(&mut self, output: ConsoleOutput) {
        let entry = ConsoleEntry::new(output);
        self.entries.insert(0, entry); // Most recent first

        // Maintain size limit
        if self.entries.len() > self.max_entries {
            self.entries.truncate(self.max_entries);
        }
    }

    /// Clear all output entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get all visible entries
    pub fn entries(&self) -> Vec<&ConsoleEntry> {
        self.entries.iter().collect()
    }

    #[cfg(test)]
    /// Get number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[cfg(test)]
    /// Check if output manager is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

}

impl Default for ConsoleOutputManager {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_output_creation() {
        let info = ConsoleOutput::info("Test info");
        assert_eq!(info.message(), "Test info");
        assert_eq!(info.output_type(), "info");

        let success = ConsoleOutput::success("Test success");
        assert_eq!(success.output_type(), "success");
        assert_eq!(success.message(), "Test success");

        let error = ConsoleOutput::error("Test error");
        assert_eq!(error.output_type(), "error");
        assert_eq!(error.message(), "Test error");

    }

    #[test]
    fn test_console_output_display() {
        let info = ConsoleOutput::info("Hello");
        assert_eq!(info.to_string(), "[INFO] Hello");

        let success = ConsoleOutput::success("Done");
        assert_eq!(success.to_string(), "[SUCCESS] Done");

        let error = ConsoleOutput::error("Failed");
        assert_eq!(error.to_string(), "[ERROR] Failed");

        let command = ConsoleOutput::echo("help");
        assert_eq!(command.to_string(), "> help");

        let empty = ConsoleOutput::empty();
        assert_eq!(empty.to_string(), "");
    }

    #[test]
    fn test_console_entry() {
        let output = ConsoleOutput::info("Test message");
        let entry = ConsoleEntry::new(output.clone());
        
        assert_eq!(entry.output, output);
    }

    #[test]
    fn test_console_output_manager() {
        let mut manager = ConsoleOutputManager::new();
        assert!(manager.is_empty());

        manager.add_output(ConsoleOutput::info("First message"));
        manager.add_output(ConsoleOutput::success("Success message"));

        assert_eq!(manager.len(), 2);

        // All messages should be visible
        let visible = manager.entries();
        assert_eq!(visible.len(), 2);
    }

    #[test]
    fn test_output_manager_size_limit() {
        let mut manager = ConsoleOutputManager::with_settings(2);
        
        manager.add_output(ConsoleOutput::info("First"));
        manager.add_output(ConsoleOutput::info("Second"));
        manager.add_output(ConsoleOutput::info("Third")); // Should remove "First"

        assert_eq!(manager.len(), 2);
        let entries = manager.entries();
        assert_eq!(entries[0].output.message(), "Third"); // Most recent first
        assert_eq!(entries[1].output.message(), "Second");
    }

    #[test]
    fn test_output_manager_clear() {
        let mut manager = ConsoleOutputManager::new();
        manager.add_output(ConsoleOutput::info("Test"));
        assert!(!manager.is_empty());
        
        manager.clear();
        assert!(manager.is_empty());
    }

}