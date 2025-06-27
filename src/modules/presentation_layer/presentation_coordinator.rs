//! # Presentation Coordinator
//!
//! This module provides the main PresentationCoordinator implementation that
//! coordinates between different UI systems, manages state synchronization,
//! and handles event routing.

use std::rc::Rc;
use std::cell::RefCell;
use crate::modules::application_core::PriorityEventBus;
use crate::modules::presentation_layer::{
    UICoordinator, UIState, UIError, UIEvent, EventRouter, EventHandler, EventResult,
    ImmersiveRenderer, StubImmersiveRenderer, CanvasContext, RenderContext,
};

#[cfg(debug_assertions)]
use crate::modules::presentation_layer::DebugState;

/// Main coordinator implementation for UI systems
pub struct PresentationCoordinator {
    /// Immersive renderer (stub for now, will be replaced with wgpu renderer)
    immersive_renderer: Box<dyn ImmersiveRenderer>,
    /// Current UI state
    current_state: UIState,
    /// Debug state (debug builds only)
    #[cfg(debug_assertions)]
    debug_state: DebugState,
    /// Event router for UI coordination
    event_router: EventRouter,
    /// Canvas context for rendering
    canvas_context: Option<CanvasContext>,
    /// Event bus connection
    event_bus: Option<Rc<RefCell<PriorityEventBus>>>,
    /// Performance metrics
    performance_metrics: std::collections::HashMap<String, f64>,
    /// Coordinator lifecycle state
    initialized: bool,
    started: bool,
}

impl PresentationCoordinator {
    /// Create a new presentation coordinator
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            immersive_renderer: Box::new(StubImmersiveRenderer::new()),
            current_state: UIState::new(),
            #[cfg(debug_assertions)]
            debug_state: DebugState::new(),
            event_router: EventRouter::new(),
            canvas_context: None,
            event_bus: None,
            performance_metrics: std::collections::HashMap::new(),
            initialized: false,
            started: false,
        })
    }

    /// Initialize canvas for rendering
    pub fn initialize_canvas(&mut self, canvas_id: impl Into<String>, width: u32, height: u32) -> Result<(), UIError> {
        let canvas_context = CanvasContext::new(width, height, canvas_id);
        
        // Initialize immersive renderer with canvas
        self.immersive_renderer.initialize(canvas_context.clone())?;
        
        self.canvas_context = Some(canvas_context);
        Ok(())
    }

    /// Update performance metrics
    fn update_performance_metrics(&mut self) {
        let start_time = std::time::Instant::now();
        
        // Get metrics from immersive renderer
        let renderer_metrics = self.immersive_renderer.get_performance_metrics();
        for (key, value) in renderer_metrics {
            self.performance_metrics.insert(format!("renderer_{}", key), value);
        }
        
        // Get metrics from event router
        let router_metrics = self.event_router.get_performance_metrics();
        for (key, value) in router_metrics {
            self.performance_metrics.insert(format!("router_{}", key), *value);
        }
        
        // Add coordination overhead metrics
        let coordination_time = start_time.elapsed().as_micros() as f64;
        self.performance_metrics.insert("coordination_overhead_us".to_string(), coordination_time);
        
        // Add general metrics
        self.performance_metrics.insert("event_handlers_count".to_string(), self.event_router.handler_count() as f64);
        self.performance_metrics.insert("canvas_width".to_string(), 
            self.canvas_context.as_ref().map(|c| c.width as f64).unwrap_or(0.0));
        self.performance_metrics.insert("canvas_height".to_string(), 
            self.canvas_context.as_ref().map(|c| c.height as f64).unwrap_or(0.0));
    }

    /// Render current frame
    fn render_frame(&mut self) -> Result<(), UIError> {
        if let Some(ref canvas_context) = self.canvas_context {
            let render_context = RenderContext::new(canvas_context.clone(), self.current_state.clone());
            self.immersive_renderer.render_frame(&render_context)?;
        }
        Ok(())
    }

    /// Setup default event handlers
    fn setup_event_handlers(&mut self) {
        // Register self as an event handler for coordination events
        let coordinator_handler = CoordinatorEventHandler::new();
        self.event_router.register_handler(Box::new(coordinator_handler));
    }
}

impl Default for PresentationCoordinator {
    fn default() -> Self {
        Self::new().expect("Failed to create PresentationCoordinator")
    }
}

impl UICoordinator for PresentationCoordinator {
    fn render_immersive_ui(&mut self, state: &UIState) -> Result<(), UIError> {
        self.current_state = state.clone();
        self.immersive_renderer.update_state(state)?;
        self.render_frame()?;
        self.update_performance_metrics();
        Ok(())
    }

    #[cfg(debug_assertions)]
    fn render_debug_overlay(&mut self, debug_state: &DebugState) -> Result<(), UIError> {
        self.debug_state = debug_state.clone();
        // Debug overlay rendering will be implemented in Task 2
        Ok(())
    }

    #[cfg(debug_assertions)]
    fn toggle_debug_overlay(&mut self, visible: bool) -> Result<(), UIError> {
        self.debug_state.set_overlay_visible(visible);
        
        // Route debug toggle event
        let event = UIEvent::DebugToggle(visible);
        self.event_router.route_event(event);
        
        Ok(())
    }

    fn handle_ui_event(&mut self, event: UIEvent) -> Result<(), UIError> {
        let results = self.event_router.route_event(event);
        
        // Check if any event handling failed
        for result in &results {
            if let Some(error_msg) = result.error_message() {
                return Err(UIError::EventError(error_msg));
            }
        }
        
        // Update performance metrics after event handling
        self.update_performance_metrics();
        
        Ok(())
    }

    fn update_state(&mut self, state: UIState) -> Result<(), UIError> {
        self.current_state = state.clone();
        self.immersive_renderer.update_state(&state)?;
        
        // Route state update event
        let event = UIEvent::StateUpdate(state);
        self.event_router.route_event(event);
        
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), UIError> {
        if self.initialized {
            return Ok(());
        }

        // Initialize default canvas if none provided
        if self.canvas_context.is_none() {
            self.initialize_canvas("presentation-canvas", 800, 600)?;
        }

        // Setup event handlers
        self.setup_event_handlers();

        self.initialized = true;
        Ok(())
    }

    fn start(&mut self) -> Result<(), UIError> {
        if !self.initialized {
            return Err(UIError::InitializationError("Coordinator not initialized".to_string()));
        }
        if self.started {
            return Ok(());
        }

        // Verify renderer is ready
        if !self.immersive_renderer.is_ready() {
            return Err(UIError::InitializationError("Immersive renderer not ready".to_string()));
        }

        self.started = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), UIError> {
        if !self.started {
            return Ok(());
        }

        self.started = false;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), UIError> {
        if self.started {
            self.stop()?;
        }

        if !self.initialized {
            return Ok(());
        }

        // Shutdown immersive renderer
        self.immersive_renderer.shutdown()?;

        // Reset event router
        self.event_router.reset();

        // Clear state
        self.canvas_context = None;
        self.event_bus = None;
        self.performance_metrics.clear();

        self.initialized = false;
        Ok(())
    }

    fn connect_event_bus(&mut self, event_bus: Rc<RefCell<PriorityEventBus>>) -> Result<(), UIError> {
        self.event_bus = Some(event_bus);
        Ok(())
    }

    fn get_current_state(&self) -> &UIState {
        &self.current_state
    }

    fn get_performance_metrics(&self) -> std::collections::HashMap<String, f64> {
        self.performance_metrics.clone()
    }
}

/// Event handler for coordinator-specific events
struct CoordinatorEventHandler {
    name: &'static str,
}

impl CoordinatorEventHandler {
    fn new() -> Self {
        Self {
            name: "CoordinatorEventHandler",
        }
    }
}

impl EventHandler for CoordinatorEventHandler {
    fn handle_event(&mut self, event: UIEvent) -> EventResult {
        match event {
            UIEvent::CanvasResize { width, height } => {
                // Canvas resize would be handled here
                // For now, just acknowledge the event
                let _ = (width, height);
                EventResult::Handled
            },
            UIEvent::PerformanceUpdate { metrics } => {
                // Performance updates would be processed here
                let _ = metrics;
                EventResult::Handled
            },
            UIEvent::Error { message: _, severity: _ } => {
                // Error events would be logged/processed here
                EventResult::Handled
            },
            _ => EventResult::NotHandled,
        }
    }

    fn can_handle(&self, event: &UIEvent) -> bool {
        matches!(event, 
            UIEvent::CanvasResize { .. } |
            UIEvent::PerformanceUpdate { .. } |
            UIEvent::Error { .. }
        )
    }

    fn handler_name(&self) -> &'static str {
        self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presentation_coordinator_creation() {
        let coordinator = PresentationCoordinator::new();
        assert!(coordinator.is_ok());
        
        let coordinator = coordinator.unwrap();
        assert!(!coordinator.initialized);
        assert!(!coordinator.started);
    }

    #[test]
    fn test_coordinator_initialization() {
        let mut coordinator = PresentationCoordinator::new().unwrap();
        
        let result = coordinator.initialize();
        assert!(result.is_ok());
        assert!(coordinator.initialized);
        
        // Canvas should be automatically initialized
        assert!(coordinator.canvas_context.is_some());
    }

    #[test]
    fn test_coordinator_canvas_initialization() {
        let mut coordinator = PresentationCoordinator::new().unwrap();
        
        let result = coordinator.initialize_canvas("test-canvas", 1024, 768);
        assert!(result.is_ok());
        
        let canvas = coordinator.canvas_context.as_ref().unwrap();
        assert_eq!(canvas.canvas_id, "test-canvas");
        assert_eq!(canvas.width, 1024);
        assert_eq!(canvas.height, 768);
    }

    #[test]
    fn test_coordinator_lifecycle() {
        let mut coordinator = PresentationCoordinator::new().unwrap();
        
        // Initialize
        coordinator.initialize().unwrap();
        assert!(coordinator.initialized);
        
        // Start
        let start_result = coordinator.start();
        assert!(start_result.is_ok());
        assert!(coordinator.started);
        
        // Stop
        coordinator.stop().unwrap();
        assert!(!coordinator.started);
        
        // Shutdown
        coordinator.shutdown().unwrap();
        assert!(!coordinator.initialized);
    }

    #[test]
    fn test_ui_state_update() {
        let mut coordinator = PresentationCoordinator::new().unwrap();
        coordinator.initialize().unwrap();
        
        let mut new_state = UIState::new();
        new_state.set_mode("test-mode");
        new_state.set_data("key1", "value1");
        
        let result = coordinator.update_state(new_state.clone());
        assert!(result.is_ok());
        
        assert_eq!(coordinator.get_current_state().mode, "test-mode");
        assert_eq!(coordinator.get_current_state().get_data("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_immersive_ui_rendering() {
        let mut coordinator = PresentationCoordinator::new().unwrap();
        coordinator.initialize().unwrap();
        
        let state = UIState::new();
        let result = coordinator.render_immersive_ui(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_event_handling() {
        let mut coordinator = PresentationCoordinator::new().unwrap();
        coordinator.initialize().unwrap();
        
        let event = UIEvent::UserInteraction(UserInteraction::Click { x: 100.0, y: 200.0 });
        let result = coordinator.handle_ui_event(event);
        assert!(result.is_ok());
    }

    #[test]
    fn test_performance_metrics() {
        let mut coordinator = PresentationCoordinator::new().unwrap();
        coordinator.initialize().unwrap();
        
        // Trigger metrics update by rendering
        let state = UIState::new();
        coordinator.render_immersive_ui(&state).unwrap();
        
        let metrics = coordinator.get_performance_metrics();
        assert!(!metrics.is_empty());
        assert!(metrics.contains_key("coordination_overhead_us"));
    }

    #[test]
    fn test_event_bus_connection() {
        let mut coordinator = PresentationCoordinator::new().unwrap();
        
        // Create a mock event bus (simplified for testing)
        let event_bus = Rc::new(RefCell::new(PriorityEventBus::new()));
        
        let result = coordinator.connect_event_bus(event_bus);
        assert!(result.is_ok());
        assert!(coordinator.event_bus.is_some());
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_debug_overlay_toggle() {
        let mut coordinator = PresentationCoordinator::new().unwrap();
        coordinator.initialize().unwrap();
        
        // Test enabling debug overlay
        let result = coordinator.toggle_debug_overlay(true);
        assert!(result.is_ok());
        assert!(coordinator.debug_state.overlay_visible);
        
        // Test disabling debug overlay
        let result = coordinator.toggle_debug_overlay(false);
        assert!(result.is_ok());
        assert!(!coordinator.debug_state.overlay_visible);
    }

    #[test]
    fn test_coordinator_event_handler() {
        let mut handler = CoordinatorEventHandler::new();
        
        // Test canvas resize event
        let resize_event = UIEvent::CanvasResize { width: 1024, height: 768 };
        assert!(handler.can_handle(&resize_event));
        
        let result = handler.handle_event(resize_event);
        assert!(result.is_success());
        
        // Test unhandled event
        let click_event = UIEvent::UserInteraction(UserInteraction::Click { x: 0.0, y: 0.0 });
        assert!(!handler.can_handle(&click_event));
        
        let result = handler.handle_event(click_event);
        assert!(matches!(result, EventResult::NotHandled));
    }

    #[test]
    fn test_coordinator_start_without_initialization() {
        let mut coordinator = PresentationCoordinator::new().unwrap();
        
        let result = coordinator.start();
        assert!(result.is_err());
        
        match result.unwrap_err() {
            UIError::InitializationError(msg) => assert!(msg.contains("not initialized")),
            _ => panic!("Expected InitializationError"),
        }
    }
}