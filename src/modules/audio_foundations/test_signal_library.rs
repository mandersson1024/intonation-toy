// Test Signal Library - STORY-016
// Pre-recorded test signals for automated testing and development

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use super::signal_generator::{SignalGenerator, WebSignalGenerator, WaveformType, SignalConfig};

/// Test signal library for automated testing and development
pub struct TestSignalLibrary {
    signals: HashMap<String, TestSignal>,
    generator: Arc<Mutex<WebSignalGenerator>>,
}

/// Pre-recorded test signal with metadata
#[derive(Debug, Clone)]
pub struct TestSignal {
    pub name: String,
    pub description: String,
    pub samples: Vec<f32>,
    pub sample_rate: f32,
    pub expected_frequency: Option<f32>,
    pub expected_amplitude: f32,
    pub signal_type: TestSignalType,
    pub duration_ms: u32,
}

/// Types of test signals available
#[derive(Debug, Clone, PartialEq)]
pub enum TestSignalType {
    PureTone,
    ComplexTone,
    Sweep,
    Noise,
    Musical,
    Calibration,
}

impl TestSignalLibrary {
    /// Create new test signal library
    pub fn new(sample_rate: f32) -> Self {
        let mut library = Self {
            signals: HashMap::new(),
            generator: Arc::new(Mutex::new(WebSignalGenerator::new(sample_rate))),
        };
        
        library.generate_standard_signals();
        library
    }
    
    /// Generate standard test signals
    fn generate_standard_signals(&mut self) {
        self.add_musical_notes();
        self.add_calibration_tones();
        self.add_complex_signals();
        self.add_noise_signals();
        self.add_sweep_signals();
    }
    
    /// Add musical note test signals
    fn add_musical_notes(&mut self) {
        let musical_notes = vec![
            ("A4", 440.0, "Concert pitch A4"),
            ("C4", 261.63, "Middle C"),
            ("E4", 329.63, "E above middle C"),
            ("G4", 392.00, "G above middle C"),
            ("A3", 220.0, "A3 below concert pitch"),
            ("A5", 880.0, "A5 above concert pitch"),
            ("C5", 523.25, "C5 high C"),
            ("F#4", 369.99, "F sharp 4"),
            ("Bb4", 466.16, "B flat 4"),
            ("D4", 293.66, "D above middle C"),
        ];
        
        for (note_name, frequency, description) in musical_notes {
            let generator = self.generator.lock().unwrap();
            let samples = generator.generate_sine(frequency, 0.7, 2000); // 2 second duration
            drop(generator);
            
            let signal = TestSignal {
                name: format!("musical_{}", note_name.to_lowercase()),
                description: format!("{} - {}", description, note_name),
                samples,
                sample_rate: 44100.0,
                expected_frequency: Some(frequency as f32),
                expected_amplitude: 0.7,
                signal_type: TestSignalType::Musical,
                duration_ms: 2000,
            };
            
            self.signals.insert(signal.name.clone(), signal);
        }
    }
    
    /// Add calibration tone signals
    fn add_calibration_tones(&mut self) {
        let calibration_frequencies = vec![
            (100.0, "Low frequency calibration"),
            (250.0, "Low-mid frequency calibration"),
            (440.0, "Reference frequency A4"),
            (1000.0, "1kHz calibration tone"),
            (2000.0, "2kHz calibration tone"),
            (4000.0, "4kHz calibration tone"),
            (8000.0, "High frequency calibration"),
        ];
        
        for (frequency, description) in calibration_frequencies {
            let generator = self.generator.lock().unwrap();
            let samples = generator.generate_sine(frequency, 0.5, 5000); // 5 second duration
            drop(generator);
            
            let signal = TestSignal {
                name: format!("calibration_{}hz", frequency as u32),
                description: format!("{} - {}Hz", description, frequency),
                samples,
                sample_rate: 44100.0,
                expected_frequency: Some(frequency as f32),
                expected_amplitude: 0.5,
                signal_type: TestSignalType::Calibration,
                duration_ms: 5000,
            };
            
            self.signals.insert(signal.name.clone(), signal);
        }
    }
    
    /// Add complex signal test cases
    fn add_complex_signals(&mut self) {
        // Complex tones with harmonics (simulated with multiple sine waves)
        let complex_tones = vec![
            (220.0, "Complex A3 with harmonics"),
            (440.0, "Complex A4 with harmonics"),
            (880.0, "Complex A5 with harmonics"),
        ];
        
        for (fundamental, description) in complex_tones {
            let generator = self.generator.lock().unwrap();
            let mut samples = generator.generate_sine(fundamental, 0.4, 3000);
            
            // Add harmonics
            let harmonic2 = generator.generate_sine(fundamental * 2.0, 0.2, 3000);
            let harmonic3 = generator.generate_sine(fundamental * 3.0, 0.1, 3000);
            
            // Mix harmonics
            for (i, sample) in samples.iter_mut().enumerate() {
                if i < harmonic2.len() {
                    *sample += harmonic2[i];
                }
                if i < harmonic3.len() {
                    *sample += harmonic3[i];
                }
                // Normalize to prevent clipping
                *sample *= 0.8;
            }
            drop(generator);
            
            let signal = TestSignal {
                name: format!("complex_{}hz", fundamental as u32),
                description: format!("{} - {}Hz", description, fundamental),
                samples,
                sample_rate: 44100.0,
                expected_frequency: Some(fundamental as f32),
                expected_amplitude: 0.8,
                signal_type: TestSignalType::ComplexTone,
                duration_ms: 3000,
            };
            
            self.signals.insert(signal.name.clone(), signal);
        }
    }
    
    /// Add noise signals for testing
    fn add_noise_signals(&mut self) {
        let generator = self.generator.lock().unwrap();
        
        // Pink noise
        let pink_noise = generator.generate_noise(0.3, 10000); // 10 second duration
        let pink_signal = TestSignal {
            name: "pink_noise".to_string(),
            description: "Pink noise for testing noise handling".to_string(),
            samples: pink_noise,
            sample_rate: 44100.0,
            expected_frequency: None,
            expected_amplitude: 0.3,
            signal_type: TestSignalType::Noise,
            duration_ms: 10000,
        };
        self.signals.insert(pink_signal.name.clone(), pink_signal);
        
        // Low amplitude noise
        let quiet_noise = generator.generate_noise(0.05, 5000);
        let quiet_signal = TestSignal {
            name: "quiet_noise".to_string(),
            description: "Low amplitude noise for sensitivity testing".to_string(),
            samples: quiet_noise,
            sample_rate: 44100.0,
            expected_frequency: None,
            expected_amplitude: 0.05,
            signal_type: TestSignalType::Noise,
            duration_ms: 5000,
        };
        self.signals.insert(quiet_signal.name.clone(), quiet_signal);
        
        drop(generator);
    }
    
    /// Add frequency sweep signals
    fn add_sweep_signals(&mut self) {
        let sweep_configs = vec![
            (100.0, 1000.0, "Low to mid frequency sweep"),
            (1000.0, 8000.0, "Mid to high frequency sweep"),
            (50.0, 5000.0, "Full range frequency sweep"),
            (440.0, 880.0, "Octave sweep A4 to A5"),
        ];
        
        for (start_freq, end_freq, description) in sweep_configs {
            let generator = self.generator.lock().unwrap();
            let samples = generator.generate_sweep(start_freq, end_freq, 0.6, 8000); // 8 second duration
            drop(generator);
            
            let signal = TestSignal {
                name: format!("sweep_{}hz_to_{}hz", start_freq as u32, end_freq as u32),
                description: format!("{} - {}Hz to {}Hz", description, start_freq, end_freq),
                samples,
                sample_rate: 44100.0,
                expected_frequency: Some(((start_freq + end_freq) / 2.0) as f32), // Average frequency
                expected_amplitude: 0.6,
                signal_type: TestSignalType::Sweep,
                duration_ms: 8000,
            };
            
            self.signals.insert(signal.name.clone(), signal);
        }
    }
    
    /// Get test signal by name
    pub fn get_signal(&self, name: &str) -> Option<&TestSignal> {
        self.signals.get(name)
    }
    
    /// Get all available signal names
    pub fn get_signal_names(&self) -> Vec<String> {
        self.signals.keys().cloned().collect()
    }
    
    /// Get signals by type
    pub fn get_signals_by_type(&self, signal_type: TestSignalType) -> Vec<&TestSignal> {
        self.signals.values()
            .filter(|signal| signal.signal_type == signal_type)
            .collect()
    }
    
    /// Get musical note signals
    pub fn get_musical_notes(&self) -> Vec<&TestSignal> {
        self.get_signals_by_type(TestSignalType::Musical)
    }
    
    /// Get calibration tones
    pub fn get_calibration_tones(&self) -> Vec<&TestSignal> {
        self.get_signals_by_type(TestSignalType::Calibration)
    }
    
    /// Get complex tone signals
    pub fn get_complex_tones(&self) -> Vec<&TestSignal> {
        self.get_signals_by_type(TestSignalType::ComplexTone)
    }
    
    /// Get noise signals
    pub fn get_noise_signals(&self) -> Vec<&TestSignal> {
        self.get_signals_by_type(TestSignalType::Noise)
    }
    
    /// Get sweep signals
    pub fn get_sweep_signals(&self) -> Vec<&TestSignal> {
        self.get_signals_by_type(TestSignalType::Sweep)
    }
    
    /// Add custom test signal
    pub fn add_custom_signal(&mut self, signal: TestSignal) {
        self.signals.insert(signal.name.clone(), signal);
    }
    
    /// Generate signal on demand
    pub fn generate_signal(&self, config: SignalConfig) -> Option<TestSignal> {
        let generator = self.generator.lock().ok()?;
        
        let samples = match config.waveform {
            WaveformType::Sine => generator.generate_sine(
                config.frequency, 
                config.amplitude, 
                config.duration_ms.unwrap_or(1000)
            ),
            WaveformType::Sawtooth => generator.generate_sawtooth(
                config.frequency, 
                config.amplitude, 
                config.duration_ms.unwrap_or(1000)
            ),
            WaveformType::Square => generator.generate_square(
                config.frequency, 
                config.amplitude, 
                config.duration_ms.unwrap_or(1000)
            ),
            WaveformType::Triangle => generator.generate_triangle(
                config.frequency, 
                config.amplitude, 
                config.duration_ms.unwrap_or(1000)
            ),
            WaveformType::Sweep => {
                let end_freq = config.sweep_end_freq.unwrap_or(config.frequency * 2.0);
                generator.generate_sweep(
                    config.frequency,
                    end_freq,
                    config.amplitude,
                    config.duration_ms.unwrap_or(1000)
                )
            },
            WaveformType::PinkNoise => generator.generate_noise(
                config.amplitude, 
                config.duration_ms.unwrap_or(1000)
            ),
        };
        
        if samples.is_empty() {
            return None;
        }
        
        Some(TestSignal {
            name: format!("custom_{:?}_{:.0}hz", config.waveform, config.frequency),
            description: format!("Custom {:?} signal at {:.0}Hz", config.waveform, config.frequency),
            samples,
            sample_rate: config.sample_rate,
            expected_frequency: if config.waveform != WaveformType::PinkNoise {
                Some(config.frequency as f32)
            } else {
                None
            },
            expected_amplitude: config.amplitude,
            signal_type: match config.waveform {
                WaveformType::PinkNoise => TestSignalType::Noise,
                WaveformType::Sweep => TestSignalType::Sweep,
                _ => TestSignalType::PureTone,
            },
            duration_ms: config.duration_ms.unwrap_or(1000),
        })
    }
    
    /// Get signal statistics
    pub fn get_library_stats(&self) -> LibraryStats {
        let mut stats = LibraryStats {
            total_signals: self.signals.len(),
            musical_count: 0,
            calibration_count: 0,
            complex_count: 0,
            noise_count: 0,
            sweep_count: 0,
            pure_tone_count: 0,
            total_duration_ms: 0,
            avg_amplitude: 0.0,
        };
        
        let mut amplitude_sum = 0.0;
        
        for signal in self.signals.values() {
            match signal.signal_type {
                TestSignalType::Musical => stats.musical_count += 1,
                TestSignalType::Calibration => stats.calibration_count += 1,
                TestSignalType::ComplexTone => stats.complex_count += 1,
                TestSignalType::Noise => stats.noise_count += 1,
                TestSignalType::Sweep => stats.sweep_count += 1,
                TestSignalType::PureTone => stats.pure_tone_count += 1,
            }
            
            stats.total_duration_ms += signal.duration_ms;
            amplitude_sum += signal.expected_amplitude;
        }
        
        if stats.total_signals > 0 {
            stats.avg_amplitude = amplitude_sum / stats.total_signals as f32;
        }
        
        stats
    }
}

/// Statistics about the test signal library
#[derive(Debug, Clone)]
pub struct LibraryStats {
    pub total_signals: usize,
    pub musical_count: usize,
    pub calibration_count: usize,
    pub complex_count: usize,
    pub noise_count: usize,
    pub sweep_count: usize,
    pub pure_tone_count: usize,
    pub total_duration_ms: u32,
    pub avg_amplitude: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_library_creation() {
        let library = TestSignalLibrary::new(44100.0);
        let stats = library.get_library_stats();
        
        assert!(stats.total_signals > 0);
        assert!(stats.musical_count > 0);
        assert!(stats.calibration_count > 0);
        assert!(stats.complex_count > 0);
        assert!(stats.noise_count > 0);
        assert!(stats.sweep_count > 0);
    }
    
    #[test]
    fn test_signal_retrieval() {
        let library = TestSignalLibrary::new(44100.0);
        
        // Test musical note retrieval
        let a4_signal = library.get_signal("musical_a4");
        assert!(a4_signal.is_some());
        let a4 = a4_signal.unwrap();
        assert_eq!(a4.expected_frequency, Some(440.0));
        
        // Test calibration tone retrieval
        let cal_signal = library.get_signal("calibration_1000hz");
        assert!(cal_signal.is_some());
        let cal = cal_signal.unwrap();
        assert_eq!(cal.expected_frequency, Some(1000.0));
    }
    
    #[test]
    fn test_signal_types() {
        let library = TestSignalLibrary::new(44100.0);
        
        let musical_notes = library.get_musical_notes();
        assert!(!musical_notes.is_empty());
        
        let calibration_tones = library.get_calibration_tones();
        assert!(!calibration_tones.is_empty());
        
        let noise_signals = library.get_noise_signals();
        assert!(!noise_signals.is_empty());
    }
    
    #[test]
    fn test_custom_signal_generation() {
        let library = TestSignalLibrary::new(44100.0);
        
        let config = SignalConfig {
            waveform: WaveformType::Sine,
            frequency: 555.0,
            amplitude: 0.8,
            duration_ms: Some(500),
            sweep_end_freq: None,
            sample_rate: 44100.0,
        };
        
        let signal = library.generate_signal(config);
        assert!(signal.is_some());
        
        let signal = signal.unwrap();
        assert_eq!(signal.expected_frequency, Some(555.0));
        assert_eq!(signal.expected_amplitude, 0.8);
        assert_eq!(signal.duration_ms, 500);
    }
}