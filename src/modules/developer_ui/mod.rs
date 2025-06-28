//! # Developer UI Module
//!
//! This module provides comprehensive debugging tools and developer interfaces
//! for the pitch-toy application. The entire module is conditionally compiled
//! for debug builds only to ensure zero impact on production builds.
//!
//! ## Architecture
//!
//! The Developer UI module is completely separate from the presentation layer
//! and provides:
//! - Debug component registration and management
//! - Overlay management for debug interfaces
//! - Developer-specific hooks and state management
//! - Performance monitoring and debugging tools
//!
//! ## Conditional Compilation
//!
//! This module uses `#[cfg(debug_assertions)]` to ensure complete exclusion
//! from production builds, maintaining zero build size and runtime impact.

use std::rc::Rc;
use std::cell::RefCell;
use crate::modules::application_core::{Module, PriorityEventBus, ModuleId, RegistryError};

// Conditionally compile the entire module for debug builds only
#[cfg(debug_assertions)]
pub mod debug_app;

#[cfg(debug_assertions)]
pub mod overlay_manager;

#[cfg(debug_assertions)]
pub mod components;

#[cfg(debug_assertions)]
pub mod hooks;

#[cfg(debug_assertions)]
pub mod utils;

#[cfg(debug_assertions)]
pub mod debug_component_registry;

#[cfg(debug_assertions)]
pub mod startup_shutdown_coordinator;

#[cfg(test)]
pub mod integration_tests;

#[cfg(debug_assertions)]
pub use debug_app::DebugApp;

#[cfg(debug_assertions)]
pub use overlay_manager::OverlayManager;

#[cfg(debug_assertions)]
pub use debug_component_registry::DebugComponentRegistry;

#[cfg(debug_assertions)]
pub use startup_shutdown_coordinator::StartupShutdownCoordinator;

/// Developer UI Module - Available only in debug builds
#[cfg(debug_assertions)]
pub struct DeveloperUIModule {
    debug_app: DebugApp,
    overlay_manager: OverlayManager,
    component_registry: DebugComponentRegistry,
    startup_shutdown_coordinator: StartupShutdownCoordinator,
    event_bus: Option<Rc<RefCell<PriorityEventBus>>>,
    initialized: bool,
    started: bool,
}

#[cfg(debug_assertions)]
impl DeveloperUIModule {
    /// Create a new Developer UI module instance
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            debug_app: DebugApp::new()?,
            overlay_manager: OverlayManager::new()?,
            component_registry: DebugComponentRegistry::new(),
            startup_shutdown_coordinator: StartupShutdownCoordinator::new(),
            event_bus: None,
            initialized: false,
            started: false,
        })
    }

    /// Register a debug component with the component registry
    pub fn register_debug_component<T: DebugComponent + 'static>(&mut self, component: T) -> Result<(), Box<dyn std::error::Error>> {
        self.component_registry.register_component(Box::new(component))
    }

    /// Register all available debug components
    pub fn register_all_debug_components(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Component registration will be implemented as components are migrated
        Ok(())
    }

    /// Setup debug event integration with main application
    pub fn setup_debug_event_integration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(event_bus) = &self.event_bus {
            self.debug_app.connect_event_bus(event_bus.clone())?;
            self.overlay_manager.connect_event_bus(event_bus.clone())?;
        }
        Ok(())
    }
    
    /// Create module with real-time event subscriptions
    pub fn new_with_subscriptions() -> Result<Self, Box<dyn std::error::Error>> {
        let mut module = Self::new()?;
        
        // Subscribe to audio events for real-time UI updates
        module.setup_audio_event_subscriptions();
        
        Ok(module)
    }
    
    /// Setup real-time audio event subscriptions
    fn setup_audio_event_subscriptions(&mut self) {
        // TODO: Implement actual event subscription when TypedEventBus integration is complete
        // For now, log that subscription capability is ready
        web_sys::console::log_1(&"DeveloperUI: Audio event subscription capability ready".into());
        
        // This will be implemented to subscribe to:
        // - AudioEvent::PitchDetected for real-time pitch display
        // - AudioEvent::DeviceChanged for device status display
        // - PerformanceEvent::AudioMetricsUpdate for performance monitoring display
    }
    
    // Event handler implementations (placeholders for full implementation)
    fn update_pitch_visualization(frequency: f32, clarity: f32, timestamp: std::time::Instant) {
        // TODO: Update real-time pitch visualization
        web_sys::console::log_1(&format!("Real-time pitch: {:.2}Hz (clarity: {:.2})", frequency, clarity).into());
    }
    
    fn update_device_status(device: &Option<crate::types::AudioDeviceInfo>) {
        // TODO: Update device status in UI
        if let Some(device) = device {
            web_sys::console::log_1(&format!("Audio device changed to: {}", device.label).into());
        }
    }
    
    fn update_performance_display(latency_ms: f32, cpu_usage: f32) {
        // TODO: Update performance metrics in UI
        web_sys::console::log_1(&format!("Audio performance: {:.1}ms latency, {:.1}% CPU", latency_ms, cpu_usage).into());
    }
}

#[cfg(debug_assertions)]
impl Module for DeveloperUIModule {
    fn module_id(&self) -> ModuleId {
        ModuleId::new("developer_ui")
    }

    fn module_name(&self) -> &str {
        "Developer UI"
    }

    fn module_version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dependencies(&self) -> Vec<ModuleId> {
        vec![
            ModuleId::new("application_core"),
            ModuleId::new("audio_foundations"),
        ]
    }

    fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.initialized {
            return Ok(());
        }

        // Register debug components
        self.register_all_debug_components()?;
        
        // Initialize debug overlay system
        self.overlay_manager.initialize()?;
        
        // Connect to main application event system
        self.setup_debug_event_integration()?;
        
        // Initialize debug app
        self.debug_app.initialize()?;
        
        self.initialized = true;
        Ok(())
    }

    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.initialized {
            return Err("Module not initialized".into());
        }
        if self.started {
            return Ok(());
        }

        // Start debug subsystems
        self.started = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.started {
            return Ok(());
        }

        self.started = false;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.started {
            self.stop()?;
        }

        if !self.initialized {
            return Ok(());
        }

        self.debug_app.shutdown()?;
        self.overlay_manager.shutdown()?;
        self.component_registry.clear();
        
        self.initialized = false;
        Ok(())
    }
}

/// Trait for debug components that can be registered with the Developer UI
#[cfg(debug_assertions)]
pub trait DebugComponent {
    /// Get the component name
    fn name(&self) -> &'static str;
    
    /// Initialize the debug component
    fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Shutdown the debug component
    fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Check if the component is currently active
    fn is_active(&self) -> bool;
}

// Re-export components for easy access when in debug mode
#[cfg(debug_assertions)]
pub mod exports {
    pub use super::components::*;
    pub use super::hooks::*;
    pub use super::utils::*;
    pub use super::{DeveloperUIModule, DebugComponent};
}

// For production builds, provide empty exports to prevent compilation errors
#[cfg(not(debug_assertions))]
pub mod exports {
    // Empty module for production builds
} 