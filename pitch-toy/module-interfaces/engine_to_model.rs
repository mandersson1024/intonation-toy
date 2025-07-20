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

pub struct EngineToModelInterface {
    audio_analysis_source: DataSource<Option<AudioAnalysis>>,
    audio_errors_source: DataSource<Vec<AudioError>>,
    permission_state_source: DataSource<PermissionState>,
}

impl EngineToModelInterface {
    pub fn new() -> Self {
        Self {
            audio_analysis_source: DataSource::new(None),
            audio_errors_source: DataSource::new(Vec::new()),
            permission_state_source: DataSource::new(PermissionState::NotRequested),
        }
    }
}
