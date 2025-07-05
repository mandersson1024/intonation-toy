// Input Handler - Keyboard and input management for debug console
//
// This module handles keyboard navigation and input processing for the debug console.
// It provides utilities for command history navigation and input validation.

use web_sys::KeyboardEvent;

/// Input handler for debug console keyboard events
pub struct InputHandler;

impl InputHandler {
    /// Handle keyboard events for the debug console
    pub fn handle_keyboard_event(event: &KeyboardEvent) -> Option<ConsoleKeyboardAction> {
        match event.key().as_str() {
            "Enter" => Some(ConsoleKeyboardAction::ExecuteCommand),
            "ArrowUp" => Some(ConsoleKeyboardAction::NavigateHistoryPrevious),
            "ArrowDown" => Some(ConsoleKeyboardAction::NavigateHistoryNext),
            "Escape" => Some(ConsoleKeyboardAction::ToggleVisibility),
            "Tab" => Some(ConsoleKeyboardAction::AutoComplete),
            _ => None,
        }
    }

    /// Validate command input
    pub fn validate_command(command: &str) -> bool {
        !command.trim().is_empty()
    }

    /// Sanitize command input
    pub fn sanitize_command(command: &str) -> String {
        command.trim().to_string()
    }
}

/// Actions that can be triggered by keyboard events
#[derive(Debug, PartialEq)]
pub enum ConsoleKeyboardAction {
    ExecuteCommand,
    NavigateHistoryPrevious,
    NavigateHistoryNext,
    ToggleVisibility,
    AutoComplete,
}