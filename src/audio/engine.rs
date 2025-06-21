use wasm_bindgen::prelude::*;
use crate::audio::pitch_detector::{PitchAlgorithm, PitchConfig, PitchDetector};

/// Core audio engine for real-time pitch detection and processing
#[wasm_bindgen]
pub struct AudioEngine {
    sample_rate: f32,
    buffer_size: usize,
    enabled: bool,
    pitch_detector: Option<PitchDetector>,
    pitch_config: PitchConfig,
}

#[wasm_bindgen]
impl AudioEngine {
    /// Create a new AudioEngine instance
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: f32, buffer_size: usize) -> AudioEngine {
        let pitch_config = PitchConfig::new(sample_rate);
        AudioEngine {
            sample_rate,
            buffer_size,
            enabled: true,
            pitch_detector: None,
            pitch_config,
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

    /// Set pitch detection algorithm
    #[wasm_bindgen]
    pub fn set_pitch_algorithm(&mut self, algorithm: PitchAlgorithm) {
        self.pitch_config.set_algorithm(algorithm);
        // Reset detector to apply new configuration
        self.pitch_detector = None;
    }

    /// Configure pitch detection frequency range
    #[wasm_bindgen]
    pub fn set_pitch_frequency_range(&mut self, min_freq: f32, max_freq: f32) {
        self.pitch_config.set_frequency_range(min_freq, max_freq);
        // Reset detector to apply new configuration
        self.pitch_detector = None;
    }

    /// Detect pitch from audio buffer
    #[wasm_bindgen]
    pub fn detect_pitch_from_buffer(&mut self, audio_buffer: &[f32]) -> f32 {
        if !self.enabled || audio_buffer.is_empty() {
            return -1.0;
        }

        // Initialize detector if needed
        if self.pitch_detector.is_none() {
            self.pitch_detector = Some(PitchDetector::new(self.pitch_config.clone()));
        }

        if let Some(ref mut detector) = self.pitch_detector {
            match detector.detect_pitch(audio_buffer) {
                Some(result) if result.is_valid() => result.frequency(),
                _ => -1.0, // Invalid or no pitch detected
            }
        } else {
            -1.0
        }
    }

    /// Process audio buffer with pitch detection
    #[wasm_bindgen]
    pub fn process_audio_with_pitch(&mut self, input: &[f32]) -> Vec<f32> {
        if !self.enabled {
            return vec![0.0; input.len()];
        }

        // Process audio (basic passthrough for now)
        let mut output = input.to_vec();
        
        // Apply basic gain for testing
        for sample in &mut output {
            *sample *= 0.8;
        }

        // Detect pitch for feedback (doesn't affect audio output)
        let _pitch = self.detect_pitch_from_buffer(input);
        
        output
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
        assert!(engine.pitch_detector.is_none()); // Initially None
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

    #[test]
    fn test_pitch_detection_integration() {
        let mut engine = AudioEngine::new(44100.0, 1024);
        
        // Test empty buffer
        let empty_buffer: Vec<f32> = vec![];
        let pitch = engine.detect_pitch_from_buffer(&empty_buffer);
        assert_eq!(pitch, -1.0);
        
        // Test with test buffer (won't detect real pitch but should not crash)
        let test_buffer = vec![0.1; 1024];
        let pitch = engine.detect_pitch_from_buffer(&test_buffer);
        // Should return -1.0 (no pitch detected) or a frequency value
        assert!(pitch == -1.0 || pitch >= 80.0);
    }

    #[test]
    fn test_pitch_algorithm_configuration() {
        let mut engine = AudioEngine::new(44100.0, 1024);
        
        // Test algorithm switching
        engine.set_pitch_algorithm(PitchAlgorithm::McLeod);
        // Should not crash - detector gets reset
        let test_buffer = vec![0.1; 1024];
        let _pitch = engine.detect_pitch_from_buffer(&test_buffer);
        
        engine.set_pitch_algorithm(PitchAlgorithm::YIN);
        let _pitch = engine.detect_pitch_from_buffer(&test_buffer);
    }

    #[test]
    fn test_pitch_frequency_range_configuration() {
        let mut engine = AudioEngine::new(44100.0, 1024);
        
        // Test frequency range configuration
        engine.set_pitch_frequency_range(100.0, 1500.0);
        // Should not crash - detector gets reset
        let test_buffer = vec![0.1; 1024];
        let _pitch = engine.detect_pitch_from_buffer(&test_buffer);
    }

    #[test]
    fn test_process_audio_with_pitch() {
        let mut engine = AudioEngine::new(44100.0, 1024);
        let input = vec![0.5, 0.25, -0.25, -0.5];

        let output = engine.process_audio_with_pitch(&input);

        assert_eq!(output.len(), input.len());
        // Gain reduction of 0.8 still applies
        assert_eq!(output[0], 0.4);
        assert_eq!(output[1], 0.2);
        assert_eq!(output[2], -0.2);
        assert_eq!(output[3], -0.4);
    }

    #[test]
    fn test_disabled_engine_pitch_detection() {
        let mut engine = AudioEngine::new(44100.0, 1024);
        engine.set_enabled(false);

        let test_buffer = vec![0.1; 1024];
        let pitch = engine.detect_pitch_from_buffer(&test_buffer);
        
        assert_eq!(pitch, -1.0); // Should return -1.0 when disabled
    }
}
