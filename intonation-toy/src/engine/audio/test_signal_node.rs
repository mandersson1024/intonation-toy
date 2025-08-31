use web_sys::{AudioContext, OscillatorNode, GainNode, OscillatorType, AudioNode};
use super::legacy_media_stream_node::AudioError;
use super::signal_generator::SignalGeneratorConfig;

pub struct TestSignalAudioNode {
    legacy_oscillator_node: OscillatorNode,
    legacy_gain_node: GainNode,
}

impl TestSignalAudioNode {
    pub fn new(audio_context: &AudioContext, config: SignalGeneratorConfig) -> Result<Self, AudioError> {
        let oscillator = audio_context
            .create_oscillator()
            .map_err(|_| AudioError::Generic("Failed to create oscillator node".to_string()))?;
        
        oscillator.set_type(OscillatorType::Sine);
        
        oscillator
            .frequency()
            .set_value(config.frequency);

        let gain_node = audio_context
            .create_gain()
            .map_err(|_| AudioError::Generic("Failed to create gain node".to_string()))?;
        
        gain_node
            .gain()
            .set_value(if config.enabled { config.amplitude } else { 0.0 });
        
        oscillator
            .connect_with_audio_node(&gain_node)
            .map_err(|_| AudioError::Generic("Failed to connect oscillator to gain node".to_string()))?;
                
        oscillator
            .start()
            .map_err(|_| AudioError::Generic("Failed to start oscillator".to_string()))?;
        
        Ok(Self {
            legacy_oscillator_node: oscillator,
            legacy_gain_node: gain_node,
        })
    }
    
    pub fn update_config(&mut self, new_config: SignalGeneratorConfig) {
        self.legacy_oscillator_node.frequency().set_value(new_config.frequency);
        let amplitude = if new_config.enabled { new_config.amplitude } else { 0.0 };
        self.legacy_gain_node.gain().set_value(amplitude);
    }
    
    pub fn disable(&mut self) {
        self.legacy_gain_node.gain().set_value(0.0);
    }
    
    pub fn connect_to(&mut self, destination: &AudioNode) -> Result<(), AudioError> {
        self.legacy_gain_node
            .connect_with_audio_node(destination)
            .map_err(|_| AudioError::Generic("Failed to connect test signal to destination".to_string()))?;
        Ok(())
    }
    
    pub fn cleanup(&mut self) {
        let _ = self.legacy_oscillator_node.stop();
        let _ = self.legacy_oscillator_node.disconnect();
        let _ = self.legacy_gain_node.disconnect();
    }
}

impl Drop for TestSignalAudioNode {
    fn drop(&mut self) {
        self.cleanup();
    }
}