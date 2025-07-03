use wasm_bindgen::prelude::*;
use web_sys::MediaStream;
use std::fmt;
use crate::modules::audio::permission::{PermissionManager, AudioPermission};

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
    state: AudioPermission,
    stream: Option<MediaStream>,
    stream_info: AudioStreamInfo,
}

impl MicrophoneManager {
    /// Create new microphone manager
    pub fn new() -> Self {
        Self {
            state: AudioPermission::Uninitialized,
            stream: None,
            stream_info: AudioStreamInfo::default(),
        }
    }

    /// Get current microphone state
    pub fn state(&self) -> &AudioPermission {
        &self.state
    }

    /// Get current stream info
    pub fn stream_info(&self) -> &AudioStreamInfo {
        &self.stream_info
    }

    /// Check if getUserMedia API is supported
    pub fn is_supported() -> bool {
        PermissionManager::is_supported()
    }

    /// Request microphone permission and access
    pub async fn request_permission(&mut self) -> Result<(), AudioError> {
        // Check API support
        if !Self::is_supported() {
            self.state = AudioPermission::Unavailable;
            return Err(AudioError::NotSupported(
                "getUserMedia API not supported".to_string()
            ));
        }

        self.state = AudioPermission::Requesting;

        match PermissionManager::request_microphone_permission().await {
            Ok(stream) => {
                // Update stream info with actual stream properties
                self.update_stream_info(&stream)?;
                
                self.stream = Some(stream);
                self.state = AudioPermission::Granted;
                Ok(())
            }
            Err(e) => {
                self.state = PermissionManager::error_to_permission(&e);
                Err(e)
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
    /// Note: This functionality is now provided by AudioContextManager
    pub async fn enumerate_devices(&self) -> Result<Vec<(String, String)>, AudioError> {
        // Stub implementation - returns empty list as placeholder
        Ok(Vec::new())
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
        self.state = AudioPermission::Uninitialized;
    }

    /// Get current MediaStream if available
    pub fn get_stream(&self) -> Option<&MediaStream> {
        self.stream.as_ref()
    }

    /// Check if microphone is currently active
    pub fn is_active(&self) -> bool {
        matches!(self.state, AudioPermission::Granted) && self.stream.is_some()
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
        assert_eq!(AudioPermission::Uninitialized.to_string(), "Uninitialized");
        assert_eq!(AudioPermission::Requesting.to_string(), "Requesting");
        assert_eq!(AudioPermission::Granted.to_string(), "Granted");
        assert_eq!(AudioPermission::Denied.to_string(), "Denied");
        assert_eq!(AudioPermission::Unavailable.to_string(), "Unavailable");
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
        assert_eq!(*manager.state(), AudioPermission::Uninitialized);
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