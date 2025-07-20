use action::{Action};

#[derive(Debug, Clone, PartialEq)]
pub struct RequestMicrophonePermissionAction;

pub struct ModelToEngineInterface {
    request_microphone_permission: Action<RequestMicrophonePermissionAction>,
}

impl ModelToEngineInterface {
    /// Create a new Model → Engine interface with all actions
    pub fn new() -> Self {
        Self {
            request_microphone_permission: Action::new(),
        }
    }
}

