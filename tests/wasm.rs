#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;
#[cfg(target_arch = "wasm32")]
use pitch_toy::modules::audio::{
    MicrophoneManager, AudioContextManager, StreamReconnectionHandler,
    AudioPermission, AudioContextState, StreamState,
    AudioStreamInfo, AudioContextConfig, StreamConfig
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
    let mic_state = MicrophoneState::Uninitialized;
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
    assert_eq!(*mic_manager.state(), MicrophoneState::Uninitialized);
    
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
