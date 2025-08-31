use web_sys::{AudioContext, MediaStream};
use crate::common::dev_log;
use super::{
    signal_flow::AudioSignalFlow,
    worklet::AudioWorkletManager,
    AudioError,
    SignalGeneratorConfig,
    signal_generator::TuningForkConfig,
    PitchData, VolumeLevelData, AudioWorkletStatus,
};

/// AudioPipeline orchestrates the complete audio processing system
/// 
/// This component integrates the declarative AudioSignalFlow with behavioral management,
/// providing a high-level interface for the engine layer while coordinating specialized
/// audio managers for different concerns.
/// 
/// Architecture:
/// - AudioSignalFlow: Defines and creates the Web Audio API node graph
/// - AudioWorkletManager: Manages worklet-specific operations 
/// - AudioPipeline: Orchestrates the system and provides unified API
pub struct AudioPipeline {
    signal_flow: AudioSignalFlow,
    worklet_manager: AudioWorkletManager,
    audio_context: AudioContext,
}

impl AudioPipeline {
    /// Creates a new AudioPipeline with complete audio processing chain
    /// 
    /// This method creates the declarative signal flow, initializes the worklet manager,
    /// and sets up the complete audio processing pipeline ready for use.
    /// 
    /// # Parameters
    /// - `media_stream`: The microphone input stream
    /// - `audio_context`: The Web Audio API context
    /// 
    /// # Returns
    /// Result containing the configured AudioPipeline or error description
    pub fn new(
        media_stream: MediaStream,
        audio_context: AudioContext,
    ) -> Result<Self, String> {
        dev_log!("Creating AudioPipeline with integrated signal flow");

        // Create the AudioWorkletManager (which creates the worklet node)
        let worklet_manager = AudioWorkletManager::new(audio_context.clone())?;

        // Create the media stream source node
        let microphone_source = audio_context
            .create_media_stream_source(&media_stream)
            .map_err(|e| format!("Failed to create media stream source: {:?}", e))?;

        // Create the complete signal flow with all nodes and connections
        let signal_flow = AudioSignalFlow::new(
            audio_context.clone(),
            microphone_source,
            // Get the worklet node from the manager - we'll need to add a getter method
            worklet_manager.get_worklet_node().clone(),
        );

        dev_log!("✓ AudioPipeline created with signal flow and worklet manager");

        Ok(Self {
            signal_flow,
            worklet_manager,
            audio_context,
        })
    }

    /// Setup message handling for the audio worklet
    pub fn setup_message_handling(&mut self) -> Result<(), AudioError> {
        self.worklet_manager.setup_message_handling()
    }

    /// Connect microphone input to the audio pipeline
    /// 
    /// Note: The signal flow already handles the basic connections,
    /// this method handles any additional setup needed for microphone processing
    pub fn connect_microphone(&mut self, _microphone_source: &web_sys::AudioNode, _route_through_analyser: bool) -> Result<(), AudioError> {
        // The signal flow already connects: input -> input_gain -> analyser -> worklet
        // The microphone source is already connected in the signal flow constructor
        dev_log!("Microphone connected via signal flow");
        Ok(())
    }

    /// Start audio processing
    pub fn start_processing(&mut self) -> Result<(), AudioError> {
        self.worklet_manager.start_processing()
    }

    /// Stop audio processing  
    pub fn stop_processing(&mut self) -> Result<(), AudioError> {
        self.worklet_manager.stop_processing()
    }

    /// Check if audio processing is active
    pub fn is_processing(&self) -> bool {
        self.worklet_manager.is_processing()
    }

    /// Set microphone volume using the input gain node from signal flow
    pub fn set_microphone_volume(&mut self, volume: f32) -> Result<(), AudioError> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        self.signal_flow.input_gain.gain().set_value(clamped_volume);
        dev_log!("Set microphone volume to {:.2} via signal flow", clamped_volume);
        Ok(())
    }

    /// Update test signal configuration
    pub fn update_test_signal_config(&mut self, config: SignalGeneratorConfig) {
        if config.enabled {
            // Configure test signal oscillator from signal flow
            self.signal_flow.test_signal_osc.frequency().set_value(config.frequency);
            self.signal_flow.test_signal_gain.gain().set_value(config.amplitude);
            
            // Start the oscillator if not already started
            if let Err(e) = self.signal_flow.test_signal_osc.start() {
                dev_log!("Test signal oscillator might already be started: {:?}", e);
            }
            
            dev_log!("Test signal enabled: {} Hz, amplitude: {}", config.frequency, config.amplitude);
        } else {
            // Disable by setting gain to zero
            self.signal_flow.test_signal_gain.gain().set_value(0.0);
            dev_log!("Test signal disabled");
        }
    }

    /// Update tuning fork configuration
    pub fn update_tuning_fork_config(&mut self, config: TuningForkConfig) {
        // TuningForkConfig doesn't have enabled field, assume enabled if volume > 0
        if config.volume > 0.0 {
            // Configure tuning fork oscillator from signal flow
            self.signal_flow.tuning_fork_osc.frequency().set_value(config.frequency);
            self.signal_flow.tuning_fork_gain.gain().set_value(config.volume);
            
            // Start the oscillator if not already started
            if let Err(e) = self.signal_flow.tuning_fork_osc.start() {
                dev_log!("Tuning fork oscillator might already be started: {:?}", e);
            }
            
            dev_log!("Tuning fork enabled: {} Hz, volume: {}", config.frequency, config.volume);
        } else {
            // Disable by setting gain to zero
            self.signal_flow.tuning_fork_gain.gain().set_value(0.0);
            dev_log!("Tuning fork disabled");
        }
    }

    /// Set whether to output audio stream to speakers
    pub fn set_output_to_speakers(&mut self, enabled: bool) {
        if enabled {
            // The signal flow already connects worklet to destination
            // This could control a master output gain in the future
            dev_log!("Speaker output enabled via signal flow");
        } else {
            // Could disconnect from destination or use a master gain
            dev_log!("Speaker output disabled (not fully implemented)");
        }
    }

    /// Get current audio worklet status
    pub fn get_status(&self) -> AudioWorkletStatus {
        self.worklet_manager.get_status()
    }

    /// Get current volume data
    pub fn get_volume_data(&self) -> Option<VolumeLevelData> {
        self.worklet_manager.get_volume_data()
    }

    /// Get current pitch data
    pub fn get_pitch_data(&self) -> Option<PitchData> {
        self.worklet_manager.get_pitch_data()
    }

    /// Get buffer pool statistics
    pub fn get_buffer_pool_statistics(&self) -> Option<super::message_protocol::BufferPoolStats> {
        self.worklet_manager.get_buffer_pool_statistics()
    }

    /// Set volume detector for audio analysis
    pub fn set_volume_detector(&mut self, detector: super::volume_detector::VolumeDetector) {
        self.worklet_manager.set_volume_detector(detector);
    }

    /// Set pitch analyzer for audio analysis
    pub fn set_pitch_analyzer(&mut self, analyzer: std::rc::Rc<std::cell::RefCell<super::pitch_analyzer::PitchAnalyzer>>) {
        self.worklet_manager.set_pitch_analyzer(analyzer);
    }

    /// Disconnect and cleanup the audio pipeline
    pub fn disconnect(&mut self) -> Result<(), AudioError> {
        // Stop oscillators
        let _ = self.signal_flow.test_signal_osc.stop();
        let _ = self.signal_flow.tuning_fork_osc.stop();
        
        // Cleanup worklet manager
        self.worklet_manager.disconnect()?;
        
        dev_log!("AudioPipeline disconnected and cleaned up");
        Ok(())
    }

    /// Get reference to the signal flow (for future specialized managers)
    pub fn get_signal_flow(&self) -> &AudioSignalFlow {
        &self.signal_flow
    }

    /// Get mutable reference to the signal flow (for future specialized managers)
    pub fn get_signal_flow_mut(&mut self) -> &mut AudioSignalFlow {
        &mut self.signal_flow
    }
}

impl Drop for AudioPipeline {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}