//! # Presentation Layer Module
//!
//! The Presentation Layer module provides comprehensive UI coordination infrastructure
//! for the pitch-toy application. It manages coordination between different UI systems,
//! including future immersive rendering, debug overlays, and event routing.
//!
//! ## Architecture
//!
//! The Presentation Layer module provides:
//! - UI coordination between different rendering systems
//! - Debug overlay integration with conditional compilation
//! - Event routing infrastructure for UI interactions
//! - State synchronization framework across UI subsystems
//! - Performance monitoring for coordination overhead
//!
//! ## Key Components
//!
//! - [`UICoordinator`]: Core trait for UI coordination
//! - [`PresentationCoordinator`]: Main coordinator implementation
//! - [`ImmersiveRenderer`]: Interface for future graphics integration
//! - [`DebugOverlay`]: Debug UI overlay system (debug builds only)
//! - [`UIEvent`]: Event system for UI coordination

use std::rc::Rc;
use std::cell::RefCell;
use crate::modules::application_core::{Module, PriorityEventBus, ModuleId};

pub mod ui_coordinator;
pub mod presentation_coordinator;
pub mod immersive_renderer;
pub mod event_handler;
pub mod theme_manager;
pub mod theme_renderer;
pub mod theme_selection;
pub mod graphics_theme_integration;

// Debug overlay - conditionally compiled for debug builds only
#[cfg(debug_assertions)]
pub mod debug_overlay;

// Debug styling - conditionally compiled for debug builds only
#[cfg(debug_assertions)]
pub mod debug_styling;

// Re-exports for clean API surface
pub use ui_coordinator::*;
pub use presentation_coordinator::*;
pub use immersive_renderer::*;
pub use event_handler::*;
pub use theme_manager::*;
pub use theme_renderer::*;
pub use theme_selection::*;
pub use graphics_theme_integration::*;

#[cfg(debug_assertions)]
pub use debug_overlay::*;

#[cfg(debug_assertions)]
pub use debug_styling::*;

/// Presentation Layer Module - Core UI coordination
pub struct PresentationLayerModule {
    coordinator: PresentationCoordinator,
    event_bus: Option<Rc<RefCell<PriorityEventBus>>>,
    #[cfg(debug_assertions)]
    debug_overlay: Option<DebugOverlay>,
    initialized: bool,
    started: bool,
}

impl PresentationLayerModule {
    /// Create a new Presentation Layer module instance
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            coordinator: PresentationCoordinator::new()?,
            event_bus: None,
            #[cfg(debug_assertions)]
            debug_overlay: None,
            initialized: false,
            started: false,
        })
    }

    /// Setup UI event integration with main application
    pub fn setup_ui_event_integration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(event_bus) = &self.event_bus {
            self.coordinator.connect_event_bus(event_bus.clone())?;
            
            #[cfg(debug_assertions)]
            if let Some(ref mut debug_overlay) = self.debug_overlay {
                debug_overlay.connect_event_bus(event_bus.clone())?;
            }
        }
        Ok(())
    }

    /// Connect event bus for UI coordination
    pub fn connect_event_bus(&mut self, event_bus: Rc<RefCell<PriorityEventBus>>) -> Result<(), Box<dyn std::error::Error>> {
        self.event_bus = Some(event_bus);
        self.setup_ui_event_integration()
    }

    /// Initialize debug overlay system (debug builds only)
    #[cfg(debug_assertions)]
    pub fn initialize_debug_overlay(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.debug_overlay.is_none() {
            self.debug_overlay = Some(DebugOverlay::new()?);
        }
        Ok(())
    }
}

impl Module for PresentationLayerModule {
    fn module_id(&self) -> ModuleId {
        ModuleId::new("presentation_layer")
    }

    fn module_name(&self) -> &str {
        "Presentation Layer"
    }

    fn module_version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dependencies(&self) -> Vec<ModuleId> {
        vec![
            ModuleId::new("application_core"),
        ]
    }

    fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.initialized {
            return Ok(());
        }

        // Initialize UI coordinator
        self.coordinator.initialize()?;
        
        // Initialize debug overlay (debug builds only)
        #[cfg(debug_assertions)]
        self.initialize_debug_overlay()?;
        
        // Connect to application event system
        self.setup_ui_event_integration()?;
        
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

        // Start UI coordination
        self.coordinator.start()?;
        
        self.started = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.started {
            return Ok(());
        }

        self.coordinator.stop()?;
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

        self.coordinator.shutdown()?;
        
        #[cfg(debug_assertions)]
        if let Some(ref mut debug_overlay) = self.debug_overlay {
            debug_overlay.shutdown()?;
        }
        
        self.initialized = false;
        Ok(())
    }
}

impl Default for PresentationLayerModule {
    fn default() -> Self {
        Self::new().expect("Failed to create PresentationLayerModule")
    }
}