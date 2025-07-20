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
pub struct Accuracy {
    pub closest_note: Note,
    pub accuracy: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TuningSystem {
    EqualTemperament,
    JustIntonation,
    // Additional tuning systems can be added here
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
pub struct RequestMicrophonePermissionAction;

#[derive(Debug, Clone, PartialEq)]
pub struct SetTuningSystemAction {
    pub tuning_system: TuningSystem,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetRootNoteAction {
    pub root_note: Note,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IncreaseRootNoteAction;

#[derive(Debug, Clone, PartialEq)]
pub struct DecreaseRootNoteAction;

impl Action for RequestMicrophonePermissionAction {}
impl Action for SetTuningSystemAction {}
impl Action for SetRootNoteAction {}
impl Action for IncreaseRootNoteAction {}
impl Action for DecreaseRootNoteAction {}

pub type VolumeObservable = ObservableData<Volume>;
pub type PitchObservable = ObservableData<Pitch>;
pub type AccuracyObservable = ObservableData<Accuracy>;
pub type TuningSystemObservable = ObservableData<TuningSystem>;
pub type ErrorsObservable = ObservableData<Vec<Error>>;
pub type PermissionStateObservable = ObservableData<PermissionState>;