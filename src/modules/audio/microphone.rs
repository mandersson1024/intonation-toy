use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{MediaStream, MediaStreamConstraints};
use std::fmt;

/// Microphone permission and device states
#[derive(Debug, Clone, PartialEq)]
pub enum MicrophoneState {
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

impl fmt::Display for MicrophoneState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MicrophoneState::Uninitialized => write!(f, "Uninitialized"),
            MicrophoneState::Requesting => write!(f, "Requesting"),
            MicrophoneState::Granted => write!(f, "Granted"),
            MicrophoneState::Denied => write!(f, "Denied"),
            MicrophoneState::Unavailable => write!(f, "Unavailable"),
        }
    }
}

/// Audio stream information
#[derive(Debug, Clone)]
pub struct AudioStreamInfo {
    pub sample_rate: f64,
    pub buffer_size: u32,
    pub device_id: Option<String>,
    pub device_label: Option<String>,
}

impl Default for AudioStreamInfo {
    fn default() -> Self {
        Self {
            sample_rate: 48000.0, // 48kHz default
            buffer_size: 1024,    // Production buffer size
            device_id: None,
            device_label: None,
        }
    }
}

/// Audio processing errors
#[derive(Debug, Clone)]
pub enum AudioError {
    /// Permission denied by user
    PermissionDenied(String),
    /// Device not available
    DeviceUnavailable(String),
    /// Browser API not supported
    NotSupported(String),
    /// Stream initialization failed
    StreamInitFailed(String),
    /// Generic error with context
    Generic(String),
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            AudioError::DeviceUnavailable(msg) => write!(f, "Device unavailable: {}", msg),
            AudioError::NotSupported(msg) => write!(f, "Not supported: {}", msg),
            AudioError::StreamInitFailed(msg) => write!(f, "Stream initialization failed: {}", msg),
            AudioError::Generic(msg) => write!(f, "Audio error: {}", msg),
        }
    }
}

/// Microphone manager handles getUserMedia permissions and device access
pub struct MicrophoneManager {
    state: MicrophoneState,
    stream: Option<MediaStream>,
    stream_info: AudioStreamInfo,
}

impl MicrophoneManager {
    /// Create new microphone manager
    pub fn new() -> Self {
        Self {
            state: MicrophoneState::Uninitialized,
            stream: None,
            stream_info: AudioStreamInfo::default(),
        }
    }

    /// Get current microphone state
    pub fn state(&self) -> &MicrophoneState {
        &self.state
    }

    /// Get current stream info
    pub fn stream_info(&self) -> &AudioStreamInfo {
        &self.stream_info
    }

    /// Check if getUserMedia API is supported
    pub fn is_supported() -> bool {
        let window = web_sys::window();
        if let Some(window) = window {
            let navigator = window.navigator();
            return navigator.media_devices().is_ok();
        }
        false
    }

    /// Request microphone permission and access
    pub async fn request_permission(&mut self) -> Result<(), AudioError> {
        // Check API support
        if !Self::is_supported() {
            self.state = MicrophoneState::Unavailable;
            return Err(AudioError::NotSupported(
                "getUserMedia API not supported".to_string()
            ));
        }

        self.state = MicrophoneState::Requesting;

        // Get navigator and media devices
        let window = web_sys::window()
            .ok_or_else(|| AudioError::Generic("No window object".to_string()))?;
        
        let navigator = window.navigator();
        let media_devices = navigator.media_devices()
            .map_err(|_| AudioError::NotSupported("MediaDevices not available".to_string()))?;

        // Create audio constraints
        let mut constraints = MediaStreamConstraints::new();
        constraints.set_audio(&JsValue::TRUE);
        constraints.set_video(&JsValue::FALSE);

        // Request user media
        let promise = media_devices.get_user_media_with_constraints(&constraints)
            .map_err(|e| AudioError::Generic(format!("Failed to call getUserMedia: {:?}", e)))?;

        match JsFuture::from(promise).await {
            Ok(stream_js) => {
                let stream = MediaStream::from(stream_js);
                
                // Update stream info with actual stream properties
                self.update_stream_info(&stream)?;
                
                self.stream = Some(stream);
                self.state = MicrophoneState::Granted;
                Ok(())
            }
            Err(e) => {
                // Determine error type from JS error
                let error_msg = format!("{:?}", e);
                
                if error_msg.contains("NotAllowedError") || error_msg.contains("PermissionDeniedError") {
                    self.state = MicrophoneState::Denied;
                    Err(AudioError::PermissionDenied("User denied microphone access".to_string()))
                } else if error_msg.contains("NotFoundError") || error_msg.contains("DevicesNotFoundError") {
                    self.state = MicrophoneState::Unavailable;
                    Err(AudioError::DeviceUnavailable("No microphone device found".to_string()))
                } else {
                    self.state = MicrophoneState::Unavailable;
                    Err(AudioError::Generic(format!("getUserMedia failed: {}", error_msg)))
                }
            }
        }
    }

    /// Update stream information from MediaStream
    fn update_stream_info(&mut self, _stream: &MediaStream) -> Result<(), AudioError> {
        // Note: MediaStreamTrack.getSettings() is not available in all browsers
        // For now, we'll use default values and improve this in future iterations
        // when browser compatibility requirements are better defined
        
        // Keep default values for sample rate and buffer size
        // TODO: Implement proper settings extraction when browser support improves
        
        Ok(())
    }

    /// Get available audio input devices
    pub async fn enumerate_devices(&self) -> Result<Vec<(String, String)>, AudioError> {
        if !Self::is_supported() {
            return Err(AudioError::NotSupported("MediaDevices API not supported".to_string()));
        }

        let window = web_sys::window()
            .ok_or_else(|| AudioError::Generic("No window object".to_string()))?;
        
        let navigator = window.navigator();
        let media_devices = navigator.media_devices()
            .map_err(|_| AudioError::NotSupported("MediaDevices not available".to_string()))?;

        let promise = media_devices.enumerate_devices()
            .map_err(|e| AudioError::Generic(format!("Failed to enumerate devices: {:?}", e)))?;

        match JsFuture::from(promise).await {
            Ok(devices_js) => {
                let devices = js_sys::Array::from(&devices_js);
                let mut audio_devices = Vec::new();

                for i in 0..devices.length() {
                    if let Some(device_info) = devices.get(i).dyn_ref::<web_sys::MediaDeviceInfo>() {
                        if device_info.kind() == web_sys::MediaDeviceKind::Audioinput {
                            let device_id = device_info.device_id();
                            let label = device_info.label();
                            audio_devices.push((device_id, label));
                        }
                    }
                }

                Ok(audio_devices)
            }
            Err(e) => Err(AudioError::Generic(format!("Device enumeration failed: {:?}", e)))
        }
    }

    /// Stop current microphone stream
    pub fn stop_stream(&mut self) {
        if let Some(stream) = &self.stream {
            let tracks = stream.get_tracks();
            for i in 0..tracks.length() {
                if let Some(track) = tracks.get(i).dyn_ref::<web_sys::MediaStreamTrack>() {
                    track.stop();
                }
            }
        }
        
        self.stream = None;
        self.state = MicrophoneState::Uninitialized;
    }

    /// Get current MediaStream if available
    pub fn get_stream(&self) -> Option<&MediaStream> {
        self.stream.as_ref()
    }

    /// Check if microphone is currently active
    pub fn is_active(&self) -> bool {
        matches!(self.state, MicrophoneState::Granted) && self.stream.is_some()
    }
}

impl Drop for MicrophoneManager {
    fn drop(&mut self) {
        self.stop_stream();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_microphone_state_display() {
        assert_eq!(MicrophoneState::Uninitialized.to_string(), "Uninitialized");
        assert_eq!(MicrophoneState::Requesting.to_string(), "Requesting");
        assert_eq!(MicrophoneState::Granted.to_string(), "Granted");
        assert_eq!(MicrophoneState::Denied.to_string(), "Denied");
        assert_eq!(MicrophoneState::Unavailable.to_string(), "Unavailable");
    }

    #[test]
    fn test_audio_stream_info_default() {
        let info = AudioStreamInfo::default();
        assert_eq!(info.sample_rate, 48000.0);
        assert_eq!(info.buffer_size, 1024);
        assert!(info.device_id.is_none());
        assert!(info.device_label.is_none());
    }

    #[test]
    fn test_audio_error_display() {
        let error = AudioError::PermissionDenied("test".to_string());
        assert_eq!(error.to_string(), "Permission denied: test");
        
        let error = AudioError::DeviceUnavailable("test".to_string());
        assert_eq!(error.to_string(), "Device unavailable: test");
        
        let error = AudioError::NotSupported("test".to_string());
        assert_eq!(error.to_string(), "Not supported: test");
    }

    #[test]
    fn test_microphone_manager_new() {
        let manager = MicrophoneManager::new();
        assert_eq!(*manager.state(), MicrophoneState::Uninitialized);
        assert!(!manager.is_active());
        assert!(manager.get_stream().is_none());
    }

    #[test]
    fn test_microphone_manager_stream_info() {
        let manager = MicrophoneManager::new();
        let info = manager.stream_info();
        assert_eq!(info.sample_rate, 48000.0);
        assert_eq!(info.buffer_size, 1024);
    }
}