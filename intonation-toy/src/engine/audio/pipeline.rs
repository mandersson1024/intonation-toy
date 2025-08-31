use web_sys::{AudioContext, MediaStream};
use crate::common::dev_log;
use super::{
    signal_flow::AudioSignalFlow,
    worklet_manager::AudioWorkletManager,
    analyzer::AudioAnalyzer,
    generator::AudioGenerator,
    router::AudioRouter,
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
/// - AudioAnalyzer: Manages audio analysis operations
/// - AudioGenerator: Manages audio generation operations
/// - AudioRouter: Manages audio routing and mixing operations
/// - AudioPipeline: Orchestrates the system and provides unified API
pub struct AudioPipeline {
    signal_flow: AudioSignalFlow,
    worklet_manager: AudioWorkletManager,
    analyzer: AudioAnalyzer,
    generator: AudioGenerator,
    router: AudioRouter,
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

        // First create the worklet node using the existing helper
        let worklet_node = Self::create_worklet_node(&audio_context)?;

        // Create the media stream source node
        let microphone_source = audio_context
            .create_media_stream_source(&media_stream)
            .map_err(|e| format!("Failed to create media stream source: {:?}", e))?;

        // Create the complete signal flow with all nodes and connections
        let signal_flow = AudioSignalFlow::new(
            audio_context.clone(),
            microphone_source,
            worklet_node.clone(),
        );

        // Create the focused AudioWorkletManager with the worklet node
        let worklet_manager = AudioWorkletManager::new(worklet_node)?;

        // Create the AudioAnalyzer with the analyser node from signal flow
        let analyzer = AudioAnalyzer::new(signal_flow.analyser.clone())
            .map_err(|e| format!("Failed to create AudioAnalyzer: {}", e))?;

        // Create the AudioGenerator with oscillator and gain nodes from signal flow
        let generator = AudioGenerator::new(
            signal_flow.test_signal_osc.clone(),
            signal_flow.test_signal_gain.clone(),
            signal_flow.tuning_fork_osc.clone(),
            signal_flow.tuning_fork_gain.clone(),
        ).map_err(|e| format!("Failed to create AudioGenerator: {}", e))?;

        // Create the AudioRouter with gain nodes from signal flow
        let router = AudioRouter::new(
            signal_flow.input_gain.clone(),
        ).map_err(|e| format!("Failed to create AudioRouter: {}", e))?;

        dev_log!("✓ AudioPipeline created with signal flow, worklet manager, analyzer, generator, and router");

        Ok(Self {
            signal_flow,
            worklet_manager,
            analyzer,
            generator,
            router,
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

    /// Set microphone volume using the router
    pub fn set_microphone_volume(&mut self, volume: f32) -> Result<(), AudioError> {
        self.router.set_microphone_volume(volume)
    }

    /// Update test signal configuration
    pub fn update_test_signal_config(&mut self, config: SignalGeneratorConfig) {
        self.generator.update_test_signal_config(config);
    }

    /// Update tuning fork configuration
    pub fn update_tuning_fork_config(&mut self, config: TuningForkConfig) {
        self.generator.update_tuning_fork_config(config);
    }

    /// Set whether to output audio stream to speakers
    pub fn set_output_to_speakers(&mut self, enabled: bool) {
        self.router.set_output_to_speakers(enabled);
    }

    /// Get current audio worklet status
    pub fn get_status(&self) -> AudioWorkletStatus {
        self.worklet_manager.get_status()
    }

    /// Get current volume data
    pub fn get_volume_data(&self) -> Option<VolumeLevelData> {
        self.analyzer.get_volume_data()
    }

    /// Get current pitch data  
    pub fn get_pitch_data(&self) -> Option<PitchData> {
        self.analyzer.get_pitch_data()
    }

    /// Get buffer pool statistics
    pub fn get_buffer_pool_statistics(&self) -> Option<super::message_protocol::BufferPoolStats> {
        self.worklet_manager.get_buffer_pool_statistics()
    }

    /// Set volume detector for audio analysis
    pub fn set_volume_detector(&mut self, detector: super::volume_detector::VolumeDetector) -> Result<(), AudioError> {
        // Set the volume detector in the analyzer (connects to analyser node)
        self.analyzer.set_volume_detector(detector)?;
        
        // Also provide it to the worklet manager for message handling
        if let Some(volume_detector) = self.analyzer.get_volume_detector() {
            self.worklet_manager.set_volume_detector(volume_detector.borrow().clone());
        }
        
        Ok(())
    }

    /// Set pitch analyzer for audio analysis  
    pub fn set_pitch_analyzer(&mut self, analyzer_ref: std::rc::Rc<std::cell::RefCell<super::pitch_analyzer::PitchAnalyzer>>) {
        // Set the pitch analyzer in the analyzer
        self.analyzer.set_pitch_analyzer(analyzer_ref.clone());
        
        // Also provide it to the worklet manager for message handling
        self.worklet_manager.set_pitch_analyzer(analyzer_ref);
    }

    /// Disconnect and cleanup the audio pipeline
    pub fn disconnect(&mut self) -> Result<(), AudioError> {
        // Cleanup generator (stops oscillators)
        self.generator.disconnect()?;
        
        // Cleanup router (mutes audio)
        self.router.disconnect()?;
        
        // Cleanup analyzer
        self.analyzer.disconnect()?;
        
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

    /// Create AudioWorkletNode with standard configuration
    /// 
    /// This method creates an AudioWorkletNode using standard configuration options.
    /// The worklet module must already be loaded in the AudioContext before calling this method.
    /// 
    /// # Parameters
    /// - `audio_context`: Reference to the AudioContext with worklet module loaded
    /// 
    /// # Returns
    /// Returns `Result<AudioWorkletNode, String>` where:
    /// - On success: AudioWorkletNode ready for use
    /// - On error: String describing what went wrong
    fn create_worklet_node(audio_context: &AudioContext) -> Result<web_sys::AudioWorkletNode, String> {
        use web_sys::{AudioWorkletNodeOptions, AudioWorkletNode};
        
        dev_log!("Creating AudioWorkletNode with standard configuration");
        
        // Create AudioWorkletNode with default options
        let options = AudioWorkletNodeOptions::new();
        options.set_number_of_inputs(1);
        options.set_number_of_outputs(1);
        
        // Set channel configuration
        let output_channels = js_sys::Array::of1(&js_sys::Number::from(1u32));
        options.set_channel_count(1);
        options.set_channel_count_mode(web_sys::ChannelCountMode::Explicit);
        options.set_channel_interpretation(web_sys::ChannelInterpretation::Speakers);
        options.set_output_channel_count(&output_channels);
        
        // Create the AudioWorkletNode with the registered processor
        let worklet_node = AudioWorkletNode::new_with_options(audio_context, "pitch-processor", &options)
            .map_err(|e| format!("Failed to create AudioWorkletNode 'pitch-processor': {:?}", e))?;
        
        dev_log!("✓ AudioWorkletNode created successfully");
        Ok(worklet_node)
    }
}

impl Drop for AudioPipeline {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}