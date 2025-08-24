use web_sys::{AudioContext, GainNode, AudioWorkletNode, MediaStreamAudioSourceNode, OscillatorNode, AnalyserNode};
use super::AudioError;
use super::signal_generator::{TuningForkConfig, SignalGeneratorConfig};

/// Represents the complete audio signal flow with all Web Audio API nodes
/// 
/// This struct manages the creation and storage of all audio nodes in the signal processing chain.
/// The nodes are created but not initialized - initialization happens externally after creation.
/// 
/// Signal Flow:
/// ```
/// Microphone Input → Microphone Gain → Mixer Gain → AudioWorklet → [Optional] Speakers
///                           ↓ (parallel)
///                       AnalyserNode
///                           
/// Test Signal → Mixer Gain ↗
/// 
/// Tuning Fork → Speakers (direct, independent path)
/// ```
pub struct AudioSignalFlow {
    // Core audio processing chain
    pub microphone_source: MediaStreamAudioSourceNode,
    pub microphone_gain: GainNode,
    pub mixer_gain: GainNode,
    pub audioworklet_node: AudioWorkletNode,
    
    // Analysis nodes (parallel taps)
    pub analyser_node: AnalyserNode,
    
    // Additional audio sources
    pub test_signal_oscillator: OscillatorNode,
    pub test_signal_gain: GainNode,
    pub tuning_fork_oscillator: OscillatorNode,
    pub tuning_fork_gain: GainNode,
    
    // Audio context reference
    audio_context: AudioContext,
    
    // Connection state tracking
    output_to_speakers: bool,
}

impl AudioSignalFlow {
    /// Creates a new AudioSignalFlow with all nodes initialized and connected
    pub fn new(
        context: AudioContext,
        microphone_source: MediaStreamAudioSourceNode,
        worklet: AudioWorkletNode,
    ) -> Result<Self, AudioError> {
        let microphone_gain = Self::create_microphone_gain_node(&context)?;
        let mixer_gain = Self::create_mixer_gain_node(&context)?;
        let analyser_node = Self::create_analyser_node(&context)?;
        
        let (test_signal_oscillator, test_signal_gain) = Self::create_test_signal_nodes(&context)?;
        let (tuning_fork_oscillator, tuning_fork_gain) = Self::create_tuning_fork_nodes(&context)?;
        
        let mut signal_flow = Self {
            microphone_source,
            microphone_gain,
            mixer_gain,
            audioworklet_node: worklet,
            analyser_node,
            test_signal_oscillator,
            test_signal_gain,
            tuning_fork_oscillator,
            tuning_fork_gain,
            audio_context: context,
            output_to_speakers: false,
        };
        
        signal_flow.setup_connections()?;
        
        Ok(signal_flow)
    }
    
    /// Creates the microphone gain node for volume control
    fn create_microphone_gain_node(context: &AudioContext) -> Result<GainNode, AudioError> {
        let gain_node = context
            .create_gain()
            .map_err(|e| AudioError::Generic(format!("Failed to create microphone gain node: {:?}", e)))?;
        
        // Set initial gain to unity (1.0)
        gain_node.gain().set_value(1.0);
        
        Ok(gain_node)
    }
    
    /// Creates the mixer gain node for combining audio sources
    fn create_mixer_gain_node(context: &AudioContext) -> Result<GainNode, AudioError> {
        let gain_node = context
            .create_gain()
            .map_err(|e| AudioError::Generic(format!("Failed to create mixer gain node: {:?}", e)))?;
        
        // Set mixer gain to unity (1.0)
        gain_node.gain().set_value(1.0);
        
        Ok(gain_node)
    }
    
    /// Creates the analyser node for audio analysis
    fn create_analyser_node(context: &AudioContext) -> Result<AnalyserNode, AudioError> {
        let analyser = context
            .create_analyser()
            .map_err(|e| AudioError::Generic(format!("Failed to create analyser node: {:?}", e)))?;
        
        analyser.set_fft_size(128);
        analyser.set_smoothing_time_constant(0.0);
        
        Ok(analyser)
    }
    
    /// Creates test signal oscillator and gain nodes
    fn create_test_signal_nodes(context: &AudioContext) -> Result<(OscillatorNode, GainNode), AudioError> {
        let oscillator = context
            .create_oscillator()
            .map_err(|e| AudioError::Generic(format!("Failed to create test signal oscillator: {:?}", e)))?;
        
        let gain_node = context
            .create_gain()
            .map_err(|e| AudioError::Generic(format!("Failed to create test signal gain: {:?}", e)))?;
        
        oscillator.frequency().set_value(440.0);
        gain_node.gain().set_value(0.0);
        
        oscillator.connect_with_audio_node(&gain_node)
            .map_err(|e| AudioError::Generic(format!("Failed to connect test signal oscillator to gain: {:?}", e)))?;
        
        oscillator.start()
            .map_err(|e| AudioError::Generic(format!("Failed to start test signal oscillator: {:?}", e)))?;
        
        Ok((oscillator, gain_node))
    }
    
    /// Creates tuning fork oscillator and gain nodes
    fn create_tuning_fork_nodes(context: &AudioContext) -> Result<(OscillatorNode, GainNode), AudioError> {
        let oscillator = context
            .create_oscillator()
            .map_err(|e| AudioError::Generic(format!("Failed to create tuning fork oscillator: {:?}", e)))?;
        
        let gain_node = context
            .create_gain()
            .map_err(|e| AudioError::Generic(format!("Failed to create tuning fork gain: {:?}", e)))?;
        
        oscillator.frequency().set_value(440.0);
        gain_node.gain().set_value(0.1);
        
        oscillator.connect_with_audio_node(&gain_node)
            .map_err(|e| AudioError::Generic(format!("Failed to connect tuning fork oscillator to gain: {:?}", e)))?;
        
        gain_node.connect_with_audio_node(&context.destination())
            .map_err(|e| AudioError::Generic(format!("Failed to connect tuning fork to speakers: {:?}", e)))?;
        
        oscillator.start()
            .map_err(|e| AudioError::Generic(format!("Failed to start tuning fork oscillator: {:?}", e)))?;
        
        Ok((oscillator, gain_node))
    }
    
    /// Sets up the complete signal flow connections
    /// 
    /// This method connects all the created nodes according to the signal flow diagram.
    /// It does not initialize any processing - that happens externally.
    fn setup_connections(&mut self) -> Result<(), AudioError> {
        self.microphone_source.connect_with_audio_node(&self.microphone_gain)
            .map_err(|e| AudioError::Generic(format!("Failed to connect microphone source to gain: {:?}", e)))?;
        
        self.microphone_gain.connect_with_audio_node(&self.analyser_node)
            .map_err(|e| AudioError::Generic(format!("Failed to connect microphone gain to analyser: {:?}", e)))?;
        
        self.microphone_gain.connect_with_audio_node(&self.mixer_gain)
            .map_err(|e| AudioError::Generic(format!("Failed to connect microphone gain to mixer: {:?}", e)))?;
        
        self.test_signal_gain.connect_with_audio_node(&self.mixer_gain)
            .map_err(|e| AudioError::Generic(format!("Failed to connect test signal to mixer: {:?}", e)))?;
        
        self.mixer_gain.connect_with_audio_node(&self.audioworklet_node)
            .map_err(|e| AudioError::Generic(format!("Failed to connect mixer to worklet: {:?}", e)))?;
        
        Ok(())
    }
    
    
    /// Connects or disconnects AudioWorklet output to speakers
    pub fn set_output_to_speakers(&mut self, enabled: bool) -> Result<(), AudioError> {
        if self.output_to_speakers == enabled {
            return Ok(());
        }
        
        if enabled {
            self.audioworklet_node.connect_with_audio_node(&self.audio_context.destination())
                .map_err(|e| AudioError::Generic(format!("Failed to connect to speakers: {:?}", e)))?;
        } else {
            self.audioworklet_node.disconnect_with_audio_node(&self.audio_context.destination())
                .map_err(|e| AudioError::Generic(format!("Failed to disconnect from speakers: {:?}", e)))?;
        }
        
        self.output_to_speakers = enabled;
        Ok(())
    }
    
    
    /// Updates the test signal configuration
    pub fn update_test_signal(&mut self, config: SignalGeneratorConfig) {
        self.test_signal_oscillator.frequency().set_value(config.frequency);
        let amplitude = if config.enabled { config.amplitude } else { 0.0 };
        self.test_signal_gain.gain().set_value(amplitude);
    }
    
    /// Disables the test signal
    pub fn disable_test_signal(&mut self) {
        self.test_signal_gain.gain().set_value(0.0);
    }
    
    
    /// Updates the tuning fork configuration
    pub fn update_tuning_fork(&mut self, config: TuningForkConfig) {
        self.tuning_fork_oscillator.frequency().set_value(config.frequency);
        self.tuning_fork_gain.gain().set_value(config.volume);
    }
    
    /// Sets microphone volume by adjusting the microphone gain node
    pub fn set_microphone_volume(&self, volume: f32) {
        let clamped_volume = volume.clamp(0.0, 1.0);
        self.microphone_gain.gain().set_value(clamped_volume);
    }
    
    /// Gets the mixer gain node for external connections
    pub fn get_mixer_gain(&self) -> &GainNode {
        &self.mixer_gain
    }
    
    /// Gets the microphone gain node for external access
    pub fn get_microphone_gain(&self) -> &GainNode {
        &self.microphone_gain
    }
    
    /// Gets the analyser node for external analysis
    pub fn get_analyser_node(&self) -> &AnalyserNode {
        &self.analyser_node
    }
    
    /// Gets the AudioWorklet node for external access
    pub fn get_audioworklet_node(&self) -> &AudioWorkletNode {
        &self.audioworklet_node
    }
}

impl Drop for AudioSignalFlow {
    fn drop(&mut self) {
        let _ = self.audioworklet_node.disconnect();
        let _ = self.mixer_gain.disconnect();
        let _ = self.microphone_gain.disconnect();
        let _ = self.analyser_node.disconnect();
        
        let _ = self.test_signal_oscillator.stop();
        let _ = self.test_signal_oscillator.disconnect();
        let _ = self.test_signal_gain.disconnect();
        
        let _ = self.tuning_fork_oscillator.stop();
        let _ = self.tuning_fork_oscillator.disconnect();
        let _ = self.tuning_fork_gain.disconnect();
    }
}
