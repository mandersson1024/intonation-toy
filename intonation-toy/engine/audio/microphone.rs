use wasm_bindgen::prelude::*;
use web_sys::MediaStream;
use std::fmt;
use super::permission::{PermissionManager, AudioPermission};
use super::buffer::STANDARD_SAMPLE_RATE;

/// Audio stream information
#[derive(Debug, Clone)]
pub struct AudioStreamInfo {
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub device_id: Option<String>,
    pub device_label: Option<String>,
}

impl Default for AudioStreamInfo {
    fn default() -> Self {
        Self {
            sample_rate: STANDARD_SAMPLE_RATE,
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

impl Default for MicrophoneManager {
    fn default() -> Self {
        Self::new()
    }
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
    
    let mut context_borrowed = audio_context.borrow_mut();
    let _ = context_borrowed.resume_if_suspended().await;
    
    // Wait for AudioContext to actually be running  
    let audio_context_ref = {
        let context_borrowed = audio_context.borrow();
        context_borrowed.get_audio_context_manager().clone()
    };
    
    // Wait up to 500ms for AudioContext to be running
    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 50;
    
    loop {
        let state = {
            let context_manager = audio_context_ref.borrow();
            let context = context_manager.get_context()
                .ok_or_else(|| "AudioContext not available after resume".to_string())?;
            context.state()
        };
        
        if state == web_sys::AudioContextState::Running || attempts >= MAX_ATTEMPTS {
            break;
        }
        
        
        // Simple delay using setTimeout equivalent
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            let window = web_sys::window().unwrap();
            window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 10).unwrap();
        });
        
        if let Err(_) = wasm_bindgen_futures::JsFuture::from(promise).await {
            // If delay fails, just continue
        }
        
        attempts += 1;
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
                if !worklet_manager.is_processing() && worklet_manager.start_processing().is_err() {
                    // Processing start failed - AudioWorklet will handle error reporting
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

/// Connect an existing MediaStream to the AudioWorklet
/// 
/// This function takes a MediaStream that was already obtained (e.g., from a user gesture)
/// and connects it to the AudioWorklet for processing. This bypasses the permission request
/// since the stream is already available.
/// 
/// # Arguments
/// 
/// * `media_stream` - The MediaStream to connect (should contain audio tracks)
/// * `audio_context` - The audio system context for managing the connection
/// 
/// # Returns
/// 
/// Returns `Result<(), String>` indicating success or failure of the connection.
pub async fn connect_existing_mediastream_to_audioworklet(
    media_stream: web_sys::MediaStream,
    audio_context: &std::cell::RefCell<super::context::AudioSystemContext>
) -> Result<(), String> {
    use crate::common::dev_log;
    
    dev_log!("Connecting existing MediaStream to AudioWorklet");
    
    // Update permission state to indicate connection attempt
    {
        let context_borrowed = audio_context.borrow();
        context_borrowed.set_permission_state(super::permission::AudioPermission::Requesting);
    }
    
    // Check if stream has active tracks
    let tracks = media_stream.get_tracks();
    dev_log!("MediaStream has {} tracks", tracks.length());
    
    if tracks.length() == 0 {
        return Err("MediaStream has no tracks".to_string());
    }
    
    for i in 0..tracks.length() {
        if let Some(track) = tracks.get(i).dyn_ref::<web_sys::MediaStreamTrack>() {
            dev_log!("Track {}: kind={}, enabled={}, ready_state={:?}", 
                i, track.kind(), track.enabled(), track.ready_state());
        }
    }
    
    // Check if AudioContext needs resume and extract resume promise
    let resume_promise = {
        let audio_system_context = audio_context.borrow();
        let audio_context_manager = audio_system_context.get_audio_context_manager();
        let audio_ctx_borrowed = audio_context_manager.borrow();
        
        if let Some(context) = audio_ctx_borrowed.get_context() {
            
            // Check if resume is needed
            if context.state() == web_sys::AudioContextState::Suspended {
                context.resume().ok()
            } else {
                None
            }
        } else {
            None
        }
    };
    
    // Await resume promise if needed (borrows are dropped here)
    if let Some(promise) = resume_promise {
        if let Ok(_) = wasm_bindgen_futures::JsFuture::from(promise).await {
        } 
    }
    
    // Create media source (new borrow after async operation)
    let source = {
        let audio_system_context = audio_context.borrow();
        let audio_context_manager = audio_system_context.get_audio_context_manager();
        let audio_ctx_borrowed = audio_context_manager.borrow();
        
        if let Some(context) = audio_ctx_borrowed.get_context() {
            
            match context.create_media_stream_source(&media_stream) {
                Ok(source) => {
                    dev_log!("✓ Created MediaStreamAudioSourceNode from existing stream");
                    source
                }
                Err(e) => {
                    dev_log!("✗ Failed to create MediaStreamAudioSourceNode: {:?}", e);
                    return Err(format!("Failed to create audio source: {:?}", e));
                }
            }
        } else {
            dev_log!("✗ No audio context available");
            return Err("Audio context not available".to_string());
        }
    };
    
    dev_log!("DEBUG: Connecting existing microphone source to AudioWorklet");
    
    // Connect to AudioWorklet manager
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
            dev_log!("✓ Existing MediaStream successfully connected to AudioWorklet");
            
            // Update permission state to Granted since we successfully connected
            {
                let context_borrowed = audio_context.borrow();
                context_borrowed.set_permission_state(super::permission::AudioPermission::Granted);
            }
            
            // Ensure processing is active after connection
            {
                let mut context_borrowed = audio_context.borrow_mut();
                if let Some(ref mut worklet_manager) = context_borrowed.get_audioworklet_manager_mut() {
                    if !worklet_manager.is_processing() && worklet_manager.start_processing().is_err() {
                        // Processing start failed - AudioWorklet will handle error reporting
                    }
                    
                } else {
                    dev_log!("No AudioWorklet manager available");
                }
            } // Drop mutable borrow here
            
            // Refresh audio devices now that permission is granted
            {
                let context_borrowed = audio_context.borrow();
                let manager_rc = context_borrowed.get_audio_context_manager_rc();
                drop(context_borrowed);

                wasm_bindgen_futures::spawn_local(async move {
                    match manager_rc.try_borrow_mut() {
                        Ok(mut manager) => {
                            if let Err(_) = manager.refresh_audio_devices().await {
                                // Handle error if needed
                            }
                        }
                        Err(_) => {
                            // AudioContextManager busy, skip refresh
                        }
                    }
                });
            }
            
            Ok(())
        }
        Err(e) => {
            dev_log!("✗ Failed to connect existing MediaStream to AudioWorklet: {:?}", e);
            
            // Update permission state to indicate connection failure
            {
                let context_borrowed = audio_context.borrow();
                context_borrowed.set_permission_state(super::permission::AudioPermission::Unavailable);
            }
            
            Err(format!("Failed to connect microphone: {:?}", e))
        }
    }
}

