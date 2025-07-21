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