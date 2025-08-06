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
//! use intonation_toy::model::DataModel;
//! use intonation_toy::shared_types::{
//!     EngineUpdateResult,
//!     ModelUpdateResult,
//! };
//! use intonation_toy::presentation::PresentationLayerActions;
//! 
//! // Create model without interface dependencies
//! let mut model = DataModel::create()?;
//! 
//! // Process engine data and get results for presentation
//! let engine_data = EngineUpdateResult {
//!     audio_analysis: None,
//!     audio_errors: Vec::new(),
//!     permission_state: crate::shared_types::PermissionState::NotRequested,
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
//! - `apply_root_note_change()` - Updates internal root note state
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

use crate::shared_types::{EngineUpdateResult, ModelUpdateResult, Volume, Pitch, IntonationData, TuningSystem, Scale, Error, PermissionState, MidiNote, is_valid_midi_note, semitone_in_scale};
use crate::presentation::PresentationLayerActions;
use crate::common::warn_log;

/// Validation error types for action processing
/// 
/// These error types represent specific validation failures that can occur
/// when processing presentation layer actions. They provide detailed information
/// about why an action was rejected by business logic validation.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ValidationError {
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
    RootNoteAlreadySet(MidiNote),
    /// Invalid frequency value
    InvalidFrequency(f32),
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
pub(crate) struct RequestMicrophonePermissionAction;

/// Validated audio system configuration
/// 
/// This struct represents an audio system configuration that has been validated
/// by the model layer's business logic. It contains the tuning system that will
/// be applied to the audio processing pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigureAudioSystemAction {
    pub tuning_system: TuningSystem,
}

/// Validated tuning configuration update
/// 
/// This struct represents a tuning configuration update that has been validated
/// by the model layer's business logic. It contains the complete tuning configuration
/// including tuning system and root note.
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateTuningConfigurationAction {
    pub tuning_system: TuningSystem,
    pub root_note: MidiNote,
}

/// Validated root note audio configuration
/// 
/// This struct represents a root note audio configuration that has been validated
/// by the model layer's business logic. It contains the audio generation settings
/// including enabled state and frequency.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigureRootNoteAudioAction {
    pub enabled: bool,
    pub frequency: f32,
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
    /// Validated audio system configurations
    pub audio_system_configurations: Vec<ConfigureAudioSystemAction>,
    
    /// Validated tuning configuration updates
    pub tuning_configurations: Vec<UpdateTuningConfigurationAction>,
    
    /// Validated root note audio configurations
    pub root_note_audio_configurations: Vec<ConfigureRootNoteAudioAction>,
}

impl ModelLayerActions {
    /// Create a new instance with empty action collections
    /// 
    /// Returns a new `ModelLayerActions` struct with all action vectors initialized
    /// as empty. This is used as the starting point for collecting processed actions.
    pub fn new() -> Self {
        Self {
            audio_system_configurations: Vec::new(),
            tuning_configurations: Vec::new(),
            root_note_audio_configurations: Vec::new(),
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
/// use pitch_toy::shared_types::EngineUpdateResult;
/// 
/// let mut model = DataModel::create()
///     .expect("DataModel creation should always succeed");
/// 
/// let engine_data = EngineUpdateResult {
///     audio_analysis: None,
///     audio_errors: Vec::new(),
///     permission_state: crate::shared_types::PermissionState::NotRequested,
/// };
/// 
/// let presentation_data = model.update(0.0, engine_data);
/// ```
pub struct DataModel {
    // Model layer now operates without interface dependencies
    // Data flows through method parameters and return values
    
    /// Current tuning system used for pitch calculations
    tuning_system: TuningSystem,
    
    /// Current root note for tuning calculations
    root_note: MidiNote,
    
    /// Current scale for note filtering
    current_scale: Scale,
}

/// Standard A4 = 440Hz reference frequency for Equal Temperament
/// 
/// This constant represents the internationally accepted standard reference frequency
/// for Equal Temperament tuning. A4 = 440Hz serves as the baseline from which all
/// other note frequencies are calculated in Equal Temperament.
/// 
/// **Important**: This constant is different from the root pitch frequency returned
/// by `get_root_pitch()`. While this constant is always 440.0Hz regardless of
/// configuration, the root pitch varies based on the selected root note.
/// 
/// # Standard Reference
/// 
/// - Established by ISO 16:1975 and reaffirmed by various music organizations
/// - Used as the reference point for calculating all other note frequencies
/// - Independent of root note or tuning system configuration changes
/// 
/// # Usage
/// 
/// Use this constant when you need the absolute A4 = 440Hz reference frequency
/// rather than the current root pitch frequency. For example:
/// - Displaying standard tuning information to users
/// - Calculating frequency ratios relative to the standard
/// - Providing a consistent reference point across different configurations
pub const REFERENCE_FREQUENCY: f32 = 440.0;

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
            root_note: 57, // Standard A3 root note (MIDI 57)
            current_scale: Scale::Chromatic,
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
                peak_amplitude: audio_analysis.volume_level.peak_amplitude,
                rms_amplitude: audio_analysis.volume_level.rms_amplitude,
            };
            
            let pitch = match audio_analysis.pitch {
                crate::shared_types::Pitch::Detected(frequency, clarity) => {
                    Pitch::Detected(frequency, clarity)
                }
                crate::shared_types::Pitch::NotDetected => {
                    Pitch::NotDetected
                }
            };
            
            (volume, pitch)
        } else {
            // No audio analysis available - return defaults
            (
                Volume { peak_amplitude: -60.0, rms_amplitude: -60.0 }, // Silent levels
                Pitch::NotDetected
            )
        };
        
        // Convert engine errors to model errors
        let errors: Vec<Error> = engine_data.audio_errors.into_iter().map(|engine_error| {
            match engine_error {
                crate::shared_types::Error::ProcessingError(msg) => {
                    Error::ProcessingError(msg)
                }
                crate::shared_types::Error::MicrophonePermissionDenied => {
                    Error::MicrophonePermissionDenied
                }
                crate::shared_types::Error::MicrophoneNotAvailable => {
                    Error::MicrophoneNotAvailable
                }
                crate::shared_types::Error::BrowserApiNotSupported => {
                    Error::BrowserApiNotSupported
                }
            }
        }).collect();
        
        // Convert engine permission state to model permission state
        let permission_state = match engine_data.permission_state {
            crate::shared_types::PermissionState::NotRequested => {
                PermissionState::NotRequested
            }
            crate::shared_types::PermissionState::Requested => {
                PermissionState::Requested
            }
            crate::shared_types::PermissionState::Granted => {
                PermissionState::Granted
            }
            crate::shared_types::PermissionState::Denied => {
                PermissionState::Denied
            }
        };
        
        // Calculate accuracy based on detected pitch with full tuning context
        let accuracy = match pitch {
            Pitch::Detected(frequency, clarity) => {
                // Processing detected pitch with tuning system and root note
                
                // Apply tuning-aware frequency to note conversion
                let (closest_midi_note, accuracy_cents) = self.frequency_to_note_and_accuracy(frequency);
                
                // Result: Note and cents offset calculated
                
                IntonationData {
                    closest_midi_note,
                    cents_offset: accuracy_cents,
                }
            }
            Pitch::NotDetected => {
                // No pitch detected - return default values
                IntonationData {
                    closest_midi_note: self.root_note, // Use MidiNote directly
                    cents_offset: 0.0, // No offset when no pitch is detected
                }
            }
        };
        
        // Calculate interval semitones between detected note and root note
        let interval_semitones = match pitch {
            Pitch::Detected(_, _) => {
                (accuracy.closest_midi_note as i32) - (self.root_note as i32)
            }
            Pitch::NotDetected => 0, // No interval when no pitch detected
        };

        // Interval calculation: detected MIDI - root MIDI = interval semitones

        // Return processed model data with both legacy and flattened fields
        let result = ModelUpdateResult {
            volume,
            pitch,
            accuracy: accuracy.clone(), // Keep for backward compatibility
            tuning_system: self.tuning_system.clone(),
            scale: self.current_scale.clone(),
            errors,
            permission_state,
            // New flattened fields for easier access
            closest_midi_note: accuracy.closest_midi_note,
            cents_offset: accuracy.cents_offset,
            interval_semitones,
            root_note: self.root_note,
            root_note_audio_enabled: engine_data.root_note_audio_enabled,
        };
        
        result
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
        
        // Process tuning system changes
        for tuning_change in presentation_actions.tuning_system_changes {
            match self.validate_tuning_system_change_with_error(&tuning_change.tuning_system) {
                Ok(()) => {
                    let config = ConfigureAudioSystemAction {
                        tuning_system: tuning_change.tuning_system.clone(),
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
            // root_note is already a MidiNote
            let midi_note = root_note_adjustment.root_note;
            match self.validate_root_note_adjustment_with_error(&midi_note) {
                    Ok(()) => {
                        let config = UpdateTuningConfigurationAction {
                            tuning_system: self.tuning_system.clone(),
                            root_note: midi_note,
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
        
        // Process scale changes
        for scale_change in presentation_actions.scale_changes {
            // Only apply if scale is different from current
            if scale_change.scale != self.current_scale {
                self.apply_scale_change(&scale_change);
                // No model-layer action created since scale changes are internal
            }
        }
        
        // Process root note audio configurations
        crate::common::dev_log!("MODEL: Processing {} root note audio configurations", presentation_actions.root_note_audio_configurations.len());
        for root_note_audio_config in presentation_actions.root_note_audio_configurations {
            crate::common::dev_log!("MODEL: Processing root note audio config - enabled: {}", root_note_audio_config.enabled);
            
            match self.validate_root_note_audio_configuration_with_error(&root_note_audio_config) {
                Ok(()) => {
                    let config = ConfigureRootNoteAudioAction {
                        enabled: root_note_audio_config.enabled,
                        frequency: root_note_audio_config.frequency,
                    };
                    
                    // Apply the state change to internal model state
                    self.apply_root_note_audio_change(&config);
                    
                    // Add validated action for engine execution
                    model_actions.root_note_audio_configurations.push(config);
                    
                    crate::common::dev_log!("MODEL: ✓ Root note audio configuration validated and queued for engine execution");
                }
                Err(error) => {
                    // Log validation error but continue processing other actions
                    let error_message = format!("Root note audio configuration validation failed: {:?}", error);
                    crate::common::warn_log!("{}", error_message);
                    validation_errors.push(error);
                }
            }
        }
        
        ProcessedActions {
            actions: model_actions,
            validation_errors,
        }
    }

    #[cfg(test)]
    pub fn process_user_actions_test(&mut self, presentation_actions: PresentationLayerActions) -> ProcessedActions {
        self.process_user_actions(presentation_actions)
    }
    
    /// Convert a frequency to the closest musical note with tuning system and scale awareness
    /// 
    /// This method applies the current tuning system, root note context, and scale filtering
    /// to convert raw frequency data into musical note identification. The conversion process:
    /// 
    /// 1. Validates the input frequency (must be positive)
    /// 2. Calculates a root pitch frequency based on tuning system and root note
    /// 3. Converts frequency to MIDI note space using tuning-specific formulas
    /// 4. Maps MIDI note to the closest musical note
    /// 5. Applies scale filtering to find the nearest scale member if needed
    /// 6. Calculates accuracy in cents (1/100th of a semitone)
    /// 
    /// # Scale Filtering
    /// 
    /// When the detected note is not in the current scale, the method finds the
    /// nearest scale member by searching outward (±1, ±2, ±3 semitones) until
    /// a scale member is found. For equal distances, it favors the upward direction.
    /// 
    /// # Tuning System Application
    /// 
    /// The tuning system affects how frequencies map to notes:
    /// - Equal Temperament: Each semitone is exactly 2^(1/12) ratio apart
    /// - Future systems (Just Intonation, etc.) will use different ratios
    /// 
    /// # Root Note Context
    /// 
    /// The root note determines the reference point for all calculations:
    /// - Changes the root pitch frequency used for MIDI conversion
    /// - Affects which frequencies are considered "in tune"
    /// - Allows the same frequency to map to different accuracy values
    /// 
    /// # Returns
    /// 
    /// Returns a tuple of (Note, accuracy_cents) where:
    /// - Note: The closest musical note to the frequency (filtered by scale)
    /// - accuracy_cents: Deviation in cents (negative = flat, positive = sharp)
    fn frequency_to_note_and_accuracy(&self, frequency: f32) -> (MidiNote, f32) {
        // Handle edge case: invalid or zero frequency
        if frequency <= 0.0 {
            warn_log!("[MODEL] Invalid frequency for note conversion: {}", frequency);
            return (69, 0.0); // Return A4 (MIDI 69) as default
        }
        
        // Handle edge case: extremely low frequency (below human hearing ~20Hz)
        if frequency < 20.0 {
            warn_log!("[MODEL] Extremely low frequency detected: {}Hz", frequency);
            // Still process but warn in debug mode
        }
        
        // Handle edge case: extremely high frequency (above typical musical range ~4186Hz for C8)
        if frequency > 4186.0 {
            warn_log!("[MODEL] Extremely high frequency detected: {}Hz", frequency);
            // Still process but warn in debug mode
        }
        
        // Get root pitch frequency based on current tuning system and root note
        // This is the key to tuning-aware processing
        let root_pitch = self.get_root_pitch();
        
        // Converting frequency with tuning system and root pitch
        
        // Use the scale-aware calculation from the tuning module
        let interval_result = crate::theory::tuning::frequency_to_interval_semitones_scale_aware(
            self.tuning_system.clone(),
            root_pitch,
            frequency,
            self.current_scale,
        );
        
        // Calculate MIDI note from root note plus interval
        let raw_midi_note = self.root_note as i32 + interval_result.semitones;
        
        // Clamp to valid MIDI range (0-127)
        let clamped_midi_note = raw_midi_note.max(0).min(127) as u8;
        
        // Validate using the utility function
        let final_midi_note = if is_valid_midi_note(clamped_midi_note) {
            clamped_midi_note
        } else {
            69 // Default to A4 if validation fails
        };
        
        (final_midi_note, interval_result.cents)
    }
    
    
    
    /// Get the root pitch frequency for the current root note
    /// 
    /// Root pitch is always calculated using Equal Temperament regardless of the 
    /// active tuning system. This is distinct from the `REFERENCE_FREQUENCY` 
    /// constant which always represents A4 = 440Hz.
    /// 
    /// # Root Pitch Calculation
    /// - Uses A4 = 440Hz as the standard reference
    /// - Calculates other notes using Equal Temperament formula
    /// 
    /// This ensures consistent frequency mapping regardless of which tuning system
    /// is used for interval calculations in other parts of the application.
    /// 
    /// # Root Note Impact
    /// 
    /// The root note changes which frequency is considered the "tonic":
    /// - If root is A, then A4 = 440Hz is the root pitch
    /// - If root is C, then C4 = 261.63Hz becomes the root pitch
    /// - All other frequencies are calculated relative to this root pitch
    fn get_root_pitch(&self) -> f32 {
        // Use the centralized function for consistency
        crate::theory::tuning::midi_note_to_standard_frequency(self.root_note)
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
    /// - Compatibility checks with current tuning system
    /// - Musical theory validation
    fn validate_root_note_adjustment_with_error(&self, new_root_note: &MidiNote) -> Result<(), ValidationError> {
        if *new_root_note == self.root_note {
            Err(ValidationError::RootNoteAlreadySet(*new_root_note))
        } else {
            Ok(())
        }
    }
    
    /// Validate root note audio configuration request
    /// 
    /// Validates a root note audio configuration request by checking that the frequency
    /// is valid and the configuration is different from the current state.
    /// Future implementations will add:
    /// - Frequency range validation
    /// - Audio system compatibility checks
    /// - Volume level validation
    fn validate_root_note_audio_configuration_with_error(&self, config: &crate::presentation::ConfigureRootNoteAudio) -> Result<(), ValidationError> {
        // Validate frequency is positive
        if config.frequency <= 0.0 {
            return Err(ValidationError::InvalidFrequency(config.frequency));
        }
        
        // Allow configuration even if state is the same to ensure proper synchronization
        Ok(())
    }
    
    /// Apply tuning system change to internal state
    /// 
    /// Updates the internal tuning system and root pitch frequency based on a validated
    /// tuning system change. This method should only be called with actions that have
    /// passed business logic validation.
    /// 
    /// # Arguments
    /// 
    /// * `action` - The validated tuning system configuration to apply
    /// 
    /// # Current Implementation
    /// 
    /// Updates the internal tuning system based on a validated tuning system change.
    /// Future implementations will add:
    /// - State change notifications
    /// - Logging of configuration changes
    /// - Validation of state consistency after changes
    fn apply_tuning_system_change(&mut self, action: &ConfigureAudioSystemAction) {
        crate::common::dev_log!(
            "Model layer: Tuning system changed from {:?} to {:?}",
            self.tuning_system, action.tuning_system
        );
        self.tuning_system = action.tuning_system.clone();
    }
    
    /// Apply scale change to internal state
    /// 
    /// Updates the internal scale based on a validated scale change.
    /// This method should only be called with actions that have passed business logic validation.
    /// 
    /// # Arguments
    /// 
    /// * `action` - The validated scale change to apply
    /// 
    /// # Current Implementation
    /// 
    /// Updates the internal scale based on a validated scale change.
    /// Future implementations will add:
    /// - State change notifications
    /// - Logging of configuration changes
    /// - Validation of state consistency after changes
    fn apply_scale_change(&mut self, action: &crate::presentation::ScaleChangeAction) {
        crate::common::dev_log!(
            "Model layer: Scale changed from {:?} to {:?}",
            self.current_scale, action.scale
        );
        self.current_scale = action.scale;
    }
    
    /// Apply root note change to internal state
    /// 
    /// Updates the internal root note based on a validated root note adjustment.
    /// This method should only be called with actions that have passed business logic validation.
    /// 
    /// # Arguments
    /// 
    /// * `action` - The validated tuning configuration to apply
    /// 
    /// # Current Implementation
    /// 
    /// Updates the internal tuning system and root note directly from the validated action.
    /// Future implementations will add:
    /// - State change notifications
    /// - Logging of configuration changes
    /// - Validation of state consistency after changes
    /// - Recalculation of derived values
    fn apply_root_note_change(&mut self, action: &UpdateTuningConfigurationAction) {
        crate::common::dev_log!(
            "Model layer: Root note changed from {} to {}",
            self.root_note, action.root_note
        );
        self.tuning_system = action.tuning_system.clone();
        self.root_note = action.root_note;
    }
    
    /// Apply root note audio configuration change to internal state
    /// 
    /// Updates the internal root note audio enabled state based on a validated
    /// root note audio configuration change. This method should only be called with
    /// actions that have passed business logic validation.
    /// Future implementations will add:
    /// - State change notifications
    /// - Logging of configuration changes
    /// - Validation of state consistency after changes
    fn apply_root_note_audio_change(&mut self, action: &ConfigureRootNoteAudioAction) {
        crate::common::dev_log!(
            "Model layer: Root note audio configuration passed through - enabled: {}",
            if action.enabled { "enabled" } else { "disabled" }
        );
        // Model no longer stores root note audio state - engine owns this state
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
            permission_state: crate::shared_types::PermissionState::NotRequested,
            root_note_audio_enabled: false,
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
                permission_state: crate::shared_types::PermissionState::NotRequested,
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
            permission_state: crate::shared_types::PermissionState::NotRequested,
            root_note_audio_enabled: false,
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
        let audio_analysis = crate::shared_types::AudioAnalysis {
            volume_level: crate::shared_types::Volume { peak_amplitude: -10.0, rms_amplitude: -15.0 },
            pitch: crate::shared_types::Pitch::Detected(440.0, 0.95),
            fft_data: None,
            timestamp: 1.0,
        };
        
        let engine_data = EngineUpdateResult {
            audio_analysis: Some(audio_analysis),
            audio_errors: Vec::new(),
            permission_state: crate::shared_types::PermissionState::Granted,
            root_note_audio_enabled: false,
        };
        
        let result = model.update(1.0, engine_data);
        
        // Should detect A note with perfect accuracy (0.0 cents)
        assert_eq!(result.accuracy.closest_midi_note, 69);
        assert!(result.accuracy.cents_offset.abs() < 1.0, "Cents offset should be nearly zero for 440Hz A4, got {}", result.accuracy.cents_offset);
    }

    /// Test pitch accuracy calculation with slightly flat C4
    #[wasm_bindgen_test]
    fn test_pitch_accuracy_calculation_flat_c4() {
        let mut model = DataModel::create().unwrap();
        
        // C4 is approximately 261.63 Hz, test with 260 Hz (slightly flat)
        let audio_analysis = crate::shared_types::AudioAnalysis {
            volume_level: crate::shared_types::Volume { peak_amplitude: -10.0, rms_amplitude: -15.0 },
            pitch: crate::shared_types::Pitch::Detected(260.0, 0.90),
            fft_data: None,
            timestamp: 1.0,
        };
        
        let engine_data = EngineUpdateResult {
            audio_analysis: Some(audio_analysis),
            audio_errors: Vec::new(),
            permission_state: crate::shared_types::PermissionState::Granted,
            root_note_audio_enabled: false,
        };
        
        let result = model.update(1.0, engine_data);
        
        // Should detect C note with some inaccuracy (flat)
        assert_eq!(result.accuracy.closest_midi_note, 60);
        assert!(result.accuracy.cents_offset < 0.0, "Cents offset should be negative (flat) for 260Hz (expected ~261.63Hz)");
        assert!(result.accuracy.cents_offset.abs() < 50.0, "Cents offset should be within reasonable range for a recognizable pitch");
    }

    /// Test behavior with no pitch detected
    #[wasm_bindgen_test]
    fn test_pitch_accuracy_no_pitch_detected() {
        let mut model = DataModel::create().unwrap();
        
        let audio_analysis = crate::shared_types::AudioAnalysis {
            volume_level: crate::shared_types::Volume { peak_amplitude: -60.0, rms_amplitude: -60.0 },
            pitch: crate::shared_types::Pitch::NotDetected,
            fft_data: None,
            timestamp: 1.0,
        };
        
        let engine_data = EngineUpdateResult {
            audio_analysis: Some(audio_analysis),
            audio_errors: Vec::new(),
            permission_state: crate::shared_types::PermissionState::Granted,
            root_note_audio_enabled: false,
        };
        
        let result = model.update(1.0, engine_data);
        
        // Should have zero cents offset when no pitch is detected
        assert_eq!(result.accuracy.cents_offset, 0.0);
        assert_eq!(result.pitch, crate::shared_types::Pitch::NotDetected);
    }

    /// Test that tuning system is properly returned
    #[wasm_bindgen_test]
    fn test_tuning_system_propagation() {
        let mut model = DataModel::create().unwrap();
        
        let engine_data = EngineUpdateResult {
            audio_analysis: None,
            audio_errors: Vec::new(),
            permission_state: crate::shared_types::PermissionState::NotRequested,
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
            root_note: 69, // A4 as MIDI note
        });
        
        let result = model.process_user_actions(actions);
        
        // Should have no successful actions
        assert_eq!(result.actions.tuning_configurations.len(), 0);
        
        // Should have one validation error
        assert_eq!(result.validation_errors.len(), 1);
        assert_eq!(
            result.validation_errors[0],
            ValidationError::RootNoteAlreadySet(69) // A4 = MIDI 69
        );
    }

    /// Test successful validation with different values
    #[wasm_bindgen_test]
    fn test_successful_action_processing() {
        let mut model = DataModel::create().unwrap();
        
        // Create presentation actions with valid changes
        let mut actions = PresentationLayerActions::new();
        // Since we only have EqualTemperament, we'll test by changing root note only
        actions.root_note_adjustments.push(crate::presentation::AdjustRootNote {
            root_note: 60,
        });
        
        let result = model.process_user_actions(actions);
        
        // Should have successful actions
        assert_eq!(result.actions.tuning_configurations.len(), 1);
        
        // Should have no validation errors
        assert_eq!(result.validation_errors.len(), 0);
        
        // Verify model state was updated
        assert_eq!(model.tuning_system, TuningSystem::EqualTemperament);
        assert_eq!(model.root_note, 60); // C4 = MIDI 60
    }

    /// Test mixed success and failure cases
    #[wasm_bindgen_test]
    fn test_mixed_validation_results() {
        let mut model = DataModel::create().unwrap();
        
        // Create presentation actions with some valid and some invalid changes
        let mut actions = PresentationLayerActions::new();
        
        // Invalid: same tuning system
        actions.tuning_system_changes.push(crate::presentation::ChangeTuningSystem {
            tuning_system: TuningSystem::EqualTemperament,
        });
        
        // Invalid: same root note
        actions.root_note_adjustments.push(crate::presentation::AdjustRootNote {
            root_note: 69, // A4
        });
        
        // Valid: different root note
        actions.root_note_adjustments.push(crate::presentation::AdjustRootNote {
            root_note: 62, // D4
        });
        
        let result = model.process_user_actions(actions);
        
        // Should have successful actions for the valid root note change
        assert_eq!(result.actions.audio_system_configurations.len(), 0);
        assert_eq!(result.actions.tuning_configurations.len(), 1);
        
        // Should have validation errors for failed actions
        assert_eq!(result.validation_errors.len(), 2);
        
        // Verify specific errors
        assert!(result.validation_errors.contains(&ValidationError::TuningSystemAlreadyActive(TuningSystem::EqualTemperament)));
        assert!(result.validation_errors.contains(&ValidationError::RootNoteAlreadySet(69))); // A4 = MIDI 69
        
        // Verify model state was updated only for valid actions
        assert_eq!(model.tuning_system, TuningSystem::EqualTemperament);
        assert_eq!(model.root_note, 62); // D4 = MIDI 62
    }

    /// Test that same raw frequency is processed differently with different root notes
    #[wasm_bindgen_test]
    fn test_raw_frequency_processing_with_different_tuning_contexts() {
        let mut model = DataModel::create().unwrap();
        
        // Test frequency: 440Hz (A4 in standard tuning)
        let test_frequency = 440.0;
        let audio_analysis = crate::shared_types::AudioAnalysis {
            volume_level: crate::shared_types::Volume { peak_amplitude: -10.0, rms_amplitude: -15.0 },
            pitch: crate::shared_types::Pitch::Detected(test_frequency, 0.95),
            fft_data: None,
            timestamp: 1.0,
        };
        
        let engine_data = EngineUpdateResult {
            audio_analysis: Some(audio_analysis.clone()),
            audio_errors: Vec::new(),
            permission_state: crate::shared_types::PermissionState::Granted,
        };
        
        // First test with root note A (default)
        let result_a = model.update(1.0, engine_data.clone());
        assert_eq!(result_a.accuracy.closest_midi_note, 69);
        assert!(result_a.accuracy.cents_offset.abs() < 1.0, "440Hz should be perfectly in tune with A root");
        
        // Change root note to C
        let mut actions = PresentationLayerActions::new();
        actions.root_note_adjustments.push(crate::presentation::AdjustRootNote {
            root_note: 60,
        });
        model.process_user_actions(actions);
        
        // Test same frequency with C root note
        let result_c = model.update(2.0, engine_data.clone());
        assert_eq!(result_c.accuracy.closest_midi_note, 69);
        // With C as root, 440Hz (A) should show some cents deviation since it's not a perfect interval
        assert!(result_c.accuracy.cents_offset.abs() > 1.0, "440Hz should show cents deviation with C root");
        
        // Change root note to F#
        let mut actions = PresentationLayerActions::new();
        actions.root_note_adjustments.push(crate::presentation::AdjustRootNote {
            root_note: 66,
        });
        model.process_user_actions(actions);
        
        // Test same frequency with F# root note
        let result_fsharp = model.update(3.0, engine_data);
        assert_eq!(result_fsharp.accuracy.closest_midi_note, 69);
        // The cents offset should be different again
        assert_ne!(result_a.accuracy.cents_offset, result_fsharp.accuracy.cents_offset, 
            "Same frequency should have different cents offset with different root notes");
    }

    /// Test that frequency_to_note_and_accuracy properly applies tuning context
    #[wasm_bindgen_test]
    fn test_frequency_to_note_conversion_with_tuning_context() {
        let mut model = DataModel::create().unwrap();
        
        // Test C4 frequency (261.63 Hz)
        let c4_freq = 261.63;
        let (midi_note, cents) = model.frequency_to_note_and_accuracy(c4_freq);
        assert_eq!(midi_note, 60);
        assert!(cents.abs() < 1.0, "C4 should be nearly perfect");
        
        // Test frequencies between notes
        let between_c_and_csharp = 269.0; // Between C4 (261.63) and C#4 (277.18)
        let (midi_note, cents) = model.frequency_to_note_and_accuracy(between_c_and_csharp);
        assert_eq!(midi_note, 60, "269Hz should be closer to C than C#");
        assert!(cents > 0.0, "Should be sharp relative to C");
        assert!(cents < 50.0, "Should be less than 50 cents sharp");
        
        // Test with different root note
        let mut actions = PresentationLayerActions::new();
        actions.root_note_adjustments.push(crate::presentation::AdjustRootNote {
            root_note: 62,
        });
        model.process_user_actions(actions);
        
        // Same frequency should still map to same note but with different reference
        let (midi_note_d_root, cents_d_root) = model.frequency_to_note_and_accuracy(c4_freq);
        assert_eq!(midi_note_d_root, 60, "Note identification should be absolute");
        // Cents calculation will be relative to D root
    }


    /// Test get_root_pitch for different root notes
    #[wasm_bindgen_test]
    fn test_root_pitch_calculation() {
        let mut model = DataModel::create().unwrap();
        
        // Test with A root (default)
        let a_root_pitch = model.get_root_pitch();
        assert!((a_root_pitch - 440.0).abs() < 0.01, "A root should give 440Hz root pitch");
        
        // Test with C root
        let mut actions = PresentationLayerActions::new();
        actions.root_note_adjustments.push(crate::presentation::AdjustRootNote {
            root_note: 60,
        });
        model.process_user_actions(actions);
        
        let c_root_pitch = model.get_root_pitch();
        assert!((c_root_pitch - 261.63).abs() < 0.01, "C root should give ~261.63Hz root pitch");
        
        // Test with other roots
        let test_roots = vec![
            (62, 293.66),
            (64, 329.63),
            (65, 349.23),
            (67, 392.00),
            (71, 493.88),
        ];
        
        for (root_note, expected_freq) in test_roots {
            let mut actions = PresentationLayerActions::new();
            actions.root_note_adjustments.push(crate::presentation::AdjustRootNote {
                root_note: root_note,
            });
            model.process_user_actions(actions);
            
            let root_pitch = model.get_root_pitch();
            assert!((root_pitch - expected_freq).abs() < 0.5, 
                "Root {} should give ~{}Hz root pitch, got {}Hz", 
                root_note, expected_freq, root_pitch);
        }
    }

    /// Test edge case frequency handling
    #[wasm_bindgen_test]
    fn test_edge_case_frequency_handling() {
        let mut model = DataModel::create().unwrap();
        
        // Test zero frequency
        let (midi_note, cents) = model.frequency_to_note_and_accuracy(0.0);
        assert_eq!(midi_note, 69);
        assert_eq!(cents, 0.0);
        
        // Test negative frequency
        let (midi_note, cents) = model.frequency_to_note_and_accuracy(-100.0);
        assert_eq!(midi_note, 69);
        assert_eq!(cents, 0.0);
        
        // Test very low frequency (below hearing range)
        let (midi_note, cents) = model.frequency_to_note_and_accuracy(10.0);
        // Should still process but might be inaccurate
        assert!(midi_note != 69 || cents != 0.0, "Should process very low frequency");
        
        // Test very high frequency (above musical range)
        let (midi_note, cents) = model.frequency_to_note_and_accuracy(10000.0);
        // Should still process
        assert!(cents.abs() <= 50.0, "Should clamp to reasonable cents range");
        
        // Test with no audio analysis
        let engine_data = EngineUpdateResult {
            audio_analysis: None,
            audio_errors: Vec::new(),
            permission_state: crate::shared_types::PermissionState::Granted,
        };
        
        let result = model.update(1.0, engine_data);
        assert_eq!(result.pitch, Pitch::NotDetected);
        assert_eq!(result.accuracy.cents_offset, 0.0);
    }

    /// Test scale field initialization in DataModel
    #[wasm_bindgen_test]
    fn test_data_model_scale_initialization() {
        let model = DataModel::create().unwrap();
        assert_eq!(model.current_scale, Scale::Chromatic, "Default scale should be Chromatic");
    }

    /// Test scale change processing
    #[wasm_bindgen_test]
    fn test_scale_change_processing() {
        let mut model = DataModel::create().unwrap();
        
        // Create presentation actions with scale change
        let mut actions = PresentationLayerActions::new();
        actions.scale_changes.push(crate::presentation::ScaleChangeAction {
            scale: Scale::Minor,
        });
        
        let result = model.process_user_actions(actions);
        
        // Should have no model-layer actions (scale changes are internal)
        assert_eq!(result.actions.audio_system_configurations.len(), 0);
        assert_eq!(result.actions.tuning_configurations.len(), 0);
        
        // Should have no validation errors
        assert_eq!(result.validation_errors.len(), 0);
        
        // Verify model state was updated
        assert_eq!(model.current_scale, Scale::Minor);
    }

    /// Test scale filtering in frequency_to_note_and_accuracy
    #[wasm_bindgen_test]
    fn test_scale_aware_note_filtering() {
        let mut model = DataModel::create().unwrap();
        
        // Set scale to Major
        model.current_scale = Scale::Major;
        
        // Test frequency for C# (not in C Major scale)
        let csharp_freq = 277.18; // C#4
        let (midi_note, cents) = model.frequency_to_note_and_accuracy(csharp_freq);
        
        // Should snap to nearest scale note (D4 = MIDI 62)
        assert_eq!(midi_note, 62, "C# should snap to D in C Major scale");
        assert!(cents < 0.0, "Should be flat relative to D");
        
        // Test frequency for D (in C Major scale)
        let d_freq = 293.66; // D4
        let (midi_note, cents) = model.frequency_to_note_and_accuracy(d_freq);
        
        // Should remain as D
        assert_eq!(midi_note, 62, "D should remain D in C Major scale");
        assert!(cents.abs() < 1.0, "Should be nearly in tune");
        
        // Change scale to Chromatic
        model.current_scale = Scale::Chromatic;
        
        // Test C# again - should now be accepted
        let (midi_note, cents) = model.frequency_to_note_and_accuracy(csharp_freq);
        assert_eq!(midi_note, 61, "C# should be C# in Chromatic scale");
        assert!(cents.abs() < 1.0, "Should be nearly in tune");
        
        // Test with MajorPentatonic scale
        model.current_scale = Scale::MajorPentatonic;
        
        // Test frequency for F (not in C Major Pentatonic)
        let f_freq = 349.23; // F4
        let (midi_note, cents) = model.frequency_to_note_and_accuracy(f_freq);
        
        // Should snap to nearest pentatonic note (E4 = MIDI 64 or G4 = MIDI 67)
        assert!(midi_note == 64 || midi_note == 67, "F should snap to E or G in C Major Pentatonic");
        
        // Test with MinorPentatonic scale
        model.current_scale = Scale::MinorPentatonic;
        
        // Test frequency for B (not in C Minor Pentatonic)
        let b_freq = 493.88; // B4
        let (midi_note, cents) = model.frequency_to_note_and_accuracy(b_freq);
        
        // Should snap to nearest pentatonic note (Bb4 = MIDI 70 or C5 = MIDI 72)
        assert!(midi_note == 70 || midi_note == 72, "B should snap to Bb or C in C Minor Pentatonic");
    }

    /// Test scale filtering with different root notes
    #[wasm_bindgen_test]
    fn test_scale_filtering_with_different_roots() {
        let mut model = DataModel::create().unwrap();
        
        // Set root to G (MIDI 67) and scale to Major
        let mut actions = PresentationLayerActions::new();
        actions.root_note_adjustments.push(crate::presentation::AdjustRootNote {
            root_note: 67, // G4
        });
        model.process_user_actions(actions);
        model.current_scale = Scale::Major;
        
        // Test frequency for G# (not in G Major scale)
        let gsharp_freq = 415.30; // G#4
        let (midi_note, cents) = model.frequency_to_note_and_accuracy(gsharp_freq);
        
        // Should snap to nearest scale note (A4 = MIDI 69)
        assert_eq!(midi_note, 69, "G# should snap to A in G Major scale");
        assert!(cents < 0.0, "Should be flat relative to A");
        
        // Test frequency for F# (in G Major scale - major 7th)
        let fsharp_freq = 369.99; // F#4
        let (midi_note, cents) = model.frequency_to_note_and_accuracy(fsharp_freq);
        
        // Should remain as F#
        assert_eq!(midi_note, 66, "F# should remain F# in G Major scale");
        assert!(cents.abs() < 1.0, "Should be nearly in tune");
        
        // Test with MajorPentatonic scale and root D (MIDI 62)
        let mut actions = PresentationLayerActions::new();
        actions.root_note_adjustments.push(crate::presentation::AdjustRootNote {
            root_note: 62, // D4
        });
        model.process_user_actions(actions);
        model.current_scale = Scale::MajorPentatonic;
        
        // Test frequency for G (not in D Major Pentatonic - which has D, E, F#, A, B)
        let g_freq = 392.00; // G4
        let (midi_note, cents) = model.frequency_to_note_and_accuracy(g_freq);
        
        // Should snap to nearest pentatonic note (F#4 = MIDI 66 or A4 = MIDI 69)
        assert!(midi_note == 66 || midi_note == 69, "G should snap to F# or A in D Major Pentatonic");
        
        // Test with MinorPentatonic scale and root A (MIDI 69)
        let mut actions = PresentationLayerActions::new();
        actions.root_note_adjustments.push(crate::presentation::AdjustRootNote {
            root_note: 69, // A4
        });
        model.process_user_actions(actions);
        model.current_scale = Scale::MinorPentatonic;
        
        // Test frequency for B (not in A Minor Pentatonic - which has A, C, D, E, G)
        let b_freq = 493.88; // B4
        let (midi_note, cents) = model.frequency_to_note_and_accuracy(b_freq);
        
        // Should snap to nearest pentatonic note (C5 = MIDI 72)
        assert_eq!(midi_note, 72, "B should snap to C in A Minor Pentatonic");
    }

    /// Test scale persistence in ModelUpdateResult
    #[wasm_bindgen_test]
    fn test_scale_in_model_update_result() {
        let mut model = DataModel::create().unwrap();
        
        // Change scale to Minor
        let mut actions = PresentationLayerActions::new();
        actions.scale_changes.push(crate::presentation::ScaleChangeAction {
            scale: Scale::Minor,
        });
        model.process_user_actions(actions);
        
        // Create engine data
        let engine_data = EngineUpdateResult {
            audio_analysis: None,
            audio_errors: Vec::new(),
            permission_state: crate::shared_types::PermissionState::NotRequested,
        };
        
        let result = model.update(1.0, engine_data);
        
        // Verify scale is included in result
        assert_eq!(result.scale, Scale::Minor);
    }

    /// Test multiple scale changes in sequence
    #[wasm_bindgen_test]
    fn test_multiple_scale_changes() {
        let mut model = DataModel::create().unwrap();
        
        // Create presentation actions with multiple scale changes
        let mut actions = PresentationLayerActions::new();
        actions.scale_changes.push(crate::presentation::ScaleChangeAction {
            scale: Scale::Minor,
        });
        actions.scale_changes.push(crate::presentation::ScaleChangeAction {
            scale: Scale::Chromatic,
        });
        actions.scale_changes.push(crate::presentation::ScaleChangeAction {
            scale: Scale::Major,
        });
        actions.scale_changes.push(crate::presentation::ScaleChangeAction {
            scale: Scale::MajorPentatonic,
        });
        actions.scale_changes.push(crate::presentation::ScaleChangeAction {
            scale: Scale::MinorPentatonic,
        });
        
        model.process_user_actions(actions);
        
        // Final scale should be MinorPentatonic
        assert_eq!(model.current_scale, Scale::MinorPentatonic);
    }

    /// Test scale change to same scale (no-op)
    #[wasm_bindgen_test]
    fn test_scale_change_to_same_scale() {
        let mut model = DataModel::create().unwrap();
        
        // Initial scale is Chromatic
        assert_eq!(model.current_scale, Scale::Chromatic);
        
        // Try to change to Chromatic again
        let mut actions = PresentationLayerActions::new();
        actions.scale_changes.push(crate::presentation::ScaleChangeAction {
            scale: Scale::Chromatic,
        });
        
        model.process_user_actions(actions);
        
        // Scale should still be Chromatic (no-op)
        assert_eq!(model.current_scale, Scale::Chromatic);
    }

    /// Test edge cases in scale-aware note filtering
    #[wasm_bindgen_test]
    fn test_scale_filtering_edge_cases() {
        let mut model = DataModel::create().unwrap();
        model.current_scale = Scale::Major;
        
        // Test very low frequency
        let (midi_note, _) = model.frequency_to_note_and_accuracy(20.0);
        // Should still apply scale filtering
        let interval = (midi_note as i32) - (model.root_note as i32);
        assert!(semitone_in_scale(Scale::Major, interval), "Low frequency note should be in scale");
        
        // Test very high frequency
        let (midi_note, _) = model.frequency_to_note_and_accuracy(4000.0);
        // Should still apply scale filtering
        let interval = (midi_note as i32) - (model.root_note as i32);
        assert!(semitone_in_scale(Scale::Major, interval), "High frequency note should be in scale");
        
        // Test frequency at MIDI boundary (127)
        model.root_note = 120; // Very high root
        let (midi_note, _) = model.frequency_to_note_and_accuracy(3000.0);
        assert!(midi_note <= 127, "MIDI note should be clamped to valid range");
        
        // Test edge cases with MajorPentatonic scale
        model.current_scale = Scale::MajorPentatonic;
        model.root_note = 60; // C4
        
        // Test low frequency with pentatonic
        let (midi_note, _) = model.frequency_to_note_and_accuracy(30.0);
        let interval = (midi_note as i32) - (model.root_note as i32);
        assert!(semitone_in_scale(Scale::MajorPentatonic, interval), "Low frequency note should be in pentatonic scale");
        
        // Test edge cases with MinorPentatonic scale
        model.current_scale = Scale::MinorPentatonic;
        
        // Test high frequency with pentatonic
        let (midi_note, _) = model.frequency_to_note_and_accuracy(3500.0);
        let interval = (midi_note as i32) - (model.root_note as i32);
        assert!(semitone_in_scale(Scale::MinorPentatonic, interval), "High frequency note should be in pentatonic scale");
    }

    /// Test engine-to-model data flow integration
    #[wasm_bindgen_test]
    fn test_engine_to_model_data_flow() {
        let mut model = DataModel::create().unwrap();
        
        // Create comprehensive engine data
        let audio_analysis = crate::shared_types::AudioAnalysis {
            volume_level: crate::shared_types::Volume { 
                peak_amplitude: -6.0, 
                rms_amplitude: -12.0 
            },
            pitch: crate::shared_types::Pitch::Detected(523.25, 0.88), // C5
            fft_data: None,
            timestamp: 1.0,
        };
        
        let engine_data = EngineUpdateResult {
            audio_analysis: Some(audio_analysis),
            audio_errors: vec![crate::shared_types::Error::ProcessingError("Test error".to_string())],
            permission_state: crate::shared_types::PermissionState::Granted,
            root_note_audio_enabled: false,
        };
        
        // Process engine data
        let result = model.update(1.0, engine_data);
        
        // Verify raw frequency was processed with tuning context
        assert_eq!(result.accuracy.closest_midi_note, 72);
        assert!(result.accuracy.cents_offset.abs() < 5.0, "C5 should be nearly in tune");
        
        // Verify volume data passed through
        assert_eq!(result.volume.peak_amplitude, -6.0);
        assert_eq!(result.volume.rms_amplitude, -12.0);
        
        // Verify pitch data passed through
        match result.pitch {
            Pitch::Detected(freq, clarity) => {
                assert_eq!(freq, 523.25);
                assert_eq!(clarity, 0.88);
            }
            _ => panic!("Expected detected pitch"),
        }
        
        // Verify error propagation
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0], Error::ProcessingError("Test error".to_string()));
        
        // Verify permission state
        assert_eq!(result.permission_state, PermissionState::Granted);
        
        // Verify tuning system is included
        assert_eq!(result.tuning_system, TuningSystem::EqualTemperament);
    }
}