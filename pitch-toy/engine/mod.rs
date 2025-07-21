// Engine Layer - Audio processing and hardware interface
// Handles low-level audio operations and browser API interactions

pub mod audio;
pub mod platform;

use crate::module_interfaces::{
    engine_to_model::EngineToModelInterface,
    model_to_engine::ModelToEngineInterface,
};

/// AudioEngine - The engine layer of the three-layer architecture
/// 
/// This struct represents the audio processing and hardware interface layer
/// of the application. It handles low-level audio operations, browser API
/// interactions, and microphone/speaker communication.
/// 
/// # Example
/// 
/// ```no_run
/// use pitch_toy::module_interfaces::*;
/// use pitch_toy::engine::AudioEngine;
/// 
/// let engine_to_model = engine_to_model::EngineToModelInterface::new();
/// let model_to_engine = model_to_engine::ModelToEngineInterface::new();
/// 
/// let engine = AudioEngine::create(
///     engine_to_model,
///     model_to_engine,
/// ).await.expect("AudioEngine creation should succeed");
/// ```
pub struct AudioEngine {
    /// Interface for sending data to the model
    /// Contains setters for audio analysis, errors, and permission state
    _engine_to_model: std::rc::Rc<EngineToModelInterface>,
    
    /// Interface for receiving actions from the model
    /// Contains listeners for microphone permission requests
    _model_to_engine: std::rc::Rc<ModelToEngineInterface>,
    
    /// Audio system context for managing audio processing
    audio_context: Option<std::rc::Rc<std::cell::RefCell<audio::AudioSystemContext>>>,
}

impl AudioEngine {
    /// Create a new AudioEngine with the required interfaces
    /// 
    /// This constructor accepts the interfaces that connect the engine layer
    /// to the model layer and initializes the audio processing system.
    /// 
    /// # Arguments
    /// 
    /// * `engine_to_model` - Interface for sending audio data to the model
    /// * `model_to_engine` - Interface for receiving actions from the model
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(AudioEngine)` on successful initialization, or `Err(String)`
    /// if audio system initialization fails.
    pub async fn create(
        engine_to_model: std::rc::Rc<EngineToModelInterface>,
        model_to_engine: std::rc::Rc<ModelToEngineInterface>,
    ) -> Result<Self, String> {
        // Initialize audio system with interface-based communication
        let audio_context = match audio::initialize_audio_system_with_interfaces(
            &engine_to_model,
            &model_to_engine,
        ).await {
            Ok(context) => {
                crate::common::dev_log!("✓ Audio system initialized successfully in AudioEngine");
                Some(std::rc::Rc::new(std::cell::RefCell::new(context)))
            }
            Err(e) => {
                crate::common::dev_log!("✗ Audio system initialization failed: {}", e);
                crate::common::dev_log!("AudioEngine will continue without audio functionality");
                None
            }
        };
        
        Ok(Self {
            _engine_to_model: engine_to_model,
            _model_to_engine: model_to_engine,
            audio_context,
        })
    }

    /// Update the engine layer with a new timestamp
    /// 
    /// This method is called by the main render loop to update the engine's state.
    /// It should process audio data, handle device changes, and push updates
    /// to the model layer.
    /// 
    /// # Arguments
    /// 
    /// * `timestamp` - The current timestamp in seconds since application start
    pub fn update(&mut self, _timestamp: f64) {
        // Update audio context if available
        // Note: AudioSystemContext doesn't have an update method - it's handled by the worklet
        // This method is kept for future engine-level updates
    }
    
    /// Set up UI action listeners with the audio system
    /// 
    /// This method configures the engine to listen for UI control actions
    /// like microphone permission requests, test signals, etc.
    pub fn setup_ui_listeners(
        &self,
        ui_listeners: crate::UIControlListeners,
        microphone_permission_setter: impl observable_data::DataSetter<audio::AudioPermission> + Clone + 'static,
    ) {
        if let Some(ref context) = self.audio_context {
            audio::setup_ui_action_listeners_with_context(
                ui_listeners,
                microphone_permission_setter,
                context.clone(),
            );
        }
    }
    
    /// Set up debug action listeners with the audio system
    /// 
    /// This method configures the engine to listen for debug actions
    /// from the debug GUI.
    pub fn setup_debug_listeners(
        &self,
        debug_actions: &crate::module_interfaces::debug_actions::DebugActionsInterface,
    ) {
        if let Some(ref context) = self.audio_context {
            audio::context::AudioSystemContext::setup_debug_action_listeners(
                context,
                debug_actions,
            );
        }
    }
}