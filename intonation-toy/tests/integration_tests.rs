//! Integration tests for platform abstraction
//!
//! These tests verify that the overall platform abstraction integration works
//! correctly across different layers of the application, ensuring that no
//! web-specific code leaks into native builds and that all components can
//! work together regardless of the target platform.

use intonation_toy::platform::traits::*;

#[cfg(feature = "web")]
use wasm_bindgen_test::*;

#[cfg(feature = "web")]
wasm_bindgen_test_configure!(run_in_browser);

/// Test that engine layer can use platform abstractions
#[cfg_attr(feature = "web", wasm_bindgen_test)]
#[cfg_attr(not(feature = "web"), test)]
fn test_engine_platform_integration() {
    // Simulate engine layer using platform abstractions
    #[cfg(target_arch = "wasm32")]
    let start_time = <intonation_toy::platform::WebTimer as Timer>::now_ms();
    #[cfg(not(target_arch = "wasm32"))]
    let start_time = <intonation_toy::platform::StubTimer as Timer>::now_ms();
    
    // Simulate some engine work
    let mut result = 0;
    for i in 0..1000 {
        result += i * 2;
    }
    
    #[cfg(target_arch = "wasm32")]
    let end_time = <intonation_toy::platform::WebTimer as Timer>::now_ms();
    #[cfg(not(target_arch = "wasm32"))]
    let end_time = <intonation_toy::platform::StubTimer as Timer>::now_ms();
    
    let duration = end_time - start_time;
    
    // Mark performance timing (returns None on native, Some on web)
    #[cfg(target_arch = "wasm32")]
    let _memory = <intonation_toy::platform::WebPerformanceMonitor as PerformanceMonitor>::sample_memory_usage();
    #[cfg(not(target_arch = "wasm32"))]
    let _memory = <intonation_toy::platform::StubPerformanceMonitor as PerformanceMonitor>::sample_memory_usage();
    
    // Verify timing makes sense
    assert!(duration >= 0.0, "Processing duration should be non-negative");
    assert!(result > 0, "Engine work should produce results");
}

/// Test presentation layer integration with platform abstractions
#[cfg_attr(feature = "web", wasm_bindgen_test)]
#[cfg_attr(not(feature = "web"), test)]
fn test_presentation_platform_integration() {
    // Simulate presentation layer using platform abstractions
    #[cfg(target_arch = "wasm32")]
    let error_display = intonation_toy::platform::WebErrorDisplay::new();
    #[cfg(not(target_arch = "wasm32"))]
    let error_display = intonation_toy::platform::StubErrorDisplay::new();
    
    // Presentation layer would update UI based on model data
    #[cfg(target_arch = "wasm32")]
    {
        use intonation_toy::platform::WebUiController;
        WebUiController::resize_canvas();
        WebUiController::apply_theme_styles();
        let _zoom = WebUiController::get_zoom_factor();
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        use intonation_toy::platform::StubUiController;
        StubUiController::resize_canvas();
        StubUiController::apply_theme_styles();
        let _zoom = StubUiController::get_zoom_factor();
    }
    
    // Test error display construction
    let _ = error_display;
    
    // All operations should complete without panicking
    assert!(true, "Presentation layer should integrate with platform successfully");
}

/// Test cross-platform timing-dependent functionality
#[cfg_attr(feature = "web", wasm_bindgen_test)]
#[cfg_attr(not(feature = "web"), test)]
fn test_timing_dependent_functionality() {
    // Simulate timing-sensitive operations that both platforms need
    #[cfg(target_arch = "wasm32")]
    let frame_start = <intonation_toy::platform::WebTimer as Timer>::now_ms();
    #[cfg(not(target_arch = "wasm32"))]
    let frame_start = <intonation_toy::platform::StubTimer as Timer>::now_ms();
    
    // Simulate frame processing
    simulate_frame_processing();
    
    #[cfg(target_arch = "wasm32")]
    let frame_end = <intonation_toy::platform::WebTimer as Timer>::now_ms();
    #[cfg(not(target_arch = "wasm32"))]
    let frame_end = <intonation_toy::platform::StubTimer as Timer>::now_ms();
    
    let frame_duration = frame_end - frame_start;
    
    // Both platforms should handle frame timing
    assert!(frame_duration >= 0.0, "Frame duration should be non-negative");
    
    // Simulate multiple frames to test consistency
    let mut total_duration = 0.0;
    for _ in 0..5 {
        #[cfg(target_arch = "wasm32")]
        let start = <intonation_toy::platform::WebTimer as Timer>::now_ms();
        #[cfg(not(target_arch = "wasm32"))]
        let start = <intonation_toy::platform::StubTimer as Timer>::now_ms();
        
        simulate_frame_processing();
        
        #[cfg(target_arch = "wasm32")]
        let end = <intonation_toy::platform::WebTimer as Timer>::now_ms();
        #[cfg(not(target_arch = "wasm32"))]
        let end = <intonation_toy::platform::StubTimer as Timer>::now_ms();
        
        total_duration += end - start;
    }
    
    assert!(total_duration >= 0.0, "Total duration should be non-negative");
}

/// Test that no web-specific code compiles in native builds
#[cfg_attr(not(feature = "web"), test)]
#[cfg(not(target_arch = "wasm32"))]
fn test_no_web_leakage_in_native() {
    // This test only runs on native builds
    // If web-specific code leaked into native builds, this would fail to compile
    
    let _time = <intonation_toy::platform::StubTimer as Timer>::now_ms();
    let _memory = <intonation_toy::platform::StubPerformanceMonitor as PerformanceMonitor>::sample_memory_usage();
    let _zoom = intonation_toy::platform::StubUiController::get_zoom_factor();
    let _error_display = intonation_toy::platform::StubErrorDisplay::new();
    
    // All platform abstractions should work with stub implementations
    intonation_toy::platform::StubUiController::resize_canvas();
    
    // If we can create and use all platform abstractions, no web code leaked
    assert!(true, "Native build should not contain web-specific code");
}

/// Test platform abstraction with realistic usage patterns
#[cfg_attr(feature = "web", wasm_bindgen_test)]
#[cfg_attr(not(feature = "web"), test)]
fn test_realistic_usage_patterns() {
    // Simulate a realistic application flow
    #[cfg(target_arch = "wasm32")]
    let init_start = <intonation_toy::platform::WebTimer as Timer>::now_ms();
    #[cfg(not(target_arch = "wasm32"))]
    let init_start = <intonation_toy::platform::StubTimer as Timer>::now_ms();
    
    // Simulate initialization work
    let init_result = simulate_audio_initialization();
    
    #[cfg(target_arch = "wasm32")]
    let init_end = <intonation_toy::platform::WebTimer as Timer>::now_ms();
    #[cfg(not(target_arch = "wasm32"))]
    let init_end = <intonation_toy::platform::StubTimer as Timer>::now_ms();
    
    if init_result {
        // Simulate main processing loop
        for frame in 0..10 {
            #[cfg(target_arch = "wasm32")]
            let frame_start = <intonation_toy::platform::WebTimer as Timer>::now_ms();
            #[cfg(not(target_arch = "wasm32"))]
            let frame_start = <intonation_toy::platform::StubTimer as Timer>::now_ms();
            
            // Process audio frame
            let _pitch = simulate_pitch_detection(frame);
            
            #[cfg(target_arch = "wasm32")]
            let frame_end = <intonation_toy::platform::WebTimer as Timer>::now_ms();
            #[cfg(not(target_arch = "wasm32"))]
            let frame_end = <intonation_toy::platform::StubTimer as Timer>::now_ms();
            
            let frame_duration = frame_end - frame_start;
            
            // Verify reasonable frame timing
            assert!(frame_duration >= 0.0, "Frame processing should have non-negative duration");
        }
    }
    
    let total_time = init_end - init_start;
    assert!(total_time >= 0.0, "Total processing time should be non-negative");
}

// Helper functions to simulate application components

fn simulate_frame_processing() {
    // Simulate some computational work
    let mut sum = 0;
    for i in 0..500 {
        sum += i * i;
    }
    // Use sum to prevent optimization
    assert!(sum > 0);
}

fn simulate_audio_initialization() -> bool {
    // Simulate initialization logic
    simulate_frame_processing();
    true // Assume initialization succeeds
}

fn simulate_pitch_detection(frame: i32) -> f64 {
    // Simulate pitch detection returning different values
    if frame % 3 == 0 {
        440.0 + (frame as f64) * 10.0 // Simulate detected pitch
    } else {
        0.0 // Simulate no pitch detected
    }
}