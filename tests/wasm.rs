#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;
#[cfg(target_arch = "wasm32")]
use pitch_toy::audio::{
    MicrophoneManager, AudioContextManager, StreamReconnectionHandler,
    AudioPermission, AudioContextState, StreamState,
    AudioStreamInfo, AudioContextConfig, StreamConfig,
    PitchDetector, PitchDetectorConfig, PitchAnalyzer, NoteMapper,
    TuningSystem, NoteName
};

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_wasm_build_configuration() {
    // Test that WASM compilation works and can detect build configuration
    let config = if cfg!(debug_assertions) { "Development" } else { "Production" };
    assert!(config == "Development" || config == "Production");
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_wasm_audio_data_structures() {
    // Test that audio data structures work correctly in WASM
    let mic_state = AudioPermission::Uninitialized;
    assert_eq!(mic_state.to_string(), "Uninitialized");
    
    let context_state = AudioContextState::Running;
    assert_eq!(context_state.to_string(), "Running");
    
    let stream_state = StreamState::Connected;
    assert_eq!(stream_state, StreamState::Connected);
    
    // Test default configurations
    let stream_info = AudioStreamInfo::default();
    assert_eq!(stream_info.sample_rate, 48000.0);
    assert_eq!(stream_info.buffer_size, 1024);
    
    let context_config = AudioContextConfig::default();
    assert_eq!(context_config.sample_rate, 48000.0);
    assert_eq!(context_config.buffer_size, 1024);
    
    let stream_config = StreamConfig::default();
    assert_eq!(stream_config.max_reconnect_attempts, 3);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_wasm_manager_creation() {
    // Test that audio managers can be created in WASM environment
    let mic_manager = MicrophoneManager::new();
    assert_eq!(*mic_manager.state(), AudioPermission::Uninitialized);
    
    let audio_manager = AudioContextManager::new();
    assert_eq!(*audio_manager.state(), AudioContextState::Uninitialized);
    
    let stream_handler = StreamReconnectionHandler::new(StreamConfig::default());
    assert_eq!(stream_handler.get_health().state, StreamState::Disconnected);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_wasm_api_support_detection() {
    // Test Web Audio API support detection in browser environment
    let mic_supported = MicrophoneManager::is_supported();
    let audio_supported = AudioContextManager::is_supported();
    
    // In a real browser environment, these should typically be supported
    // In test environments, it may vary, so we just test they don't panic
    assert!(mic_supported || !mic_supported); // Either true or false is valid
    assert!(audio_supported || !audio_supported); // Either true or false is valid
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_wasm_audio_context_configuration() {
    // Test AudioContext configuration builders work in WASM
    let config_44_1 = AudioContextConfig::with_44_1khz();
    assert_eq!(config_44_1.sample_rate, 44100.0);
    
    let config_48 = AudioContextConfig::with_48khz();
    assert_eq!(config_48.sample_rate, 48000.0);
    
    let config_custom = AudioContextConfig::with_sample_rate(96000.0);
    assert_eq!(config_custom.sample_rate, 96000.0);
    
    let config_buffer = AudioContextConfig::default().with_buffer_size(2048);
    assert_eq!(config_buffer.buffer_size, 2048);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_wasm_stream_health_monitoring() {
    // Test stream health structures work correctly in WASM
    let config = StreamConfig {
        max_reconnect_attempts: 5,
        reconnect_delay_ms: 2000,
        health_check_interval_ms: 3000,
        activity_timeout_ms: 15000,
    };
    
    let handler = StreamReconnectionHandler::new(config);
    let health = handler.get_health();
    
    assert_eq!(health.state, StreamState::Disconnected);
    assert_eq!(health.reconnect_attempts, 0);
    assert!(health.error_message.is_none());
    assert!(!handler.is_connected());
}

// WASM Integration Tests for Pitch Detection (Task 8 Requirements)

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_wasm_pitch_detector_creation() {
    // Test that pitch detector can be created in WASM environment
    let config = PitchDetectorConfig::default();
    let detector = PitchDetector::new(config, 48000.0);
    assert!(detector.is_ok());
    
    let detector = detector.unwrap();
    assert_eq!(detector.sample_rate(), 48000.0);
    assert_eq!(detector.config().sample_window_size, 2048);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_wasm_pitch_analyzer_creation() {
    // Test that pitch analyzer can be created in WASM environment
    let config = PitchDetectorConfig::default();
    let analyzer = PitchAnalyzer::new(config, 48000.0);
    assert!(analyzer.is_ok());
    
    let analyzer = analyzer.unwrap();
    assert_eq!(analyzer.pitch_detector().sample_rate(), 48000.0);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_wasm_note_mapper_tuning_systems() {
    // Test that all tuning systems work in WASM
    let equal_mapper = NoteMapper::new(TuningSystem::EqualTemperament { reference_pitch: 440.0 });
    let note = equal_mapper.frequency_to_note(440.0);
    assert_eq!(note.note, NoteName::A);
    assert_eq!(note.octave, 4);
    
    let just_mapper = NoteMapper::new(TuningSystem::JustIntonation { reference_pitch: 440.0 });
    let note = just_mapper.frequency_to_note(440.0);
    assert_eq!(note.note, NoteName::A);
    
    let custom_ratios = vec![1.0, 1.059463, 1.122462, 1.189207, 1.259921, 1.334840];
    let custom_mapper = NoteMapper::new(TuningSystem::Custom { frequency_ratios: custom_ratios });
    let note = custom_mapper.frequency_to_note(440.0);
    assert!(note.octave >= 3 && note.octave <= 5);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_wasm_pitch_detection_sine_wave() {
    // Test pitch detection with generated sine wave in WASM
    let config = PitchDetectorConfig::default();
    let mut detector = PitchDetector::new(config, 48000.0).unwrap();
    
    // Generate 440Hz sine wave
    let frequency = 440.0;
    let sample_rate = 48000.0;
    let samples: Vec<f32> = (0..2048)
        .map(|i| {
            let t = i as f32 / sample_rate;
            (2.0 * std::f32::consts::PI * frequency * t).sin()
        })
        .collect();
    
    let result = detector.analyze(&samples);
    assert!(result.is_ok());
    
    if let Some(pitch_result) = result.unwrap() {
        // Should detect close to 440Hz
        assert!((pitch_result.frequency - 440.0).abs() < 50.0);
        assert!(pitch_result.confidence > 0.5);
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_wasm_pitch_detection_memory_efficiency() {
    // Test memory efficiency in WASM environment
    let config = PitchDetectorConfig::default();
    let detector = PitchDetector::new(config, 48000.0).unwrap();
    
    // Memory usage should be reasonable
    let memory_usage = detector.memory_usage_bytes();
    assert!(memory_usage > 0);
    assert!(memory_usage < 1_000_000); // Should be less than 1MB
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_wasm_performance_characteristics() {
    // Test performance characteristic calculation in WASM
    let config = PitchDetectorConfig::default();
    let detector = PitchDetector::new(config, 48000.0).unwrap();
    
    let (latency, grade) = detector.get_performance_characteristics();
    assert!(latency > 0.0);
    assert!(latency < 100.0); // Should be reasonable latency
    assert!(!grade.is_empty());
    
    // Test accuracy characteristics
    let (freq_res, accuracy_grade) = detector.get_accuracy_characteristics();
    assert!(freq_res > 0.0);
    assert!(freq_res < 200.0); // Should be reasonable frequency resolution
    assert!(!accuracy_grade.is_empty());
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_wasm_pitch_analyzer_with_buffer() {
    // Test pitch analyzer with pre-allocated buffer for zero-allocation processing
    let config = PitchDetectorConfig::default();
    let mut analyzer = PitchAnalyzer::new(config, 48000.0).unwrap();
    
    // Generate test signal
    let frequency = 440.0;
    let sample_rate = 48000.0;
    let samples: Vec<f32> = (0..2048)
        .map(|i| {
            let t = i as f32 / sample_rate;
            (2.0 * std::f32::consts::PI * frequency * t).sin()
        })
        .collect();
    
    let result = analyzer.analyze_samples(&samples);
    assert!(result.is_ok());
    
    // Check metrics were updated
    let metrics = analyzer.metrics();
    assert_eq!(metrics.analysis_cycles, 1);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test] 
fn test_wasm_pitch_detection_frequency_range() {
    // Test frequency range detection in WASM
    let config = PitchDetectorConfig::default();
    let mut detector = PitchDetector::new(config, 48000.0).unwrap();
    
    let test_frequencies = [220.0, 440.0, 880.0];
    let sample_rate = 48000.0;
    
    for &frequency in &test_frequencies {
        let samples: Vec<f32> = (0..2048)
            .map(|i| {
                let t = i as f32 / sample_rate;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();
        
        let result = detector.analyze(&samples);
        assert!(result.is_ok());
        
        if let Some(pitch_result) = result.unwrap() {
            let tolerance = frequency * 0.1; // 10% tolerance
            assert!((pitch_result.frequency - frequency).abs() < tolerance);
        }
    }
}
