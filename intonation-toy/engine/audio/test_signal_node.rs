use web_sys::{AudioContext, OscillatorNode, GainNode, OscillatorType, AudioNode};
use crate::common::dev_log;
use super::microphone::AudioError;
use super::signal_generator::SignalGeneratorConfig;

pub struct TestSignalAudioNode {
    audio_context: AudioContext,
    oscillator: OscillatorNode,
    gain_node: GainNode,
    config: SignalGeneratorConfig,
    is_connected: bool,
}

impl TestSignalAudioNode {
    pub fn new(audio_context: &AudioContext, config: SignalGeneratorConfig, connect_to_destination: bool) -> Result<Self, AudioError> {
        dev_log!("Creating TestSignalAudioNode with config: {:?}", config);
        
        let oscillator = audio_context
            .create_oscillator()
            .map_err(|_| AudioError::Generic("Failed to create oscillator node".to_string()))?;
        
        let gain_node = audio_context
            .create_gain()
            .map_err(|_| AudioError::Generic("Failed to create gain node".to_string()))?;
        
        oscillator.set_type(OscillatorType::Sine);
        
        oscillator
            .frequency()
            .set_value(config.frequency);
        
        let amplitude = if config.enabled { config.amplitude } else { 0.0 };
        gain_node
            .gain()
            .set_value(amplitude);
        
        oscillator
            .connect_with_audio_node(&gain_node)
            .map_err(|_| AudioError::Generic("Failed to connect oscillator to gain node".to_string()))?;
        
        if connect_to_destination {
            gain_node
                .connect_with_audio_node(&audio_context.destination())
                .map_err(|_| AudioError::Generic("Failed to connect gain node to destination".to_string()))?;
        }
        
        oscillator
            .start()
            .map_err(|_| AudioError::Generic("Failed to start oscillator".to_string()))?;
        
        dev_log!("TestSignalAudioNode created successfully");
        
        Ok(Self {
            audio_context: audio_context.clone(),
            oscillator,
            gain_node,
            config,
            is_connected: connect_to_destination,
        })
    }
    
    pub fn update_config(&mut self, new_config: SignalGeneratorConfig) {
        dev_log!("Updating TestSignalAudioNode config: {:?}", new_config);
        
        if new_config.frequency != self.config.frequency {
            self.set_frequency(new_config.frequency);
        }
        
        if new_config.amplitude != self.config.amplitude {
            self.set_amplitude(new_config.amplitude);
        }
        
        
        if new_config.enabled != self.config.enabled {
            if new_config.enabled {
                self.enable();
            } else {
                self.disable();
            }
        }
        
        self.config = new_config;
    }
    
    pub fn set_frequency(&mut self, frequency: f32) {
        if frequency != self.config.frequency {
            dev_log!("Setting frequency to: {}", frequency);
            self.oscillator.frequency().set_value(frequency);
            self.config.frequency = frequency;
        }
    }
    
    pub fn set_amplitude(&mut self, amplitude: f32) {
        if amplitude != self.config.amplitude {
            dev_log!("Setting amplitude to: {}", amplitude);
            self.config.amplitude = amplitude;
            if self.config.enabled {
                self.gain_node.gain().set_value(amplitude);
            }
        }
    }
    
    pub fn enable(&mut self) {
        if !self.config.enabled {
            dev_log!("Enabling TestSignalAudioNode");
            self.gain_node.gain().set_value(self.config.amplitude);
            self.config.enabled = true;
        }
    }
    
    pub fn disable(&mut self) {
        if self.config.enabled {
            dev_log!("Disabling TestSignalAudioNode");
            self.gain_node.gain().set_value(0.0);
            self.config.enabled = false;
        }
    }
    
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    
    pub fn get_config(&self) -> &SignalGeneratorConfig {
        &self.config
    }
    
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }
    
    pub fn connect_to(&mut self, destination: &AudioNode) -> Result<(), AudioError> {
        dev_log!("Connecting TestSignalAudioNode to external destination");
        self.gain_node
            .connect_with_audio_node(destination)
            .map_err(|_| AudioError::Generic("Failed to connect test signal to destination".to_string()))?;
        self.is_connected = true;
        Ok(())
    }
    
    pub fn disconnect_from(&mut self, destination: &AudioNode) -> Result<(), AudioError> {
        dev_log!("Disconnecting TestSignalAudioNode from external destination");
        self.gain_node
            .disconnect_with_audio_node(destination)
            .map_err(|_| AudioError::Generic("Failed to disconnect test signal from destination".to_string()))?;
        Ok(())
    }
    
    pub fn get_output_node(&self) -> &GainNode {
        &self.gain_node
    }
    
    pub fn cleanup(&mut self) {
        if self.is_connected {
            dev_log!("Cleaning up TestSignalAudioNode");
            
            if let Err(e) = self.oscillator.stop() {
                dev_log!("Error stopping oscillator: {:?}", e);
            }
            
            if let Err(e) = self.oscillator.disconnect() {
                dev_log!("Error disconnecting oscillator: {:?}", e);
            }
            
            if let Err(e) = self.gain_node.disconnect() {
                dev_log!("Error disconnecting gain node: {:?}", e);
            }
            
            self.is_connected = false;
            dev_log!("TestSignalAudioNode cleanup completed");
        }
    }
}

impl Drop for TestSignalAudioNode {
    fn drop(&mut self) {
        self.cleanup();
    }
}