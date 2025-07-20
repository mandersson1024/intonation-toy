use action::{Action};
use super::model_to_presentation::{TuningSystem, Note};

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

pub struct PresentationToModelInterface {
    // Actions (owned by presentation)
    request_microphone_permission: Action<RequestMicrophonePermissionAction>,
    set_tuning_system: Action<SetTuningSystemAction>,
    set_root_note: Action<SetRootNoteAction>,
    increase_root_note: Action<IncreaseRootNoteAction>,
    decrease_root_note: Action<DecreaseRootNoteAction>,
}

impl PresentationToModelInterface {
    /// Create a new Presentation → Model interface with all actions
    pub fn new() -> Self {
        Self {
            request_microphone_permission: Action::new(),
            set_tuning_system: Action::new(),
            set_root_note: Action::new(),
            increase_root_note: Action::new(),
            decrease_root_note: Action::new(),
        }
    }
}
