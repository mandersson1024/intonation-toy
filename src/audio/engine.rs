use wasm_bindgen::prelude::*;

/// Core audio engine for real-time pitch detection and processing
#[wasm_bindgen]
pub struct AudioEngine {
    sample_rate: f32,
    buffer_size: usize,
    enabled: bool,
}

#[wasm_bindgen]
impl AudioEngine {
    /// Create a new AudioEngine instance
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: f32, buffer_size: usize) -> AudioEngine {
        AudioEngine {
            sample_rate,
            buffer_size,
            enabled: true,
        }
    }

    /// Process audio buffer - basic implementation for WASM pipeline validation
    #[wasm_bindgen]
    pub fn process_audio_buffer(&mut self, input: &[f32]) -> Vec<f32> {
        if !self.enabled {
            return vec![0.0; input.len()];
        }

        // Basic passthrough processing for pipeline validation
        // Real pitch detection will be implemented in Story 1.2
        let mut output = input.to_vec();

        // Apply basic gain for testing
        for sample in &mut output {
            *sample *= 0.8;
        }

        output
    }

    /// Enable/disable audio processing
    #[wasm_bindgen]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Get current sample rate
    #[wasm_bindgen]
    pub fn get_sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Get buffer size
    #[wasm_bindgen]
    pub fn get_buffer_size(&self) -> usize {
        self.buffer_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_engine_creation() {
        let engine = AudioEngine::new(44100.0, 1024);
        assert_eq!(engine.get_sample_rate(), 44100.0);
        assert_eq!(engine.get_buffer_size(), 1024);
        assert!(engine.enabled);
    }

    #[test]
    fn test_audio_engine_enable_disable() {
        let mut engine = AudioEngine::new(44100.0, 1024);

        // Initially enabled
        assert!(engine.enabled);

        // Test disable
        engine.set_enabled(false);
        assert!(!engine.enabled);

        // Test re-enable
        engine.set_enabled(true);
        assert!(engine.enabled);
    }

    #[test]
    fn test_audio_processing_enabled() {
        let mut engine = AudioEngine::new(44100.0, 1024);
        let input = vec![1.0, 0.5, -0.5, -1.0];

        let output = engine.process_audio_buffer(&input);

        assert_eq!(output.len(), input.len());
        // Gain reduction of 0.8
        assert_eq!(output[0], 0.8);
        assert_eq!(output[1], 0.4);
        assert_eq!(output[2], -0.4);
        assert_eq!(output[3], -0.8);
    }

    #[test]
    fn test_audio_processing_disabled() {
        let mut engine = AudioEngine::new(44100.0, 1024);
        engine.set_enabled(false);

        let input = vec![1.0, 0.5, -0.5, -1.0];
        let output = engine.process_audio_buffer(&input);

        assert_eq!(output.len(), input.len());
        // All zeros when disabled
        assert!(output.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_empty_buffer_processing() {
        let mut engine = AudioEngine::new(44100.0, 1024);
        let input: Vec<f32> = vec![];

        let output = engine.process_audio_buffer(&input);

        assert_eq!(output.len(), 0);
    }

    #[test]
    fn test_large_buffer_processing() {
        let mut engine = AudioEngine::new(44100.0, 1024);
        let input = vec![0.5; 2048]; // Larger than configured buffer size

        let output = engine.process_audio_buffer(&input);

        assert_eq!(output.len(), 2048);
        assert!(output.iter().all(|&x| x == 0.4)); // 0.5 * 0.8
    }
}
