
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
pub struct Accuracy {
    pub closest_note: Note,
    pub accuracy: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TuningSystem {
    EqualTemperament,
    JustIntonation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Note {
    C, CSharp, D, DSharp, E, F, FSharp, G, GSharp, A, ASharp, B,
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
pub struct ModelUpdateResult {
    pub volume: Volume,
    pub pitch: Pitch,
    pub accuracy: Accuracy,
    pub tuning_system: TuningSystem,
    pub errors: Vec<Error>,
    pub permission_state: PermissionState,
}


#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_model_update_result_creation() {
        let test_volume = Volume { peak: 0.8, rms: 0.6 };
        let test_pitch = Pitch::Detected(440.0, 0.9);
        let test_accuracy = Accuracy {
            closest_note: Note::A,
            accuracy: 0.95,
        };
        let test_errors = vec![Error::ProcessingError("Test error".to_string())];

        let update_result = ModelUpdateResult {
            volume: test_volume.clone(),
            pitch: test_pitch.clone(),
            accuracy: test_accuracy.clone(),
            tuning_system: TuningSystem::EqualTemperament,
            errors: test_errors.clone(),
            permission_state: PermissionState::Granted,
        };

        assert_eq!(update_result.volume, test_volume);
        assert_eq!(update_result.pitch, test_pitch);
        assert_eq!(update_result.accuracy, test_accuracy);
        assert_eq!(update_result.tuning_system, TuningSystem::EqualTemperament);
        assert_eq!(update_result.errors, test_errors);
        assert_eq!(update_result.permission_state, PermissionState::Granted);
    }
}
