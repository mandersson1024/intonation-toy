// Simple test to understand just intonation cents calculation behavior

use pitch_toy::model::DataModel;
use pitch_toy::shared_types::{EngineUpdateResult, AudioAnalysis, Volume, Pitch, PermissionState, TuningSystem};
use pitch_toy::presentation::{PresentationLayerActions, ChangeTuningSystem};

fn main() {
    println!("Testing Just Intonation cents calculation...");
    
    let mut model = DataModel::create().expect("Model creation should succeed");
    
    // Test with a perfect fifth frequency (3:2 ratio = 659.25 Hz from 440Hz)
    let test_frequency = 659.25; // Perfect fifth from A4 (440Hz)
    
    // Create test audio data
    let audio_analysis = AudioAnalysis {
        volume_level: Volume { peak_amplitude: -10.0, rms_amplitude: -15.0 },
        pitch: Pitch::Detected(test_frequency, 0.95),
        fft_data: None,
        timestamp: 1.0,
    };
    
    let engine_data = EngineUpdateResult {
        audio_analysis: Some(audio_analysis),
        audio_errors: Vec::new(),
        permission_state: PermissionState::Granted,
    };
    
    // Test with Equal Temperament first
    println!("\n--- Equal Temperament ---");
    let result_et = model.update(1.0, engine_data.clone());
    println!("Frequency: {}Hz", test_frequency);
    println!("Closest MIDI note: {}", result_et.accuracy.closest_midi_note);
    println!("Cents offset: {}", result_et.accuracy.cents_offset);
    
    // Switch to Just Intonation
    let mut actions = PresentationLayerActions::new();
    actions.tuning_system_changes.push(ChangeTuningSystem {
        tuning_system: TuningSystem::JustIntonation,
    });
    let _ = model.process_user_actions(actions);
    
    // Test with Just Intonation
    println!("\n--- Just Intonation ---");
    let result_ji = model.update(2.0, engine_data.clone());
    println!("Frequency: {}Hz", test_frequency);
    println!("Closest MIDI note: {}", result_ji.accuracy.closest_midi_note);
    println!("Cents offset: {}", result_ji.accuracy.cents_offset);
    
    // Test with other frequencies to understand the pattern
    let test_frequencies = vec![
        (440.0, "A4 - Root note"),
        (550.0, "Major third from A4"),
        (586.67, "Perfect fourth from A4"), 
        (659.25, "Perfect fifth from A4"),
        (733.33, "Minor sixth from A4"),
        (880.0, "Octave from A4"),
    ];
    
    println!("\n--- Multiple frequency tests with Just Intonation ---");
    for (freq, description) in test_frequencies {
        let test_audio = AudioAnalysis {
            volume_level: Volume { peak_amplitude: -10.0, rms_amplitude: -15.0 },
            pitch: Pitch::Detected(freq, 0.95),
            fft_data: None,
            timestamp: 1.0,
        };
        
        let test_engine_data = EngineUpdateResult {
            audio_analysis: Some(test_audio),
            audio_errors: Vec::new(),
            permission_state: PermissionState::Granted,
        };
        
        let result = model.update(3.0, test_engine_data);
        println!("{}: {}Hz -> MIDI {} with {}cents", 
                description, freq, result.accuracy.closest_midi_note, result.accuracy.cents_offset);
    }
}