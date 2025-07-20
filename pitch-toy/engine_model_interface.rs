use crate::action::Action;
use crate::observable_data::ObservableData;

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
pub struct RequestMicrophonePermissionAction;

impl Action for RequestMicrophonePermissionAction {}

pub type AudioAnalysisObservable = ObservableData<Option<AudioAnalysis>>;
pub type AudioErrorsObservable = ObservableData<Vec<AudioError>>;
pub type PermissionStateObservable = ObservableData<PermissionState>;