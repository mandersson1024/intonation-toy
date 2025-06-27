use super::GraphicsError;
use web_sys::HtmlCanvasElement;
use wasm_bindgen::JsCast;
use std::rc::Rc;

/// Handle for managing wgpu context lifecycle
#[derive(Debug, Clone)]
pub struct WgpuContextHandle {
    inner: Rc<WgpuContextInner>,
}

/// Internal wgpu context implementation
#[derive(Debug)]
struct WgpuContextInner {
    // Placeholder for future wgpu integration
    // Will contain: Instance, Adapter, Device, Queue, Surface
    canvas_id: Option<String>,
    initialized: bool,
}

/// Trait for wgpu context management
pub trait WgpuContext {
    /// Initialize the wgpu context with a canvas
    fn initialize_with_canvas(&mut self, canvas: &HtmlCanvasElement) -> Result<(), GraphicsError>;
    
    /// Get the current surface configuration
    fn get_surface_config(&self) -> Option<SurfaceConfiguration>;
    
    /// Resize the surface to match canvas dimensions
    fn resize_surface(&mut self, width: u32, height: u32) -> Result<(), GraphicsError>;
    
    /// Check if the context is initialized
    fn is_initialized(&self) -> bool;
    
    /// Cleanup the context resources
    fn cleanup(&mut self) -> Result<(), GraphicsError>;
}

/// Surface configuration for wgpu rendering
#[derive(Debug, Clone)]
pub struct SurfaceConfiguration {
    pub width: u32,
    pub height: u32,
    pub format: SurfaceFormat,
    pub present_mode: PresentMode,
}

/// Surface format options
#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceFormat {
    Bgra8Unorm,
    Rgba8Unorm,
}

/// Present mode options for surface rendering
#[derive(Debug, Clone, PartialEq)]
pub enum PresentMode {
    Fifo,      // VSync
    Immediate, // No VSync
    Mailbox,   // Triple buffering
}

impl WgpuContextHandle {
    /// Create a new wgpu context handle
    pub fn new() -> Result<Self, GraphicsError> {
        let inner = WgpuContextInner {
            canvas_id: None,
            initialized: false,
        };
        
        Ok(Self {
            inner: Rc::new(inner),
        })
    }
}

impl WgpuContext for WgpuContextHandle {
    fn initialize_with_canvas(&mut self, canvas: &HtmlCanvasElement) -> Result<(), GraphicsError> {
        let canvas_id = canvas.id();
        if canvas_id.is_empty() {
            return Err(GraphicsError::CanvasNotAvailable(
                "Canvas must have an ID for wgpu initialization".to_string()
            ));
        }
        
        // Get canvas dimensions
        let width = canvas.width();
        let height = canvas.height();
        
        if width == 0 || height == 0 {
            return Err(GraphicsError::InvalidConfiguration(
                "Canvas dimensions must be greater than 0".to_string()
            ));
        }
        
        // TODO: Initialize actual wgpu context
        // This is a placeholder implementation
        // Real implementation would:
        // 1. Create wgpu::Instance
        // 2. Request adapter with canvas surface
        // 3. Request device and queue
        // 4. Create surface for canvas
        // 5. Configure surface with appropriate format
        
        // For now, we'll simulate successful initialization
        // Since we're in WASM single-threaded context, we can safely clone and modify
        let new_inner = WgpuContextInner {
            canvas_id: Some(canvas_id),
            initialized: true,
        };
        self.inner = Rc::new(new_inner);
        
        Ok(())
    }
    
    fn get_surface_config(&self) -> Option<SurfaceConfiguration> {
        if !self.inner.initialized {
            return None;
        }
        
        // TODO: Return actual surface configuration
        // For now, return default configuration
        Some(SurfaceConfiguration {
            width: 800,
            height: 600,
            format: SurfaceFormat::Bgra8Unorm,
            present_mode: PresentMode::Fifo,
        })
    }
    
    fn resize_surface(&mut self, width: u32, height: u32) -> Result<(), GraphicsError> {
        if !self.inner.initialized {
            return Err(GraphicsError::ContextInitializationFailed(
                "Context not initialized".to_string()
            ));
        }
        
        if width == 0 || height == 0 {
            return Err(GraphicsError::InvalidConfiguration(
                "Surface dimensions must be greater than 0".to_string()
            ));
        }
        
        // TODO: Resize actual wgpu surface
        // For now, just validate the operation
        
        Ok(())
    }
    
    fn is_initialized(&self) -> bool {
        self.inner.initialized
    }
    
    fn cleanup(&mut self) -> Result<(), GraphicsError> {
        // TODO: Cleanup actual wgpu resources
        // For now, just reset state
        
        // Since we're in WASM single-threaded context, we can safely replace the inner
        let new_inner = WgpuContextInner {
            canvas_id: None,
            initialized: false,
        };
        self.inner = Rc::new(new_inner);
        
        Ok(())
    }
}

/// Canvas integration utilities for wgpu web backend
pub struct CanvasManager {
    canvas_contexts: std::collections::HashMap<String, WgpuContextHandle>,
}

impl CanvasManager {
    /// Create a new canvas manager
    pub fn new() -> Self {
        Self {
            canvas_contexts: std::collections::HashMap::new(),
        }
    }
    
    /// Acquire and validate a canvas element for wgpu rendering
    pub fn acquire_canvas(&mut self, canvas_id: &str) -> Result<HtmlCanvasElement, GraphicsError> {
        let window = web_sys::window()
            .ok_or_else(|| GraphicsError::CanvasNotAvailable("Window not available".to_string()))?;
        
        let document = window.document()
            .ok_or_else(|| GraphicsError::CanvasNotAvailable("Document not available".to_string()))?;
        
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| GraphicsError::CanvasNotAvailable(format!("Canvas with ID '{}' not found", canvas_id)))?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| GraphicsError::CanvasNotAvailable(format!("Element '{}' is not a canvas", canvas_id)))?;
        
        // Validate canvas dimensions
        if canvas.width() == 0 || canvas.height() == 0 {
            return Err(GraphicsError::InvalidConfiguration(
                "Canvas dimensions must be greater than 0".to_string()
            ));
        }
        
        Ok(canvas)
    }
    
    /// Initialize wgpu context for a canvas
    pub fn initialize_canvas_context(&mut self, canvas_id: &str) -> Result<(), GraphicsError> {
        let canvas = self.acquire_canvas(canvas_id)?;
        let mut context = WgpuContextHandle::new()?;
        
        context.initialize_with_canvas(&canvas)?;
        self.canvas_contexts.insert(canvas_id.to_string(), context);
        
        Ok(())
    }
    
    /// Get a canvas context by ID
    pub fn get_canvas_context(&self, canvas_id: &str) -> Option<&WgpuContextHandle> {
        self.canvas_contexts.get(canvas_id)
    }
    
    /// Get a mutable canvas context by ID
    pub fn get_canvas_context_mut(&mut self, canvas_id: &str) -> Option<&mut WgpuContextHandle> {
        self.canvas_contexts.get_mut(canvas_id)
    }
    
    /// Handle canvas resize events
    pub fn handle_canvas_resize(&mut self, canvas_id: &str, width: u32, height: u32) -> Result<(), GraphicsError> {
        let context = self.canvas_contexts.get_mut(canvas_id)
            .ok_or_else(|| GraphicsError::CanvasNotAvailable(format!("No context for canvas '{}'", canvas_id)))?;
        
        context.resize_surface(width, height)
    }
    
    /// Cleanup canvas context
    pub fn cleanup_canvas_context(&mut self, canvas_id: &str) -> Result<(), GraphicsError> {
        if let Some(mut context) = self.canvas_contexts.remove(canvas_id) {
            context.cleanup()?;
        }
        Ok(())
    }
    
    /// Cleanup all canvas contexts
    pub fn cleanup_all_contexts(&mut self) -> Result<(), GraphicsError> {
        for (_, mut context) in self.canvas_contexts.drain() {
            context.cleanup()?;
        }
        Ok(())
    }
}

impl Default for CanvasManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SurfaceConfiguration {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            format: SurfaceFormat::Bgra8Unorm,
            present_mode: PresentMode::Fifo,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wgpu_context_creation() {
        let context = WgpuContextHandle::new();
        assert!(context.is_ok());
        
        let context = context.unwrap();
        assert!(!context.is_initialized());
    }
    
    #[test]
    fn test_surface_configuration_default() {
        let config = SurfaceConfiguration::default();
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert_eq!(config.format, SurfaceFormat::Bgra8Unorm);
        assert_eq!(config.present_mode, PresentMode::Fifo);
    }
    
    #[test]
    fn test_canvas_manager_creation() {
        let manager = CanvasManager::new();
        assert_eq!(manager.canvas_contexts.len(), 0);
    }
}