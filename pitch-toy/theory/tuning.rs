use crate::shared_types::{MidiNote, TuningSystem};

pub fn interval_frequency(
    tuning_system: TuningSystem,
    root_frequency_hz: f32,
    interval_semitones: i32,
) -> f32 {
    match tuning_system {
        TuningSystem::EqualTemperament => {
            root_frequency_hz * 2.0_f32.powf(interval_semitones as f32 / 12.0)
        }
        TuningSystem::JustIntonation => {
            let octaves = interval_semitones.div_euclid(12);
            let semitone_in_octave = interval_semitones.rem_euclid(12);
            
            let ratio = match semitone_in_octave {
                0 => 1.0 / 1.0,   // Unison
                1 => 16.0 / 15.0, // Minor second
                2 => 9.0 / 8.0,   // Major second
                3 => 6.0 / 5.0,   // Minor third
                4 => 5.0 / 4.0,   // Major third
                5 => 4.0 / 3.0,   // Perfect fourth
                6 => 45.0 / 32.0, // Tritone
                7 => 3.0 / 2.0,   // Perfect fifth
                8 => 8.0 / 5.0,   // Minor sixth
                9 => 5.0 / 3.0,   // Major sixth
                10 => 9.0 / 5.0,  // Minor seventh
                11 => 15.0 / 8.0, // Major seventh
                _ => unreachable!(),
            };
            
            root_frequency_hz * ratio * 2.0_f32.powi(octaves)
        }
    }
}

pub fn midi_note_to_frequency_et(midi_note: MidiNote) -> f32 {
    440.0 * 2.0_f32.powf((midi_note as f32 - 69.0) / 12.0)
}

pub fn frequency_to_interval_semitones(
    tuning_system: TuningSystem,
    root_frequency_hz: f32,
    target_frequency_hz: f32,
) -> f32 {
    match tuning_system {
        TuningSystem::EqualTemperament => {
            12.0 * (target_frequency_hz / root_frequency_hz).log2()
        }
        TuningSystem::JustIntonation => {
            let ratio = target_frequency_hz / root_frequency_hz;
            let octaves = ratio.log2().floor();
            let ratio_in_octave = ratio / 2.0_f32.powf(octaves);
            
            let ji_ratios = [
                (0, 1.0 / 1.0),
                (1, 16.0 / 15.0),
                (2, 9.0 / 8.0),
                (3, 6.0 / 5.0),
                (4, 5.0 / 4.0),
                (5, 4.0 / 3.0),
                (6, 45.0 / 32.0),
                (7, 3.0 / 2.0),
                (8, 8.0 / 5.0),
                (9, 5.0 / 3.0),
                (10, 9.0 / 5.0),
                (11, 15.0 / 8.0),
            ];
            
            let (closest_semitone, _) = ji_ratios
                .iter()
                .min_by(|(_, r1), (_, r2)| {
                    let diff1 = (ratio_in_octave - r1).abs();
                    let diff2 = (ratio_in_octave - r2).abs();
                    diff1.partial_cmp(&diff2).unwrap()
                })
                .unwrap();
            
            octaves * 12.0 + *closest_semitone as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal_temperament_matches_existing_formula() {
        let root_freq = 440.0;
        let midi_a4 = 69;
        
        for offset in -12..=12 {
            let expected = 440.0 * 2.0_f32.powf(offset as f32 / 12.0);
            let actual = interval_frequency(TuningSystem::EqualTemperament, root_freq, offset);
            assert!((expected - actual).abs() < 0.001);
        }
        
        let freq_c4 = midi_note_to_frequency_et(60);
        assert!((freq_c4 - 261.626).abs() < 0.01);
        
        let freq_a4 = midi_note_to_frequency_et(69);
        assert!((freq_a4 - 440.0).abs() < 0.001);
    }

    #[test]
    fn test_just_intonation_ratios() {
        let root_freq = 440.0;
        
        let perfect_fifth = interval_frequency(TuningSystem::JustIntonation, root_freq, 7);
        assert!((perfect_fifth - root_freq * 3.0 / 2.0).abs() < 0.001);
        
        let major_third = interval_frequency(TuningSystem::JustIntonation, root_freq, 4);
        assert!((major_third - root_freq * 5.0 / 4.0).abs() < 0.001);
        
        let octave = interval_frequency(TuningSystem::JustIntonation, root_freq, 12);
        assert!((octave - root_freq * 2.0).abs() < 0.001);
        
        let octave_down = interval_frequency(TuningSystem::JustIntonation, root_freq, -12);
        assert!((octave_down - root_freq / 2.0).abs() < 0.001);
    }

    #[test]
    fn test_frequency_to_interval_equal_temperament() {
        let root_freq = 440.0;
        
        let interval = frequency_to_interval_semitones(
            TuningSystem::EqualTemperament,
            root_freq,
            root_freq * 2.0,
        );
        assert!((interval - 12.0).abs() < 0.001);
        
        let interval = frequency_to_interval_semitones(
            TuningSystem::EqualTemperament,
            root_freq,
            root_freq * 2.0_f32.powf(7.0 / 12.0),
        );
        assert!((interval - 7.0).abs() < 0.001);
    }

    #[test]
    fn test_frequency_to_interval_just_intonation() {
        let root_freq = 440.0;
        
        let interval = frequency_to_interval_semitones(
            TuningSystem::JustIntonation,
            root_freq,
            root_freq * 3.0 / 2.0,
        );
        assert!((interval - 7.0).abs() < 0.001);
        
        let interval = frequency_to_interval_semitones(
            TuningSystem::JustIntonation,
            root_freq,
            root_freq * 5.0 / 4.0,
        );
        assert!((interval - 4.0).abs() < 0.001);
    }
}