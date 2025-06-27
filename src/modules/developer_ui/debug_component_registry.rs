//! # Debug Component Registry
//!
//! Provides component management and overlay coordination for debug components.
//! This registry manages the lifecycle, registration, and discovery of debug
//! components within the Developer UI module.

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::modules::application_core::ModuleError;
use super::DebugComponent;

/// Registry for managing debug components
#[cfg(debug_assertions)]
pub struct DebugComponentRegistry {
    components: HashMap<String, Box<dyn DebugComponent>>,
    active_components: Vec<String>,
    overlay_state: OverlayState,
}

/// State management for debug overlay system
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct OverlayState {
    pub visible: bool,
    pub position: OverlayPosition,
    pub size: OverlaySize,
    pub transparency: f32,
}

/// Position configuration for debug overlays
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct OverlayPosition {
    pub x: f32,
    pub y: f32,
}

/// Size configuration for debug overlays
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct OverlaySize {
    pub width: f32,
    pub height: f32,
}

#[cfg(debug_assertions)]
impl Default for OverlayState {
    fn default() -> Self {
        Self {
            visible: true,
            position: OverlayPosition { x: 10.0, y: 10.0 },
            size: OverlaySize { width: 400.0, height: 300.0 },
            transparency: 0.9,
        }
    }
}

#[cfg(debug_assertions)]
impl DebugComponentRegistry {
    /// Create a new debug component registry
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            active_components: Vec::new(),
            overlay_state: OverlayState::default(),
        }
    }

    /// Register a debug component
    pub fn register_component(&mut self, component: Box<dyn DebugComponent>) -> Result<(), ModuleError> {
        let name = component.name().to_string();
        
        if self.components.contains_key(&name) {
            return Err(ModuleError::InitializationError(
                format!("Debug component '{}' is already registered", name)
            ));
        }

        self.components.insert(name.clone(), component);
        
        // Initialize the component if registry is active
        if let Some(component) = self.components.get_mut(&name) {
            component.initialize()?;
        }
        
        Ok(())
    }

    /// Unregister a debug component
    pub fn unregister_component(&mut self, name: &str) -> Result<(), ModuleError> {
        if let Some(mut component) = self.components.remove(name) {
            component.shutdown()?;
        }
        
        // Remove from active components list
        self.active_components.retain(|comp_name| comp_name != name);
        
        Ok(())
    }

    /// Activate a debug component for display
    pub fn activate_component(&mut self, name: &str) -> Result<(), ModuleError> {
        if !self.components.contains_key(name) {
            return Err(ModuleError::RuntimeError(
                format!("Debug component '{}' not found", name)
            ));
        }

        if !self.active_components.contains(&name.to_string()) {
            self.active_components.push(name.to_string());
        }
        
        Ok(())
    }

    /// Deactivate a debug component
    pub fn deactivate_component(&mut self, name: &str) -> Result<(), ModuleError> {
        self.active_components.retain(|comp_name| comp_name != name);
        Ok(())
    }

    /// Get list of registered component names
    pub fn get_registered_components(&self) -> Vec<String> {
        self.components.keys().cloned().collect()
    }

    /// Get list of active component names
    pub fn get_active_components(&self) -> Vec<String> {
        self.active_components.clone()
    }

    /// Check if a component is registered
    pub fn is_component_registered(&self, name: &str) -> bool {
        self.components.contains_key(name)
    }

    /// Check if a component is active
    pub fn is_component_active(&self, name: &str) -> bool {
        self.active_components.contains(&name.to_string())
    }

    /// Get current overlay state
    pub fn get_overlay_state(&self) -> &OverlayState {
        &self.overlay_state
    }

    /// Update overlay state
    pub fn update_overlay_state(&mut self, state: OverlayState) {
        self.overlay_state = state;
    }

    /// Toggle overlay visibility
    pub fn toggle_overlay_visibility(&mut self) {
        self.overlay_state.visible = !self.overlay_state.visible;
    }

    /// Clear all components and reset registry
    pub fn clear(&mut self) {
        // Shutdown all components
        for (_, mut component) in self.components.drain() {
            let _ = component.shutdown();
        }
        
        self.active_components.clear();
        self.overlay_state = OverlayState::default();
    }

    /// Initialize all registered components
    pub fn initialize_all_components(&mut self) -> Result<(), ModuleError> {
        for (name, component) in self.components.iter_mut() {
            if let Err(e) = component.initialize() {
                eprintln!("Failed to initialize debug component '{}': {:?}", name, e);
            }
        }
        Ok(())
    }

    /// Shutdown all registered components
    pub fn shutdown_all_components(&mut self) -> Result<(), ModuleError> {
        for (name, component) in self.components.iter_mut() {
            if let Err(e) = component.shutdown() {
                eprintln!("Failed to shutdown debug component '{}': {:?}", name, e);
            }
        }
        Ok(())
    }

    /// Get component count for debugging
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Get active component count
    pub fn active_component_count(&self) -> usize {
        self.active_components.len()
    }
}

/// Trait for components that support debug overlay coordination
#[cfg(debug_assertions)]
pub trait OverlayCoordinator {
    /// Handle overlay position changes
    fn on_overlay_position_changed(&mut self, position: OverlayPosition) -> Result<(), ModuleError>;
    
    /// Handle overlay size changes
    fn on_overlay_size_changed(&mut self, size: OverlaySize) -> Result<(), ModuleError>;
    
    /// Handle overlay visibility changes
    fn on_overlay_visibility_changed(&mut self, visible: bool) -> Result<(), ModuleError>;
}

#[cfg(test)]
#[cfg(debug_assertions)]
mod tests {
    use super::*;

    struct MockDebugComponent {
        name: &'static str,
        initialized: bool,
        active: bool,
    }

    impl MockDebugComponent {
        fn new(name: &'static str) -> Self {
            Self {
                name,
                initialized: false,
                active: false,
            }
        }
    }

    impl DebugComponent for MockDebugComponent {
        fn name(&self) -> &'static str {
            self.name
        }

        fn initialize(&mut self) -> Result<(), ModuleError> {
            self.initialized = true;
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), ModuleError> {
            self.initialized = false;
            self.active = false;
            Ok(())
        }

        fn is_active(&self) -> bool {
            self.active
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = DebugComponentRegistry::new();
        assert_eq!(registry.component_count(), 0);
        assert_eq!(registry.active_component_count(), 0);
    }

    #[test]
    fn test_component_registration() {
        let mut registry = DebugComponentRegistry::new();
        let component = MockDebugComponent::new("test_component");
        
        assert!(registry.register_component(Box::new(component)).is_ok());
        assert_eq!(registry.component_count(), 1);
        assert!(registry.is_component_registered("test_component"));
    }

    #[test]
    fn test_component_activation() {
        let mut registry = DebugComponentRegistry::new();
        let component = MockDebugComponent::new("test_component");
        
        registry.register_component(Box::new(component)).unwrap();
        assert!(registry.activate_component("test_component").is_ok());
        assert!(registry.is_component_active("test_component"));
        assert_eq!(registry.active_component_count(), 1);
    }

    #[test]
    fn test_overlay_state_management() {
        let mut registry = DebugComponentRegistry::new();
        let initial_state = registry.get_overlay_state();
        assert!(initial_state.visible);

        registry.toggle_overlay_visibility();
        assert!(!registry.get_overlay_state().visible);
    }
} 