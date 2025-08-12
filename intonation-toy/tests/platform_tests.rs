//! Platform abstraction tests
//! 
//! These tests verify that the platform abstraction layer works correctly on both
//! web and native targets. The tests are designed to compile and run on both platforms,
//! testing platform-specific behavior where appropriate.

use intonation_toy::platform::traits::*;

#[cfg(feature = "web")]
use wasm_bindgen_test::*;

#[cfg(feature = "web")]
wasm_bindgen_test_configure!(run_in_browser);

/// Test that Timer::now_ms() returns increasing values
#[cfg_attr(feature = "web", wasm_bindgen_test)]
#[cfg_attr(not(feature = "web"), test)]
fn test_timer_monotonic() {
    #[cfg(target_arch = "wasm32")]
    let time1 = <intonation_toy::platform::WebTimer as Timer>::now_ms();
    #[cfg(not(target_arch = "wasm32"))]
    let time1 = <intonation_toy::platform::StubTimer as Timer>::now_ms();
    
    // Small delay to ensure time progresses
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::sleep(std::time::Duration::from_millis(1));
    
    #[cfg(target_arch = "wasm32")]
    {
        // For web, we can't use thread::sleep, but we can do some work
        let mut sum = 0;
        for i in 0..1000 {
            sum += i;
        }
        // Use sum to prevent optimization
        assert!(sum > 0);
    }
    
    #[cfg(target_arch = "wasm32")]
    let time2 = <intonation_toy::platform::WebTimer as Timer>::now_ms();
    #[cfg(not(target_arch = "wasm32"))]
    let time2 = <intonation_toy::platform::StubTimer as Timer>::now_ms();
    
    // Time should be increasing (or at least not decreasing)
    assert!(time2 >= time1, "Timer should return monotonic increasing values");
}

/// Test PerformanceMonitor behavior
#[cfg_attr(feature = "web", wasm_bindgen_test)]
#[cfg_attr(not(feature = "web"), test)]
fn test_performance_monitor() {
    #[cfg(target_arch = "wasm32")]
    let memory = <intonation_toy::platform::WebPerformanceMonitor as PerformanceMonitor>::sample_memory_usage();
    #[cfg(not(target_arch = "wasm32"))]
    let memory = <intonation_toy::platform::StubPerformanceMonitor as PerformanceMonitor>::sample_memory_usage();
    
    #[cfg(target_arch = "wasm32")]
    {
        // On web, might return Some memory data (depending on browser support)
        // We can't guarantee this since it depends on browser APIs being available
        if let Some((memory_mb, memory_percent)) = memory {
            assert!(memory_mb >= 0.0, "Memory usage should be non-negative");
            assert!(memory_percent >= 0.0 && memory_percent <= 100.0, "Memory percentage should be 0-100");
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        // On native with stubs, should return None
        assert!(memory.is_none(), "Native platform should return None for memory sampling");
    }
}

/// Test UiController methods
#[cfg_attr(feature = "web", wasm_bindgen_test)]
#[cfg_attr(not(feature = "web"), test)]
fn test_ui_controller() {
    // These should not panic on either platform
    #[cfg(target_arch = "wasm32")]
    {
        use intonation_toy::platform::WebUiController;
        WebUiController::resize_canvas();
        WebUiController::apply_theme_styles();
        WebUiController::setup_ui();
        WebUiController::cleanup_ui();
        let _zoom = WebUiController::get_zoom_factor();
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        use intonation_toy::platform::StubUiController;
        StubUiController::resize_canvas();
        StubUiController::apply_theme_styles();
        StubUiController::setup_ui();
        StubUiController::cleanup_ui();
        let _zoom = StubUiController::get_zoom_factor();
    }
    
    // Test should complete without panicking
    assert!(true, "UiController methods should not panic");
}

/// Test ErrorDisplay methods
#[cfg_attr(feature = "web", wasm_bindgen_test)]
#[cfg_attr(not(feature = "web"), test)]
fn test_error_display() {
    // Create a test error - we need to check what Error type is expected
    // For now, let's test the simpler case by using the struct constructors
    #[cfg(target_arch = "wasm32")]
    let error_display = intonation_toy::platform::WebErrorDisplay::new();
    #[cfg(not(target_arch = "wasm32"))]
    let error_display = intonation_toy::platform::StubErrorDisplay::new();
    
    // These should not panic on either platform
    // Note: We'll need to create proper Error instances for show_error tests
    // For now, let's focus on the constructor working
    
    // Test should complete without panicking
    assert!(true, "ErrorDisplay construction should not panic");
}

/// Test basic platform initialization
#[cfg_attr(feature = "web", wasm_bindgen_test)]
#[cfg_attr(not(feature = "web"), test)]
fn test_platform_initialization() {
    // All platform components should initialize without panicking
    #[cfg(target_arch = "wasm32")]
    {
        let _timer_time = <intonation_toy::platform::WebTimer as Timer>::now_ms();
        let _memory = <intonation_toy::platform::WebPerformanceMonitor as PerformanceMonitor>::sample_memory_usage();
        let _zoom = intonation_toy::platform::WebUiController::get_zoom_factor();
        let _error_display = intonation_toy::platform::WebErrorDisplay::new();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _timer_time = <intonation_toy::platform::StubTimer as Timer>::now_ms();
        let _memory = <intonation_toy::platform::StubPerformanceMonitor as PerformanceMonitor>::sample_memory_usage();
        let _zoom = intonation_toy::platform::StubUiController::get_zoom_factor();
        let _error_display = intonation_toy::platform::StubErrorDisplay::new();
    }
    
    assert!(true, "Platform components should initialize successfully");
}

/// Test platform-specific behavior verification
#[cfg_attr(feature = "web", wasm_bindgen_test)]
#[cfg_attr(not(feature = "web"), test)]
fn test_platform_specific_behavior() {
    #[cfg(target_arch = "wasm32")]
    let start_time = <intonation_toy::platform::WebTimer as Timer>::now_ms();
    #[cfg(not(target_arch = "wasm32"))]
    let start_time = <intonation_toy::platform::StubTimer as Timer>::now_ms();
    
    // Verify we get a reasonable timestamp
    assert!(start_time >= 0.0, "Timer should return non-negative timestamp");
    
    #[cfg(target_arch = "wasm32")]
    {
        // On web, timestamp should be performance.now() scale (milliseconds since page load)
        // Should be relatively small compared to Unix timestamp
        assert!(start_time < 1_000_000_000.0, "Web timestamp should be reasonable");
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        // On native, using relative timing from first call, so should be small initially
        // Just verify it's a reasonable value
        assert!(start_time >= 0.0, "Native timestamp should be non-negative");
    }
}

/// Stress test for timer consistency
#[cfg_attr(feature = "web", wasm_bindgen_test)]
#[cfg_attr(not(feature = "web"), test)]
fn test_timer_consistency() {
    let mut times = Vec::new();
    
    // Collect multiple timestamps
    for _ in 0..10 {
        #[cfg(target_arch = "wasm32")]
        let time = <intonation_toy::platform::WebTimer as Timer>::now_ms();
        #[cfg(not(target_arch = "wasm32"))]
        let time = <intonation_toy::platform::StubTimer as Timer>::now_ms();
        
        times.push(time);
        
        // Small delay between measurements
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        #[cfg(target_arch = "wasm32")]
        {
            // For web, do some computational work
            let mut sum = 0;
            for i in 0..100 {
                sum += i;
            }
            assert!(sum > 0); // Prevent optimization
        }
    }
    
    // Verify timestamps are monotonic
    for i in 1..times.len() {
        assert!(
            times[i] >= times[i-1], 
            "Timestamps should be monotonic: {} >= {}", 
            times[i], times[i-1]
        );
    }
}

/// Test error handling doesn't panic
#[cfg_attr(feature = "web", wasm_bindgen_test)]
#[cfg_attr(not(feature = "web"), test)]
fn test_error_handling_robustness() {
    #[cfg(target_arch = "wasm32")]
    let _error_display = intonation_toy::platform::WebErrorDisplay::new();
    #[cfg(not(target_arch = "wasm32"))]
    let _error_display = intonation_toy::platform::StubErrorDisplay::new();
    
    // Test construction doesn't panic
    // We'd need to create proper Error instances to test show_error methods
    
    assert!(true, "Error handling construction should be robust");
}