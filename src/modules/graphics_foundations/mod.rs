use crate::modules::application_core::module_registry::{Module, ModuleId};
use std::collections::HashMap;
use web_sys::HtmlCanvasElement;

// Module exports
pub use capabilities::*;
pub use wgpu_context::*;
pub use render_pipeline::*;
pub use visualization_renderer::*;
pub use presentation_integration::*;

// Module components
mod capabilities;
mod wgpu_context;
mod render_pipeline;
mod visualization_renderer;
mod presentation_integration;

/// Graphics error types for domain-specific error handling
#[derive(Debug, Clone)]
pub enum GraphicsError {
    ContextInitializationFailed(String),
    CapabilityDetectionFailed(String),
    CanvasNotAvailable(String),
    WgpuNotSupported(String),
    WebGlNotSupported(String),
    ResourceAllocationFailed(String),
    InvalidConfiguration(String),
}

impl std::fmt::Display for GraphicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphicsError::ContextInitializationFailed(msg) => {
                write!(f, "Graphics context initialization failed: {}", msg)
            }
            GraphicsError::CapabilityDetectionFailed(msg) => {
                write!(f, "Graphics capability detection failed: {}", msg)
            }
            GraphicsError::CanvasNotAvailable(msg) => {
                write!(f, "Canvas element not available: {}", msg)
            }
            GraphicsError::WgpuNotSupported(msg) => {
                write!(f, "WebGPU not supported: {}", msg)
            }
            GraphicsError::WebGlNotSupported(msg) => {
                write!(f, "WebGL not supported: {}", msg)
            }
            GraphicsError::ResourceAllocationFailed(msg) => {
                write!(f, "Graphics resource allocation failed: {}", msg)
            }
            GraphicsError::InvalidConfiguration(msg) => {
                write!(f, "Invalid graphics configuration: {}", msg)
            }
        }
    }
}

impl std::error::Error for GraphicsError {}

/// Core trait for Graphics Foundations module functionality
pub trait GraphicsFoundations: Module {
    /// Create a rendering context for the given canvas element
    fn create_rendering_context(&self, canvas: &HtmlCanvasElement) -> Result<RenderingContext, GraphicsError>;
    
    /// Get the graphics capabilities of the current environment
    fn get_graphics_capabilities(&self) -> GraphicsCapabilities;
    
    /// Check if graphics capabilities are available
    fn is_graphics_available(&self) -> bool;
    
    /// Initialize graphics resources
    fn initialize_graphics(&mut self) -> Result<(), GraphicsError>;
    
    /// Cleanup graphics resources
    fn cleanup_graphics(&mut self) -> Result<(), GraphicsError>;
}

/// Rendering context for graphics operations
#[derive(Debug)]
pub struct RenderingContext {
    pub wgpu_context: Option<WgpuContextHandle>,
    pub capabilities: GraphicsCapabilities,
    pub canvas_id: String,
}

/// Graphics Foundations module implementation
pub struct GraphicsFoundationsModule {
    capabilities: Option<GraphicsCapabilities>,
    wgpu_context: Option<WgpuContextHandle>,
    canvas_contexts: HashMap<String, RenderingContext>,
    initialized: bool,
}

impl GraphicsFoundationsModule {
    /// Create a new Graphics Foundations module instance
    pub fn new() -> Self {
        Self {
            capabilities: None,
            wgpu_context: None,
            canvas_contexts: HashMap::new(),
            initialized: false,
        }
    }
    
    /// Detect graphics capabilities for the current environment
    pub fn detect_graphics_capabilities(&self) -> Result<GraphicsCapabilities, GraphicsError> {
        GraphicsCapabilities::detect()
    }
    
    /// Setup module integration with application core
    fn setup_module_integration(&mut self) -> Result<(), GraphicsError> {
        // Module integration logic will be implemented here
        Ok(())
    }
}

impl Module for GraphicsFoundationsModule {
    fn module_id(&self) -> ModuleId {
        ModuleId::new("graphics_foundations")
    }
    
    fn module_name(&self) -> &str {
        "Graphics Foundations"
    }
    
    fn module_version(&self) -> &str {
        "1.0.0"
    }
    
    fn dependencies(&self) -> Vec<ModuleId> {
        vec![
            ModuleId::new("platform_abstraction"),
        ]
    }
    
    fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.initialized {
            return Ok(());
        }
        
        // Detect graphics capabilities
        self.capabilities = Some(self.detect_graphics_capabilities()?);
        
        // Initialize wgpu context if supported
        if self.capabilities.as_ref().unwrap().webgpu_supported || 
           self.capabilities.as_ref().unwrap().webgl_supported {
            self.wgpu_context = Some(WgpuContextHandle::new()?);
        }
        
        // Setup module integration
        self.setup_module_integration()?;
        
        self.initialized = true;
        Ok(())
    }
    
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.initialized {
            return Err("Module not initialized".into());
        }
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Cleanup canvas contexts
        self.canvas_contexts.clear();
        
        // Cleanup wgpu context
        if let Some(mut context) = self.wgpu_context.take() {
            context.cleanup()?;
        }
        
        self.capabilities = None;
        self.initialized = false;
        Ok(())
    }
}

impl GraphicsFoundations for GraphicsFoundationsModule {
    fn create_rendering_context(&self, canvas: &HtmlCanvasElement) -> Result<RenderingContext, GraphicsError> {
        let capabilities = self.capabilities.as_ref()
            .ok_or_else(|| GraphicsError::ContextInitializationFailed("Graphics capabilities not detected".to_string()))?;
        
        let canvas_id = canvas.id();
        if canvas_id.is_empty() {
            return Err(GraphicsError::CanvasNotAvailable("Canvas element must have an ID".to_string()));
        }
        
        let wgpu_context = if capabilities.webgpu_supported || capabilities.webgl_supported {
            self.wgpu_context.clone()
        } else {
            None
        };
        
        Ok(RenderingContext {
            wgpu_context,
            capabilities: capabilities.clone(),
            canvas_id,
        })
    }
    
    fn get_graphics_capabilities(&self) -> GraphicsCapabilities {
        self.capabilities.clone().unwrap_or_default()
    }
    
    fn is_graphics_available(&self) -> bool {
        self.capabilities.as_ref()
            .map(|caps| caps.webgpu_supported || caps.webgl_supported)
            .unwrap_or(false)
    }
    
    fn initialize_graphics(&mut self) -> Result<(), GraphicsError> {
        if self.capabilities.is_none() {
            self.capabilities = Some(self.detect_graphics_capabilities()?);
        }
        Ok(())
    }
    
    fn cleanup_graphics(&mut self) -> Result<(), GraphicsError> {
        self.canvas_contexts.clear();
        if let Some(context) = self.wgpu_context.take() {
            context.cleanup()?;
        }
        Ok(())
    }
}

impl Default for GraphicsFoundationsModule {
    fn default() -> Self {
        Self::new()
    }
}