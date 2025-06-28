//! Pitch-Toy Modular Architecture
//! 
//! This module provides the complete modular system infrastructure for the application.

pub mod application_core;
pub mod audio_foundations;
pub mod data_management;
pub mod developer_ui;
pub mod graphics_foundations;
pub mod platform_abstraction;
pub mod presentation_layer;

// Re-export key types for easy access
pub use application_core::{
    ApplicationLifecycleCoordinator, 
    ApplicationConfig, 
    ApplicationState, 
    CoreError,
    ModuleId,
    ModuleState
};

pub use audio_foundations::AudioFoundationsModule;

#[cfg(debug_assertions)]
pub use developer_ui::DeveloperUIModule;