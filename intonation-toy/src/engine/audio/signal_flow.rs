use web_sys::{AudioContext, GainNode, AudioWorkletNode, MediaStreamAudioSourceNode, OscillatorNode, AnalyserNode};
use super::AudioError;
use super::signal_generator::{TuningForkConfig, SignalGeneratorConfig};
use crate::common::dev_log;

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
    pub microphone_source: Option<MediaStreamAudioSourceNode>,
    pub microphone_gain: Option<GainNode>,
    pub mixer_gain: Option<GainNode>,
    pub audioworklet_node: Option<AudioWorkletNode>,
    
    // Analysis nodes (parallel taps)
    pub analyser_node: Option<AnalyserNode>,
    
    // Additional audio sources
    pub test_signal_oscillator: Option<OscillatorNode>,
    pub test_signal_gain: Option<GainNode>,
    pub tuning_fork_oscillator: Option<OscillatorNode>,
    pub tuning_fork_gain: Option<GainNode>,
    
    // Audio context reference
    audio_context: Option<AudioContext>,
    
    // Connection state tracking
    is_connected: bool,
    output_to_speakers: bool,
}

impl AudioSignalFlow {
    /// Creates a new AudioSignalFlow with all nodes uninitialized
    pub fn new() -> Self {
        Self {
            microphone_source: None,
            microphone_gain: None,
            mixer_gain: None,
            audioworklet_node: None,
            analyser_node: None,
            test_signal_oscillator: None,
            test_signal_gain: None,
            tuning_fork_oscillator: None,
            tuning_fork_gain: None,
            audio_context: None,
            is_connected: false,
            output_to_speakers: false,
        }
    }
    
    /// Sets the audio context for node creation
    pub fn set_audio_context(&mut self, context: AudioContext) {
        self.audio_context = Some(context);
    }
    
    /// Creates all audio nodes in the signal flow and establishes connections
    /// 
    /// This method creates all the Web Audio API nodes and connects them according to
    /// the signal flow diagram. The nodes are stored in the struct for later use.
    pub fn create_nodes(&mut self) -> Result<(), AudioError> {
        let context = self.audio_context.as_ref()
            .ok_or_else(|| AudioError::Generic("Audio context not set".to_string()))?;
        
        self.microphone_gain = Some(self.create_microphone_gain_node(context)?);
        self.mixer_gain = Some(self.create_mixer_gain_node(context)?);
        self.analyser_node = Some(self.create_analyser_node(context)?);
        
        let (test_osc, test_gain) = self.create_test_signal_nodes(context)?;
        self.test_signal_oscillator = Some(test_osc);
        self.test_signal_gain = Some(test_gain);
        
        let (fork_osc, fork_gain) = self.create_tuning_fork_nodes(context)?;
        self.tuning_fork_oscillator = Some(fork_osc);
        self.tuning_fork_gain = Some(fork_gain);
        
        self.setup_connections()?;

        Ok(())
    }
    
    /// Creates the microphone gain node for volume control
    fn create_microphone_gain_node(&self, context: &AudioContext) -> Result<GainNode, AudioError> {
        let gain_node = context
            .create_gain()
            .map_err(|e| AudioError::Generic(format!("Failed to create microphone gain node: {:?}", e)))?;
        
        // Set initial gain to unity (1.0)
        gain_node.gain().set_value(1.0);
        
        Ok(gain_node)
    }
    
    /// Creates the mixer gain node for combining audio sources
    fn create_mixer_gain_node(&self, context: &AudioContext) -> Result<GainNode, AudioError> {
        let gain_node = context
            .create_gain()
            .map_err(|e| AudioError::Generic(format!("Failed to create mixer gain node: {:?}", e)))?;
        
        // Set mixer gain to unity (1.0)
        gain_node.gain().set_value(1.0);
        
        Ok(gain_node)
    }
    
    /// Creates the analyser node for audio analysis
    fn create_analyser_node(&self, context: &AudioContext) -> Result<AnalyserNode, AudioError> {
        let analyser = context
            .create_analyser()
            .map_err(|e| AudioError::Generic(format!("Failed to create analyser node: {:?}", e)))?;
        
        analyser.set_fft_size(128);
        analyser.set_smoothing_time_constant(0.0);
        
        Ok(analyser)
    }
    
    /// Creates test signal oscillator and gain nodes
    fn create_test_signal_nodes(&self, context: &AudioContext) -> Result<(OscillatorNode, GainNode), AudioError> {
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
    fn create_tuning_fork_nodes(&self, context: &AudioContext) -> Result<(OscillatorNode, GainNode), AudioError> {
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
        if self.is_connected {
            return Ok(());
        }
        
        let mic_gain = self.microphone_gain.as_ref()
            .ok_or_else(|| AudioError::Generic("Microphone gain node not created".to_string()))?;
        let mixer_gain = self.mixer_gain.as_ref()
            .ok_or_else(|| AudioError::Generic("Mixer gain node not created".to_string()))?;
        let analyser = self.analyser_node.as_ref()
            .ok_or_else(|| AudioError::Generic("Analyser node not created".to_string()))?;
        let test_gain = self.test_signal_gain.as_ref()
            .ok_or_else(|| AudioError::Generic("Test signal gain not created".to_string()))?;
        
        mic_gain.connect_with_audio_node(analyser)
            .map_err(|e| AudioError::Generic(format!("Failed to connect microphone gain to analyser: {:?}", e)))?;
        
        mic_gain.connect_with_audio_node(mixer_gain)
            .map_err(|e| AudioError::Generic(format!("Failed to connect microphone gain to mixer: {:?}", e)))?;
        
        test_gain.connect_with_audio_node(mixer_gain)
            .map_err(|e| AudioError::Generic(format!("Failed to connect test signal to mixer: {:?}", e)))?;
        
        self.is_connected = true;
        Ok(())
    }
    
    /// Connects a microphone source to the signal flow
    pub fn connect_microphone_source(&mut self, source: MediaStreamAudioSourceNode) -> Result<(), AudioError> {
        let mic_gain = self.microphone_gain.as_ref()
            .ok_or_else(|| AudioError::Generic("Microphone gain node not available".to_string()))?;
        
        // Connect microphone source to microphone gain
        source.connect_with_audio_node(mic_gain)
            .map_err(|e| AudioError::Generic(format!("Failed to connect microphone source to gain: {:?}", e)))?;
        
        self.microphone_source = Some(source);
        dev_log!("Connected microphone source to signal flow");
        Ok(())
    }
    
    /// Connects the AudioWorklet node to the mixer
    pub fn connect_audioworklet(&mut self, worklet: AudioWorkletNode) -> Result<(), AudioError> {
        let mixer_gain = self.mixer_gain.as_ref()
            .ok_or_else(|| AudioError::Generic("Mixer gain node not available".to_string()))?;
        
        // Connect mixer to worklet
        mixer_gain.connect_with_audio_node(&worklet)
            .map_err(|e| AudioError::Generic(format!("Failed to connect mixer to worklet: {:?}", e)))?;
        
        self.audioworklet_node = Some(worklet);
        dev_log!("Connected AudioWorklet to signal flow");
        Ok(())
    }
    
    /// Connects or disconnects AudioWorklet output to speakers
    pub fn set_output_to_speakers(&mut self, enabled: bool) -> Result<(), AudioError> {
        if self.output_to_speakers == enabled {
            return Ok(()); // Already in desired state
        }
        
        let worklet = self.audioworklet_node.as_ref()
            .ok_or_else(|| AudioError::Generic("AudioWorklet not connected".to_string()))?;
        let context = self.audio_context.as_ref()
            .ok_or_else(|| AudioError::Generic("Audio context not available".to_string()))?;
        
        if enabled {
            // Connect to speakers
            worklet.connect_with_audio_node(&context.destination())
                .map_err(|e| AudioError::Generic(format!("Failed to connect to speakers: {:?}", e)))?;
            dev_log!("🔊 Connected AudioWorklet to speakers");
        } else {
            // Disconnect from speakers only
            worklet.disconnect_with_audio_node(&context.destination())
                .map_err(|e| AudioError::Generic(format!("Failed to disconnect from speakers: {:?}", e)))?;
            dev_log!("🔇 Disconnected AudioWorklet from speakers");
        }
        
        self.output_to_speakers = enabled;
        Ok(())
    }
    
    /// Creates a test signal oscillator and connects it to the mixer
    pub fn create_test_signal(&mut self, config: SignalGeneratorConfig) -> Result<(), AudioError> {
        let context = self.audio_context.as_ref()
            .ok_or_else(|| AudioError::Generic("Audio context not set".to_string()))?;
        let mixer_gain = self.mixer_gain.as_ref()
            .ok_or_else(|| AudioError::Generic("Mixer gain node not available".to_string()))?;
        
        // Create oscillator
        let oscillator = context
            .create_oscillator()
            .map_err(|e| AudioError::Generic(format!("Failed to create test signal oscillator: {:?}", e)))?;
        
        // Create gain node
        let gain_node = context
            .create_gain()
            .map_err(|e| AudioError::Generic(format!("Failed to create test signal gain: {:?}", e)))?;
        
        // Configure oscillator and gain from config
        oscillator.frequency().set_value(config.frequency);
        let amplitude = if config.enabled { config.amplitude } else { 0.0 };
        gain_node.gain().set_value(amplitude);
        
        // Connect oscillator to gain
        oscillator.connect_with_audio_node(&gain_node)
            .map_err(|e| AudioError::Generic(format!("Failed to connect test signal oscillator to gain: {:?}", e)))?;
        
        // Connect gain to mixer
        gain_node.connect_with_audio_node(mixer_gain)
            .map_err(|e| AudioError::Generic(format!("Failed to connect test signal to mixer: {:?}", e)))?;
        
        // Start oscillator
        oscillator.start()
            .map_err(|e| AudioError::Generic(format!("Failed to start test signal oscillator: {:?}", e)))?;
        
        self.test_signal_oscillator = Some(oscillator);
        self.test_signal_gain = Some(gain_node);
        dev_log!("Created and connected test signal oscillator to signal flow");
        Ok(())
    }
    
    /// Updates the test signal configuration if it exists
    pub fn update_test_signal(&mut self, config: SignalGeneratorConfig) -> Result<(), AudioError> {
        let oscillator = self.test_signal_oscillator.as_ref()
            .ok_or_else(|| AudioError::Generic("Test signal oscillator not created".to_string()))?;
        let gain = self.test_signal_gain.as_ref()
            .ok_or_else(|| AudioError::Generic("Test signal gain not created".to_string()))?;
        
        oscillator.frequency().set_value(config.frequency);
        let amplitude = if config.enabled { config.amplitude } else { 0.0 };
        gain.gain().set_value(amplitude);
        
        dev_log!("Updated test signal: {}Hz, amplitude {:.2}, enabled: {}", 
                 config.frequency, config.amplitude, config.enabled);
        Ok(())
    }
    
    /// Disables the test signal if it exists
    pub fn disable_test_signal(&mut self) -> Result<(), AudioError> {
        if let Some(gain) = &self.test_signal_gain {
            gain.gain().set_value(0.0);
            dev_log!("Disabled test signal");
            Ok(())
        } else {
            Err(AudioError::Generic("Test signal gain not created".to_string()))
        }
    }
    
    /// Creates a tuning fork oscillator with default configuration (440Hz, 0.1 volume)
    pub fn create_tuning_fork(&mut self) -> Result<(), AudioError> {
        let context = self.audio_context.as_ref()
            .ok_or_else(|| AudioError::Generic("Audio context not set".to_string()))?;
        
        // Create oscillator
        let oscillator = context
            .create_oscillator()
            .map_err(|e| AudioError::Generic(format!("Failed to create tuning fork oscillator: {:?}", e)))?;
        
        // Create gain node
        let gain_node = context
            .create_gain()
            .map_err(|e| AudioError::Generic(format!("Failed to create tuning fork gain: {:?}", e)))?;
        
        // Set default frequency (440Hz A4) and volume (0.1)
        oscillator.frequency().set_value(440.0);
        gain_node.gain().set_value(0.1);
        
        // Connect oscillator to gain
        oscillator.connect_with_audio_node(&gain_node)
            .map_err(|e| AudioError::Generic(format!("Failed to connect tuning fork oscillator to gain: {:?}", e)))?;
        
        // Connect gain to speakers (direct path)
        gain_node.connect_with_audio_node(&context.destination())
            .map_err(|e| AudioError::Generic(format!("Failed to connect tuning fork to speakers: {:?}", e)))?;
        
        // Start oscillator
        oscillator.start()
            .map_err(|e| AudioError::Generic(format!("Failed to start tuning fork oscillator: {:?}", e)))?;
        
        self.tuning_fork_oscillator = Some(oscillator);
        self.tuning_fork_gain = Some(gain_node);
        dev_log!("Created tuning fork oscillator in signal flow");
        Ok(())
    }
    
    /// Updates the tuning fork configuration if it exists
    pub fn update_tuning_fork(&mut self, config: TuningForkConfig) -> Result<(), AudioError> {
        let oscillator = self.tuning_fork_oscillator.as_ref()
            .ok_or_else(|| AudioError::Generic("Tuning fork oscillator not created".to_string()))?;
        let gain = self.tuning_fork_gain.as_ref()
            .ok_or_else(|| AudioError::Generic("Tuning fork gain not created".to_string()))?;
        
        oscillator.frequency().set_value(config.frequency);
        gain.gain().set_value(config.volume);
        
        dev_log!("Updated tuning fork: {}Hz, volume {:.2}", config.frequency, config.volume);
        Ok(())
    }
    
    /// Sets microphone volume by adjusting the microphone gain node
    pub fn set_microphone_volume(&self, volume: f32) -> Result<(), AudioError> {
        let mic_gain = self.microphone_gain.as_ref()
            .ok_or_else(|| AudioError::Generic("Microphone gain node not available".to_string()))?;
        
        let clamped_volume = volume.clamp(0.0, 1.0);
        mic_gain.gain().set_value(clamped_volume);
        
        dev_log!("Set microphone volume to {:.2}", clamped_volume);
        Ok(())
    }
    
    /// Gets the mixer gain node for external connections
    pub fn get_mixer_gain(&self) -> Option<&GainNode> {
        self.mixer_gain.as_ref()
    }
    
    /// Gets the microphone gain node for external access
    pub fn get_microphone_gain(&self) -> Option<&GainNode> {
        self.microphone_gain.as_ref()
    }
    
    /// Gets the analyser node for external analysis
    pub fn get_analyser_node(&self) -> Option<&AnalyserNode> {
        self.analyser_node.as_ref()
    }
    
    /// Gets the AudioWorklet node for external access
    pub fn get_audioworklet_node(&self) -> Option<&AudioWorkletNode> {
        self.audioworklet_node.as_ref()
    }
}

impl Default for AudioSignalFlow {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AudioSignalFlow {
    fn drop(&mut self) {
        dev_log!("Disconnecting all signal flow nodes");
        
        // Disconnect AudioWorklet
        if let Some(worklet) = &self.audioworklet_node {
            let _ = worklet.disconnect();
            dev_log!("Disconnected AudioWorklet");
        }
        
        // Disconnect mixer gain
        if let Some(mixer) = &self.mixer_gain {
            let _ = mixer.disconnect();
            dev_log!("Disconnected mixer gain");
        }
        
        // Disconnect microphone gain
        if let Some(mic_gain) = &self.microphone_gain {
            let _ = mic_gain.disconnect();
            dev_log!("Disconnected microphone gain");
        }
        
        // Disconnect analyser node
        if let Some(analyser) = &self.analyser_node {
            let _ = analyser.disconnect();
            dev_log!("Disconnected analyser node");
        }
        
        // Clean up test signal
        if let Some(oscillator) = self.test_signal_oscillator.take() {
            let _ = oscillator.stop();
            let _ = oscillator.disconnect();
            dev_log!("Stopped and disconnected test signal oscillator");
        }
        if let Some(gain) = self.test_signal_gain.take() {
            let _ = gain.disconnect();
            dev_log!("Disconnected test signal gain");
        }
        
        // Clean up tuning fork
        if let Some(oscillator) = self.tuning_fork_oscillator.take() {
            let _ = oscillator.stop();
            let _ = oscillator.disconnect();
            dev_log!("Stopped and disconnected tuning fork oscillator");
        }
        if let Some(gain) = self.tuning_fork_gain.take() {
            let _ = gain.disconnect();
            dev_log!("Disconnected tuning fork gain");
        }
        
        dev_log!("AudioSignalFlow dropped");
    }
}
