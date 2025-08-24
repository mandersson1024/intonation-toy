use web_sys::{AudioContext, GainNode, AudioWorkletNode, MediaStreamAudioSourceNode, OscillatorNode, AnalyserNode};

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
    ) -> Self {
        // Create nodes
        let microphone_gain = context.create_gain().unwrap();
        let mixer_gain = context.create_gain().unwrap();
        let analyser_node = context.create_analyser().unwrap();
        let test_signal_oscillator = context.create_oscillator().unwrap();
        let test_signal_gain = context.create_gain().unwrap();
        let tuning_fork_oscillator = context.create_oscillator().unwrap();
        let tuning_fork_gain = context.create_gain().unwrap();
        
        // Connect internal nodes
        test_signal_oscillator.connect_with_audio_node(&test_signal_gain).unwrap();
        tuning_fork_oscillator.connect_with_audio_node(&tuning_fork_gain).unwrap();
        tuning_fork_gain.connect_with_audio_node(&context.destination()).unwrap();
        
        // Start oscillators
        test_signal_oscillator.start().unwrap();
        tuning_fork_oscillator.start().unwrap();
        
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
        
        signal_flow.setup_connections();
        
        signal_flow
    }
    
    
    /// Sets up the complete signal flow connections
    /// 
    /// This method connects all the created nodes according to the signal flow diagram.
    /// It does not initialize any processing - that happens externally.
    fn setup_connections(&mut self) {
        self.microphone_source.connect_with_audio_node(&self.microphone_gain).unwrap();
        self.microphone_gain.connect_with_audio_node(&self.analyser_node).unwrap();
        self.microphone_gain.connect_with_audio_node(&self.mixer_gain).unwrap();
        self.test_signal_gain.connect_with_audio_node(&self.mixer_gain).unwrap();
        self.mixer_gain.connect_with_audio_node(&self.audioworklet_node).unwrap();
    }
    
    
    /// Connects or disconnects AudioWorklet output to speakers
    pub fn set_output_to_speakers(&mut self, enabled: bool) {
        if self.output_to_speakers == enabled {
            return;
        }
        
        if enabled {
            self.audioworklet_node.connect_with_audio_node(&self.audio_context.destination()).unwrap();
        } else {
            self.audioworklet_node.disconnect_with_audio_node(&self.audio_context.destination()).unwrap();
        }
        
        self.output_to_speakers = enabled;
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
