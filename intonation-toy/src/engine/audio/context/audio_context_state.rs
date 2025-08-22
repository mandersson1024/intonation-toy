use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum AudioContextState {
    Uninitialized,
    Initializing,
    Running,
    Suspended,
    Closed,
}

impl fmt::Display for AudioContextState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioContextState::Uninitialized => write!(f, "Uninitialized"),
            AudioContextState::Initializing => write!(f, "Initializing"),
            AudioContextState::Running => write!(f, "Running"),
            AudioContextState::Suspended => write!(f, "Suspended"),
            AudioContextState::Closed => write!(f, "Closed"),
        }
    }
}