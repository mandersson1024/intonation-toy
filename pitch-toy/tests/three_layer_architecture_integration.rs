//! Three-Layer Architecture Integration Tests
//! 
//! Tests for the three-layer architecture implementation including:
//! - Layer creation and initialization
//! - Inter-layer communication through interfaces
//! - Debug GUI observational access
//! - Integration with existing audio functionality

use wasm_bindgen_test::*;
use std::rc::Rc;
use std::cell::RefCell;
use observable_data::DataSetter;

// Integration tests can run in both browser and node environments
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_all_layers_can_be_created_without_panicking() {
    // Create shared interfaces using Rc for proper interface sharing
    let engine_to_model = Rc::new(pitch_toy::module_interfaces::engine_to_model::EngineToModelInterface::new());
    let model_to_engine = Rc::new(pitch_toy::module_interfaces::model_to_engine::ModelToEngineInterface::new());
    let model_to_presentation = Rc::new(pitch_toy::module_interfaces::model_to_presentation::ModelToPresentationInterface::new());
    let presentation_to_model = Rc::new(pitch_toy::module_interfaces::presentation_to_model::PresentationToModelInterface::new());

    // Test engine layer creation
    // Note: AudioEngine::create is async and requires audio context, so we test interface acceptance
    let engine_interfaces = (engine_to_model.clone(), model_to_engine.clone());
    assert!(engine_interfaces.0.audio_analysis_observer().get().is_none());
    
    // Test that listener can be created
    let _listener = engine_interfaces.1.request_microphone_permission_listener();

    // Test model layer creation
    let model_result = pitch_toy::model::DataModel::create();
    assert!(model_result.is_ok());
    
    let mut model = model_result.unwrap();
    
    // Create dummy engine data for testing
    let dummy_engine_data = pitch_toy::module_interfaces::engine_to_model::EngineUpdateResult {
        audio_analysis: None,
        audio_errors: Vec::new(),
        permission_state: pitch_toy::module_interfaces::engine_to_model::PermissionState::NotRequested,
    };
    
    // Test model layer can be called without panicking
    let _result1 = model.update(0.0, dummy_engine_data.clone());
    let _result2 = model.update(1.0, dummy_engine_data.clone());
    let _result3 = model.update(100.0, dummy_engine_data);

    // Test presentation layer creation
    let presenter_result = pitch_toy::presentation::Presenter::create();
    assert!(presenter_result.is_ok());
    
    let mut presenter = presenter_result.unwrap();
    
    // Test presentation layer can be called without panicking
    let test_model_data = pitch_toy::module_interfaces::model_to_presentation::ModelUpdateResult {
        volume: pitch_toy::module_interfaces::model_to_presentation::Volume { peak: -10.0, rms: -15.0 },
        pitch: pitch_toy::module_interfaces::model_to_presentation::Pitch::NotDetected,
        accuracy: pitch_toy::module_interfaces::model_to_presentation::Accuracy {
            closest_note: pitch_toy::module_interfaces::model_to_presentation::Note::A,
            accuracy: 1.0,
        },
        tuning_system: pitch_toy::module_interfaces::model_to_presentation::TuningSystem::EqualTemperament,
        errors: Vec::new(),
        permission_state: pitch_toy::module_interfaces::model_to_presentation::PermissionState::NotRequested,
    };
    presenter.update(0.0, test_model_data.clone());
    presenter.update(1.0, test_model_data.clone());
    presenter.update(100.0, test_model_data);
}

#[wasm_bindgen_test]
fn test_interface_data_flow_between_layers() {
    // Create interfaces
    let engine_to_model = Rc::new(pitch_toy::module_interfaces::engine_to_model::EngineToModelInterface::new());
    let model_to_engine = Rc::new(pitch_toy::module_interfaces::model_to_engine::ModelToEngineInterface::new());
    
    // Test audio analysis data flow
    let audio_analysis_setter = engine_to_model.audio_analysis_setter();
    let audio_analysis_observer = engine_to_model.audio_analysis_observer();
    
    // Initially should be None
    assert!(audio_analysis_observer.get().is_none());
    
    // Set some test data
    let test_analysis = pitch_toy::module_interfaces::engine_to_model::AudioAnalysis {
        volume_level: pitch_toy::module_interfaces::engine_to_model::Volume {
            peak: -10.0,
            rms: -15.0,
        },
        pitch: pitch_toy::module_interfaces::engine_to_model::Pitch::Detected(440.0, 0.8),
        fft_data: None,
        timestamp: 1.0,
    };
    
    audio_analysis_setter.set(Some(test_analysis.clone()));
    
    // Verify data flows through interface
    let received_data = audio_analysis_observer.get();
    assert!(received_data.is_some());
    let received_analysis = received_data.unwrap();
    assert_eq!(received_analysis.volume_level.peak, -10.0);
    assert_eq!(received_analysis.volume_level.rms, -15.0);
    assert_eq!(received_analysis.timestamp, 1.0);
    
    match received_analysis.pitch {
        pitch_toy::module_interfaces::engine_to_model::Pitch::Detected(freq, clarity) => {
            assert_eq!(freq, 440.0);
            assert_eq!(clarity, 0.8);
        }
        _ => panic!("Expected detected pitch"),
    }

    // Test permission state flow
    let permission_setter = engine_to_model.permission_state_setter();
    let permission_observer = engine_to_model.permission_state_observer();
    
    permission_setter.set(pitch_toy::module_interfaces::engine_to_model::PermissionState::Granted);
    assert_eq!(permission_observer.get(), pitch_toy::module_interfaces::engine_to_model::PermissionState::Granted);
    
    // Test action flow from model to engine
    let microphone_action_trigger = model_to_engine.request_microphone_permission_trigger();
    let microphone_action_listener = model_to_engine.request_microphone_permission_listener();
    
    // Fire an action
    let test_action = pitch_toy::module_interfaces::model_to_engine::RequestMicrophonePermissionAction;
    microphone_action_trigger.fire(test_action);
    
    // Test that action system works (we use a simple counter to verify)
    let received_actions = Rc::new(RefCell::new(0));
    let received_clone = received_actions.clone();
    
    microphone_action_listener.listen(move |_action| {
        *received_clone.borrow_mut() += 1;
    });
    
    microphone_action_trigger.fire(pitch_toy::module_interfaces::model_to_engine::RequestMicrophonePermissionAction);
    
    // Give action system a moment to process (in real usage this would be immediate)
    assert_eq!(*received_actions.borrow(), 1);
}

#[wasm_bindgen_test]
fn test_debug_gui_observational_access() {
    // Create debug actions interface (still needed for debug panel)
    let debug_actions = pitch_toy::module_interfaces::debug_actions::DebugActionsInterface::new();
    
    // Create debug-specific data sources for testing
    use observable_data::DataSource;
    
    let audio_devices_source = DataSource::new(pitch_toy::engine::audio::AudioDevices {
        input_devices: vec![(String::from("test-input"), String::from("Test Input Device"))],
        output_devices: vec![(String::from("test-output"), String::from("Test Output Device"))],
    });
    let performance_metrics_source = DataSource::new(pitch_toy::debug::egui::data_types::PerformanceMetrics {
        fps: 60.0,
        memory_usage: 25.0,
        audio_latency: 10.0,
        cpu_usage: 30.0,
    });
    let audioworklet_status_source = DataSource::new(pitch_toy::debug::egui::data_types::AudioWorkletStatus::default());
    let buffer_pool_stats_source = DataSource::new(None::<pitch_toy::engine::audio::message_protocol::BufferPoolStats>);
    
    // Create HybridLiveData (debug GUI's data source) without interface
    let hybrid_live_data = pitch_toy::live_data::HybridLiveData::new();
    
    // Test that debug GUI can access data directly
    // Note: The new HybridLiveData starts with default data, not from observers
    // For this test, we'll update it with the test data
    let mut hybrid_live_data = hybrid_live_data;
    hybrid_live_data.update_debug_data(
        Some(test_audio_devices),
        Some(test_performance_metrics),
        Some(pitch_toy::debug::egui::data_types::AudioWorkletStatus::default()),
        None,
    );
    
    let audio_devices = &hybrid_live_data.audio_devices;
    assert_eq!(audio_devices.input_devices.len(), 1);
    assert_eq!(audio_devices.input_devices[0].1, "Test Input Device");
    assert_eq!(audio_devices.output_devices.len(), 1);
    assert_eq!(audio_devices.output_devices[0].1, "Test Output Device");
    
    let performance_metrics = &hybrid_live_data.performance_metrics;
    assert_eq!(performance_metrics.fps, 60.0);
    assert_eq!(performance_metrics.memory_usage, 25.0);
    
    // Test that debug GUI placeholder methods work (interface-free mode)
    assert!(hybrid_live_data.get_volume_level().is_none()); // Placeholder returns None
    assert!(hybrid_live_data.get_pitch_data().is_none()); // Placeholder returns None
    
    // Verify debug GUI uses placeholder implementations
    // These will be updated when Task 8 (debug layer update pattern) is implemented
    let volume_data = hybrid_live_data.get_volume_level();
    assert!(volume_data.is_none(), "Volume data should be None in placeholder implementation");
    
    let pitch_data = hybrid_live_data.get_pitch_data();
    assert!(pitch_data.is_none(), "Pitch data should be None in placeholder implementation");
    
    // Verify permission placeholder
    let permission = hybrid_live_data.get_microphone_permission();
    assert_eq!(permission, pitch_toy::engine::audio::AudioPermission::Uninitialized, 
               "Permission should be Uninitialized in placeholder implementation");
    
    // Test debug actions interface (GUI can trigger actions)
    let test_signal_trigger = debug_actions.test_signal_trigger();
    let test_signal_listener = debug_actions.test_signal_listener();
    
    let test_action = pitch_toy::module_interfaces::debug_actions::TestSignalAction {
        enabled: true,
        frequency: 1000.0,
        volume: 50.0, // 0-100 percentage
        waveform: pitch_toy::engine::audio::TestWaveform::Sine,
    };
    
    test_signal_trigger.fire(test_action.clone());
    
    // Test that action system works with a counter
    let received_actions = Rc::new(RefCell::new(Vec::new()));
    let received_clone = received_actions.clone();
    
    test_signal_listener.listen(move |action| {
        received_clone.borrow_mut().push(action);
    });
    
    test_signal_trigger.fire(pitch_toy::module_interfaces::debug_actions::TestSignalAction {
        enabled: false,
        frequency: 2000.0,
        volume: 75.0,
        waveform: pitch_toy::engine::audio::TestWaveform::Square,
    });
    
    assert_eq!(received_actions.borrow().len(), 1);
    assert!(!received_actions.borrow()[0].enabled);
    assert_eq!(received_actions.borrow()[0].frequency, 2000.0);
}

#[wasm_bindgen_test]
fn test_existing_functionality_regression() {
    // Test that existing functionality still works after three-layer architecture changes
    
    // Test interface creation
    let engine_to_model = Rc::new(pitch_toy::module_interfaces::engine_to_model::EngineToModelInterface::new());
    let model_to_engine = Rc::new(pitch_toy::module_interfaces::model_to_engine::ModelToEngineInterface::new());
    let model_to_presentation = Rc::new(pitch_toy::module_interfaces::model_to_presentation::ModelToPresentationInterface::new());
    let presentation_to_model = Rc::new(pitch_toy::module_interfaces::presentation_to_model::PresentationToModelInterface::new());
    let debug_actions = pitch_toy::module_interfaces::debug_actions::DebugActionsInterface::new();
    
    // Test that all interface components work
    assert!(engine_to_model.audio_analysis_observer().get().is_none());
    assert!(engine_to_model.audio_errors_observer().get().is_empty());
    assert_eq!(engine_to_model.permission_state_observer().get(), 
               pitch_toy::module_interfaces::engine_to_model::PermissionState::NotRequested);
    
    // Test action system functionality
    let microphone_trigger = model_to_engine.request_microphone_permission_trigger();
    let microphone_listener = model_to_engine.request_microphone_permission_listener();
    
    microphone_trigger.fire(pitch_toy::module_interfaces::model_to_engine::RequestMicrophonePermissionAction);
    
    // Test with action listener
    let received_count = Rc::new(RefCell::new(0));
    let received_clone = received_count.clone();
    
    microphone_listener.listen(move |_action| {
        *received_clone.borrow_mut() += 1;
    });
    
    microphone_trigger.fire(pitch_toy::module_interfaces::model_to_engine::RequestMicrophonePermissionAction);
    assert_eq!(*received_count.borrow(), 1);
    
    // Test debug actions
    let test_signal_trigger = debug_actions.test_signal_trigger();
    let test_signal_listener = debug_actions.test_signal_listener();
    
    test_signal_trigger.fire(pitch_toy::module_interfaces::debug_actions::TestSignalAction {
        enabled: false,
        frequency: 440.0,
        volume: 0.0,
        waveform: pitch_toy::engine::audio::TestWaveform::Sine,
    });
    
    // Test with action listener
    let debug_received_count = Rc::new(RefCell::new(0));
    let debug_received_clone = debug_received_count.clone();
    
    test_signal_listener.listen(move |_action| {
        *debug_received_clone.borrow_mut() += 1;
    });
    
    test_signal_trigger.fire(pitch_toy::module_interfaces::debug_actions::TestSignalAction {
        enabled: true,
        frequency: 880.0,
        volume: 50.0,
        waveform: pitch_toy::engine::audio::TestWaveform::Sine,
    });
    
    assert_eq!(*debug_received_count.borrow(), 1);
    
    // Test layer creation (basic smoke test)
    let model = pitch_toy::model::DataModel::create();
    assert!(model.is_ok());
    
    let presenter = pitch_toy::presentation::Presenter::create();
    assert!(presenter.is_ok());
}

#[wasm_bindgen_test]
fn test_layer_update_sequence() {
    // Test that layers can be updated in the proper sequence without panicking
    
    let engine_to_model = Rc::new(pitch_toy::module_interfaces::engine_to_model::EngineToModelInterface::new());
    let model_to_engine = Rc::new(pitch_toy::module_interfaces::model_to_engine::ModelToEngineInterface::new());
    let model_to_presentation = Rc::new(pitch_toy::module_interfaces::model_to_presentation::ModelToPresentationInterface::new());
    let presentation_to_model = Rc::new(pitch_toy::module_interfaces::presentation_to_model::PresentationToModelInterface::new());
    
    let mut model = pitch_toy::model::DataModel::create()
        .expect("Model creation should succeed");
    
    let mut presenter = pitch_toy::presentation::Presenter::create()
        .expect("Presenter creation should succeed");
    
    // Test update sequence for multiple frames
    for frame in 0..10 {
        let timestamp = frame as f64 * 0.016; // ~60 FPS
        
        // Note: Engine layer update would happen here in real application
        // For this test, we simulate by setting some interface data
        if frame % 3 == 0 {
            engine_to_model.audio_analysis_setter().set(Some(
                pitch_toy::module_interfaces::engine_to_model::AudioAnalysis {
                    volume_level: pitch_toy::module_interfaces::engine_to_model::Volume {
                        peak: -10.0 - (frame as f32),
                        rms: -15.0 - (frame as f32),
                    },
                    pitch: pitch_toy::module_interfaces::engine_to_model::Pitch::NotDetected,
                    fft_data: None,
                    timestamp,
                }
            ));
        }
        
        // Create dummy engine data for model update
        let dummy_engine_data = pitch_toy::module_interfaces::engine_to_model::EngineUpdateResult {
            audio_analysis: None,
            audio_errors: Vec::new(),
            permission_state: pitch_toy::module_interfaces::engine_to_model::PermissionState::NotRequested,
        };
        
        // Model layer update
        let _model_result = model.update(timestamp, dummy_engine_data);
        
        // Presentation layer update
        let test_model_data = pitch_toy::module_interfaces::model_to_presentation::ModelUpdateResult {
            volume: pitch_toy::module_interfaces::model_to_presentation::Volume { peak: -10.0, rms: -15.0 },
            pitch: pitch_toy::module_interfaces::model_to_presentation::Pitch::NotDetected,
            accuracy: pitch_toy::module_interfaces::model_to_presentation::Accuracy {
                closest_note: pitch_toy::module_interfaces::model_to_presentation::Note::A,
                accuracy: 1.0,
            },
            tuning_system: pitch_toy::module_interfaces::model_to_presentation::TuningSystem::EqualTemperament,
            errors: Vec::new(),
            permission_state: pitch_toy::module_interfaces::model_to_presentation::PermissionState::NotRequested,
        };
        presenter.update(timestamp, test_model_data);
    }
    
    // Verify that data flows through the interfaces
    let final_audio_data = engine_to_model.audio_analysis_observer().get();
    assert!(final_audio_data.is_some());
    let audio_data = final_audio_data.unwrap();
    assert!(audio_data.volume_level.peak <= -10.0); // Should have been updated
}

#[wasm_bindgen_test]
fn test_render_loop_functionality() {
    // Test that the render loop structure can be used without panicking
    // This simulates what happens in the actual application's render loop
    
    let engine_to_model = std::rc::Rc::new(pitch_toy::module_interfaces::engine_to_model::EngineToModelInterface::new());
    let model_to_engine = std::rc::Rc::new(pitch_toy::module_interfaces::model_to_engine::ModelToEngineInterface::new());
    let model_to_presentation = std::rc::Rc::new(pitch_toy::module_interfaces::model_to_presentation::ModelToPresentationInterface::new());
    let presentation_to_model = std::rc::Rc::new(pitch_toy::module_interfaces::presentation_to_model::PresentationToModelInterface::new());
    
    // Create layer instances (like in lib.rs)
    let model_result = pitch_toy::model::DataModel::create();
    assert!(model_result.is_ok());
    let mut model = Some(model_result.unwrap());
    
    let presenter_result = pitch_toy::presentation::Presenter::create();
    assert!(presenter_result.is_ok());
    let mut presenter = Some(presenter_result.unwrap());
    
    // Simulate the render loop sequence from lib.rs (lines 207-223)
    for frame in 0..10 {
        let timestamp = frame as f64 * 0.016; // ~60 FPS timestamp
        
        // Three-layer update sequence (engine → model → presenter)
        // Note: engine layer update would happen here in real application
        // For this test, we simulate by setting some interface data periodically
        if frame % 5 == 0 {
            engine_to_model.audio_analysis_setter().set(Some(
                pitch_toy::module_interfaces::engine_to_model::AudioAnalysis {
                    volume_level: pitch_toy::module_interfaces::engine_to_model::Volume {
                        peak: -5.0 - (frame as f32),
                        rms: -10.0 - (frame as f32),
                    },
                    pitch: if frame == 0 {
                        pitch_toy::module_interfaces::engine_to_model::Pitch::Detected(440.0, 0.7)
                    } else {
                        pitch_toy::module_interfaces::engine_to_model::Pitch::NotDetected
                    },
                    fft_data: None,
                    timestamp,
                }
            ));
        }
        
        // Update model layer (like in lib.rs line 216)
        if let Some(ref mut model) = model {
            // Create dummy engine data for model update
            let dummy_engine_data = pitch_toy::module_interfaces::engine_to_model::EngineUpdateResult {
                audio_analysis: None,
                audio_errors: Vec::new(),
                permission_state: pitch_toy::module_interfaces::engine_to_model::PermissionState::NotRequested,
            };
            let _model_result = model.update(timestamp, dummy_engine_data);
        }
        
        // Update presentation layer (like in lib.rs line 221)
        if let Some(ref mut presenter) = presenter {
            let test_model_data = pitch_toy::module_interfaces::model_to_presentation::ModelUpdateResult {
                volume: pitch_toy::module_interfaces::model_to_presentation::Volume { peak: -10.0, rms: -15.0 },
                pitch: pitch_toy::module_interfaces::model_to_presentation::Pitch::NotDetected,
                accuracy: pitch_toy::module_interfaces::model_to_presentation::Accuracy {
                    closest_note: pitch_toy::module_interfaces::model_to_presentation::Note::A,
                    accuracy: 1.0,
                },
                tuning_system: pitch_toy::module_interfaces::model_to_presentation::TuningSystem::EqualTemperament,
                errors: Vec::new(),
                permission_state: pitch_toy::module_interfaces::model_to_presentation::PermissionState::NotRequested,
            };
            presenter.update(timestamp, test_model_data);
        }
    }
    
    // Verify that the system worked without panicking and data flowed
    let final_data = engine_to_model.audio_analysis_observer().get();
    assert!(final_data.is_some());
    let data = final_data.unwrap();
    assert!(data.volume_level.peak <= -5.0); // Should have been updated in the loop
    
    // Test that layers can be dropped safely
    drop(model);
    drop(presenter);
    
    // Test that interfaces still work after layer cleanup
    engine_to_model.permission_state_setter().set(pitch_toy::module_interfaces::engine_to_model::PermissionState::Granted);
    assert_eq!(engine_to_model.permission_state_observer().get(), 
               pitch_toy::module_interfaces::engine_to_model::PermissionState::Granted);
}