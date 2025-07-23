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
//! data through method parameters and return values. It provides basic audio
//! data transformation functionality.
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
        // TODO: Initialize internal state for pitch tracking, tuning systems, etc.
        Ok(Self {})
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
    /// This is a basic implementation that:
    /// 1. Processes audio analysis from engine data into volume and pitch information
    /// 2. Provides placeholder values for accuracy and tuning system
    /// 3. Passes through errors and permission state
    /// 
    /// # Future Enhancement
    /// 
    /// When fully implemented, this method will:
    /// 1. Transform frequency data into musical notes
    /// 2. Update pitch tracking history and patterns
    /// 3. Calculate accuracy metrics based on pitch stability
    /// 4. Apply different tuning systems
    /// 5. Handle complex audio analysis processing
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
        
        // Return processed model data
        ModelUpdateResult {
            volume,
            pitch,
            accuracy: Accuracy {
                closest_note: Note::A,  // Placeholder - would calculate from pitch
                accuracy: 0.0,         // Placeholder - would calculate based on pitch stability
            },
            tuning_system: TuningSystem::EqualTemperament, // Placeholder - would come from user settings
            errors,
            permission_state,
        }
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
}