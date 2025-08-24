use crate::common::{dev_log, shared_types::{Volume, Pitch, AudioAnalysis}};
use super::{AudioContextManager, AudioContextState};

pub struct AudioSystemContext {
    audio_context_manager: std::rc::Rc<std::cell::RefCell<AudioContextManager>>,
    audioworklet_manager: Option<super::super::worklet::AudioWorkletManager>,
    pitch_analyzer: Option<std::rc::Rc<std::cell::RefCell<super::super::pitch_analyzer::PitchAnalyzer>>>,
    is_initialized: bool,
    initialization_error: Option<String>,
    permission_state: std::cell::Cell<super::super::AudioPermission>,
}

impl AudioSystemContext {

    pub fn new_return_based() -> Self {
        Self {
            audio_context_manager: std::rc::Rc::new(std::cell::RefCell::new(AudioContextManager::default())),
            audioworklet_manager: None,
            pitch_analyzer: None,
            is_initialized: false,
            initialization_error: None,
            permission_state: std::cell::Cell::new(super::super::AudioPermission::Uninitialized),
        }
    }

    pub async fn initialize(&mut self) -> Result<(), String> {
        self.initialization_error = None;
        
        self.audio_context_manager.borrow_mut().initialize().await
            .map_err(|e| {
                let error_msg = format!("Failed to initialize AudioContextManager: {}", e);
                dev_log!("✗ {}", error_msg);
                self.initialization_error = Some(error_msg.clone());
                error_msg
            })?;
        dev_log!("✓ AudioContextManager initialized");

        let mut worklet_manager = super::super::worklet::AudioWorkletManager::new_return_based();
        worklet_manager.initialize(&self.audio_context_manager.borrow()).await
            .map_err(|e| {
                let error_msg = format!("Failed to initialize AudioWorkletManager: {:?}", e);
                dev_log!("✗ {}", error_msg);
                self.initialization_error = Some(error_msg.clone());
                error_msg
            })?;
        
        self.audioworklet_manager = Some(worklet_manager);
        dev_log!("✓ AudioWorkletManager initialized for return-based pattern");

        let config = super::super::pitch_detector::PitchDetectorConfig::default();
        let sample_rate = crate::app_config::STANDARD_SAMPLE_RATE;
        
        let analyzer = super::super::pitch_analyzer::PitchAnalyzer::new(config, sample_rate)
            .map_err(|e| {
                let error_msg = format!("Failed to initialize PitchAnalyzer: {}", e);
                dev_log!("✗ {}", error_msg);
                self.initialization_error = Some(error_msg.clone());
                error_msg
            })?;
        
        let analyzer_rc = std::rc::Rc::new(std::cell::RefCell::new(analyzer));
        self.pitch_analyzer = Some(analyzer_rc.clone());
        
        if let Some(ref mut worklet_manager) = self.audioworklet_manager {
            worklet_manager.set_pitch_analyzer(analyzer_rc);
            dev_log!("✓ PitchAnalyzer connected to AudioWorkletManager");
        }
        dev_log!("✓ PitchAnalyzer initialized for return-based pattern");

        let audio_context = {
            let manager = self.audio_context_manager.borrow();
            manager.get_context()
                .cloned()
                .ok_or("Audio context not available for AnalyserVolumeDetector".to_string())?
        };
        
        let volume_detector = super::super::analyser_volume_detector::AnalyserVolumeDetector::new(&audio_context)
            .map_err(|e| format!("Failed to create AnalyserVolumeDetector: {:?}", e))?;
        
        if let Some(ref mut worklet_manager) = self.audioworklet_manager {
            worklet_manager.set_volume_detector(volume_detector);
            worklet_manager.setup_message_handling()
                .map_err(|e| {
                    let error_msg = format!("Failed to setup message handling: {:?}", e);
                    dev_log!("✗ {}", error_msg);
                    self.initialization_error = Some(error_msg.clone());
                    error_msg
                })?;
        }
        
        dev_log!("✓ AnalyserVolumeDetector initialized and configured");

        super::super::set_global_audio_context_manager(self.audio_context_manager.clone());
        dev_log!("✓ AudioContextManager stored globally for device change callbacks");

        match AudioContextManager::enumerate_devices_internal().await {
            Ok((input_devices, output_devices)) => {
                let mut manager = self.audio_context_manager.borrow_mut();
                let devices = super::AudioDevices { input_devices, output_devices };
                manager.set_cached_devices(devices);
                dev_log!("✓ Initial device refresh completed - device cache populated");
            }
            Err(e) => {
                dev_log!("Initial device refresh failed: {:?}", e);
            }
        }

        {
            let manager_rc = self.audio_context_manager.clone();
            let callback = move || {
                dev_log!("Device change detected in AudioSystemContext - refreshing device list");
                
                let manager_rc_async = manager_rc.clone();
                
                wasm_bindgen_futures::spawn_local(async move {
                    match AudioContextManager::enumerate_devices_internal().await {
                        Ok((input_devices, output_devices)) => {
                            match manager_rc_async.try_borrow_mut() {
                                Ok(mut manager) => {
                                    let devices = super::AudioDevices { input_devices, output_devices };
                                    manager.set_cached_devices(devices);
                                    dev_log!("AudioSystemContext auto device refresh completed successfully");
                                }
                                Err(_) => {
                                    dev_log!("AudioContextManager busy during AudioSystemContext auto device refresh");
                                }
                            }
                        }
                        Err(e) => {
                            dev_log!("AudioSystemContext auto device refresh failed: {:?}", e);
                        }
                    }
                });
            };
            
            match self.audio_context_manager.try_borrow_mut() {
                Ok(mut manager) => {
                    match manager.setup_device_change_listener(callback) {
                        Ok(_) => {
                            dev_log!("✓ AudioSystemContext device change listener set up successfully");
                        }
                        Err(e) => {
                            dev_log!("Failed to set up AudioSystemContext device change listener: {:?}", e);
                        }
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
        
        let _ = self.audio_context_manager.borrow_mut().close();
        
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
    
    pub fn get_audioworklet_status(&self) -> Option<super::super::data_types::AudioWorkletStatus> {
        self.audioworklet_manager.as_ref().map(|worklet| worklet.get_status())
    }
    
    pub fn get_buffer_pool_stats(&self) -> Option<super::super::message_protocol::BufferPoolStats> {
        self.audioworklet_manager.as_ref().and_then(|worklet| worklet.get_buffer_pool_statistics())
    }
    

    pub fn get_audioworklet_manager(&self) -> Option<&super::super::worklet::AudioWorkletManager> {
        self.audioworklet_manager.as_ref()
    }
    
    
    pub fn get_audio_devices(&self) -> super::AudioDevices {
        self.audio_context_manager.try_borrow()
            .map(|borrowed| borrowed.get_cached_devices().clone())
            .unwrap_or_default()
    }
    

    pub fn get_audioworklet_manager_mut(&mut self) -> Option<&mut super::super::worklet::AudioWorkletManager> {
        self.audioworklet_manager.as_mut()
    }

    pub fn get_pitch_analyzer(&self) -> Option<&std::rc::Rc<std::cell::RefCell<super::super::pitch_analyzer::PitchAnalyzer>>> {
        self.pitch_analyzer.as_ref()
    }


    pub fn collect_audio_analysis(&self, timestamp: f64) -> Option<crate::common::shared_types::AudioAnalysis> {
        if !self.is_initialized {
            return None;
        }

        let volume_data = self.audioworklet_manager.as_ref().and_then(|w| w.get_volume_data());
        let volume = volume_data.as_ref().map(|data| Volume {
            peak_amplitude: data.peak_amplitude,
            rms_amplitude: data.rms_amplitude,
        });
        
        // Extract FFT data from volume data when available
        let fft_data = volume_data.and_then(|data| data.fft_data.clone());
        
        let pitch_data = self.pitch_analyzer.as_ref()
            .and_then(|analyzer| analyzer.try_borrow().ok())
            .and_then(|borrowed| borrowed.get_latest_pitch_data());
        let pitch = pitch_data.map(|data| {
            if data.frequency > 0.0 {
                Pitch::Detected(data.frequency, data.clarity)
            } else {
                Pitch::NotDetected
            }
        });
        
        (volume.is_some() || pitch.is_some()).then(|| AudioAnalysis {
            volume_level: volume.unwrap_or(Volume { peak_amplitude: 0.0, rms_amplitude: 0.0 }),
            pitch: pitch.unwrap_or(Pitch::NotDetected),
            fft_data,
            timestamp: timestamp.max(js_sys::Date::now()),
        })
    }

    pub fn collect_audio_errors(&self) -> Vec<crate::common::shared_types::Error> {
        let mut errors = Vec::new();
        
        if let Some(error_msg) = &self.initialization_error {
            errors.push(crate::common::shared_types::Error::ProcessingError(error_msg.clone()));
        }
        
        if let Ok(context_manager) = self.audio_context_manager.try_borrow() {
            if !context_manager.is_running() {
                let error_msg = match context_manager.state() {
                    AudioContextState::Closed => Some("AudioContext is closed"),
                    AudioContextState::Suspended => Some("AudioContext is suspended"),
                    _ => None,
                };
                if let Some(msg) = error_msg {
                    errors.push(crate::common::shared_types::Error::ProcessingError(msg.to_string()));
                }
            }
        }
        
        errors
    }

    pub fn collect_permission_state(&self) -> crate::common::shared_types::PermissionState {
        match self.permission_state.get() {
            super::super::AudioPermission::Uninitialized => crate::common::shared_types::PermissionState::NotRequested,
            super::super::AudioPermission::Requesting => crate::common::shared_types::PermissionState::Requested,
            super::super::AudioPermission::Granted => crate::common::shared_types::PermissionState::Granted,
            super::super::AudioPermission::Denied => crate::common::shared_types::PermissionState::Denied,
            super::super::AudioPermission::Unavailable => crate::common::shared_types::PermissionState::Denied,
        }
    }
    
    pub fn configure_tuning_fork(&mut self, config: super::super::TuningForkConfig) {
        if let Some(worklet) = &mut self.audioworklet_manager {
            worklet.update_tuning_fork_config(config);
        }
    }
    
    pub fn set_permission_state(&self, state: super::super::AudioPermission) {
        self.permission_state.set(state);
    }
    
}