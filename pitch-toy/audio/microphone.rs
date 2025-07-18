use wasm_bindgen::prelude::*;
use web_sys::MediaStream;
use std::fmt;
use crate::audio::permission::{PermissionManager, AudioPermission};

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


/// Connect microphone to AudioWorklet using AudioSystemContext
/// 
/// This function establishes the complete audio processing pipeline from microphone input
/// to AudioWorklet processing using dependency injection with AudioSystemContext.
/// 
/// # Parameters
/// - `audio_context`: AudioSystemContext instance containing all audio components
/// 
/// # Returns
/// - `Ok(())` if the connection was successful
/// - `Err(String)` with error details if the connection failed
/// 
/// # Process
/// 1. Requests microphone permission from the user
/// 2. Creates MediaStreamAudioSourceNode from the microphone stream
/// 3. Connects the source to the AudioWorklet processor
/// 4. Starts audio processing if not already active
/// 
/// # Example
/// ```rust
/// let context = AudioSystemContext::new(/* setters */);
/// match connect_microphone_to_audioworklet_with_context(&context).await {
///     Ok(_) => println!("Microphone connected successfully"),
///     Err(e) => eprintln!("Failed to connect microphone: {}", e),
/// }
/// ```
pub async fn connect_microphone_to_audioworklet_with_context(
    audio_context: &std::cell::RefCell<super::context::AudioSystemContext>
) -> Result<(), String> {
    use crate::common::dev_log;
    
    dev_log!("Starting connect_microphone_to_audioworklet_with_context");
    dev_log!("Requesting microphone permission and connecting to AudioWorklet");
    
    // Request microphone permission and get stream
    let media_stream = match PermissionManager::request_microphone_permission().await {
        Ok(stream) => {
            dev_log!("✓ Microphone permission granted, received MediaStream");
            // Check if stream has active tracks
            let tracks = stream.get_tracks();
            dev_log!("MediaStream has {} tracks", tracks.length());
            for i in 0..tracks.length() {
                if let Some(track) = tracks.get(i).dyn_ref::<web_sys::MediaStreamTrack>() {
                    dev_log!("Track {}: kind={}, enabled={}, ready_state={:?}", 
                        i, track.kind(), track.enabled(), track.ready_state());
                }
            }
            stream
        }
        Err(e) => {
            dev_log!("✗ Microphone permission failed: {:?}", e);
            return Err(format!("Failed to get microphone permission: {:?}", e));
        }
    };
    
    // Get audio context and AudioWorklet manager from context
    dev_log!("Getting AudioContext manager from context");
    let audio_context_instance = {
        let context_borrowed = audio_context.borrow();
        let manager_ref = context_borrowed.get_audio_context_manager();
        manager_ref.borrow().get_context()
            .ok_or_else(|| "AudioContext not available".to_string())?
            .clone()
    };
    
    // Resume AudioContext if suspended (required for processing to start)
    {
        let mut context_borrowed = audio_context.borrow_mut();
        dev_log!("DEBUG: Resuming AudioContext");
        if let Err(e) = context_borrowed.resume_if_suspended().await {
            dev_log!("⚠️ Failed to resume AudioContext: {:?}", e);
        } else {
            dev_log!("✓ AudioContext resumed for microphone processing");
        }
    }
    
    dev_log!("DEBUG: Getting AudioWorklet manager from context");
    let _audioworklet_manager = audio_context.borrow().get_audioworklet_manager()
        .ok_or_else(|| "AudioWorklet manager not initialized in context".to_string())?;
    
    // Create audio source from MediaStream
    dev_log!("DEBUG: Creating MediaStreamAudioSourceNode");
    
    let source = match audio_context_instance.create_media_stream_source(&media_stream) {
        Ok(source_node) => {
            dev_log!("✓ Created MediaStreamAudioSourceNode from microphone");
            source_node
        }
        Err(e) => {
            dev_log!("✗ Failed to create audio source: {:?}", e);
            return Err(format!("Failed to create audio source: {:?}", e));
        }
    };
    
    // Connect microphone source to AudioWorklet
    dev_log!("DEBUG: Connecting microphone source to AudioWorklet");
    
    // Note: We need to use global access here because the context provides read-only access
    // This is a limitation of the current design that could be improved in future iterations
    // Note: Global access has been removed as part of migration to AudioSystemContext
    // Use the context parameter to access the AudioWorklet manager
    let result = {
        let mut context_borrowed = audio_context.borrow_mut();
        if let Some(ref mut worklet_manager) = context_borrowed.get_audioworklet_manager_mut() {
            worklet_manager.connect_microphone(source.as_ref())
        } else {
            return Err("AudioWorklet manager not available in context".to_string());
        }
    };
    
    match result {
        Ok(_) => {
            dev_log!("✓ Microphone successfully connected to AudioWorklet");
            
            // Note: No need to connect to destination - microphone → AudioWorklet is sufficient for processing
            
            // Ensure processing is active after connection
            let mut context_borrowed = audio_context.borrow_mut();
            if let Some(ref mut worklet_manager) = context_borrowed.get_audioworklet_manager_mut() {
                dev_log!("Found worklet manager, checking if processing: {}", worklet_manager.is_processing());
                if !worklet_manager.is_processing() {
                    dev_log!("Starting AudioWorklet processing after microphone connection...");
                    match worklet_manager.start_processing() {
                        Ok(_) => {
                            dev_log!("AudioWorklet processing started - audio pipeline active");
                        }
                        Err(e) => {
                            dev_log!("Failed to start processing after microphone connection: {:?}", e);
                        }
                    }
                } else {
                    dev_log!("AudioWorklet already processing - audio pipeline active");
                }
            } else {
                dev_log!("No AudioWorklet manager available");
            }
            
            // Status is published automatically by the AudioWorklet manager
            
            Ok(())
        }
        Err(e) => {
            dev_log!("✗ Failed to connect microphone to AudioWorklet: {:?}", e);
            Err(format!("Failed to connect microphone: {:?}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_microphone_state_display() {
        assert_eq!(AudioPermission::Uninitialized.to_string(), "Uninitialized");
        assert_eq!(AudioPermission::Requesting.to_string(), "Requesting");
        assert_eq!(AudioPermission::Granted.to_string(), "Granted");
        assert_eq!(AudioPermission::Denied.to_string(), "Denied");
        assert_eq!(AudioPermission::Unavailable.to_string(), "Unavailable");
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_stream_info_default() {
        let info = AudioStreamInfo::default();
        assert_eq!(info.sample_rate, 48000.0);
        assert_eq!(info.buffer_size, 1024);
        assert!(info.device_id.is_none());
        assert!(info.device_label.is_none());
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_error_display() {
        let error = AudioError::PermissionDenied("test".to_string());
        assert_eq!(error.to_string(), "Permission denied: test");
        
        let error = AudioError::DeviceUnavailable("test".to_string());
        assert_eq!(error.to_string(), "Device unavailable: test");
        
        let error = AudioError::NotSupported("test".to_string());
        assert_eq!(error.to_string(), "Not supported: test");
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_microphone_manager_new() {
        let manager = MicrophoneManager::new();
        assert_eq!(*manager.state(), AudioPermission::Uninitialized);
        assert!(!manager.is_active());
        assert!(manager.get_stream().is_none());
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_microphone_manager_stream_info() {
        let manager = MicrophoneManager::new();
        let info = manager.stream_info();
        assert_eq!(info.sample_rate, 48000.0);
        assert_eq!(info.buffer_size, 1024);
    }
}