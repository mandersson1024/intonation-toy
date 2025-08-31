use web_sys::{AudioContext, GainNode, OscillatorNode};
use super::legacy_media_stream_node::AudioError;
use super::signal_generator::TuningForkConfig;

pub struct TuningForkAudioNode {
    audio_context: AudioContext,
    oscillator: OscillatorNode,
    gain_node: GainNode,
    config: TuningForkConfig,
    is_connected: bool,
}

impl TuningForkAudioNode {
    pub fn new(audio_context: &AudioContext, config: TuningForkConfig) -> Result<Self, AudioError> {
        let oscillator = audio_context.create_oscillator()
            .map_err(|_| AudioError::Generic("Failed to create oscillator".to_string()))?;
        
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
            .map_err(|_| AudioError::Generic("Failed to create periodic wave".to_string()))?;
        
        oscillator.set_periodic_wave(&periodic_wave);
        oscillator.frequency().set_value(config.frequency);
        
        let gain_node = audio_context.create_gain()
            .map_err(|_| AudioError::Generic("Failed to create gain node".to_string()))?;
        gain_node.gain().set_value(config.volume);
        
        oscillator.connect_with_audio_node(&gain_node)
            .map_err(|_| AudioError::Generic("Failed to connect oscillator to gain".to_string()))?;
        gain_node.connect_with_audio_node(&audio_context.destination())
            .map_err(|_| AudioError::Generic("Failed to connect gain to destination".to_string()))?;
        oscillator.start()
            .map_err(|_| AudioError::Generic("Failed to start oscillator".to_string()))?;
        
        
        Ok(Self {
            audio_context: audio_context.clone(),
            oscillator,
            gain_node,
            config,
            is_connected: true,
        })
    }
    
    fn ramp_gain(&self, target: f32) {
        if self.gain_node.gain().set_target_at_time(target, self.audio_context.current_time(), 0.05).is_err() {
            self.gain_node.gain().set_value(target);
        }
    }

    pub fn update_config(&mut self, config: TuningForkConfig) {
        if (self.config.frequency - config.frequency).abs() > f32::EPSILON {
            self.oscillator.frequency().set_value(config.frequency);
            self.config.frequency = config.frequency;
        }
        
        if (self.config.volume - config.volume).abs() > f32::EPSILON {
            self.config.volume = config.volume;
            self.ramp_gain(config.volume);
        }
    }
    
    fn cleanup(&mut self) {
        if self.is_connected {
            let _ = self.oscillator.stop();
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