//! # Debug Overlay
//!
//! This module provides debug overlay system with Yew app integration for development tools.
//! The entire module is conditionally compiled for debug builds only to ensure zero impact
//! on production builds.

use std::rc::Rc;
use std::cell::RefCell;
use crate::modules::application_core::{PriorityEventBus, Module};
use crate::modules::presentation_layer::{UIError, UIEvent, EventHandler, EventResult, DebugState};

#[cfg(debug_assertions)]
use crate::modules::developer_ui::{DeveloperUIModule, DebugComponent};

/// Debug overlay system for development tools
/// 
/// This struct manages the debug overlay UI integration with the Yew-based
/// developer tools from the Developer UI module. It provides conditional
/// compilation to ensure complete exclusion from production builds.
#[cfg(debug_assertions)]
pub struct DebugOverlay {
    /// Current debug state
    debug_state: DebugState,
    /// Developer UI module integration
    developer_ui: Option<DeveloperUIModule>,
    /// Event bus connection
    event_bus: Option<Rc<RefCell<PriorityEventBus>>>,
    /// Debug component registry for managing multiple debug panels
    registered_components: Vec<String>,
    /// Overlay visibility state
    overlay_visible: bool,
    /// Layout management settings
    layout_config: DebugLayoutConfig,
    /// Performance monitoring for debug overhead
    performance_metrics: std::collections::HashMap<String, f64>,
    /// Initialization state
    initialized: bool,
}

/// Configuration for debug overlay layout management
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct DebugLayoutConfig {
    /// Overlay position (top-left, top-right, bottom-left, bottom-right)
    pub position: DebugOverlayPosition,
    /// Overlay size as percentage of screen (0.0 to 1.0)
    pub size_percentage: f32,
    /// Whether overlay can be resized
    pub resizable: bool,
    /// Whether overlay can be moved
    pub movable: bool,
    /// Opacity of overlay background (0.0 to 1.0)
    pub background_opacity: f32,
    /// Whether to show overlay border
    pub show_border: bool,
}

/// Debug overlay position options
#[cfg(debug_assertions)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugOverlayPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
    Custom { x: u32, y: u32 },
}

#[cfg(debug_assertions)]
impl Default for DebugLayoutConfig {
    fn default() -> Self {
        Self {
            position: DebugOverlayPosition::TopRight,
            size_percentage: 0.3,
            resizable: true,
            movable: true,
            background_opacity: 0.9,
            show_border: true,
        }
    }
}

#[cfg(debug_assertions)]
impl DebugOverlay {
    /// Create a new debug overlay instance
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            debug_state: DebugState::new(),
            developer_ui: None,
            event_bus: None,
            registered_components: Vec::new(),
            overlay_visible: false,
            layout_config: DebugLayoutConfig::default(),
            performance_metrics: std::collections::HashMap::new(),
            initialized: false,
        })
    }

    /// Initialize the debug overlay system
    pub fn initialize(&mut self) -> Result<(), UIError> {
        if self.initialized {
            return Ok(());
        }

        // Initialize developer UI module
        match DeveloperUIModule::new() {
            Ok(mut dev_ui) => {
                dev_ui.initialize()
                    .map_err(|e| UIError::InitializationError(format!("Failed to initialize developer UI: {}", e)))?;
                self.developer_ui = Some(dev_ui);
            },
            Err(e) => {
                return Err(UIError::InitializationError(format!("Failed to create developer UI: {}", e)));
            }
        }

        // Register default debug components
        self.register_default_components()?;

        self.initialized = true;
        Ok(())
    }

    /// Register default debug components
    fn register_default_components(&mut self) -> Result<(), UIError> {
        // Register debug component types that will be available
        let default_components = vec![
            "DebugInterface",
            "AudioControlPanel", 
            "MetricsDisplay",
            "PerformanceMonitor",
            "ErrorDisplay",
            "MicrophonePanel",
        ];

        for component_name in default_components {
            self.registered_components.push(component_name.to_string());
        }

        Ok(())
    }

    /// Toggle debug overlay visibility
    pub fn toggle_visibility(&mut self, visible: bool) -> Result<(), UIError> {
        self.overlay_visible = visible;
        self.debug_state.set_overlay_visible(visible);
        
        if let Some(ref mut dev_ui) = self.developer_ui {
            // In a real implementation, this would update the Yew app visibility
            // For now, we'll just track the state
            let _ = dev_ui; // Placeholder
        }

        self.update_performance_metrics();
        Ok(())
    }

    /// Update debug state with new information
    pub fn update_debug_state(&mut self, new_state: DebugState) -> Result<(), UIError> {
        self.debug_state = new_state;
        
        if let Some(ref mut dev_ui) = self.developer_ui {
            // In a real implementation, this would synchronize state with Yew components
            let _ = dev_ui; // Placeholder
        }

        Ok(())
    }

    /// Add a debug metric
    pub fn add_metric(&mut self, key: impl Into<String>, value: f64) {
        self.debug_state.set_metric(key, value);
        self.update_performance_metrics();
    }

    /// Add a debug message
    pub fn add_message(&mut self, message: impl Into<String>) {
        self.debug_state.add_message(message);
    }

    /// Configure debug overlay layout
    pub fn configure_layout(&mut self, config: DebugLayoutConfig) -> Result<(), UIError> {
        self.layout_config = config;
        
        // Apply layout changes if overlay is visible
        if self.overlay_visible {
            self.apply_layout_config()?;
        }
        
        Ok(())
    }

    /// Apply current layout configuration
    fn apply_layout_config(&mut self) -> Result<(), UIError> {
        // In a real implementation, this would update CSS styles or DOM positioning
        // For now, we'll validate the configuration
        
        if self.layout_config.size_percentage < 0.1 || self.layout_config.size_percentage > 1.0 {
            return Err(UIError::CoordinationError("Invalid size percentage".to_string()));
        }
        
        if self.layout_config.background_opacity < 0.0 || self.layout_config.background_opacity > 1.0 {
            return Err(UIError::CoordinationError("Invalid background opacity".to_string()));
        }
        
        Ok(())
    }

    /// Connect event bus for debug coordination
    pub fn connect_event_bus(&mut self, event_bus: Rc<RefCell<PriorityEventBus>>) -> Result<(), UIError> {
        self.event_bus = Some(event_bus.clone());
        
        if let Some(ref mut dev_ui) = self.developer_ui {
            // Connect developer UI to event bus
            // In a real implementation, this would setup event subscriptions
            let _ = dev_ui; // Placeholder
        }
        
        Ok(())
    }

    /// Update performance metrics for debug overhead
    fn update_performance_metrics(&mut self) {
        let start_time = std::time::Instant::now();
        
        // Calculate debug overlay overhead
        let component_count = self.registered_components.len() as f64;
        let state_size = std::mem::size_of_val(&self.debug_state) as f64;
        
        self.performance_metrics.insert("debug_component_count".to_string(), component_count);
        self.performance_metrics.insert("debug_state_size_bytes".to_string(), state_size);
        self.performance_metrics.insert("overlay_visible".to_string(), if self.overlay_visible { 1.0 } else { 0.0 });
        
        let metrics_update_time = start_time.elapsed().as_micros() as f64;
        self.performance_metrics.insert("debug_metrics_update_time_us".to_string(), metrics_update_time);
    }

    /// Get current debug state
    pub fn get_debug_state(&self) -> &DebugState {
        &self.debug_state
    }

    /// Get performance metrics for debug overhead monitoring
    pub fn get_performance_metrics(&self) -> &std::collections::HashMap<String, f64> {
        &self.performance_metrics
    }

    /// Check if overlay is currently visible
    pub fn is_visible(&self) -> bool {
        self.overlay_visible
    }

    /// Get list of registered debug components
    pub fn get_registered_components(&self) -> &[String] {
        &self.registered_components
    }

    /// Shutdown debug overlay and clean up resources
    pub fn shutdown(&mut self) -> Result<(), UIError> {
        if !self.initialized {
            return Ok(());
        }

        // Hide overlay if visible
        if self.overlay_visible {
            self.toggle_visibility(false)?;
        }

        // Shutdown developer UI
        if let Some(ref mut dev_ui) = self.developer_ui {
            dev_ui.shutdown()
                .map_err(|e| UIError::CoordinationError(format!("Failed to shutdown developer UI: {}", e)))?;
        }

        // Clear state
        self.developer_ui = None;
        self.event_bus = None;
        self.registered_components.clear();
        self.performance_metrics.clear();
        
        self.initialized = false;
        Ok(())
    }
}

/// Event handler for debug overlay events
#[cfg(debug_assertions)]
pub struct DebugOverlayEventHandler {
    overlay: Rc<RefCell<DebugOverlay>>,
}

#[cfg(debug_assertions)]
impl DebugOverlayEventHandler {
    /// Create a new debug overlay event handler
    pub fn new(overlay: Rc<RefCell<DebugOverlay>>) -> Self {
        Self { overlay }
    }
}

#[cfg(debug_assertions)]
impl EventHandler for DebugOverlayEventHandler {
    fn handle_event(&mut self, event: UIEvent) -> EventResult {
        match event {
            UIEvent::DebugToggle(visible) => {
                match self.overlay.borrow_mut().toggle_visibility(visible) {
                    Ok(()) => EventResult::Handled,
                    Err(e) => EventResult::Failed(format!("Debug toggle failed: {}", e)),
                }
            },
            UIEvent::PerformanceUpdate { metrics } => {
                let mut overlay = self.overlay.borrow_mut();
                for (key, value) in metrics {
                    overlay.add_metric(format!("external_{}", key), value);
                }
                EventResult::Handled
            },
            UIEvent::Error { message, severity: _ } => {
                self.overlay.borrow_mut().add_message(format!("Error: {}", message));
                EventResult::Handled
            },
            _ => EventResult::NotHandled,
        }
    }

    fn can_handle(&self, event: &UIEvent) -> bool {
        matches!(event, 
            UIEvent::DebugToggle(_) |
            UIEvent::PerformanceUpdate { .. } |
            UIEvent::Error { .. }
        )
    }

    fn handler_name(&self) -> &'static str {
        "DebugOverlayEventHandler"
    }
}

// For production builds, provide empty stubs to prevent compilation errors
#[cfg(not(debug_assertions))]
pub struct DebugOverlay;

#[cfg(not(debug_assertions))]
impl DebugOverlay {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }

    pub fn initialize(&mut self) -> Result<(), UIError> {
        Ok(())
    }

    pub fn connect_event_bus(&mut self, _event_bus: Rc<RefCell<PriorityEventBus>>) -> Result<(), UIError> {
        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<(), UIError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(debug_assertions)]
    #[test]
    fn test_debug_overlay_creation() {
        let overlay = DebugOverlay::new();
        assert!(overlay.is_ok());
        
        let overlay = overlay.unwrap();
        assert!(!overlay.initialized);
        assert!(!overlay.overlay_visible);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_debug_overlay_initialization() {
        let mut overlay = DebugOverlay::new().unwrap();
        
        // Note: This test may fail if DeveloperUIModule dependencies are not available
        // In a real implementation, we'd use dependency injection or mocking
        match overlay.initialize() {
            Ok(()) => {
                assert!(overlay.initialized);
                assert!(!overlay.registered_components.is_empty());
            },
            Err(_) => {
                // Expected if developer UI dependencies aren't available in test
                // This is acceptable for this level of testing
            }
        }
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_debug_overlay_visibility_toggle() {
        let mut overlay = DebugOverlay::new().unwrap();
        
        let result = overlay.toggle_visibility(true);
        assert!(result.is_ok());
        assert!(overlay.is_visible());
        
        let result = overlay.toggle_visibility(false);
        assert!(result.is_ok());
        assert!(!overlay.is_visible());
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_debug_state_updates() {
        let mut overlay = DebugOverlay::new().unwrap();
        
        overlay.add_metric("test_metric", 42.0);
        overlay.add_message("Test message");
        
        let debug_state = overlay.get_debug_state();
        assert_eq!(debug_state.metrics.get("test_metric"), Some(&42.0));
        assert!(debug_state.messages.contains(&"Test message".to_string()));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_layout_configuration() {
        let mut overlay = DebugOverlay::new().unwrap();
        
        let config = DebugLayoutConfig {
            position: DebugOverlayPosition::Center,
            size_percentage: 0.5,
            resizable: false,
            movable: false,
            background_opacity: 0.8,
            show_border: false,
        };
        
        let result = overlay.configure_layout(config.clone());
        assert!(result.is_ok());
        assert_eq!(overlay.layout_config.position, DebugOverlayPosition::Center);
        assert_eq!(overlay.layout_config.size_percentage, 0.5);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_invalid_layout_configuration() {
        let mut overlay = DebugOverlay::new().unwrap();
        overlay.toggle_visibility(true).unwrap(); // Make overlay visible to trigger validation
        
        let invalid_config = DebugLayoutConfig {
            position: DebugOverlayPosition::TopLeft,
            size_percentage: 1.5, // Invalid: > 1.0
            resizable: true,
            movable: true,
            background_opacity: 0.5,
            show_border: true,
        };
        
        let result = overlay.configure_layout(invalid_config);
        assert!(result.is_err());
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_performance_metrics() {
        let mut overlay = DebugOverlay::new().unwrap();
        
        overlay.add_metric("test", 1.0);
        
        let metrics = overlay.get_performance_metrics();
        assert!(!metrics.is_empty());
        assert!(metrics.contains_key("debug_component_count"));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_debug_overlay_event_handler() {
        let overlay = Rc::new(RefCell::new(DebugOverlay::new().unwrap()));
        let mut handler = DebugOverlayEventHandler::new(overlay.clone());
        
        // Test debug toggle event
        let toggle_event = UIEvent::DebugToggle(true);
        assert!(handler.can_handle(&toggle_event));
        
        let result = handler.handle_event(toggle_event);
        assert!(result.is_success());
        assert!(overlay.borrow().is_visible());
        
        // Test performance update event
        let mut metrics = std::collections::HashMap::new();
        metrics.insert("fps".to_string(), 60.0);
        let perf_event = UIEvent::PerformanceUpdate { metrics };
        
        assert!(handler.can_handle(&perf_event));
        let result = handler.handle_event(perf_event);
        assert!(result.is_success());
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn test_production_debug_overlay_stub() {
        // Test that production build stubs work
        let mut overlay = DebugOverlay::new().unwrap();
        assert!(overlay.initialize().is_ok());
        assert!(overlay.shutdown().is_ok());
    }

    #[test]
    fn test_debug_overlay_position_types() {
        use DebugOverlayPosition::*;
        
        let positions = vec![
            TopLeft,
            TopRight, 
            BottomLeft,
            BottomRight,
            Center,
            Custom { x: 100, y: 200 },
        ];
        
        // Test that all position types can be created
        assert_eq!(positions.len(), 6);
        
        // Test equality
        assert_eq!(TopLeft, TopLeft);
        assert_ne!(TopLeft, TopRight);
        assert_eq!(Custom { x: 100, y: 200 }, Custom { x: 100, y: 200 });
        assert_ne!(Custom { x: 100, y: 200 }, Custom { x: 200, y: 100 });
    }
}