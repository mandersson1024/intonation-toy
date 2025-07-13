// Console Output System
// Handles formatting and display of console command results and messages

use dev_console::ConsoleOutput;

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
            max_entries: 1000,
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
}

impl Default for ConsoleOutputManager {
    fn default() -> Self {
        Self::new()
    }
}