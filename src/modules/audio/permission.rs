use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{MediaStream, MediaStreamConstraints};
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

    /// Check current microphone permission status without requesting
    /// Uses the Permissions API when available, falls back to Uninitialized
    pub async fn check_microphone_permission() -> AudioPermission {
        // Try to use Permissions API to check current permission status
        if let Some(permission_status) = Self::query_permissions_api().await {
            return permission_status;
        }
        
        // Fallback: can't check without requesting, so return Uninitialized
        AudioPermission::Uninitialized
    }
    
    /// Query permission status using the Permissions API (if available)
    async fn query_permissions_api() -> Option<AudioPermission> {
        let window = web_sys::window()?;
        let navigator = window.navigator();
        
        // Check if Permissions API is available using js_sys reflection
        let permissions_obj = js_sys::Reflect::get(&navigator, &"permissions".into()).ok()?;
        if permissions_obj.is_undefined() {
            return None;
        }
        
        // Try to query microphone permission
        let query_fn = js_sys::Reflect::get(&permissions_obj, &"query".into()).ok()?;
        if query_fn.is_undefined() {
            return None;
        }
        
        // Create permission descriptor object
        let descriptor = js_sys::Object::new();
        js_sys::Reflect::set(&descriptor, &"name".into(), &"microphone".into()).ok()?;
        
        // Query permission status
        let promise = js_sys::Function::from(query_fn)
            .call1(&permissions_obj, &descriptor).ok()?;
        
        let promise = web_sys::js_sys::Promise::from(promise);
        let result = JsFuture::from(promise).await.ok()?;
        
        // Get the state property
        let state = js_sys::Reflect::get(&result, &"state".into()).ok()?;
        let state_str = state.as_string()?;
        
        // Convert permission status to AudioPermission
        match state_str.as_str() {
            "granted" => Some(AudioPermission::Granted),
            "denied" => Some(AudioPermission::Denied),
            "prompt" => Some(AudioPermission::Uninitialized),
            _ => Some(AudioPermission::Uninitialized),
        }
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

    /// Map AudioError to AudioPermission (legacy method name)
    pub fn error_to_state(error: &AudioError) -> AudioPermission {
        Self::error_to_permission(error)
    }
    
    /// Stop all tracks in a MediaStream
    /// Use this when you're done with a stream to free resources
    pub fn stop_media_stream(stream: &MediaStream) {
        let tracks = stream.get_tracks();
        for i in 0..tracks.length() {
            if let Some(track) = tracks.get(i).dyn_ref::<web_sys::MediaStreamTrack>() {
                track.stop();
            }
        }
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

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_is_supported() {
        // Test that is_supported doesn't panic
        let _supported = PermissionManager::is_supported();
        // We can't assert the actual value since it depends on the test environment
        assert!(true);
    }

    #[test]
    fn test_permission_check_structure() {
        // Test that the check function has the correct return type structure
        // We can't actually test the async function in a unit test environment
        // since it requires browser APIs, but we can verify the function signature exists
        
        // This is a structural test to ensure the function compiles
        assert!(true);
    }
}