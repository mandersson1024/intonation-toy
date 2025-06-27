//! # Graphics Theme Integration
//!
//! This module provides integration between the theme system and Graphics
//! Foundations module. It manages theme-aware graphics initialization,
//! resource allocation, and rendering pipeline management.

use crate::modules::presentation_layer::theme_manager::*;
use crate::modules::presentation_layer::theme_renderer::*;
use crate::modules::graphics_foundations::{GraphicsFoundations, GraphicsError, GraphicsCapabilities};
use std::rc::Rc;
use std::cell::RefCell;

/// Graphics theme integration errors
#[derive(Debug, Clone)]
pub enum GraphicsThemeError {
    /// Graphics system not initialized
    GraphicsNotInitialized,
    /// Theme renderer not available
    ThemeRendererNotAvailable,
    /// Graphics capabilities incompatible with theme requirements
    IncompatibleCapabilities(String),
    /// Graphics system error during theme operations
    GraphicsError(String),
    /// Theme system error during graphics operations
    ThemeError(ThemeError),
}

impl std::fmt::Display for GraphicsThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GraphicsNotInitialized => write!(f, "Graphics system not initialized"),
            Self::ThemeRendererNotAvailable => write!(f, "Theme renderer not available"),
            Self::IncompatibleCapabilities(msg) => write!(f, "Incompatible graphics capabilities: {}", msg),
            Self::GraphicsError(msg) => write!(f, "Graphics system error: {}", msg),
            Self::ThemeError(err) => write!(f, "Theme system error: {}", err),
        }
    }
}

impl std::error::Error for GraphicsThemeError {}

impl From<ThemeError> for GraphicsThemeError {
    fn from(err: ThemeError) -> Self {
        Self::ThemeError(err)
    }
}

impl From<GraphicsError> for GraphicsThemeError {
    fn from(err: GraphicsError) -> Self {
        Self::GraphicsError(err.to_string())
    }
}

/// Graphics theme integration manager
pub struct GraphicsThemeIntegration {
    graphics_module: Option<Rc<RefCell<dyn GraphicsFoundations>>>,
    theme_renderer: Option<Box<dyn ThemeRenderer>>,
    theme_registry: Rc<RefCell<ThemeRegistry>>,
    graphics_capabilities: Option<GraphicsCapabilities>,
    initialized: bool,
}

impl GraphicsThemeIntegration {
    /// Create new graphics theme integration
    pub fn new(theme_registry: Rc<RefCell<ThemeRegistry>>) -> Self {
        Self {
            graphics_module: None,
            theme_renderer: None,
            theme_registry,
            graphics_capabilities: None,
            initialized: false,
        }
    }
    
    /// Initialize with graphics module
    pub fn initialize_with_graphics(
        &mut self,
        graphics: Rc<RefCell<dyn GraphicsFoundations>>
    ) -> Result<(), GraphicsThemeError> {
        if self.initialized {
            return Ok(());
        }
        
        // Store graphics module reference
        self.graphics_module = Some(graphics.clone());
        
        // Get graphics capabilities
        self.graphics_capabilities = {
            let graphics_ref = graphics.borrow();
            Some(graphics_ref.get_graphics_capabilities())
        };
        
        // Initialize theme renderer with graphics context
        let mut theme_renderer = ThemeRendererImpl::new(self.theme_registry.clone());
        {
            let graphics_ref = graphics.borrow();
            theme_renderer.initialize_with_graphics(graphics_ref.as_ref())
                .map_err(|e| GraphicsThemeError::GraphicsError(e.to_string()))?;
        }
        self.theme_renderer = Some(Box::new(theme_renderer));
        
        // Validate theme compatibility with graphics capabilities
        self.validate_all_themes_compatibility()?;
        
        self.initialized = true;
        Ok(())
    }
    
    /// Set active theme with graphics integration
    pub fn set_active_theme(&mut self, theme: UserThemeChoice) -> Result<(), GraphicsThemeError> {
        if !self.initialized {
            return Err(GraphicsThemeError::GraphicsNotInitialized);
        }
        
        let theme_renderer = self.theme_renderer.as_mut()
            .ok_or(GraphicsThemeError::ThemeRendererNotAvailable)?;
        
        // Validate theme compatibility first
        self.validate_theme_compatibility(theme)?;
        
        // Set theme in renderer
        theme_renderer.set_active_theme(theme)
            .map_err(|e| GraphicsThemeError::GraphicsError(e.to_string()))?;
        
        // Update theme registry
        {
            let mut registry = self.theme_registry.borrow_mut();
            registry.set_theme(theme)?;
        }
        
        Ok(())
    }
    
    /// Get current active theme
    pub fn get_active_theme(&self) -> Option<UserThemeChoice> {
        self.theme_renderer.as_ref()
            .and_then(|renderer| renderer.get_active_theme())
    }
    
    /// Validate theme compatibility with current graphics capabilities
    pub fn validate_theme_compatibility(&self, theme: UserThemeChoice) -> Result<(), GraphicsThemeError> {
        let capabilities = self.graphics_capabilities.as_ref()
            .ok_or(GraphicsThemeError::GraphicsNotInitialized)?;
        
        let registry = self.theme_registry.borrow();
        let theme_def = registry.get_theme(theme)?;
        
        // Check WebGPU/WebGL support
        if !capabilities.webgpu_supported && !capabilities.webgl_supported {
            return Err(GraphicsThemeError::IncompatibleCapabilities(
                "Neither WebGPU nor WebGL is supported".to_string()
            ));
        }
        
        // Check compute shader support for complex themes
        if theme_def.shader_variants.compute_shaders.len() > 5 && !capabilities.webgpu_supported {
            return Err(GraphicsThemeError::IncompatibleCapabilities(
                format!("Theme '{}' requires WebGPU for compute shaders", theme_def.display_name)
            ));
        }
        
        // Check particle system requirements
        if theme_def.particle_systems.max_particles > 1000 && !capabilities.webgpu_supported {
            return Err(GraphicsThemeError::IncompatibleCapabilities(
                format!("Theme '{}' requires WebGPU for high particle count", theme_def.display_name)
            ));
        }
        
        Ok(())
    }
    
    /// Validate all themes compatibility
    fn validate_all_themes_compatibility(&self) -> Result<(), GraphicsThemeError> {
        for choice in UserThemeChoice::all() {
            if let Err(e) = self.validate_theme_compatibility(choice) {
                // Log warning but don't fail initialization
                web_sys::console::warn_1(&format!(
                    "Theme {:?} not compatible with current graphics capabilities: {}",
                    choice, e
                ).into());
            }
        }
        Ok(())
    }
    
    /// Get graphics capabilities
    pub fn get_graphics_capabilities(&self) -> Option<&GraphicsCapabilities> {
        self.graphics_capabilities.as_ref()
    }
    
    /// Get theme rendering statistics
    pub fn get_theme_rendering_statistics(&self) -> Option<ThemeRenderingStatistics> {
        self.theme_renderer.as_ref()
            .map(|renderer| renderer.get_rendering_statistics())
    }
    
    /// Hot-swap theme without interrupting rendering
    pub fn hot_swap_theme(&mut self, theme: UserThemeChoice) -> Result<(), GraphicsThemeError> {
        if !self.initialized {
            return Err(GraphicsThemeError::GraphicsNotInitialized);
        }
        
        let theme_renderer = self.theme_renderer.as_mut()
            .ok_or(GraphicsThemeError::ThemeRendererNotAvailable)?;
        
        // Validate compatibility
        self.validate_theme_compatibility(theme)?;
        
        // Perform hot-swap
        theme_renderer.hot_swap_theme(theme)
            .map_err(|e| GraphicsThemeError::GraphicsError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Get available themes filtered by graphics compatibility
    pub fn get_compatible_themes(&self) -> Vec<UserThemeChoice> {
        UserThemeChoice::all()
            .into_iter()
            .filter(|&choice| self.validate_theme_compatibility(choice).is_ok())
            .collect()
    }
    
    /// Cleanup graphics theme integration
    pub fn cleanup(&mut self) -> Result<(), GraphicsThemeError> {
        if let Some(theme_renderer) = &mut self.theme_renderer {
            // Cleanup all theme resources
            for choice in UserThemeChoice::all() {
                let _ = theme_renderer.cleanup_theme_resources(choice);
            }
        }
        
        self.theme_renderer = None;
        self.graphics_module = None;
        self.graphics_capabilities = None;
        self.initialized = false;
        
        Ok(())
    }
}

/// Graphics theme integration performance metrics
#[derive(Debug, Clone)]
pub struct GraphicsThemePerformanceMetrics {
    pub theme_switch_duration_ms: f32,
    pub graphics_capabilities_score: f32,
    pub memory_usage_mb: f32,
    pub active_theme: Option<UserThemeChoice>,
    pub compatible_themes_count: usize,
    pub total_themes_count: usize,
}

impl GraphicsThemeIntegration {
    /// Get performance metrics for monitoring
    pub fn get_performance_metrics(&self) -> GraphicsThemePerformanceMetrics {
        let compatible_themes = self.get_compatible_themes();
        let rendering_stats = self.get_theme_rendering_statistics();
        
        GraphicsThemePerformanceMetrics {
            theme_switch_duration_ms: rendering_stats
                .as_ref()
                .map(|stats| stats.theme_switch_time_ms)
                .unwrap_or(0.0),
            graphics_capabilities_score: self.calculate_capabilities_score(),
            memory_usage_mb: rendering_stats
                .as_ref()
                .map(|stats| stats.memory_usage_bytes as f32 / (1024.0 * 1024.0))
                .unwrap_or(0.0),
            active_theme: self.get_active_theme(),
            compatible_themes_count: compatible_themes.len(),
            total_themes_count: UserThemeChoice::all().len(),
        }
    }
    
    /// Calculate graphics capabilities score (0.0 - 1.0)
    fn calculate_capabilities_score(&self) -> f32 {
        if let Some(capabilities) = &self.graphics_capabilities {
            let mut score = 0.0;
            
            // WebGPU support is best
            if capabilities.webgpu_supported {
                score += 0.6;
            } else if capabilities.webgl_supported {
                score += 0.3;
            }
            
            // Additional capabilities
            if capabilities.max_texture_size >= 2048 {
                score += 0.2;
            }
            
            if !capabilities.supported_formats.is_empty() {
                score += 0.2;
            }
            
            score.min(1.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_graphics_theme_integration_creation() {
        let registry = Rc::new(RefCell::new(ThemeRegistry::new()));
        let integration = GraphicsThemeIntegration::new(registry);
        
        assert!(!integration.initialized);
        assert!(integration.graphics_module.is_none());
        assert!(integration.theme_renderer.is_none());
    }
    
    #[test]
    fn test_theme_compatibility_validation() {
        let registry = Rc::new(RefCell::new(ThemeRegistry::new()));
        let integration = GraphicsThemeIntegration::new(registry);
        
        // Without graphics capabilities, should fail
        let result = integration.validate_theme_compatibility(UserThemeChoice::Playful);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_compatible_themes_filtering() {
        let registry = Rc::new(RefCell::new(ThemeRegistry::new()));
        let integration = GraphicsThemeIntegration::new(registry);
        
        // Without graphics initialization, no themes should be compatible
        let compatible = integration.get_compatible_themes();
        assert_eq!(compatible.len(), 0);
    }
    
    #[test]
    fn test_performance_metrics() {
        let registry = Rc::new(RefCell::new(ThemeRegistry::new()));
        let integration = GraphicsThemeIntegration::new(registry);
        
        let metrics = integration.get_performance_metrics();
        assert_eq!(metrics.theme_switch_duration_ms, 0.0);
        assert_eq!(metrics.graphics_capabilities_score, 0.0);
        assert_eq!(metrics.compatible_themes_count, 0);
        assert_eq!(metrics.total_themes_count, 2);
    }
}