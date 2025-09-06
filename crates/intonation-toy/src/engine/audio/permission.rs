use std::fmt;

/// Microphone permission and device states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioPermission {
    /// Initial state, no permission requested yet
    Uninitialized,
    /// Permission request in progress
    Requesting,
    /// Permission granted and microphone accessible
    Granted,
    /// Permission denied by user 
    Denied,
    /// Device unavailable or not found
    Unavailable,
}

impl fmt::Display for AudioPermission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioPermission::Uninitialized => write!(f, "Uninitialized"),
            AudioPermission::Requesting => write!(f, "Requesting"),
            AudioPermission::Granted => write!(f, "Granted"),
            AudioPermission::Denied => write!(f, "Denied"),
            AudioPermission::Unavailable => write!(f, "Unavailable"),
        }
    }
}

/// Check if getUserMedia API is supported
pub fn is_user_media_supported() -> bool {
    let window = web_sys::window();
    if let Some(window) = window {
        let navigator = window.navigator();
        return navigator.media_devices().is_ok();
    }
    false
}
