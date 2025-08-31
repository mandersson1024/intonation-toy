use std::fmt;

#[derive(Debug, Clone)]
pub enum AudioError {
    PermissionDenied(String),
    DeviceUnavailable(String),
    NotSupported(String),
    StreamInitFailed(String),
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
pub fn connect_mediastream_to_audioworklet(
    media_stream: web_sys::MediaStream,
    audio_context: &std::cell::RefCell<super::context::AudioSystemContext>
) -> Result<(), String> {
    audio_context.borrow().set_permission_state(super::permission::AudioPermission::Requesting);
    
    let tracks = media_stream.get_tracks();
    if tracks.length() == 0 {
        return Err("MediaStream has no tracks".to_string());
    }
    
    let source = {
        let audio_system_context = audio_context.borrow();
        
        let context = audio_system_context.get_audio_context()
            .ok_or("Audio context not available")?;
            
        context.create_media_stream_source(&media_stream)
            .map_err(|e| format!("Failed to create audio source: {:?}", e))?
    };
    
    let result = {
        let mut context_borrowed = audio_context.borrow_mut();
        context_borrowed.get_audioworklet_manager_mut()
            .ok_or("AudioWorklet manager not available".to_string())
            .and_then(|worklet_manager| worklet_manager.connect_microphone(source.as_ref(), false).map_err(|e| e.to_string()))
    };
    
    match result {
        Ok(_) => {
            audio_context.borrow().set_permission_state(super::permission::AudioPermission::Granted);
            
            {
                let mut context_borrowed = audio_context.borrow_mut();
                if let Some(ref mut worklet_manager) = context_borrowed.get_audioworklet_manager_mut() {
                    if !worklet_manager.is_processing() {
                        let _ = worklet_manager.start_processing();
                    }
                }
            }
            
            
            Ok(())
        }
        Err(e) => {
            audio_context.borrow().set_permission_state(super::permission::AudioPermission::Unavailable);
            Err(format!("Failed to connect microphone: {:?}", e))
        }
    }
}

