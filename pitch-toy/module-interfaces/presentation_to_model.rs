use action::{Action, ActionTrigger, ActionListener};
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

impl Default for PresentationToModelInterface {
    fn default() -> Self {
        Self::new()
    }
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

    /// Get a trigger for requesting microphone permission that the presentation can use
    pub fn request_microphone_permission_trigger(&self) -> ActionTrigger<RequestMicrophonePermissionAction> {
        self.request_microphone_permission.trigger()
    }

    /// Get a trigger for setting tuning system that the presentation can use
    pub fn set_tuning_system_trigger(&self) -> ActionTrigger<SetTuningSystemAction> {
        self.set_tuning_system.trigger()
    }

    /// Get a trigger for setting root note that the presentation can use
    pub fn set_root_note_trigger(&self) -> ActionTrigger<SetRootNoteAction> {
        self.set_root_note.trigger()
    }

    /// Get a trigger for increasing root note that the presentation can use
    pub fn increase_root_note_trigger(&self) -> ActionTrigger<IncreaseRootNoteAction> {
        self.increase_root_note.trigger()
    }

    /// Get a trigger for decreasing root note that the presentation can use
    pub fn decrease_root_note_trigger(&self) -> ActionTrigger<DecreaseRootNoteAction> {
        self.decrease_root_note.trigger()
    }

    /// Get a listener for microphone permission requests that the model can use
    pub fn request_microphone_permission_listener(&self) -> ActionListener<RequestMicrophonePermissionAction> {
        self.request_microphone_permission.listener()
    }

    /// Get a listener for tuning system changes that the model can use
    pub fn set_tuning_system_listener(&self) -> ActionListener<SetTuningSystemAction> {
        self.set_tuning_system.listener()
    }

    /// Get a listener for root note changes that the model can use
    pub fn set_root_note_listener(&self) -> ActionListener<SetRootNoteAction> {
        self.set_root_note.listener()
    }

    /// Get a listener for root note increases that the model can use
    pub fn increase_root_note_listener(&self) -> ActionListener<IncreaseRootNoteAction> {
        self.increase_root_note.listener()
    }

    /// Get a listener for root note decreases that the model can use
    pub fn decrease_root_note_listener(&self) -> ActionListener<DecreaseRootNoteAction> {
        self.decrease_root_note.listener()
    }
}
