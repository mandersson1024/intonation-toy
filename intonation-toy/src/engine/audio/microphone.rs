use wasm_bindgen::prelude::*;
use std::fmt;
use super::permission::PermissionManager;


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
pub struct MicrophoneManager;

impl MicrophoneManager {
    /// Check if getUserMedia API is supported
    pub fn is_supported() -> bool {
        PermissionManager::is_supported()
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
        if wasm_bindgen_futures::JsFuture::from(promise).await.is_ok() {
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
                            if manager.refresh_audio_devices().await.is_err() {
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

