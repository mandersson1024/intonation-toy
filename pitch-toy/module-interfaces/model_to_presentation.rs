use observable_data::{DataSource, DataObserver, DataSourceSetter};

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

/// Bundled interface containing all Model → Presentation data sources
pub struct ModelToPresentationInterface {
    // Data sources (owned by model)
    volume_source: DataSource<Volume>,
    pitch_source: DataSource<Pitch>,
    accuracy_source: DataSource<Accuracy>,
    tuning_system_source: DataSource<TuningSystem>,
    errors_source: DataSource<Vec<Error>>,
    permission_state_source: DataSource<PermissionState>,
}

impl Default for ModelToPresentationInterface {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelToPresentationInterface {
    /// Create a new Model → Presentation interface with all data sources
    pub fn new() -> Self {
        Self {
            volume_source: DataSource::new(Volume { peak: 0.0, rms: 0.0 }),
            pitch_source: DataSource::new(Pitch::NotDetected),
            accuracy_source: DataSource::new(Accuracy {
                closest_note: Note::A,
                accuracy: 0.0,
            }),
            tuning_system_source: DataSource::new(TuningSystem::EqualTemperament),
            errors_source: DataSource::new(Vec::new()),
            permission_state_source: DataSource::new(PermissionState::NotRequested),
        }
    }

    /// Get a setter for volume data that the model can use to push updates
    pub fn volume_setter(&self) -> DataSourceSetter<Volume> {
        self.volume_source.setter()
    }

    /// Get a setter for pitch data that the model can use to push updates
    pub fn pitch_setter(&self) -> DataSourceSetter<Pitch> {
        self.pitch_source.setter()
    }

    /// Get a setter for accuracy data that the model can use to push updates
    pub fn accuracy_setter(&self) -> DataSourceSetter<Accuracy> {
        self.accuracy_source.setter()
    }

    /// Get a setter for tuning system that the model can use to push updates
    pub fn tuning_system_setter(&self) -> DataSourceSetter<TuningSystem> {
        self.tuning_system_source.setter()
    }

    /// Get a setter for errors that the model can use to push updates
    pub fn errors_setter(&self) -> DataSourceSetter<Vec<Error>> {
        self.errors_source.setter()
    }

    /// Get a setter for permission state that the model can use to push updates
    pub fn permission_state_setter(&self) -> DataSourceSetter<PermissionState> {
        self.permission_state_source.setter()
    }

    /// Get an observer for volume data that the presentation can use to read updates
    pub fn volume_observer(&self) -> DataObserver<Volume> {
        self.volume_source.observer()
    }

    /// Get an observer for pitch data that the presentation can use to read updates
    pub fn pitch_observer(&self) -> DataObserver<Pitch> {
        self.pitch_source.observer()
    }

    /// Get an observer for accuracy data that the presentation can use to read updates
    pub fn accuracy_observer(&self) -> DataObserver<Accuracy> {
        self.accuracy_source.observer()
    }

    /// Get an observer for tuning system that the presentation can use to read updates
    pub fn tuning_system_observer(&self) -> DataObserver<TuningSystem> {
        self.tuning_system_source.observer()
    }

    /// Get an observer for errors that the presentation can use to read updates
    pub fn errors_observer(&self) -> DataObserver<Vec<Error>> {
        self.errors_source.observer()
    }

    /// Get an observer for permission state that the presentation can use to read updates
    pub fn permission_state_observer(&self) -> DataObserver<PermissionState> {
        self.permission_state_source.observer()
    }
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
