//! Signal Generator Tests - STORY-016
//! 
//! Comprehensive test suite for signal generation functionality

#[cfg(test)]
mod tests {
    use super::super::signal_generator::*;
    use std::f64::consts::PI;

    const SAMPLE_RATE: u32 = 44100;
    const TEST_FREQUENCY: f64 = 440.0; // A4
    const TEST_AMPLITUDE: f32 = 0.5;
    const TEST_DURATION: u32 = 100; // 100ms

    #[test]
    fn test_sine_wave_generation() {
        let generator = WebSignalGenerator::new(SAMPLE_RATE);
        let result = generator.generate_sine(TEST_FREQUENCY, TEST_AMPLITUDE, TEST_DURATION);
        
        assert!(result.is_ok());
        let signal = result.unwrap();
        
        // Verify buffer size
        let expected_samples = (SAMPLE_RATE as f64 * TEST_DURATION as f64 / 1000.0) as usize;
        assert_eq!(signal.len(), expected_samples);
        
        // Verify amplitude range
        for sample in &signal {
            assert!(*sample >= -TEST_AMPLITUDE && *sample <= TEST_AMPLITUDE);
        }
        
        // Verify sine wave properties (should cross zero at expected intervals)
        let period_samples = SAMPLE_RATE as f64 / TEST_FREQUENCY;
        let zero_crossing_sample = (period_samples / 2.0) as usize;
        
        if signal.len() > zero_crossing_sample {
            // At half period, sine should be close to zero
            assert!(signal[zero_crossing_sample].abs() < 0.1);
        }
    }

    #[test]
    fn test_sawtooth_wave_generation() {
        let generator = WebSignalGenerator::new(SAMPLE_RATE);
        let result = generator.generate_sawtooth(TEST_FREQUENCY, TEST_AMPLITUDE, TEST_DURATION);
        
        assert!(result.is_ok());
        let signal = result.unwrap();
        
        // Verify buffer size
        let expected_samples = (SAMPLE_RATE as f64 * TEST_DURATION as f64 / 1000.0) as usize;
        assert_eq!(signal.len(), expected_samples);
        
        // Verify amplitude range
        for sample in &signal {
            assert!(*sample >= -TEST_AMPLITUDE && *sample <= TEST_AMPLITUDE);
        }
        
        // Verify sawtooth properties (linear ramp)
        let period_samples = SAMPLE_RATE as f64 / TEST_FREQUENCY;
        if signal.len() > period_samples as usize {
            let first_sample = signal[0];
            let quarter_period_sample = signal[(period_samples / 4.0) as usize];
            let half_period_sample = signal[(period_samples / 2.0) as usize];
            
            // Should increase linearly
            assert!(quarter_period_sample > first_sample);
            assert!(half_period_sample > quarter_period_sample);
        }
    }

    #[test]
    fn test_square_wave_generation() {
        let generator = WebSignalGenerator::new(SAMPLE_RATE);
        let result = generator.generate_square(TEST_FREQUENCY, TEST_AMPLITUDE, TEST_DURATION);
        
        assert!(result.is_ok());
        let signal = result.unwrap();
        
        // Verify buffer size
        let expected_samples = (SAMPLE_RATE as f64 * TEST_DURATION as f64 / 1000.0) as usize;
        assert_eq!(signal.len(), expected_samples);
        
        // Verify square wave properties (should only have two values)
        for sample in &signal {
            assert!(*sample == TEST_AMPLITUDE || *sample == -TEST_AMPLITUDE);
        }
        
        // Verify transitions occur at expected intervals
        let period_samples = SAMPLE_RATE as f64 / TEST_FREQUENCY;
        let half_period = (period_samples / 2.0) as usize;
        
        if signal.len() > half_period {
            let first_half = signal[0];
            let second_half = signal[half_period];
            
            // Should be opposite values
            assert_ne!(first_half, second_half);
        }
    }

    #[test]
    fn test_triangle_wave_generation() {
        let generator = WebSignalGenerator::new(SAMPLE_RATE);
        let result = generator.generate_triangle(TEST_FREQUENCY, TEST_AMPLITUDE, TEST_DURATION);
        
        assert!(result.is_ok());
        let signal = result.unwrap();
        
        // Verify buffer size
        let expected_samples = (SAMPLE_RATE as f64 * TEST_DURATION as f64 / 1000.0) as usize;
        assert_eq!(signal.len(), expected_samples);
        
        // Verify amplitude range
        for sample in &signal {
            assert!(*sample >= -TEST_AMPLITUDE && *sample <= TEST_AMPLITUDE);
        }
        
        // Verify triangle properties (linear increase/decrease)
        let period_samples = SAMPLE_RATE as f64 / TEST_FREQUENCY;
        if signal.len() > period_samples as usize {
            let quarter_period = (period_samples / 4.0) as usize;
            let half_period = (period_samples / 2.0) as usize;
            let three_quarter_period = (3.0 * period_samples / 4.0) as usize;
            
            // Should increase to quarter, peak at half, decrease at three quarter
            if signal.len() > three_quarter_period {
                assert!(signal[quarter_period] > signal[0]);
                assert!(signal[half_period] > signal[quarter_period]);
                assert!(signal[three_quarter_period] < signal[half_period]);
            }
        }
    }

    #[test]
    fn test_frequency_sweep_generation() {
        let generator = WebSignalGenerator::new(SAMPLE_RATE);
        let start_freq = 100.0;
        let end_freq = 1000.0;
        let result = generator.generate_sweep(start_freq, end_freq, TEST_AMPLITUDE, 500); // 500ms
        
        assert!(result.is_ok());
        let signal = result.unwrap();
        
        // Verify buffer size
        let expected_samples = (SAMPLE_RATE as f64 * 500.0 / 1000.0) as usize;
        assert_eq!(signal.len(), expected_samples);
        
        // Verify amplitude range
        for sample in &signal {
            assert!(*sample >= -TEST_AMPLITUDE && *sample <= TEST_AMPLITUDE);
        }
        
        // Sweep should have varying instantaneous frequency
        assert!(!signal.is_empty());
    }

    #[test]
    fn test_white_noise_generation() {
        let generator = WebSignalGenerator::new(SAMPLE_RATE);
        let result = generator.generate_white_noise(TEST_AMPLITUDE, TEST_DURATION);
        
        assert!(result.is_ok());
        let signal = result.unwrap();
        
        // Verify buffer size
        let expected_samples = (SAMPLE_RATE as f64 * TEST_DURATION as f64 / 1000.0) as usize;
        assert_eq!(signal.len(), expected_samples);
        
        // Verify amplitude range
        for sample in &signal {
            assert!(*sample >= -TEST_AMPLITUDE && *sample <= TEST_AMPLITUDE);
        }
        
        // Verify randomness (consecutive samples should generally be different)
        let mut different_samples = 0;
        for i in 0..signal.len().saturating_sub(1) {
            if signal[i] != signal[i + 1] {
                different_samples += 1;
            }
        }
        
        // At least 95% of consecutive samples should be different
        let min_different = (signal.len() * 95) / 100;
        assert!(different_samples >= min_different);
    }

    #[test]
    fn test_pink_noise_generation() {
        let generator = WebSignalGenerator::new(SAMPLE_RATE);
        let result = generator.generate_pink_noise(TEST_AMPLITUDE, TEST_DURATION);
        
        assert!(result.is_ok());
        let signal = result.unwrap();
        
        // Verify buffer size
        let expected_samples = (SAMPLE_RATE as f64 * TEST_DURATION as f64 / 1000.0) as usize;
        assert_eq!(signal.len(), expected_samples);
        
        // Pink noise should have different characteristics than white noise
        // but same basic validation
        for sample in &signal {
            assert!(*sample >= -TEST_AMPLITUDE && *sample <= TEST_AMPLITUDE);
        }
    }

    #[test]
    fn test_parameter_validation() {
        let generator = WebSignalGenerator::new(SAMPLE_RATE);
        
        // Invalid frequency (too high)
        let result = generator.generate_sine(25000.0, TEST_AMPLITUDE, TEST_DURATION);
        assert!(result.is_err());
        assert!(matches!(result, Err(SignalError::InvalidFrequency(_))));
        
        // Invalid frequency (negative)
        let result = generator.generate_sine(-100.0, TEST_AMPLITUDE, TEST_DURATION);
        assert!(result.is_err());
        assert!(matches!(result, Err(SignalError::InvalidFrequency(_))));
        
        // Invalid amplitude (too high)
        let result = generator.generate_sine(TEST_FREQUENCY, 2.0, TEST_DURATION);
        assert!(result.is_err());
        assert!(matches!(result, Err(SignalError::InvalidAmplitude(_))));
        
        // Invalid amplitude (negative)
        let result = generator.generate_sine(TEST_FREQUENCY, -0.5, TEST_DURATION);
        assert!(result.is_err());
        assert!(matches!(result, Err(SignalError::InvalidAmplitude(_))));
        
        // Invalid duration (zero)
        let result = generator.generate_sine(TEST_FREQUENCY, TEST_AMPLITUDE, 0);
        assert!(result.is_err());
        assert!(matches!(result, Err(SignalError::InvalidDuration(_))));
        
        // Invalid duration (too long)
        let result = generator.generate_sine(TEST_FREQUENCY, TEST_AMPLITUDE, 400_000);
        assert!(result.is_err());
        assert!(matches!(result, Err(SignalError::InvalidDuration(_))));
    }

    #[test]
    fn test_real_time_generation_lifecycle() {
        let mut generator = WebSignalGenerator::new(SAMPLE_RATE);
        
        // Initially not active
        assert!(!generator.is_real_time_active());
        
        // Start real-time generation
        let config = SignalConfig {
            waveform: WaveformType::Sine,
            frequency: TEST_FREQUENCY,
            amplitude: TEST_AMPLITUDE,
            duration_ms: None, // Continuous
            sample_rate: SAMPLE_RATE,
            phase_offset: 0.0,
            sweep_end_freq: None,
        };
        
        let result = generator.start_real_time_generation(config);
        assert!(result.is_ok());
        assert!(generator.is_real_time_active());
        
        // Generate buffers
        let buffer_size = 1024;
        let result = generator.get_next_buffer(buffer_size);
        assert!(result.is_ok());
        let buffer1 = result.unwrap();
        assert_eq!(buffer1.len(), buffer_size);
        
        // Generate another buffer (should be continuous)
        let result = generator.get_next_buffer(buffer_size);
        assert!(result.is_ok());
        let buffer2 = result.unwrap();
        assert_eq!(buffer2.len(), buffer_size);
        
        // Buffers should be different (continuous signal)
        assert_ne!(buffer1, buffer2);
        
        // Stop generation
        let result = generator.stop_real_time_generation();
        assert!(result.is_ok());
        assert!(!generator.is_real_time_active());
        
        // Should fail to generate after stopping
        let result = generator.get_next_buffer(buffer_size);
        assert!(result.is_err());
        assert!(matches!(result, Err(SignalError::RealTimeError(_))));
    }

    #[test]
    fn test_real_time_waveform_types() {
        let mut generator = WebSignalGenerator::new(SAMPLE_RATE);
        let buffer_size = 512;
        
        let waveforms = vec![
            WaveformType::Sine,
            WaveformType::Sawtooth,
            WaveformType::Square,
            WaveformType::Triangle,
            WaveformType::PinkNoise,
            WaveformType::WhiteNoise,
        ];
        
        for waveform in waveforms {
            let config = SignalConfig {
                waveform,
                frequency: TEST_FREQUENCY,
                amplitude: TEST_AMPLITUDE,
                duration_ms: None,
                sample_rate: SAMPLE_RATE,
                phase_offset: 0.0,
                sweep_end_freq: None,
            };
            
            let result = generator.start_real_time_generation(config);
            assert!(result.is_ok(), "Failed to start {} generation", format!("{:?}", waveform));
            
            let result = generator.get_next_buffer(buffer_size);
            assert!(result.is_ok(), "Failed to generate {} buffer", format!("{:?}", waveform));
            
            let buffer = result.unwrap();
            assert_eq!(buffer.len(), buffer_size);
            
            // Verify amplitude range
            for sample in &buffer {
                assert!(*sample >= -TEST_AMPLITUDE && *sample <= TEST_AMPLITUDE,
                    "Amplitude out of range for {:?}: {}", waveform, sample);
            }
            
            generator.stop_real_time_generation().unwrap();
        }
    }

    #[test]
    fn test_phase_accumulator_continuity() {
        let mut generator = WebSignalGenerator::new(SAMPLE_RATE);
        
        let config = SignalConfig {
            waveform: WaveformType::Sine,
            frequency: TEST_FREQUENCY,
            amplitude: TEST_AMPLITUDE,
            duration_ms: None,
            sample_rate: SAMPLE_RATE,
            phase_offset: 0.0,
            sweep_end_freq: None,
        };
        
        generator.start_real_time_generation(config).unwrap();
        
        // Generate several small buffers
        let buffer_size = 100;
        let mut combined_signal = Vec::new();
        
        for _ in 0..10 {
            let buffer = generator.get_next_buffer(buffer_size).unwrap();
            combined_signal.extend_from_slice(&buffer);
        }
        
        // Signal should be continuous (no discontinuities)
        let mut max_discontinuity: f32 = 0.0;
        for i in 1..combined_signal.len() {
            let diff = (combined_signal[i] - combined_signal[i-1]).abs();
            max_discontinuity = max_discontinuity.max(diff);
        }
        
        // For a sine wave at 440Hz with reasonable buffer size,
        // discontinuities should be small
        assert!(max_discontinuity < 0.5, "Large discontinuity detected: {}", max_discontinuity);
        
        generator.stop_real_time_generation().unwrap();
    }

    #[test]
    fn test_test_signal_library_musical_notes() {
        let library = TestSignalLibrary::new(SAMPLE_RATE);
        
        // Test various notes
        let notes = vec![
            ("A", 4, 440.0),    // A4 = 440Hz
            ("C", 4, 261.626),  // Middle C
            ("A", 0, 27.5),     // A0
            ("G#", 7, 3322.438), // G#7
        ];
        
        for (note, octave, expected_freq) in notes {
            let result = library.generate_musical_note(note, octave, 100);
            assert!(result.is_ok(), "Failed to generate note {}{}", note, octave);
            
            let signal = result.unwrap();
            assert!(!signal.is_empty());
            
            // Verify frequency is approximately correct by checking note conversion
            let freq_result = library.note_to_frequency(note, octave);
            assert!(freq_result.is_ok());
            let freq = freq_result.unwrap();
            assert!((freq - expected_freq).abs() < 1.0, 
                "Frequency mismatch for {}{}: expected {}, got {}", note, octave, expected_freq, freq);
        }
    }

    #[test]
    fn test_test_signal_library_chord_generation() {
        let library = TestSignalLibrary::new(SAMPLE_RATE);
        
        // C major chord (C-E-G)
        let c_major_frequencies = vec![261.626, 329.628, 391.995];
        let result = library.generate_chord(&c_major_frequencies, 200);
        
        assert!(result.is_ok());
        let signal = result.unwrap();
        
        // Should have appropriate length
        let expected_samples = (SAMPLE_RATE as f64 * 200.0 / 1000.0) as usize;
        assert_eq!(signal.len(), expected_samples);
        
        // Signal should not be empty and should have reasonable amplitude
        assert!(!signal.is_empty());
        
        // Test empty frequencies
        let result = library.generate_chord(&[], 100);
        assert!(result.is_err());
        assert!(matches!(result, Err(SignalError::InvalidFrequency(_))));
    }

    #[test]
    fn test_test_signal_library_tuning_references() {
        let library = TestSignalLibrary::new(SAMPLE_RATE);
        
        let references = vec![
            (TuningReference::A440, 440.0),
            (TuningReference::A442, 442.0),
            (TuningReference::A432, 432.0),
            (TuningReference::MiddleC, 261.626),
        ];
        
        for (reference, expected_freq) in references {
            let result = library.generate_tuning_reference(reference, 100);
            assert!(result.is_ok(), "Failed to generate tuning reference {:?}", reference);
            
            let signal = result.unwrap();
            assert!(!signal.is_empty());
            
            // All tuning references should generate valid signals
            for sample in &signal {
                assert!(sample.abs() <= 0.7); // Should be within amplitude range
            }
        }
    }

    #[test]
    fn test_signal_injector_lifecycle() {
        let mut injector = SignalInjector::new(SAMPLE_RATE);
        
        // Initially not active
        assert!(!injector.is_injection_active());
        
        // Start injection
        let config = SignalConfig {
            waveform: WaveformType::Sine,
            frequency: TEST_FREQUENCY,
            amplitude: 0.3,
            duration_ms: None,
            sample_rate: SAMPLE_RATE,
            phase_offset: 0.0,
            sweep_end_freq: None,
        };
        
        let result = injector.start_injection(config);
        assert!(result.is_ok());
        assert!(injector.is_injection_active());
        
        // Test buffer injection
        let mut audio_buffer = vec![0.1; 1024]; // Original signal
        let original_buffer = audio_buffer.clone();
        
        let result = injector.inject_into_buffer(&mut audio_buffer, 0.5); // 50% mix
        assert!(result.is_ok());
        
        // Buffer should be modified
        assert_ne!(audio_buffer, original_buffer);
        
        // Stop injection
        let result = injector.stop_injection();
        assert!(result.is_ok());
        assert!(!injector.is_injection_active());
        
        // Should not modify buffer when inactive
        let mut test_buffer = vec![0.1; 512];
        let original_test_buffer = test_buffer.clone();
        
        let result = injector.inject_into_buffer(&mut test_buffer, 0.5);
        assert!(result.is_ok());
        assert_eq!(test_buffer, original_test_buffer); // Should be unchanged
    }

    #[test]
    fn test_signal_injector_mix_ratios() {
        let mut injector = SignalInjector::new(SAMPLE_RATE);
        
        let config = SignalConfig {
            waveform: WaveformType::Sine,
            frequency: TEST_FREQUENCY,
            amplitude: 1.0,
            duration_ms: None,
            sample_rate: SAMPLE_RATE,
            phase_offset: 0.0,
            sweep_end_freq: None,
        };
        
        injector.start_injection(config).unwrap();
        
        // Test different mix ratios
        let original_value = 0.5;
        
        // 100% replacement (mix_ratio = 0.0)
        let mut buffer = vec![original_value; 100];
        injector.inject_into_buffer(&mut buffer, 0.0).unwrap();
        // Should be completely replaced
        for sample in &buffer {
            assert_ne!(*sample, original_value);
        }
        
        // 100% original (mix_ratio = 1.0)
        let mut buffer = vec![original_value; 100];
        injector.inject_into_buffer(&mut buffer, 1.0).unwrap();
        // Should be unchanged
        for sample in &buffer {
            assert_eq!(*sample, original_value);
        }
        
        // 50% mix (mix_ratio = 0.5)
        let mut buffer = vec![original_value; 100];
        injector.inject_into_buffer(&mut buffer, 0.5).unwrap();
        // Should be modified but not completely replaced
        let mut has_original = false;
        let mut has_different = false;
        for sample in &buffer {
            if (*sample - original_value).abs() < 0.001 {
                has_original = true;
            } else {
                has_different = true;
            }
        }
        // For a sine wave, we should have both original and modified values
        // depending on the phase of the sine wave
        assert!(has_different, "Mix should produce different values");
        
        injector.stop_injection().unwrap();
    }

    #[test]
    fn test_edge_cases_and_robustness() {
        let generator = WebSignalGenerator::new(SAMPLE_RATE);
        
        // Very short duration
        let result = generator.generate_sine(TEST_FREQUENCY, TEST_AMPLITUDE, 1);
        assert!(result.is_ok());
        let signal = result.unwrap();
        assert!(!signal.is_empty());
        
        // Very low frequency
        let result = generator.generate_sine(1.0, TEST_AMPLITUDE, 1000);
        assert!(result.is_ok());
        
        // Very high frequency (but within valid range)
        let max_freq = (SAMPLE_RATE / 2) as f64 - 1.0;
        let result = generator.generate_sine(max_freq, TEST_AMPLITUDE, 100);
        assert!(result.is_ok());
        
        // Very low amplitude
        let result = generator.generate_sine(TEST_FREQUENCY, 0.001, TEST_DURATION);
        assert!(result.is_ok());
        let signal = result.unwrap();
        for sample in &signal {
            assert!(sample.abs() <= 0.001);
        }
        
        // Maximum amplitude
        let result = generator.generate_sine(TEST_FREQUENCY, 1.0, TEST_DURATION);
        assert!(result.is_ok());
        let signal = result.unwrap();
        for sample in &signal {
            assert!(sample.abs() <= 1.0);
        }
    }

    #[test]
    fn test_signal_quality_metrics() {
        let generator = WebSignalGenerator::new(SAMPLE_RATE);
        
        // Generate pure sine wave
        let result = generator.generate_sine(TEST_FREQUENCY, 1.0, 1000);
        assert!(result.is_ok());
        let signal = result.unwrap();
        
        // Calculate RMS amplitude
        let rms = (signal.iter().map(|x| x * x).sum::<f32>() / signal.len() as f32).sqrt();
        let expected_rms = 1.0 / (2.0_f32).sqrt(); // RMS of sine wave
        assert!((rms - expected_rms).abs() < 0.01, "RMS mismatch: expected {}, got {}", expected_rms, rms);
        
        // Check frequency content (zero crossings)
        let mut zero_crossings = 0;
        for i in 1..signal.len() {
            if signal[i-1] * signal[i] < 0.0 {
                zero_crossings += 1;
            }
        }
        
        // For a 1-second 440Hz sine wave, expect ~880 zero crossings
        let duration_sec = 1.0;
        let expected_crossings = (2.0 * TEST_FREQUENCY * duration_sec) as usize;
        let tolerance = expected_crossings / 10; // 10% tolerance
        
        assert!((zero_crossings as i32 - expected_crossings as i32).abs() < tolerance as i32,
            "Zero crossings mismatch: expected ~{}, got {}", expected_crossings, zero_crossings);
    }

    #[test]
    fn test_note_conversion_edge_cases() {
        let library = TestSignalLibrary::new(SAMPLE_RATE);
        
        // Test invalid notes
        let result = library.note_to_frequency("X", 4);
        assert!(result.is_err());
        assert!(matches!(result, Err(SignalError::InvalidFrequency(_))));
        
        // Test invalid octave
        let result = library.note_to_frequency("A", 10);
        assert!(result.is_err());
        assert!(matches!(result, Err(SignalError::InvalidFrequency(_))));
        
        // Test sharp and flat notes
        let result = library.note_to_frequency("C#", 4);
        assert!(result.is_ok());
        
        let result = library.note_to_frequency("Db", 4);
        assert!(result.is_ok());
        
        // C# and Db should give same frequency
        let cs_freq = library.note_to_frequency("C#", 4).unwrap();
        let db_freq = library.note_to_frequency("Db", 4).unwrap();
        assert!((cs_freq - db_freq).abs() < 0.001);
    }
} 