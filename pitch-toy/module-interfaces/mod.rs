//! Module Interfaces for Three-Layer Architecture
//!
//! This module defines the communication interfaces between the three layers
//! of the pitch-toy architecture: Engine, Model, and Presentation.
//!
//! ## Interface Usage Pattern
//!
//! Each interface is a Rust struct containing `DataSource<T>` objects (from the
//! `observable_data` crate) and `Action<T>` objects (from the `action` crate).
//! These are not abstract interfaces but concrete structs with methods that
//! return setters, observers, triggers, and listeners.
//!
//! ## Example Usage
//!
//! ```rust
//! use std::rc::Rc;
//! use pitch_toy::module_interfaces::{
//!     engine_to_model::EngineToModelInterface,
//!     model_to_engine::ModelToEngineInterface,
//! };
//!
//! // Create interfaces (typically done in lib.rs)
//! let engine_to_model = Rc::new(EngineToModelInterface::new());
//! let model_to_engine = Rc::new(ModelToEngineInterface::new());
//!
//! // Engine layer extracts setters to push data
//! let audio_setter = engine_to_model.audio_analysis_setter();
//! let permission_setter = engine_to_model.permission_state_setter();
//!
//! // Engine layer extracts listeners to respond to actions
//! let permission_listener = model_to_engine.request_microphone_permission_listener();
//!
//! // Model layer extracts observers to read data
//! let audio_observer = engine_to_model.audio_analysis_observer();
//!
//! // Model layer extracts triggers to send actions
//! let permission_trigger = model_to_engine.request_microphone_permission_trigger();
//! ```
//!
//! ## Interface Communication Flow
//!
//! - **Engine → Model**: Audio analysis data, errors, permission state
//! - **Model → Engine**: Permission request actions
//! - **Model → Presentation**: Processed data for visualization  
//! - **Presentation → Model**: User actions and configuration changes

pub mod engine_to_model;
pub mod model_to_engine;
pub mod model_to_presentation;
pub mod presentation_to_model;
pub mod debug_actions;

/// Three-layer architecture interface bundle
/// 
/// Contains all the interfaces needed to connect the three layers
/// (engine, model, presentation) using the defined communication patterns.
pub struct ThreeLayerInterfaces {
    pub engine_to_model: engine_to_model::EngineToModelInterface,
    pub model_to_engine: model_to_engine::ModelToEngineInterface,
    pub model_to_presentation: model_to_presentation::ModelToPresentationInterface,
    pub presentation_to_model: presentation_to_model::PresentationToModelInterface,
    pub debug_actions: debug_actions::DebugActionsInterface,
}

impl ThreeLayerInterfaces {
    /// Create a complete set of interfaces for the three-layer architecture
    /// 
    /// This factory function creates all interfaces needed for proper
    /// communication between the engine, model, and presentation layers.
    /// 
    /// # Returns
    /// 
    /// Returns a ThreeLayerInterfaces struct containing all interface instances.
    pub fn create() -> Self {
        Self {
            engine_to_model: engine_to_model::EngineToModelInterface::new(),
            model_to_engine: model_to_engine::ModelToEngineInterface::new(),
            model_to_presentation: model_to_presentation::ModelToPresentationInterface::new(),
            presentation_to_model: presentation_to_model::PresentationToModelInterface::new(),
            debug_actions: debug_actions::DebugActionsInterface::new(),
        }
    }
}