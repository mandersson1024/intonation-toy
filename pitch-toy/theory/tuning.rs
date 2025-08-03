use crate::shared_types::{MidiNote, TuningSystem};

/// Just Intonation frequency ratios for the 12-tone chromatic scale
/// 
/// These ratios represent the harmonic relationships between notes in Just Intonation.
/// Index corresponds to semitones from the root (0-11).
const JUST_INTONATION_RATIOS: [(i32, f32); 12] = [
    (0, 1.0 / 1.0),     // Unison
    (1, 16.0 / 15.0),   // Minor second
    (2, 9.0 / 8.0),     // Major second
    (3, 6.0 / 5.0),     // Minor third
    (4, 5.0 / 4.0),     // Major third
    (5, 4.0 / 3.0),     // Perfect fourth
    (6, 45.0 / 32.0),   // Tritone
    (7, 3.0 / 2.0),     // Perfect fifth
    (8, 8.0 / 5.0),     // Minor sixth
    (9, 5.0 / 3.0),     // Major sixth
    (10, 9.0 / 5.0),    // Minor seventh
    (11, 15.0 / 8.0),   // Major seventh
];

/// Get the Just Intonation ratio for a given semitone interval
fn get_just_intonation_ratio(semitone: i32) -> f32 {
    let semitone_in_octave = semitone.rem_euclid(12) as usize;
    JUST_INTONATION_RATIOS[semitone_in_octave].1
}

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
            let ratio = get_just_intonation_ratio(interval_semitones);
            root_frequency_hz * ratio * 2.0_f32.powi(octaves)
        }
    }
}

pub fn midi_note_to_standard_frequency(midi_note: MidiNote) -> f32 {
    // We refer to Equal Temperament A4=440 as "Standard Tuning"
    // and the frequencies of the notes as "standard frequencies"
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
            
            let (closest_semitone, _) = JUST_INTONATION_RATIOS
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

/// Calculate the difference between two frequencies in cents
/// 
/// Cents are a logarithmic unit of measure used for musical intervals.
/// There are 1200 cents in an octave. A positive result means frequency2 is higher than frequency1.
/// 
/// This definition of cents is independent of tuning system, so the distance between two 
/// semitones in a given tuning system might be more or less than 100 cents. However, 
/// the octave interval is always the same at 1200 cents across all tuning systems.
/// 
/// # Arguments
/// * `frequency1_hz` - The first frequency in Hz
/// * `frequency2_hz` - The second frequency in Hz
/// 
/// # Returns
/// The difference in cents (frequency2 relative to frequency1)
pub fn cents_delta(frequency1_hz: f32, frequency2_hz: f32) -> f32 {
    1200.0 * (frequency2_hz / frequency1_hz).log2()
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
        
        let freq_c4 = midi_note_to_standard_frequency(60);
        assert!((freq_c4 - 261.626).abs() < 0.01);
        
        let freq_a4 = midi_note_to_standard_frequency(69);
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

    #[test]
    fn test_cents_delta() {
        let base_freq = 440.0;
        
        // Octave up should be 1200 cents
        let octave_up = cents_delta(base_freq, base_freq * 2.0);
        assert!((octave_up - 1200.0).abs() < 0.001);
        
        // Octave down should be -1200 cents
        let octave_down = cents_delta(base_freq, base_freq / 2.0);
        assert!((octave_down + 1200.0).abs() < 0.001);
        
        // Perfect fifth (3:2 ratio) should be approximately 702 cents
        let perfect_fifth = cents_delta(base_freq, base_freq * 3.0 / 2.0);
        assert!((perfect_fifth - 701.955).abs() < 0.01);
        
        // Same frequency should be 0 cents
        let same_freq = cents_delta(base_freq, base_freq);
        assert!(same_freq.abs() < 0.001);
        
        // Semitone in equal temperament should be 100 cents
        let semitone = cents_delta(base_freq, base_freq * 2.0_f32.powf(1.0 / 12.0));
        assert!((semitone - 100.0).abs() < 0.001);
    }
}