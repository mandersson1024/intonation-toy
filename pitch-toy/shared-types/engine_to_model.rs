
#[derive(Debug, Clone, PartialEq)]
pub struct Volume {
    pub peak: f32,
    pub rms: f32,
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
pub enum AudioError {
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
    pub audio_errors: Vec<AudioError>,
    pub permission_state: PermissionState,
}


#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_engine_update_result_creation() {
        let test_analysis = AudioAnalysis {
            volume_level: Volume { peak: 0.5, rms: 0.3 },
            pitch: Pitch::Detected(440.0, 0.9),
            fft_data: None,
            timestamp: 123.456,
        };

        let test_errors = vec![AudioError::MicrophonePermissionDenied];

        let update_result = EngineUpdateResult {
            audio_analysis: Some(test_analysis.clone()),
            audio_errors: test_errors.clone(),
            permission_state: PermissionState::Granted,
        };

        assert_eq!(update_result.audio_analysis, Some(test_analysis));
        assert_eq!(update_result.audio_errors, test_errors);
        assert_eq!(update_result.permission_state, PermissionState::Granted);
    }
}
