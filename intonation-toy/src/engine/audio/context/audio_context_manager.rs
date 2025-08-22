use web_sys::{AudioContext, AudioContextOptions};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use crate::common::dev_log;
use super::super::AudioError;
use super::{AudioContextState, AudioDevices};
use crate::app_config::STANDARD_SAMPLE_RATE;

pub struct AudioContextManager {
    context: Option<AudioContext>,
    state: AudioContextState,
    recreation_attempts: u32,
    cached_devices: Option<AudioDevices>,
    device_change_callback: Option<Closure<dyn FnMut(web_sys::Event)>>,
}


impl AudioContextManager {
    pub fn new() -> Self {
        Self {
            context: None,
            state: AudioContextState::Uninitialized,
            recreation_attempts: 0,
            cached_devices: None,
            device_change_callback: None,
        }
    }
    
    
    pub fn state(&self) -> &AudioContextState {
        &self.state
    }
    
    pub fn is_supported() -> bool {
        let Some(window) = web_sys::window() else { return false; };
        js_sys::Reflect::has(&window, &"AudioContext".into()).unwrap_or(false) ||
        js_sys::Reflect::has(&window, &"webkitAudioContext".into()).unwrap_or(false)
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
        self.recreation_attempts = 0;
        Ok(())
    }
    
    pub async fn resume(&mut self) -> Result<(), AudioError> {
        let context = self.context.as_ref()
            .ok_or_else(|| AudioError::Generic("No AudioContext available".to_string()))?;
            
        if context.state() != web_sys::AudioContextState::Suspended {
            return Ok(());
        }
        
        dev_log!("Resuming suspended AudioContext");
        let _ = context.resume()
            .map_err(|e| {
                dev_log!("✗ Failed to resume AudioContext: {:?}", e);
                AudioError::Generic(format!("Failed to resume AudioContext: {:?}", e))
            })?;
            
        self.state = AudioContextState::Running;
        dev_log!("✓ AudioContext resume initiated");
        Ok(())
    }
    
    
    pub async fn close(&mut self) -> Result<(), AudioError> {
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
        self.context.as_ref()
            .map(|context| matches!(self.state, AudioContextState::Running) && 
                          context.state() == web_sys::AudioContextState::Running)
            .unwrap_or(false)
    }
    
    
    
    async fn enumerate_devices_internal() -> Result<(Vec<(String, String)>, Vec<(String, String)>), AudioError> {
        let window = web_sys::window()
            .ok_or_else(|| AudioError::Generic("No window object".to_string()))?;
        
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
            .map(|d| !d.label().is_empty())
            .unwrap_or(false);

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

    pub async fn refresh_audio_devices(&mut self) -> Result<(), AudioError> {
        let (input_devices, output_devices) = Self::enumerate_devices_internal().await?;
        self.cached_devices = Some(AudioDevices { input_devices, output_devices });
        Ok(())
    }

    pub fn get_cached_devices(&self) -> &AudioDevices {
        static EMPTY_DEVICES: AudioDevices = AudioDevices {
            input_devices: Vec::new(),
            output_devices: Vec::new(),
        };
        self.cached_devices.as_ref().unwrap_or(&EMPTY_DEVICES)
    }

    pub fn setup_device_change_listener<F>(&mut self, callback: F) -> Result<(), AudioError>
    where
        F: Fn() + 'static,
    {
        if self.device_change_callback.is_some() {
            return Ok(());
        }
        
        let window = web_sys::window()
            .ok_or_else(|| AudioError::Generic("No window available for device change listener".to_string()))?;
        
        let media_devices = window.navigator().media_devices()
            .map_err(|_| AudioError::NotSupported("MediaDevices not available for device change listener".to_string()))?;
        
        let device_change_closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            dev_log!("Audio devices changed - triggering callback");
            callback();
        }) as Box<dyn FnMut(_)>);
        
        media_devices.add_event_listener_with_callback(
            "devicechange", 
            device_change_closure.as_ref().unchecked_ref()
        ).map_err(|e| AudioError::Generic(format!("Failed to add device change listener: {:?}", e)))?;
        
        dev_log!("Device change listener set up successfully");
        self.device_change_callback = Some(device_change_closure);
        Ok(())
    }
    
    pub fn remove_device_change_listener(&mut self) -> Result<(), AudioError> {
        if let Some(callback) = &self.device_change_callback {
            let window = web_sys::window()
                .ok_or_else(|| AudioError::Generic("No window available".to_string()))?;
            
            let media_devices = window.navigator().media_devices()
                .map_err(|_| AudioError::NotSupported("MediaDevices not available".to_string()))?;
            
            media_devices.remove_event_listener_with_callback(
                "devicechange",
                callback.as_ref().unchecked_ref()
            ).map_err(|e| AudioError::Generic(format!("Failed to remove device change listener: {:?}", e)))?;
            
            dev_log!("Device change listener removed");
        }
        
        self.device_change_callback = None;
        Ok(())
    }
    
}

impl Drop for AudioContextManager {
    fn drop(&mut self) {
        let _ = self.remove_device_change_listener();
        if let Some(context) = &self.context {
            let _ = context.close();
        }
    }
}