//! AudioContext management for real-time audio processing

use web_sys::{AudioContext, AudioContextOptions};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use std::fmt;
use crate::common::dev_log;
use super::AudioError;
use crate::app_config::STANDARD_SAMPLE_RATE;

#[derive(Debug, Clone, PartialEq)]
pub enum AudioContextState {
    Uninitialized,
    Initializing,
    Running,
    Suspended,
    Closed,
    Recreating,
}

impl fmt::Display for AudioContextState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioContextState::Uninitialized => write!(f, "Uninitialized"),
            AudioContextState::Initializing => write!(f, "Initializing"),
            AudioContextState::Running => write!(f, "Running"),
            AudioContextState::Suspended => write!(f, "Suspended"),
            AudioContextState::Closed => write!(f, "Closed"),
            AudioContextState::Recreating => write!(f, "Recreating"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AudioContextConfig {
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub max_recreation_attempts: u32,
}

impl Default for AudioContextConfig {
    fn default() -> Self {
        Self {
            sample_rate: STANDARD_SAMPLE_RATE,
            buffer_size: 1024,    // Production buffer size
            max_recreation_attempts: 3,
        }
    }
}

impl AudioContextConfig {
    pub fn with_buffer_size(mut self, buffer_size: u32) -> Self {
        self.buffer_size = buffer_size;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct AudioDevices {
    pub input_devices: Vec<(String, String)>,
    pub output_devices: Vec<(String, String)>,
}

pub struct AudioContextManager {
    context: Option<AudioContext>,
    state: AudioContextState,
    config: AudioContextConfig,
    recreation_attempts: u32,
    cached_devices: Option<AudioDevices>,
    device_change_callback: Option<Closure<dyn FnMut(web_sys::Event)>>,
}


impl AudioContextManager {
    pub fn new() -> Self {
        Self {
            context: None,
            state: AudioContextState::Uninitialized,
            config: AudioContextConfig::default(),
            recreation_attempts: 0,
            cached_devices: None,
            device_change_callback: None,
        }
    }
    
    
    pub fn state(&self) -> &AudioContextState {
        &self.state
    }
    
    pub fn config(&self) -> &AudioContextConfig {
        &self.config
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
        dev_log!("Initializing AudioContext with sample rate: {}Hz", self.config.sample_rate);
        
        let options = AudioContextOptions::new();
        options.set_sample_rate(self.config.sample_rate as f32);
        
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

pub struct AudioSystemContext {
    audio_context_manager: std::rc::Rc<std::cell::RefCell<AudioContextManager>>,
    audioworklet_manager: Option<super::worklet::AudioWorkletManager>,
    pitch_analyzer: Option<std::rc::Rc<std::cell::RefCell<super::pitch_analyzer::PitchAnalyzer>>>,
    is_initialized: bool,
    initialization_error: Option<String>,
    permission_state: std::cell::Cell<super::AudioPermission>,
}

impl AudioSystemContext {

    pub fn new_return_based() -> Self {
        Self {
            audio_context_manager: std::rc::Rc::new(std::cell::RefCell::new(AudioContextManager::new())),
            audioworklet_manager: None,
            pitch_analyzer: None,
            is_initialized: false,
            initialization_error: None,
            permission_state: std::cell::Cell::new(super::AudioPermission::Uninitialized),
        }
    }


    pub async fn initialize(&mut self) -> Result<(), String> {
        use crate::common::dev_log;
        
        self.initialization_error = None;
        
        // Step 1: Initialize AudioContextManager
        let init_result = {
            let mut manager = self.audio_context_manager.borrow_mut();
            manager.initialize().await
        };
        if let Err(e) = init_result {
            let error_msg = format!("Failed to initialize AudioContextManager: {}", e);
            dev_log!("✗ {}", error_msg);
            self.initialization_error = Some(error_msg.clone());
            return Err(error_msg);
        }
        dev_log!("✓ AudioContextManager initialized");

        // Step 2: Initialize AudioWorkletManager (simplified for return-based pattern)
        let mut worklet_manager = super::worklet::AudioWorkletManager::new_return_based();
        
        // Initialize the worklet with the audio context
        let worklet_init_result = {
            let manager = self.audio_context_manager.borrow();
            worklet_manager.initialize(&manager).await
        };
        if let Err(e) = worklet_init_result {
            let error_msg = format!("Failed to initialize AudioWorkletManager: {:?}", e);
            dev_log!("✗ {}", error_msg);
            self.initialization_error = Some(error_msg.clone());
            return Err(error_msg);
        }
        
        self.audioworklet_manager = Some(worklet_manager);
        dev_log!("✓ AudioWorkletManager initialized for return-based pattern");

        // Step 3: Initialize PitchAnalyzer (simplified for return-based pattern)
        let config = super::pitch_detector::PitchDetectorConfig::default();
        let sample_rate = {
            let borrowed = self.audio_context_manager.borrow();
            borrowed.config().sample_rate
        };
        
        match super::pitch_analyzer::PitchAnalyzer::new(config, sample_rate) {
            Ok(analyzer) => {
                // Create analyzer without setter (return-based pattern)
                let analyzer_rc = std::rc::Rc::new(std::cell::RefCell::new(analyzer));
                self.pitch_analyzer = Some(analyzer_rc.clone());
                
                // Connect the pitch analyzer to the AudioWorkletManager so it receives audio data
                if let Some(ref mut worklet_manager) = self.audioworklet_manager {
                    worklet_manager.set_pitch_analyzer(analyzer_rc);
                    dev_log!("✓ PitchAnalyzer connected to AudioWorkletManager");
                }
                
                dev_log!("✓ PitchAnalyzer initialized for return-based pattern");
            }
            Err(e) => {
                let error_msg = format!("Failed to initialize PitchAnalyzer: {}", e);
                dev_log!("✗ {}", error_msg);
                self.initialization_error = Some(error_msg.clone());
                return Err(error_msg);
            }
        }

        // Step 4: Initialize VolumeDetector
        let volume_detector = super::volume_detector::VolumeDetector::new_default();
        
        // Configure VolumeDetector in AudioWorkletManager
        if let Some(ref mut worklet_manager) = self.audioworklet_manager {
            worklet_manager.set_volume_detector(volume_detector);
            
            // Setup message handling now that volume detector is configured
            if let Err(e) = worklet_manager.setup_message_handling() {
                let error_msg = format!("Failed to setup message handling: {:?}", e);
                dev_log!("✗ {}", error_msg);
                self.initialization_error = Some(error_msg.clone());
                return Err(error_msg);
            }
        }
        
        dev_log!("✓ VolumeDetector initialized and configured");

        // Step 5: Store AudioContextManager globally for device change callbacks
        super::set_global_audio_context_manager(self.audio_context_manager.clone());
        dev_log!("✓ AudioContextManager stored globally for device change callbacks");

        // Step 6: Perform initial device refresh to populate the cache
        {
            let mut manager = self.audio_context_manager.borrow_mut();
            if let Err(_e) = manager.refresh_audio_devices().await {
                dev_log!("Initial device refresh failed: {:?}", _e);
            } else {
                dev_log!("✓ Initial device refresh completed - device cache populated");
            }
        }

        // Step 7: Set up device change listener to automatically refresh device cache
        {
            let manager_rc = self.audio_context_manager.clone();
            let callback = move || {
                dev_log!("Device change detected in AudioSystemContext - refreshing device list");
                
                // Clone for the async closure
                let manager_rc_async = manager_rc.clone();
                
                // Spawn async task to refresh devices
                wasm_bindgen_futures::spawn_local(async move {
                    match manager_rc_async.try_borrow_mut() {
                        Ok(mut manager) => {
                            if let Err(_e) = manager.refresh_audio_devices().await {
                                dev_log!("AudioSystemContext auto device refresh failed: {:?}", _e);
                            } else {
                                dev_log!("AudioSystemContext auto device refresh completed successfully");
                            }
                        }
                        Err(_) => {
                            dev_log!("AudioContextManager busy during AudioSystemContext auto device refresh");
                        }
                    }
                });
            };
            
            // Set up the listener in the AudioContextManager
            match self.audio_context_manager.try_borrow_mut() {
                Ok(mut manager) => {
                    if let Err(_e) = manager.setup_device_change_listener(callback) {
                        dev_log!("Failed to set up AudioSystemContext device change listener: {:?}", _e);
                    } else {
                        dev_log!("✓ AudioSystemContext device change listener set up successfully");
                    }
                }
                Err(_) => {
                    dev_log!("AudioContextManager busy, cannot set up AudioSystemContext device change listener");
                }
            }
        }

        self.is_initialized = true;
        dev_log!("✓ AudioSystemContext fully initialized");
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<(), String> {
        dev_log!("Shutting down AudioSystemContext");
        
        if let Some(ref mut worklet_manager) = self.audioworklet_manager {
            let _ = worklet_manager.stop_processing();
            let _ = worklet_manager.disconnect();
        }
        self.audioworklet_manager = None;
        self.pitch_analyzer = None;
        
        let _ = self.audio_context_manager.borrow_mut().close().await;
        
        self.is_initialized = false;
        dev_log!("✓ AudioSystemContext shutdown completed");
        Ok(())
    }



    pub fn get_audio_context_manager(&self) -> &std::rc::Rc<std::cell::RefCell<AudioContextManager>> {
        &self.audio_context_manager
    }
    
    pub fn get_audio_context_manager_rc(&self) -> std::rc::Rc<std::cell::RefCell<AudioContextManager>> {
        self.audio_context_manager.clone()
    }

    pub fn get_audioworklet_manager(&self) -> Option<&super::worklet::AudioWorkletManager> {
        self.audioworklet_manager.as_ref()
    }
    
    pub fn get_audioworklet_status(&self) -> Option<super::data_types::AudioWorkletStatus> {
        self.audioworklet_manager.as_ref().map(|worklet| worklet.get_status())
    }
    
    pub fn get_audio_devices(&self) -> super::AudioDevices {
        self.audio_context_manager.try_borrow()
            .map(|borrowed| borrowed.get_cached_devices().clone())
            .unwrap_or_default()
    }
    
    pub fn get_buffer_pool_stats(&self) -> Option<super::message_protocol::BufferPoolStats> {
        self.audioworklet_manager.as_ref().and_then(|worklet| worklet.get_buffer_pool_statistics())
    }

    pub fn get_audioworklet_manager_mut(&mut self) -> Option<&mut super::worklet::AudioWorkletManager> {
        self.audioworklet_manager.as_mut()
    }

    pub fn get_pitch_analyzer(&self) -> Option<&std::rc::Rc<std::cell::RefCell<super::pitch_analyzer::PitchAnalyzer>>> {
        self.pitch_analyzer.as_ref()
    }

    pub fn get_pitch_analyzer_clone(&self) -> Option<std::rc::Rc<std::cell::RefCell<super::pitch_analyzer::PitchAnalyzer>>> {
        self.pitch_analyzer.as_ref().cloned()
    }



    

    pub fn collect_audio_analysis(&self, timestamp: f64) -> Option<crate::shared_types::AudioAnalysis> {
        if !self.is_initialized {
            return None;
        }

        let volume_data = self.audioworklet_manager.as_ref().and_then(|w| w.get_volume_data());
        let volume = convert_volume_data(volume_data);
        
        let pitch_data = self.pitch_analyzer.as_ref()
            .and_then(|analyzer| analyzer.try_borrow().ok())
            .and_then(|borrowed| borrowed.get_latest_pitch_data());
        let pitch = convert_pitch_data(pitch_data);
        
        merge_audio_analysis(volume, pitch, timestamp)
    }

    pub fn collect_audio_errors(&self) -> Vec<crate::shared_types::Error> {
        let mut errors = Vec::new();
        
        if let Some(error_msg) = &self.initialization_error {
            errors.push(crate::shared_types::Error::ProcessingError(error_msg.clone()));
        }
        
        if let Ok(context_manager) = self.audio_context_manager.try_borrow() {
            if !context_manager.is_running() {
                match context_manager.state() {
                    AudioContextState::Closed => errors.push(crate::shared_types::Error::ProcessingError("AudioContext is closed".to_string())),
                    AudioContextState::Suspended => errors.push(crate::shared_types::Error::ProcessingError("AudioContext is suspended".to_string())),
                    _ => {}
                }
            }
        }
        
        errors
    }

    pub fn collect_permission_state(&self) -> crate::shared_types::PermissionState {
        match self.permission_state.get() {
            super::AudioPermission::Uninitialized => crate::shared_types::PermissionState::NotRequested,
            super::AudioPermission::Requesting => crate::shared_types::PermissionState::Requested,
            super::AudioPermission::Granted => crate::shared_types::PermissionState::Granted,
            super::AudioPermission::Denied => crate::shared_types::PermissionState::Denied,
            super::AudioPermission::Unavailable => crate::shared_types::PermissionState::Denied,
        }
    }
    
    pub fn configure_tuning_fork(&mut self, config: super::TuningForkConfig) {
        if let Some(ref mut worklet) = self.audioworklet_manager {
            worklet.update_tuning_fork_config(config);
        }
    }
    
    pub fn set_permission_state(&self, state: super::AudioPermission) {
        self.permission_state.set(state);
    }
    
}

pub fn convert_volume_data(volume_data: Option<super::data_types::VolumeLevelData>) -> Option<crate::shared_types::Volume> {
    volume_data.map(|data| crate::shared_types::Volume {
        peak_amplitude: data.peak_amplitude,
        rms_amplitude: data.rms_amplitude,
    })
}

pub fn convert_pitch_data(pitch_data: Option<super::data_types::PitchData>) -> Option<crate::shared_types::Pitch> {
    pitch_data.map(|data| {
        if data.frequency > 0.0 {
            crate::shared_types::Pitch::Detected(data.frequency, data.clarity)
        } else {
            crate::shared_types::Pitch::NotDetected
        }
    })
}

pub fn merge_audio_analysis(
    volume: Option<crate::shared_types::Volume>,
    pitch: Option<crate::shared_types::Pitch>,
    timestamp: f64
) -> Option<crate::shared_types::AudioAnalysis> {
    (volume.is_some() || pitch.is_some()).then(|| crate::shared_types::AudioAnalysis {
        volume_level: volume.unwrap_or(crate::shared_types::Volume { peak_amplitude: -60.0, rms_amplitude: -60.0 }),
        pitch: pitch.unwrap_or(crate::shared_types::Pitch::NotDetected),
        fft_data: None,
        timestamp: timestamp.max(js_sys::Date::now()),
    })
}

