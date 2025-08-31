use web_sys::{AudioContext, OscillatorNode, GainNode, OscillatorType, AudioNode};
use super::legacy_media_stream_node::AudioError;
use super::signal_generator::SignalGeneratorConfig;

pub struct TestSignalAudioNode {
    oscillator: OscillatorNode,
    gain_node: GainNode,
    config: SignalGeneratorConfig,
    is_connected: bool,
}

impl TestSignalAudioNode {
    pub fn new(audio_context: &AudioContext, config: SignalGeneratorConfig, connect_to_destination: bool) -> Result<Self, AudioError> {
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
        
        Ok(Self {
            oscillator,
            gain_node,
            config,
            is_connected: connect_to_destination,
        })
    }
    
    pub fn update_config(&mut self, new_config: SignalGeneratorConfig) {
        if new_config.frequency != self.config.frequency {
            self.oscillator.frequency().set_value(new_config.frequency);
        }
        
        if new_config.amplitude != self.config.amplitude && new_config.enabled {
            self.gain_node.gain().set_value(new_config.amplitude);
        }
        
        if new_config.enabled != self.config.enabled {
            let amplitude = if new_config.enabled { new_config.amplitude } else { 0.0 };
            self.gain_node.gain().set_value(amplitude);
        }
        
        self.config = new_config;
    }
    
    
    
    pub fn disable(&mut self) {
        self.gain_node.gain().set_value(0.0);
        self.config.enabled = false;
    }
    
    pub fn connect_to(&mut self, destination: &AudioNode) -> Result<(), AudioError> {
        self.gain_node
            .connect_with_audio_node(destination)
            .map_err(|_| AudioError::Generic("Failed to connect test signal to destination".to_string()))?;
        self.is_connected = true;
        Ok(())
    }
    
    pub fn cleanup(&mut self) {
        if self.is_connected {
            let _ = self.oscillator.stop();
            let _ = self.oscillator.disconnect();
            let _ = self.gain_node.disconnect();
            self.is_connected = false;
        }
    }
}

impl Drop for TestSignalAudioNode {
    fn drop(&mut self) {
        self.cleanup();
    }
}