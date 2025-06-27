use super::{GraphicsError, GraphicsCapabilities, RenderingContext};
use crate::modules::presentation_layer::{UIEvent, UIState};
use web_sys::HtmlCanvasElement;
use std::collections::HashMap;

/// Interface for Graphics Foundations integration with Presentation Layer
pub trait PresentationGraphicsInterface {
    /// Initialize graphics context for UI coordination
    fn initialize_ui_graphics(&mut self) -> Result<(), GraphicsError>;
    
    /// Handle UI events related to graphics
    fn handle_ui_event(&mut self, event: &GraphicsUIEvent) -> Result<(), GraphicsError>;
    
    /// Update graphics state for UI synchronization
    fn update_graphics_state(&mut self, state: &GraphicsUIState) -> Result<(), GraphicsError>;
    
    /// Get current graphics state for UI components
    fn get_graphics_state(&self) -> GraphicsUIState;
    
    /// Create rendering context for UI components
    fn create_ui_rendering_context(&self, canvas_id: &str) -> Result<UIRenderingContext, GraphicsError>;
    
    /// Cleanup graphics resources used by UI
    fn cleanup_ui_graphics(&mut self) -> Result<(), GraphicsError>;
}

/// UI events specific to graphics operations
#[derive(Debug, Clone)]
pub enum GraphicsUIEvent {
    /// Request to initialize graphics for a canvas
    InitializeCanvas {
        canvas_id: String,
        width: u32,
        height: u32,
    },
    /// Request to resize a graphics canvas
    ResizeCanvas {
        canvas_id: String,
        width: u32,
        height: u32,
    },
    /// Request to toggle graphics features
    ToggleGraphicsFeature {
        feature: GraphicsFeature,
        enabled: bool,
    },
    /// Request graphics performance information
    RequestPerformanceInfo,
    /// Request graphics capabilities information
    RequestCapabilitiesInfo,
    /// Cleanup graphics resources for a canvas
    CleanupCanvas {
        canvas_id: String,
    },
}

/// Graphics features that can be controlled via UI
#[derive(Debug, Clone, PartialEq)]
pub enum GraphicsFeature {
    PitchVisualization,
    WaveformDisplay,
    SpectrumAnalysis,
    PerformanceOverlay,
    DebugWireframe,
}

/// Graphics state for UI synchronization
#[derive(Debug, Clone)]
pub struct GraphicsUIState {
    pub graphics_enabled: bool,
    pub active_canvases: Vec<String>,
    pub capabilities: Option<GraphicsCapabilities>,
    pub performance_metrics: GraphicsPerformanceMetrics,
    pub active_features: HashMap<GraphicsFeature, bool>,
    pub error_state: Option<GraphicsError>,
}

/// Performance metrics for graphics operations
#[derive(Debug, Clone)]
pub struct GraphicsPerformanceMetrics {
    pub render_time_ms: f32,
    pub frame_rate: f32,
    pub memory_usage_mb: f32,
    pub context_switches: u64,
    pub draw_calls_per_frame: u32,
}

/// Rendering context for UI components
#[derive(Debug, Clone)]
pub struct UIRenderingContext {
    pub canvas_id: String,
    pub context: RenderingContext,
    pub ui_state: UIComponentState,
}

/// State of UI components using graphics
#[derive(Debug, Clone)]
pub struct UIComponentState {
    pub visible: bool,
    pub interactive: bool,
    pub opacity: f32,
    pub z_index: i32,
    pub dirty: bool,
}

/// Graphics event handler for UI coordination
pub struct GraphicsEventHandler {
    canvas_contexts: HashMap<String, UIRenderingContext>,
    ui_state: GraphicsUIState,
    event_handlers: HashMap<GraphicsFeature, Box<dyn Fn(&GraphicsUIEvent) -> Result<(), GraphicsError>>>,
}

impl GraphicsEventHandler {
    /// Create a new graphics event handler
    pub fn new() -> Self {
        Self {
            canvas_contexts: HashMap::new(),
            ui_state: GraphicsUIState::default(),
            event_handlers: HashMap::new(),
        }
    }
    
    /// Register an event handler for a graphics feature
    pub fn register_feature_handler<F>(&mut self, feature: GraphicsFeature, handler: F)
    where
        F: Fn(&GraphicsUIEvent) -> Result<(), GraphicsError> + 'static,
    {
        self.event_handlers.insert(feature, Box::new(handler));
    }
    
    /// Process a graphics UI event
    pub fn process_event(&mut self, event: &GraphicsUIEvent) -> Result<(), GraphicsError> {
        match event {
            GraphicsUIEvent::InitializeCanvas { canvas_id, width, height } => {
                self.handle_canvas_initialization(canvas_id, *width, *height)
            }
            GraphicsUIEvent::ResizeCanvas { canvas_id, width, height } => {
                self.handle_canvas_resize(canvas_id, *width, *height)
            }
            GraphicsUIEvent::ToggleGraphicsFeature { feature, enabled } => {
                self.handle_feature_toggle(feature.clone(), *enabled)
            }
            GraphicsUIEvent::RequestPerformanceInfo => {
                self.handle_performance_request()
            }
            GraphicsUIEvent::RequestCapabilitiesInfo => {
                self.handle_capabilities_request()
            }
            GraphicsUIEvent::CleanupCanvas { canvas_id } => {
                self.handle_canvas_cleanup(canvas_id)
            }
        }
    }
    
    /// Handle canvas initialization
    fn handle_canvas_initialization(&mut self, canvas_id: &str, width: u32, height: u32) -> Result<(), GraphicsError> {
        // Validate canvas dimensions
        if width == 0 || height == 0 {
            return Err(GraphicsError::InvalidConfiguration(
                "Canvas dimensions must be greater than 0".to_string()
            ));
        }
        
        // Create UI rendering context
        let ui_context = UIRenderingContext {
            canvas_id: canvas_id.to_string(),
            context: RenderingContext {
                wgpu_context: None, // Will be initialized by graphics module
                capabilities: self.ui_state.capabilities.clone().unwrap_or_default(),
                canvas_id: canvas_id.to_string(),
            },
            ui_state: UIComponentState {
                visible: true,
                interactive: true,
                opacity: 1.0,
                z_index: 0,
                dirty: true,
            },
        };
        
        self.canvas_contexts.insert(canvas_id.to_string(), ui_context);
        self.ui_state.active_canvases.push(canvas_id.to_string());
        
        Ok(())
    }
    
    /// Handle canvas resize
    fn handle_canvas_resize(&mut self, canvas_id: &str, width: u32, height: u32) -> Result<(), GraphicsError> {
        if let Some(context) = self.canvas_contexts.get_mut(canvas_id) {
            context.ui_state.dirty = true;
            // TODO: Update canvas dimensions in rendering context
            Ok(())
        } else {
            Err(GraphicsError::CanvasNotAvailable(format!("Canvas '{}' not found", canvas_id)))
        }
    }
    
    /// Handle feature toggle
    fn handle_feature_toggle(&mut self, feature: GraphicsFeature, enabled: bool) -> Result<(), GraphicsError> {
        self.ui_state.active_features.insert(feature.clone(), enabled);
        
        // Call registered handler if available
        if let Some(handler) = self.event_handlers.get(&feature) {
            let event = GraphicsUIEvent::ToggleGraphicsFeature { feature, enabled };
            handler(&event)?;
        }
        
        Ok(())
    }
    
    /// Handle performance information request
    fn handle_performance_request(&mut self) -> Result<(), GraphicsError> {
        // Update performance metrics
        // In a real implementation, this would gather actual metrics
        self.ui_state.performance_metrics = GraphicsPerformanceMetrics {
            render_time_ms: 16.67, // ~60 FPS
            frame_rate: 60.0,
            memory_usage_mb: 32.0,
            context_switches: 0,
            draw_calls_per_frame: 10,
        };
        
        Ok(())
    }
    
    /// Handle capabilities information request
    fn handle_capabilities_request(&mut self) -> Result<(), GraphicsError> {
        // Update capabilities information
        if self.ui_state.capabilities.is_none() {
            self.ui_state.capabilities = Some(GraphicsCapabilities::detect()?);
        }
        
        Ok(())
    }
    
    /// Handle canvas cleanup
    fn handle_canvas_cleanup(&mut self, canvas_id: &str) -> Result<(), GraphicsError> {
        self.canvas_contexts.remove(canvas_id);
        self.ui_state.active_canvases.retain(|id| id != canvas_id);
        
        Ok(())
    }
    
    /// Get current graphics UI state
    pub fn get_ui_state(&self) -> &GraphicsUIState {
        &self.ui_state
    }
    
    /// Get canvas context by ID
    pub fn get_canvas_context(&self, canvas_id: &str) -> Option<&UIRenderingContext> {
        self.canvas_contexts.get(canvas_id)
    }
    
    /// Check if a feature is enabled
    pub fn is_feature_enabled(&self, feature: &GraphicsFeature) -> bool {
        self.ui_state.active_features.get(feature).copied().unwrap_or(false)
    }
}

impl Default for GraphicsUIState {
    fn default() -> Self {
        Self {
            graphics_enabled: false,
            active_canvases: Vec::new(),
            capabilities: None,
            performance_metrics: GraphicsPerformanceMetrics::default(),
            active_features: HashMap::new(),
            error_state: None,
        }
    }
}

impl Default for GraphicsPerformanceMetrics {
    fn default() -> Self {
        Self {
            render_time_ms: 0.0,
            frame_rate: 0.0,
            memory_usage_mb: 0.0,
            context_switches: 0,
            draw_calls_per_frame: 0,
        }
    }
}

impl Default for UIComponentState {
    fn default() -> Self {
        Self {
            visible: true,
            interactive: true,
            opacity: 1.0,
            z_index: 0,
            dirty: false,
        }
    }
}

/// Graphics state manager for UI coordination
pub struct GraphicsStateManager {
    state: GraphicsUIState,
    event_handler: GraphicsEventHandler,
    subscribers: Vec<Box<dyn Fn(&GraphicsUIState)>>,
}

impl GraphicsStateManager {
    /// Create a new graphics state manager
    pub fn new() -> Self {
        Self {
            state: GraphicsUIState::default(),
            event_handler: GraphicsEventHandler::new(),
            subscribers: Vec::new(),
        }
    }
    
    /// Subscribe to state changes
    pub fn subscribe<F>(&mut self, callback: F)
    where
        F: Fn(&GraphicsUIState) + 'static,
    {
        self.subscribers.push(Box::new(callback));
    }
    
    /// Update graphics state and notify subscribers
    pub fn update_state(&mut self, new_state: GraphicsUIState) {
        self.state = new_state;
        self.notify_subscribers();
    }
    
    /// Process UI event and update state
    pub fn process_ui_event(&mut self, event: &GraphicsUIEvent) -> Result<(), GraphicsError> {
        self.event_handler.process_event(event)?;
        self.state = self.event_handler.get_ui_state().clone();
        self.notify_subscribers();
        Ok(())
    }
    
    /// Get current state
    pub fn get_state(&self) -> &GraphicsUIState {
        &self.state
    }
    
    /// Notify all subscribers of state changes
    fn notify_subscribers(&self) {
        for callback in &self.subscribers {
            callback(&self.state);
        }
    }
}

impl Default for GraphicsStateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_graphics_event_handler_creation() {
        let handler = GraphicsEventHandler::new();
        assert_eq!(handler.canvas_contexts.len(), 0);
        assert!(!handler.ui_state.graphics_enabled);
    }
    
    #[test]
    fn test_graphics_ui_state_default() {
        let state = GraphicsUIState::default();
        assert!(!state.graphics_enabled);
        assert!(state.active_canvases.is_empty());
        assert!(state.capabilities.is_none());
    }
    
    #[test]
    fn test_feature_toggle() {
        let mut handler = GraphicsEventHandler::new();
        let event = GraphicsUIEvent::ToggleGraphicsFeature {
            feature: GraphicsFeature::PitchVisualization,
            enabled: true,
        };
        
        let result = handler.process_event(&event);
        assert!(result.is_ok());
        assert!(handler.is_feature_enabled(&GraphicsFeature::PitchVisualization));
    }
    
    #[test]
    fn test_canvas_initialization() {
        let mut handler = GraphicsEventHandler::new();
        let event = GraphicsUIEvent::InitializeCanvas {
            canvas_id: "test_canvas".to_string(),
            width: 800,
            height: 600,
        };
        
        let result = handler.process_event(&event);
        assert!(result.is_ok());
        assert!(handler.get_canvas_context("test_canvas").is_some());
    }
}