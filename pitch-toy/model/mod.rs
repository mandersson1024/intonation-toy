//! Model Layer - Data transformation and state management
//! 
//! This layer is responsible for:
//! - State management and business logic
//! - Data transformation between engine and presentation layers
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
//! 
//! ```rust
//! use pitch_toy::model::DataModel;
//! use pitch_toy::module_interfaces::{
//!     engine_to_model::EngineUpdateResult,
//!     model_to_presentation::ModelUpdateResult,
//! };
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
    reference_a4: f32,
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
    
    /// Calculate expected frequency for a given note
    fn note_to_frequency(&self, note: &Note, octave: i32) -> f32 {
        let note_offset = match note {
            Note::C => 0,
            Note::CSharp => 1,
            Note::D => 2,
            Note::DSharp => 3,
            Note::E => 4,
            Note::F => 5,
            Note::FSharp => 6,
            Note::G => 7,
            Note::GSharp => 8,
            Note::A => 9,
            Note::ASharp => 10,
            Note::B => 11,
        };
        
        // Calculate MIDI note number (A4 = 69, octave 4)
        let midi_note = (octave - 4) * 12 + note_offset + 69 - 9; // A is at index 9
        
        // Convert MIDI note to frequency
        self.reference_a4 * 2.0_f32.powf((midi_note - 69) as f32 / 12.0)
    }
    
    /// Normalize accuracy value to a 0.0-1.0 range
    /// 0.0 = perfectly in tune, 1.0 = 50 cents (half semitone) or worse
    fn normalize_accuracy(&self, cents: f32) -> f32 {
        let abs_cents = cents.abs();
        // Clamp to max 50 cents for normalization
        let clamped_cents = abs_cents.min(50.0);
        clamped_cents / 50.0
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
}