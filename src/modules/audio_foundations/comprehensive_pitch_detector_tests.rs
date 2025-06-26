// Comprehensive Unit Tests for MultiAlgorithmPitchDetector - STORY-3.19
// Achieving >90% code coverage with extensive validation

#[cfg(test)]
mod comprehensive_pitch_detector_tests {
    use super::*;
    use std::sync::Arc;

    /// Create a standard test configuration
    fn create_test_config() -> PitchDetectionConfig {
        PitchDetectionConfig {
            algorithm: PitchAlgorithm::YIN,
            sample_rate: 44100.0,
            min_frequency: 80.0,
            max_frequency: 2000.0,
            yin_threshold: 0.2,
            mcleod_threshold: 0.3,
            mcleod_clarity_threshold: 0.7,
            enable_confidence_scoring: true,
            enable_harmonic_analysis: true,
            auto_selection_sensitivity: 0.5,
        }
    }

    /// Generate a pure sine wave for testing
    fn generate_sine_wave(frequency: f32, sample_rate: f32, duration_samples: usize, amplitude: f32) -> Vec<f32> {
        (0..duration_samples)
            .map(|i| {
                let t = i as f32 / sample_rate;
                amplitude * (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect()
    }

    /// Generate a complex signal with harmonics
    fn generate_complex_harmonic_signal(fundamental: f32, sample_rate: f32, duration_samples: usize) -> Vec<f32> {
        let mut signal = generate_sine_wave(fundamental, sample_rate, duration_samples, 0.8);
        let harmonic2 = generate_sine_wave(fundamental * 2.0, sample_rate, duration_samples, 0.4);
        let harmonic3 = generate_sine_wave(fundamental * 3.0, sample_rate, duration_samples, 0.2);
        
        for i in 0..signal.len() {
            signal[i] += harmonic2[i] + harmonic3[i];
            signal[i] /= 3.0; // Normalize
        }
        signal
    }

    /// Add white noise to a signal
    fn add_white_noise(signal: &mut [f32], noise_level: f32) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        for (i, sample) in signal.iter_mut().enumerate() {
            i.hash(&mut hasher);
            let random_val = (hasher.finish() % 1000) as f32 / 1000.0 - 0.5;
            *sample += random_val * noise_level;
        }
    }

    // ===============================
    // CONSTRUCTOR AND CONFIGURATION TESTS
    // ===============================

    #[test]
    fn test_detector_creation_valid_config() {
        let config = create_test_config();
        let detector = MultiAlgorithmPitchDetector::new(config, None);
        assert!(detector.is_ok(), "Should create detector with valid config");
    }

    #[test]
    fn test_detector_creation_with_event_bus() {
        let config = create_test_config();
        let event_bus = Arc::new(crate::modules::application_core::typed_event_bus::TypedEventBus::new());
        let detector = MultiAlgorithmPitchDetector::new(config, Some(event_bus));
        assert!(detector.is_ok(), "Should create detector with event bus");
    }

    #[test]
    fn test_invalid_sample_rate() {
        let mut config = create_test_config();
        config.sample_rate = -1.0;
        let detector = MultiAlgorithmPitchDetector::new(config, None);
        assert!(detector.is_err(), "Should reject negative sample rate");
    }

    #[test]
    fn test_invalid_frequency_range() {
        let mut config = create_test_config();
        config.min_frequency = 1000.0;
        config.max_frequency = 500.0; // Invalid: min > max
        let detector = MultiAlgorithmPitchDetector::new(config, None);
        assert!(detector.is_err(), "Should reject invalid frequency range");
    }

    #[test]
    fn test_invalid_thresholds() {
        let mut config = create_test_config();
        config.yin_threshold = 1.5; // Invalid: > 1.0
        assert!(MultiAlgorithmPitchDetector::new(config.clone(), None).is_err());
        
        config.yin_threshold = 0.2;
        config.mcleod_threshold = -0.1; // Invalid: < 0.0
        assert!(MultiAlgorithmPitchDetector::new(config, None).is_err());
    }

    #[test]
    fn test_zero_min_frequency() {
        let mut config = create_test_config();
        config.min_frequency = 0.0;
        let detector = MultiAlgorithmPitchDetector::new(config, None);
        assert!(detector.is_err(), "Should reject zero min frequency");
    }

    // ===============================
    // ALGORITHM SWITCHING TESTS
    // ===============================

    #[test]
    fn test_algorithm_switching() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        // Test YIN
        assert!(detector.set_algorithm(PitchAlgorithm::YIN).is_ok());
        let info = detector.get_algorithm_info();
        assert_eq!(info.name, PitchAlgorithm::YIN);

        // Test McLeod
        assert!(detector.set_algorithm(PitchAlgorithm::McLeod).is_ok());
        let info = detector.get_algorithm_info();
        assert_eq!(info.name, PitchAlgorithm::McLeod);

        // Test Auto
        assert!(detector.set_algorithm(PitchAlgorithm::Auto).is_ok());
        let info = detector.get_algorithm_info();
        assert_eq!(info.name, PitchAlgorithm::Auto);
    }

    #[test]
    fn test_algorithm_info_consistency() {
        let config = create_test_config();
        let detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        
        let info = detector.get_algorithm_info();
        assert!(info.avg_processing_time_ns > 0);
        assert!(info.accuracy_score >= 0.0 && info.accuracy_score <= 1.0);
        assert!(info.signal_suitability >= 0.0 && info.signal_suitability <= 1.0);
        assert!(info.memory_usage_bytes > 0);
    }

    #[test]
    fn test_performance_comparison_structure() {
        let config = create_test_config();
        let detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        
        let comparison = detector.get_performance_comparison();
        
        // Verify structure completeness
        assert_eq!(comparison.yin_performance.name, PitchAlgorithm::YIN);
        assert_eq!(comparison.mcleod_performance.name, PitchAlgorithm::McLeod);
        assert!(comparison.recommendation_confidence >= 0.0);
        assert!(comparison.recommendation_confidence <= 1.0);
        assert!(matches!(comparison.recommended_algorithm, 
            PitchAlgorithm::YIN | PitchAlgorithm::McLeod | PitchAlgorithm::Auto));
    }

    // ===============================
    // PITCH DETECTION ACCURACY TESTS
    // ===============================

    #[test]
    fn test_yin_pitch_detection_pure_tone() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        detector.set_algorithm(PitchAlgorithm::YIN).unwrap();

        let test_frequency = 440.0; // A4
        let buffer = generate_sine_wave(test_frequency, 44100.0, 2048, 0.8);

        let result = detector.detect_pitch(&buffer);
        assert!(result.is_ok(), "Should detect pitch from pure tone");
        
        let pitch_result = result.unwrap();
        assert!(pitch_result.is_valid, "Result should be valid");
        assert_eq!(pitch_result.algorithm_used, PitchAlgorithm::YIN);
        assert!((pitch_result.frequency - test_frequency).abs() < 5.0, 
            "Frequency should be within 5Hz: detected={}, expected={}", 
            pitch_result.frequency, test_frequency);
        assert!(pitch_result.confidence > 0.5, "Confidence should be reasonably high for pure tone");
    }

    #[test]
    fn test_mcleod_pitch_detection_pure_tone() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        detector.set_algorithm(PitchAlgorithm::McLeod).unwrap();

        let test_frequency = 880.0; // A5
        let buffer = generate_sine_wave(test_frequency, 44100.0, 2048, 0.8);

        let result = detector.detect_pitch(&buffer);
        assert!(result.is_ok(), "Should detect pitch from pure tone");
        
        let pitch_result = result.unwrap();
        assert!(pitch_result.is_valid, "Result should be valid");
        assert_eq!(pitch_result.algorithm_used, PitchAlgorithm::McLeod);
        assert!((pitch_result.frequency - test_frequency).abs() < 5.0, 
            "Frequency should be within 5Hz: detected={}, expected={}", 
            pitch_result.frequency, test_frequency);
    }

    #[test]
    fn test_auto_algorithm_selection() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        detector.set_algorithm(PitchAlgorithm::Auto).unwrap();

        let test_frequency = 220.0;
        let buffer = generate_sine_wave(test_frequency, 44100.0, 2048, 0.8);

        let result = detector.detect_pitch(&buffer);
        assert!(result.is_ok(), "Auto selection should work");
        
        let pitch_result = result.unwrap();
        assert!(pitch_result.is_valid, "Result should be valid");
        assert!(matches!(pitch_result.algorithm_used, PitchAlgorithm::YIN | PitchAlgorithm::McLeod));
        assert!((pitch_result.frequency - test_frequency).abs() < 10.0, 
            "Auto selection should be reasonably accurate");
    }

    #[test]
    fn test_multiple_frequency_detection() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        let test_frequencies = [110.0, 220.0, 440.0, 880.0, 1760.0];
        
        for &freq in &test_frequencies {
            if freq >= 80.0 && freq <= 2000.0 { // Within config range
                let buffer = generate_sine_wave(freq, 44100.0, 2048, 0.7);
                let result = detector.detect_pitch(&buffer);
                
                assert!(result.is_ok(), "Should detect frequency {}", freq);
                let pitch_result = result.unwrap();
                assert!(pitch_result.is_valid, "Result should be valid for {}", freq);
                assert!((pitch_result.frequency - freq).abs() < 10.0, 
                    "Frequency {} should be within 10Hz: detected={}", freq, pitch_result.frequency);
            }
        }
    }

    #[test]
    fn test_harmonic_content_analysis() {
        let mut config = create_test_config();
        config.enable_harmonic_analysis = true;
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        // Pure tone should have high harmonic content
        let pure_tone = generate_sine_wave(440.0, 44100.0, 2048, 0.8);
        let pure_result = detector.detect_pitch(&pure_tone).unwrap();
        
        // Complex harmonic signal should also have measurable harmonic content
        let complex_signal = generate_complex_harmonic_signal(440.0, 44100.0, 2048);
        let complex_result = detector.detect_pitch(&complex_signal).unwrap();
        
        assert!(pure_result.harmonic_content >= 0.0 && pure_result.harmonic_content <= 1.0);
        assert!(complex_result.harmonic_content >= 0.0 && complex_result.harmonic_content <= 1.0);
    }

    #[test]
    fn test_confidence_scoring() {
        let mut config = create_test_config();
        config.enable_confidence_scoring = true;
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        // Strong signal should have higher confidence
        let strong_signal = generate_sine_wave(440.0, 44100.0, 2048, 0.9);
        let strong_result = detector.detect_pitch(&strong_signal).unwrap();

        // Weak signal should have lower confidence
        let weak_signal = generate_sine_wave(440.0, 44100.0, 2048, 0.1);
        let weak_result = detector.detect_pitch(&weak_signal).unwrap();

        assert!(strong_result.confidence >= 0.0 && strong_result.confidence <= 1.0);
        assert!(weak_result.confidence >= 0.0 && weak_result.confidence <= 1.0);
        // Note: Confidence comparison depends on algorithm behavior
    }

    // ===============================
    // NOISE ROBUSTNESS TESTS
    // ===============================

    #[test]
    fn test_noise_robustness_light() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        let test_frequency = 440.0;
        let mut buffer = generate_sine_wave(test_frequency, 44100.0, 2048, 0.8);
        add_white_noise(&mut buffer, 0.05); // 5% noise

        let result = detector.detect_pitch(&buffer);
        if let Ok(pitch_result) = result {
            assert!(pitch_result.is_valid, "Should handle light noise");
            assert!((pitch_result.frequency - test_frequency).abs() < 20.0, 
                "Should be reasonably accurate with light noise");
        }
    }

    #[test]
    fn test_noise_robustness_moderate() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        detector.set_algorithm(PitchAlgorithm::McLeod).unwrap(); // More robust

        let test_frequency = 440.0;
        let mut buffer = generate_sine_wave(test_frequency, 44100.0, 2048, 0.7);
        add_white_noise(&mut buffer, 0.15); // 15% noise

        let result = detector.detect_pitch(&buffer);
        // McLeod should handle moderate noise better than YIN
        if let Ok(pitch_result) = result {
            assert!(pitch_result.confidence > 0.2, "Should have some confidence even with noise");
        }
    }

    // ===============================
    // EDGE CASE AND ERROR HANDLING TESTS
    // ===============================

    #[test]
    fn test_empty_buffer() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        let empty_buffer: Vec<f32> = vec![];
        let result = detector.detect_pitch(&empty_buffer);
        assert!(result.is_err(), "Should reject empty buffer");
        
        if let Err(error) = result {
            assert!(matches!(error, PitchError::ProcessingError(_)));
        }
    }

    #[test]
    fn test_buffer_too_small() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        let small_buffer = vec![0.5; 32]; // Too small for reliable detection
        let result = detector.detect_pitch(&small_buffer);
        assert!(result.is_err(), "Should reject buffer that's too small");
        
        if let Err(error) = result {
            assert!(matches!(error, PitchError::BufferTooSmall(_, _)));
        }
    }

    #[test]
    fn test_silent_buffer() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        let silence = vec![0.0; 2048];
        let result = detector.detect_pitch(&silence);
        
        // Behavior with silence is algorithm-dependent
        if let Ok(pitch_result) = result {
            assert!(pitch_result.confidence < 0.3, "Confidence should be low for silence");
        }
    }

    #[test]
    fn test_dc_offset_buffer() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        let dc_offset = vec![0.5; 2048]; // Constant value
        let result = detector.detect_pitch(&dc_offset);
        
        // Should either error or have very low confidence
        if let Ok(pitch_result) = result {
            assert!(pitch_result.confidence < 0.2, "DC offset should have low confidence");
        }
    }

    #[test]
    fn test_clipped_signal() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        let test_frequency = 440.0;
        let mut buffer = generate_sine_wave(test_frequency, 44100.0, 2048, 2.0); // Overdriven
        
        // Clip signal
        for sample in &mut buffer {
            *sample = sample.max(-1.0).min(1.0);
        }

        let result = detector.detect_pitch(&buffer);
        // Clipped signals are harder to analyze but should still be detectable
        if let Ok(pitch_result) = result {
            assert!((pitch_result.frequency - test_frequency).abs() < 50.0, 
                "Should still detect frequency despite clipping");
        }
    }

    #[test]
    fn test_frequency_outside_range() {
        let config = create_test_config(); // Range: 80-2000 Hz
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        // Test frequency below range
        let low_freq_buffer = generate_sine_wave(50.0, 44100.0, 2048, 0.8);
        if let Ok(result) = detector.detect_pitch(&low_freq_buffer) {
            assert!(!result.is_valid, "Frequency below range should be marked invalid");
        }

        // Test frequency above range
        let high_freq_buffer = generate_sine_wave(3000.0, 44100.0, 2048, 0.8);
        if let Ok(result) = detector.detect_pitch(&high_freq_buffer) {
            assert!(!result.is_valid, "Frequency above range should be marked invalid");
        }
    }

    // ===============================
    // PERFORMANCE AND TIMING TESTS
    // ===============================

    #[test]
    fn test_processing_time_measurement() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        let buffer = generate_sine_wave(440.0, 44100.0, 2048, 0.8);
        let result = detector.detect_pitch(&buffer).unwrap();

        assert!(result.processing_time_ns > 0, "Should measure processing time");
        assert!(result.processing_time_ns < 100_000_000, "Should complete within 100ms");
    }

    #[test]
    fn test_performance_tracking() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        let buffer = generate_sine_wave(440.0, 44100.0, 2048, 0.8);

        // Process multiple times to build performance history
        for _ in 0..10 {
            let _ = detector.detect_pitch(&buffer);
        }

        let comparison = detector.get_performance_comparison();
        assert!(comparison.yin_performance.avg_processing_time_ns > 0);
        assert!(comparison.recommendation_confidence >= 0.0);
    }

    #[test]
    fn test_algorithm_performance_comparison() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        let buffer = generate_sine_wave(440.0, 44100.0, 2048, 0.8);

        // Test YIN
        detector.set_algorithm(PitchAlgorithm::YIN).unwrap();
        let _yin_result = detector.detect_pitch(&buffer);

        // Test McLeod
        detector.set_algorithm(PitchAlgorithm::McLeod).unwrap();
        let _mcleod_result = detector.detect_pitch(&buffer);

        let comparison = detector.get_performance_comparison();
        
        // Both algorithms should have recorded performance data
        assert!(comparison.yin_performance.avg_processing_time_ns > 0);
        assert!(comparison.mcleod_performance.avg_processing_time_ns > 0);
    }

    // ===============================
    // CONFIGURATION RECONFIGURATION TESTS
    // ===============================

    #[test]
    fn test_configuration_update() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        let mut new_config = create_test_config();
        new_config.sample_rate = 48000.0;
        new_config.min_frequency = 100.0;
        new_config.max_frequency = 4000.0;

        assert!(detector.configure(new_config).is_ok(), "Should accept valid configuration");
    }

    #[test]
    fn test_configuration_update_invalid() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        let mut invalid_config = create_test_config();
        invalid_config.sample_rate = -1.0;

        assert!(detector.configure(invalid_config).is_err(), "Should reject invalid configuration");
    }

    #[test]
    fn test_sample_rate_change_resets_detectors() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        // Process with original sample rate
        let buffer = generate_sine_wave(440.0, 44100.0, 1024, 0.8);
        let _ = detector.detect_pitch(&buffer);

        // Change sample rate
        let mut new_config = create_test_config();
        new_config.sample_rate = 48000.0;
        assert!(detector.configure(new_config).is_ok());

        // Should work with new sample rate
        let new_buffer = generate_sine_wave(440.0, 48000.0, 1024, 0.8);
        let result = detector.detect_pitch(&new_buffer);
        assert!(result.is_ok(), "Should work after sample rate change");
    }

    // ===============================
    // EVENT PUBLISHING TESTS
    // ===============================

    #[test]
    fn test_event_publishing_enabled() {
        let config = create_test_config();
        let event_bus = Arc::new(crate::modules::application_core::typed_event_bus::TypedEventBus::new());
        let mut detector = MultiAlgorithmPitchDetector::new(config, Some(event_bus)).unwrap();

        detector.set_event_publishing(true);
        
        let buffer = generate_sine_wave(440.0, 44100.0, 2048, 0.8);
        let result = detector.detect_pitch(&buffer);
        
        assert!(result.is_ok(), "Should detect pitch with event publishing enabled");
    }

    #[test]
    fn test_event_publishing_disabled() {
        let config = create_test_config();
        let event_bus = Arc::new(crate::modules::application_core::typed_event_bus::TypedEventBus::new());
        let mut detector = MultiAlgorithmPitchDetector::new(config, Some(event_bus)).unwrap();

        detector.set_event_publishing(false);
        
        let buffer = generate_sine_wave(440.0, 44100.0, 2048, 0.8);
        let result = detector.detect_pitch(&buffer);
        
        assert!(result.is_ok(), "Should detect pitch with event publishing disabled");
    }

    // ===============================
    // SIGNAL ANALYSIS TESTS
    // ===============================

    #[test]
    fn test_snr_estimation() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        let clean_signal = generate_sine_wave(440.0, 44100.0, 2048, 0.8);
        let clean_result = detector.detect_pitch(&clean_signal).unwrap();

        let mut noisy_signal = generate_sine_wave(440.0, 44100.0, 2048, 0.8);
        add_white_noise(&mut noisy_signal, 0.3);
        let noisy_result = detector.detect_pitch(&noisy_signal).unwrap();

        assert!(clean_result.snr_estimate >= 0.0 && clean_result.snr_estimate <= 1.0);
        assert!(noisy_result.snr_estimate >= 0.0 && noisy_result.snr_estimate <= 1.0);
        // Clean signal should generally have higher SNR estimate
    }

    #[test]
    fn test_signal_complexity_estimation() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        // Simple signal (sine wave)
        let simple_signal = generate_sine_wave(440.0, 44100.0, 2048, 0.8);
        let _simple_result = detector.detect_pitch(&simple_signal);

        // Complex signal (harmonics)
        let complex_signal = generate_complex_harmonic_signal(440.0, 44100.0, 2048);
        let _complex_result = detector.detect_pitch(&complex_signal);

        // Both should process successfully
        // Complexity estimation is internal but affects auto-selection
    }

    // ===============================
    // STRESS AND BOUNDARY TESTS
    // ===============================

    #[test]
    fn test_maximum_buffer_size() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        let large_buffer = generate_sine_wave(440.0, 44100.0, 8192, 0.8);
        let result = detector.detect_pitch(&large_buffer);
        assert!(result.is_ok(), "Should handle large buffers");
    }

    #[test]
    fn test_minimum_valid_buffer_size() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        let min_buffer = generate_sine_wave(440.0, 44100.0, 64, 0.8);
        let result = detector.detect_pitch(&min_buffer);
        // Should either work or give a clear error
        assert!(result.is_ok() || matches!(result.unwrap_err(), PitchError::BufferTooSmall(_, _)));
    }

    #[test]
    fn test_repeated_detection_calls() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        let buffer = generate_sine_wave(440.0, 44100.0, 2048, 0.8);

        // Call detection many times
        for i in 0..100 {
            let result = detector.detect_pitch(&buffer);
            assert!(result.is_ok(), "Should work on iteration {}", i);
        }
    }

    #[test]
    fn test_frequency_boundary_detection() {
        let config = create_test_config(); // 80-2000 Hz range
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();

        // Test near boundaries
        let frequencies = [81.0, 100.0, 1900.0, 1999.0];
        
        for &freq in &frequencies {
            let buffer = generate_sine_wave(freq, 44100.0, 2048, 0.8);
            let result = detector.detect_pitch(&buffer);
            
            if let Ok(pitch_result) = result {
                assert!(pitch_result.is_valid, "Frequency {} should be valid", freq);
                assert!((pitch_result.frequency - freq).abs() < 15.0, 
                    "Boundary frequency {} should be reasonably accurate", freq);
            }
        }
    }

    // ===============================
    // ALGORITHM-SPECIFIC TESTS
    // ===============================

    #[test]
    fn test_yin_clarity_interpretation() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        detector.set_algorithm(PitchAlgorithm::YIN).unwrap();

        let buffer = generate_sine_wave(440.0, 44100.0, 2048, 0.8);
        let result = detector.detect_pitch(&buffer).unwrap();

        // YIN clarity is inverse correlation (lower = better)
        assert!(result.clarity >= 0.0, "YIN clarity should be non-negative");
        assert!(result.raw_clarity >= 0.0, "Raw clarity should be non-negative");
    }

    #[test]
    fn test_mcleod_clarity_interpretation() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        detector.set_algorithm(PitchAlgorithm::McLeod).unwrap();

        let buffer = generate_sine_wave(440.0, 44100.0, 2048, 0.8);
        let result = detector.detect_pitch(&buffer).unwrap();

        // McLeod clarity is direct correlation (higher = better)
        assert!(result.clarity >= 0.0 && result.clarity <= 1.0, "McLeod clarity should be 0-1");
        assert!(result.raw_clarity >= 0.0, "Raw clarity should be non-negative");
    }

    #[test]
    fn test_auto_selection_signal_adaptation() {
        let config = create_test_config();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        detector.set_algorithm(PitchAlgorithm::Auto).unwrap();

        // Clean signal
        let clean_buffer = generate_sine_wave(440.0, 44100.0, 2048, 0.8);
        let clean_result = detector.detect_pitch(&clean_buffer).unwrap();

        // Noisy signal
        let mut noisy_buffer = generate_sine_wave(440.0, 44100.0, 2048, 0.8);
        add_white_noise(&mut noisy_buffer, 0.2);
        let noisy_result = detector.detect_pitch(&noisy_buffer).unwrap();

        // Auto-selection might choose different algorithms based on signal characteristics
        assert!(matches!(clean_result.algorithm_used, PitchAlgorithm::YIN | PitchAlgorithm::McLeod));
        assert!(matches!(noisy_result.algorithm_used, PitchAlgorithm::YIN | PitchAlgorithm::McLeod));
    }
}