use web_sys::{AudioContext, AudioNode, GainNode, AnalyserNode, AudioWorkletNode, MediaStreamAudioSourceNode};
use super::{AudioError, VolumeDetector, TuningForkAudioNode, TestSignalAudioNode};
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
///                    Volume Detector (AnalyserNode)
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
    pub volume_detector: Option<VolumeDetector>,
    
    // Additional audio sources
    pub test_signal_node: Option<TestSignalAudioNode>,
    pub tuning_fork_node: Option<TuningForkAudioNode>,
    
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
            volume_detector: None,
            test_signal_node: None,
            tuning_fork_node: None,
            audio_context: None,
            is_connected: false,
            output_to_speakers: false,
        }
    }
    
    /// Sets the audio context for node creation
    pub fn set_audio_context(&mut self, context: AudioContext) {
        self.audio_context = Some(context);
    }
    
    /// Creates all audio nodes in the signal flow
    /// 
    /// This method creates all the Web Audio API nodes but does not initialize them.
    /// The nodes are stored in the struct for later connection and initialization.
    pub fn create_nodes(&mut self) -> Result<(), AudioError> {
        let context = self.audio_context.as_ref()
            .ok_or_else(|| AudioError::Generic("Audio context not set".to_string()))?;
        
        // Create microphone gain node
        self.microphone_gain = Some(self.create_microphone_gain_node(context)?);
        dev_log!("Created microphone gain node");
        
        // Create mixer gain node (central mixing point)
        self.mixer_gain = Some(self.create_mixer_gain_node(context)?);
        dev_log!("Created mixer gain node");
        
        // Create volume detector (analyser node for volume/FFT analysis)
        self.volume_detector = Some(self.create_volume_detector(context)?);
        dev_log!("Created volume detector node");
        
        dev_log!("✓ All audio nodes created successfully");
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
    
    /// Creates the volume detector with analyser node
    fn create_volume_detector(&self, context: &AudioContext) -> Result<VolumeDetector, AudioError> {
        VolumeDetector::new(context)
    }
    
    /// Sets up the complete signal flow connections
    /// 
    /// This method connects all the created nodes according to the signal flow diagram.
    /// It does not initialize any processing - that happens externally.
    pub fn setup_connections(&mut self) -> Result<(), AudioError> {
        if self.is_connected {
            dev_log!("Signal flow already connected, skipping setup");
            return Ok(());
        }
        
        // Ensure all required nodes exist
        let mic_gain = self.microphone_gain.as_ref()
            .ok_or_else(|| AudioError::Generic("Microphone gain node not created".to_string()))?;
        let mixer_gain = self.mixer_gain.as_ref()
            .ok_or_else(|| AudioError::Generic("Mixer gain node not created".to_string()))?;
        let volume_detector = self.volume_detector.as_ref()
            .ok_or_else(|| AudioError::Generic("Volume detector not created".to_string()))?;
        
        // Connect microphone gain to volume detector (parallel tap for analysis)
        volume_detector.connect_source(mic_gain)
            .map_err(|e| AudioError::Generic(format!("Failed to connect microphone gain to volume detector: {:?}", e)))?;
        dev_log!("Connected microphone gain to volume detector");
        
        // Connect microphone gain to mixer
        mic_gain.connect_with_audio_node(mixer_gain)
            .map_err(|e| AudioError::Generic(format!("Failed to connect microphone gain to mixer: {:?}", e)))?;
        dev_log!("Connected microphone gain to mixer");
        
        self.is_connected = true;
        dev_log!("✓ Signal flow connections established");
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
    
    /// Connects a test signal node to the mixer
    pub fn connect_test_signal(&mut self, test_signal: TestSignalAudioNode) -> Result<(), AudioError> {
        let mixer_gain = self.mixer_gain.as_ref()
            .ok_or_else(|| AudioError::Generic("Mixer gain node not available".to_string()))?;
        
        // Connect test signal to mixer
        test_signal.connect_to(mixer_gain)
            .map_err(|e| AudioError::Generic(format!("Failed to connect test signal to mixer: {:?}", e)))?;
        
        self.test_signal_node = Some(test_signal);
        dev_log!("Connected test signal to signal flow");
        Ok(())
    }
    
    /// Sets the tuning fork node (connects directly to speakers, independent path)
    pub fn set_tuning_fork_node(&mut self, tuning_fork: TuningForkAudioNode) {
        self.tuning_fork_node = Some(tuning_fork);
        dev_log!("Set tuning fork node in signal flow");
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
    
    /// Gets the volume detector for external analysis
    pub fn get_volume_detector(&self) -> Option<&VolumeDetector> {
        self.volume_detector.as_ref()
    }
    
    /// Gets the AudioWorklet node for external access
    pub fn get_audioworklet_node(&self) -> Option<&AudioWorkletNode> {
        self.audioworklet_node.as_ref()
    }
    
    /// Checks if signal flow is properly connected
    pub fn is_connected(&self) -> bool {
        self.is_connected && 
        self.microphone_gain.is_some() && 
        self.mixer_gain.is_some() && 
        self.volume_detector.is_some()
    }
    
    /// Disconnects all nodes and cleans up the signal flow
    pub fn disconnect_all(&mut self) -> Result<(), AudioError> {
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
        
        // Disconnect volume detector
        if let Some(volume_detector) = &self.volume_detector {
            let _ = volume_detector.disconnect();
            dev_log!("Disconnected volume detector");
        }
        
        // Clean up test signal
        if let Some(mut test_signal) = self.test_signal_node.take() {
            test_signal.cleanup();
            dev_log!("Cleaned up test signal");
        }
        
        // Clean up tuning fork (Drop trait handles cleanup)
        if self.tuning_fork_node.take().is_some() {
            dev_log!("Cleaned up tuning fork node");
        }
        
        // Clear all references
        self.microphone_source = None;
        self.microphone_gain = None;
        self.mixer_gain = None;
        self.audioworklet_node = None;
        self.volume_detector = None;
        self.test_signal_node = None;
        self.tuning_fork_node = None;
        
        self.is_connected = false;
        self.output_to_speakers = false;
        
        dev_log!("✓ Signal flow cleanup completed");
        Ok(())
    }
}

impl Default for AudioSignalFlow {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AudioSignalFlow {
    fn drop(&mut self) {
        let _ = self.disconnect_all();
        dev_log!("AudioSignalFlow dropped");
    }
}