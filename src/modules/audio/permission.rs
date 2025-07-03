use web_sys::MediaStream;
use super::{AudioError, AudioPermission};

/// Permission manager for getUserMedia API
pub struct PermissionManager;

impl PermissionManager {
    /// Check if getUserMedia API is supported
    pub fn is_supported() -> bool {
        let window = web_sys::window();
        if let Some(window) = window {
            let navigator = window.navigator();
            return navigator.media_devices().is_ok();
        }
        false
    }

    /// Request microphone permission and return MediaStream
    /// TODO: Implement actual permission request functionality
    pub async fn request_microphone_permission() -> Result<MediaStream, AudioError> {
        // Stub implementation - returns error indicating not implemented
        Err(AudioError::Generic("Permission request not implemented yet".to_string()))
    }

    /// Map AudioError to AudioPermission
    pub fn error_to_permission(error: &AudioError) -> AudioPermission {
        match error {
            AudioError::PermissionDenied(_) => AudioPermission::Denied,
            AudioError::DeviceUnavailable(_) => AudioPermission::Unavailable,
            AudioError::NotSupported(_) => AudioPermission::Unavailable,
            AudioError::StreamInitFailed(_) => AudioPermission::Unavailable,
            AudioError::Generic(_) => AudioPermission::Unavailable,
        }
    }

    /// Map AudioError to AudioPermission (legacy method name)
    pub fn error_to_state(error: &AudioError) -> AudioPermission {
        Self::error_to_permission(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_to_permission_mapping() {
        let error = AudioError::PermissionDenied("test".to_string());
        assert_eq!(PermissionManager::error_to_permission(&error), AudioPermission::Denied);

        let error = AudioError::DeviceUnavailable("test".to_string());
        assert_eq!(PermissionManager::error_to_permission(&error), AudioPermission::Unavailable);

        let error = AudioError::NotSupported("test".to_string());
        assert_eq!(PermissionManager::error_to_permission(&error), AudioPermission::Unavailable);

        let error = AudioError::StreamInitFailed("test".to_string());
        assert_eq!(PermissionManager::error_to_permission(&error), AudioPermission::Unavailable);

        let error = AudioError::Generic("test".to_string());
        assert_eq!(PermissionManager::error_to_permission(&error), AudioPermission::Unavailable);
    }

    #[test]
    fn test_error_to_state_mapping() {
        let error = AudioError::PermissionDenied("test".to_string());
        assert_eq!(PermissionManager::error_to_state(&error), AudioPermission::Denied);

        let error = AudioError::DeviceUnavailable("test".to_string());
        assert_eq!(PermissionManager::error_to_state(&error), AudioPermission::Unavailable);

        let error = AudioError::NotSupported("test".to_string());
        assert_eq!(PermissionManager::error_to_state(&error), AudioPermission::Unavailable);
    }
}