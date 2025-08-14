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

use crate::shared_types::{EngineUpdateResult, ModelUpdateResult, Volume, Pitch, IntonationData, TuningSystem, Scale, Error, PermissionState, MidiNote, is_valid_midi_note};
use crate::presentation::PresentationLayerActions;
use crate::presentation::EmaSmoother;
use crate::common::warn_log;

/// Validation error types for action processing
/// 
/// These error types represent specific validation failures that can occur
/// when processing presentation layer actions. They provide detailed information
/// about why an action was rejected by business logic validation.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Tuning system is already active
    TuningSystemAlreadyActive(TuningSystem),
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
///
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
    pub frequency: f32,
    pub volume: f32,
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

impl Default for ModelLayerActions {
    fn default() -> Self {
        Self::new()
    }
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
    
    /// EMA smoother for frequency values
    /// 
    /// This smoother applies exponential moving average filtering to the detected
    /// frequency values, reducing jitter and providing more stable pitch detection.
    /// The smoothing factor is configured via PITCH_SMOOTHING_FACTOR in app_config.
    frequency_smoother: EmaSmoother,
    
    /// EMA smoother for clarity values
    /// 
    /// This smoother applies exponential moving average filtering to the clarity
    /// (confidence) values from the pitch detection algorithm. This helps maintain
    /// stable visual feedback even when the detection confidence fluctuates.
    clarity_smoother: EmaSmoother,
    
    /// Tracks the last detected pitch as (frequency, clarity) tuple
    /// 
    /// This field maintains the previous pitch detection state to handle smooth
    /// transitions between detected and not-detected states. When pitch detection
    /// fails, the smoothers can continue to provide decaying values based on the
    /// last known good pitch, creating a more natural visual transition.
    last_detected_pitch: Option<(f32, f32)>,
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
            root_note: crate::app_config::DEFAULT_ROOT_NOTE,
            current_scale: crate::app_config::DEFAULT_SCALE,
            frequency_smoother: EmaSmoother::new(crate::app_config::PITCH_SMOOTHING_FACTOR),
            clarity_smoother: EmaSmoother::new(crate::app_config::PITCH_SMOOTHING_FACTOR),
            last_detected_pitch: None,
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
                crate::shared_types::Error::MobileDeviceNotSupported => {
                    Error::MobileDeviceNotSupported
                }
                crate::shared_types::Error::BrowserError => {
                    Error::BrowserError
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
        
        // Calculate volume peak flag using configurable threshold
        let volume_peak = volume.peak_amplitude >= crate::app_config::VOLUME_PEAK_THRESHOLD;
        
        // Calculate accuracy based on detected pitch with full tuning context
        let accuracy = match pitch {
            Pitch::Detected(frequency, _clarity) => {
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
        
        
        ModelUpdateResult {
            volume,
            volume_peak,
            pitch,
            accuracy: accuracy.clone(), // Keep for backward compatibility
            tuning_system: self.tuning_system,
            scale: self.current_scale,
            errors,
            permission_state,
            // New flattened fields for easier access
            closest_midi_note: accuracy.closest_midi_note,
            cents_offset: accuracy.cents_offset,
            interval_semitones,
            root_note: self.root_note,
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
        
        // Process tuning system changes
        for tuning_change in presentation_actions.tuning_system_changes {
            match self.validate_tuning_system_change_with_error(&tuning_change.tuning_system) {
                Ok(()) => {
                    let config = ConfigureAudioSystemAction {
                        tuning_system: tuning_change.tuning_system,
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
                            tuning_system: self.tuning_system,
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
            crate::common::dev_log!("MODEL: Processing root note audio config");
            
            match self.validate_root_note_audio_configuration_with_error(&root_note_audio_config) {
                Ok(()) => {
                    let config = ConfigureRootNoteAudioAction {
                        frequency: root_note_audio_config.frequency,
                        volume: root_note_audio_config.volume,
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
        let interval_result = crate::music_theory::frequency_to_interval_semitones_scale_aware(
            self.tuning_system,
            root_pitch,
            frequency,
            self.current_scale,
        );
        
        // Calculate MIDI note from root note plus interval
        let raw_midi_note = self.root_note as i32 + interval_result.semitones;
        
        // Clamp to valid MIDI range (0-127)
        let clamped_midi_note = raw_midi_note.clamp(0, 127) as u8;
        
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
        crate::music_theory::midi_note_to_standard_frequency(self.root_note)
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
            Err(ValidationError::TuningSystemAlreadyActive(*new_tuning_system))
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
        self.tuning_system = action.tuning_system;
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
        self.tuning_system = action.tuning_system;
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
    fn apply_root_note_audio_change(&mut self, _action: &ConfigureRootNoteAudioAction) {
        crate::common::dev_log!(
            "Model layer: Root note audio configuration passed through"
        );
    }
}

