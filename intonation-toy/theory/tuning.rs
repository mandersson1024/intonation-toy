use crate::shared_types::{MidiNote, TuningSystem, Scale, semitone_in_scale};

/// Represents an interval as a base semitone with cents deviation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntervalSemitones {
    pub semitones: i32,
    pub cents: f32,
}

/// Just Intonation frequency ratios for the 12-tone chromatic scale
/// 
/// These ratios represent the harmonic relationships between notes in Just Intonation.
/// Index corresponds to semitones from the root (0-11).
const JUST_INTONATION_RATIOS: [(i32, f32); 12] = [
    (0, 1.0),           // Unison
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

/// We refer to Equal Temperament A4=440 as "Standard Tuning"
/// and the frequencies of the notes as "standard frequencies"
pub fn midi_note_to_standard_frequency(midi_note: MidiNote) -> f32 {
    440.0 * 2.0_f32.powf((midi_note as f32 - 69.0) / 12.0)
}

/// Convert a frequency to its interval relative to a root frequency
/// 
/// Returns the base semitone interval and cents deviation separately.
/// This handles cases where cents deviation may exceed ±100 cents.
/// 
/// For Equal Temperament: Base semitone is rounded to nearest, cents show deviation
/// For Just Intonation: Base semitone is closest just interval, cents show deviation from that
pub fn frequency_to_interval_semitones(
    tuning_system: TuningSystem,
    root_frequency_hz: f32,
    target_frequency_hz: f32,
) -> IntervalSemitones {
    match tuning_system {
        TuningSystem::EqualTemperament => {
            let total_cents = cents_delta(root_frequency_hz, target_frequency_hz);
            let base_semitones = (total_cents / 100.0).round() as i32;
            let base_freq = root_frequency_hz * 2.0_f32.powf(base_semitones as f32 / 12.0);
            let cents_deviation = cents_delta(base_freq, target_frequency_hz);
            
            IntervalSemitones {
                semitones: base_semitones,
                cents: cents_deviation,
            }
        }
        TuningSystem::JustIntonation => {
            let ratio = target_frequency_hz / root_frequency_hz;
            let octaves = ratio.log2().floor() as i32;
            let ratio_in_octave = ratio / 2.0_f32.powf(octaves as f32);
            
            let (closest_semitone, closest_ratio) = JUST_INTONATION_RATIOS
                .iter()
                .min_by(|(_, r1), (_, r2)| {
                    let target_ratio_freq = root_frequency_hz * ratio_in_octave;
                    let just_freq1 = root_frequency_hz * r1;
                    let just_freq2 = root_frequency_hz * r2;
                    let cents_diff1 = cents_delta(just_freq1, target_ratio_freq).abs();
                    let cents_diff2 = cents_delta(just_freq2, target_ratio_freq).abs();
                    cents_diff1.partial_cmp(&cents_diff2).unwrap()
                })
                .unwrap();
            
            let base_semitones = octaves * 12 + *closest_semitone;
            let just_intonation_freq = root_frequency_hz * closest_ratio * 2.0_f32.powf(octaves as f32);
            let cents_deviation = cents_delta(just_intonation_freq, target_frequency_hz);
            
            IntervalSemitones {
                semitones: base_semitones,
                cents: cents_deviation,
            }
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

/// Find the closest scale note to a given semitone interval
/// 
/// This function searches for the nearest scale member when the candidate semitone
/// is not in the scale. If the candidate is already in the scale, it's returned unchanged.
/// Otherwise, it searches outward (±1, ±2, ±3 semitones) until finding a scale member.
/// For ties at equal distance, it favors the upward direction.
fn find_closest_scale_note(candidate_semitone: i32, scale: Scale) -> i32 {
    // If already in scale, return as-is
    if semitone_in_scale(scale, candidate_semitone) {
        return candidate_semitone;
    }
    
    // Search outward for the closest scale member
    for distance in 1..=12 {
        // Check upward first (favoring upward for ties)
        let upward = candidate_semitone + distance;
        if semitone_in_scale(scale, upward) {
            return upward;
        }
        
        // Check downward
        let downward = candidate_semitone - distance;
        if semitone_in_scale(scale, downward) {
            return downward;
        }
    }
    
    // Fallback - should not happen for valid scales
    candidate_semitone
}

/// Scale-aware frequency to interval conversion
/// 
/// Converts a frequency to its interval relative to a root frequency,
/// but filters the result to the nearest scale member. This is useful
/// for applications that want to show intonation relative to scale notes only.
/// 
/// This implementation directly finds the closest scale note by frequency distance
/// rather than first rounding to chromatic semitones, which prevents issues where
/// non-scale chromatic notes are closer in semitone count but further in frequency.
pub fn frequency_to_interval_semitones_scale_aware(
    tuning_system: TuningSystem,
    root_frequency_hz: f32,
    target_frequency_hz: f32,
    scale: Scale,
) -> IntervalSemitones {
    // For chromatic scale, use the standard algorithm since all notes are in scale
    if scale == Scale::Chromatic {
        return frequency_to_interval_semitones(
            tuning_system,
            root_frequency_hz,
            target_frequency_hz,
        );
    }
    
    // For non-chromatic scales, find the closest scale note by frequency distance
    let mut closest_semitone = 0;
    let mut smallest_cents_distance = f32::INFINITY;
    
    // Search across a reasonable range of octaves (±4 octaves = ±48 semitones)
    // This covers the typical range of musical instruments and human voice
    for semitone in -48..=48 {
        // Skip notes not in the scale
        if !semitone_in_scale(scale, semitone) {
            continue;
        }
        
        // Calculate the frequency for this scale note
        let scale_note_frequency = interval_frequency(
            tuning_system.clone(),
            root_frequency_hz,
            semitone,
        );
        
        // Calculate cents distance to target frequency
        let cents_distance = cents_delta(scale_note_frequency, target_frequency_hz).abs();
        
        // Update if this is the closest scale note found so far
        if cents_distance < smallest_cents_distance {
            smallest_cents_distance = cents_distance;
            closest_semitone = semitone;
        }
    }
    
    // Calculate the final cents offset relative to the closest scale note
    let scale_note_frequency = interval_frequency(
        tuning_system,
        root_frequency_hz,
        closest_semitone,
    );
    let cents_offset = cents_delta(scale_note_frequency, target_frequency_hz);
    
    IntervalSemitones {
        semitones: closest_semitone,
        cents: cents_offset,
    }
}

/// Scale-aware interval frequency calculation
/// 
/// Returns the frequency for the closest scale member to the given interval.
/// This is useful for getting the "target" frequency that a scale-aware
/// intonation system would expect for a given interval.
pub fn interval_frequency_scale_aware(
    tuning_system: TuningSystem,
    root_frequency_hz: f32,
    interval_semitones: i32,
    scale: Scale,
) -> f32 {
    let scale_semitone = find_closest_scale_note(interval_semitones, scale);
    interval_frequency(tuning_system, root_frequency_hz, scale_semitone)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal_temperament_matches_existing_formula() {
        let root_freq = 440.0;
        
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
        assert_eq!(interval.semitones, 12);
        assert!(interval.cents.abs() < 0.001);
        
        let interval = frequency_to_interval_semitones(
            TuningSystem::EqualTemperament,
            root_freq,
            root_freq * 2.0_f32.powf(7.0 / 12.0),
        );
        assert_eq!(interval.semitones, 7);
        assert!(interval.cents.abs() < 0.001);
    }

    #[test]
    fn test_frequency_to_interval_just_intonation() {
        let root_freq = 440.0;
        
        let interval = frequency_to_interval_semitones(
            TuningSystem::JustIntonation,
            root_freq,
            root_freq * 3.0 / 2.0,
        );
        assert_eq!(interval.semitones, 7);
        assert!(interval.cents.abs() < 0.001);
        
        let interval = frequency_to_interval_semitones(
            TuningSystem::JustIntonation,
            root_freq,
            root_freq * 5.0 / 4.0,
        );
        assert_eq!(interval.semitones, 4);
        assert!(interval.cents.abs() < 0.001);
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

    #[test]
    fn test_find_closest_scale_note() {
        // Test notes already in Major scale
        assert_eq!(find_closest_scale_note(0, Scale::Major), 0);   // Root
        assert_eq!(find_closest_scale_note(2, Scale::Major), 2);   // Major 2nd
        assert_eq!(find_closest_scale_note(4, Scale::Major), 4);   // Major 3rd
        
        // Test notes not in Major scale - should round to nearest scale note
        assert_eq!(find_closest_scale_note(1, Scale::Major), 2);   // Minor 2nd -> Major 2nd (up)
        assert_eq!(find_closest_scale_note(3, Scale::Major), 2);   // Minor 3rd -> Major 2nd (down)
        assert_eq!(find_closest_scale_note(6, Scale::Major), 7);   // Tritone -> Perfect 5th (up)
        assert_eq!(find_closest_scale_note(8, Scale::Major), 7);   // Minor 6th -> Perfect 5th (down)
        assert_eq!(find_closest_scale_note(10, Scale::Major), 11); // Minor 7th -> Major 7th (up)
        
        // Test chromatic scale - all notes should remain unchanged
        for i in 0..12 {
            assert_eq!(find_closest_scale_note(i, Scale::Chromatic), i);
        }
        
        // Test octave handling
        assert_eq!(find_closest_scale_note(13, Scale::Major), 14); // Octave + Minor 2nd -> Octave + Major 2nd
        assert_eq!(find_closest_scale_note(-11, Scale::Major), -10); // -Minor 2nd -> -Major 2nd
        
        // Test MajorPentatonic scale
        assert_eq!(find_closest_scale_note(0, Scale::MajorPentatonic), 0);   // Root (C)
        assert_eq!(find_closest_scale_note(2, Scale::MajorPentatonic), 2);   // Major 2nd (D)
        assert_eq!(find_closest_scale_note(4, Scale::MajorPentatonic), 4);   // Major 3rd (E)
        assert_eq!(find_closest_scale_note(7, Scale::MajorPentatonic), 7);   // Perfect 5th (G)
        assert_eq!(find_closest_scale_note(9, Scale::MajorPentatonic), 9);   // Major 6th (A)
        
        // Test notes not in MajorPentatonic scale
        assert_eq!(find_closest_scale_note(1, Scale::MajorPentatonic), 2);   // Db -> D (up)
        assert_eq!(find_closest_scale_note(3, Scale::MajorPentatonic), 2);   // Eb -> D (down)
        assert_eq!(find_closest_scale_note(5, Scale::MajorPentatonic), 4);   // F -> E (down)
        assert_eq!(find_closest_scale_note(6, Scale::MajorPentatonic), 7);   // Gb -> G (up)
        assert_eq!(find_closest_scale_note(8, Scale::MajorPentatonic), 7);   // Ab -> G (down)
        assert_eq!(find_closest_scale_note(10, Scale::MajorPentatonic), 9);  // Bb -> A (down)
        assert_eq!(find_closest_scale_note(11, Scale::MajorPentatonic), 12); // B -> C octave (up)
        
        // Test MinorPentatonic scale
        assert_eq!(find_closest_scale_note(0, Scale::MinorPentatonic), 0);   // Root (C)
        assert_eq!(find_closest_scale_note(3, Scale::MinorPentatonic), 3);   // Minor 3rd (Eb)
        assert_eq!(find_closest_scale_note(5, Scale::MinorPentatonic), 5);   // Perfect 4th (F)
        assert_eq!(find_closest_scale_note(7, Scale::MinorPentatonic), 7);   // Perfect 5th (G)
        assert_eq!(find_closest_scale_note(10, Scale::MinorPentatonic), 10); // Minor 7th (Bb)
        
        // Test notes not in MinorPentatonic scale
        assert_eq!(find_closest_scale_note(1, Scale::MinorPentatonic), 0);   // Db -> C (down)
        assert_eq!(find_closest_scale_note(2, Scale::MinorPentatonic), 3);   // D -> Eb (up)
        assert_eq!(find_closest_scale_note(4, Scale::MinorPentatonic), 3);   // E -> Eb (down)
        assert_eq!(find_closest_scale_note(6, Scale::MinorPentatonic), 5);   // Gb -> F (down)
        assert_eq!(find_closest_scale_note(8, Scale::MinorPentatonic), 7);   // Ab -> G (down)
        assert_eq!(find_closest_scale_note(9, Scale::MinorPentatonic), 10);  // A -> Bb (up)
        assert_eq!(find_closest_scale_note(11, Scale::MinorPentatonic), 10); // B -> Bb (down)
        
        // Test pentatonic octave handling
        assert_eq!(find_closest_scale_note(13, Scale::MajorPentatonic), 14); // Octave + Db -> Octave + D
        assert_eq!(find_closest_scale_note(-11, Scale::MinorPentatonic), -10); // -B -> -Bb
    }

    #[test]
    fn test_frequency_to_interval_semitones_scale_aware() {
        let root_freq = 440.0; // A4
        
        // Test Equal Temperament with Major scale
        // A# (1 semitone) should map to B (2 semitones) in Major scale
        let a_sharp_freq = root_freq * 2.0_f32.powf(1.0 / 12.0);
        let interval = frequency_to_interval_semitones_scale_aware(
            TuningSystem::EqualTemperament,
            root_freq,
            a_sharp_freq,
            Scale::Major,
        );
        assert_eq!(interval.semitones, 2); // Should be mapped to B (Major 2nd)
        
        // The cents should be negative (A# is flat relative to B)
        assert!(interval.cents < 0.0);
        assert!((interval.cents + 100.0).abs() < 0.001); // Should be -100 cents
        
        // Test frequency already in scale (B natural)
        let b_freq = root_freq * 2.0_f32.powf(2.0 / 12.0);
        let interval = frequency_to_interval_semitones_scale_aware(
            TuningSystem::EqualTemperament,
            root_freq,
            b_freq,
            Scale::Major,
        );
        assert_eq!(interval.semitones, 2);
        assert!(interval.cents.abs() < 0.001); // Should be exactly 0 cents
        
        // Test Chromatic scale behavior (should be identical to non-scale-aware)
        let interval_chromatic = frequency_to_interval_semitones_scale_aware(
            TuningSystem::EqualTemperament,
            root_freq,
            a_sharp_freq,
            Scale::Chromatic,
        );
        let interval_raw = frequency_to_interval_semitones(
            TuningSystem::EqualTemperament,
            root_freq,
            a_sharp_freq,
        );
        assert_eq!(interval_chromatic.semitones, interval_raw.semitones);
        assert!((interval_chromatic.cents - interval_raw.cents).abs() < 0.001);
        
        // Test MajorPentatonic scale
        // C# (3 semitones from A) should map to D (4 semitones) in A Major Pentatonic
        let c_sharp_freq = root_freq * 2.0_f32.powf(3.0 / 12.0);
        let interval = frequency_to_interval_semitones_scale_aware(
            TuningSystem::EqualTemperament,
            root_freq,
            c_sharp_freq,
            Scale::MajorPentatonic,
        );
        assert_eq!(interval.semitones, 2); // Should be mapped to B (Major 2nd from A)
        assert!(interval.cents > 0.0); // C# is sharp relative to B
        
        // Test MinorPentatonic scale
        // B (2 semitones from A) should map to C (3 semitones) in A Minor Pentatonic
        let b_freq = root_freq * 2.0_f32.powf(2.0 / 12.0);
        let interval = frequency_to_interval_semitones_scale_aware(
            TuningSystem::EqualTemperament,
            root_freq,
            b_freq,
            Scale::MinorPentatonic,
        );
        assert_eq!(interval.semitones, 3); // Should be mapped to C (Minor 3rd from A)
        assert!(interval.cents < 0.0); // B is flat relative to C
        
        // Test frequency that's already in pentatonic scale
        let e_freq = root_freq * 2.0_f32.powf(7.0 / 12.0); // E (Perfect 5th from A)
        let interval = frequency_to_interval_semitones_scale_aware(
            TuningSystem::EqualTemperament,
            root_freq,
            e_freq,
            Scale::MajorPentatonic,
        );
        assert_eq!(interval.semitones, 7); // Should remain as Perfect 5th
        assert!(interval.cents.abs() < 0.001); // Should be exactly 0 cents
    }

    #[test]
    fn test_frequency_to_interval_semitones_scale_aware_just_intonation() {
        let root_freq = 440.0;
        
        // Test Just Intonation with Major scale
        // Use a frequency that would naturally map to a non-scale note
        let interval = frequency_to_interval_semitones_scale_aware(
            TuningSystem::JustIntonation,
            root_freq,
            root_freq * 16.0 / 15.0, // Minor 2nd in JI
            Scale::Major,
        );
        // Should map to Major 2nd (2 semitones)
        assert_eq!(interval.semitones, 2);
        // Cents should be negative (minor 2nd is flat relative to major 2nd)
        assert!(interval.cents < 0.0);
    }

    #[test]
    fn test_interval_frequency_scale_aware() {
        let root_freq = 440.0;
        
        // Test that asking for a non-scale interval returns the closest scale note frequency
        let freq = interval_frequency_scale_aware(
            TuningSystem::EqualTemperament,
            root_freq,
            1, // Minor 2nd (not in Major scale)
            Scale::Major,
        );
        
        // Should return frequency for Major 2nd (2 semitones)
        let expected_freq = interval_frequency(
            TuningSystem::EqualTemperament,
            root_freq,
            2,
        );
        assert!((freq - expected_freq).abs() < 0.001);
        
        // Test that scale notes remain unchanged
        let freq = interval_frequency_scale_aware(
            TuningSystem::EqualTemperament,
            root_freq,
            4, // Major 3rd (in Major scale)
            Scale::Major,
        );
        
        let expected_freq = interval_frequency(
            TuningSystem::EqualTemperament,
            root_freq,
            4,
        );
        assert!((freq - expected_freq).abs() < 0.001);
        
        // Test Chromatic scale (should be identical to non-scale-aware)
        let freq_chromatic = interval_frequency_scale_aware(
            TuningSystem::EqualTemperament,
            root_freq,
            6, // Tritone
            Scale::Chromatic,
        );
        
        let freq_raw = interval_frequency(
            TuningSystem::EqualTemperament,
            root_freq,
            6,
        );
        assert!((freq_chromatic - freq_raw).abs() < 0.001);
        
        // Test MajorPentatonic scale
        // Request Minor 3rd (3 semitones), should get Major 2nd (2 semitones) as closest
        let freq_pentatonic = interval_frequency_scale_aware(
            TuningSystem::EqualTemperament,
            root_freq,
            3, // Minor 3rd (not in Major Pentatonic)
            Scale::MajorPentatonic,
        );
        
        let expected_freq = interval_frequency(
            TuningSystem::EqualTemperament,
            root_freq,
            2, // Major 2nd (closest in Major Pentatonic)
        );
        assert!((freq_pentatonic - expected_freq).abs() < 0.001);
        
        // Test MinorPentatonic scale
        // Request Major 2nd (2 semitones), should get Minor 3rd (3 semitones) as closest
        let freq_minor_penta = interval_frequency_scale_aware(
            TuningSystem::EqualTemperament,
            root_freq,
            2, // Major 2nd (not in Minor Pentatonic)
            Scale::MinorPentatonic,
        );
        
        let expected_freq = interval_frequency(
            TuningSystem::EqualTemperament,
            root_freq,
            3, // Minor 3rd (closest in Minor Pentatonic)
        );
        assert!((freq_minor_penta - expected_freq).abs() < 0.001);
        
        // Test that pentatonic scale notes remain unchanged
        let freq = interval_frequency_scale_aware(
            TuningSystem::EqualTemperament,
            root_freq,
            7, // Perfect 5th (in both Major and Minor Pentatonic)
            Scale::MajorPentatonic,
        );
        
        let expected_freq = interval_frequency(
            TuningSystem::EqualTemperament,
            root_freq,
            7,
        );
        assert!((freq - expected_freq).abs() < 0.001);
    }
    
    #[test]
    fn test_frequency_based_scale_detection_bug_fix() {
        // Test for the specific bug where 226.6Hz with A3=220Hz root was incorrectly
        // mapping to B3 instead of A3 in non-chromatic scales
        
        let root_freq = 220.0; // A3
        
        // Test 224.4Hz - should map to A3 with positive cents
        let interval_224_4 = frequency_to_interval_semitones_scale_aware(
            TuningSystem::EqualTemperament,
            root_freq,
            224.4,
            Scale::Major, // Non-chromatic scale
        );
        assert_eq!(interval_224_4.semitones, 0); // Should be A3 (0 semitones from root)
        assert!(interval_224_4.cents > 0.0); // Should be sharp (positive cents)
        assert!(interval_224_4.cents < 50.0); // Should be reasonable cents offset
        
        // Test 226.6Hz - this was the problematic case
        // It should map to A3 (+51.4 cents), NOT B3 (-148.8 cents)
        let interval_226_6 = frequency_to_interval_semitones_scale_aware(
            TuningSystem::EqualTemperament,
            root_freq,
            226.6,
            Scale::Major,
        );
        assert_eq!(interval_226_6.semitones, 0); // Should be A3 (0 semitones from root)
        assert!(interval_226_6.cents > 0.0); // Should be sharp (positive cents)
        assert!(interval_226_6.cents > 50.0 && interval_226_6.cents < 55.0); // Should be ~51.4 cents
        
        // Verify that this is indeed closer to A3 than to B3
        let a3_freq = interval_frequency(TuningSystem::EqualTemperament, root_freq, 0);
        let b3_freq = interval_frequency(TuningSystem::EqualTemperament, root_freq, 2);
        let cents_to_a3 = cents_delta(a3_freq, 226.6).abs();
        let cents_to_b3 = cents_delta(b3_freq, 226.6).abs();
        assert!(cents_to_a3 < cents_to_b3, "226.6Hz should be closer to A3 than B3");
        
        // Test that chromatic scale still works correctly (regression test)
        let interval_chromatic = frequency_to_interval_semitones_scale_aware(
            TuningSystem::EqualTemperament,
            root_freq,
            226.6,
            Scale::Chromatic,
        );
        // In chromatic scale, this should round to the nearest semitone (A# = 1 semitone)
        let raw_interval = frequency_to_interval_semitones(
            TuningSystem::EqualTemperament,
            root_freq,
            226.6,
        );
        assert_eq!(interval_chromatic.semitones, raw_interval.semitones);
        assert!((interval_chromatic.cents - raw_interval.cents).abs() < 0.001);
        
        // Additional test: Demonstrate the bug is fixed by comparing old vs new behavior
        // Calculate what the old algorithm would have done (chromatic rounding first)
        let old_raw_interval = frequency_to_interval_semitones(
            TuningSystem::EqualTemperament,
            root_freq,
            226.6,
        );
        // Old algorithm: 226.6Hz → 51.4 cents → rounds to 1 semitone (A#) → maps to B (2 semitones)
        let old_closest_scale_note = find_closest_scale_note(old_raw_interval.semitones, Scale::Major);
        assert_eq!(old_raw_interval.semitones, 1); // Would round to A# (1 semitone)
        assert_eq!(old_closest_scale_note, 2); // Would map to B (2 semitones)
        
        // New algorithm correctly finds A (0 semitones) as closer
        assert_eq!(interval_226_6.semitones, 0); // Maps directly to A (0 semitones)
        assert!(interval_226_6.cents < 60.0); // Positive cents (sharp from A)
        
        // Verify the new algorithm gives better results
        assert!(interval_226_6.cents.abs() < 100.0); // Much closer than old (-148.8 cents from B)
    }

    #[test]
    fn test_pentatonic_scale_edge_cases() {
        // Test edge cases specific to pentatonic scales
        let root_freq = 440.0;
        
        // Test B (11 semitones) in MajorPentatonic - should map to next octave C (12)
        let freq = interval_frequency_scale_aware(
            TuningSystem::EqualTemperament,
            root_freq,
            11, // B (not in C Major Pentatonic)
            Scale::MajorPentatonic,
        );
        
        let expected_freq = interval_frequency(
            TuningSystem::EqualTemperament,
            root_freq,
            12, // C (octave)
        );
        assert!((freq - expected_freq).abs() < 0.001);
        
        // Test Just Intonation with pentatonic scales
        let interval = frequency_to_interval_semitones_scale_aware(
            TuningSystem::JustIntonation,
            root_freq,
            root_freq * 7.0 / 6.0, // Septimal minor third (not in pentatonic)
            Scale::MajorPentatonic,
        );
        // Should map to Major 2nd (2 semitones) in Major Pentatonic
        assert_eq!(interval.semitones, 2);
        
        // Test very large intervals with pentatonic scales
        let freq = interval_frequency_scale_aware(
            TuningSystem::EqualTemperament,
            root_freq,
            25, // 2 octaves + minor 2nd
            Scale::MinorPentatonic,
        );
        
        // Should map to 2 octaves + root (24 semitones)
        let expected_freq = interval_frequency(
            TuningSystem::EqualTemperament,
            root_freq,
            24,
        );
        assert!((freq - expected_freq).abs() < 0.001);
    }
}