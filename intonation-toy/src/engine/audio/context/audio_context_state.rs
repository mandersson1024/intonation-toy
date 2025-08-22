#[derive(Debug, Clone, PartialEq)]
pub enum AudioContextState {
    Uninitialized,
    Initializing,
    Running,
    Suspended,
    Closed,
}