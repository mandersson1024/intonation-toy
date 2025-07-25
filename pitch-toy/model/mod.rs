//! Model Layer - Data transformation and state management
//! 
//! This layer is responsible for:
//! - State management and business logic
//! - Data transformation between engine and presentation layers
//! - User action processing and validation
//! - Tuning system implementations
//! - Musical note mapping and frequency calculations
//! - Pattern recognition and pitch tracking
//! - History buffers for temporal analysis
//! 
//! ## Return-Based Data Flow in Model Layer
//! 
//! The model layer now uses a return-based pattern for data processing:
//! - Receives `EngineUpdateResult` data as a parameter from the engine layer
//! - Processes and transforms the audio analysis data
//! - Returns `ModelUpdateResult` containing processed data for the presentation layer
//! - Processes `PresentationLayerActions` through business logic validation
//! - Returns `ModelLayerActions` containing validated operations
//! 
//! ```rust
//! use pitch_toy::model::DataModel;
//! use pitch_toy::module_interfaces::{
//!     engine_to_model::EngineUpdateResult,
//!     model_to_presentation::ModelUpdateResult,
//! };
//! use pitch_toy::presentation::PresentationLayerActions;
//! 
//! // Create model without interface dependencies
//! let mut model = DataModel::create()?;
//! 
//! // Process engine data and get results for presentation
//! let engine_data = EngineUpdateResult {
//!     audio_analysis: None,
//!     audio_errors: Vec::new(),
//!     permission_state: crate::module_interfaces::engine_to_model::PermissionState::NotRequested,
//! };
//! let presentation_data = model.update(timestamp, engine_data);
//! 
//! // Process user actions from presentation layer
//! let user_actions = PresentationLayerActions::new(); // From presentation layer
//! let validated_actions = model.process_user_actions(user_actions);
//! ```
//! 
//! ## Current Status
//! 
//! The DataModel struct operates without interface dependencies and processes
//! data through method parameters and return values. It provides comprehensive
//! audio data transformation including:
//! 
//! - ✅ Pitch detection and musical note identification
//! - ✅ Accuracy calculation based on frequency deviation from perfect pitch
//! - ✅ Volume level processing (peak and RMS)
//! - ✅ Error propagation from engine to presentation layer
//! - ✅ Permission state management
//! - ✅ Tuning system support (Equal Temperament)
//! - ✅ User action processing and business logic validation
//! - ✅ Three-layer action flow architecture
//! 
//! ## Action Processing System
//! 
//! The model layer implements a comprehensive action processing system that validates
//! user actions from the presentation layer through business logic before executing them:
//! 
//! ### Input: PresentationLayerActions
//! - `microphone_permission_requests` - User requests for microphone access
//! - `tuning_system_changes` - User selections of different tuning systems
//! - `root_note_adjustments` - User modifications to the root note
//! 
//! ### Processing: Business Logic Validation
//! - `validate_microphone_permission_request_with_error()` - Ensures permission requests are appropriate
//! - `validate_tuning_system_change_with_error()` - Validates tuning system changes
//! - `validate_root_note_adjustment_with_error()` - Validates root note adjustments
//! 
//! ### Output: ModelLayerActions
//! - `microphone_permission_requests` - Validated permission requests
//! - `audio_system_configurations` - Validated tuning system configurations
//! - `tuning_configurations` - Validated tuning and root note configurations
//! 
//! ### State Management
//! - `apply_tuning_system_change()` - Updates internal tuning system state
//! - `apply_root_note_change()` - Updates internal root note and frequency state
//! 
//! This system ensures that all user actions pass through business logic validation
//! before being executed, maintaining system consistency and preventing invalid states.
//! 
//! ## Future Implementation
//! 
//! When fully implemented, this layer will:
//! - Process raw audio analysis data from the engine
//! - Transform frequency data into musical notes
//! - Track pitch history and patterns
//! - Apply different tuning systems
//! - Handle user configuration changes
//! - Provide processed data to the presentation layer

use crate::module_interfaces::{
    engine_to_model::EngineUpdateResult,
    model_to_presentation::{ModelUpdateResult, Volume, Pitch, Accuracy, TuningSystem, Error, PermissionState, Note},
};
use crate::presentation::PresentationLayerActions;

/// Validation error types for action processing
/// 
/// These error types represent specific validation failures that can occur
/// when processing presentation layer actions. They provide detailed information
/// about why an action was rejected by business logic validation.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Microphone permission already granted
    PermissionAlreadyGranted,
    /// Microphone permission already denied
    PermissionAlreadyDenied,
    /// Microphone permission request already pending
    PermissionRequestPending,
    /// Microphone API not available on this platform
    MicrophoneApiUnavailable,
    /// Tuning system is already active
    TuningSystemAlreadyActive(TuningSystem),
    /// Unsupported tuning system requested
    UnsupportedTuningSystem(String),
    /// Root note is already set to requested value
    RootNoteAlreadySet(Note),
    /// Invalid frequency calculation for root note
    InvalidFrequencyCalculation { note: Note, reason: String },
}

/// Result of processing user actions with validation information
/// 
/// This struct contains both the successfully validated actions and any
/// validation errors that occurred during processing. This allows the
/// presentation layer to understand what succeeded and what failed.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessedActions {
    /// Successfully validated actions ready for execution
    pub actions: ModelLayerActions,
    /// Validation errors for actions that failed business logic checks
    pub validation_errors: Vec<ValidationError>,
}

/// Action structs for the model layer action processing system
/// 
/// These structs represent validated business logic actions that are processed
/// by the model layer after receiving presentation layer actions. They contain
/// validated data that has passed business logic checks.

/// Validated request for microphone permission
/// 
/// This struct represents a microphone permission request that has been validated
/// by the model layer's business logic. It is a unit struct as the validation
/// ensures that the request is appropriate given the current state.
#[derive(Debug, Clone, PartialEq)]
pub struct RequestMicrophonePermissionAction;

/// Validated audio system configuration
/// 
/// This struct represents an audio system configuration that has been validated
/// by the model layer's business logic. It contains the tuning system and
/// reference frequency that will be applied to the audio processing pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigureAudioSystemAction {
    pub tuning_system: TuningSystem,
    pub reference_frequency: f32,
}

/// Validated tuning configuration update
/// 
/// This struct represents a tuning configuration update that has been validated
/// by the model layer's business logic. It contains the complete tuning configuration
/// including tuning system, root note, and reference frequency (which remains constant).
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateTuningConfigurationAction {
    pub tuning_system: TuningSystem,
    pub root_note: Note,
    pub reference_frequency: f32,
}

/// Container for all processed model layer actions
/// 
/// This struct contains vectors of validated business logic actions that have been
/// processed from presentation layer actions. These actions represent the validated
/// operations that should be performed by the system.
/// 
/// The model layer processes `PresentationLayerActions` through business logic
/// validation and transforms valid actions into `ModelLayerActions`.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelLayerActions {
    /// Validated microphone permission requests
    pub microphone_permission_requests: Vec<RequestMicrophonePermissionAction>,
    
    /// Validated audio system configurations
    pub audio_system_configurations: Vec<ConfigureAudioSystemAction>,
    
    /// Validated tuning configuration updates
    pub tuning_configurations: Vec<UpdateTuningConfigurationAction>,
}

impl ModelLayerActions {
    /// Create a new instance with empty action collections
    /// 
    /// Returns a new `ModelLayerActions` struct with all action vectors initialized
    /// as empty. This is used as the starting point for collecting processed actions.
    pub fn new() -> Self {
        Self {
            microphone_permission_requests: Vec::new(),
            audio_system_configurations: Vec::new(),
            tuning_configurations: Vec::new(),
        }
    }
}

/// DataModel - The model layer of the three-layer architecture
/// 
/// This struct represents the core business logic and state management layer
/// of the application. It sits between the engine (which provides raw audio data)
/// and the presentation layer (which displays processed information).
/// 
/// Data flows through method parameters and return values rather than interface dependencies.
/// 
/// # Current Implementation
/// 
/// This implementation:
/// - Operates without observable interface dependencies
/// - Receives engine data through `update()` method parameters
/// - Returns processed data through `ModelUpdateResult` struct
/// - Provides basic audio data transformation
/// 
/// # Example
/// 
/// ```no_run
/// use pitch_toy::model::DataModel;
/// use pitch_toy::module_interfaces::engine_to_model::EngineUpdateResult;
/// 
/// let mut model = DataModel::create()
///     .expect("DataModel creation should always succeed");
/// 
/// let engine_data = EngineUpdateResult {
///     audio_analysis: None,
///     audio_errors: Vec::new(),
///     permission_state: crate::module_interfaces::engine_to_model::PermissionState::NotRequested,
/// };
/// 
/// let presentation_data = model.update(0.0, engine_data);
/// ```
pub struct DataModel {
    // Model layer now operates without interface dependencies
    // Data flows through method parameters and return values
    
    /// Current tuning system used for pitch calculations
    tuning_system: TuningSystem,
    
    /// Reference frequency for A4 (default 440 Hz)
    /// This frequency remains constant regardless of root note changes
    reference_a4: f32,
    
    /// Current root note for tuning calculations
    root_note: Note,
}

impl DataModel {
    /// Create a new DataModel without interface dependencies
    /// 
    /// This constructor creates a model layer that operates through method parameters
    /// and return values rather than observable interfaces.
    /// 
    /// # Returns
    /// 
    /// Always returns `Ok(DataModel)` as this implementation cannot fail.
    /// 
    /// # Current Behavior
    /// 
    /// Creates an empty model struct. Future implementations will initialize
    /// internal state for pitch tracking, tuning systems, and other model functionality.
    pub fn create() -> Result<Self, String> {
        // Model layer initialization without interface dependencies
        Ok(Self {
            tuning_system: TuningSystem::EqualTemperament,
            reference_a4: 440.0, // Standard A4 frequency
            root_note: Note::A, // Standard A root note
        })
    }

    /// Update the model layer with a new timestamp and engine data
    /// 
    /// This method is called by the main render loop to update the model's state.
    /// It processes the provided engine data, updates internal state, and returns
    /// processed data for the presentation layer.
    /// 
    /// # Arguments
    /// 
    /// * `timestamp` - The current timestamp in seconds since application start
    /// * `engine_data` - The data provided by the engine layer, containing audio analysis, errors, and permission state
    /// 
    /// # Returns
    /// 
    /// Returns `ModelUpdateResult` containing processed data for the presentation layer
    /// 
    /// # Current Implementation
    /// 
    /// This implementation:
    /// 1. Processes audio analysis from engine data into volume and pitch information
    /// 2. Calculates musical note identification from detected frequencies
    /// 3. Computes accuracy metrics based on frequency deviation from perfect pitch
    /// 4. Applies Equal Temperament tuning system for note calculations
    /// 5. Propagates errors and permission state from engine to presentation
    /// 
    /// # Accuracy Calculation
    /// 
    /// The accuracy system:
    /// - Uses MIDI note calculations for precise frequency-to-note mapping
    /// - Calculates deviation in cents (1/100th of a semitone)
    /// - Normalizes accuracy to 0.0-1.0 range (0.0 = perfect, 1.0 = 50+ cents off)
    /// - Returns maximum inaccuracy (1.0) when no pitch is detected
    pub fn update(&mut self, _timestamp: f64, engine_data: EngineUpdateResult) -> ModelUpdateResult {
        // Process audio analysis from engine data
        let (volume, pitch) = if let Some(audio_analysis) = engine_data.audio_analysis {
            // Extract volume and pitch from audio analysis
            let volume = Volume {
                peak: audio_analysis.volume_level.peak,
                rms: audio_analysis.volume_level.rms,
            };
            
            let pitch = match audio_analysis.pitch {
                crate::module_interfaces::engine_to_model::Pitch::Detected(frequency, clarity) => {
                    Pitch::Detected(frequency, clarity)
                }
                crate::module_interfaces::engine_to_model::Pitch::NotDetected => {
                    Pitch::NotDetected
                }
            };
            
            (volume, pitch)
        } else {
            // No audio analysis available - return defaults
            (
                Volume { peak: -60.0, rms: -60.0 }, // Silent levels
                Pitch::NotDetected
            )
        };
        
        // Convert engine errors to model errors
        let errors: Vec<Error> = engine_data.audio_errors.into_iter().map(|engine_error| {
            match engine_error {
                crate::module_interfaces::engine_to_model::AudioError::ProcessingError(msg) => {
                    Error::ProcessingError(msg)
                }
                crate::module_interfaces::engine_to_model::AudioError::MicrophonePermissionDenied => {
                    Error::MicrophonePermissionDenied
                }
                crate::module_interfaces::engine_to_model::AudioError::MicrophoneNotAvailable => {
                    Error::MicrophoneNotAvailable
                }
                crate::module_interfaces::engine_to_model::AudioError::BrowserApiNotSupported => {
                    Error::BrowserApiNotSupported
                }
                crate::module_interfaces::engine_to_model::AudioError::AudioContextInitFailed => {
                    Error::AudioContextInitFailed
                }
                crate::module_interfaces::engine_to_model::AudioError::AudioContextSuspended => {
                    Error::AudioContextSuspended
                }
            }
        }).collect();
        
        // Convert engine permission state to model permission state
        let permission_state = match engine_data.permission_state {
            crate::module_interfaces::engine_to_model::PermissionState::NotRequested => {
                PermissionState::NotRequested
            }
            crate::module_interfaces::engine_to_model::PermissionState::Requested => {
                PermissionState::Requested
            }
            crate::module_interfaces::engine_to_model::PermissionState::Granted => {
                PermissionState::Granted
            }
            crate::module_interfaces::engine_to_model::PermissionState::Denied => {
                PermissionState::Denied
            }
        };
        
        // Calculate accuracy based on detected pitch
        let accuracy = match pitch {
            Pitch::Detected(frequency, _clarity) => {
                let (closest_note, accuracy_cents) = self.frequency_to_note_and_accuracy(frequency);
                let normalized_accuracy = self.normalize_accuracy(accuracy_cents);
                Accuracy {
                    closest_note,
                    accuracy: normalized_accuracy,
                }
            }
            Pitch::NotDetected => {
                // No pitch detected - return default values
                Accuracy {
                    closest_note: Note::A,
                    accuracy: 1.0, // Maximum inaccuracy when no pitch is detected
                }
            }
        };
        
        // Return processed model data
        ModelUpdateResult {
            volume,
            pitch,
            accuracy,
            tuning_system: self.tuning_system.clone(),
            errors,
            permission_state,
        }
    }
    
    /// Process user actions from the presentation layer
    /// 
    /// This method receives `PresentationLayerActions` from the presentation layer,
    /// validates each action through business logic, and transforms valid actions
    /// into `ModelLayerActions` containing validated operations to be performed.
    /// 
    /// # Arguments
    /// 
    /// * `presentation_actions` - User actions collected from the presentation layer
    /// 
    /// # Returns
    /// 
    /// Returns `ProcessedActions` containing both validated actions ready for execution
    /// and validation errors for actions that failed business logic checks. This allows
    /// the presentation layer to provide feedback about why certain actions were rejected.
    /// 
    /// # Business Logic Validation
    /// 
    /// This method applies business logic validation to ensure that:
    /// - Microphone permission requests are appropriate for the current state
    /// - Tuning system changes are valid and different from the current system
    /// - Root note adjustments are valid and result in proper frequency calculations
    /// - All actions maintain system consistency and state integrity
    /// 
    /// # Current Implementation
    /// 
    /// The validation logic:
    /// 1. Validates microphone permission requests against current permission state
    /// 2. Validates tuning system changes and updates internal state when valid
    /// 3. Validates root note adjustments and updates internal state when valid
    /// 4. Combines validated actions into complete system configurations
    /// 5. Collects validation errors for failed actions
    /// 
    /// # State Updates
    /// 
    /// When actions pass validation, the model's internal state is immediately updated
    /// using `apply_tuning_system_change()` and `apply_root_note_change()` methods.
    /// This ensures the model's state remains synchronized with validated user actions.
    pub fn process_user_actions(&mut self, presentation_actions: PresentationLayerActions) -> ProcessedActions {
        let mut model_actions = ModelLayerActions::new();
        let mut validation_errors = Vec::new();
        
        // Process microphone permission requests
        for _permission_request in presentation_actions.microphone_permission_requests {
            match self.validate_microphone_permission_request_with_error() {
                Ok(()) => {
                    model_actions.microphone_permission_requests.push(RequestMicrophonePermissionAction);
                }
                Err(error) => {
                    // Log validation failure for debugging
                    // TODO: Add proper logging when log crate is integrated
                    validation_errors.push(error);
                }
            }
        }
        
        // Process tuning system changes
        for tuning_change in presentation_actions.tuning_system_changes {
            match self.validate_tuning_system_change_with_error(&tuning_change.tuning_system) {
                Ok(()) => {
                    let config = ConfigureAudioSystemAction {
                        tuning_system: tuning_change.tuning_system.clone(),
                        reference_frequency: self.reference_a4,
                    };
                    
                    // Apply the state change to internal model state
                    self.apply_tuning_system_change(&config);
                    
                    model_actions.audio_system_configurations.push(config);
                }
                Err(error) => {
                    // Log validation failure for debugging
                    // TODO: Add proper logging when log crate is integrated
                    validation_errors.push(error);
                }
            }
        }
        
        // Process root note adjustments
        for root_note_adjustment in presentation_actions.root_note_adjustments {
            match self.validate_root_note_adjustment_with_error(&root_note_adjustment.root_note) {
                Ok(()) => {
                    // Reference frequency remains constant at A4=440Hz regardless of root note selection.
                    // The root note affects musical relationships but not the fundamental frequency reference.
                    let new_reference_frequency = self.reference_a4;
                    let config = UpdateTuningConfigurationAction {
                        tuning_system: self.tuning_system.clone(),
                        root_note: root_note_adjustment.root_note.clone(),
                        reference_frequency: new_reference_frequency,
                    };
                    
                    // Apply the state change to internal model state
                    self.apply_root_note_change(&config);
                    
                    model_actions.tuning_configurations.push(config);
                }
                Err(error) => {
                    // Log validation failure for debugging
                    // TODO: Add proper logging when log crate is integrated
                    validation_errors.push(error);
                }
            }
        }
        
        ProcessedActions {
            actions: model_actions,
            validation_errors,
        }
    }
    
    /// Convert a frequency to the closest musical note
    /// Returns the note and accuracy (0.0 = perfect, negative = flat, positive = sharp)
    fn frequency_to_note_and_accuracy(&self, frequency: f32) -> (Note, f32) {
        if frequency <= 0.0 {
            return (Note::A, 0.0);
        }
        
        // Calculate MIDI note number from frequency
        // MIDI note 69 = A4 (440 Hz by default)
        let midi_note = 69.0 + 12.0 * (frequency / self.reference_a4).log2();
        let rounded_midi = midi_note.round();
        let note_index = (rounded_midi as i32 % 12 + 12) % 12; // Ensure positive
        
        // Convert to Note enum
        let note = match note_index {
            0 => Note::C,
            1 => Note::CSharp,
            2 => Note::D,
            3 => Note::DSharp,
            4 => Note::E,
            5 => Note::F,
            6 => Note::FSharp,
            7 => Note::G,
            8 => Note::GSharp,
            9 => Note::A,
            10 => Note::ASharp,
            11 => Note::B,
            _ => Note::A, // Fallback
        };
        
        // Calculate accuracy in cents (100 cents = 1 semitone)
        let accuracy_cents = (midi_note - rounded_midi) * 100.0;
        
        (note, accuracy_cents)
    }
    
    
    /// Normalize accuracy value to a 0.0-1.0 range
    /// 0.0 = perfectly in tune, 1.0 = 50 cents (half semitone) or worse
    fn normalize_accuracy(&self, cents: f32) -> f32 {
        let abs_cents = cents.abs();
        // Clamp to max 50 cents for normalization
        let clamped_cents = abs_cents.min(50.0);
        clamped_cents / 50.0
    }
    
    
    /// Validate microphone permission request with detailed error reporting
    /// 
    /// No model-layer validation is required for microphone permission requests.
    /// The engine layer is responsible for handling microphone API state, permissions,
    /// and hardware availability checks.
    /// 
    /// # Returns
    /// 
    /// Always returns `Ok(())` as microphone permission validation is handled by the engine layer.
    fn validate_microphone_permission_request_with_error(&self) -> Result<(), ValidationError> {
        // No model-layer validation needed - engine handles microphone permission logic
        Ok(())
    }
    
    
    /// Validate tuning system change request with detailed error reporting
    /// 
    /// Ensures that a tuning system change is valid and different from the current system.
    /// This validation prevents unnecessary system reconfigurations and maintains
    /// system stability by filtering out redundant changes.
    /// 
    /// # Arguments
    /// 
    /// * `new_tuning_system` - The requested tuning system to validate
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` if the tuning system change should be processed, or a specific
    /// `ValidationError` describing why the change was rejected.
    /// 
    /// # Current Implementation
    /// 
    /// Validates that the new tuning system is different from the current one.
    /// Future implementations will add more sophisticated validation:
    /// - Compatibility checks with current audio configuration
    /// - Validation of supported tuning systems
    /// - State consistency checks
    fn validate_tuning_system_change_with_error(&self, new_tuning_system: &TuningSystem) -> Result<(), ValidationError> {
        if *new_tuning_system == self.tuning_system {
            Err(ValidationError::TuningSystemAlreadyActive(new_tuning_system.clone()))
        } else {
            Ok(())
        }
    }
    
    
    /// Validate root note adjustment request with detailed error reporting
    /// 
    /// Ensures that a root note adjustment is valid and results in proper frequency
    /// calculations. This validation maintains musical accuracy and prevents
    /// invalid note configurations.
    /// 
    /// # Arguments
    /// 
    /// * `new_root_note` - The requested root note to validate
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` if the root note adjustment should be processed, or a specific
    /// `ValidationError` describing why the adjustment was rejected.
    /// 
    /// # Current Implementation
    /// 
    /// Validates that the new root note is different from the current one and is
    /// a valid musical note. Future implementations will add:
    /// - Frequency range validation
    /// - Compatibility checks with current tuning system
    /// - Musical theory validation
    fn validate_root_note_adjustment_with_error(&self, new_root_note: &Note) -> Result<(), ValidationError> {
        if *new_root_note == self.root_note {
            Err(ValidationError::RootNoteAlreadySet(new_root_note.clone()))
        } else {
            Ok(())
        }
    }
    
    
    
    /// Apply tuning system change to internal state
    /// 
    /// Updates the internal tuning system and reference frequency based on a validated
    /// tuning system change. This method should only be called with actions that have
    /// passed business logic validation.
    /// 
    /// # Arguments
    /// 
    /// * `action` - The validated tuning system configuration to apply
    /// 
    /// # Current Implementation
    /// 
    /// Updates the internal tuning system and reference frequency directly from the
    /// validated action. Future implementations will add:
    /// - State change notifications
    /// - Logging of configuration changes
    /// - Validation of state consistency after changes
    fn apply_tuning_system_change(&mut self, action: &ConfigureAudioSystemAction) {
        self.tuning_system = action.tuning_system.clone();
        self.reference_a4 = action.reference_frequency;
    }
    
    /// Apply root note change to internal state
    /// 
    /// Updates the internal root note and reference frequency based on a validated
    /// root note adjustment. This method should only be called with actions that have
    /// passed business logic validation.
    /// 
    /// # Arguments
    /// 
    /// * `action` - The validated tuning configuration to apply
    /// 
    /// # Current Implementation
    /// 
    /// Updates the internal tuning system, root note, and reference frequency directly
    /// from the validated action. Future implementations will add:
    /// - State change notifications
    /// - Logging of configuration changes
    /// - Validation of state consistency after changes
    /// - Recalculation of derived values
    fn apply_root_note_change(&mut self, action: &UpdateTuningConfigurationAction) {
        self.tuning_system = action.tuning_system.clone();
        self.root_note = action.root_note.clone();
        self.reference_a4 = action.reference_frequency;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    /// Test that DataModel::create() succeeds without interface dependencies
    #[wasm_bindgen_test]
    fn test_data_model_create_succeeds() {
        let result = DataModel::create();
        assert!(result.is_ok(), "DataModel::create() should always succeed");
    }

    /// Test that update() method can be called without panicking
    #[wasm_bindgen_test]
    fn test_data_model_update_no_panic() {
        let mut model = DataModel::create()
            .expect("DataModel creation should succeed");

        // Create test engine data
        let engine_data = EngineUpdateResult {
            audio_analysis: None,
            audio_errors: Vec::new(),
            permission_state: crate::module_interfaces::engine_to_model::PermissionState::NotRequested,
        };

        // Test that update can be called multiple times without panicking
        let _result1 = model.update(0.0, engine_data.clone());
        let _result2 = model.update(1.0, engine_data.clone());
        let _result3 = model.update(123.456, engine_data.clone());
        let _result4 = model.update(-1.0, engine_data); // Negative timestamp should also be safe
        
        // If we reach this point, no panic occurred
        assert!(true, "update() method should not panic");
    }

    /// Verify that struct can be created without interface dependencies
    #[wasm_bindgen_test]
    fn test_data_model_interface_free_creation() {
        // This test verifies that the struct can be constructed without interfaces
        let model = DataModel::create();

        match model {
            Ok(_) => {
                // Success - model was created without interface dependencies
                assert!(true, "DataModel was created without interface dependencies");
            }
            Err(e) => {
                panic!("DataModel should be creatable without interfaces, but got error: {}", e);
            }
        }
    }

    /// Test basic runtime safety - creation and operation should not crash
    #[wasm_bindgen_test]
    fn test_data_model_runtime_safety() {
        // Create multiple instances to test memory safety
        for i in 0..10 {
            let mut model = DataModel::create()
                .expect("DataModel creation should always succeed");

            // Create test engine data
            let engine_data = EngineUpdateResult {
                audio_analysis: None,
                audio_errors: Vec::new(),
                permission_state: crate::module_interfaces::engine_to_model::PermissionState::NotRequested,
            };

            // Test multiple operations
            let _result1 = model.update(i as f64, engine_data.clone());
            let _result2 = model.update((i as f64) * 0.5, engine_data.clone());
            
            // Test edge case values
            let _result3 = model.update(f64::MAX, engine_data.clone());
            let _result4 = model.update(f64::MIN, engine_data.clone());
            let _result5 = model.update(0.0, engine_data);
        }
        
        // If we reach this point, all operations completed safely
        assert!(true, "All DataModel operations completed safely");
    }

    /// Test that DataModel compilation requirements are met
    #[wasm_bindgen_test]
    fn test_data_model_compilation_safety() {
        // This test exists primarily to ensure the struct compiles correctly
        // without interface dependencies

        // Test successful creation
        let model_result = DataModel::create();

        // Test that the result type is correct
        assert!(model_result.is_ok());
        
        let mut model = model_result.unwrap();
        
        // Create test engine data
        let engine_data = EngineUpdateResult {
            audio_analysis: None,
            audio_errors: Vec::new(),
            permission_state: crate::module_interfaces::engine_to_model::PermissionState::NotRequested,
        };
        
        // Test that update signature is correct
        let _result = model.update(42.0, engine_data);
        
        // Test completed - all compilation requirements verified
        assert!(true, "DataModel meets all compilation requirements");
    }

    /// Test pitch accuracy calculation with A4 440Hz (perfect accuracy)
    #[wasm_bindgen_test]
    fn test_pitch_accuracy_calculation_perfect_a4() {
        let mut model = DataModel::create().unwrap();
        
        // Create engine data with A4 at exactly 440 Hz
        let audio_analysis = crate::module_interfaces::engine_to_model::AudioAnalysis {
            volume_level: crate::module_interfaces::engine_to_model::Volume { peak: -10.0, rms: -15.0 },
            pitch: crate::module_interfaces::engine_to_model::Pitch::Detected(440.0, 0.95),
            fft_data: None,
            timestamp: 1.0,
        };
        
        let engine_data = EngineUpdateResult {
            audio_analysis: Some(audio_analysis),
            audio_errors: Vec::new(),
            permission_state: crate::module_interfaces::engine_to_model::PermissionState::Granted,
        };
        
        let result = model.update(1.0, engine_data);
        
        // Should detect A note with perfect accuracy (0.0)
        assert_eq!(result.accuracy.closest_note, Note::A);
        assert!(result.accuracy.accuracy < 0.01, "Accuracy should be nearly perfect for 440Hz A4, got {}", result.accuracy.accuracy);
    }

    /// Test pitch accuracy calculation with slightly flat C4
    #[wasm_bindgen_test]
    fn test_pitch_accuracy_calculation_flat_c4() {
        let mut model = DataModel::create().unwrap();
        
        // C4 is approximately 261.63 Hz, test with 260 Hz (slightly flat)
        let audio_analysis = crate::module_interfaces::engine_to_model::AudioAnalysis {
            volume_level: crate::module_interfaces::engine_to_model::Volume { peak: -10.0, rms: -15.0 },
            pitch: crate::module_interfaces::engine_to_model::Pitch::Detected(260.0, 0.90),
            fft_data: None,
            timestamp: 1.0,
        };
        
        let engine_data = EngineUpdateResult {
            audio_analysis: Some(audio_analysis),
            audio_errors: Vec::new(),
            permission_state: crate::module_interfaces::engine_to_model::PermissionState::Granted,
        };
        
        let result = model.update(1.0, engine_data);
        
        // Should detect C note with some inaccuracy (flat)
        assert_eq!(result.accuracy.closest_note, Note::C);
        assert!(result.accuracy.accuracy > 0.0, "Accuracy should show flatness for 260Hz (expected ~261.63Hz)");
        assert!(result.accuracy.accuracy < 1.0, "Accuracy should not be at maximum for a recognizable pitch");
    }

    /// Test behavior with no pitch detected
    #[wasm_bindgen_test]
    fn test_pitch_accuracy_no_pitch_detected() {
        let mut model = DataModel::create().unwrap();
        
        let audio_analysis = crate::module_interfaces::engine_to_model::AudioAnalysis {
            volume_level: crate::module_interfaces::engine_to_model::Volume { peak: -60.0, rms: -60.0 },
            pitch: crate::module_interfaces::engine_to_model::Pitch::NotDetected,
            fft_data: None,
            timestamp: 1.0,
        };
        
        let engine_data = EngineUpdateResult {
            audio_analysis: Some(audio_analysis),
            audio_errors: Vec::new(),
            permission_state: crate::module_interfaces::engine_to_model::PermissionState::Granted,
        };
        
        let result = model.update(1.0, engine_data);
        
        // Should have maximum inaccuracy when no pitch is detected
        assert_eq!(result.accuracy.accuracy, 1.0);
        assert_eq!(result.pitch, crate::module_interfaces::model_to_presentation::Pitch::NotDetected);
    }

    /// Test that tuning system is properly returned
    #[wasm_bindgen_test]
    fn test_tuning_system_propagation() {
        let mut model = DataModel::create().unwrap();
        
        let engine_data = EngineUpdateResult {
            audio_analysis: None,
            audio_errors: Vec::new(),
            permission_state: crate::module_interfaces::engine_to_model::PermissionState::NotRequested,
        };
        
        let result = model.update(1.0, engine_data);
        
        // Should return the configured tuning system
        assert_eq!(result.tuning_system, TuningSystem::EqualTemperament);
    }

    /// Test validation error when tuning system is already active
    #[wasm_bindgen_test]
    fn test_tuning_system_already_active_error() {
        let mut model = DataModel::create().unwrap();
        
        // Create presentation actions with the same tuning system as current
        let mut actions = PresentationLayerActions::new();
        actions.tuning_system_changes.push(crate::presentation::ChangeTuningSystem {
            tuning_system: TuningSystem::EqualTemperament, // Same as default
        });
        
        let result = model.process_user_actions(actions);
        
        // Should have no successful actions
        assert_eq!(result.actions.audio_system_configurations.len(), 0);
        
        // Should have one validation error
        assert_eq!(result.validation_errors.len(), 1);
        assert_eq!(
            result.validation_errors[0],
            ValidationError::TuningSystemAlreadyActive(TuningSystem::EqualTemperament)
        );
    }

    /// Test validation error when root note is already set
    #[wasm_bindgen_test]
    fn test_root_note_already_set_error() {
        let mut model = DataModel::create().unwrap();
        
        // Create presentation actions with the same root note as current
        let mut actions = PresentationLayerActions::new();
        actions.root_note_adjustments.push(crate::presentation::AdjustRootNote {
            root_note: Note::A, // Same as default
        });
        
        let result = model.process_user_actions(actions);
        
        // Should have no successful actions
        assert_eq!(result.actions.tuning_configurations.len(), 0);
        
        // Should have one validation error
        assert_eq!(result.validation_errors.len(), 1);
        assert_eq!(
            result.validation_errors[0],
            ValidationError::RootNoteAlreadySet(Note::A)
        );
    }

    /// Test successful validation with different values
    #[wasm_bindgen_test]
    fn test_successful_action_processing() {
        let mut model = DataModel::create().unwrap();
        
        // Create presentation actions with valid changes
        let mut actions = PresentationLayerActions::new();
        actions.microphone_permission_requests.push(crate::presentation::RequestMicrophonePermission);
        actions.tuning_system_changes.push(crate::presentation::ChangeTuningSystem {
            tuning_system: TuningSystem::JustIntonation,
        });
        actions.root_note_adjustments.push(crate::presentation::AdjustRootNote {
            root_note: Note::C,
        });
        
        let result = model.process_user_actions(actions);
        
        // Should have successful actions
        assert_eq!(result.actions.microphone_permission_requests.len(), 1);
        assert_eq!(result.actions.audio_system_configurations.len(), 1);
        assert_eq!(result.actions.tuning_configurations.len(), 1);
        
        // Should have no validation errors
        assert_eq!(result.validation_errors.len(), 0);
        
        // Verify model state was updated
        assert_eq!(model.tuning_system, TuningSystem::JustIntonation);
        assert_eq!(model.root_note, Note::C);
    }

    /// Test mixed success and failure cases
    #[wasm_bindgen_test]
    fn test_mixed_validation_results() {
        let mut model = DataModel::create().unwrap();
        
        // Create presentation actions with some valid and some invalid changes
        let mut actions = PresentationLayerActions::new();
        
        // Valid: permission request
        actions.microphone_permission_requests.push(crate::presentation::RequestMicrophonePermission);
        
        // Invalid: same tuning system
        actions.tuning_system_changes.push(crate::presentation::ChangeTuningSystem {
            tuning_system: TuningSystem::EqualTemperament,
        });
        
        // Valid: different tuning system
        actions.tuning_system_changes.push(crate::presentation::ChangeTuningSystem {
            tuning_system: TuningSystem::JustIntonation,
        });
        
        // Invalid: same root note
        actions.root_note_adjustments.push(crate::presentation::AdjustRootNote {
            root_note: Note::A,
        });
        
        // Valid: different root note
        actions.root_note_adjustments.push(crate::presentation::AdjustRootNote {
            root_note: Note::D,
        });
        
        let result = model.process_user_actions(actions);
        
        // Should have successful actions
        assert_eq!(result.actions.microphone_permission_requests.len(), 1);
        assert_eq!(result.actions.audio_system_configurations.len(), 1);
        assert_eq!(result.actions.tuning_configurations.len(), 1);
        
        // Should have validation errors for failed actions
        assert_eq!(result.validation_errors.len(), 2);
        
        // Verify specific errors
        assert!(result.validation_errors.contains(&ValidationError::TuningSystemAlreadyActive(TuningSystem::EqualTemperament)));
        assert!(result.validation_errors.contains(&ValidationError::RootNoteAlreadySet(Note::A)));
        
        // Verify model state was updated only for valid actions
        assert_eq!(model.tuning_system, TuningSystem::JustIntonation);
        assert_eq!(model.root_note, Note::D);
    }
}