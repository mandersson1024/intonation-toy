use web_sys::{AudioContext, AudioContextOptions};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsCast;
use crate::common::dev_log;
use super::super::AudioError;
use super::AudioContextState;
use crate::app_config::STANDARD_SAMPLE_RATE;

pub struct AudioContextManager {
    context: Option<AudioContext>,
    state: AudioContextState,
}


impl Default for AudioContextManager {
    fn default() -> Self {
        Self {
            context: None,
            state: AudioContextState::Uninitialized,
        }
    }
}

impl AudioContextManager {
    pub fn state(&self) -> &AudioContextState {
        &self.state
    }
    
    pub fn is_supported() -> bool {
        web_sys::window().is_some_and(|window| {
            js_sys::Reflect::has(&window, &"AudioContext".into()).unwrap_or(false) ||
            js_sys::Reflect::has(&window, &"webkitAudioContext".into()).unwrap_or(false)
        })
    }
    
    pub async fn initialize(&mut self) -> Result<(), AudioError> {
        if !Self::is_supported() {
            return Err(AudioError::NotSupported("Web Audio API not supported".to_string()));
        }
        
        self.state = AudioContextState::Initializing;
        dev_log!("Initializing AudioContext with sample rate: {}Hz", STANDARD_SAMPLE_RATE);
        
        let options = AudioContextOptions::new();
        options.set_sample_rate(STANDARD_SAMPLE_RATE as f32);
        
        let context = AudioContext::new_with_context_options(&options)
            .map_err(|e| {
                dev_log!("✗ Failed to create AudioContext: {:?}", e);
                self.state = AudioContextState::Closed;
                AudioError::StreamInitFailed(format!("Failed to create AudioContext: {:?}", e))
            })?;
            
        dev_log!("✓ AudioContext created successfully");
        self.context = Some(context);
        self.state = AudioContextState::Running;
        Ok(())
    }
    
    pub fn close(&mut self) -> Result<(), AudioError> {
        if let Some(context) = &self.context {
            dev_log!("Closing AudioContext");
            let _ = context.close();
        }
        
        self.context = None;
        self.state = AudioContextState::Closed;
        Ok(())
    }
    
    
    pub fn get_context(&self) -> Option<&AudioContext> {
        self.context.as_ref()
    }
    
    
    pub fn is_running(&self) -> bool {
        matches!(self.state, AudioContextState::Running) &&
        self.context.as_ref()
            .is_some_and(|ctx| ctx.state() == web_sys::AudioContextState::Running)
    }
    
    
    
    pub async fn enumerate_devices_internal() -> Result<(Vec<(String, String)>, Vec<(String, String)>), AudioError> {
        let window = web_sys::window()
            .ok_or(AudioError::Generic("No window object".to_string()))?;
        
        let media_devices = window.navigator().media_devices()
            .map_err(|_| AudioError::NotSupported("MediaDevices not available".to_string()))?;

        let promise = media_devices.enumerate_devices()
            .map_err(|e| AudioError::Generic(format!("Failed to enumerate devices: {:?}", e)))?;

        let devices_js = JsFuture::from(promise).await
            .map_err(|e| AudioError::Generic(format!("Device enumeration failed: {:?}", e)))?;
            
        let devices = js_sys::Array::from(&devices_js);
        let mut input_devices = Vec::new();
        let mut output_devices = Vec::new();

        let has_permission = devices.get(0)
            .dyn_ref::<web_sys::MediaDeviceInfo>()
            .is_some_and(|d| !d.label().is_empty());

        if !has_permission {
            return Ok((input_devices, output_devices));
        }

        for i in 0..devices.length() {
            if let Some(device_info) = devices.get(i).dyn_ref::<web_sys::MediaDeviceInfo>() {
                let device_id = device_info.device_id();
                let label = device_info.label();

                match device_info.kind() {
                    web_sys::MediaDeviceKind::Audioinput => input_devices.push((device_id, label)),
                    web_sys::MediaDeviceKind::Audiooutput => output_devices.push((device_id, label)),
                    _ => {}
                }
            }
        }

        Ok((input_devices, output_devices))
    }

    


}

