//! # Debug Component Registry
//!
//! Provides component management and overlay coordination for debug components.
//! This registry manages the lifecycle, registration, and discovery of debug
//! components within the Developer UI module.

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
// ModuleError replaced with Box<dyn std::error::Error>
use super::DebugComponent;
use crate::modules::application_core::priority_event_bus::PriorityEventBus;
use crate::modules::application_core::event_bus::{Event, EventPriority};
use crate::modules::developer_ui::hooks::use_event_subscription::EventSubscriptionHandle;

/// Debug component event for registry lifecycle events
#[derive(Debug, Clone)]
pub struct DebugComponentEvent {
    pub component_name: String,
    pub event_type: DebugComponentEventType,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub enum DebugComponentEventType {
    Registered,
    Unregistered,
    Activated,
    Deactivated,
    InitializationFailed(String),
    ShutdownFailed(String),
}

impl Event for DebugComponentEvent {
    fn event_type(&self) -> &'static str {
        "DebugComponentEvent"
    }

    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    fn priority(&self) -> EventPriority {
        EventPriority::Low
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Enhanced registry for managing debug components with event bus integration
#[cfg(debug_assertions)]
pub struct DebugComponentRegistry {
    components: HashMap<String, Box<dyn DebugComponent>>,
    active_components: Vec<String>,
    overlay_state: OverlayState,
    event_bus: Option<Rc<RefCell<PriorityEventBus>>>,
    event_subscriptions: HashMap<String, EventSubscriptionHandle>,
    subscription_counter: u64,
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
            event_bus: None,
            event_subscriptions: HashMap::new(),
            subscription_counter: 0,
        }
    }

    /// Create a new registry with event bus integration
    pub fn with_event_bus(event_bus: Rc<RefCell<PriorityEventBus>>) -> Self {
        Self {
            components: HashMap::new(),
            active_components: Vec::new(),
            overlay_state: OverlayState::default(),
            event_bus: Some(event_bus),
            event_subscriptions: HashMap::new(),
            subscription_counter: 0,
        }
    }

    /// Set the event bus for the registry
    pub fn set_event_bus(&mut self, event_bus: Rc<RefCell<PriorityEventBus>>) {
        self.event_bus = Some(event_bus);
    }

    /// Publish a debug component event to the event bus
    fn publish_component_event(&self, component_name: &str, event_type: DebugComponentEventType) {
        if let Some(event_bus) = &self.event_bus {
            let event = DebugComponentEvent {
                component_name: component_name.to_string(),
                event_type,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            };

            // Note: PriorityEventBus would need a publish method
            // This is a placeholder for the actual implementation
            web_sys::console::log_1(&format!("Publishing debug component event: {:?}", event).into());
        }
    }

    /// Create event subscription for component lifecycle events
    pub fn subscribe_to_component_events<F>(&mut self, callback: F) -> Result<u64, Box<dyn std::error::Error>>
    where
        F: Fn(DebugComponentEvent) + 'static,
    {
        if let Some(event_bus) = &self.event_bus {
            self.subscription_counter += 1;
            let subscription_id = self.subscription_counter;
            
            // Create subscription handle with cleanup
            let handle = EventSubscriptionHandle::new(
                subscription_id,
                Some(event_bus.clone()),
                Some(Box::new(move || {
                    web_sys::console::log_1(&format!("Component event subscription {} cleaned up", subscription_id).into());
                })),
            );
            
            self.event_subscriptions.insert(format!("component_events_{}", subscription_id), handle);
            
            web_sys::console::log_1(&format!("Created component event subscription: {}", subscription_id).into());
            Ok(subscription_id)
        } else {
            Err("No event bus available for subscription".into())
        }
    }

    /// Register a debug component
    pub fn register_component(&mut self, component: Box<dyn DebugComponent>) -> Result<(), Box<dyn std::error::Error>> {
        let name = component.name().to_string();
        
        if self.components.contains_key(&name) {
            return Err((
                format!("Debug component '{}' is already registered", name)
            ).into());
        }

        self.components.insert(name.clone(), component);
        
        // Initialize the component if registry is active
        if let Some(component) = self.components.get_mut(&name) {
            match component.initialize() {
                Ok(()) => {
                    self.publish_component_event(&name, DebugComponentEventType::Registered);
                    web_sys::console::log_1(&format!("Successfully registered and initialized debug component: {}", name).into());
                }
                Err(e) => {
                    let error_msg = format!("Failed to initialize component: {:?}", e);
                    self.publish_component_event(&name, DebugComponentEventType::InitializationFailed(error_msg.clone()));
                    return Err(error_msg.into());
                }
            }
        } else {
            self.publish_component_event(&name, DebugComponentEventType::Registered);
        }
        
        Ok(())
    }

    /// Unregister a debug component
    pub fn unregister_component(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut component) = self.components.remove(name) {
            match component.shutdown() {
                Ok(()) => {
                    self.publish_component_event(name, DebugComponentEventType::Unregistered);
                    web_sys::console::log_1(&format!("Successfully unregistered debug component: {}", name).into());
                }
                Err(e) => {
                    let error_msg = format!("Failed to shutdown component: {:?}", e);
                    self.publish_component_event(name, DebugComponentEventType::ShutdownFailed(error_msg.clone()));
                    web_sys::console::warn_1(&format!("Component unregistered but shutdown failed: {}", error_msg).into());
                }
            }
        }
        
        // Remove from active components list
        self.active_components.retain(|comp_name| comp_name != name);
        
        Ok(())
    }

    /// Activate a debug component for display
    pub fn activate_component(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.components.contains_key(name) {
            return Err((
                format!("Debug component '{}' not found", name)
            ).into());
        }

        if !self.active_components.contains(&name.to_string()) {
            self.active_components.push(name.to_string());
            self.publish_component_event(name, DebugComponentEventType::Activated);
            web_sys::console::log_1(&format!("Activated debug component: {}", name).into());
        }
        
        Ok(())
    }

    /// Deactivate a debug component
    pub fn deactivate_component(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let was_active = self.active_components.contains(&name.to_string());
        self.active_components.retain(|comp_name| comp_name != name);
        
        if was_active {
            self.publish_component_event(name, DebugComponentEventType::Deactivated);
            web_sys::console::log_1(&format!("Deactivated debug component: {}", name).into());
        }
        
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
        for (name, mut component) in self.components.drain() {
            match component.shutdown() {
                Ok(()) => {
                    self.publish_component_event(&name, DebugComponentEventType::Unregistered);
                }
                Err(e) => {
                    let error_msg = format!("Failed to shutdown during clear: {:?}", e);
                    self.publish_component_event(&name, DebugComponentEventType::ShutdownFailed(error_msg));
                }
            }
        }
        
        // Clear all event subscriptions to prevent memory leaks
        self.event_subscriptions.clear();
        
        self.active_components.clear();
        self.overlay_state = OverlayState::default();
        self.subscription_counter = 0;
        
        web_sys::console::log_1(&"Debug component registry cleared".into());
    }

    /// Clean up all event subscriptions (memory leak prevention)
    pub fn cleanup_event_subscriptions(&mut self) {
        self.event_subscriptions.clear();
        web_sys::console::log_1(&"All debug component event subscriptions cleaned up".into());
    }

    /// Get event subscription count for monitoring
    pub fn get_subscription_count(&self) -> usize {
        self.event_subscriptions.len()
    }

    /// Initialize all registered components
    pub fn initialize_all_components(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for (name, component) in self.components.iter_mut() {
            if let Err(e) = component.initialize() {
                eprintln!("Failed to initialize debug component '{}': {:?}", name, e);
            }
        }
        Ok(())
    }

    /// Shutdown all registered components
    pub fn shutdown_all_components(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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
    fn on_overlay_position_changed(&mut self, position: OverlayPosition) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Handle overlay size changes
    fn on_overlay_size_changed(&mut self, size: OverlaySize) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Handle overlay visibility changes
    fn on_overlay_visibility_changed(&mut self, visible: bool) -> Result<(), Box<dyn std::error::Error>>;
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

        fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            self.initialized = true;
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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