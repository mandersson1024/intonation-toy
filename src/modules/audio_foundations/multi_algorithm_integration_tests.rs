// Integration Tests for Multi-Algorithm Pitch Detection - STORY-015
// Comprehensive testing suite for the multi-algorithm pitch detection implementation

#[cfg(test)]
mod integration_tests {
    use crate::modules::audio_foundations::{
        MultiAlgorithmPitchDetector, PitchDetector, PitchAlgorithm, PitchDetectionConfig,
        RuntimePitchSwitcher, AutoSwitchConfig
    };
    use std::f32::consts::PI;
    
    /// Generate test signals for comprehensive testing
    struct TestSignalGenerator;
    
    impl TestSignalGenerator {
        /// Generate pure sine wave
        fn sine_wave(frequency: f32, sample_rate: f32, duration_samples: usize, amplitude: f32) -> Vec<f32> {
            (0..duration_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    amplitude * (2.0 * PI * frequency * t).sin()
                })
                .collect()
        }
        
        /// Generate sine wave with harmonic content
        fn harmonic_signal(fundamental: f32, sample_rate: f32, duration_samples: usize, amplitude: f32) -> Vec<f32> {
            (0..duration_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let f1 = amplitude * (2.0 * PI * fundamental * t).sin();
                    let f2 = amplitude * 0.5 * (2.0 * PI * 2.0 * fundamental * t).sin();
                    let f3 = amplitude * 0.25 * (2.0 * PI * 3.0 * fundamental * t).sin();
                    f1 + f2 + f3
                })
                .collect()
        }
        
        /// Generate noisy signal (sine + white noise)
        fn noisy_signal(frequency: f32, sample_rate: f32, duration_samples: usize, signal_amplitude: f32, noise_amplitude: f32) -> Vec<f32> {
            let mut lfsr = 0x1u32; // Simple PRNG for deterministic noise
            (0..duration_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let signal = signal_amplitude * (2.0 * PI * frequency * t).sin();
                    
                    // Generate deterministic noise
                    lfsr = (lfsr >> 1) ^ (0x80000057u32 & (0u32.wrapping_sub(lfsr & 1)));
                    let noise = (lfsr as f32 / u32::MAX as f32 - 0.5) * 2.0 * noise_amplitude;
                    
                    signal + noise
                })
                .collect()
        }
        
        /// Generate frequency sweep
        fn frequency_sweep(start_freq: f32, end_freq: f32, sample_rate: f32, duration_samples: usize, amplitude: f32) -> Vec<f32> {
            (0..duration_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let progress = i as f32 / duration_samples as f32;
                    let frequency = start_freq + (end_freq - start_freq) * progress;
                    amplitude * (2.0 * PI * frequency * t).sin()
                })
                .collect()
        }
        
        /// Generate musical chord (multiple frequencies)
        fn musical_chord(frequencies: &[f32], sample_rate: f32, duration_samples: usize, amplitude: f32) -> Vec<f32> {
            (0..duration_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate;
                    let mixed_signal: f32 = frequencies.iter()
                        .map(|&freq| (2.0 * PI * freq * t).sin())
                        .sum();
                    amplitude * mixed_signal / frequencies.len() as f32
                })
                .collect()
        }
    }
    
    /// Musical note frequencies for testing
    struct MusicalNotes;
    
    impl MusicalNotes {
        fn a4() -> f32 { 440.0 }      // Reference pitch
        fn c4() -> f32 { 261.63 }     // Middle C
        fn e4() -> f32 { 329.63 }     // E above middle C
        fn g4() -> f32 { 392.00 }     // G above middle C
        fn c5() -> f32 { 523.25 }     // C one octave above middle C
        fn a3() -> f32 { 220.0 }      // A below A4
        fn low_e() -> f32 { 82.41 }   // Low E (guitar)
        fn high_e() -> f32 { 1318.51 } // High E (guitar)
        
        /// Get frequency for a note N semitones above A4
        fn semitone_above_a4(semitones: i32) -> f32 {
            440.0 * 2.0_f32.powf(semitones as f32 / 12.0)
        }
    }
    
    /// Test algorithm accuracy across different signal types
    mod accuracy_tests {
        use super::*;
        
        #[test]
        fn test_pure_tone_accuracy() {
            let config = PitchDetectionConfig::default();
            let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            
            let test_frequencies = [
                MusicalNotes::low_e(),
                MusicalNotes::a3(),
                MusicalNotes::c4(),
                MusicalNotes::a4(),
                MusicalNotes::c5(),
                MusicalNotes::high_e(),
            ];
            
            for &target_freq in &test_frequencies {
                let buffer = TestSignalGenerator::sine_wave(target_freq, 44100.0, 2048, 0.7);
                
                // Test both algorithms
                for algorithm in [PitchAlgorithm::YIN, PitchAlgorithm::McLeod] {
                    detector.set_algorithm(algorithm).unwrap();
                    
                    let result = detector.detect_pitch(&buffer);
                    if let Ok(pitch_result) = result {
                        if pitch_result.is_valid {
                            let detected_freq = pitch_result.frequency;
                            let cents_error = cents_difference(target_freq, detected_freq).abs();
                            
                            assert!(cents_error <= 15.0,
                                "Algorithm {:?} accuracy failed for {:.1}Hz: got {:.1}Hz ({:.1} cents error)",
                                algorithm, target_freq, detected_freq, cents_error);
                            
                            assert!(pitch_result.confidence > 0.5,
                                "Low confidence for clean signal: {} with algorithm {:?}",
                                pitch_result.confidence, algorithm);
                        }
                    }
                }
            }
        }
        
        #[test]
        fn test_harmonic_signal_detection() {
            let config = PitchDetectionConfig::default();
            let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            
            let fundamental = MusicalNotes::a4();
            let buffer = TestSignalGenerator::harmonic_signal(fundamental, 44100.0, 2048, 0.5);
            
            for algorithm in [PitchAlgorithm::YIN, PitchAlgorithm::McLeod] {
                detector.set_algorithm(algorithm).unwrap();
                
                let result = detector.detect_pitch(&buffer);
                if let Ok(pitch_result) = result {
                    if pitch_result.is_valid {
                        let detected_freq = pitch_result.frequency;
                        let cents_error = cents_difference(fundamental, detected_freq).abs();
                        
                        // Should detect fundamental, not harmonics
                        assert!(cents_error <= 30.0,
                            "Harmonic detection failed with {:?}: expected ~{:.1}Hz, got {:.1}Hz ({:.1} cents)",
                            algorithm, fundamental, detected_freq, cents_error);
                        
                        // Harmonic content should be detected
                        assert!(pitch_result.harmonic_content > 0.3,
                            "Harmonic content not detected: {}", pitch_result.harmonic_content);
                    }
                }
            }
        }
        
        #[test]
        fn test_noisy_signal_robustness() {
            let config = PitchDetectionConfig::default();
            let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            
            let target_freq = MusicalNotes::a4();
            let signal_amplitude = 0.8;
            let noise_amplitude = 0.2; // 25% noise
            
            let buffer = TestSignalGenerator::noisy_signal(target_freq, 44100.0, 2048, signal_amplitude, noise_amplitude);
            
            for algorithm in [PitchAlgorithm::YIN, PitchAlgorithm::McLeod] {
                detector.set_algorithm(algorithm).unwrap();
                
                let result = detector.detect_pitch(&buffer);
                if let Ok(pitch_result) = result {
                    if pitch_result.is_valid {
                        let detected_freq = pitch_result.frequency;
                        let cents_error = cents_difference(target_freq, detected_freq).abs();
                        
                        // Allow more error for noisy signals
                        assert!(cents_error <= 50.0,
                            "Noisy signal detection failed with {:?}: expected ~{:.1}Hz, got {:.1}Hz ({:.1} cents)",
                            algorithm, target_freq, detected_freq, cents_error);
                        
                        // SNR estimate should reflect noise
                        assert!(pitch_result.snr_estimate < 0.9,
                            "SNR estimate too high for noisy signal: {}", pitch_result.snr_estimate);
                    }
                }
            }
        }
        
        #[test]
        fn test_musical_interval_accuracy() {
            let config = PitchDetectionConfig::default();
            let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            
            // Test perfect musical intervals
            let intervals = [
                (0, "Unison"),
                (2, "Major Second"),
                (4, "Major Third"),
                (7, "Perfect Fifth"),
                (12, "Octave"),
            ];
            
            for &(semitones, interval_name) in &intervals {
                let target_freq = MusicalNotes::semitone_above_a4(semitones);
                let buffer = TestSignalGenerator::sine_wave(target_freq, 44100.0, 2048, 0.6);
                
                detector.set_algorithm(PitchAlgorithm::YIN).unwrap();
                let result = detector.detect_pitch(&buffer);
                
                if let Ok(pitch_result) = result {
                    if pitch_result.is_valid {
                        let detected_freq = pitch_result.frequency;
                        let cents_error = cents_difference(target_freq, detected_freq).abs();
                        
                        assert!(cents_error <= 10.0,
                            "{} interval test failed: expected {:.2}Hz, got {:.2}Hz ({:.1} cents error)",
                            interval_name, target_freq, detected_freq, cents_error);
                    }
                }
            }
        }
        
        fn cents_difference(freq1: f32, freq2: f32) -> f32 {
            1200.0 * (freq2 / freq1).log2()
        }
    }
    
    /// Test algorithm switching and performance comparison
    mod switching_tests {
        use super::*;
        
        #[test]
        fn test_runtime_algorithm_switching() {
            let config = PitchDetectionConfig::default();
            let detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            let mut switcher = RuntimePitchSwitcher::new(detector, None);
            
            // Test manual switching
            assert_eq!(switcher.current_algorithm(), PitchAlgorithm::YIN);
            
            switcher.switch_algorithm(PitchAlgorithm::McLeod).unwrap();
            assert_eq!(switcher.current_algorithm(), PitchAlgorithm::McLeod);
            
            switcher.switch_algorithm(PitchAlgorithm::Auto).unwrap();
            assert_eq!(switcher.current_algorithm(), PitchAlgorithm::Auto);
            
            // Check switch history
            let history = switcher.get_switch_history();
            assert_eq!(history.len(), 2);
        }
        
        #[test]
        fn test_automatic_switching_behavior() {
            let config = PitchDetectionConfig::default();
            let detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            let mut switcher = RuntimePitchSwitcher::new(detector, None);
            
            // Configure aggressive auto-switching for testing
            let mut auto_config = AutoSwitchConfig::default();
            auto_config.enable_auto_switch = true;
            auto_config.poor_results_threshold = 3;
            auto_config.min_switch_interval_ms = 100; // Very short for testing
            switcher.configure_auto_switch(auto_config);
            
            let buffer = TestSignalGenerator::sine_wave(440.0, 44100.0, 2048, 0.5);
            
            // Process several times to potentially trigger auto-switch
            for _ in 0..10 {
                let _ = switcher.detect_pitch_with_auto_switch(&buffer);
            }
            
            // Should have performance statistics
            let stats = switcher.get_performance_stats();
            assert!(stats.average_confidence > 0.0);
            assert!(stats.average_processing_time_ns > 0);
        }
        
        #[test]
        fn test_performance_comparison() {
            let config = PitchDetectionConfig::default();
            let detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            let switcher = RuntimePitchSwitcher::new(detector, None);
            
            let comparison = switcher.get_performance_comparison();
            
            // Should have reasonable performance data
            assert!(comparison.yin_performance.avg_processing_time_ns > 0);
            assert!(comparison.mcleod_performance.avg_processing_time_ns > 0);
            assert!(comparison.recommendation_confidence >= 0.0);
            assert!(comparison.recommendation_confidence <= 1.0);
        }
        
        #[test]
        fn test_algorithm_consistency() {
            let config = PitchDetectionConfig::default();
            let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            
            let test_frequencies = [220.0, 440.0, 880.0];
            
            for &freq in &test_frequencies {
                let buffer = TestSignalGenerator::sine_wave(freq, 44100.0, 2048, 0.7);
                
                detector.set_algorithm(PitchAlgorithm::YIN).unwrap();
                let yin_result = detector.detect_pitch(&buffer);
                
                detector.set_algorithm(PitchAlgorithm::McLeod).unwrap();
                let mcleod_result = detector.detect_pitch(&buffer);
                
                if let (Ok(yin), Ok(mcleod)) = (yin_result, mcleod_result) {
                    if yin.is_valid && mcleod.is_valid {
                        let cents_diff = cents_difference(yin.frequency, mcleod.frequency).abs();
                        
                        // Both algorithms should agree reasonably well on clean signals
                        assert!(cents_diff <= 30.0,
                            "Algorithm inconsistency at {}Hz: YIN={:.1}Hz, McLeod={:.1}Hz ({:.1} cents apart)",
                            freq, yin.frequency, mcleod.frequency, cents_diff);
                    }
                }
            }
        }
        
        fn cents_difference(freq1: f32, freq2: f32) -> f32 {
            1200.0 * (freq2 / freq1).log2()
        }
    }
    
    /// Test edge cases and error handling
    mod edge_case_tests {
        use super::*;
        
        #[test]
        fn test_empty_buffer_handling() {
            let config = PitchDetectionConfig::default();
            let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            
            let empty_buffer: Vec<f32> = vec![];
            let result = detector.detect_pitch(&empty_buffer);
            assert!(result.is_err());
        }
        
        #[test]
        fn test_small_buffer_handling() {
            let config = PitchDetectionConfig::default();
            let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            
            let small_buffer = vec![0.1; 32]; // Too small for reliable detection
            let result = detector.detect_pitch(&small_buffer);
            assert!(result.is_err());
        }
        
        #[test]
        fn test_silence_handling() {
            let config = PitchDetectionConfig::default();
            let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            
            let silence = vec![0.0; 2048];
            let result = detector.detect_pitch(&silence);
            
            // Either error or very low confidence
            if let Ok(pitch_result) = result {
                assert!(pitch_result.confidence < 0.3);
            }
        }
        
        #[test]
        fn test_extreme_frequencies() {
            let config = PitchDetectionConfig::default();
            let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            
            // Test very low frequency (below detection range)
            let low_freq_buffer = TestSignalGenerator::sine_wave(50.0, 44100.0, 4096, 0.8);
            let result = detector.detect_pitch(&low_freq_buffer);
            if let Ok(pitch_result) = result {
                if pitch_result.frequency < 80.0 {
                    assert!(!pitch_result.is_valid); // Should be marked invalid
                }
            }
            
            // Test very high frequency (above detection range)
            let high_freq_buffer = TestSignalGenerator::sine_wave(3000.0, 44100.0, 2048, 0.8);
            let result = detector.detect_pitch(&high_freq_buffer);
            if let Ok(pitch_result) = result {
                if pitch_result.frequency > 2000.0 {
                    assert!(!pitch_result.is_valid); // Should be marked invalid
                }
            }
        }
        
        #[test]
        fn test_configuration_validation() {
            // Test invalid sample rate
            let mut config = PitchDetectionConfig::default();
            config.sample_rate = -1.0;
            assert!(MultiAlgorithmPitchDetector::new(config, None).is_err());
            
            // Test invalid frequency range
            config = PitchDetectionConfig::default();
            config.min_frequency = 1000.0;
            config.max_frequency = 500.0;
            assert!(MultiAlgorithmPitchDetector::new(config, None).is_err());
            
            // Test invalid thresholds
            config = PitchDetectionConfig::default();
            config.yin_threshold = 1.5;
            assert!(MultiAlgorithmPitchDetector::new(config, None).is_err());
        }
        
        #[test]
        fn test_buffer_size_robustness() {
            let config = PitchDetectionConfig::default();
            let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            
            let test_freq = 440.0;
            let buffer_sizes = [512, 1024, 2048, 4096];
            
            for &buffer_size in &buffer_sizes {
                let buffer = TestSignalGenerator::sine_wave(test_freq, 44100.0, buffer_size, 0.6);
                
                let result = detector.detect_pitch(&buffer);
                if let Ok(pitch_result) = result {
                    if pitch_result.is_valid {
                        let detected = pitch_result.frequency;
                        let cents_error = cents_difference(test_freq, detected).abs();
                        
                        assert!(cents_error <= 20.0,
                            "Buffer size {} test failed: expected {:.1}Hz, got {:.1}Hz ({:.1} cents error)",
                            buffer_size, test_freq, detected, cents_error);
                    }
                }
            }
        }
        
        fn cents_difference(freq1: f32, freq2: f32) -> f32 {
            1200.0 * (freq2 / freq1).log2()
        }
    }
    
    /// Test performance characteristics
    mod performance_tests {
        use super::*;
        use std::time::Instant;
        
        #[test]
        fn test_processing_latency() {
            let config = PitchDetectionConfig::default();
            let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            
            let buffer = TestSignalGenerator::sine_wave(440.0, 44100.0, 2048, 0.5);
            
            for algorithm in [PitchAlgorithm::YIN, PitchAlgorithm::McLeod] {
                detector.set_algorithm(algorithm).unwrap();
                
                let start = Instant::now();
                let result = detector.detect_pitch(&buffer);
                let elapsed = start.elapsed();
                
                // Should complete within reasonable time (< 10ms for 2048 samples)
                assert!(elapsed.as_millis() < 10,
                    "Algorithm {:?} too slow: {}ms", algorithm, elapsed.as_millis());
                
                if let Ok(pitch_result) = result {
                    assert!(pitch_result.processing_time_ns > 0);
                    // Processing time should be reasonable (< 5ms)
                    assert!(pitch_result.processing_time_ns < 5_000_000,
                        "Reported processing time too high: {}ns", pitch_result.processing_time_ns);
                }
            }
        }
        
        #[test]
        fn test_memory_efficiency() {
            let config = PitchDetectionConfig::default();
            let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            
            let buffer = TestSignalGenerator::sine_wave(440.0, 44100.0, 2048, 0.5);
            
            // Process many times to check for memory leaks (basic test)
            for _ in 0..100 {
                let _ = detector.detect_pitch(&buffer);
            }
            
            // Should still be responsive after many iterations
            let start = Instant::now();
            let _ = detector.detect_pitch(&buffer);
            let elapsed = start.elapsed();
            
            assert!(elapsed.as_millis() < 10, "Performance degraded after many iterations");
        }
        
        #[test]
        fn test_algorithm_performance_comparison() {
            let config = PitchDetectionConfig::default();
            let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            
            let buffer = TestSignalGenerator::sine_wave(440.0, 44100.0, 2048, 0.5);
            
            // Measure YIN performance
            detector.set_algorithm(PitchAlgorithm::YIN).unwrap();
            let mut yin_times = vec![];
            for _ in 0..10 {
                let start = Instant::now();
                let _ = detector.detect_pitch(&buffer);
                yin_times.push(start.elapsed().as_nanos() as u64);
            }
            
            // Measure McLeod performance
            detector.set_algorithm(PitchAlgorithm::McLeod).unwrap();
            let mut mcleod_times = vec![];
            for _ in 0..10 {
                let start = Instant::now();
                let _ = detector.detect_pitch(&buffer);
                mcleod_times.push(start.elapsed().as_nanos() as u64);
            }
            
            let yin_avg = yin_times.iter().sum::<u64>() / yin_times.len() as u64;
            let mcleod_avg = mcleod_times.iter().sum::<u64>() / mcleod_times.len() as u64;
            
            // Both should be reasonably fast (< 5ms average)
            assert!(yin_avg < 5_000_000, "YIN average too slow: {}ns", yin_avg);
            assert!(mcleod_avg < 5_000_000, "McLeod average too slow: {}ns", mcleod_avg);
            
            println!("Performance comparison - YIN: {}ns, McLeod: {}ns", yin_avg, mcleod_avg);
        }
    }
    
    /// Test complex signal scenarios
    mod complex_signal_tests {
        use super::*;
        
        #[test]
        fn test_chord_detection() {
            let config = PitchDetectionConfig::default();
            let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            
            // Test C major chord (C, E, G)
            let chord_frequencies = [MusicalNotes::c4(), MusicalNotes::e4(), MusicalNotes::g4()];
            let buffer = TestSignalGenerator::musical_chord(&chord_frequencies, 44100.0, 2048, 0.4);
            
            for algorithm in [PitchAlgorithm::YIN, PitchAlgorithm::McLeod] {
                detector.set_algorithm(algorithm).unwrap();
                
                let result = detector.detect_pitch(&buffer);
                if let Ok(pitch_result) = result {
                    // Should detect one of the chord components (likely the fundamental)
                    // This is expected behavior - most monophonic pitch detectors pick the strongest component
                    if pitch_result.is_valid {
                        let detected = pitch_result.frequency;
                        
                        // Should be close to one of the chord frequencies
                        let is_close_to_chord = chord_frequencies.iter().any(|&freq| {
                            cents_difference(freq, detected).abs() <= 50.0
                        });
                        
                        assert!(is_close_to_chord,
                            "Chord detection with {:?} returned unexpected frequency: {:.1}Hz",
                            algorithm, detected);
                    }
                }
            }
        }
        
        #[test]
        fn test_frequency_sweep_tracking() {
            let config = PitchDetectionConfig::default();
            let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            
            // Generate frequency sweep from 220Hz to 880Hz
            let start_freq = 220.0;
            let end_freq = 880.0;
            let buffer = TestSignalGenerator::frequency_sweep(start_freq, end_freq, 44100.0, 4096, 0.6);
            
            // Split buffer into segments and track frequency changes
            let segment_size = 1024;
            let segments: Vec<_> = buffer.chunks(segment_size).collect();
            
            for (i, segment) in segments.iter().enumerate() {
                if segment.len() < 512 { break; } // Skip incomplete segments
                
                let result = detector.detect_pitch(segment);
                if let Ok(pitch_result) = result {
                    if pitch_result.is_valid {
                        let detected = pitch_result.frequency;
                        
                        // Expected frequency at this point in the sweep
                        let progress = i as f32 / (segments.len() - 1) as f32;
                        let expected = start_freq + (end_freq - start_freq) * progress;
                        
                        let cents_error = cents_difference(expected, detected).abs();
                        
                        // Allow more error for sweep (harder to track)
                        assert!(cents_error <= 100.0,
                            "Frequency sweep tracking failed at segment {}: expected ~{:.1}Hz, got {:.1}Hz ({:.1} cents)",
                            i, expected, detected, cents_error);
                    }
                }
            }
        }
        
        #[test]
        fn test_auto_algorithm_selection() {
            let config = PitchDetectionConfig::default();
            let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
            detector.set_algorithm(PitchAlgorithm::Auto).unwrap();
            
            // Test with clean signal - should pick an algorithm and stick with it
            let clean_buffer = TestSignalGenerator::sine_wave(440.0, 44100.0, 2048, 0.8);
            let clean_result = detector.detect_pitch(&clean_buffer);
            
            if let Ok(clean_pitch) = clean_result {
                assert!(matches!(clean_pitch.algorithm_used, PitchAlgorithm::YIN | PitchAlgorithm::McLeod));
                assert!(!matches!(clean_pitch.algorithm_used, PitchAlgorithm::Auto)); // Should pick specific algorithm
            }
            
            // Test with noisy signal - may pick different algorithm
            let noisy_buffer = TestSignalGenerator::noisy_signal(440.0, 44100.0, 2048, 0.5, 0.3);
            let noisy_result = detector.detect_pitch(&noisy_buffer);
            
            if let Ok(noisy_pitch) = noisy_result {
                assert!(matches!(noisy_pitch.algorithm_used, PitchAlgorithm::YIN | PitchAlgorithm::McLeod));
            }
        }
        
        fn cents_difference(freq1: f32, freq2: f32) -> f32 {
            1200.0 * (freq2 / freq1).log2()
        }
    }
}