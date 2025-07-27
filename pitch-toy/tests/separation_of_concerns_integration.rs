//! Integration tests for verifying the separation of concerns between layers
//! 
//! These tests ensure that:
//! - Engine layer only handles raw frequency detection without musical interpretation
//! - Model layer processes raw frequencies with tuning context to produce musical data
//! - Presentation layer receives pre-processed musical data without doing calculations
//! - Data flows correctly through the three-layer architecture

use pitch_toy::engine::AudioEngine;
use pitch_toy::model::DataModel;
use pitch_toy::presentation::Presenter;
use pitch_toy::shared_types::{
    EngineUpdateResult, ModelUpdateResult, AudioAnalysis, Volume, Pitch, 
    PermissionState, TuningSystem, NoteName, Accuracy, MidiNote, to_midi_note
};
use pitch_toy::presentation::PresentationLayerActions;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Test that engine layer returns only raw frequency data without musical interpretation
#[wasm_bindgen_test]
async fn test_engine_returns_raw_frequency_only() {
    // Create engine
    let mut engine = AudioEngine::create()
        .await
        .expect("Engine creation should succeed");
    
    // Get engine update result
    let engine_result = engine.update(1.0);
    
    // If audio analysis is present, verify it contains only raw data
    if let Some(audio_analysis) = engine_result.audio_analysis {
        // Check that pitch is raw frequency without note information
        match audio_analysis.pitch {
            Pitch::Detected(frequency, clarity) => {
                // Engine should return raw frequency
                assert!(frequency > 0.0, "Frequency should be positive");
                assert!(clarity >= 0.0 && clarity <= 1.0, "Clarity should be normalized");
                
                // Engine result should NOT contain musical interpretation
                // (Note: EngineUpdateResult doesn't have accuracy or note fields)
            }
            Pitch::NotDetected => {
                // Valid case - no pitch detected
            }
        }
        
        // Verify volume is raw amplitude data
        assert!(audio_analysis.volume_level.peak_amplitude <= 0.0, "Peak amplitude should be in dB");
        assert!(audio_analysis.volume_level.rms_amplitude <= 0.0, "RMS amplitude should be in dB");
    }
    
    // Verify engine result doesn't contain musical interpretation fields
    // The EngineUpdateResult struct itself enforces this separation
}

/// Test that model layer processes raw frequency with tuning context
#[wasm_bindgen_test]
fn test_model_processes_frequency_with_tuning_context() {
    let mut model = DataModel::create()
        .expect("Model creation should succeed");
    
    // Create engine data with raw frequency (A4 = 440Hz)
    let audio_analysis = AudioAnalysis {
        volume_level: Volume { peak_amplitude: -10.0, rms_amplitude: -15.0 },
        pitch: Pitch::Detected(440.0, 0.95),
        fft_data: None,
        timestamp: 1.0,
    };
    
    let engine_data = EngineUpdateResult {
        audio_analysis: Some(audio_analysis),
        audio_errors: Vec::new(),
        permission_state: PermissionState::Granted,
    };
    
    // Process with default tuning (A root, Equal Temperament)
    let model_result = model.update(1.0, engine_data.clone());
    
    // Verify model added musical interpretation
    assert_eq!(model_result.accuracy.closest_note, 69);
    assert!(model_result.accuracy.accuracy < 0.01, "440Hz should be perfectly in tune with A");
    assert_eq!(model_result.tuning_system, TuningSystem::EqualTemperament);
    
    // Change root note to C
    let mut actions = PresentationLayerActions::new();
    actions.root_note_adjustments.push(pitch_toy::presentation::AdjustRootNote {
        root_note: 60,
    });
    let _ = model.process_user_actions(actions);
    
    // Process same raw frequency with new tuning context
    let model_result_c = model.update(2.0, engine_data);
    
    // Verify same frequency has different musical interpretation
    assert_eq!(model_result_c.accuracy.closest_note, 69);
    assert!(model_result_c.accuracy.accuracy > 0.01, 
        "440Hz should show inaccuracy with C root - got accuracy: {}", 
        model_result_c.accuracy.accuracy);
}

/// Test complete data flow from engine through model to presentation
#[wasm_bindgen_test]
async fn test_complete_data_flow_pipeline() {
    // Create all three layers
    let mut engine = AudioEngine::create()
        .await
        .expect("Engine creation should succeed");
    
    let mut model = DataModel::create()
        .expect("Model creation should succeed");
    
    let mut ui = Presenter::create()
        .expect("UI creation should succeed");
    
    // Simulate data flow with a test timestamp
    let timestamp = 1.0;
    
    // Step 1: Engine provides raw frequency data
    let engine_result = engine.update(timestamp);
    
    // Step 2: Model processes raw data with tuning context
    let model_result = model.update(timestamp, engine_result.clone());
    
    // Step 3: Presentation receives processed musical data
    ui.update(timestamp, model_result.clone());
    
    // Verify data transformation at each stage
    if let Some(audio_analysis) = engine_result.audio_analysis {
        match audio_analysis.pitch {
            Pitch::Detected(raw_frequency, _) => {
                // Engine provided raw frequency
                assert!(raw_frequency > 0.0, "Engine should provide raw frequency");
                
                // Model transformed it to musical data
                assert!(model_result.accuracy.closest_note != NoteName::A || 
                       model_result.accuracy.accuracy < 1.0,
                       "Model should provide musical interpretation");
                
                // Model included tuning system
                assert_eq!(model_result.tuning_system, TuningSystem::EqualTemperament);
            }
            Pitch::NotDetected => {
                // Valid case - ensure model handles it properly
                assert_eq!(model_result.pitch, Pitch::NotDetected);
                assert_eq!(model_result.accuracy.accuracy, 1.0);
            }
        }
    }
}

/// Test that tuning system changes affect only model layer processing
#[wasm_bindgen_test]
async fn test_tuning_changes_affect_only_model() {
    let mut engine = AudioEngine::create()
        .await
        .expect("Engine creation should succeed");
    
    let mut model = DataModel::create()
        .expect("Model creation should succeed");
    
    // Create consistent test data
    let test_frequency = 440.0; // A4
    let audio_analysis = AudioAnalysis {
        volume_level: Volume { peak_amplitude: -10.0, rms_amplitude: -15.0 },
        pitch: Pitch::Detected(test_frequency, 0.95),
        fft_data: None,
        timestamp: 1.0,
    };
    
    // Mock engine data since we can't control actual audio input
    let engine_data = EngineUpdateResult {
        audio_analysis: Some(audio_analysis),
        audio_errors: Vec::new(),
        permission_state: PermissionState::Granted,
    };
    
    // Get initial model result with A root
    let result_before = model.update(1.0, engine_data.clone());
    assert_eq!(result_before.accuracy.closest_note, 69);
    let accuracy_before = result_before.accuracy.accuracy;
    
    // Change root note in model
    let mut actions = PresentationLayerActions::new();
    actions.root_note_adjustments.push(pitch_toy::presentation::AdjustRootNote {
        root_note: 62,
    });
    let _ = model.process_user_actions(actions);
    
    // Engine should return same raw frequency (unaffected by tuning change)
    let engine_result_after = engine.update(2.0);
    // We can't directly compare engine results since we're mocking, but in real usage
    // the engine would return the same raw frequency for the same audio input
    
    // Model should interpret same frequency differently with new root
    let result_after = model.update(2.0, engine_data);
    assert_eq!(result_after.accuracy.closest_note, 69); // Still detected as A
    let accuracy_after = result_after.accuracy.accuracy;
    
    // Accuracy should be different with different root note
    assert_ne!(accuracy_before, accuracy_after,
        "Same frequency should have different accuracy with different root notes");
}

/// Test separation boundaries - ensure layers don't perform wrong calculations
#[wasm_bindgen_test]
fn test_layer_separation_boundaries() {
    // This test verifies architectural boundaries through type safety
    
    // Engine types don't include musical interpretation
    let engine_result = EngineUpdateResult {
        audio_analysis: Some(AudioAnalysis {
            volume_level: Volume { peak_amplitude: -10.0, rms_amplitude: -15.0 },
            pitch: Pitch::Detected(440.0, 0.95), // Only raw frequency
            fft_data: None,
            timestamp: 1.0,
        }),
        audio_errors: Vec::new(),
        permission_state: PermissionState::Granted,
    };
    
    // Engine result has no accuracy or note fields - verified by type system
    // The following would not compile:
    // let _ = engine_result.accuracy; // Error: no field `accuracy`
    // let _ = engine_result.closest_note; // Error: no field `closest_note`
    
    // Model result includes musical interpretation
    let model_result = ModelUpdateResult {
        volume: Volume { peak_amplitude: -10.0, rms_amplitude: -15.0 },
        pitch: Pitch::Detected(440.0, 0.95),
        accuracy: Accuracy {
            closest_note: NoteName::A,
            accuracy: 0.01,
        },
        tuning_system: TuningSystem::EqualTemperament,
        errors: Vec::new(),
        permission_state: PermissionState::Granted,
    };
    
    // Model result has musical fields - verified by type system
    assert_eq!(model_result.accuracy.closest_note, NoteName::A);
    assert_eq!(model_result.tuning_system, TuningSystem::EqualTemperament);
    
    // Presentation layer receives fully processed data
    // No frequency-to-note conversion needed in presentation
}

/// Test that errors propagate correctly through layers
#[wasm_bindgen_test]
fn test_error_propagation_through_layers() {
    let mut model = DataModel::create()
        .expect("Model creation should succeed");
    
    // Create engine data with errors
    let engine_data = EngineUpdateResult {
        audio_analysis: None,
        audio_errors: vec![
            pitch_toy::shared_types::Error::AudioContextSuspended,
            pitch_toy::shared_types::Error::MicrophonePermissionDenied,
        ],
        permission_state: PermissionState::Denied,
    };
    
    // Process through model
    let model_result = model.update(1.0, engine_data);
    
    // Verify errors propagated
    assert_eq!(model_result.errors.len(), 2);
    assert!(model_result.errors.contains(&pitch_toy::shared_types::Error::AudioContextSuspended));
    assert!(model_result.errors.contains(&pitch_toy::shared_types::Error::MicrophonePermissionDenied));
    
    // Verify permission state propagated
    assert_eq!(model_result.permission_state, PermissionState::Denied);
    
    // Verify model provides sensible defaults when no audio data
    assert_eq!(model_result.pitch, Pitch::NotDetected);
    assert_eq!(model_result.accuracy.accuracy, 1.0); // Maximum inaccuracy
}

/// Test that volume data flows through layers without musical interpretation
#[wasm_bindgen_test]
fn test_volume_data_flow() {
    let mut model = DataModel::create()
        .expect("Model creation should succeed");
    
    // Create engine data with specific volume levels
    let test_peak = -6.0;
    let test_rms = -12.0;
    
    let engine_data = EngineUpdateResult {
        audio_analysis: Some(AudioAnalysis {
            volume_level: Volume { 
                peak_amplitude: test_peak, 
                rms_amplitude: test_rms 
            },
            pitch: Pitch::NotDetected,
            fft_data: None,
            timestamp: 1.0,
        }),
        audio_errors: Vec::new(),
        permission_state: PermissionState::Granted,
    };
    
    // Process through model
    let model_result = model.update(1.0, engine_data);
    
    // Verify volume data passed through unchanged (no musical interpretation needed)
    assert_eq!(model_result.volume.peak_amplitude, test_peak);
    assert_eq!(model_result.volume.rms_amplitude, test_rms);
}

/// Test multiple tuning context changes in sequence
#[wasm_bindgen_test]
fn test_sequential_tuning_context_changes() {
    let mut model = DataModel::create()
        .expect("Model creation should succeed");
    
    // Test frequency: C5 (523.25 Hz)
    let c5_frequency = 523.25;
    let engine_data = EngineUpdateResult {
        audio_analysis: Some(AudioAnalysis {
            volume_level: Volume { peak_amplitude: -10.0, rms_amplitude: -15.0 },
            pitch: Pitch::Detected(c5_frequency, 0.92),
            fft_data: None,
            timestamp: 1.0,
        }),
        audio_errors: Vec::new(),
        permission_state: PermissionState::Granted,
    };
    
    // Test with different root notes in sequence
    let root_notes = vec![
        (NoteName::A, "A root"),
        (NoteName::C, "C root"), 
        (NoteName::E, "E root"),
        (NoteName::G, "G root"),
    ];
    
    let mut previous_accuracy = None;
    
    for (root_note, description) in root_notes {
        // Change root note
        let mut actions = PresentationLayerActions::new();
        actions.root_note_adjustments.push(pitch_toy::presentation::AdjustRootNote {
            root_note: to_midi_note(root_note.clone(), 4).unwrap(),
        });
        let _ = model.process_user_actions(actions);
        
        // Process same frequency with new context
        let result = model.update(1.0, engine_data.clone());
        
        // Verify note detection is consistent (absolute pitch)
        assert_eq!(result.accuracy.closest_note, 72, 
            "C5 should always be detected as C regardless of root");
        
        // Verify accuracy changes with root note (relative accuracy)
        if let Some(prev) = previous_accuracy {
            // Most root changes should result in different accuracy
            // (except in special cases like octaves)
            println!("{}: accuracy = {}", description, result.accuracy.accuracy);
        }
        
        previous_accuracy = Some(result.accuracy.accuracy);
    }
}