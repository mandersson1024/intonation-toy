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
//! use pitch_toy::shared_types::{
//!     EngineUpdateResult,
//!     ModelUpdateResult,
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

use crate::shared_types::{EngineUpdateResult, ModelUpdateResult, Volume, Pitch, IntonationData, TuningSystem, Error, PermissionState, MidiNote, is_valid_midi_note};
use crate::presentation::PresentationLayerActions;
use crate::common::trace_log;

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
            root_note: 69, // Standard A4 root note (MIDI 69)
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
                crate::shared_types::Error::AudioContextInitFailed => {
                    Error::AudioContextInitFailed
                }
                crate::shared_types::Error::AudioContextSuspended => {
                    Error::AudioContextSuspended
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
                trace_log!(
                    "[MODEL] Processing detected pitch: {}Hz with clarity {} using {:?} tuning, root {:?}",
                    frequency, clarity, self.tuning_system, self.root_note
                );
                
                // Apply tuning-aware frequency to note conversion
                let (closest_midi_note, accuracy_cents) = self.frequency_to_note_and_accuracy(frequency);
                
                trace_log!(
                    "[MODEL] Result: Note {:?}, cents offset {} ({}Hz)",
                    closest_midi_note, accuracy_cents, frequency
                );
                
                IntonationData {
                    closest_midi_note,
                    cents_offset: accuracy_cents,
                }
            }
            Pitch::NotDetected => {
                // No pitch detected - return default values
                trace_log!("[MODEL] No pitch detected, returning default accuracy");
                
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

        trace_log!(
            "[MODEL] Interval calculation: detected MIDI {} - root MIDI {} = {} semitones",
            accuracy.closest_midi_note, self.root_note, interval_semitones
        );

        // Return processed model data with both legacy and flattened fields
        ModelUpdateResult {
            volume,
            pitch,
            accuracy: accuracy.clone(), // Keep for backward compatibility
            tuning_system: self.tuning_system.clone(),
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
        
        ProcessedActions {
            actions: model_actions,
            validation_errors,
        }
    }

    #[cfg(test)]
    pub fn process_user_actions_test(&mut self, presentation_actions: PresentationLayerActions) -> ProcessedActions {
        self.process_user_actions(presentation_actions)
    }
    
    /// Convert a frequency to the closest musical note with tuning system awareness
    /// 
    /// This method applies the current tuning system and root note context to convert
    /// raw frequency data into musical note identification. The conversion process:
    /// 
    /// 1. Validates the input frequency (must be positive)
    /// 2. Calculates a root pitch frequency based on tuning system and root note
    /// 3. Converts frequency to MIDI note space using tuning-specific formulas
    /// 4. Maps MIDI note to the closest musical note
    /// 5. Calculates accuracy in cents (1/100th of a semitone)
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
    /// - Note: The closest musical note to the frequency
    /// - accuracy_cents: Deviation in cents (negative = flat, positive = sharp)
    fn frequency_to_note_and_accuracy(&self, frequency: f32) -> (MidiNote, f32) {
        // Handle edge case: invalid or zero frequency
        if frequency <= 0.0 {
            #[cfg(debug_assertions)]
            web_sys::console::warn_1(&format!("[MODEL] Invalid frequency for note conversion: {}", frequency).into());
            return (69, 0.0); // Return A4 (MIDI 69) as default
        }
        
        // Handle edge case: extremely low frequency (below human hearing ~20Hz)
        if frequency < 20.0 {
            #[cfg(debug_assertions)]
            web_sys::console::warn_1(&format!("[MODEL] Extremely low frequency detected: {}Hz", frequency).into());
            // Still process but warn in debug mode
        }
        
        // Handle edge case: extremely high frequency (above typical musical range ~4186Hz for C8)
        if frequency > 4186.0 {
            #[cfg(debug_assertions)]
            web_sys::console::warn_1(&format!("[MODEL] Extremely high frequency detected: {}Hz", frequency).into());
            // Still process but warn in debug mode
        }
        
        // Get root pitch frequency based on current tuning system and root note
        // This is the key to tuning-aware processing
        let root_pitch = self.get_root_pitch();
        
        trace_log!(
            "[MODEL] Converting frequency {}Hz with tuning {:?}, root {:?}, root pitch {}Hz",
            frequency, self.tuning_system, self.root_note, root_pitch
        );
        
        // Calculate MIDI note number from frequency using tuning-specific formula
        let midi_note = match self.tuning_system {
            TuningSystem::EqualTemperament => {
                // Equal temperament: logarithmic relationship between frequency and pitch
                // Formula: MIDI = root_midi + 12 * log2(frequency / root_pitch_frequency)
                self.root_note as f32 + 12.0 * (frequency / root_pitch).log2()
            }
            TuningSystem::JustIntonation => {
                // For now, use equal temperament formula as placeholder
                // TODO: Implement proper just intonation calculations
                self.root_note as f32 + 12.0 * (frequency / root_pitch).log2()
            }
        };
        
        // Round to nearest MIDI note for note identification
        let rounded_midi = midi_note.round();
        
        // Clamp to valid MIDI range (0-127)
        let clamped_midi = rounded_midi.max(0.0).min(127.0) as u8;
        
        // Validate using the utility function
        let final_midi_note = if is_valid_midi_note(clamped_midi) {
            clamped_midi
        } else {
            69 // Default to A4 if validation fails
        };
        
        // Calculate accuracy in cents (100 cents = 1 semitone)
        // Positive = sharp, Negative = flat
        let accuracy_cents = (midi_note - rounded_midi) * 100.0;
        
        trace_log!(
            "[MODEL] Frequency {}Hz -> MIDI note {} with accuracy {} cents",
            frequency, final_midi_note, accuracy_cents
        );
        
        (final_midi_note, accuracy_cents)
    }
    
    
    
    /// Get the root pitch frequency for the current root note and tuning system
    /// 
    /// This method calculates the root pitch frequency that serves as the basis for
    /// all frequency-to-note conversions. The root pitch frequency depends on both
    /// the tuning system and the selected root note.
    /// 
    /// # Root Pitch Calculation
    /// 
    /// The root pitch frequency is calculated differently for each tuning system:
    /// 
    /// ## Equal Temperament
    /// - Uses A4 = 440Hz as the standard reference
    /// - Calculates other notes using the formula: f = 440 * 2^((n-69)/12)
    /// - Where n is the MIDI note number
    /// 
    /// ## Future Tuning Systems
    /// - Just Intonation: Will calculate based on frequency ratios from root
    /// - Pythagorean: Will use perfect fifth ratios
    /// - Custom: May allow user-defined reference frequencies
    /// 
    /// # Root Note Impact
    /// 
    /// The root note changes which frequency is considered the "tonic":
    /// - If root is A, then A4 = 440Hz is the root pitch
    /// - If root is C, then C4 = 261.63Hz becomes the root pitch
    /// - All other frequencies are calculated relative to this root pitch
    fn get_root_pitch(&self) -> f32 {
        // Base frequency is A4 = 440Hz in standard tuning
        const A4_FREQUENCY: f32 = 440.0;
        const A4_MIDI: i32 = 69; // MIDI note number for A4
        
        match self.tuning_system {
            TuningSystem::EqualTemperament => {
                // Equal temperament: fixed ratio of 2^(1/12) between semitones
                // Calculate root pitch frequency based on root note
                
                // Use the root note directly as MIDI number
                let midi_diff = self.root_note as i32 - A4_MIDI;
                
                // Calculate frequency using equal temperament formula
                // f = f0 * 2^(n/12) where n is semitone distance
                let root_pitch = A4_FREQUENCY * 2.0_f32.powf(midi_diff as f32 / 12.0);
                
                trace_log!(
                    "[MODEL] Root pitch frequency for MIDI {} in {:?}: {}Hz (diff from A4: {})",
                    self.root_note, self.tuning_system, root_pitch, midi_diff
                );
                
                root_pitch
            }
            TuningSystem::JustIntonation => {
                // For now, use the same calculation as equal temperament
                // TODO: Implement proper just intonation frequency ratios
                let midi_diff = self.root_note as i32 - A4_MIDI;
                A4_FREQUENCY * 2.0_f32.powf(midi_diff as f32 / 12.0)
            }
        }
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
            audio_errors: vec![crate::shared_types::Error::AudioContextSuspended],
            permission_state: crate::shared_types::PermissionState::Granted,
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
        assert_eq!(result.errors[0], Error::AudioContextSuspended);
        
        // Verify permission state
        assert_eq!(result.permission_state, PermissionState::Granted);
        
        // Verify tuning system is included
        assert_eq!(result.tuning_system, TuningSystem::EqualTemperament);
    }
}