//! # UI Coordinator
//!
//! This module defines the core UICoordinator trait that manages coordination
//! between different UI systems, including future immersive rendering,
//! debug overlays, and event routing.

use std::rc::Rc;
use std::cell::RefCell;
use crate::modules::application_core::PriorityEventBus;

/// UI state representation for coordination between systems
#[derive(Debug, Clone)]
pub struct UIState {
    /// Current UI mode (e.g., "debug", "immersive", "mixed")
    pub mode: String,
    /// Generic state data as key-value pairs
    pub data: std::collections::HashMap<String, String>,
    /// Timestamp of last state update
    pub timestamp: u64,
}

impl UIState {
    /// Create a new UI state with default values
    pub fn new() -> Self {
        Self {
            mode: "default".to_string(),
            data: std::collections::HashMap::new(),
            timestamp: 0,
        }
    }

    /// Update the UI mode
    pub fn set_mode(&mut self, mode: impl Into<String>) {
        self.mode = mode.into();
        self.update_timestamp();
    }

    /// Set a data value
    pub fn set_data(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.data.insert(key.into(), value.into());
        self.update_timestamp();
    }

    /// Get a data value
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    /// Update timestamp to current time
    fn update_timestamp(&mut self) {
        // In a real implementation, this would use a proper time source
        // For now, we'll use a simple counter approach
        self.timestamp = self.timestamp.wrapping_add(1);
    }
}

impl Default for UIState {
    fn default() -> Self {
        Self::new()
    }
}

/// Debug state representation for debug overlay coordination
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct DebugState {
    /// Whether debug overlay is visible
    pub overlay_visible: bool,
    /// Debug metrics and information
    pub metrics: std::collections::HashMap<String, f64>,
    /// Debug messages
    pub messages: Vec<String>,
    /// Timestamp of last debug update
    pub timestamp: u64,
}

#[cfg(debug_assertions)]
impl DebugState {
    /// Create a new debug state with default values
    pub fn new() -> Self {
        Self {
            overlay_visible: false,
            metrics: std::collections::HashMap::new(),
            messages: Vec::new(),
            timestamp: 0,
        }
    }

    /// Set overlay visibility
    pub fn set_overlay_visible(&mut self, visible: bool) {
        self.overlay_visible = visible;
        self.update_timestamp();
    }

    /// Add a metric value
    pub fn set_metric(&mut self, key: impl Into<String>, value: f64) {
        self.metrics.insert(key.into(), value);
        self.update_timestamp();
    }

    /// Add a debug message
    pub fn add_message(&mut self, message: impl Into<String>) {
        self.messages.push(message.into());
        // Keep only recent messages to prevent memory growth
        if self.messages.len() > 100 {
            self.messages.remove(0);
        }
        self.update_timestamp();
    }

    /// Update timestamp to current time
    fn update_timestamp(&mut self) {
        self.timestamp = self.timestamp.wrapping_add(1);
    }
}

#[cfg(debug_assertions)]
impl Default for DebugState {
    fn default() -> Self {
        Self::new()
    }
}

/// UI coordination errors
#[derive(Debug, Clone)]
pub enum UIError {
    /// Rendering operation failed
    RenderError(String),
    /// Event handling failed
    EventError(String),
    /// State synchronization failed
    StateError(String),
    /// Initialization failed
    InitializationError(String),
    /// Generic coordination error
    CoordinationError(String),
}

impl std::fmt::Display for UIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UIError::RenderError(msg) => write!(f, "Render error: {}", msg),
            UIError::EventError(msg) => write!(f, "Event error: {}", msg),
            UIError::StateError(msg) => write!(f, "State error: {}", msg),
            UIError::InitializationError(msg) => write!(f, "Initialization error: {}", msg),
            UIError::CoordinationError(msg) => write!(f, "Coordination error: {}", msg),
        }
    }
}

impl std::error::Error for UIError {}

/// Core trait for UI coordination between different rendering systems
pub trait UICoordinator {
    /// Render the immersive UI with the given state
    fn render_immersive_ui(&mut self, state: &UIState) -> Result<(), UIError>;
    
    /// Render debug overlay (debug builds only)
    #[cfg(debug_assertions)]
    fn render_debug_overlay(&mut self, debug_state: &DebugState) -> Result<(), UIError>;
    
    /// Toggle debug overlay visibility (debug builds only)
    #[cfg(debug_assertions)]
    fn toggle_debug_overlay(&mut self, visible: bool) -> Result<(), UIError>;
    
    /// Handle UI event
    fn handle_ui_event(&mut self, event: crate::modules::presentation_layer::UIEvent) -> Result<(), UIError>;
    
    /// Update UI state
    fn update_state(&mut self, state: UIState) -> Result<(), UIError>;
    
    /// Initialize the UI coordinator
    fn initialize(&mut self) -> Result<(), UIError>;
    
    /// Start UI coordination
    fn start(&mut self) -> Result<(), UIError>;
    
    /// Stop UI coordination
    fn stop(&mut self) -> Result<(), UIError>;
    
    /// Shutdown UI coordinator
    fn shutdown(&mut self) -> Result<(), UIError>;
    
    /// Connect event bus for coordination
    fn connect_event_bus(&mut self, event_bus: Rc<RefCell<PriorityEventBus>>) -> Result<(), UIError>;
    
    /// Get current UI state
    fn get_current_state(&self) -> &UIState;
    
    /// Get performance metrics for coordination overhead
    fn get_performance_metrics(&self) -> std::collections::HashMap<String, f64>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_state_creation() {
        let state = UIState::new();
        assert_eq!(state.mode, "default");
        assert!(state.data.is_empty());
        assert_eq!(state.timestamp, 0);
    }

    #[test]
    fn test_ui_state_updates() {
        let mut state = UIState::new();
        let initial_timestamp = state.timestamp;
        
        state.set_mode("debug");
        assert_eq!(state.mode, "debug");
        assert!(state.timestamp > initial_timestamp);
        
        state.set_data("key1", "value1");
        assert_eq!(state.get_data("key1"), Some(&"value1".to_string()));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_debug_state_creation() {
        let debug_state = DebugState::new();
        assert!(!debug_state.overlay_visible);
        assert!(debug_state.metrics.is_empty());
        assert!(debug_state.messages.is_empty());
        assert_eq!(debug_state.timestamp, 0);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_debug_state_updates() {
        let mut debug_state = DebugState::new();
        let initial_timestamp = debug_state.timestamp;
        
        debug_state.set_overlay_visible(true);
        assert!(debug_state.overlay_visible);
        assert!(debug_state.timestamp > initial_timestamp);
        
        debug_state.set_metric("latency", 1.5);
        assert_eq!(debug_state.metrics.get("latency"), Some(&1.5));
        
        debug_state.add_message("Test message");
        assert_eq!(debug_state.messages.len(), 1);
        assert_eq!(debug_state.messages[0], "Test message");
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_debug_message_limit() {
        let mut debug_state = DebugState::new();
        
        // Add more than 100 messages
        for i in 0..105 {
            debug_state.add_message(format!("Message {}", i));
        }
        
        // Should be limited to 100 messages
        assert_eq!(debug_state.messages.len(), 100);
        // First message should be "Message 5" (oldest removed)
        assert_eq!(debug_state.messages[0], "Message 5");
        // Last message should be "Message 104"
        assert_eq!(debug_state.messages[99], "Message 104");
    }
}