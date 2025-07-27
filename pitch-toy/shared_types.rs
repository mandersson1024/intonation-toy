//! Shared data types for the pitch-toy application.
//!
//! This module contains all shared data structures used for communication
//! between the three layers of the architecture:
//! - Engine → Model: Raw audio analysis and system state
//! - Model → Presentation: Processed data ready for display
//!
//! The types are organized to facilitate clear data flow and minimize
//! duplication across the application layers.

#[derive(Debug, Clone, PartialEq)]
pub struct Volume {
    pub peak_amplitude: f32,
    pub rms_amplitude: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pitch {
    Detected(f32, f32), // frequency, clarity
    NotDetected,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioAnalysis {
    pub volume_level: Volume,
    pub pitch: Pitch,
    pub fft_data: Option<Vec<f32>>, // roadmap
    pub timestamp: f64,
}


#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    MicrophonePermissionDenied,
    MicrophoneNotAvailable,
    ProcessingError(String),
    BrowserApiNotSupported,
    AudioContextInitFailed,
    AudioContextSuspended,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PermissionState {
    NotRequested,
    Requested,
    Granted,
    Denied,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EngineUpdateResult {
    pub audio_analysis: Option<AudioAnalysis>,
    pub audio_errors: Vec<Error>,
    pub permission_state: PermissionState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelUpdateResult {
    pub volume: Volume,
    pub pitch: Pitch,
    pub errors: Vec<Error>,
    pub permission_state: PermissionState,
}


#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_engine_update_result_creation() {
        let test_analysis = AudioAnalysis {
            volume_level: Volume { peak_amplitude: 0.5, rms_amplitude: 0.3 },
            pitch: Pitch::Detected(440.0, 0.9),
            fft_data: None,
            timestamp: 123.456,
        };

        let test_errors = vec![Error::MicrophonePermissionDenied];

        let update_result = EngineUpdateResult {
            audio_analysis: Some(test_analysis.clone()),
            audio_errors: test_errors.clone(),
            permission_state: PermissionState::Granted,
        };

        assert_eq!(update_result.audio_analysis, Some(test_analysis));
        assert_eq!(update_result.audio_errors, test_errors);
        assert_eq!(update_result.permission_state, PermissionState::Granted);
    }

    #[wasm_bindgen_test]
    fn test_model_update_result_creation() {
        let test_volume = Volume { peak_amplitude: 0.8, rms_amplitude: 0.6 };
        let test_pitch = Pitch::Detected(440.0, 0.9);
        let test_errors = vec![Error::ProcessingError("Test error".to_string())];

        let update_result = ModelUpdateResult {
            volume: test_volume.clone(),
            pitch: test_pitch.clone(),
            errors: test_errors.clone(),
            permission_state: PermissionState::Granted,
        };

        assert_eq!(update_result.volume, test_volume);
        assert_eq!(update_result.pitch, test_pitch);
        assert_eq!(update_result.errors, test_errors);
        assert_eq!(update_result.permission_state, PermissionState::Granted);
    }

    #[wasm_bindgen_test]
    fn test_volume_creation() {
        let volume = Volume { peak_amplitude: 1.0, rms_amplitude: 0.7 };
        assert_eq!(volume.peak_amplitude, 1.0);
        assert_eq!(volume.rms_amplitude, 0.7);
    }

    #[wasm_bindgen_test]
    fn test_pitch_variants() {
        let detected = Pitch::Detected(440.0, 0.9);
        let not_detected = Pitch::NotDetected;
        
        assert_ne!(detected, not_detected);
        
        if let Pitch::Detected(freq, clarity) = detected {
            assert_eq!(freq, 440.0);
            assert_eq!(clarity, 0.9);
        }
    }

    #[wasm_bindgen_test]
    fn test_permission_state_variants() {
        let states = vec![
            PermissionState::NotRequested,
            PermissionState::Requested,
            PermissionState::Granted,
            PermissionState::Denied,
        ];
        
        for (i, state1) in states.iter().enumerate() {
            for (j, state2) in states.iter().enumerate() {
                if i == j {
                    assert_eq!(state1, state2);
                } else {
                    assert_ne!(state1, state2);
                }
            }
        }
    }

    #[wasm_bindgen_test]
    fn test_error_variants() {
        let errors = vec![
            Error::MicrophonePermissionDenied,
            Error::MicrophoneNotAvailable,
            Error::ProcessingError("test".to_string()),
            Error::BrowserApiNotSupported,
            Error::AudioContextInitFailed,
            Error::AudioContextSuspended,
        ];
        
        assert_eq!(errors.len(), 6);
    }


    #[wasm_bindgen_test]
    fn test_audio_analysis_creation() {
        let analysis = AudioAnalysis {
            volume_level: Volume { peak_amplitude: 0.8, rms_amplitude: 0.6 },
            pitch: Pitch::Detected(440.0, 0.9),
            fft_data: Some(vec![0.1, 0.2, 0.3]),
            timestamp: 123.456,
        };
        
        assert_eq!(analysis.volume_level.peak_amplitude, 0.8);
        assert_eq!(analysis.volume_level.rms_amplitude, 0.6);
        assert_eq!(analysis.timestamp, 123.456);
        assert!(analysis.fft_data.is_some());
    }
}