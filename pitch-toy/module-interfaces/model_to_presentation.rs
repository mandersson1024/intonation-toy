use observable_data::{DataSource};

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
}
