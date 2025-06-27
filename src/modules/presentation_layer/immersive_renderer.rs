//! # Immersive Renderer
//!
//! This module defines the ImmersiveRenderer trait interface for future graphics
//! integration and provides a stub implementation for coordination testing.

use crate::modules::presentation_layer::{UIState, UIError};

/// User interaction types for immersive UI
#[derive(Debug, Clone)]
pub enum UserInteraction {
    /// Mouse/touch click at position (x, y)
    Click { x: f32, y: f32 },
    /// Mouse/touch drag from (x1, y1) to (x2, y2)
    Drag { x1: f32, y1: f32, x2: f32, y2: f32 },
    /// Keyboard input
    KeyPress { key: String, modifiers: Vec<String> },
    /// Scroll/zoom gesture
    Scroll { delta_x: f32, delta_y: f32, zoom: f32 },
    /// Hover at position (x, y)
    Hover { x: f32, y: f32 },
    /// Generic gesture with parameters
    Gesture { name: String, parameters: std::collections::HashMap<String, f32> },
}

/// Canvas context for rendering operations
#[derive(Debug, Clone)]
pub struct CanvasContext {
    /// Canvas width in pixels
    pub width: u32,
    /// Canvas height in pixels
    pub height: u32,
    /// Device pixel ratio for high-DPI displays
    pub device_pixel_ratio: f32,
    /// Canvas element ID for DOM integration
    pub canvas_id: String,
}

impl CanvasContext {
    /// Create a new canvas context
    pub fn new(width: u32, height: u32, canvas_id: impl Into<String>) -> Self {
        Self {
            width,
            height,
            device_pixel_ratio: 1.0,
            canvas_id: canvas_id.into(),
        }
    }

    /// Update canvas dimensions
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    /// Set device pixel ratio for high-DPI displays
    pub fn set_device_pixel_ratio(&mut self, ratio: f32) {
        self.device_pixel_ratio = ratio;
    }

    /// Get effective rendering width accounting for device pixel ratio
    pub fn effective_width(&self) -> u32 {
        (self.width as f32 * self.device_pixel_ratio) as u32
    }

    /// Get effective rendering height accounting for device pixel ratio
    pub fn effective_height(&self) -> u32 {
        (self.height as f32 * self.device_pixel_ratio) as u32
    }
}

/// Render context containing information about the current rendering state
#[derive(Debug, Clone)]
pub struct RenderContext {
    /// Canvas context for rendering
    pub canvas: CanvasContext,
    /// Current UI state
    pub ui_state: UIState,
    /// Rendering timestamp (frame number or time)
    pub timestamp: u64,
    /// Performance budget for this frame (in milliseconds)
    pub performance_budget_ms: f32,
}

impl RenderContext {
    /// Create a new render context
    pub fn new(canvas: CanvasContext, ui_state: UIState) -> Self {
        Self {
            canvas,
            ui_state,
            timestamp: 0,
            performance_budget_ms: 16.67, // ~60 FPS
        }
    }

    /// Update the render context with new state
    pub fn update(&mut self, ui_state: UIState) {
        self.ui_state = ui_state;
        self.timestamp = self.timestamp.wrapping_add(1);
    }

    /// Set performance budget for this frame
    pub fn set_performance_budget(&mut self, budget_ms: f32) {
        self.performance_budget_ms = budget_ms;
    }
}

/// Trait for immersive rendering systems (future graphics integration)
///
/// This trait defines the interface that will be implemented by the Graphics
/// Foundations module in future stories. For now, we provide a stub implementation
/// to enable coordination testing.
pub trait ImmersiveRenderer {
    /// Initialize the immersive renderer
    fn initialize(&mut self, canvas_context: CanvasContext) -> Result<(), UIError>;
    
    /// Render a frame with the given render context
    fn render_frame(&mut self, context: &RenderContext) -> Result<(), UIError>;
    
    /// Handle user interaction in the immersive UI
    fn handle_interaction(&mut self, interaction: UserInteraction) -> Result<(), UIError>;
    
    /// Update the renderer with new UI state
    fn update_state(&mut self, state: &UIState) -> Result<(), UIError>;
    
    /// Resize the rendering surface
    fn resize(&mut self, width: u32, height: u32) -> Result<(), UIError>;
    
    /// Get performance metrics from the renderer
    fn get_performance_metrics(&self) -> std::collections::HashMap<String, f64>;
    
    /// Check if the renderer is ready for rendering
    fn is_ready(&self) -> bool;
    
    /// Shutdown the renderer and clean up resources
    fn shutdown(&mut self) -> Result<(), UIError>;
}

/// Stub implementation of ImmersiveRenderer for coordination testing
///
/// This implementation provides a minimal, no-op renderer that can be used
/// to test the UI coordination architecture without actual graphics rendering.
/// It will be replaced by a real wgpu-based renderer in future stories.
pub struct StubImmersiveRenderer {
    canvas_context: Option<CanvasContext>,
    current_state: UIState,
    performance_metrics: std::collections::HashMap<String, f64>,
    initialized: bool,
    frame_count: u64,
}

impl StubImmersiveRenderer {
    /// Create a new stub immersive renderer
    pub fn new() -> Self {
        Self {
            canvas_context: None,
            current_state: UIState::new(),
            performance_metrics: std::collections::HashMap::new(),
            initialized: false,
            frame_count: 0,
        }
    }

    /// Update performance metrics with stub values
    fn update_performance_metrics(&mut self) {
        self.performance_metrics.insert("render_time_ms".to_string(), 0.1);
        self.performance_metrics.insert("triangles".to_string(), 0.0);
        self.performance_metrics.insert("draw_calls".to_string(), 0.0);
        self.performance_metrics.insert("memory_usage_mb".to_string(), 1.0);
        self.performance_metrics.insert("frame_rate".to_string(), 60.0);
    }
}

impl Default for StubImmersiveRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl ImmersiveRenderer for StubImmersiveRenderer {
    fn initialize(&mut self, canvas_context: CanvasContext) -> Result<(), UIError> {
        self.canvas_context = Some(canvas_context);
        self.update_performance_metrics();
        self.initialized = true;
        Ok(())
    }
    
    fn render_frame(&mut self, context: &RenderContext) -> Result<(), UIError> {
        if !self.initialized {
            return Err(UIError::RenderError("Renderer not initialized".to_string()));
        }
        
        // Stub rendering - just update frame count and metrics
        self.frame_count = self.frame_count.wrapping_add(1);
        self.current_state = context.ui_state.clone();
        self.update_performance_metrics();
        
        // Simulate minimal rendering time
        if context.performance_budget_ms < 0.1 {
            return Err(UIError::RenderError("Insufficient performance budget".to_string()));
        }
        
        Ok(())
    }
    
    fn handle_interaction(&mut self, interaction: UserInteraction) -> Result<(), UIError> {
        if !self.initialized {
            return Err(UIError::EventError("Renderer not initialized".to_string()));
        }
        
        // Stub interaction handling - just log the interaction type
        match interaction {
            UserInteraction::Click { x, y } => {
                // In a real implementation, this would handle click events
                let _ = (x, y);
            }
            UserInteraction::Drag { x1, y1, x2, y2 } => {
                // In a real implementation, this would handle drag events
                let _ = (x1, y1, x2, y2);
            }
            UserInteraction::KeyPress { key: _, modifiers: _ } => {
                // In a real implementation, this would handle keyboard input
            }
            UserInteraction::Scroll { delta_x: _, delta_y: _, zoom: _ } => {
                // In a real implementation, this would handle scroll/zoom
            }
            UserInteraction::Hover { x, y } => {
                // In a real implementation, this would handle hover events
                let _ = (x, y);
            }
            UserInteraction::Gesture { name: _, parameters: _ } => {
                // In a real implementation, this would handle custom gestures
            }
        }
        
        Ok(())
    }
    
    fn update_state(&mut self, state: &UIState) -> Result<(), UIError> {
        self.current_state = state.clone();
        Ok(())
    }
    
    fn resize(&mut self, width: u32, height: u32) -> Result<(), UIError> {
        if let Some(ref mut canvas) = self.canvas_context {
            canvas.resize(width, height);
        }
        Ok(())
    }
    
    fn get_performance_metrics(&self) -> std::collections::HashMap<String, f64> {
        self.performance_metrics.clone()
    }
    
    fn is_ready(&self) -> bool {
        self.initialized && self.canvas_context.is_some()
    }
    
    fn shutdown(&mut self) -> Result<(), UIError> {
        self.canvas_context = None;
        self.performance_metrics.clear();
        self.initialized = false;
        self.frame_count = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_context_creation() {
        let canvas = CanvasContext::new(800, 600, "test-canvas");
        assert_eq!(canvas.width, 800);
        assert_eq!(canvas.height, 600);
        assert_eq!(canvas.device_pixel_ratio, 1.0);
        assert_eq!(canvas.canvas_id, "test-canvas");
    }

    #[test]
    fn test_canvas_context_resize() {
        let mut canvas = CanvasContext::new(800, 600, "test-canvas");
        canvas.resize(1024, 768);
        assert_eq!(canvas.width, 1024);
        assert_eq!(canvas.height, 768);
    }

    #[test]
    fn test_canvas_context_device_pixel_ratio() {
        let mut canvas = CanvasContext::new(800, 600, "test-canvas");
        canvas.set_device_pixel_ratio(2.0);
        assert_eq!(canvas.device_pixel_ratio, 2.0);
        assert_eq!(canvas.effective_width(), 1600);
        assert_eq!(canvas.effective_height(), 1200);
    }

    #[test]
    fn test_render_context_creation() {
        let canvas = CanvasContext::new(800, 600, "test-canvas");
        let ui_state = UIState::new();
        let context = RenderContext::new(canvas, ui_state);
        
        assert_eq!(context.canvas.width, 800);
        assert_eq!(context.canvas.height, 600);
        assert_eq!(context.timestamp, 0);
        assert_eq!(context.performance_budget_ms, 16.67);
    }

    #[test]
    fn test_stub_renderer_initialization() {
        let mut renderer = StubImmersiveRenderer::new();
        assert!(!renderer.is_ready());
        
        let canvas = CanvasContext::new(800, 600, "test-canvas");
        let result = renderer.initialize(canvas);
        assert!(result.is_ok());
        assert!(renderer.is_ready());
    }

    #[test]
    fn test_stub_renderer_render_frame() {
        let mut renderer = StubImmersiveRenderer::new();
        let canvas = CanvasContext::new(800, 600, "test-canvas");
        renderer.initialize(canvas.clone()).unwrap();
        
        let ui_state = UIState::new();
        let context = RenderContext::new(canvas, ui_state);
        let result = renderer.render_frame(&context);
        assert!(result.is_ok());
        assert_eq!(renderer.frame_count, 1);
    }

    #[test]
    fn test_stub_renderer_handle_interaction() {
        let mut renderer = StubImmersiveRenderer::new();
        let canvas = CanvasContext::new(800, 600, "test-canvas");
        renderer.initialize(canvas).unwrap();
        
        let interaction = UserInteraction::Click { x: 100.0, y: 200.0 };
        let result = renderer.handle_interaction(interaction);
        assert!(result.is_ok());
    }

    #[test]
    fn test_stub_renderer_performance_metrics() {
        let mut renderer = StubImmersiveRenderer::new();
        let canvas = CanvasContext::new(800, 600, "test-canvas");
        renderer.initialize(canvas).unwrap();
        
        let metrics = renderer.get_performance_metrics();
        assert!(metrics.contains_key("render_time_ms"));
        assert!(metrics.contains_key("frame_rate"));
        assert_eq!(metrics.get("frame_rate"), Some(&60.0));
    }

    #[test]
    fn test_stub_renderer_uninitialized_error() {
        let mut renderer = StubImmersiveRenderer::new();
        
        let interaction = UserInteraction::Click { x: 100.0, y: 200.0 };
        let result = renderer.handle_interaction(interaction);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            UIError::EventError(msg) => assert!(msg.contains("not initialized")),
            _ => panic!("Expected EventError"),
        }
    }

    #[test]
    fn test_user_interaction_types() {
        let click = UserInteraction::Click { x: 10.0, y: 20.0 };
        let drag = UserInteraction::Drag { x1: 0.0, y1: 0.0, x2: 10.0, y2: 10.0 };
        let key = UserInteraction::KeyPress { 
            key: "a".to_string(), 
            modifiers: vec!["ctrl".to_string()] 
        };
        let scroll = UserInteraction::Scroll { delta_x: 0.0, delta_y: -10.0, zoom: 1.1 };
        let hover = UserInteraction::Hover { x: 50.0, y: 60.0 };
        let gesture = UserInteraction::Gesture { 
            name: "pinch".to_string(), 
            parameters: std::collections::HashMap::new() 
        };
        
        // Test that all interaction types can be created
        let interactions = vec![click, drag, key, scroll, hover, gesture];
        assert_eq!(interactions.len(), 6);
    }
}