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

    /// Get the message content as a string
    pub fn message(&self) -> &str {
        match self {
            Self::Info(msg) | Self::Success(msg) | Self::Warning(msg) 
            | Self::Error(msg) | Self::Echo(msg) => msg,
            Self::Empty => "",
        }
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


    /// Get HTML-safe message content (escapes special characters)
    pub fn html_safe_message(&self) -> String {
        self.message()
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
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

/// Console output entry with timestamp and formatting information
#[derive(Debug, Clone)]
pub struct ConsoleEntry {
    /// The output message
    pub output: ConsoleOutput,
    /// Timestamp when the entry was created (milliseconds since epoch)
    pub timestamp: u64,
    /// Unique identifier for this entry
    pub id: u64,
}

impl ConsoleEntry {
    /// Create a new console entry with current timestamp
    pub fn new(output: ConsoleOutput) -> Self {
        Self {
            output,
            timestamp: Self::current_timestamp(),
            id: Self::generate_id(),
        }
    }

    /// Create a new console entry with explicit timestamp
    pub fn with_timestamp(output: ConsoleOutput, timestamp: u64) -> Self {
        Self {
            output,
            timestamp,
            id: Self::generate_id(),
        }
    }

    /// Get current timestamp in milliseconds since epoch
    fn current_timestamp() -> u64 {
        #[cfg(target_arch = "wasm32")]
        {
            // Use browser's Date API for WASM builds
            js_sys::Date::now() as u64
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Use SystemTime for native builds (tests)
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        }
    }

    /// Generate a unique ID for this entry
    fn generate_id() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    /// Format timestamp as HH:MM:SS
    pub fn format_time(&self) -> String {
        let seconds = (self.timestamp / 1000) % 86400; // seconds in a day
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    }

    /// Format timestamp as HH:MM:SS.mmm (with milliseconds)
    pub fn format_time_detailed(&self) -> String {
        let millis = self.timestamp % 1000;
        format!("{}.{:03}", self.format_time(), millis)
    }
}

/// Console output manager that handles multiple output entries
#[derive(Debug, Clone)]
pub struct ConsoleOutputManager {
    /// List of console entries (most recent first)
    entries: Vec<ConsoleEntry>,
    /// Maximum number of entries to store
    max_entries: usize,
    /// Whether to show timestamps
    show_timestamps: bool,
}

impl ConsoleOutputManager {
    /// Create a new output manager
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 1000, // Same as history limit
            show_timestamps: true,
        }
    }

    /// Create a new output manager with custom settings
    pub fn with_settings(max_entries: usize, show_timestamps: bool) -> Self {
        Self {
            entries: Vec::new(),
            max_entries: max_entries.max(1),
            show_timestamps,
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

    /// Add multiple output entries
    pub fn add_outputs(&mut self, outputs: Vec<ConsoleOutput>) {
        for output in outputs {
            self.add_output(output);
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

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if output manager is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }


    /// Toggle timestamp visibility
    pub fn toggle_timestamps(&mut self) -> bool {
        self.show_timestamps = !self.show_timestamps;
        self.show_timestamps
    }

    /// Set timestamp visibility
    pub fn set_show_timestamps(&mut self, show: bool) {
        self.show_timestamps = show;
    }

    /// Get timestamp visibility setting
    pub fn show_timestamps(&self) -> bool {
        self.show_timestamps
    }

    /// Get maximum entries limit
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    /// Format output for display (with optional timestamps)
    pub fn format_entry(&self, entry: &ConsoleEntry) -> String {
        if self.show_timestamps {
            format!("[{}] {}", entry.format_time(), entry.output)
        } else {
            entry.output.to_string()
        }
    }

    /// Generate CSS class for an entry
    pub fn entry_css_class(&self, entry: &ConsoleEntry) -> String {
        format!("console-output console-{}", entry.output.output_type())
    }
}

impl Default for ConsoleOutputManager {
    fn default() -> Self {
        Self::new()
    }
}

/// CSS styles for console output (embedded for development console)
pub const CONSOLE_OUTPUT_CSS: &str = r#"
.console-output {
    font-family: 'Courier New', monospace;
    font-size: 12px;
    line-height: 1.4;
    margin: 2px 0;
    padding: 2px 4px;
    white-space: pre-wrap;
    word-wrap: break-word;
}

.console-info {
    color: #ffffff;
    background-color: rgba(255, 255, 255, 0.05);
}

.console-success {
    color: #4ade80;
    background-color: rgba(74, 222, 128, 0.1);
}

.console-warning {
    color: #fbbf24;
    background-color: rgba(251, 191, 36, 0.1);
}

.console-error {
    color: #f87171;
    background-color: rgba(248, 113, 113, 0.1);
}


.console-command {
    color: #60a5fa;
    background-color: rgba(96, 165, 250, 0.1);
    font-weight: bold;
}

.console-empty {
    height: 16.8px;
}

.console-output-container {
    max-height: 300px;
    overflow-y: auto;
    background-color: #111827;
    border: 1px solid #374151;
    border-radius: 4px;
    padding: 8px;
    scrollbar-width: thin;
    scrollbar-color: #4b5563 #1f2937;
}

.console-output-container::-webkit-scrollbar {
    width: 8px;
}

.console-output-container::-webkit-scrollbar-track {
    background: #1f2937;
}

.console-output-container::-webkit-scrollbar-thumb {
    background: #4b5563;
    border-radius: 4px;
}

.console-output-container::-webkit-scrollbar-thumb:hover {
    background: #6b7280;
}
"#;

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
    fn test_html_safe_message() {
        let output = ConsoleOutput::info("Test <script>alert('xss')</script>");
        let safe = output.html_safe_message();
        assert!(safe.contains("&lt;script&gt;"));
        assert!(safe.contains("&#x27;"));
    }

    #[test]
    fn test_console_entry() {
        let output = ConsoleOutput::info("Test message");
        let entry = ConsoleEntry::new(output.clone());
        
        assert_eq!(entry.output, output);
        assert!(entry.timestamp > 0);
        assert!(entry.id > 0);

        let time_str = entry.format_time();
        assert!(time_str.contains(':'));
        assert_eq!(time_str.len(), 8); // HH:MM:SS format

        let detailed_time = entry.format_time_detailed();
        assert!(detailed_time.contains('.'));
        assert_eq!(detailed_time.len(), 12); // HH:MM:SS.mmm format
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
        let mut manager = ConsoleOutputManager::with_settings(2, true);
        
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

    #[test]
    fn test_output_manager_formatting() {
        let manager = ConsoleOutputManager::new();
        let entry = ConsoleEntry::new(ConsoleOutput::info("Test"));
        
        let formatted = manager.format_entry(&entry);
        assert!(formatted.contains("[INFO] Test"));
        assert!(formatted.contains(':')); // Timestamp should be included

        let css_class = manager.entry_css_class(&entry);
        assert_eq!(css_class, "console-output console-info");
    }

    #[test]
    fn test_output_manager_toggles() {
        let mut manager = ConsoleOutputManager::new();
        
        // Test timestamp toggle
        assert!(manager.show_timestamps());
        let result = manager.toggle_timestamps();
        assert!(!result);
        assert!(!manager.show_timestamps());
    }

    #[test]
    fn test_console_entry_unique_ids() {
        let entry1 = ConsoleEntry::new(ConsoleOutput::info("First"));
        let entry2 = ConsoleEntry::new(ConsoleOutput::info("Second"));
        
        assert_ne!(entry1.id, entry2.id);
        assert!(entry2.id > entry1.id); // IDs should be increasing
    }
}