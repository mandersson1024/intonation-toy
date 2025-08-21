use web_sys::{AudioContext, AudioParam, GainNode, OscillatorNode};
use crate::common::dev_log;
use super::microphone::AudioError;
use super::signal_generator::TuningForkConfig;

/// Dedicated tuning fork audio node using Web Audio API's OscillatorNode
/// 
/// This node creates a separate audio path that connects directly to speakers,
/// independent of the main AudioWorklet processing pipeline. This ensures
/// tuning fork audio is always audible regardless of main output settings.
pub struct TuningForkAudioNode {
    /// Reference to the AudioContext
    audio_context: AudioContext,
    /// The oscillator node for generating tuning fork audio
    oscillator: OscillatorNode,
    /// Gain node for volume control
    gain_node: GainNode,
    /// Current configuration
    config: TuningForkConfig,
    /// Whether the node is currently connected and enabled
    is_connected: bool,
}

impl TuningForkAudioNode {
    /// Create a new tuning fork audio node
    /// 
    /// # Arguments
    /// * `audio_context` - The AudioContext to use for creating nodes
    /// * `config` - Initial configuration for the tuning fork audio
    /// 
    /// # Returns
    /// * `Ok(TuningForkAudioNode)` - Successfully created node
    /// * `Err(AudioError)` - Failed to create node
    pub fn new(audio_context: &AudioContext, config: TuningForkConfig) -> Result<Self, AudioError> {
        dev_log!("[TuningForkAudioNode] Creating new tuning fork audio node with frequency: {} Hz", config.frequency);
        
        // Create oscillator node
        let oscillator = audio_context.create_oscillator()
            .map_err(|e| AudioError::Generic(format!("Failed to create oscillator: {:?}", e)))?;
        
        let n = 16;
        let mut real = vec![0.0f32; n];
        let mut imag = vec![0.0f32; n];
        
        let amps: [f32; 9] = [
            0.0,   // DC offset
            1.0,   // fundamental
            0.85,  // 2nd
            0.55,  // 3rd
            0.40,  // 4th
            0.25,  // 5th
            0.18,  // 6th
            0.12,  // 7th
            0.08   // 8th
        ];

        for (i, &amp) in amps.iter().enumerate() {
            real[i] = amp;
        }

        let periodic_wave = audio_context.create_periodic_wave(&mut real, &mut imag)
            .map_err(|e| AudioError::Generic(format!("Failed to create periodic wave: {:?}", e)))?;
        
        oscillator.set_periodic_wave(&periodic_wave);
        
        // Set initial frequency
        oscillator.frequency().set_value(config.frequency);
        
        // Create gain node for volume control
        let gain_node = audio_context.create_gain()
            .map_err(|e| AudioError::Generic(format!("Failed to create gain node: {:?}", e)))?;
        
        // Set initial volume
        gain_node.gain().set_value(config.volume);
        
        // Connect oscillator -> gain -> destination
        oscillator.connect_with_audio_node(&gain_node)
            .map_err(|e| AudioError::Generic(format!("Failed to connect oscillator to gain: {:?}", e)))?;
        
        gain_node.connect_with_audio_node(&audio_context.destination())
            .map_err(|e| AudioError::Generic(format!("Failed to connect gain to destination: {:?}", e)))?;
        
        // Start the oscillator
        oscillator.start()
            .map_err(|e| AudioError::Generic(format!("Failed to start oscillator: {:?}", e)))?;
        
        dev_log!("[TuningForkAudioNode] Successfully created and started tuning fork audio node - gain: {}", 
                config.volume);
        
        Ok(Self {
            audio_context: audio_context.clone(),
            oscillator,
            gain_node,
            config,
            is_connected: true,
        })
    }
    
    /// Update the frequency of the tuning fork oscillator
    /// 
    /// # Arguments
    /// * `frequency` - New frequency in Hz
    pub fn set_frequency(&mut self, frequency: f32) {
        if (self.config.frequency - frequency).abs() > f32::EPSILON {
            dev_log!("[TuningForkAudioNode] Updating frequency from {} Hz to {} Hz", self.config.frequency, frequency);
            self.oscillator.frequency().set_value(frequency);
            self.config.frequency = frequency;
        }
    }
    
    
    /// Check if the tuning fork audio is currently enabled
    /// 
    /// # Returns
    /// * Always returns `true` since the node is always running
    pub fn is_enabled(&self) -> bool {
        true
    }
    
    /// Get the current configuration
    pub fn get_config(&self) -> &TuningForkConfig {
        &self.config
    }
    
    fn ramp_gain(&self, target : f32) {
        let result: Result<AudioParam, _> = self.gain_node.gain().set_target_at_time(target, self.audio_context.current_time(), 0.05);
        match result {
            Ok(_) => {},
            Err(_) => { 
                dev_log!("[TuningForkAudioNode] Error gradually changing volume");
                self.gain_node.gain().set_value(target);
             },
        }
    }

    /// Update the entire configuration
    /// 
    /// # Arguments
    /// * `config` - New configuration to apply
    pub fn update_config(&mut self, config: TuningForkConfig) {
        //dev_log!("[TuningForkAudioNode] Updating configuration - enabled: {}, frequency: {} Hz, volume: {}", config.enabled, config.frequency, config.volume);
        
        // Update frequency if changed
        self.set_frequency(config.frequency);
        
        // Update volume if changed
        if (self.config.volume - config.volume).abs() > f32::EPSILON {
            self.config.volume = config.volume;
            self.ramp_gain(config.volume);
        }
    }
    
    /// Disconnect and cleanup the audio node
    /// 
    /// This method is called automatically when the node is dropped,
    /// but can be called manually for explicit cleanup.
    fn cleanup(&mut self) {
        if self.is_connected {
            dev_log!("[TuningForkAudioNode] Cleaning up tuning fork audio node");
            
            // Stop the oscillator
            if let Err(e) = self.oscillator.stop() {
                dev_log!("[TuningForkAudioNode] Warning: Failed to stop oscillator: {:?}", e);
            }
            
            // Disconnect nodes
            let _ = self.oscillator.disconnect();
            let _ = self.gain_node.disconnect();
            
            self.is_connected = false;
        }
    }
}

impl Drop for TuningForkAudioNode {
    fn drop(&mut self) {
        self.cleanup();
    }
}