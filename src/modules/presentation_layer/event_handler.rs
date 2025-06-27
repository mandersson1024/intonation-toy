//! # Event Handler
//!
//! This module provides event routing infrastructure for UI coordination
//! between different UI systems, including immersive rendering and debug overlays.

use crate::modules::presentation_layer::{UIState, UserInteraction};

/// UI Event types for coordination between systems
#[derive(Debug, Clone)]
pub enum UIEvent {
    /// User interaction event
    UserInteraction(UserInteraction),
    /// UI state update event
    StateUpdate(UIState),
    /// Debug overlay toggle event (debug builds only)
    #[cfg(debug_assertions)]
    DebugToggle(bool),
    /// Canvas resize event
    CanvasResize { width: u32, height: u32 },
    /// Performance monitoring event
    PerformanceUpdate { metrics: std::collections::HashMap<String, f64> },
    /// Error event
    Error { message: String, severity: ErrorSeverity },
    /// No-operation event (for testing)
    NoOp,
}

/// Error severity levels for UI events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Low severity - minor issues that don't affect functionality
    Low,
    /// Medium severity - issues that may impact user experience
    Medium,
    /// High severity - critical issues that prevent normal operation
    High,
    /// Critical severity - system-threatening issues
    Critical,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSeverity::Low => write!(f, "Low"),
            ErrorSeverity::Medium => write!(f, "Medium"),
            ErrorSeverity::High => write!(f, "High"),
            ErrorSeverity::Critical => write!(f, "Critical"),
        }
    }
}

/// Event priority for routing and processing order
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    /// Low priority - background events, logging
    Low = 0,
    /// Normal priority - standard UI events
    Normal = 1,
    /// High priority - user interactions, urgent updates
    High = 2,
    /// Critical priority - errors, system events
    Critical = 3,
}

impl UIEvent {
    /// Get the priority of this event for processing order
    pub fn priority(&self) -> EventPriority {
        match self {
            UIEvent::UserInteraction(_) => EventPriority::High,
            UIEvent::StateUpdate(_) => EventPriority::Normal,
            #[cfg(debug_assertions)]
            UIEvent::DebugToggle(_) => EventPriority::Normal,
            UIEvent::CanvasResize { .. } => EventPriority::High,
            UIEvent::PerformanceUpdate { .. } => EventPriority::Low,
            UIEvent::Error { severity, .. } => match severity {
                ErrorSeverity::Low => EventPriority::Low,
                ErrorSeverity::Medium => EventPriority::Normal,
                ErrorSeverity::High => EventPriority::High,
                ErrorSeverity::Critical => EventPriority::Critical,
            },
            UIEvent::NoOp => EventPriority::Low,
        }
    }

    /// Check if this event should be processed immediately
    pub fn is_urgent(&self) -> bool {
        self.priority() >= EventPriority::High
    }

    /// Get a human-readable description of the event
    pub fn description(&self) -> String {
        match self {
            UIEvent::UserInteraction(interaction) => {
                match interaction {
                    UserInteraction::Click { x, y } => format!("Click at ({}, {})", x, y),
                    UserInteraction::Drag { x1, y1, x2, y2 } => {
                        format!("Drag from ({}, {}) to ({}, {})", x1, y1, x2, y2)
                    },
                    UserInteraction::KeyPress { key, modifiers } => {
                        if modifiers.is_empty() {
                            format!("Key press: {}", key)
                        } else {
                            format!("Key press: {} + {}", modifiers.join("+"), key)
                        }
                    },
                    UserInteraction::Scroll { delta_x, delta_y, zoom } => {
                        format!("Scroll: delta({}, {}), zoom: {}", delta_x, delta_y, zoom)
                    },
                    UserInteraction::Hover { x, y } => format!("Hover at ({}, {})", x, y),
                    UserInteraction::Gesture { name, .. } => format!("Gesture: {}", name),
                }
            },
            UIEvent::StateUpdate(state) => format!("State update: mode={}", state.mode),
            #[cfg(debug_assertions)]
            UIEvent::DebugToggle(visible) => format!("Debug toggle: {}", visible),
            UIEvent::CanvasResize { width, height } => {
                format!("Canvas resize: {}x{}", width, height)
            },
            UIEvent::PerformanceUpdate { metrics } => {
                format!("Performance update: {} metrics", metrics.len())
            },
            UIEvent::Error { message, severity } => {
                format!("{} error: {}", severity, message)
            },
            UIEvent::NoOp => "No operation".to_string(),
        }
    }
}

/// Event routing result
#[derive(Debug, Clone)]
pub enum EventResult {
    /// Event was handled successfully
    Handled,
    /// Event was handled with warnings
    HandledWithWarnings(Vec<String>),
    /// Event was not handled (no suitable handler)
    NotHandled,
    /// Event handling failed
    Failed(String),
}

impl EventResult {
    /// Check if the event was successfully handled
    pub fn is_success(&self) -> bool {
        matches!(self, EventResult::Handled | EventResult::HandledWithWarnings(_))
    }

    /// Get warning messages if any
    pub fn warnings(&self) -> Vec<String> {
        match self {
            EventResult::HandledWithWarnings(warnings) => warnings.clone(),
            _ => Vec::new(),
        }
    }

    /// Get error message if failed
    pub fn error_message(&self) -> Option<String> {
        match self {
            EventResult::Failed(msg) => Some(msg.clone()),
            _ => None,
        }
    }
}

/// Trait for components that can handle UI events
pub trait EventHandler {
    /// Handle a UI event and return the result
    fn handle_event(&mut self, event: UIEvent) -> EventResult;
    
    /// Check if this handler can process the given event type
    fn can_handle(&self, event: &UIEvent) -> bool;
    
    /// Get the priority of this handler for event routing
    fn handler_priority(&self) -> EventPriority {
        EventPriority::Normal
    }
    
    /// Get handler name for debugging
    fn handler_name(&self) -> &'static str;
}

/// Event router for coordinating events between UI systems
pub struct EventRouter {
    handlers: Vec<Box<dyn EventHandler>>,
    event_log: Vec<(UIEvent, EventResult, u64)>, // event, result, timestamp
    performance_metrics: std::collections::HashMap<String, f64>,
}

impl EventRouter {
    /// Create a new event router
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            event_log: Vec::new(),
            performance_metrics: std::collections::HashMap::new(),
        }
    }

    /// Register an event handler
    pub fn register_handler(&mut self, handler: Box<dyn EventHandler>) {
        self.handlers.push(handler);
        // Sort handlers by priority (highest first)
        self.handlers.sort_by(|a, b| b.handler_priority().cmp(&a.handler_priority()));
    }

    /// Route an event to appropriate handlers
    pub fn route_event(&mut self, event: UIEvent) -> Vec<EventResult> {
        let start_time = self.get_timestamp();
        let mut results = Vec::new();
        let is_broadcast = self.is_broadcast_event(&event);
        
        // Find handlers that can process this event
        for handler in &mut self.handlers {
            if handler.can_handle(&event) {
                let result = handler.handle_event(event.clone());
                let result_success = result.is_success();
                
                // Log the event handling
                self.event_log.push((event.clone(), result.clone(), start_time));
                results.push(result);
                
                // Stop routing if a handler successfully processed the event
                // and it's not a broadcast-type event
                if result_success && !is_broadcast {
                    break;
                }
            }
        }
        
        // If no handler processed the event, log it as not handled
        if results.is_empty() {
            let result = EventResult::NotHandled;
            self.event_log.push((event, result.clone(), start_time));
            results.push(result);
        }
        
        // Update performance metrics
        let processing_time = self.get_timestamp() - start_time;
        self.performance_metrics.insert("last_event_processing_time_us".to_string(), processing_time as f64);
        
        // Keep event log limited in size
        if self.event_log.len() > 1000 {
            self.event_log.drain(0..100); // Remove oldest 100 events
        }
        
        results
    }

    /// Check if an event should be broadcast to all handlers
    fn is_broadcast_event(&self, event: &UIEvent) -> bool {
        matches!(event, 
            UIEvent::PerformanceUpdate { .. } | 
            UIEvent::Error { .. } |
            UIEvent::CanvasResize { .. }
        )
    }

    /// Get current timestamp in microseconds
    fn get_timestamp(&self) -> u64 {
        // In a real implementation, this would use a proper time source
        // For now, we'll use a simple counter approach
        self.event_log.len() as u64
    }

    /// Get recent event log entries
    pub fn get_recent_events(&self, count: usize) -> Vec<&(UIEvent, EventResult, u64)> {
        let start = if self.event_log.len() > count {
            self.event_log.len() - count
        } else {
            0
        };
        self.event_log[start..].iter().collect()
    }

    /// Get performance metrics for event routing
    pub fn get_performance_metrics(&self) -> &std::collections::HashMap<String, f64> {
        &self.performance_metrics
    }

    /// Clear event log and reset metrics
    pub fn reset(&mut self) {
        self.event_log.clear();
        self.performance_metrics.clear();
    }

    /// Get number of registered handlers
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }
}

impl Default for EventRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock event handler for testing
    struct MockEventHandler {
        name: &'static str,
        handled_events: Vec<UIEvent>,
        can_handle_fn: fn(&UIEvent) -> bool,
    }

    impl MockEventHandler {
        fn new(name: &'static str, can_handle_fn: fn(&UIEvent) -> bool) -> Self {
            Self {
                name,
                handled_events: Vec::new(),
                can_handle_fn,
            }
        }

        fn get_handled_events(&self) -> &[UIEvent] {
            &self.handled_events
        }
    }

    impl EventHandler for MockEventHandler {
        fn handle_event(&mut self, event: UIEvent) -> EventResult {
            self.handled_events.push(event);
            EventResult::Handled
        }

        fn can_handle(&self, event: &UIEvent) -> bool {
            (self.can_handle_fn)(event)
        }

        fn handler_name(&self) -> &'static str {
            self.name
        }
    }

    #[test]
    fn test_ui_event_priority() {
        let click = UIEvent::UserInteraction(UserInteraction::Click { x: 0.0, y: 0.0 });
        assert_eq!(click.priority(), EventPriority::High);
        assert!(click.is_urgent());

        let state_update = UIEvent::StateUpdate(UIState::new());
        assert_eq!(state_update.priority(), EventPriority::Normal);
        assert!(!state_update.is_urgent());

        let error = UIEvent::Error { 
            message: "test".to_string(), 
            severity: ErrorSeverity::Critical 
        };
        assert_eq!(error.priority(), EventPriority::Critical);
        assert!(error.is_urgent());
    }

    #[test]
    fn test_event_descriptions() {
        let click = UIEvent::UserInteraction(UserInteraction::Click { x: 10.0, y: 20.0 });
        assert_eq!(click.description(), "Click at (10, 20)");

        let resize = UIEvent::CanvasResize { width: 800, height: 600 };
        assert_eq!(resize.description(), "Canvas resize: 800x600");

        let error = UIEvent::Error { 
            message: "Something went wrong".to_string(), 
            severity: ErrorSeverity::High 
        };
        assert_eq!(error.description(), "High error: Something went wrong");
    }

    #[test]
    fn test_event_result() {
        let handled = EventResult::Handled;
        assert!(handled.is_success());
        assert!(handled.warnings().is_empty());
        assert!(handled.error_message().is_none());

        let warnings = vec!["Warning 1".to_string(), "Warning 2".to_string()];
        let handled_with_warnings = EventResult::HandledWithWarnings(warnings.clone());
        assert!(handled_with_warnings.is_success());
        assert_eq!(handled_with_warnings.warnings(), warnings);

        let failed = EventResult::Failed("Error message".to_string());
        assert!(!failed.is_success());
        assert_eq!(failed.error_message(), Some("Error message".to_string()));
    }

    #[test]
    fn test_event_router_creation() {
        let router = EventRouter::new();
        assert_eq!(router.handler_count(), 0);
        assert!(router.get_performance_metrics().is_empty());
    }

    #[test]
    fn test_event_router_handler_registration() {
        let mut router = EventRouter::new();
        
        let handler = MockEventHandler::new("test", |_| true);
        router.register_handler(Box::new(handler));
        
        assert_eq!(router.handler_count(), 1);
    }

    #[test]
    fn test_event_routing() {
        let mut router = EventRouter::new();
        
        let handler = MockEventHandler::new("test", |event| {
            matches!(event, UIEvent::UserInteraction(_))
        });
        router.register_handler(Box::new(handler));
        
        let click = UIEvent::UserInteraction(UserInteraction::Click { x: 0.0, y: 0.0 });
        let results = router.route_event(click);
        
        assert_eq!(results.len(), 1);
        assert!(results[0].is_success());
    }

    #[test]
    fn test_event_routing_no_handler() {
        let mut router = EventRouter::new();
        
        let click = UIEvent::UserInteraction(UserInteraction::Click { x: 0.0, y: 0.0 });
        let results = router.route_event(click);
        
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], EventResult::NotHandled));
    }

    #[test]
    fn test_event_log() {
        let mut router = EventRouter::new();
        
        let handler = MockEventHandler::new("test", |_| true);
        router.register_handler(Box::new(handler));
        
        let click = UIEvent::UserInteraction(UserInteraction::Click { x: 0.0, y: 0.0 });
        router.route_event(click);
        
        let recent_events = router.get_recent_events(10);
        assert_eq!(recent_events.len(), 1);
    }

    #[test]
    fn test_broadcast_events() {
        let mut router = EventRouter::new();
        
        // Register two handlers that can handle the same broadcast event
        let handler1 = MockEventHandler::new("handler1", |event| {
            matches!(event, UIEvent::PerformanceUpdate { .. })
        });
        let handler2 = MockEventHandler::new("handler2", |event| {
            matches!(event, UIEvent::PerformanceUpdate { .. })
        });
        
        router.register_handler(Box::new(handler1));
        router.register_handler(Box::new(handler2));
        
        let perf_event = UIEvent::PerformanceUpdate { 
            metrics: std::collections::HashMap::new() 
        };
        let results = router.route_event(perf_event);
        
        // Both handlers should process the broadcast event
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_success()));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_debug_toggle_event() {
        let debug_event = UIEvent::DebugToggle(true);
        assert_eq!(debug_event.priority(), EventPriority::Normal);
        assert_eq!(debug_event.description(), "Debug toggle: true");
    }

    #[test]
    fn test_error_severity_display() {
        assert_eq!(ErrorSeverity::Low.to_string(), "Low");
        assert_eq!(ErrorSeverity::Medium.to_string(), "Medium");
        assert_eq!(ErrorSeverity::High.to_string(), "High");
        assert_eq!(ErrorSeverity::Critical.to_string(), "Critical");
    }
}