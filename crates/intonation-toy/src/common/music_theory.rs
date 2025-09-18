#![cfg(target_arch = "wasm32")]

use crate::common::shared_types::{MidiNote, TuningSystem, Scale, semitone_in_scale, is_valid_midi_note};
use crate::common::warn_log;

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
            tuning_system,
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

/// Convert a frequency to the closest MIDI note and cents offset
/// 
/// Takes into account the tuning system, tuning fork note, and current scale.
/// Returns None if the frequency is invalid or the resulting MIDI note is out of range.
pub fn frequency_to_midi_note_and_cents(
    frequency: f32,
    tuning_fork_note: MidiNote,
    tuning_system: TuningSystem,
    current_scale: Scale,
) -> Option<(MidiNote, f32)> {
    if frequency <= 0.0 {
        warn_log!("[MUSIC_THEORY] Invalid frequency for note conversion: {}", frequency);
        return None;
    }
    
    let root_pitch = midi_note_to_standard_frequency(tuning_fork_note);
    let interval_result = frequency_to_interval_semitones_scale_aware(
        tuning_system,
        root_pitch,
        frequency,
        current_scale,
    );
    
    let midi_note = tuning_fork_note as i32 + interval_result.semitones;
    
    if !is_valid_midi_note(midi_note) {
        return None;
    }
    
    Some((midi_note as u8, interval_result.cents))
}

/// Converts semitone offset to interval name
pub fn semitone_to_interval_name(semitone: i32) -> String {
    let semitone_in_octave = ((semitone % 12) + 12) % 12;

    let interval_name = match semitone_in_octave {
        0 => "1",
        1 => "b2",
        2 => "2",
        3 => "b3",
        4 => "3",
        5 => "4",
        6 => "#4",
        7 => "5",
        8 => "b6",
        9 => "6",
        10 => "b7",
        11 => "7",
        _ => "?",
    };

    interval_name.to_string()
}

