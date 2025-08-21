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
//! - `tuning_fork_adjustments` - User modifications to the tuning fork
//! 
//! ### Processing: Business Logic Validation
//! - `validate_microphone_permission_request_with_error()` - Ensures permission requests are appropriate
//! - `validate_tuning_system_change_with_error()` - Validates tuning system changes
//! - `validate_tuning_fork_adjustment_with_error()` - Validates tuning fork adjustments
//! 
//! ### Output: ModelLayerActions
//! - `microphone_permission_requests` - Validated permission requests
//! - `audio_system_configurations` - Validated tuning system configurations
//! - `tuning_configurations` - Validated tuning and tuning fork configurations
//! 
//! ### State Management
//! - `apply_tuning_system_change()` - Updates internal tuning system state
//! - `apply_tuning_fork_change()` - Updates internal tuning fork state
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

use crate::shared_types::{EngineUpdateResult, ModelUpdateResult, Volume, Pitch, IntonationData, TuningSystem, Scale, MidiNote, is_valid_midi_note};
use crate::presentation::PresentationLayerActions;
use crate::common::smoothing::EmaSmoother;
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
    /// Tuning fork is already set to requested value
    TuningForkNoteAlreadySet(MidiNote),
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
/// including tuning system and tuning fork.
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateTuningConfigurationAction {
    pub tuning_system: TuningSystem,
    pub tuning_fork_note: MidiNote,
}

/// Validated tuning fork audio configuration
/// 
/// This struct represents a tuning fork audio configuration that has been validated
/// by the model layer's business logic. It contains the audio generation settings
/// including enabled state and frequency.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigureTuningForkAction {
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
    
    /// Validated tuning fork audio configurations
    pub tuning_fork_configurations: Vec<ConfigureTuningForkAction>,
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
            tuning_fork_configurations: Vec::new(),
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
    
    /// Current tuning fork for tuning calculations
    tuning_fork_note: MidiNote,
    
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
            tuning_fork_note: crate::app_config::DEFAULT_TUNING_FORK_NOTE,
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
        // Process audio analysis from engine data with smoothing
        let (volume, pitch) = if let Some(audio_analysis) = engine_data.audio_analysis {
            // Extract volume from audio analysis
            let volume = Volume {
                peak_amplitude: audio_analysis.volume_level.peak_amplitude,
                rms_amplitude: audio_analysis.volume_level.rms_amplitude,
            };
            
            // Apply smoothing to pitch data
            let pitch = match audio_analysis.pitch {
                crate::shared_types::Pitch::Detected(frequency, clarity) => {
                    let smoothed_frequency = self.frequency_smoother.apply(frequency);
                    let smoothed_clarity = self.clarity_smoother.apply(clarity);
                    self.last_detected_pitch = Some((frequency, clarity));
                    Pitch::Detected(smoothed_frequency, smoothed_clarity)
                }
                crate::shared_types::Pitch::NotDetected => {
                    if let Some((last_freq, _)) = self.last_detected_pitch {
                        let smoothed_clarity = self.clarity_smoother.apply(0.0);
                        if smoothed_clarity < crate::app_config::CLARITY_THRESHOLD * 0.5 {
                            self.last_detected_pitch = None;
                            self.frequency_smoother.reset();
                            self.clarity_smoother.reset();
                            Pitch::NotDetected
                        } else {
                            Pitch::Detected(last_freq, smoothed_clarity)
                        }
                    } else {
                        self.last_detected_pitch = None;
                        self.frequency_smoother.reset();
                        self.clarity_smoother.reset();
                        Pitch::NotDetected
                    }
                }
            };
            
            (volume, pitch)
        } else {
            (Volume { peak_amplitude: -60.0, rms_amplitude: -60.0 }, Pitch::NotDetected)
        };
        
        let errors = engine_data.audio_errors;
        
        let permission_state = engine_data.permission_state;
        
        let volume_peak = volume.peak_amplitude >= crate::app_config::VOLUME_PEAK_THRESHOLD;
        
        // Handle out-of-bounds frequencies by treating them as NotDetected
        let effective_pitch = match pitch {
            Pitch::Detected(frequency, clarity) => {
                if self.frequency_to_note_and_accuracy(frequency).is_some() {
                    Pitch::Detected(frequency, clarity)
                } else {
                    // Frequency outside valid MIDI range - treat as not detected
                    Pitch::NotDetected
                }
            }
            Pitch::NotDetected => Pitch::NotDetected,
        };

        let (accuracy, interval_semitones) = match effective_pitch {
            Pitch::Detected(frequency, _clarity) => {
                // We know this will return Some() because we checked above
                let (closest_midi_note, cents_offset) = self.frequency_to_note_and_accuracy(frequency).unwrap();
                let accuracy = IntonationData { closest_midi_note: Some(closest_midi_note), cents_offset };
                let interval = (closest_midi_note as i32) - (self.tuning_fork_note as i32);
                (accuracy, interval)
            }
            Pitch::NotDetected => {
                let accuracy = IntonationData { 
                    closest_midi_note: None,
                    cents_offset: 0.0,
                };
                (accuracy, 0)
            }
        };

        ModelUpdateResult {
            volume,
            volume_peak,
            pitch: effective_pitch,
            accuracy: accuracy.clone(),
            tuning_system: self.tuning_system,
            scale: self.current_scale,
            errors,
            permission_state,
            closest_midi_note: accuracy.closest_midi_note,
            cents_offset: accuracy.cents_offset,
            interval_semitones,
            tuning_fork_note: self.tuning_fork_note,
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
    /// - Tuning fork adjustments are valid and result in proper frequency calculations
    /// - All actions maintain system consistency and state integrity
    /// 
    /// # Current Implementation
    /// 
    /// The validation logic:
    /// 1. Validates microphone permission requests against current permission state
    /// 2. Validates tuning system changes and updates internal state when valid
    /// 3. Validates tuning fork adjustments and updates internal state when valid
    /// 4. Combines validated actions into complete system configurations
    /// 5. Collects validation errors for failed actions
    /// 
    /// # State Updates
    /// 
    /// When actions pass validation, the model's internal state is immediately updated
    /// using `apply_tuning_system_change()` and `apply_tuning_fork_change()` methods.
    /// This ensures the model's state remains synchronized with validated user actions.
    pub fn process_user_actions(&mut self, presentation_actions: PresentationLayerActions) -> ProcessedActions {
        let mut model_actions = ModelLayerActions::new();
        let mut validation_errors = Vec::new();
        
        // Process tuning system changes
        for tuning_change in presentation_actions.tuning_system_changes {
            match self.validate_tuning_system_change_with_error(&tuning_change.tuning_system) {
                Ok(()) => {
                    let config = ConfigureAudioSystemAction { tuning_system: tuning_change.tuning_system };
                    self.apply_tuning_system_change(&config);
                    model_actions.audio_system_configurations.push(config);
                }
                Err(error) => validation_errors.push(error),
            }
        }
        
        // Process tuning fork adjustments
        for tuning_fork_adjustment in presentation_actions.tuning_fork_adjustments {
            let midi_note = tuning_fork_adjustment.note;
            match self.validate_tuning_fork_adjustment_with_error(&midi_note) {
                Ok(()) => {
                    let config = UpdateTuningConfigurationAction {
                        tuning_system: self.tuning_system,
                        tuning_fork_note: midi_note,
                    };
                    self.apply_tuning_fork_change(&config);
                    model_actions.tuning_configurations.push(config);
                }
                Err(error) => validation_errors.push(error),
            }
        }
        
        // Process scale changes
        for scale_change in presentation_actions.scale_changes {
            if scale_change.scale != self.current_scale {
                self.apply_scale_change(&scale_change);
            }
        }
        
        // Process tuning fork audio configurations
        crate::common::dev_log!("MODEL: Processing {} tuning fork audio configurations", presentation_actions.tuning_fork_configurations.len());
        for tuning_fork_config in presentation_actions.tuning_fork_configurations {
            crate::common::dev_log!("MODEL: Processing tuning fork audio config");
            
            match self.validate_tuning_fork_audio_configuration_with_error(&tuning_fork_config) {
                Ok(()) => {
                    let config = ConfigureTuningForkAction {
                        frequency: tuning_fork_config.frequency,
                        volume: tuning_fork_config.volume,
                    };
                    model_actions.tuning_fork_configurations.push(config);
                    crate::common::dev_log!("MODEL: ✓ Tuning fork audio configuration validated and queued for engine execution");
                }
                Err(error) => {
                    crate::common::warn_log!("Tuning fork audio configuration validation failed: {:?}", error);
                    validation_errors.push(error);
                }
            }
        }
        
        ProcessedActions { actions: model_actions, validation_errors }
    }

    
    /// Convert a frequency to the closest musical note with tuning system and scale awareness
    /// 
    /// This method applies the current tuning system, tuning fork context, and scale filtering
    /// to convert raw frequency data into musical note identification. The conversion process:
    /// 
    /// 1. Validates the input frequency (must be positive)
    /// 2. Calculates a root pitch frequency based on tuning system and tuning fork
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
    /// # Tuning Fork Context
    /// 
    /// The tuning fork determines the reference point for all calculations:
    /// - Changes the root pitch frequency used for MIDI conversion
    /// - Affects which frequencies are considered "in tune"
    /// - Allows the same frequency to map to different accuracy values
    /// 
    /// # Returns
    /// 
    /// Returns Some((Note, accuracy_cents)) where:
    /// - Note: The closest musical note to the frequency (filtered by scale)
    /// - accuracy_cents: Deviation in cents (negative = flat, positive = sharp)
    /// 
    /// Returns None if the frequency is outside the valid MIDI note range (0-127)
    fn frequency_to_note_and_accuracy(&self, frequency: f32) -> Option<(MidiNote, f32)> {
        if frequency <= 0.0 {
            warn_log!("[MODEL] Invalid frequency for note conversion: {}", frequency);
            return None;
        }
        
        let root_pitch = self.get_root_pitch();
        let interval_result = crate::music_theory::frequency_to_interval_semitones_scale_aware(
            self.tuning_system,
            root_pitch,
            frequency,
            self.current_scale,
        );
        
        let raw_midi_note = self.tuning_fork_note as i32 + interval_result.semitones;
        
        // Return None if outside valid MIDI range
        if !(0..=127).contains(&raw_midi_note) {
            return None;
        }
        
        let midi_note = raw_midi_note as u8;
        if !is_valid_midi_note(midi_note) {
            return None;
        }
        
        Some((midi_note, interval_result.cents))
    }
    
    
    
    fn get_root_pitch(&self) -> f32 {
        crate::music_theory::midi_note_to_standard_frequency(self.tuning_fork_note)
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
    
    
    /// Validate tuning fork adjustment request with detailed error reporting
    /// 
    /// Ensures that a tuning fork adjustment is valid and results in proper frequency
    /// calculations. This validation maintains musical accuracy and prevents
    /// invalid note configurations.
    /// 
    /// # Arguments
    /// 
    /// * `new_tuning_fork` - The requested tuning fork to validate
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` if the tuning fork adjustment should be processed, or a specific
    /// `ValidationError` describing why the adjustment was rejected.
    /// 
    /// # Current Implementation
    /// 
    /// Validates that the new tuning fork is different from the current one and is
    /// a valid musical note. Future implementations will add:
    /// - Compatibility checks with current tuning system
    /// - Musical theory validation
    fn validate_tuning_fork_adjustment_with_error(&self, new_tuning_fork: &MidiNote) -> Result<(), ValidationError> {
        if *new_tuning_fork == self.tuning_fork_note {
            Err(ValidationError::TuningForkNoteAlreadySet(*new_tuning_fork))
        } else {
            Ok(())
        }
    }
    
    /// Validate tuning fork audio configuration request
    /// 
    /// Validates a tuning fork audio configuration request by checking that the frequency
    /// is valid and the configuration is different from the current state.
    /// Future implementations will add:
    /// - Frequency range validation
    /// - Audio system compatibility checks
    /// - Volume level validation
    fn validate_tuning_fork_audio_configuration_with_error(&self, config: &crate::presentation::ConfigureTuningFork) -> Result<(), ValidationError> {
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
    
    /// Apply tuning fork change to internal state
    /// 
    /// Updates the internal tuning fork based on a validated tuning fork adjustment.
    /// This method should only be called with actions that have passed business logic validation.
    /// 
    /// # Arguments
    /// 
    /// * `action` - The validated tuning configuration to apply
    /// 
    /// # Current Implementation
    /// 
    /// Updates the internal tuning system and tuning fork directly from the validated action.
    /// Future implementations will add:
    /// - State change notifications
    /// - Logging of configuration changes
    /// - Validation of state consistency after changes
    /// - Recalculation of derived values
    fn apply_tuning_fork_change(&mut self, action: &UpdateTuningConfigurationAction) {
        crate::common::dev_log!(
            "Model layer: Tuning fork changed from {} to {}",
            self.tuning_fork_note, action.tuning_fork_note
        );
        self.tuning_system = action.tuning_system;
        self.tuning_fork_note = action.tuning_fork_note;
    }
}

