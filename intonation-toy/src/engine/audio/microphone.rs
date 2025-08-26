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
pub async fn connect_existing_mediastream_to_audioworklet(
    media_stream: web_sys::MediaStream,
    audio_context: &std::cell::RefCell<super::context::AudioSystemContext>
) -> Result<(), String> {
    audio_context.borrow().set_permission_state(super::permission::AudioPermission::Requesting);
    
    let tracks = media_stream.get_tracks();
    if tracks.length() == 0 {
        return Err("MediaStream has no tracks".to_string());
    }
    
    let resume_promise = {
        let audio_system_context = audio_context.borrow();
        let audio_context_manager = audio_system_context.get_audio_context_manager();
        let audio_ctx_borrowed = audio_context_manager.borrow();
        
        audio_ctx_borrowed.get_context()
            .filter(|ctx| ctx.state() == web_sys::AudioContextState::Suspended)
            .and_then(|ctx| ctx.resume().ok())
    };
    
    let source = {
        let audio_system_context = audio_context.borrow();
        let audio_context_manager = audio_system_context.get_audio_context_manager();
        let audio_ctx_borrowed = audio_context_manager.borrow();
        
        let context = audio_ctx_borrowed.get_context()
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
            
            {
                let context_borrowed = audio_context.borrow();
                let manager_rc = context_borrowed.get_audio_context_manager_rc();
                drop(context_borrowed);

                wasm_bindgen_futures::spawn_local(async move {
                    // Call the async function without holding any borrow
                    match super::context::AudioContextManager::enumerate_devices_internal().await {
                        Ok((input_devices, output_devices)) => {
                            // Now borrow to store the result
                            if let Ok(mut manager) = manager_rc.try_borrow_mut() {
                                let devices = super::context::AudioDevices { input_devices, output_devices };
                                manager.set_cached_devices(devices);
                            }
                        }
                        Err(_) => {
                            // Device enumeration failed, ignore
                        }
                    }
                });
            }
            
            Ok(())
        }
        Err(e) => {
            audio_context.borrow().set_permission_state(super::permission::AudioPermission::Unavailable);
            Err(format!("Failed to connect microphone: {:?}", e))
        }
    }
}

