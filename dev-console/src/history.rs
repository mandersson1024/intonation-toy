use serde::{Serialize, Deserialize};

const MAX_HISTORY_SIZE: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleHistory {
    commands: Vec<String>,
    current_position: Option<usize>,
}

impl Default for ConsoleHistory {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            current_position: None,
        }
    }
}

impl ConsoleHistory {
    pub fn add_command(&mut self, command: String) {
        if command.trim().is_empty() || self.commands.first().map_or(false, |last| last == &command) {
            return;
        }

        self.commands.insert(0, command);
        if self.commands.len() > MAX_HISTORY_SIZE {
            self.commands.truncate(MAX_HISTORY_SIZE);
        }
        self.current_position = None;
    }

    pub fn navigate_previous(&mut self) -> Option<&str> {
        if self.commands.is_empty() {
            return None;
        }

        let new_pos = match self.current_position {
            None => 0,
            Some(pos) => pos + 1,
        };

        if new_pos < self.commands.len() {
            self.current_position = Some(new_pos);
            Some(&self.commands[new_pos])
        } else {
            None
        }
    }

    pub fn navigate_next(&mut self) -> Option<&str> {
        match self.current_position {
            None => None,
            Some(0) => {
                self.current_position = None;
                Some("")
            }
            Some(pos) => {
                let new_pos = pos - 1;
                self.current_position = Some(new_pos);
                Some(&self.commands[new_pos])
            }
        }
    }

    pub fn reset_navigation(&mut self) {
        self.current_position = None;
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}