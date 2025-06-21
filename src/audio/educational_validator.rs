use std::f32::consts::PI;
use crate::audio::{engine::AudioEngine, pitch_detector::PitchAlgorithm};

/// Musical note information for educational validation
#[derive(Debug, Clone)]
pub struct MusicalNote {
    pub name: String,
    pub frequency: f32,
    pub octave: i32,
    pub semitone_offset: i32, // From A4 (440Hz)
}

impl MusicalNote {
    pub fn new(name: String, semitone_offset: i32) -> Self {
        let frequency = 440.0 * 2.0_f32.powf(semitone_offset as f32 / 12.0);
        let octave = 4 + (semitone_offset + 9) / 12; // A4 is reference

        MusicalNote {
            name,
            frequency,
            octave,
            semitone_offset,
        }
    }

    pub fn from_frequency(frequency: f32) -> Self {
        let semitone_offset = (12.0 * (frequency / 440.0).log2()).round() as i32;
        let note_names = ["A", "A#", "B", "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#"];
        let note_index = ((semitone_offset % 12) + 12) % 12;
        let name = note_names[note_index as usize].to_string();

        Self::new(name, semitone_offset)
    }
}

/// Educational accuracy validation result
#[derive(Debug, Clone)]
pub struct AccuracyResult {
    pub test_name: String,
    pub expected_frequency: f32,
    pub detected_frequency: Option<f32>,
    pub cents_error: Option<f32>,
    pub meets_educational_requirement: bool, // ±5 cents
    pub musical_note: MusicalNote,
    pub algorithm: PitchAlgorithm,
}

impl AccuracyResult {
    pub fn new(
        test_name: String,
        expected_frequency: f32,
        detected_frequency: Option<f32>,
        algorithm: PitchAlgorithm,
    ) -> Self {
        let cents_error = detected_frequency.map(|detected| {
            1200.0 * (detected / expected_frequency).log2()
        });

        let meets_educational_requirement = cents_error
            .map(|error| error.abs() <= 5.0)
            .unwrap_or(false);

        let musical_note = MusicalNote::from_frequency(expected_frequency);

        AccuracyResult {
            test_name,
            expected_frequency,
            detected_frequency,
            cents_error,
            meets_educational_requirement,
            musical_note,
            algorithm,
        }
    }
}

/// Educational accuracy validator for pitch detection
pub struct EducationalValidator {
    results: Vec<AccuracyResult>,
}

impl EducationalValidator {
    pub fn new() -> Self {
        EducationalValidator {
            results: Vec::new(),
        }
    }

    /// Run comprehensive educational accuracy validation
    pub fn run_all_validations(&mut self, sample_rate: f32) {
        println!("🎵 Starting educational accuracy validation...");
        
        self.validate_musical_intervals(sample_rate);
        self.validate_cents_accuracy_across_octaves(sample_rate);
        self.validate_chromatic_scale_accuracy(sample_rate);
        self.validate_common_instrument_ranges(sample_rate);
        self.validate_frequency_stability(sample_rate);
        
        println!("\n✅ Educational validation completed!");
        self.print_educational_summary();
    }

    /// Validate pitch detection accuracy with known musical intervals
    pub fn validate_musical_intervals(&mut self, sample_rate: f32) {
        println!("\n🎼 Validating musical interval accuracy...");
        
        let base_frequency = 440.0; // A4
        let buffer_size = 2048;
        
        // Test major musical intervals
        let intervals = [
            (0, "Unison"),
            (1, "Minor 2nd (Semitone)"),
            (2, "Major 2nd (Whole tone)"),
            (3, "Minor 3rd"),
            (4, "Major 3rd"),
            (5, "Perfect 4th"),
            (6, "Tritone"),
            (7, "Perfect 5th"),
            (8, "Minor 6th"),
            (9, "Major 6th"),
            (10, "Minor 7th"),
            (11, "Major 7th"),
            (12, "Octave"),
        ];

        let algorithms = [PitchAlgorithm::YIN, PitchAlgorithm::McLeod];

        for algorithm in &algorithms {
            println!("  Testing with {:?} algorithm:", algorithm);
            
            for &(semitones, interval_name) in &intervals {
                let target_frequency = base_frequency * 2.0_f32.powf(semitones as f32 / 12.0);
                let test_buffer = self.generate_sine_wave(target_frequency, sample_rate, buffer_size);
                
                let mut engine = AudioEngine::new(sample_rate, buffer_size);
                engine.set_pitch_algorithm(*algorithm);
                
                let detected_frequency = engine.detect_pitch_from_buffer(&test_buffer);
                let detected = if detected_frequency > 0.0 { Some(detected_frequency) } else { None };
                
                let result = AccuracyResult::new(
                    format!("interval_{}_{:?}", interval_name.replace(" ", "_"), algorithm),
                    target_frequency,
                    detected,
                    *algorithm,
                );
                
                if let Some(cents_error) = result.cents_error {
                    println!("    {}: {:.1} cents error ({})", 
                        interval_name, 
                        cents_error.abs(),
                        if result.meets_educational_requirement { "✅" } else { "❌" });
                } else {
                    println!("    {}: No detection (❌)", interval_name);
                }
                
                self.results.push(result);
            }
        }
    }

    /// Validate cent accuracy across different octaves
    pub fn validate_cents_accuracy_across_octaves(&mut self, sample_rate: f32) {
        println!("\n🎹 Validating cents accuracy across octaves...");
        
        let buffer_size = 2048;
        
        // Test A notes across multiple octaves
        let octave_tests = [
            ("A2", 110.0),   // 2nd octave
            ("A3", 220.0),   // 3rd octave  
            ("A4", 440.0),   // 4th octave (reference)
            ("A5", 880.0),   // 5th octave
            ("A6", 1760.0),  // 6th octave
        ];

        let algorithms = [PitchAlgorithm::YIN, PitchAlgorithm::McLeod];

        for algorithm in &algorithms {
            println!("  Testing octaves with {:?}:", algorithm);
            
            for &(note_name, frequency) in &octave_tests {
                let test_buffer = self.generate_sine_wave(frequency, sample_rate, buffer_size);
                
                let mut engine = AudioEngine::new(sample_rate, buffer_size);
                engine.set_pitch_algorithm(*algorithm);
                
                let detected_frequency = engine.detect_pitch_from_buffer(&test_buffer);
                let detected = if detected_frequency > 0.0 { Some(detected_frequency) } else { None };
                
                let result = AccuracyResult::new(
                    format!("octave_{}_{:?}", note_name, algorithm),
                    frequency,
                    detected,
                    *algorithm,
                );
                
                if let Some(cents_error) = result.cents_error {
                    println!("    {}: {:.1} cents error ({})", 
                        note_name, 
                        cents_error.abs(),
                        if result.meets_educational_requirement { "✅" } else { "❌" });
                } else {
                    println!("    {}: No detection (❌)", note_name);
                }
                
                self.results.push(result);
            }
        }
    }

    /// Validate chromatic scale accuracy (all 12 semitones)
    pub fn validate_chromatic_scale_accuracy(&mut self, sample_rate: f32) {
        println!("\n🎵 Validating chromatic scale accuracy...");
        
        let buffer_size = 2048;
        let base_frequency = 440.0; // A4
        
        let chromatic_notes = [
            "A", "A#", "B", "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#"
        ];

        let algorithm = PitchAlgorithm::YIN; // Use one algorithm for chromatic test
        
        println!("  Testing chromatic scale with {:?}:", algorithm);
        
        for (i, note_name) in chromatic_notes.iter().enumerate() {
            let semitone_offset = i as i32;
            let frequency = base_frequency * 2.0_f32.powf(semitone_offset as f32 / 12.0);
            let test_buffer = self.generate_sine_wave(frequency, sample_rate, buffer_size);
            
            let mut engine = AudioEngine::new(sample_rate, buffer_size);
            engine.set_pitch_algorithm(algorithm);
            
            let detected_frequency = engine.detect_pitch_from_buffer(&test_buffer);
            let detected = if detected_frequency > 0.0 { Some(detected_frequency) } else { None };
            
            let result = AccuracyResult::new(
                format!("chromatic_{}4", note_name),
                frequency,
                detected,
                algorithm,
            );
            
            if let Some(cents_error) = result.cents_error {
                println!("    {}4 ({:.1}Hz): {:.1} cents error ({})", 
                    note_name, 
                    frequency,
                    cents_error.abs(),
                    if result.meets_educational_requirement { "✅" } else { "❌" });
            } else {
                println!("    {}4 ({:.1}Hz): No detection (❌)", note_name, frequency);
            }
            
            self.results.push(result);
        }
    }

    /// Validate common instrument frequency ranges
    pub fn validate_common_instrument_ranges(&mut self, sample_rate: f32) {
        println!("\n🎸 Validating common instrument ranges...");
        
        let buffer_size = 2048;
        
        // Common instrument fundamental frequencies
        let instrument_tests = [
            ("Guitar Low E", 82.41),    // E2
            ("Guitar A String", 110.0),  // A2
            ("Guitar D String", 146.83), // D3
            ("Guitar G String", 196.0),  // G3
            ("Guitar B String", 246.94), // B3
            ("Guitar High E", 329.63),   // E4
            ("Piano Middle C", 261.63),  // C4
            ("Violin G String", 196.0),  // G3
            ("Violin D String", 293.66), // D4
            ("Violin A String", 440.0),  // A4
            ("Violin E String", 659.25), // E5
            ("Bass E String", 41.20),    // E1
            ("Soprano C", 1046.50),      // C6
        ];

        let algorithm = PitchAlgorithm::YIN;
        
        for &(instrument, frequency) in &instrument_tests {
            // Skip frequencies outside our detection range
            if frequency < 80.0 || frequency > 2000.0 {
                continue;
            }
            
            let test_buffer = self.generate_sine_wave(frequency, sample_rate, buffer_size);
            
            let mut engine = AudioEngine::new(sample_rate, buffer_size);
            engine.set_pitch_algorithm(algorithm);
            
            let detected_frequency = engine.detect_pitch_from_buffer(&test_buffer);
            let detected = if detected_frequency > 0.0 { Some(detected_frequency) } else { None };
            
            let result = AccuracyResult::new(
                format!("instrument_{}", instrument.replace(" ", "_")),
                frequency,
                detected,
                algorithm,
            );
            
            if let Some(cents_error) = result.cents_error {
                println!("  {}: {:.1} cents error ({})", 
                    instrument,
                    cents_error.abs(),
                    if result.meets_educational_requirement { "✅" } else { "❌" });
            } else {
                println!("  {}: No detection (❌)", instrument);
            }
            
            self.results.push(result);
        }
    }

    /// Validate frequency stability for sustained notes
    pub fn validate_frequency_stability(&mut self, sample_rate: f32) {
        println!("\n🎵 Validating frequency stability for sustained notes...");
        
        let buffer_size = 1024;
        let test_frequency = 440.0; // A4
        let algorithm = PitchAlgorithm::YIN;
        
        // Test sustained note with multiple consecutive buffers
        let num_buffers = 10;
        let mut detections = Vec::new();
        
        let mut engine = AudioEngine::new(sample_rate, buffer_size);
        engine.set_pitch_algorithm(algorithm);
        
        for i in 0..num_buffers {
            // Add slight phase variation to simulate real sustained note
            let phase_offset = i as f32 * 0.1;
            let test_buffer = self.generate_sine_wave_with_phase(
                test_frequency, sample_rate, buffer_size, phase_offset
            );
            
            let detected_frequency = engine.detect_pitch_from_buffer(&test_buffer);
            if detected_frequency > 0.0 {
                detections.push(detected_frequency);
            }
        }
        
        if !detections.is_empty() {
            let mean_frequency = detections.iter().sum::<f32>() / detections.len() as f32;
            let variance = detections.iter()
                .map(|&f| (f - mean_frequency).powi(2))
                .sum::<f32>() / detections.len() as f32;
            let std_deviation = variance.sqrt();
            
            // Convert standard deviation to cents
            let cents_std_dev = 1200.0 * (std_deviation / mean_frequency).abs();
            
            let is_stable = cents_std_dev <= 10.0; // ±10 cents stability requirement
            
            println!("  Sustained note stability:");
            println!("    Mean frequency: {:.2}Hz", mean_frequency);
            println!("    Standard deviation: {:.3}Hz ({:.1} cents)", std_deviation, cents_std_dev);
            println!("    Stability: {} ({})", 
                if is_stable { "Good" } else { "Poor" },
                if is_stable { "✅" } else { "❌" });
            
            let result = AccuracyResult::new(
                "frequency_stability".to_string(),
                test_frequency,
                Some(mean_frequency),
                algorithm,
            );
            
            self.results.push(result);
        } else {
            println!("  No stable detections found (❌)");
        }
    }

    /// Generate sine wave test signal
    fn generate_sine_wave(&self, frequency: f32, sample_rate: f32, samples: usize) -> Vec<f32> {
        (0..samples)
            .map(|i| {
                let t = i as f32 / sample_rate;
                0.8 * (2.0 * PI * frequency * t).sin()
            })
            .collect()
    }

    /// Generate sine wave with phase offset
    fn generate_sine_wave_with_phase(&self, frequency: f32, sample_rate: f32, samples: usize, phase: f32) -> Vec<f32> {
        (0..samples)
            .map(|i| {
                let t = i as f32 / sample_rate;
                0.8 * (2.0 * PI * frequency * t + phase).sin()
            })
            .collect()
    }

    /// Print comprehensive educational accuracy summary
    pub fn print_educational_summary(&self) {
        println!("\n📊 EDUCATIONAL ACCURACY SUMMARY");
        println!("===============================");
        
        let total_tests = self.results.len();
        let successful_detections = self.results.iter()
            .filter(|r| r.detected_frequency.is_some())
            .count();
        
        let educational_passes = self.results.iter()
            .filter(|r| r.meets_educational_requirement)
            .count();
        
        println!("📈 Overall Statistics:");
        println!("  Total tests: {}", total_tests);
        println!("  Successful detections: {} ({:.1}%)", 
            successful_detections, 
            100.0 * successful_detections as f32 / total_tests as f32);
        println!("  Educational accuracy (±5 cents): {} ({:.1}%)", 
            educational_passes,
            100.0 * educational_passes as f32 / total_tests as f32);
        
        // Algorithm comparison
        for algorithm in &[PitchAlgorithm::YIN, PitchAlgorithm::McLeod] {
            let algo_results: Vec<_> = self.results.iter()
                .filter(|r| r.algorithm == *algorithm)
                .collect();
            
            if !algo_results.is_empty() {
                let algo_passes = algo_results.iter()
                    .filter(|r| r.meets_educational_requirement)
                    .count();
                
                println!("  {:?} accuracy: {} / {} ({:.1}%)",
                    algorithm,
                    algo_passes,
                    algo_results.len(),
                    100.0 * algo_passes as f32 / algo_results.len() as f32);
            }
        }
        
        // Detailed breakdown by test category
        let categories = ["interval", "octave", "chromatic", "instrument", "stability"];
        
        for category in &categories {
            let category_results: Vec<_> = self.results.iter()
                .filter(|r| r.test_name.contains(category))
                .collect();
            
            if !category_results.is_empty() {
                let category_passes = category_results.iter()
                    .filter(|r| r.meets_educational_requirement)
                    .count();
                
                println!("  {} tests: {} / {} ({:.1}%)",
                    category,
                    category_passes,
                    category_results.len(),
                    100.0 * category_passes as f32 / category_results.len() as f32);
            }
        }
        
        // Worst performers
        let mut errors: Vec<_> = self.results.iter()
            .filter_map(|r| r.cents_error.map(|e| (r, e.abs())))
            .collect();
        errors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        if !errors.is_empty() {
            println!("\n❌ Largest errors:");
            for (result, error) in errors.iter().take(5) {
                println!("  {}: {:.1} cents error", result.test_name, error);
            }
        }
        
        // Best performers
        let mut good_results: Vec<_> = self.results.iter()
            .filter(|r| r.meets_educational_requirement)
            .filter_map(|r| r.cents_error.map(|e| (r, e.abs())))
            .collect();
        good_results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        if !good_results.is_empty() {
            println!("\n✅ Best accuracy:");
            for (result, error) in good_results.iter().take(5) {
                println!("  {}: {:.1} cents error", result.test_name, error);
            }
        }
        
        // Educational verdict
        let overall_accuracy = 100.0 * educational_passes as f32 / total_tests as f32;
        println!("\n🎓 Educational Assessment:");
        if overall_accuracy >= 80.0 {
            println!("  EXCELLENT: Suitable for music education ({:.1}% accuracy)", overall_accuracy);
        } else if overall_accuracy >= 60.0 {
            println!("  GOOD: Acceptable for music education ({:.1}% accuracy)", overall_accuracy);
        } else if overall_accuracy >= 40.0 {
            println!("  FAIR: Limited music education use ({:.1}% accuracy)", overall_accuracy);
        } else {
            println!("  POOR: Not suitable for music education ({:.1}% accuracy)", overall_accuracy);
        }
    }

    /// Get all validation results
    pub fn get_results(&self) -> &[AccuracyResult] {
        &self.results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_musical_note_creation() {
        let note = MusicalNote::new("A".to_string(), 0);
        assert_eq!(note.name, "A");
        assert_eq!(note.frequency, 440.0);
        assert_eq!(note.octave, 4);
        assert_eq!(note.semitone_offset, 0);
    }

    #[test]
    fn test_musical_note_from_frequency() {
        let note = MusicalNote::from_frequency(440.0);
        assert_eq!(note.name, "A");
        assert_eq!(note.semitone_offset, 0);
        
        let note_c = MusicalNote::from_frequency(261.63); // C4
        assert_eq!(note_c.name, "C");
    }

    #[test]
    fn test_accuracy_result_creation() {
        let result = AccuracyResult::new(
            "test_note".to_string(),
            440.0,
            Some(442.0),
            PitchAlgorithm::YIN,
        );

        assert_eq!(result.expected_frequency, 440.0);
        assert_eq!(result.detected_frequency, Some(442.0));
        assert!(result.cents_error.is_some());
        
        // 442/440 = 1.00454, log2(1.00454) * 1200 ≈ 7.85 cents
        let cents = result.cents_error.unwrap();
        assert!((cents - 7.85).abs() < 0.1);
        assert!(!result.meets_educational_requirement); // >5 cents
    }

    #[test]
    fn test_educational_validator_creation() {
        let validator = EducationalValidator::new();
        assert_eq!(validator.results.len(), 0);
    }

    #[test]
    fn test_sine_wave_generation() {
        let validator = EducationalValidator::new();
        let wave = validator.generate_sine_wave(440.0, 44100.0, 1024);
        
        assert_eq!(wave.len(), 1024);
        
        // Check that we have a sine wave (should have both positive and negative values)
        let has_positive = wave.iter().any(|&x| x > 0.0);
        let has_negative = wave.iter().any(|&x| x < 0.0);
        
        assert!(has_positive);
        assert!(has_negative);
        
        // Check amplitude is reasonable
        let max_amplitude = wave.iter().map(|x| x.abs()).fold(0.0, f32::max);
        assert!(max_amplitude > 0.5 && max_amplitude <= 1.0);
    }
} 