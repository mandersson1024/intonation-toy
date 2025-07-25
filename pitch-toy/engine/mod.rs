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
//! 
//! // Access the aggregated data
//! if let Some(analysis) = result.audio_analysis {
//!     println!("Volume: {} dB", analysis.volume_level.peak);
//!     match analysis.pitch {
//!         Pitch::Detected(freq, clarity) => {
//!             println!("Pitch: {} Hz (clarity: {})", freq, clarity);
//!         }
//!         Pitch::NotDetected => println!("No pitch detected"),
//!     }
//! }
//! 
//! // Check for errors
//! for error in &result.audio_errors {
//!     eprintln!("Audio error: {:?}", error);
//! }
//! 
//! // Check permission state
//! match result.permission_state {
//!     PermissionState::Granted => println!("Microphone access granted"),
//!     PermissionState::Denied => println!("Microphone access denied"),
//!     _ => {}
//! }
//! ```

pub mod audio;
pub mod platform;

use crate::module_interfaces::engine_to_model::EngineUpdateResult;
use crate::module_interfaces::model_to_presentation::{TuningSystem, Note};
use crate::model::{ModelLayerActions, RequestMicrophonePermissionAction, ConfigureAudioSystemAction, UpdateTuningConfigurationAction};

/// Execution action for microphone permission requests
/// 
/// This unit struct represents the execution of a microphone permission request 
/// at the engine layer. It contains no additional data as the execution process
/// is handled entirely by the existing microphone connection functionality.
#[derive(Debug, Clone, PartialEq)]
pub struct ExecuteMicrophonePermissionRequest;

/// Execution action for audio system configuration
/// 
/// This struct represents the execution of audio system configuration at the engine layer.
/// It configures the audio worklet with a specific tuning system and sets the root frequency
/// for the tonic note (the first note of the scale).
/// 
/// The `root_frequency` represents the frequency assigned to the tonic (first degree) of the
/// current tuning system, not an absolute tuning reference.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigureAudioSystem {
    /// The tuning system to configure in the audio processing pipeline
    pub tuning_system: TuningSystem,
    /// The frequency (in Hz) assigned to the tonic note of the tuning system
    pub root_frequency: f32,
}

/// Execution action for tuning configuration updates
/// 
/// This struct represents the execution of tuning configuration updates at the engine layer.
/// It updates the audio system's tuning configuration with a specific root note and assigns
/// the appropriate root frequency for that note.
/// 
/// The `root_frequency` is the frequency assigned to the specified `root_note` when it
/// functions as the tonic (first degree) of the scale.
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateTuningConfiguration {
    /// The tuning system being used
    pub tuning_system: TuningSystem,
    /// The root note that will serve as the tonic
    pub root_note: Note,
    /// The frequency (in Hz) assigned to the root note as the tonic
    pub root_frequency: f32,
}

/// Container for all executed engine layer actions
/// 
/// This struct contains vectors of low-level execution actions that have been
/// processed by the engine layer. These actions represent the actual operations
/// performed on the audio system hardware and processing pipeline.
/// 
/// The engine layer receives `ModelLayerActions` from the model layer, transforms
/// them into executable operations, performs the execution, and returns the results
/// as `EngineLayerActions` for logging and feedback purposes.
#[derive(Debug, Clone, PartialEq)]
pub struct EngineLayerActions {
    /// Executed microphone permission requests
    pub microphone_permission_requests: Vec<ExecuteMicrophonePermissionRequest>,
    
    /// Executed audio system configurations
    pub audio_system_configurations: Vec<ConfigureAudioSystem>,
    
    /// Executed tuning configuration updates  
    pub tuning_configurations: Vec<UpdateTuningConfiguration>,
}

impl EngineLayerActions {
    /// Create a new instance with empty action collections
    /// 
    /// Returns a new `EngineLayerActions` struct with all action vectors initialized
    /// as empty. This is used as the starting point for collecting executed actions.
    pub fn new() -> Self {
        Self {
            microphone_permission_requests: Vec::new(),
            audio_system_configurations: Vec::new(),
            tuning_configurations: Vec::new(),
        }
    }
}

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
    
    /// Get audio context reference for microphone permission handling
    pub fn get_audio_context(&self) -> Option<&std::rc::Rc<std::cell::RefCell<audio::AudioSystemContext>>> {
        self.audio_context.as_ref()
    }
    
    /// Set up UI action listeners with the audio system
    /// 
    /// This method configures the engine to listen for UI control actions
    /// like microphone permission requests, test signals, etc.
    pub fn setup_ui_listeners(
        &self,
        ui_listeners: crate::UIControlListeners,
    ) {
        if let Some(ref context) = self.audio_context {
            audio::setup_ui_action_listeners_with_context(
                ui_listeners,
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
    
    /// Execute model layer actions and return executed actions for logging/feedback
    /// 
    /// This method receives validated actions from the model layer, transforms them
    /// into engine-specific execution actions, performs the actual execution using
    /// existing audio system functionality, and returns the executed actions for
    /// logging and feedback purposes.
    /// 
    /// # Arguments
    /// 
    /// * `model_actions` - Validated actions from the model layer to execute
    /// 
    /// # Returns
    /// 
    /// Returns `Result<EngineLayerActions, String>` containing either the successfully
    /// executed actions or an error message if execution failed.
    /// 
    /// # Execution Process
    /// 
    /// 1. Transforms model actions into engine execution actions (mapping reference_frequency to root_frequency)
    /// 2. Executes each action type using existing engine functionality:
    ///    - Microphone permission requests use `connect_microphone_to_audioworklet_with_context()`
    ///    - Audio system configurations configure the audio worklet with tuning system and root frequency
    ///    - Tuning configurations update the audio system with root note and root frequency
    /// 3. Collects executed actions for logging and feedback
    /// 4. Provides comprehensive error handling with detailed logging
    /// 
    /// # Root Frequency vs Reference Frequency
    /// 
    /// The engine layer uses "root_frequency" terminology to reflect that we're setting
    /// the frequency for a specific tonic note rather than changing absolute tuning standards.
    /// The model layer's "reference_frequency" is mapped to "root_frequency" during transformation.
    pub async fn execute_actions(&mut self, model_actions: ModelLayerActions) -> Result<EngineLayerActions, String> {
        crate::common::dev_log!("Engine layer executing {} model actions", 
            model_actions.microphone_permission_requests.len() + 
            model_actions.audio_system_configurations.len() + 
            model_actions.tuning_configurations.len());
        
        let mut engine_actions = EngineLayerActions::new();
        
        // Execute microphone permission requests
        let mic_results = self.execute_microphone_permission_requests(
            &model_actions.microphone_permission_requests
        ).await;
        
        match mic_results {
            Ok(executed_requests) => {
                engine_actions.microphone_permission_requests = executed_requests;
                crate::common::dev_log!("✓ Executed {} microphone permission requests", 
                    engine_actions.microphone_permission_requests.len());
            }
            Err(e) => {
                crate::common::dev_log!("✗ Failed to execute microphone permission requests: {}", e);
                return Err(format!("Microphone permission execution failed: {}", e));
            }
        }
        
        // Execute audio system configurations
        let audio_results = self.execute_audio_system_configurations(
            &model_actions.audio_system_configurations
        ).await;
        
        match audio_results {
            Ok(executed_configs) => {
                engine_actions.audio_system_configurations = executed_configs;
                crate::common::dev_log!("✓ Executed {} audio system configurations", 
                    engine_actions.audio_system_configurations.len());
            }
            Err(e) => {
                crate::common::dev_log!("✗ Failed to execute audio system configurations: {}", e);
                return Err(format!("Audio system configuration failed: {}", e));
            }
        }
        
        // Execute tuning configurations
        let tuning_results = self.execute_tuning_configurations(
            &model_actions.tuning_configurations
        ).await;
        
        match tuning_results {
            Ok(executed_tunings) => {
                engine_actions.tuning_configurations = executed_tunings;
                crate::common::dev_log!("✓ Executed {} tuning configurations", 
                    engine_actions.tuning_configurations.len());
            }
            Err(e) => {
                crate::common::dev_log!("✗ Failed to execute tuning configurations: {}", e);
                return Err(format!("Tuning configuration failed: {}", e));
            }
        }
        
        let total_executed = engine_actions.microphone_permission_requests.len() + 
                           engine_actions.audio_system_configurations.len() + 
                           engine_actions.tuning_configurations.len();
        
        crate::common::dev_log!("✓ Engine layer successfully executed {} total actions", total_executed);
        
        Ok(engine_actions)
    }
    
    /// Execute microphone permission requests using existing microphone functionality
    /// 
    /// This method processes validated microphone permission requests from the model layer
    /// and executes them using the existing `connect_microphone_to_audioworklet_with_context()`
    /// functionality from the microphone module.
    /// 
    /// # Arguments
    /// 
    /// * `permission_requests` - Vector of validated permission requests to execute
    /// 
    /// # Returns
    /// 
    /// Returns `Result<Vec<ExecuteMicrophonePermissionRequest>, String>` containing
    /// either the successfully executed permission requests or an error message.
    async fn execute_microphone_permission_requests(
        &self,
        permission_requests: &[RequestMicrophonePermissionAction]
    ) -> Result<Vec<ExecuteMicrophonePermissionRequest>, String> {
        let mut executed_requests = Vec::new();
        
        for _request in permission_requests {
            if let Some(ref audio_context) = self.audio_context {
                crate::common::dev_log!("Executing microphone permission request using connect_microphone_to_audioworklet_with_context()");
                
                match audio::microphone::connect_microphone_to_audioworklet_with_context(audio_context).await {
                    Ok(_) => {
                        executed_requests.push(ExecuteMicrophonePermissionRequest);
                        crate::common::dev_log!("✓ Microphone permission request executed successfully");
                    }
                    Err(e) => {
                        crate::common::dev_log!("✗ Microphone permission execution failed: {}", e);
                        return Err(format!("Microphone connection failed: {}", e));
                    }
                }
            } else {
                crate::common::dev_log!("✗ No audio context available for microphone permission execution");
                return Err("Audio system not initialized".to_string());
            }
        }
        
        Ok(executed_requests)
    }
    
    /// Execute audio system configurations with tuning system and root frequency
    /// 
    /// This method processes validated audio system configurations from the model layer
    /// and configures the audio worklet with the specified tuning system and root frequency
    /// for the tonic note.
    /// 
    /// # Arguments
    /// 
    /// * `system_configs` - Vector of validated system configurations to execute
    /// 
    /// # Returns
    /// 
    /// Returns `Result<Vec<ConfigureAudioSystem>, String>` containing either the
    /// successfully executed configurations or an error message.
    async fn execute_audio_system_configurations(
        &self,
        system_configs: &[ConfigureAudioSystemAction]
    ) -> Result<Vec<ConfigureAudioSystem>, String> {
        let mut executed_configs = Vec::new();
        
        for config in system_configs {
            // Transform model action to engine action, mapping reference_frequency to root_frequency
            // Note: For now we use a default root frequency of 440Hz (A4) as a placeholder
            // Future implementations will calculate the proper root frequency based on the tuning system
            let root_frequency = self.calculate_root_frequency_for_tuning_system(&config.tuning_system);
            
            let engine_config = ConfigureAudioSystem {
                tuning_system: config.tuning_system.clone(),
                root_frequency,
            };
            
            crate::common::dev_log!("Configuring audio system with tuning: {:?}, root frequency: {} Hz", 
                engine_config.tuning_system, engine_config.root_frequency);
            
            // Execute the configuration using audio system context
            if let Some(ref audio_context) = self.audio_context {
                match self.configure_audio_worklet_with_tuning(&engine_config, audio_context).await {
                    Ok(_) => {
                        executed_configs.push(engine_config);
                        crate::common::dev_log!("✓ Audio system configuration executed successfully");
                    }
                    Err(e) => {
                        crate::common::dev_log!("✗ Audio system configuration failed: {}", e);
                        return Err(format!("Audio worklet configuration failed: {}", e));
                    }
                }
            } else {
                crate::common::dev_log!("✗ No audio context available for audio system configuration");
                return Err("Audio system not initialized".to_string());
            }
        }
        
        Ok(executed_configs)
    }
    
    /// Execute tuning configurations with root note and root frequency
    /// 
    /// This method processes validated tuning configurations from the model layer
    /// and updates the audio system's tuning configuration with the specified root note
    /// and calculates the appropriate root frequency for that note.
    /// 
    /// # Arguments
    /// 
    /// * `tuning_configs` - Vector of validated tuning configurations to execute
    /// 
    /// # Returns
    /// 
    /// Returns `Result<Vec<UpdateTuningConfiguration>, String>` containing either the
    /// successfully executed configurations or an error message.
    async fn execute_tuning_configurations(
        &self,
        tuning_configs: &[UpdateTuningConfigurationAction]
    ) -> Result<Vec<UpdateTuningConfiguration>, String> {
        let mut executed_configs = Vec::new();
        
        for config in tuning_configs {
            // Calculate the root frequency for the specified root note
            let root_frequency = self.calculate_root_frequency_for_note(&config.root_note);
            
            let engine_config = UpdateTuningConfiguration {
                tuning_system: config.tuning_system.clone(),
                root_note: config.root_note.clone(),
                root_frequency,
            };
            
            crate::common::dev_log!("Updating tuning configuration - tuning: {:?}, root note: {:?}, root frequency: {} Hz", 
                engine_config.tuning_system, engine_config.root_note, engine_config.root_frequency);
            
            // Execute the tuning update using audio system context
            if let Some(ref audio_context) = self.audio_context {
                match self.update_audio_worklet_tuning(&engine_config, audio_context).await {
                    Ok(_) => {
                        executed_configs.push(engine_config);
                        crate::common::dev_log!("✓ Tuning configuration executed successfully");
                    }
                    Err(e) => {
                        crate::common::dev_log!("✗ Tuning configuration failed: {}", e);
                        return Err(format!("Audio worklet tuning update failed: {}", e));
                    }
                }
            } else {
                crate::common::dev_log!("✗ No audio context available for tuning configuration");
                return Err("Audio system not initialized".to_string());
            }
        }
        
        Ok(executed_configs)
    }
    
    /// Calculate root frequency for a tuning system (placeholder implementation)
    /// 
    /// This method calculates the appropriate root frequency for the tonic note
    /// of a given tuning system. This is a placeholder implementation that uses
    /// standard frequencies. Future implementations will provide proper calculation
    /// based on tuning system characteristics.
    /// 
    /// # Arguments
    /// 
    /// * `tuning_system` - The tuning system to calculate root frequency for
    /// 
    /// # Returns
    /// 
    /// Returns the root frequency in Hz for the tonic note of the tuning system
    fn calculate_root_frequency_for_tuning_system(&self, tuning_system: &TuningSystem) -> f32 {
        // Placeholder implementation - returns A4 (440Hz) for all tuning systems
        // TODO: Implement proper frequency calculation based on tuning system characteristics
        match tuning_system {
            TuningSystem::EqualTemperament => 440.0, // A4 in Equal Temperament
            TuningSystem::JustIntonation => 440.0,   // A4 in Just Intonation (placeholder)
        }
    }
    
    /// Calculate root frequency for a specific note (placeholder implementation)
    /// 
    /// This method calculates the frequency for a specific note when it serves
    /// as the root/tonic of the current tuning system. This is a placeholder
    /// implementation using Equal Temperament calculations.
    /// 
    /// # Arguments
    /// 
    /// * `note` - The note to calculate frequency for as the root
    /// 
    /// # Returns
    /// 
    /// Returns the frequency in Hz for the note when serving as the tonic
    fn calculate_root_frequency_for_note(&self, note: &Note) -> f32 {
        // Placeholder implementation using Equal Temperament calculations
        // Base frequency: A4 = 440Hz (MIDI note 69)
        // TODO: Implement proper calculation based on current tuning system
        
        let midi_note_offset = match note {
            Note::C => -9,      // C4 is 9 semitones below A4
            Note::CSharp => -8, // C#4 is 8 semitones below A4
            Note::D => -7,      // D4 is 7 semitones below A4
            Note::DSharp => -6, // D#4 is 6 semitones below A4
            Note::E => -5,      // E4 is 5 semitones below A4
            Note::F => -4,      // F4 is 4 semitones below A4
            Note::FSharp => -3, // F#4 is 3 semitones below A4
            Note::G => -2,      // G4 is 2 semitones below A4
            Note::GSharp => -1, // G#4 is 1 semitone below A4
            Note::A => 0,       // A4 is the reference
            Note::ASharp => 1,  // A#4 is 1 semitone above A4
            Note::B => 2,       // B4 is 2 semitones above A4
        };
        
        // Calculate frequency using Equal Temperament formula: f = 440 * 2^(n/12)
        440.0 * (2.0_f32).powf(midi_note_offset as f32 / 12.0)
    }
    
    /// Configure audio worklet with tuning system and root frequency (placeholder implementation)
    /// 
    /// This method configures the audio worklet with a specific tuning system and
    /// root frequency for the tonic note. This is a placeholder implementation that
    /// demonstrates the architecture.
    /// 
    /// # Arguments
    /// 
    /// * `config` - The audio system configuration to apply
    /// * `audio_context` - The audio system context to configure
    /// 
    /// # Returns
    /// 
    /// Returns `Result<(), String>` indicating success or failure of configuration
    async fn configure_audio_worklet_with_tuning(
        &self,
        config: &ConfigureAudioSystem,
        _audio_context: &std::rc::Rc<std::cell::RefCell<audio::AudioSystemContext>>
    ) -> Result<(), String> {
        // Placeholder implementation - logs the configuration that would be applied
        // TODO: Implement actual audio worklet configuration with tuning system and root frequency
        crate::common::dev_log!("PLACEHOLDER: Configuring audio worklet with tuning system {:?} and root frequency {} Hz",
            config.tuning_system, config.root_frequency);
        
        // For now, always succeed to demonstrate the action execution flow
        Ok(())
    }
    
    /// Update audio worklet tuning configuration (placeholder implementation)
    /// 
    /// This method updates the audio worklet's tuning configuration with a specific
    /// root note and root frequency. This is a placeholder implementation that
    /// demonstrates the architecture.
    /// 
    /// # Arguments
    /// 
    /// * `config` - The tuning configuration to apply
    /// * `audio_context` - The audio system context to update
    /// 
    /// # Returns
    /// 
    /// Returns `Result<(), String>` indicating success or failure of the update
    async fn update_audio_worklet_tuning(
        &self,
        config: &UpdateTuningConfiguration,
        _audio_context: &std::rc::Rc<std::cell::RefCell<audio::AudioSystemContext>>
    ) -> Result<(), String> {
        // Placeholder implementation - logs the configuration that would be applied
        // TODO: Implement actual audio worklet tuning update with root note and frequency
        crate::common::dev_log!("PLACEHOLDER: Updating audio worklet tuning - system: {:?}, root note: {:?}, root frequency: {} Hz",
            config.tuning_system, config.root_note, config.root_frequency);
        
        // For now, always succeed to demonstrate the action execution flow
        Ok(())
    }
}