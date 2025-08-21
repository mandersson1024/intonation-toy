use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{MediaStream, MediaStreamConstraints};
use super::AudioError;
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
    /// Must be called from a user gesture (button click, etc.)
    pub async fn request_microphone_permission() -> Result<MediaStream, AudioError> {
        // Check API support
        if !Self::is_supported() {
            return Err(AudioError::NotSupported(
                "getUserMedia API not supported".to_string()
            ));
        }

        // Get navigator and media devices
        let window = web_sys::window()
            .ok_or_else(|| AudioError::Generic("No window object".to_string()))?;
        
        let navigator = window.navigator();
        let media_devices = navigator.media_devices()
            .map_err(|_| AudioError::NotSupported("MediaDevices not available".to_string()))?;

        // Create audio constraints
        let constraints = MediaStreamConstraints::new();
        constraints.set_audio(&JsValue::TRUE);
        constraints.set_video(&JsValue::FALSE);

        // Request user media - must be in same call stack as user gesture
        let promise = media_devices.get_user_media_with_constraints(&constraints)
            .map_err(|e| AudioError::Generic(format!("Failed to call getUserMedia: {:?}", e)))?;

        match JsFuture::from(promise).await {
            Ok(stream_js) => {
                let stream = MediaStream::from(stream_js);
                Ok(stream)
            }
            Err(e) => {
                // Determine error type from JS error
                let error_msg = format!("{:?}", e);
                
                if error_msg.contains("NotAllowedError") || error_msg.contains("PermissionDeniedError") {
                    Err(AudioError::PermissionDenied("User denied microphone access".to_string()))
                } else if error_msg.contains("NotFoundError") || error_msg.contains("DevicesNotFoundError") {
                    Err(AudioError::DeviceUnavailable("No microphone device found".to_string()))
                } else {
                    Err(AudioError::Generic(format!("getUserMedia failed: {}", error_msg)))
                }
            }
        }
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

    

}


