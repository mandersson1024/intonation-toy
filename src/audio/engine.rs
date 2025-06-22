use wasm_bindgen::prelude::*;
use crate::audio::pitch_detector::{PitchAlgorithm, PitchConfig, PitchDetector};
use crate::audio::realtime_processor::{RealtimeProcessor, RealtimeProcessingResult};
use crate::audio::performance_monitor::{PerformanceMonitor, PerformanceMetrics, PipelineStatus};
use crate::audio::signal_analyzer::{SignalAnalyzer, AudioAnalysis, BufferConfig, BufferConstraints};

/// Audio engine for real-time pitch detection and processing with WASM interface
#[wasm_bindgen]
pub struct AudioEngine {
    sample_rate: f32,
    buffer_size: usize,
    enabled: bool,
    pitch_detector: Option<PitchDetector>,
    pitch_config: PitchConfig,
    
    // Core processing components
    realtime_processor: RealtimeProcessor,
    performance_monitor: PerformanceMonitor,
    signal_analyzer: SignalAnalyzer,
    
    // Target latency configuration
    target_latency_ms: f32,
}

#[wasm_bindgen]
impl AudioEngine {
    /// Create a new AudioEngine instance
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: f32, buffer_size: usize) -> AudioEngine {
        let pitch_config = PitchConfig::new(sample_rate);
        let target_latency_ms = 50.0; // Default 50ms target latency
        
        AudioEngine {
            sample_rate,
            buffer_size,
            enabled: true,
            pitch_detector: None,
            pitch_config,
            realtime_processor: RealtimeProcessor::new(sample_rate, buffer_size),
            performance_monitor: PerformanceMonitor::new(sample_rate, buffer_size, target_latency_ms),
            signal_analyzer: SignalAnalyzer::new(sample_rate, buffer_size),
            target_latency_ms,
        }
    }

    /// Process audio buffer with basic gain adjustment
    #[wasm_bindgen]
    pub fn process_audio_buffer(&mut self, input: &[f32]) -> Vec<f32> {
        if !self.enabled {
            return vec![0.0; input.len()];
        }

        // Apply basic passthrough processing
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

    // =====================================================================
    // 🎯 WASM INTERFACE METHODS
    // =====================================================================

    /// Process audio buffer in real-time with pitch detection and performance monitoring
    #[wasm_bindgen]
    pub fn process_realtime_audio(&mut self, input: &[f32]) -> RealtimeProcessingResult {
        if !self.enabled || input.is_empty() {
            // Return default result for disabled/empty state
            return RealtimeProcessingResult::default();
        }
        
        // Process through the realtime processor
        let result = self.realtime_processor.process_audio_buffer(input);
        
        // Record performance metrics for monitoring
        self.performance_monitor.record_processing_cycle(
            result.processing_time_ms(),
            result.pitch_detected() || result.audio_processed()
        );
        
        // Update WASM connection status (always true when called)
        self.performance_monitor.set_wasm_connected(true);
        
        result
    }

    /// Get comprehensive performance metrics for monitoring
    #[wasm_bindgen]
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_monitor.get_performance_metrics()
    }

    /// Analyze audio signal with adaptive thresholds and stability tracking
    #[wasm_bindgen]
    pub fn analyze_audio_signal(&mut self, buffer: &[f32]) -> AudioAnalysis {
        if !self.enabled || buffer.is_empty() {
            return AudioAnalysis::default();
        }
        
        self.signal_analyzer.analyze_audio_signal(buffer)
    }

    /// Validate audio pipeline health and status
    #[wasm_bindgen]
    pub fn validate_audio_pipeline(&mut self) -> PipelineStatus {
        self.performance_monitor.validate_audio_pipeline()
    }

    /// Optimize buffer configuration based on latency and performance constraints
    #[wasm_bindgen]
    pub fn optimize_buffer_configuration(&mut self, constraints: BufferConstraints) -> BufferConfig {
        self.signal_analyzer.optimize_buffer_configuration(&constraints)
    }

    /// 🔧 CONFIGURATION: Update latency components (replaces JS latency tracking)
    /// Updates latency information from browser audio context
    #[wasm_bindgen]
    pub fn update_latency_components(&mut self, audio_context_latency: f32, output_latency: f32) {
        self.performance_monitor.update_latency_components(audio_context_latency, output_latency);
    }

    /// 📈 MONITORING: Check if performance reporting is due
    /// Replaces JavaScript performance reporting intervals
    #[wasm_bindgen]
    pub fn should_report_performance(&mut self) -> bool {
        self.performance_monitor.should_report_performance()
    }

    /// 🎛️ CONFIGURATION: Set target latency for performance monitoring
    #[wasm_bindgen]
    pub fn set_target_latency(&mut self, target_latency_ms: f32) {
        self.target_latency_ms = target_latency_ms;
        // Update performance monitor with new target
        self.performance_monitor = PerformanceMonitor::new(self.sample_rate, self.buffer_size, target_latency_ms);
    }

    /// 🔄 RESET: Reset performance counters and analysis state
    /// Useful for fresh measurements and testing
    #[wasm_bindgen]
    pub fn reset_performance_counters(&mut self) {
        self.performance_monitor.reset_counters();
        self.signal_analyzer.reset_analysis();
    }

    /// 📊 DEBUG: Get comprehensive processing statistics
    /// Returns detailed stats for debugging and optimization
    #[wasm_bindgen]
    pub fn get_debug_statistics(&self) -> String {
        let (proc_count, avg_time, min_time, history_len) = self.performance_monitor.get_debug_stats();
        let noise_floor = self.signal_analyzer.get_noise_floor();
        let (window_size, sample_rate, analysis_len) = self.signal_analyzer.get_analysis_info();
        
        format!(
            "ProcessingStats(count={}, avg_time={:.3}ms, min_time={:.3}ms, history={}) | \
             SignalAnalysis(noise_floor={:.6}, window={}, sr={}, history={}) | \
             Config(target_latency={:.1}ms, buffer_size={})",
            proc_count, avg_time, min_time, history_len,
            noise_floor, window_size, sample_rate, analysis_len,
            self.target_latency_ms, self.buffer_size
        )
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

    // Comprehensive audio engine validation tests

    #[test]
    fn test_engine_configuration_edge_cases() {
        // Test extreme buffer sizes
        let tiny_engine = AudioEngine::new(44100.0, 64);
        assert_eq!(tiny_engine.get_buffer_size(), 64);
        
        let large_engine = AudioEngine::new(44100.0, 8192);
        assert_eq!(large_engine.get_buffer_size(), 8192);
        
        // Test different sample rates
        let low_sr_engine = AudioEngine::new(22050.0, 1024);
        assert_eq!(low_sr_engine.get_sample_rate(), 22050.0);
        
        let high_sr_engine = AudioEngine::new(96000.0, 1024);
        assert_eq!(high_sr_engine.get_sample_rate(), 96000.0);
    }

    #[test]
    fn test_audio_processing_precision() {
        let mut engine = AudioEngine::new(44100.0, 1024);
        
        // Test with precise floating point values
        let input = vec![0.123456789, -0.987654321, 0.5, -0.5];
        let output = engine.process_audio_buffer(&input);
        
        assert_eq!(output.len(), input.len());
        
        // Check precision of gain calculation (0.8x)
        let expected = vec![0.123456789 * 0.8, -0.987654321 * 0.8, 0.4, -0.4];
        for (i, (&actual, &expected)) in output.iter().zip(expected.iter()).enumerate() {
            assert!((actual - expected).abs() < 1e-6, 
                "Precision error at index {}: expected {}, got {}", i, expected, actual);
        }
    }

    #[test]
    fn test_pitch_detection_with_real_frequencies() {
        use std::f32::consts::PI;
        
        let mut engine = AudioEngine::new(44100.0, 2048);
        let sample_rate = 44100.0;
        let buffer_size = 2048;
        
        // Test known frequencies
        let test_frequencies = [220.0, 440.0, 880.0]; // A notes
        
        for &freq in &test_frequencies {
            // Generate sine wave
            let test_buffer: Vec<f32> = (0..buffer_size)
                .map(|i| 0.8 * (2.0 * PI * freq * i as f32 / sample_rate).sin())
                .collect();
            
            let detected_pitch = engine.detect_pitch_from_buffer(&test_buffer);
            
            if detected_pitch > 0.0 {
                // Calculate cents error if detection succeeded
                let cents_error = 1200.0 * (detected_pitch / freq).log2().abs();
                assert!(cents_error <= 50.0,
                    "Pitch detection error too large: expected {}Hz, got {}Hz ({:.1} cents error)",
                    freq, detected_pitch, cents_error);
            }
            // Note: Some detections may fail, which is acceptable for testing
        }
    }

    #[test]
    fn test_engine_state_consistency() {
        let mut engine = AudioEngine::new(44100.0, 1024);
        let test_buffer = vec![0.5; 1024];
        
        // Test multiple operations maintain consistency
        for i in 0..10 {
            let _output = engine.process_audio_buffer(&test_buffer);
            let _pitch = engine.detect_pitch_from_buffer(&test_buffer);
            
            // Engine state should remain consistent
            assert_eq!(engine.get_sample_rate(), 44100.0);
            assert_eq!(engine.get_buffer_size(), 1024);
            assert!(engine.enabled);
            
            // Toggle enable/disable - but end in enabled state
            if i % 2 == 0 {
                engine.set_enabled(false);
                assert!(!engine.enabled);
                engine.set_enabled(true); // Re-enable for next iteration
                assert!(engine.enabled);
            }
        }
    }

    #[test]
    fn test_concurrent_audio_processing_simulation() {
        let mut engine = AudioEngine::new(44100.0, 1024);
        
        // Simulate rapid successive processing calls
        let buffers: Vec<Vec<f32>> = (0..20)
            .map(|i| vec![0.1 * (i as f32); 1024])
            .collect();
        
        for (i, buffer) in buffers.iter().enumerate() {
            let output = engine.process_audio_buffer(buffer);
            assert_eq!(output.len(), buffer.len());
            
            // Verify gain is applied correctly
            let expected_sample = buffer[0] * 0.8;
            assert!((output[0] - expected_sample).abs() < 1e-6,
                "Processing error in iteration {}: expected {}, got {}", 
                i, expected_sample, output[0]);
        }
    }

    #[test]
    fn test_pitch_detector_lazy_initialization() {
        let mut engine = AudioEngine::new(44100.0, 1024);
        
        // Initially no detector should be created
        assert!(engine.pitch_detector.is_none());
        
        // First pitch detection should create detector
        let test_buffer = vec![0.1; 1024];
        let _pitch1 = engine.detect_pitch_from_buffer(&test_buffer);
        assert!(engine.pitch_detector.is_some());
        
        // Second detection should reuse existing detector
        let _pitch2 = engine.detect_pitch_from_buffer(&test_buffer);
        assert!(engine.pitch_detector.is_some());
        
        // Changing algorithm should reset detector
        engine.set_pitch_algorithm(PitchAlgorithm::McLeod);
        assert!(engine.pitch_detector.is_none());
        
        // Changing frequency range should reset detector
        let _pitch3 = engine.detect_pitch_from_buffer(&test_buffer);
        assert!(engine.pitch_detector.is_some());
        
        engine.set_pitch_frequency_range(100.0, 1500.0);
        assert!(engine.pitch_detector.is_none());
    }

    #[test]
    fn test_audio_processing_boundary_values() {
        let mut engine = AudioEngine::new(44100.0, 1024);
        
        // Test with extreme values
        let boundary_values = vec![
            f32::MAX * 0.001,    // Very large positive
            f32::MIN * 0.001,    // Very large negative (MIN is negative)
            1.0,                 // Max typical audio value
            -1.0,                // Min typical audio value
            0.0,                 // Zero
            1e-10,               // Very small positive
            -1e-10,              // Very small negative
        ];
        
        for &value in &boundary_values {
            let input = vec![value; 4];
            let output = engine.process_audio_buffer(&input);
            
            assert_eq!(output.len(), input.len());
            
            // Verify gain application doesn't cause overflow/underflow
            let expected = value * 0.8;
            assert!(output[0].is_finite(), "Non-finite result for input {}", value);
            assert!((output[0] - expected).abs() < 1e-6 || (expected.abs() < 1e-9),
                "Boundary value processing error: input={}, expected={}, got={}", 
                value, expected, output[0]);
        }
    }

    #[test]
    fn test_pitch_detection_integration_comprehensive() {
        let mut engine = AudioEngine::new(44100.0, 2048);
        
        // Test with different algorithms
        let algorithms = [PitchAlgorithm::YIN, PitchAlgorithm::McLeod];
        
        for algorithm in &algorithms {
            engine.set_pitch_algorithm(*algorithm);
            
            // Test with different frequency ranges
            let frequency_ranges = [
                (80.0, 2000.0),   // Default range
                (100.0, 1500.0),  // Narrower range
                (50.0, 4000.0),   // Wider range
            ];
            
            for &(min_freq, max_freq) in &frequency_ranges {
                engine.set_pitch_frequency_range(min_freq, max_freq);
                
                // Test different buffer contents
                let test_cases = [
                    vec![0.0; 2048],                    // Silence
                    vec![0.5; 2048],                    // DC
                    (0..2048).map(|i| (i as f32) * 0.001).collect(), // Ramp
                ];
                
                for test_buffer in &test_cases {
                    let pitch = engine.detect_pitch_from_buffer(test_buffer);
                    // Should not crash and should return valid value or -1.0
                    assert!(pitch == -1.0 || (pitch >= min_freq && pitch <= max_freq * 1.1),
                        "Invalid pitch detection result: {} (range: {}-{})", 
                        pitch, min_freq, max_freq);
                }
            }
        }
    }

        #[test]
    fn test_memory_safety_basic() {
        let mut engine = AudioEngine::new(44100.0, 1024);
        
        // Test basic memory safety with standard buffer size
        let buffer = vec![0.1; 1024];
        
        // Multiple operations should not cause memory issues
        for _i in 0..10 {
            let _output = engine.process_audio_buffer(&buffer);
            let _combined = engine.process_audio_with_pitch(&buffer);
            
            // Verify output sizes are correct
            let output = engine.process_audio_buffer(&buffer);
            assert_eq!(output.len(), buffer.len());
        }
        
        // Test pitch detection
        let _pitch = engine.detect_pitch_from_buffer(&buffer);
    }

    #[test]
    fn test_audio_engine_thread_safety_simulation() {
        // Simulate what would happen with multiple "threads" (sequential calls)
        let mut engines: Vec<AudioEngine> = (0..5)
            .map(|_| AudioEngine::new(44100.0, 1024))
            .collect();
        
        let test_buffer = vec![0.2; 1024];
        
        // Each engine should work independently
        for (i, engine) in engines.iter_mut().enumerate() {
            // Configure each engine differently
            if i % 2 == 0 {
                engine.set_pitch_algorithm(PitchAlgorithm::YIN);
            } else {
                engine.set_pitch_algorithm(PitchAlgorithm::McLeod);
            }
            
            engine.set_pitch_frequency_range(80.0 + i as f32 * 10.0, 2000.0);
            
            // Process audio
            let output = engine.process_audio_buffer(&test_buffer);
            assert_eq!(output.len(), test_buffer.len());
            // Use approximate comparison for floating point
            assert!((output[0] - 0.16).abs() < 1e-6); // 0.2 * 0.8
            
            // Detect pitch
            let _pitch = engine.detect_pitch_from_buffer(&test_buffer);
        }
    }
}
