use crate::common::dev_log;
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
            audio_context_manager: std::rc::Rc::new(std::cell::RefCell::new(AudioContextManager::new())),
            audioworklet_manager: None,
            pitch_analyzer: None,
            is_initialized: false,
            initialization_error: None,
            permission_state: std::cell::Cell::new(super::super::AudioPermission::Uninitialized),
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
        let mut worklet_manager = super::super::worklet::AudioWorkletManager::new_return_based();
        
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
        let config = super::super::pitch_detector::PitchDetectorConfig::default();
        let sample_rate = crate::app_config::STANDARD_SAMPLE_RATE;
        
        match super::super::pitch_analyzer::PitchAnalyzer::new(config, sample_rate) {
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
        let volume_detector = super::super::volume_detector::VolumeDetector::new_default();
        
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
        super::super::set_global_audio_context_manager(self.audio_context_manager.clone());
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

    pub fn get_audioworklet_manager(&self) -> Option<&super::super::worklet::AudioWorkletManager> {
        self.audioworklet_manager.as_ref()
    }
    
    pub fn get_audioworklet_status(&self) -> Option<super::super::data_types::AudioWorkletStatus> {
        self.audioworklet_manager.as_ref().map(|worklet| worklet.get_status())
    }
    
    pub fn get_audio_devices(&self) -> super::AudioDevices {
        self.audio_context_manager.try_borrow()
            .map(|borrowed| borrowed.get_cached_devices().clone())
            .unwrap_or_default()
    }
    
    pub fn get_buffer_pool_stats(&self) -> Option<super::super::message_protocol::BufferPoolStats> {
        self.audioworklet_manager.as_ref().and_then(|worklet| worklet.get_buffer_pool_statistics())
    }

    pub fn get_audioworklet_manager_mut(&mut self) -> Option<&mut super::super::worklet::AudioWorkletManager> {
        self.audioworklet_manager.as_mut()
    }

    pub fn get_pitch_analyzer(&self) -> Option<&std::rc::Rc<std::cell::RefCell<super::super::pitch_analyzer::PitchAnalyzer>>> {
        self.pitch_analyzer.as_ref()
    }

    pub fn get_pitch_analyzer_clone(&self) -> Option<std::rc::Rc<std::cell::RefCell<super::super::pitch_analyzer::PitchAnalyzer>>> {
        self.pitch_analyzer.as_ref().cloned()
    }



    

    pub fn collect_audio_analysis(&self, timestamp: f64) -> Option<crate::common::shared_types::AudioAnalysis> {
        if !self.is_initialized {
            return None;
        }

        let volume_data = self.audioworklet_manager.as_ref().and_then(|w| w.get_volume_data());
        let volume = super::convert_volume_data(volume_data);
        
        let pitch_data = self.pitch_analyzer.as_ref()
            .and_then(|analyzer| analyzer.try_borrow().ok())
            .and_then(|borrowed| borrowed.get_latest_pitch_data());
        let pitch = super::convert_pitch_data(pitch_data);
        
        super::merge_audio_analysis(volume, pitch, timestamp)
    }

    pub fn collect_audio_errors(&self) -> Vec<crate::common::shared_types::Error> {
        let mut errors = Vec::new();
        
        if let Some(error_msg) = &self.initialization_error {
            errors.push(crate::common::shared_types::Error::ProcessingError(error_msg.clone()));
        }
        
        if let Ok(context_manager) = self.audio_context_manager.try_borrow() {
            if !context_manager.is_running() {
                match context_manager.state() {
                    AudioContextState::Closed => errors.push(crate::common::shared_types::Error::ProcessingError("AudioContext is closed".to_string())),
                    AudioContextState::Suspended => errors.push(crate::common::shared_types::Error::ProcessingError("AudioContext is suspended".to_string())),
                    _ => {}
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
        if let Some(ref mut worklet) = self.audioworklet_manager {
            worklet.update_tuning_fork_config(config);
        }
    }
    
    pub fn set_permission_state(&self, state: super::super::AudioPermission) {
        self.permission_state.set(state);
    }
    
}