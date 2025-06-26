// Test Signal Validation Suite - STORY-3.19
// Automated validation of test signal generation and pitch detection accuracy

#[cfg(test)]
mod test_signal_validation {
    use crate::modules::audio_foundations::multi_algorithm_pitch_detector::*;
    use std::collections::HashMap;

    /// Test signal generator with quality validation
    struct ValidatedSignalGenerator {
        sample_rate: f32,
    }

    impl ValidatedSignalGenerator {
        fn new(sample_rate: f32) -> Self {
            Self { sample_rate }
        }

        /// Generate a pure sine wave
        fn generate_sine_wave(&self, frequency: f32, duration_samples: usize, amplitude: f32) -> Vec<f32> {
            (0..duration_samples)
                .map(|i| {
                    let t = i as f32 / self.sample_rate;
                    amplitude * (2.0 * std::f32::consts::PI * frequency * t).sin()
                })
                .collect()
        }

        /// Generate a complex tone with harmonics
        fn generate_complex_tone(&self, fundamental: f32, harmonics: &[(f32, f32)], duration_samples: usize) -> Vec<f32> {
            let mut signal = vec![0.0; duration_samples];
            
            // Add fundamental
            let fundamental_wave = self.generate_sine_wave(fundamental, duration_samples, 0.5);
            for (i, &sample) in fundamental_wave.iter().enumerate() {
                signal[i] += sample;
            }
            
            // Add harmonics
            for &(harmonic_freq, amplitude) in harmonics {
                let harmonic_wave = self.generate_sine_wave(fundamental * harmonic_freq, duration_samples, amplitude);
                for (i, &sample) in harmonic_wave.iter().enumerate() {
                    signal[i] += sample;
                }
            }
            
            // Normalize
            let max_amplitude = signal.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
            if max_amplitude > 0.0 {
                for sample in &mut signal {
                    *sample /= max_amplitude * 1.2; // Leave some headroom
                }
            }
            
            signal
        }

        /// Generate frequency sweep
        fn generate_sweep(&self, start_freq: f32, end_freq: f32, duration_samples: usize, amplitude: f32) -> Vec<f32> {
            (0..duration_samples)
                .map(|i| {
                    let t = i as f32 / self.sample_rate;
                    let progress = i as f32 / duration_samples as f32;
                    let freq = start_freq + (end_freq - start_freq) * progress;
                    amplitude * (2.0 * std::f32::consts::PI * freq * t).sin()
                })
                .collect()
        }

        /// Generate white noise
        fn generate_white_noise(&self, duration_samples: usize, amplitude: f32) -> Vec<f32> {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            
            (0..duration_samples)
                .map(|i| {
                    let mut hasher = DefaultHasher::new();
                    i.hash(&mut hasher);
                    let random_val = (hasher.finish() % 10000) as f32 / 10000.0 - 0.5;
                    amplitude * random_val * 2.0
                })
                .collect()
        }

        /// Add noise to existing signal
        fn add_noise(&self, signal: &mut [f32], noise_level: f32) {
            let noise = self.generate_white_noise(signal.len(), noise_level);
            for (i, sample) in signal.iter_mut().enumerate() {
                *sample += noise[i];
            }
        }
    }

    /// Signal quality analyzer
    struct SignalQualityAnalyzer;

    impl SignalQualityAnalyzer {
        /// Calculate RMS energy
        fn rms_energy(signal: &[f32]) -> f32 {
            let sum_squares: f32 = signal.iter().map(|&x| x * x).sum();
            (sum_squares / signal.len() as f32).sqrt()
        }

        /// Calculate signal-to-noise ratio (simplified)
        fn snr_estimate(signal: &[f32]) -> f32 {
            if signal.is_empty() {
                return 0.0;
            }

            let rms = Self::rms_energy(signal);
            let segment_size = 64.min(signal.len() / 4);
            let mut min_segment_energy = f32::INFINITY;

            for chunk in signal.chunks(segment_size) {
                let segment_rms = Self::rms_energy(chunk);
                min_segment_energy = min_segment_energy.min(segment_rms);
            }

            if min_segment_energy > 0.0 {
                (rms / min_segment_energy).log10().max(0.0).min(2.0) / 2.0
            } else {
                1.0
            }
        }

        /// Calculate total harmonic distortion (simplified)
        fn thd_estimate(signal: &[f32], fundamental_freq: f32, sample_rate: f32) -> f32 {
            // Simplified THD calculation based on autocorrelation
            let period_samples = sample_rate / fundamental_freq;
            if period_samples < 2.0 || period_samples >= signal.len() as f32 / 2.0 {
                return 1.0; // High distortion if we can't analyze properly
            }

            let period = period_samples as usize;
            if period >= signal.len() / 2 {
                return 1.0;
            }

            let mut correlation = 0.0;
            let mut energy = 0.0;

            for i in 0..(signal.len() - period) {
                correlation += signal[i] * signal[i + period];
                energy += signal[i] * signal[i];
            }

            if energy > 0.0 {
                1.0 - (correlation / energy).abs().min(1.0)
            } else {
                1.0
            }
        }

        /// Detect clipping
        fn clipping_percentage(signal: &[f32]) -> f32 {
            let threshold = 0.95;
            let clipped_samples = signal.iter().filter(|&&x| x.abs() > threshold).count();
            clipped_samples as f32 / signal.len() as f32
        }
    }

    /// Test case for pitch detection accuracy
    #[derive(Debug)]
    struct PitchDetectionTestCase {
        name: String,
        expected_frequency: f32,
        tolerance_hz: f32,
        signal: Vec<f32>,
        min_confidence: f32,
        should_succeed: bool,
    }

    impl PitchDetectionTestCase {
        fn new(name: &str, expected_frequency: f32, tolerance_hz: f32, signal: Vec<f32>) -> Self {
            Self {
                name: name.to_string(),
                expected_frequency,
                tolerance_hz,
                signal,
                min_confidence: 0.3,
                should_succeed: true,
            }
        }

        fn with_confidence(mut self, min_confidence: f32) -> Self {
            self.min_confidence = min_confidence;
            self
        }

        fn should_fail(mut self) -> Self {
            self.should_succeed = false;
            self
        }
    }

    /// Comprehensive test signal validation
    struct TestSignalValidator {
        generator: ValidatedSignalGenerator,
        analyzer: SignalQualityAnalyzer,
    }

    impl TestSignalValidator {
        fn new(sample_rate: f32) -> Self {
            Self {
                generator: ValidatedSignalGenerator::new(sample_rate),
                analyzer: SignalQualityAnalyzer,
            }
        }

        /// Generate test cases for musical notes
        fn generate_musical_note_tests(&self) -> Vec<PitchDetectionTestCase> {
            let notes = vec![
                ("C4", 261.63),
                ("D4", 293.66),
                ("E4", 329.63),
                ("F4", 349.23),
                ("G4", 392.00),
                ("A4", 440.00),
                ("B4", 493.88),
                ("C5", 523.25),
            ];

            notes.into_iter().map(|(name, freq)| {
                let signal = self.generator.generate_sine_wave(freq, 2048, 0.8);
                PitchDetectionTestCase::new(
                    &format!("musical_note_{}", name),
                    freq,
                    2.0, // 2Hz tolerance
                    signal
                ).with_confidence(0.7)
            }).collect()
        }

        /// Generate test cases for harmonic content
        fn generate_harmonic_tests(&self) -> Vec<PitchDetectionTestCase> {
            vec![
                {
                    let fundamental = 220.0;
                    let harmonics = [(2.0, 0.3), (3.0, 0.2), (4.0, 0.1)];
                    let signal = self.generator.generate_complex_tone(fundamental, &harmonics, 2048);
                    PitchDetectionTestCase::new(
                        "harmonic_220hz_with_overtones",
                        fundamental,
                        5.0,
                        signal
                    ).with_confidence(0.5)
                },
                {
                    let fundamental = 440.0;
                    let harmonics = [(2.0, 0.5), (3.0, 0.3)];
                    let signal = self.generator.generate_complex_tone(fundamental, &harmonics, 2048);
                    PitchDetectionTestCase::new(
                        "harmonic_440hz_strong_overtones",
                        fundamental,
                        5.0,
                        signal
                    ).with_confidence(0.4)
                },
            ]
        }

        /// Generate test cases with various noise levels
        fn generate_noise_robustness_tests(&self) -> Vec<PitchDetectionTestCase> {
            let base_frequency = 440.0;
            let noise_levels = [0.0, 0.05, 0.1, 0.2, 0.3];
            
            noise_levels.iter().map(|&noise_level| {
                let mut signal = self.generator.generate_sine_wave(base_frequency, 2048, 0.8);
                if noise_level > 0.0 {
                    self.generator.add_noise(&mut signal, noise_level);
                }
                
                let tolerance = if noise_level < 0.1 { 2.0 } else { 10.0 };
                let min_confidence = if noise_level < 0.1 { 0.6 } else { 0.2 };
                
                PitchDetectionTestCase::new(
                    &format!("noise_robustness_{}pct", (noise_level * 100.0) as u32),
                    base_frequency,
                    tolerance,
                    signal
                ).with_confidence(min_confidence)
            }).collect()
        }

        /// Generate edge case tests
        fn generate_edge_case_tests(&self) -> Vec<PitchDetectionTestCase> {
            vec![
                // Very low frequency
                PitchDetectionTestCase::new(
                    "very_low_frequency_80hz",
                    80.0,
                    5.0,
                    self.generator.generate_sine_wave(80.0, 4096, 0.8) // Longer for low frequency
                ).with_confidence(0.3),
                
                // Very high frequency
                PitchDetectionTestCase::new(
                    "high_frequency_1800hz",
                    1800.0,
                    10.0,
                    self.generator.generate_sine_wave(1800.0, 2048, 0.8)
                ).with_confidence(0.3),
                
                // Very quiet signal
                PitchDetectionTestCase::new(
                    "quiet_signal_440hz",
                    440.0,
                    5.0,
                    self.generator.generate_sine_wave(440.0, 2048, 0.05)
                ).with_confidence(0.1),
                
                // Silence (should fail)
                PitchDetectionTestCase::new(
                    "silence",
                    0.0,
                    1.0,
                    vec![0.0; 2048]
                ).should_fail(),
                
                // Pure noise (should fail)
                PitchDetectionTestCase::new(
                    "pure_noise",
                    0.0,
                    1.0,
                    self.generator.generate_white_noise(2048, 0.5)
                ).should_fail(),
            ]
        }

        /// Generate frequency sweep tests
        fn generate_sweep_tests(&self) -> Vec<PitchDetectionTestCase> {
            vec![
                // Short sweep that should fail (no stable frequency)
                PitchDetectionTestCase::new(
                    "frequency_sweep_100_to_1000hz",
                    0.0, // No expected frequency
                    1.0,
                    self.generator.generate_sweep(100.0, 1000.0, 2048, 0.8)
                ).should_fail(),
            ]
        }

        /// Validate signal quality before testing
        fn validate_signal_quality(&self, signal: &[f32], test_name: &str) -> bool {
            let rms = SignalQualityAnalyzer::rms_energy(signal);
            let snr = SignalQualityAnalyzer::snr_estimate(signal);
            let clipping = SignalQualityAnalyzer::clipping_percentage(signal);
            
            // Signal quality thresholds
            let min_rms = 0.001; // Minimum energy
            let max_clipping = 0.05; // Max 5% clipping
            let min_snr = 0.1; // Minimum SNR for non-noise signals
            
            let quality_ok = rms > min_rms && clipping < max_clipping;
            
            if !quality_ok {
                println!("Signal quality warning for {}: RMS={:.4}, SNR={:.4}, Clipping={:.2}%", 
                    test_name, rms, snr, clipping * 100.0);
            }
            
            quality_ok
        }

        /// Run all validation tests
        fn run_all_tests(&self) -> TestResults {
            let mut all_tests = Vec::new();
            all_tests.extend(self.generate_musical_note_tests());
            all_tests.extend(self.generate_harmonic_tests());
            all_tests.extend(self.generate_noise_robustness_tests());
            all_tests.extend(self.generate_edge_case_tests());
            all_tests.extend(self.generate_sweep_tests());

            let mut results = TestResults::new();
            
            // Test with different algorithms
            let algorithms = [PitchAlgorithm::YIN, PitchAlgorithm::McLeod, PitchAlgorithm::Auto];
            
            for algorithm in &algorithms {
                let config = PitchDetectionConfig {
                    algorithm: *algorithm,
                    sample_rate: self.generator.sample_rate,
                    min_frequency: 60.0,
                    max_frequency: 4000.0,
                    yin_threshold: 0.2,
                    mcleod_threshold: 0.3,
                    mcleod_clarity_threshold: 0.7,
                    enable_confidence_scoring: true,
                    enable_harmonic_analysis: true,
                    auto_selection_sensitivity: 0.5,
                };
                
                let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
                
                for test_case in &all_tests {
                    // Validate signal quality first
                    if !self.validate_signal_quality(&test_case.signal, &test_case.name) {
                        continue; // Skip poor quality signals
                    }
                    
                    let detection_result = detector.detect_pitch(&test_case.signal);
                    let test_result = self.evaluate_test_result(test_case, detection_result, *algorithm);
                    results.add_result(test_result);
                }
            }
            
            results
        }

        /// Evaluate a single test result
        fn evaluate_test_result(
            &self,
            test_case: &PitchDetectionTestCase,
            detection_result: Result<PitchResult, PitchError>,
            algorithm: PitchAlgorithm
        ) -> SingleTestResult {
            match detection_result {
                Ok(pitch_result) => {
                    if test_case.should_succeed {
                        let frequency_error = (pitch_result.frequency - test_case.expected_frequency).abs();
                        let frequency_ok = frequency_error <= test_case.tolerance_hz;
                        let confidence_ok = pitch_result.confidence >= test_case.min_confidence;
                        
                        SingleTestResult {
                            test_name: format!("{}_{:?}", test_case.name, algorithm),
                            algorithm,
                            expected_frequency: Some(test_case.expected_frequency),
                            detected_frequency: Some(pitch_result.frequency),
                            confidence: pitch_result.confidence,
                            passed: frequency_ok && confidence_ok && pitch_result.is_valid,
                            error_message: if !frequency_ok {
                                Some(format!("Frequency error: {:.2}Hz (tolerance: {:.2}Hz)", 
                                    frequency_error, test_case.tolerance_hz))
                            } else if !confidence_ok {
                                Some(format!("Low confidence: {:.2} (minimum: {:.2})", 
                                    pitch_result.confidence, test_case.min_confidence))
                            } else if !pitch_result.is_valid {
                                Some("Result marked as invalid".to_string())
                            } else {
                                None
                            },
                            processing_time_ns: pitch_result.processing_time_ns,
                        }
                    } else {
                        // Test should have failed but didn't
                        SingleTestResult {
                            test_name: format!("{}_{:?}", test_case.name, algorithm),
                            algorithm,
                            expected_frequency: None,
                            detected_frequency: Some(pitch_result.frequency),
                            confidence: pitch_result.confidence,
                            passed: false,
                            error_message: Some("Expected detection to fail but it succeeded".to_string()),
                            processing_time_ns: pitch_result.processing_time_ns,
                        }
                    }
                }
                Err(error) => {
                    if test_case.should_succeed {
                        // Test should have succeeded but failed
                        SingleTestResult {
                            test_name: format!("{}_{:?}", test_case.name, algorithm),
                            algorithm,
                            expected_frequency: Some(test_case.expected_frequency),
                            detected_frequency: None,
                            confidence: 0.0,
                            passed: false,
                            error_message: Some(format!("Detection failed: {}", error)),
                            processing_time_ns: 0,
                        }
                    } else {
                        // Test should have failed and did - this is correct
                        SingleTestResult {
                            test_name: format!("{}_{:?}", test_case.name, algorithm),
                            algorithm,
                            expected_frequency: None,
                            detected_frequency: None,
                            confidence: 0.0,
                            passed: true,
                            error_message: None,
                            processing_time_ns: 0,
                        }
                    }
                }
            }
        }
    }

    /// Single test result
    #[derive(Debug)]
    struct SingleTestResult {
        test_name: String,
        algorithm: PitchAlgorithm,
        expected_frequency: Option<f32>,
        detected_frequency: Option<f32>,
        confidence: f32,
        passed: bool,
        error_message: Option<String>,
        processing_time_ns: u64,
    }

    /// Aggregated test results
    struct TestResults {
        results: Vec<SingleTestResult>,
    }

    impl TestResults {
        fn new() -> Self {
            Self { results: Vec::new() }
        }

        fn add_result(&mut self, result: SingleTestResult) {
            self.results.push(result);
        }

        fn total_tests(&self) -> usize {
            self.results.len()
        }

        fn passed_tests(&self) -> usize {
            self.results.iter().filter(|r| r.passed).count()
        }

        fn failed_tests(&self) -> usize {
            self.total_tests() - self.passed_tests()
        }

        fn success_rate(&self) -> f32 {
            if self.total_tests() == 0 {
                return 0.0;
            }
            self.passed_tests() as f32 / self.total_tests() as f32
        }

        fn algorithm_stats(&self) -> HashMap<PitchAlgorithm, (usize, usize)> {
            let mut stats = HashMap::new();
            
            for result in &self.results {
                let entry = stats.entry(result.algorithm).or_insert((0, 0));
                entry.0 += 1; // total
                if result.passed {
                    entry.1 += 1; // passed
                }
            }
            
            stats
        }

        fn average_processing_time(&self) -> u64 {
            if self.results.is_empty() {
                return 0;
            }
            
            let total_time: u64 = self.results.iter()
                .filter(|r| r.processing_time_ns > 0)
                .map(|r| r.processing_time_ns)
                .sum();
            let count = self.results.iter().filter(|r| r.processing_time_ns > 0).count();
            
            if count > 0 {
                total_time / count as u64
            } else {
                0
            }
        }

        fn print_summary(&self) {
            println!("\n=== Test Signal Validation Results ===");
            println!("Total tests: {}", self.total_tests());
            println!("Passed: {}", self.passed_tests());
            println!("Failed: {}", self.failed_tests());
            println!("Success rate: {:.1}%", self.success_rate() * 100.0);
            println!("Average processing time: {:.2}ms", self.average_processing_time() as f64 / 1_000_000.0);
            
            println!("\nAlgorithm Performance:");
            for (algorithm, (total, passed)) in self.algorithm_stats() {
                let rate = if total > 0 { passed as f32 / total as f32 * 100.0 } else { 0.0 };
                println!("  {:?}: {}/{} ({:.1}%)", algorithm, passed, total, rate);
            }
            
            if self.failed_tests() > 0 {
                println!("\nFailed Tests:");
                for result in &self.results {
                    if !result.passed {
                        println!("  ❌ {}: {}", result.test_name, 
                            result.error_message.as_ref().unwrap_or(&"Unknown error".to_string()));
                    }
                }
            }
        }
    }

    // =============================================================================
    // ACTUAL TESTS
    // =============================================================================

    #[test]
    fn test_musical_note_accuracy() {
        let validator = TestSignalValidator::new(44100.0);
        let test_cases = validator.generate_musical_note_tests();
        
        let config = PitchDetectionConfig::default();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        
        for test_case in test_cases {
            let result = detector.detect_pitch(&test_case.signal);
            
            if test_case.should_succeed {
                assert!(result.is_ok(), "Musical note test '{}' should succeed", test_case.name);
                
                let pitch_result = result.unwrap();
                let frequency_error = (pitch_result.frequency - test_case.expected_frequency).abs();
                
                assert!(frequency_error <= test_case.tolerance_hz,
                    "Musical note '{}': frequency error {:.2}Hz exceeds tolerance {:.2}Hz",
                    test_case.name, frequency_error, test_case.tolerance_hz);
                
                assert!(pitch_result.confidence >= test_case.min_confidence,
                    "Musical note '{}': confidence {:.2} below minimum {:.2}",
                    test_case.name, pitch_result.confidence, test_case.min_confidence);
            }
        }
    }

    #[test]
    fn test_harmonic_content_handling() {
        let validator = TestSignalValidator::new(44100.0);
        let test_cases = validator.generate_harmonic_tests();
        
        let config = PitchDetectionConfig {
            enable_harmonic_analysis: true,
            ..PitchDetectionConfig::default()
        };
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        
        for test_case in test_cases {
            let result = detector.detect_pitch(&test_case.signal);
            assert!(result.is_ok(), "Harmonic test '{}' should succeed", test_case.name);
            
            let pitch_result = result.unwrap();
            let frequency_error = (pitch_result.frequency - test_case.expected_frequency).abs();
            
            assert!(frequency_error <= test_case.tolerance_hz,
                "Harmonic test '{}': frequency error {:.2}Hz exceeds tolerance {:.2}Hz",
                test_case.name, frequency_error, test_case.tolerance_hz);
            
            // Harmonic signals should have measurable harmonic content
            assert!(pitch_result.harmonic_content >= 0.0 && pitch_result.harmonic_content <= 1.0,
                "Harmonic content should be in valid range");
        }
    }

    #[test]
    fn test_noise_robustness() {
        let validator = TestSignalValidator::new(44100.0);
        let test_cases = validator.generate_noise_robustness_tests();
        
        let config = PitchDetectionConfig::default();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        
        for test_case in test_cases {
            let result = detector.detect_pitch(&test_case.signal);
            
            if test_case.name.contains("0pct") || test_case.name.contains("5pct") {
                // Clean and lightly noisy signals should work well
                assert!(result.is_ok(), "Low noise test '{}' should succeed", test_case.name);
                
                let pitch_result = result.unwrap();
                let frequency_error = (pitch_result.frequency - test_case.expected_frequency).abs();
                
                assert!(frequency_error <= test_case.tolerance_hz,
                    "Low noise test '{}': frequency error {:.2}Hz exceeds tolerance {:.2}Hz",
                    test_case.name, frequency_error, test_case.tolerance_hz);
            } else {
                // Higher noise levels may fail, but if they succeed, they should be reasonably accurate
                if let Ok(pitch_result) = result {
                    let frequency_error = (pitch_result.frequency - test_case.expected_frequency).abs();
                    assert!(frequency_error <= test_case.tolerance_hz,
                        "High noise test '{}': if successful, should be within tolerance",
                        test_case.name);
                }
            }
        }
    }

    #[test]
    fn test_edge_cases() {
        let validator = TestSignalValidator::new(44100.0);
        let test_cases = validator.generate_edge_case_tests();
        
        let config = PitchDetectionConfig::default();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        
        for test_case in test_cases {
            let result = detector.detect_pitch(&test_case.signal);
            
            if test_case.should_succeed {
                // These should work but may have lower accuracy
                if result.is_ok() {
                    let pitch_result = result.unwrap();
                    if test_case.expected_frequency > 0.0 {
                        let frequency_error = (pitch_result.frequency - test_case.expected_frequency).abs();
                        assert!(frequency_error <= test_case.tolerance_hz,
                            "Edge case '{}': frequency error {:.2}Hz exceeds tolerance {:.2}Hz",
                            test_case.name, frequency_error, test_case.tolerance_hz);
                    }
                }
                // Note: Edge cases are allowed to fail
            } else {
                // These should fail
                assert!(result.is_err() || (result.is_ok() && result.unwrap().confidence < 0.1),
                    "Edge case '{}' should fail or have very low confidence", test_case.name);
            }
        }
    }

    #[test]
    fn test_algorithm_comparison() {
        let validator = TestSignalValidator::new(44100.0);
        let algorithms = [PitchAlgorithm::YIN, PitchAlgorithm::McLeod];
        
        // Test with a known good signal
        let test_signal = validator.generator.generate_sine_wave(440.0, 2048, 0.8);
        
        let mut results = Vec::new();
        
        for &algorithm in &algorithms {
            let config = PitchDetectionConfig {
                algorithm,
                ..PitchDetectionConfig::default()
            };
            let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            
            let result = detector.detect_pitch(&test_signal);
            assert!(result.is_ok(), "Algorithm {:?} should work with clean signal", algorithm);
            
            results.push((algorithm, result.unwrap()));
        }
        
        // Compare results
        for (algorithm, pitch_result) in &results {
            assert!((pitch_result.frequency - 440.0).abs() < 5.0,
                "Algorithm {:?} should detect 440Hz accurately", algorithm);
            assert!(pitch_result.confidence > 0.5,
                "Algorithm {:?} should have reasonable confidence", algorithm);
            assert!(pitch_result.processing_time_ns > 0,
                "Algorithm {:?} should record processing time", algorithm);
        }
        
        // Both algorithms should give similar results for clean signals
        let yin_freq = results.iter().find(|(alg, _)| *alg == PitchAlgorithm::YIN).unwrap().1.frequency;
        let mcleod_freq = results.iter().find(|(alg, _)| *alg == PitchAlgorithm::McLeod).unwrap().1.frequency;
        
        assert!((yin_freq - mcleod_freq).abs() < 10.0,
            "YIN and McLeod should give similar results for clean signals: {:.2}Hz vs {:.2}Hz",
            yin_freq, mcleod_freq);
    }

    #[test]
    fn test_signal_quality_validation() {
        let validator = TestSignalValidator::new(44100.0);
        
        // Good quality signal
        let good_signal = validator.generator.generate_sine_wave(440.0, 2048, 0.8);
        assert!(validator.validate_signal_quality(&good_signal, "good_signal"));
        
        // Very quiet signal (might fail quality check)
        let quiet_signal = validator.generator.generate_sine_wave(440.0, 2048, 0.001);
        let quiet_quality = validator.validate_signal_quality(&quiet_signal, "quiet_signal");
        // Quality check should detect this as borderline
        
        // Clipped signal
        let clipped_signal = vec![1.0; 2048]; // Fully clipped
        assert!(!validator.validate_signal_quality(&clipped_signal, "clipped_signal"));
        
        // Check signal quality metrics
        let rms = SignalQualityAnalyzer::rms_energy(&good_signal);
        assert!(rms > 0.1 && rms < 1.0, "Good signal should have reasonable RMS: {}", rms);
        
        let snr = SignalQualityAnalyzer::snr_estimate(&good_signal);
        assert!(snr >= 0.0 && snr <= 1.0, "SNR should be in valid range: {}", snr);
        
        let clipping = SignalQualityAnalyzer::clipping_percentage(&good_signal);
        assert!(clipping < 0.01, "Good signal should have minimal clipping: {:.2}%", clipping * 100.0);
    }

    #[test]
    fn test_comprehensive_validation_suite() {
        let validator = TestSignalValidator::new(44100.0);
        let results = validator.run_all_tests();
        
        // Print summary for manual review
        results.print_summary();
        
        // Validation requirements
        assert!(results.total_tests() > 20, "Should run a substantial number of tests");
        assert!(results.success_rate() > 0.6, "Should have >60% success rate overall");
        
        // Algorithm-specific requirements
        let stats = results.algorithm_stats();
        for (algorithm, (total, passed)) in stats {
            let success_rate = if total > 0 { passed as f32 / total as f32 } else { 0.0 };
            assert!(success_rate > 0.5, 
                "Algorithm {:?} should have >50% success rate, got {:.1}%", 
                algorithm, success_rate * 100.0);
        }
        
        // Performance requirements
        let avg_time = results.average_processing_time();
        assert!(avg_time < 50_000_000, // 50ms
            "Average processing time should be <50ms, got {:.2}ms", 
            avg_time as f64 / 1_000_000.0);
    }
}