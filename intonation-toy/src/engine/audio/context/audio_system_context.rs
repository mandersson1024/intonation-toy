use crate::common::{dev_log, shared_types::{Volume, Pitch, AudioAnalysis}};
use web_sys::{AudioContext, AudioContextState};

pub struct AudioSystemContext {
    audio_context: Option<AudioContext>,
    audioworklet_manager: Option<super::super::worklet::AudioWorkletManager>,
    pitch_analyzer: Option<std::rc::Rc<std::cell::RefCell<super::super::pitch_analyzer::PitchAnalyzer>>>,
    is_initialized: bool,
    initialization_error: Option<String>,
    permission_state: std::cell::Cell<super::super::AudioPermission>,
}

impl AudioSystemContext {
    
    pub fn create(audio_context: web_sys::AudioContext) -> Result<Self, String> {
        let mut result = Self {
            audio_context: Some(audio_context.clone()),
            audioworklet_manager: None,
            pitch_analyzer: None,
            is_initialized: false,
            initialization_error: None,
            permission_state: std::cell::Cell::new(super::super::AudioPermission::Uninitialized),
        };
        dev_log!("✓ AudioContext attached");

        let mut worklet_manager = super::super::worklet::AudioWorkletManager::new_return_based();
        let _worklet_node = worklet_manager.create_worklet_node(&audio_context)
            .map_err(|e| {
                let error_msg = format!("Failed to create AudioWorkletNode: {}", e);
                dev_log!("✗ {}", error_msg);
                result.initialization_error = Some(error_msg.clone());
                error_msg
            })?;
        result.audioworklet_manager = Some(worklet_manager);
        dev_log!("✓ AudioWorkletManager created with internal node creation");

        let config = super::super::pitch_detector::PitchDetectorConfig::default();
        let sample_rate = audio_context.sample_rate() as u32;
        
        if sample_rate != crate::app_config::STANDARD_SAMPLE_RATE {
            dev_log!("⚠ Audio context sample rate ({} Hz) differs from standard rate ({} Hz)", 
                sample_rate, crate::app_config::STANDARD_SAMPLE_RATE);
        }
        
        let analyzer = super::super::pitch_analyzer::PitchAnalyzer::new(config, sample_rate)
            .map_err(|e| {
                let error_msg = format!("Failed to initialize PitchAnalyzer: {}", e);
                dev_log!("✗ {}", error_msg);
                result.initialization_error = Some(error_msg.clone());
                error_msg
            })?;
        
        let analyzer_rc = std::rc::Rc::new(std::cell::RefCell::new(analyzer));
        result.pitch_analyzer = Some(analyzer_rc.clone());
        
        if let Some(ref mut worklet_manager) = result.audioworklet_manager {
            worklet_manager.set_pitch_analyzer(analyzer_rc);
            dev_log!("✓ PitchAnalyzer connected to AudioWorkletManager");
        }
        dev_log!("✓ PitchAnalyzer initialized for return-based pattern");

        let volume_detector = super::super::volume_detector::VolumeDetector::new(&audio_context)
            .map_err(|e| format!("Failed to create VolumeDetector: {:?}", e))?;
        
        if let Some(ref mut worklet_manager) = result.audioworklet_manager {
            worklet_manager.set_volume_detector(volume_detector);
            worklet_manager.setup_message_handling()
                .map_err(|e| {
                    let error_msg = format!("Failed to setup message handling: {:?}", e);
                    dev_log!("✗ {}", error_msg);
                    result.initialization_error = Some(error_msg.clone());
                    error_msg
                })?;
        }
        
        dev_log!("✓ VolumeDetector initialized and configured");


        result.is_initialized = true;
        dev_log!("✓ AudioSystemContext fully initialized");
        Ok(result)
    }


    pub async fn shutdown(&mut self) -> Result<(), String> {
        dev_log!("Shutting down AudioSystemContext");
        
        if let Some(ref mut worklet_manager) = self.audioworklet_manager {
            let _ = worklet_manager.stop_processing();
            let _ = worklet_manager.disconnect();
        }
        self.audioworklet_manager = None;
        self.pitch_analyzer = None;
        
        if let Some(ref context) = self.audio_context {
            dev_log!("Closing AudioContext");
            let _ = context.close();
        }
        self.audio_context = None;
        
        self.is_initialized = false;
        dev_log!("✓ AudioSystemContext shutdown completed");
        Ok(())
    }

    pub fn get_audio_context(&self) -> Option<&AudioContext> {
        self.audio_context.as_ref()
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
    
    
    

    pub fn get_audioworklet_manager_mut(&mut self) -> Option<&mut super::super::worklet::AudioWorkletManager> {
        self.audioworklet_manager.as_mut()
    }

    pub fn get_pitch_analyzer(&self) -> Option<&std::rc::Rc<std::cell::RefCell<super::super::pitch_analyzer::PitchAnalyzer>>> {
        self.pitch_analyzer.as_ref()
    }


    pub fn collect_audio_analysis(&self) -> Option<crate::common::shared_types::AudioAnalysis> {
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
        })
    }

    pub fn collect_audio_errors(&self) -> Vec<crate::common::shared_types::Error> {
        let mut errors = Vec::new();
        
        if let Some(error_msg) = &self.initialization_error {
            errors.push(crate::common::shared_types::Error::ProcessingError(error_msg.clone()));
        }
        
        if let Some(ref context) = self.audio_context {
            if context.state() != AudioContextState::Running {
                let error_msg = match context.state() {
                    AudioContextState::Closed => Some("AudioContext is closed"),
                    // Suspended is a normal state before user interaction, not an error
                    AudioContextState::Suspended => None,
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