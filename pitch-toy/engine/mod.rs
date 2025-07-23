//! Engine Layer - Audio processing and hardware interface
//!
//! This layer handles low-level audio operations and browser API interactions.
//! It communicates with the Model layer by returning structured data from update calls.
//!
//! ## Data Flow in Engine Layer
//!
//! The engine layer:
//! - Processes audio data from microphone and browser APIs
//! - Returns structured data via EngineUpdateResult from update() calls
//! - Provides audio analysis, error information, and permission state
//!
//! ```rust
//! use pitch_toy::engine::AudioEngine;
//!
//! // Create engine without dependencies
//! let mut engine = AudioEngine::create().await?;
//!
//! // Engine returns data directly from update calls
//! let result = engine.update(timestamp);
//! // result contains: audio_analysis, audio_errors, permission_state
//! ```

pub mod audio;
pub mod platform;

use crate::module_interfaces::engine_to_model::EngineUpdateResult;

/// AudioEngine - The engine layer of the three-layer architecture
/// 
/// This struct represents the audio processing and hardware interface layer
/// of the application. It handles low-level audio operations, browser API
/// interactions, and microphone/speaker communication.
/// 
/// # Example
/// 
/// ```no_run
/// use pitch_toy::engine::AudioEngine;
/// 
/// let engine = AudioEngine::create()
///     .await.expect("AudioEngine creation should succeed");
/// ```
pub struct AudioEngine {
    /// Audio system context for managing audio processing
    audio_context: Option<std::rc::Rc<std::cell::RefCell<audio::AudioSystemContext>>>,
}

impl AudioEngine {
    /// Create a new AudioEngine without observable data dependencies
    /// 
    /// This constructor initializes the audio processing system using direct
    /// data return patterns instead of the observable data pattern.
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(AudioEngine)` on successful initialization, or `Err(String)`
    /// if audio system initialization fails.
    pub async fn create() -> Result<Self, String> {
        crate::common::dev_log!("Creating AudioEngine with return-based pattern");
        
        // Create audio context using the new return-based constructor
        let mut audio_context = audio::AudioSystemContext::new_return_based();
        
        // Initialize the audio system
        match audio_context.initialize().await {
            Ok(()) => {
                crate::common::dev_log!("✓ AudioEngine created and initialized successfully");
                Ok(Self {
                    audio_context: Some(std::rc::Rc::new(std::cell::RefCell::new(audio_context))),
                })
            }
            Err(e) => {
                crate::common::dev_log!("⚠ AudioEngine created but audio system initialization failed: {}", e);
                // Still create the engine but without audio context for now
                // This allows the application to continue running
                Ok(Self {
                    audio_context: None,
                })
            }
        }
    }

    /// Update the engine layer with a new timestamp
    /// 
    /// This method is called by the main render loop to update the engine's state.
    /// It processes audio data, handles device changes, and returns updates
    /// for the model layer.
    /// 
    /// # Arguments
    /// 
    /// * `timestamp` - The current timestamp in seconds since application start
    /// 
    /// # Returns
    /// 
    /// Returns `EngineUpdateResult` containing audio analysis data, errors, and permission state
    pub fn update(&mut self, timestamp: f64) -> EngineUpdateResult {
        // Collect audio analysis data
        let audio_analysis = self.collect_audio_analysis(timestamp);
        
        // Collect audio errors
        let audio_errors = self.collect_audio_errors();
        
        // Collect permission state
        let permission_state = self.collect_permission_state();
        
        EngineUpdateResult {
            audio_analysis,
            audio_errors,
            permission_state,
        }
    }
    
    /// Collect current audio analysis data from the audio system
    fn collect_audio_analysis(&self, timestamp: f64) -> Option<crate::module_interfaces::engine_to_model::AudioAnalysis> {
        if let Some(ref context) = self.audio_context {
            let borrowed_context = context.borrow();
            borrowed_context.collect_audio_analysis(timestamp)
        } else {
            // No audio context available
            None
        }
    }
    
    /// Collect current audio errors from the audio system
    fn collect_audio_errors(&self) -> Vec<crate::module_interfaces::engine_to_model::AudioError> {
        if let Some(ref context) = self.audio_context {
            let borrowed_context = context.borrow();
            borrowed_context.collect_audio_errors()
        } else {
            // No audio context available - return appropriate error
            vec![crate::module_interfaces::engine_to_model::AudioError::ProcessingError("Audio system not initialized".to_string())]
        }
    }
    
    /// Collect current permission state from the audio system
    fn collect_permission_state(&self) -> crate::module_interfaces::engine_to_model::PermissionState {
        if let Some(ref context) = self.audio_context {
            let borrowed_context = context.borrow();
            borrowed_context.collect_permission_state()
        } else {
            // No audio context available
            crate::module_interfaces::engine_to_model::PermissionState::NotRequested
        }
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