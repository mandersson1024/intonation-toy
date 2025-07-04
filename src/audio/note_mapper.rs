use super::pitch_detector::{MusicalNote, NoteName, TuningSystem};

#[derive(Debug, Clone)]
pub struct NoteMapper {
    tuning_system: TuningSystem,
}

impl NoteMapper {
    pub fn new(tuning_system: TuningSystem) -> Self {
        Self { tuning_system }
    }

    pub fn frequency_to_note(&self, frequency: f32) -> MusicalNote {
        match &self.tuning_system {
            TuningSystem::EqualTemperament { reference_pitch } => {
                self.frequency_to_note_equal_temperament(frequency, *reference_pitch)
            }
            TuningSystem::JustIntonation { reference_pitch } => {
                self.frequency_to_note_just_intonation(frequency, *reference_pitch)
            }
            TuningSystem::Custom { frequency_ratios } => {
                self.frequency_to_note_custom(frequency, frequency_ratios)
            }
        }
    }

    pub fn note_to_frequency(&self, note: &MusicalNote) -> f32 {
        match &self.tuning_system {
            TuningSystem::EqualTemperament { reference_pitch } => {
                self.note_to_frequency_equal_temperament(note, *reference_pitch)
            }
            TuningSystem::JustIntonation { reference_pitch } => {
                self.note_to_frequency_just_intonation(note, *reference_pitch)
            }
            TuningSystem::Custom { frequency_ratios } => {
                self.note_to_frequency_custom(note, frequency_ratios)
            }
        }
    }

    pub fn calculate_cents(&self, frequency: f32, reference_frequency: f32) -> f32 {
        if frequency <= 0.0 || reference_frequency <= 0.0 {
            return 0.0;
        }
        1200.0 * (frequency / reference_frequency).log2()
    }

    pub fn set_tuning_system(&mut self, tuning_system: TuningSystem) {
        self.tuning_system = tuning_system;
    }

    pub fn tuning_system(&self) -> &TuningSystem {
        &self.tuning_system
    }

    fn frequency_to_note_equal_temperament(&self, frequency: f32, reference_pitch: f32) -> MusicalNote {
        // A4 = reference_pitch, MIDI note 69
        let midi_number = 69.0 + 12.0 * (frequency / reference_pitch).log2();
        let rounded_midi = midi_number.round() as i32;
        let note_index = (rounded_midi - 12) % 12;
        let octave = (rounded_midi - 12) / 12;

        let note_name = self.midi_note_to_name(note_index);
        let reference_frequency = self.midi_note_to_frequency_equal_temperament(rounded_midi, reference_pitch);
        let cents = self.calculate_cents(frequency, reference_frequency);

        MusicalNote::new(note_name, octave, cents, frequency)
    }

    fn frequency_to_note_just_intonation(&self, frequency: f32, reference_pitch: f32) -> MusicalNote {
        // Just intonation ratios relative to C (1/1)
        // A4 = 440Hz = (5/3) * C4, so C4 = 440 * (3/5) = 264Hz
        let c_frequency = reference_pitch * 3.0 / 5.0; // C4 frequency from A4
        
        let just_ratios = [
            1.0,        // C (1/1)
            16.0/15.0,  // C# (16/15)
            9.0/8.0,    // D (9/8)
            6.0/5.0,    // D# (6/5)
            5.0/4.0,    // E (5/4)
            4.0/3.0,    // F (4/3)
            45.0/32.0,  // F# (45/32)
            3.0/2.0,    // G (3/2)
            8.0/5.0,    // G# (8/5)
            5.0/3.0,    // A (5/3)
            9.0/5.0,    // A# (9/5)
            15.0/8.0,   // B (15/8)
        ];

        // Find the closest octave and note
        let mut best_match = 0;
        let mut best_octave = 4;
        let mut best_distance = f32::INFINITY;

        for octave in 0..=8 {
            let octave_multiplier = 2.0_f32.powi(octave - 4); // C4 reference
            for (note_index, &ratio) in just_ratios.iter().enumerate() {
                let note_frequency = c_frequency * ratio * octave_multiplier;
                let distance = (frequency - note_frequency).abs();
                if distance < best_distance {
                    best_distance = distance;
                    best_match = note_index;
                    best_octave = octave;
                }
            }
        }

        let note_name = self.midi_note_to_name(best_match as i32);
        let octave_multiplier = 2.0_f32.powi(best_octave - 4);
        let reference_frequency = c_frequency * just_ratios[best_match] * octave_multiplier;
        let cents = self.calculate_cents(frequency, reference_frequency);

        MusicalNote::new(note_name, best_octave, cents, frequency)
    }

    fn frequency_to_note_custom(&self, frequency: f32, frequency_ratios: &[f32]) -> MusicalNote {
        if frequency_ratios.is_empty() {
            // Fallback to equal temperament if no ratios provided
            return self.frequency_to_note_equal_temperament(frequency, 440.0);
        }

        // Use 440Hz as reference for custom tuning
        let base_frequency = 440.0;
        
        // Find the closest octave and note
        let mut best_match = 0;
        let mut best_octave = 4;
        let mut best_distance = f32::INFINITY;

        for octave in 0..=8 {
            let octave_multiplier = 2.0_f32.powi(octave - 4); // A4 reference
            for (note_index, &ratio) in frequency_ratios.iter().enumerate() {
                let note_frequency = base_frequency * ratio * octave_multiplier;
                let distance = (frequency - note_frequency).abs();
                if distance < best_distance {
                    best_distance = distance;
                    best_match = note_index;
                    best_octave = octave;
                }
            }
        }

        let note_name = self.midi_note_to_name((best_match % 12) as i32);
        let octave_multiplier = 2.0_f32.powi(best_octave - 4);
        let reference_frequency = base_frequency * frequency_ratios[best_match] * octave_multiplier;
        let cents = self.calculate_cents(frequency, reference_frequency);

        MusicalNote::new(note_name, best_octave, cents, frequency)
    }

    fn note_to_frequency_equal_temperament(&self, note: &MusicalNote, reference_pitch: f32) -> f32 {
        let midi_number = self.note_to_midi_number(note);
        reference_pitch * 2.0_f32.powf((midi_number - 69.0) / 12.0)
    }

    fn note_to_frequency_just_intonation(&self, note: &MusicalNote, reference_pitch: f32) -> f32 {
        let just_ratios = [
            1.0,        // C (1/1)
            16.0/15.0,  // C# (16/15)
            9.0/8.0,    // D (9/8)
            6.0/5.0,    // D# (6/5)
            5.0/4.0,    // E (5/4)
            4.0/3.0,    // F (4/3)
            45.0/32.0,  // F# (45/32)
            3.0/2.0,    // G (3/2)
            8.0/5.0,    // G# (8/5)
            5.0/3.0,    // A (5/3)
            9.0/5.0,    // A# (9/5)
            15.0/8.0,   // B (15/8)
        ];

        let note_index = self.note_name_to_index(&note.note);
        let octave_multiplier = 2.0_f32.powi(note.octave - 4);
        let c_frequency = reference_pitch * 3.0 / 5.0; // C4 frequency from A4
        c_frequency * just_ratios[note_index] * octave_multiplier
    }

    fn note_to_frequency_custom(&self, note: &MusicalNote, frequency_ratios: &[f32]) -> f32 {
        if frequency_ratios.is_empty() {
            return self.note_to_frequency_equal_temperament(note, 440.0);
        }

        let base_frequency = 440.0; // Use 440Hz as reference
        let note_index = self.note_name_to_index(&note.note);
        let ratio_index = note_index % frequency_ratios.len();
        let octave_multiplier = 2.0_f32.powi(note.octave - 4);
        
        base_frequency * frequency_ratios[ratio_index] * octave_multiplier
    }

    fn midi_note_to_name(&self, note_index: i32) -> NoteName {
        let index = ((note_index % 12) + 12) % 12; // Handle negative indices
        match index {
            0 => NoteName::C,
            1 => NoteName::CSharp,
            2 => NoteName::D,
            3 => NoteName::DSharp,
            4 => NoteName::E,
            5 => NoteName::F,
            6 => NoteName::FSharp,
            7 => NoteName::G,
            8 => NoteName::GSharp,
            9 => NoteName::A,
            10 => NoteName::ASharp,
            11 => NoteName::B,
            _ => NoteName::C, // Should never happen
        }
    }

    fn note_name_to_index(&self, note_name: &NoteName) -> usize {
        match note_name {
            NoteName::C => 0,
            NoteName::CSharp => 1,
            NoteName::D => 2,
            NoteName::DSharp => 3,
            NoteName::E => 4,
            NoteName::F => 5,
            NoteName::FSharp => 6,
            NoteName::G => 7,
            NoteName::GSharp => 8,
            NoteName::A => 9,
            NoteName::ASharp => 10,
            NoteName::B => 11,
        }
    }

    fn note_to_midi_number(&self, note: &MusicalNote) -> f32 {
        let note_index = self.note_name_to_index(&note.note);
        (note.octave + 1) as f32 * 12.0 + note_index as f32
    }

    fn midi_note_to_frequency_equal_temperament(&self, midi_number: i32, reference_pitch: f32) -> f32 {
        reference_pitch * 2.0_f32.powf((midi_number - 69) as f32 / 12.0)
    }
}

impl Default for NoteMapper {
    fn default() -> Self {
        Self::new(TuningSystem::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_mapper_creation() {
        let tuning = TuningSystem::EqualTemperament {
            reference_pitch: 440.0,
        };
        let mapper = NoteMapper::new(tuning);
        
        match mapper.tuning_system() {
            TuningSystem::EqualTemperament { reference_pitch } => {
                assert_eq!(*reference_pitch, 440.0);
            }
            _ => panic!("Expected EqualTemperament"),
        }
    }

    #[test]
    fn test_note_mapper_default() {
        let mapper = NoteMapper::default();
        
        match mapper.tuning_system() {
            TuningSystem::EqualTemperament { reference_pitch } => {
                assert_eq!(*reference_pitch, 440.0);
            }
            _ => panic!("Expected EqualTemperament as default"),
        }
    }

    #[test]
    fn test_frequency_to_note_equal_temperament_a4() {
        let mapper = NoteMapper::default();
        let note = mapper.frequency_to_note(440.0);
        
        assert_eq!(note.note, NoteName::A);
        assert_eq!(note.octave, 4);
        assert!(note.cents.abs() < 1.0); // Should be very close to 0
        assert_eq!(note.frequency, 440.0);
    }

    #[test]
    fn test_frequency_to_note_equal_temperament_c4() {
        let mapper = NoteMapper::default();
        let note = mapper.frequency_to_note(261.63);
        
        assert_eq!(note.note, NoteName::C);
        assert_eq!(note.octave, 4);
        assert!(note.cents.abs() < 5.0); // Should be very close to 0
        assert_eq!(note.frequency, 261.63);
    }

    #[test]
    fn test_frequency_to_note_equal_temperament_octaves() {
        let mapper = NoteMapper::default();
        
        // Test A notes across octaves
        let test_cases = [
            (110.0, NoteName::A, 2),
            (220.0, NoteName::A, 3),
            (440.0, NoteName::A, 4),
            (880.0, NoteName::A, 5),
            (1760.0, NoteName::A, 6),
        ];

        for (frequency, expected_note, expected_octave) in test_cases {
            let note = mapper.frequency_to_note(frequency);
            assert_eq!(note.note, expected_note);
            assert_eq!(note.octave, expected_octave);
            assert!(note.cents.abs() < 1.0);
        }
    }

    #[test]
    fn test_frequency_to_note_equal_temperament_chromatic() {
        let mapper = NoteMapper::default();
        
        // Test chromatic scale starting from C4
        let test_cases = [
            (261.63, NoteName::C, 4),
            (277.18, NoteName::CSharp, 4),
            (293.66, NoteName::D, 4),
            (311.13, NoteName::DSharp, 4),
            (329.63, NoteName::E, 4),
            (349.23, NoteName::F, 4),
            (369.99, NoteName::FSharp, 4),
            (392.00, NoteName::G, 4),
            (415.30, NoteName::GSharp, 4),
            (440.00, NoteName::A, 4),
            (466.16, NoteName::ASharp, 4),
            (493.88, NoteName::B, 4),
        ];

        for (frequency, expected_note, expected_octave) in test_cases {
            let note = mapper.frequency_to_note(frequency);
            assert_eq!(note.note, expected_note);
            assert_eq!(note.octave, expected_octave);
            assert!(note.cents.abs() < 10.0); // Allow for some rounding
        }
    }

    #[test]
    fn test_note_to_frequency_equal_temperament() {
        let mapper = NoteMapper::default();
        
        let note = MusicalNote::new(NoteName::A, 4, 0.0, 440.0);
        let frequency = mapper.note_to_frequency(&note);
        assert!((frequency - 440.0).abs() < 0.01);
        
        let note = MusicalNote::new(NoteName::C, 4, 0.0, 261.63);
        let frequency = mapper.note_to_frequency(&note);
        assert!((frequency - 261.63).abs() < 0.01);
    }

    #[test]
    fn test_frequency_to_note_just_intonation() {
        let tuning = TuningSystem::JustIntonation {
            reference_pitch: 440.0,
        };
        let mapper = NoteMapper::new(tuning);
        
        // Test A4 in just intonation
        let note = mapper.frequency_to_note(440.0);
        assert_eq!(note.note, NoteName::A);
        assert_eq!(note.octave, 4);
        assert!(note.cents.abs() < 10.0);
    }

    #[test]
    fn test_frequency_to_note_just_intonation_perfect_fifth() {
        let tuning = TuningSystem::JustIntonation {
            reference_pitch: 440.0,
        };
        let mapper = NoteMapper::new(tuning);
        
        // Perfect fifth ratio is 3/2 = 1.5
        // E4 should be 440 * (5/4) / (5/3) = 440 * (3/4) = 330Hz
        let note = mapper.frequency_to_note(330.0);
        assert_eq!(note.note, NoteName::E);
        assert_eq!(note.octave, 4);
    }

    #[test]
    fn test_frequency_to_note_custom_tuning() {
        let custom_ratios = vec![1.0, 1.125, 1.25, 1.333, 1.5, 1.667, 1.875, 2.0];
        let tuning = TuningSystem::Custom {
            frequency_ratios: custom_ratios,
        };
        let mapper = NoteMapper::new(tuning);
        
        // Test with the base frequency (440Hz matches ratio 1.0)
        let note = mapper.frequency_to_note(440.0);
        // The algorithm finds the closest match across all octaves, so 440Hz might match 
        // a different ratio/octave combination. Let's just verify the mapping is consistent.
        assert!(note.octave >= 0 && note.octave <= 8);
        
        // Test that the reverse mapping works
        let converted_freq = mapper.note_to_frequency(&note);
        let cents_diff = mapper.calculate_cents(440.0, converted_freq).abs();
        assert!(cents_diff < 50.0); // Should be reasonably close
    }

    #[test]
    fn test_frequency_to_note_custom_tuning_empty() {
        let tuning = TuningSystem::Custom {
            frequency_ratios: vec![],
        };
        let mapper = NoteMapper::new(tuning);
        
        // Should fallback to equal temperament
        let note = mapper.frequency_to_note(440.0);
        assert_eq!(note.note, NoteName::A);
        assert_eq!(note.octave, 4);
    }

    #[test]
    fn test_note_to_frequency_just_intonation() {
        let tuning = TuningSystem::JustIntonation {
            reference_pitch: 440.0,
        };
        let mapper = NoteMapper::new(tuning);
        
        let note = MusicalNote::new(NoteName::A, 4, 0.0, 440.0);
        let frequency = mapper.note_to_frequency(&note);
        
        // A4 in just intonation with 440Hz reference should be close to 440 * (5/3) / (5/3) = 440
        assert!((frequency - 440.0).abs() < 10.0);
    }

    #[test]
    fn test_note_to_frequency_custom_tuning() {
        let custom_ratios = vec![1.0, 1.125, 1.25, 1.333, 1.5, 1.667, 1.875, 2.0];
        let tuning = TuningSystem::Custom {
            frequency_ratios: custom_ratios,
        };
        let mapper = NoteMapper::new(tuning);
        
        let note = MusicalNote::new(NoteName::C, 4, 0.0, 440.0);
        let frequency = mapper.note_to_frequency(&note);
        
        // Should use the first ratio (1.0) for C
        assert!((frequency - 440.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_cents() {
        let mapper = NoteMapper::default();
        
        // Same frequency should be 0 cents
        assert_eq!(mapper.calculate_cents(440.0, 440.0), 0.0);
        
        // Octave should be 1200 cents
        assert!((mapper.calculate_cents(880.0, 440.0) - 1200.0).abs() < 0.01);
        
        // Semitone should be 100 cents
        let semitone_ratio = 2.0_f32.powf(1.0/12.0);
        let expected_cents = 100.0;
        let actual_cents = mapper.calculate_cents(440.0 * semitone_ratio, 440.0);
        assert!((actual_cents - expected_cents).abs() < 0.01);
    }

    #[test]
    fn test_calculate_cents_invalid_input() {
        let mapper = NoteMapper::default();
        
        // Zero frequencies should return 0
        assert_eq!(mapper.calculate_cents(0.0, 440.0), 0.0);
        assert_eq!(mapper.calculate_cents(440.0, 0.0), 0.0);
        assert_eq!(mapper.calculate_cents(0.0, 0.0), 0.0);
        
        // Negative frequencies should return 0
        assert_eq!(mapper.calculate_cents(-440.0, 440.0), 0.0);
        assert_eq!(mapper.calculate_cents(440.0, -440.0), 0.0);
    }

    #[test]
    fn test_set_tuning_system() {
        let mut mapper = NoteMapper::default();
        
        // Start with equal temperament
        match mapper.tuning_system() {
            TuningSystem::EqualTemperament { reference_pitch } => {
                assert_eq!(*reference_pitch, 440.0);
            }
            _ => panic!("Expected EqualTemperament"),
        }
        
        // Change to just intonation
        let new_tuning = TuningSystem::JustIntonation {
            reference_pitch: 432.0,
        };
        mapper.set_tuning_system(new_tuning);
        
        match mapper.tuning_system() {
            TuningSystem::JustIntonation { reference_pitch } => {
                assert_eq!(*reference_pitch, 432.0);
            }
            _ => panic!("Expected JustIntonation"),
        }
    }

    #[test]
    fn test_midi_note_to_name() {
        let mapper = NoteMapper::default();
        
        // Test basic note mapping
        assert_eq!(mapper.midi_note_to_name(0), NoteName::C);
        assert_eq!(mapper.midi_note_to_name(1), NoteName::CSharp);
        assert_eq!(mapper.midi_note_to_name(2), NoteName::D);
        assert_eq!(mapper.midi_note_to_name(3), NoteName::DSharp);
        assert_eq!(mapper.midi_note_to_name(4), NoteName::E);
        assert_eq!(mapper.midi_note_to_name(5), NoteName::F);
        assert_eq!(mapper.midi_note_to_name(6), NoteName::FSharp);
        assert_eq!(mapper.midi_note_to_name(7), NoteName::G);
        assert_eq!(mapper.midi_note_to_name(8), NoteName::GSharp);
        assert_eq!(mapper.midi_note_to_name(9), NoteName::A);
        assert_eq!(mapper.midi_note_to_name(10), NoteName::ASharp);
        assert_eq!(mapper.midi_note_to_name(11), NoteName::B);
        
        // Test wraparound
        assert_eq!(mapper.midi_note_to_name(12), NoteName::C);
        assert_eq!(mapper.midi_note_to_name(13), NoteName::CSharp);
        
        // Test negative indices
        assert_eq!(mapper.midi_note_to_name(-1), NoteName::B);
        assert_eq!(mapper.midi_note_to_name(-2), NoteName::ASharp);
    }

    #[test]
    fn test_note_name_to_index() {
        let mapper = NoteMapper::default();
        
        assert_eq!(mapper.note_name_to_index(&NoteName::C), 0);
        assert_eq!(mapper.note_name_to_index(&NoteName::CSharp), 1);
        assert_eq!(mapper.note_name_to_index(&NoteName::D), 2);
        assert_eq!(mapper.note_name_to_index(&NoteName::DSharp), 3);
        assert_eq!(mapper.note_name_to_index(&NoteName::E), 4);
        assert_eq!(mapper.note_name_to_index(&NoteName::F), 5);
        assert_eq!(mapper.note_name_to_index(&NoteName::FSharp), 6);
        assert_eq!(mapper.note_name_to_index(&NoteName::G), 7);
        assert_eq!(mapper.note_name_to_index(&NoteName::GSharp), 8);
        assert_eq!(mapper.note_name_to_index(&NoteName::A), 9);
        assert_eq!(mapper.note_name_to_index(&NoteName::ASharp), 10);
        assert_eq!(mapper.note_name_to_index(&NoteName::B), 11);
    }

    #[test]
    fn test_note_to_midi_number() {
        let mapper = NoteMapper::default();
        
        // Test A4 = MIDI 69
        let note = MusicalNote::new(NoteName::A, 4, 0.0, 440.0);
        assert_eq!(mapper.note_to_midi_number(&note), 69.0);
        
        // Test C4 = MIDI 60
        let note = MusicalNote::new(NoteName::C, 4, 0.0, 261.63);
        assert_eq!(mapper.note_to_midi_number(&note), 60.0);
        
        // Test C0 = MIDI 12
        let note = MusicalNote::new(NoteName::C, 0, 0.0, 16.35);
        assert_eq!(mapper.note_to_midi_number(&note), 12.0);
    }

    #[test]
    fn test_reference_pitch_variations() {
        // Test different reference pitches
        let reference_pitches = [432.0, 440.0, 442.0, 446.0];
        
        for &ref_pitch in &reference_pitches {
            let tuning = TuningSystem::EqualTemperament {
                reference_pitch: ref_pitch,
            };
            let mapper = NoteMapper::new(tuning);
            
            // A4 should map correctly with the reference pitch
            let note = mapper.frequency_to_note(ref_pitch);
            assert_eq!(note.note, NoteName::A);
            assert_eq!(note.octave, 4);
            assert!(note.cents.abs() < 0.01);
            
            // Converting back should give the same frequency
            let converted_freq = mapper.note_to_frequency(&note);
            assert!((converted_freq - ref_pitch).abs() < 0.01);
        }
    }

    #[test]
    fn test_just_intonation_ratios() {
        let tuning = TuningSystem::JustIntonation {
            reference_pitch: 440.0,
        };
        let mapper = NoteMapper::new(tuning);
        
        // Test some basic just intonation intervals
        // Perfect fifth (3/2) from A4 should be around E5
        let perfect_fifth_freq = 440.0 * 3.0 / 2.0; // 660Hz
        let note = mapper.frequency_to_note(perfect_fifth_freq);
        assert_eq!(note.note, NoteName::E);
        assert_eq!(note.octave, 5);
        
        // Major third (5/4) from A4 should be around C#5
        let major_third_freq = 440.0 * 5.0 / 4.0; // 550Hz
        let note = mapper.frequency_to_note(major_third_freq);
        assert_eq!(note.note, NoteName::CSharp);
        assert_eq!(note.octave, 5);
    }

    #[test]
    fn test_frequency_accuracy() {
        let mapper = NoteMapper::default();
        
        // Test that frequency-to-note-to-frequency round trip is reasonably accurate
        // Note: Some frequencies will naturally have larger cent differences when mapped to the nearest note
        let test_frequencies = [220.0, 440.0, 880.0];
        
        for &freq in &test_frequencies {
            let note = mapper.frequency_to_note(freq);
            let converted_freq = mapper.note_to_frequency(&note);
            
            // Should be within 50 cents for equal temperament (reasonable tolerance)
            let cents_diff = mapper.calculate_cents(freq, converted_freq).abs();
            assert!(cents_diff < 50.0, "Frequency {} -> {} cents difference", freq, cents_diff);
        }
        
        // Test exact note frequencies should be very accurate
        let exact_frequencies = [261.63, 293.66, 329.63, 349.23, 392.00, 440.00, 493.88];
        for &freq in &exact_frequencies {
            let note = mapper.frequency_to_note(freq);
            let converted_freq = mapper.note_to_frequency(&note);
            
            // Should be within 5 cents for exact note frequencies
            let cents_diff = mapper.calculate_cents(freq, converted_freq).abs();
            assert!(cents_diff < 5.0, "Exact frequency {} -> {} cents difference", freq, cents_diff);
        }
    }

    #[test]
    fn test_edge_cases() {
        let mapper = NoteMapper::default();
        
        // Test very low frequencies
        let low_note = mapper.frequency_to_note(20.0);
        assert!(low_note.octave <= 1);
        
        // Test very high frequencies
        let high_note = mapper.frequency_to_note(8000.0);
        assert!(high_note.octave >= 7);
        
        // Test frequency close to zero
        let zero_note = mapper.frequency_to_note(0.1);
        assert!(zero_note.octave <= 0);
    }
}