use web_sys::{AudioContext, OscillatorNode, GainNode, OscillatorType};
use crate::common::dev_log;
use super::microphone::AudioError;
use super::signal_generator::RootNoteAudioConfig;

/// Dedicated root note audio node using Web Audio API's OscillatorNode
/// 
/// This node creates a separate audio path that connects directly to speakers,
/// independent of the main AudioWorklet processing pipeline. This ensures
/// root note audio is always audible regardless of main output settings.
pub struct RootNoteAudioNode {
    /// Reference to the AudioContext
    audio_context: AudioContext,
    /// The oscillator node for generating root note audio
    oscillator: OscillatorNode,
    /// Gain node for volume control
    gain_node: GainNode,
    /// Current configuration
    config: RootNoteAudioConfig,
    /// Whether the node is currently connected and enabled
    is_connected: bool,
}

impl RootNoteAudioNode {
    /// Create a new root note audio node
    /// 
    /// # Arguments
    /// * `audio_context` - The AudioContext to use for creating nodes
    /// * `config` - Initial configuration for the root note audio
    /// 
    /// # Returns
    /// * `Ok(RootNoteAudioNode)` - Successfully created node
    /// * `Err(AudioError)` - Failed to create node
    pub fn new(audio_context: &AudioContext, config: RootNoteAudioConfig) -> Result<Self, AudioError> {
        dev_log!("[RootNoteAudioNode] Creating new root note audio node with frequency: {} Hz", config.frequency);
        
        // Create oscillator node
        let oscillator = audio_context.create_oscillator()
            .map_err(|e| AudioError::Generic(format!("Failed to create oscillator: {:?}", e)))?;
        
        // Set oscillator to sine wave
        oscillator.set_type(OscillatorType::Sine);
        
        // Set initial frequency
        oscillator.frequency().set_value(config.frequency);
        
        // Create gain node for volume control
        let gain_node = audio_context.create_gain()
            .map_err(|e| AudioError::Generic(format!("Failed to create gain node: {:?}", e)))?;
        
        // Set initial volume based on config.enabled state
        let initial_gain = if config.enabled { 0.1 } else { 0.0 };
        gain_node.gain().set_value(initial_gain);
        
        // Connect oscillator -> gain -> destination
        oscillator.connect_with_audio_node(&gain_node)
            .map_err(|e| AudioError::Generic(format!("Failed to connect oscillator to gain: {:?}", e)))?;
        
        gain_node.connect_with_audio_node(&audio_context.destination())
            .map_err(|e| AudioError::Generic(format!("Failed to connect gain to destination: {:?}", e)))?;
        
        // Start the oscillator
        oscillator.start()
            .map_err(|e| AudioError::Generic(format!("Failed to start oscillator: {:?}", e)))?;
        
        dev_log!("[RootNoteAudioNode] Successfully created and started root note audio node - enabled: {}, gain: {}", 
                config.enabled, initial_gain);
        
        Ok(Self {
            audio_context: audio_context.clone(),
            oscillator,
            gain_node,
            config,
            is_connected: true,
        })
    }
    
    /// Update the frequency of the root note oscillator
    /// 
    /// # Arguments
    /// * `frequency` - New frequency in Hz
    pub fn set_frequency(&mut self, frequency: f32) {
        if (self.config.frequency - frequency).abs() > f32::EPSILON {
            dev_log!("[RootNoteAudioNode] Updating frequency from {} Hz to {} Hz", self.config.frequency, frequency);
            self.oscillator.frequency().set_value(frequency);
            self.config.frequency = frequency;
        }
    }
    
    /// Enable root note audio output
    /// 
    /// Sets the gain to an audible level (0.1) to make the root note audible
    pub fn enable(&mut self) {
        if !self.config.enabled {
            dev_log!("[RootNoteAudioNode] Enabling root note audio output");
            self.gain_node.gain().set_value(0.1); // Moderate volume level
            self.config.enabled = true;
        }
    }
    
    /// Disable root note audio output
    /// 
    /// Sets the gain to zero to mute the root note while keeping the oscillator running
    pub fn disable(&mut self) {
        if self.config.enabled {
            dev_log!("[RootNoteAudioNode] Disabling root note audio output");
            self.gain_node.gain().set_value(0.0);
            self.config.enabled = false;
        }
    }
    
    /// Check if the root note audio is currently enabled
    /// 
    /// # Returns
    /// * `true` if enabled and audible
    /// * `false` if disabled or muted
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    
    /// Get the current configuration
    pub fn get_config(&self) -> &RootNoteAudioConfig {
        &self.config
    }
    
    /// Update the entire configuration
    /// 
    /// # Arguments
    /// * `config` - New configuration to apply
    pub fn update_config(&mut self, config: RootNoteAudioConfig) {
        dev_log!("[RootNoteAudioNode] Updating configuration - enabled: {}, frequency: {} Hz", 
                config.enabled, config.frequency);
        
        // Update frequency if changed
        self.set_frequency(config.frequency);
        
        // Update enabled state
        if config.enabled != self.config.enabled {
            if config.enabled {
                self.enable();
            } else {
                self.disable();
            }
        }
    }
    
    /// Disconnect and cleanup the audio node
    /// 
    /// This method is called automatically when the node is dropped,
    /// but can be called manually for explicit cleanup.
    fn cleanup(&mut self) {
        if self.is_connected {
            dev_log!("[RootNoteAudioNode] Cleaning up root note audio node");
            
            // Stop the oscillator
            if let Err(e) = self.oscillator.stop() {
                dev_log!("[RootNoteAudioNode] Warning: Failed to stop oscillator: {:?}", e);
            }
            
            // Disconnect nodes
            let _ = self.oscillator.disconnect();
            let _ = self.gain_node.disconnect();
            
            self.is_connected = false;
        }
    }
}

impl Drop for RootNoteAudioNode {
    fn drop(&mut self) {
        self.cleanup();
    }
}