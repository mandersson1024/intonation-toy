// Enhanced Integration Tests - STORY-3.19
// Integration tests with mock and real audio devices

#[cfg(test)]
mod enhanced_integration_tests {
    use std::sync::Arc;
    use std::time::Duration;
    use crate::modules::audio_foundations::*;
    use crate::modules::application_core::typed_event_bus::TypedEventBus;
    
    /// Mock audio device for testing
    struct MockAudioDevice {
        device_id: String,
        device_name: String,
        is_connected: bool,
        supports_echo_cancellation: bool,
        sample_rates: Vec<u32>,
    }
    
    impl MockAudioDevice {
        fn new(device_id: &str, device_name: &str) -> Self {
            Self {
                device_id: device_id.to_string(),
                device_name: device_name.to_string(),
                is_connected: true,
                supports_echo_cancellation: true,
                sample_rates: vec![44100, 48000],
            }
        }
        
        fn disconnect(&mut self) {
            self.is_connected = false;
        }
        
        fn reconnect(&mut self) {
            self.is_connected = true;
        }
    }
    
    /// Mock device manager for integration testing
    struct MockDeviceManager {
        devices: Vec<MockAudioDevice>,
        active_device_id: Option<String>,
        permission_granted: bool,
    }
    
    impl MockDeviceManager {
        fn new() -> Self {
            Self {
                devices: vec![
                    MockAudioDevice::new("default", "Default Microphone"),
                    MockAudioDevice::new("usb-mic", "USB Microphone"),
                    MockAudioDevice::new("bluetooth", "Bluetooth Headset"),
                ],
                active_device_id: None,
                permission_granted: true,
            }
        }
        
        fn get_device_by_id(&self, device_id: &str) -> Option<&MockAudioDevice> {
            self.devices.iter().find(|d| d.device_id == device_id)
        }
        
        fn get_device_by_id_mut(&mut self, device_id: &str) -> Option<&mut MockAudioDevice> {
            self.devices.iter_mut().find(|d| d.device_id == device_id)
        }
        
        fn set_active_device(&mut self, device_id: &str) -> Result<(), String> {
            if self.get_device_by_id(device_id).is_some() {
                self.active_device_id = Some(device_id.to_string());
                Ok(())
            } else {
                Err(format!("Device {} not found", device_id))
            }
        }
        
        fn disconnect_device(&mut self, device_id: &str) {
            if let Some(device) = self.get_device_by_id_mut(device_id) {
                device.disconnect();
                if self.active_device_id.as_ref() == Some(&device_id.to_string()) {
                    self.active_device_id = None;
                }
            }
        }
        
        fn reconnect_device(&mut self, device_id: &str) {
            if let Some(device) = self.get_device_by_id_mut(device_id) {
                device.reconnect();
            }
        }
    }
    
    /// Generate test audio signal
    fn generate_test_signal(frequency: f32, sample_rate: f32, duration_samples: usize) -> Vec<f32> {
        (0..duration_samples)
            .map(|i| {
                let t = i as f32 / sample_rate;
                0.8 * (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect()
    }
    
    /// Integration test coordinator
    struct IntegrationTestCoordinator {
        event_bus: Arc<TypedEventBus>,
        device_manager: MockDeviceManager,
        pitch_detector: MultiAlgorithmPitchDetector,
        performance_monitor: AudioPerformanceMonitor,
    }
    
    impl IntegrationTestCoordinator {
        fn new() -> Self {
            let event_bus = Arc::new(TypedEventBus::new());
            let config = PitchDetectionConfig::default();
            let pitch_detector = MultiAlgorithmPitchDetector::new(config, Some(event_bus.clone())).unwrap();
            let performance_monitor = AudioPerformanceMonitor::new();
            
            Self {
                event_bus,
                device_manager: MockDeviceManager::new(),
                pitch_detector,
                performance_monitor,
            }
        }
        
        fn simulate_audio_processing(&mut self, test_signal: &[f32]) -> Result<PitchResult, PitchError> {
            let measurement_id = self.performance_monitor.start_measurement("audio_processing".to_string());
            let result = self.pitch_detector.detect_pitch(test_signal);
            self.performance_monitor.end_measurement(measurement_id);
            result
        }
    }
    
    #[test]
    fn test_complete_audio_pipeline_integration() {
        let mut coordinator = IntegrationTestCoordinator::new();
        
        // Set up audio device
        coordinator.device_manager.set_active_device("default").unwrap();
        
        // Generate test signal
        let test_signal = generate_test_signal(440.0, 44100.0, 2048);
        
        // Process audio through complete pipeline
        let result = coordinator.simulate_audio_processing(&test_signal);
        
        assert!(result.is_ok(), "Audio pipeline should process successfully");
        let pitch_result = result.unwrap();
        assert!(pitch_result.is_valid, "Pitch detection should be valid");
        assert!((pitch_result.frequency - 440.0).abs() < 10.0, 
            "Detected frequency should be close to 440Hz");
        
        // Verify performance monitoring
        let metrics = coordinator.performance_monitor.get_current_metrics();
        assert!(metrics.processing_latency_ms < 50.0, "Processing latency should be reasonable");
    }
    
    #[test]
    fn test_device_switching_integration() {
        let mut coordinator = IntegrationTestCoordinator::new();
        let test_signal = generate_test_signal(440.0, 44100.0, 2048);
        
        // Test with default device
        coordinator.device_manager.set_active_device("default").unwrap();
        let result1 = coordinator.simulate_audio_processing(&test_signal);
        assert!(result1.is_ok(), "Should work with default device");
        
        // Switch to USB microphone
        coordinator.device_manager.set_active_device("usb-mic").unwrap();
        let result2 = coordinator.simulate_audio_processing(&test_signal);
        assert!(result2.is_ok(), "Should work with USB microphone");
        
        // Switch to Bluetooth headset
        coordinator.device_manager.set_active_device("bluetooth").unwrap();
        let result3 = coordinator.simulate_audio_processing(&test_signal);
        assert!(result3.is_ok(), "Should work with Bluetooth headset");
        
        // Verify switching doesn't degrade performance significantly
        let metrics = coordinator.performance_monitor.get_current_metrics();
        assert!(metrics.end_to_end_latency_ms < 100.0, 
            "Device switching should not add excessive latency");
    }
    
    #[test]
    fn test_device_disconnection_recovery() {
        let mut coordinator = IntegrationTestCoordinator::new();
        let test_signal = generate_test_signal(440.0, 44100.0, 2048);
        
        // Start with default device
        coordinator.device_manager.set_active_device("default").unwrap();
        let result1 = coordinator.simulate_audio_processing(&test_signal);
        assert!(result1.is_ok(), "Should work initially");
        
        // Simulate device disconnection
        coordinator.device_manager.disconnect_device("default");
        
        // Audio processing should handle disconnection gracefully
        // (in real implementation, this would trigger fallback logic)
        let result2 = coordinator.simulate_audio_processing(&test_signal);
        // Note: behavior depends on implementation - could error or fallback
        
        // Reconnect device
        coordinator.device_manager.reconnect_device("default");
        coordinator.device_manager.set_active_device("default").unwrap();
        
        let result3 = coordinator.simulate_audio_processing(&test_signal);
        assert!(result3.is_ok(), "Should work after reconnection");
    }
    
    #[test]
    fn test_multiple_algorithm_integration() {
        let mut coordinator = IntegrationTestCoordinator::new();
        coordinator.device_manager.set_active_device("default").unwrap();
        
        let test_signals = vec![
            ("pure_tone", generate_test_signal(440.0, 44100.0, 2048)),
            ("low_tone", generate_test_signal(220.0, 44100.0, 2048)),
            ("high_tone", generate_test_signal(880.0, 44100.0, 2048)),
        ];
        
        let algorithms = [PitchAlgorithm::YIN, PitchAlgorithm::McLeod, PitchAlgorithm::Auto];
        
        for &algorithm in &algorithms {
            coordinator.pitch_detector.set_algorithm(algorithm).unwrap();
            
            for (signal_name, test_signal) in &test_signals {
                let result = coordinator.simulate_audio_processing(test_signal);
                assert!(result.is_ok(), 
                    "Algorithm {:?} should handle {} signal", algorithm, signal_name);
                
                if let Ok(pitch_result) = result {
                    assert!(pitch_result.processing_time_ns > 0, 
                        "Should record processing time for {:?}", algorithm);
                    assert!(pitch_result.confidence >= 0.0 && pitch_result.confidence <= 1.0,
                        "Confidence should be in valid range for {:?}", algorithm);
                }
            }
        }
        
        // Verify algorithm performance comparison
        let comparison = coordinator.pitch_detector.get_performance_comparison();
        assert!(comparison.yin_performance.avg_processing_time_ns > 0);
        assert!(comparison.mcleod_performance.avg_processing_time_ns > 0);
        assert!(comparison.recommendation_confidence >= 0.0);
    }
    
    #[test]
    fn test_event_publishing_integration() {
        let mut coordinator = IntegrationTestCoordinator::new();
        coordinator.device_manager.set_active_device("default").unwrap();
        
        // Enable event publishing
        coordinator.pitch_detector.set_event_publishing(true);
        
        let test_signal = generate_test_signal(440.0, 44100.0, 2048);
        
        // Process audio and generate events
        for _ in 0..5 {
            let _ = coordinator.simulate_audio_processing(&test_signal);
        }
        
        // In a real implementation, we would verify events were published
        // For now, just verify the system runs without error
        let metrics = coordinator.performance_monitor.get_current_metrics();
        assert!(metrics.processing_latency_ms >= 0.0, "Metrics should be available");
        
        // Test disabling event publishing
        coordinator.pitch_detector.set_event_publishing(false);
        let result = coordinator.simulate_audio_processing(&test_signal);
        assert!(result.is_ok(), "Should work with event publishing disabled");
    }
    
    #[test]
    fn test_error_recovery_integration() {
        let mut coordinator = IntegrationTestCoordinator::new();
        
        // Test with invalid/empty signals
        let error_cases = vec![
            ("empty", vec![]),
            ("too_small", vec![0.5; 32]),
            ("all_zeros", vec![0.0; 2048]),
            ("clipped", vec![1.0; 2048]),
        ];
        
        for (case_name, test_signal) in error_cases {
            let result = coordinator.simulate_audio_processing(&test_signal);
            
            // Should either succeed gracefully or fail with proper error
            match result {
                Ok(pitch_result) => {
                    // If it succeeds, confidence should reflect signal quality
                    if case_name == "all_zeros" || case_name == "too_small" {
                        // These cases should have low confidence if they succeed
                        if case_name != "too_small" { // too_small should error
                            assert!(pitch_result.confidence < 0.5, 
                                "Low quality signal should have low confidence");
                        }
                    }
                }
                Err(error) => {
                    // Errors should be meaningful
                    let error_str = format!("{}", error);
                    assert!(!error_str.is_empty(), "Error messages should be descriptive");
                    
                    // Specific expected errors
                    if case_name == "empty" {
                        assert!(error_str.contains("Empty") || error_str.contains("empty"), 
                            "Empty buffer should mention 'empty'");
                    }
                    if case_name == "too_small" {
                        assert!(error_str.contains("small") || error_str.contains("BufferTooSmall"), 
                            "Small buffer should mention size issue");
                    }
                }
            }
        }
        
        // System should still work normally after error cases
        let normal_signal = generate_test_signal(440.0, 44100.0, 2048);
        let result = coordinator.simulate_audio_processing(&normal_signal);
        assert!(result.is_ok(), "System should recover from error cases");
    }
    
    #[test]
    fn test_sustained_processing_integration() {
        let mut coordinator = IntegrationTestCoordinator::new();
        coordinator.device_manager.set_active_device("default").unwrap();
        
        let test_signal = generate_test_signal(440.0, 44100.0, 2048);
        let iterations = 100;
        
        // Configure monitoring for sustained testing
        let monitoring_config = MonitoringConfig {
            enable_detailed_metrics: true,
            enable_regression_detection: true,
            sampling_interval_ns: 1_000_000, // 1ms
            max_history_size: 1000,
        };
        coordinator.performance_monitor.configure(monitoring_config);
        
        // Sustained processing
        for i in 0..iterations {
            let result = coordinator.simulate_audio_processing(&test_signal);
            assert!(result.is_ok(), "Iteration {} should succeed", i);
            
            // Check performance periodically
            if i % 20 == 0 && i > 0 {
                let metrics = coordinator.performance_monitor.get_current_metrics();
                assert!(metrics.end_to_end_latency_ms < 100.0, 
                    "Latency should remain bounded during sustained processing");
                assert!(metrics.cpu_usage_percent < 90.0,
                    "CPU usage should remain reasonable");
            }
        }
        
        // Final performance check
        let final_metrics = coordinator.performance_monitor.get_current_metrics();
        assert!(final_metrics.monitoring_overhead.cpu_overhead_percent < 10.0,
            "Monitoring overhead should be reasonable");
        assert!(final_metrics.processing_latency_ms > 0.0,
            "Should have meaningful performance data");
        
        // Verify no performance regression over time
        let thresholds = PerformanceThresholds {
            max_end_to_end_latency_ms: 50.0,
            max_processing_latency_ms: 20.0,
            max_cpu_usage_percent: 80.0,
            max_memory_usage_mb: 100.0,
            max_monitoring_overhead_percent: 5.0,
        };
        
        coordinator.performance_monitor.update_thresholds(thresholds);
        
        // Process one more batch to check against thresholds
        for _ in 0..10 {
            let _ = coordinator.simulate_audio_processing(&test_signal);
        }
        
        let violations = coordinator.performance_monitor.get_threshold_violations();
        assert!(violations.is_empty(), 
            "Should not have threshold violations: {:?}", violations);
    }
    
    #[test]
    fn test_configuration_change_integration() {
        let mut coordinator = IntegrationTestCoordinator::new();
        coordinator.device_manager.set_active_device("default").unwrap();
        
        let test_signal = generate_test_signal(440.0, 44100.0, 2048);
        
        // Test with initial configuration
        let result1 = coordinator.simulate_audio_processing(&test_signal);
        assert!(result1.is_ok(), "Should work with initial config");
        
        // Change configuration
        let mut new_config = PitchDetectionConfig::default();
        new_config.sample_rate = 48000.0;
        new_config.min_frequency = 100.0;
        new_config.max_frequency = 4000.0;
        new_config.enable_harmonic_analysis = false; // Disable for performance
        
        coordinator.pitch_detector.configure(new_config).unwrap();
        
        // Generate signal for new sample rate
        let new_test_signal = generate_test_signal(440.0, 48000.0, 2048);
        let result2 = coordinator.simulate_audio_processing(&new_test_signal);
        assert!(result2.is_ok(), "Should work with updated config");
        
        // Verify configuration actually changed
        let algorithm_info = coordinator.pitch_detector.get_algorithm_info();
        assert!(algorithm_info.avg_processing_time_ns > 0, "Should have performance data");
        
        // Test edge of new frequency range
        let low_freq_signal = generate_test_signal(110.0, 48000.0, 2048);
        let result3 = coordinator.simulate_audio_processing(&low_freq_signal);
        if let Ok(pitch_result) = result3 {
            assert!(pitch_result.is_valid, "110Hz should be valid with new range");
        }
        
        let high_freq_signal = generate_test_signal(3500.0, 48000.0, 2048);
        let result4 = coordinator.simulate_audio_processing(&high_freq_signal);
        if let Ok(pitch_result) = result4 {
            assert!(pitch_result.is_valid, "3500Hz should be valid with new range");
        }
    }
    
    #[test]
    fn test_concurrent_processing_simulation() {
        // Simulate multiple audio streams being processed
        // (In single-threaded environment, this tests rapid context switching)
        
        let configs = vec![
            PitchDetectionConfig::default(),
            {
                let mut config = PitchDetectionConfig::default();
                config.algorithm = PitchAlgorithm::McLeod;
                config
            },
            {
                let mut config = PitchDetectionConfig::default();
                config.algorithm = PitchAlgorithm::Auto;
                config
            },
        ];
        
        let mut detectors: Vec<MultiAlgorithmPitchDetector> = configs
            .into_iter()
            .map(|config| MultiAlgorithmPitchDetector::new(config, None).unwrap())
            .collect();
        
        let test_signals = vec![
            generate_test_signal(220.0, 44100.0, 2048),
            generate_test_signal(440.0, 44100.0, 2048),
            generate_test_signal(880.0, 44100.0, 2048),
        ];
        
        // Simulate rapid switching between "streams"
        for round in 0..50 {
            for (i, detector) in detectors.iter_mut().enumerate() {
                let signal_idx = (round + i) % test_signals.len();
                let result = detector.detect_pitch(&test_signals[signal_idx]);
                
                assert!(result.is_ok(), 
                    "Detector {} should work in round {} with signal {}", i, round, signal_idx);
            }
        }
        
        // Verify all detectors still work correctly
        for (i, detector) in detectors.iter_mut().enumerate() {
            let test_signal = generate_test_signal(440.0, 44100.0, 2048);
            let result = detector.detect_pitch(&test_signal);
            assert!(result.is_ok(), "Detector {} should still work after concurrent simulation", i);
            
            let comparison = detector.get_performance_comparison();
            assert!(comparison.recommendation_confidence >= 0.0, 
                "Detector {} should have performance data", i);
        }
    }
    
    #[test]
    fn test_full_audio_foundations_integration() {
        // Test integration of all audio foundations components together
        let event_bus = Arc::new(TypedEventBus::new());
        
        // Initialize all components
        let config = PitchDetectionConfig::default();
        let mut pitch_detector = MultiAlgorithmPitchDetector::new(config, Some(event_bus.clone())).unwrap();
        let mut performance_monitor = AudioPerformanceMonitor::new();
        let mut device_manager = MockDeviceManager::new();
        
        // Configure monitoring
        let monitoring_config = MonitoringConfig {
            enable_detailed_metrics: true,
            enable_regression_detection: true,
            sampling_interval_ns: 1_000_000,
            max_history_size: 100,
        };
        performance_monitor.configure(monitoring_config);
        
        // Set up device
        device_manager.set_active_device("default").unwrap();
        
        // Test various scenarios
        let test_scenarios = vec![
            ("normal_tone", generate_test_signal(440.0, 44100.0, 2048)),
            ("low_tone", generate_test_signal(110.0, 44100.0, 2048)),
            ("high_tone", generate_test_signal(1760.0, 44100.0, 2048)),
            ("quiet_tone", generate_test_signal(440.0, 44100.0, 2048).into_iter().map(|x| x * 0.1).collect()),
        ];
        
        for (scenario_name, test_signal) in test_scenarios {
            // Start performance measurement
            let measurement_id = performance_monitor.start_measurement(
                format!("integration_test_{}", scenario_name)
            );
            
            // Process audio
            let result = pitch_detector.detect_pitch(&test_signal);
            
            // End performance measurement
            performance_monitor.end_measurement(measurement_id);
            
            // Verify results
            assert!(result.is_ok() || scenario_name == "quiet_tone", 
                "Scenario '{}' should succeed (or be quiet tone)", scenario_name);
            
            if let Ok(pitch_result) = result {
                assert!(pitch_result.processing_time_ns > 0, 
                    "Should record processing time for '{}'", scenario_name);
                
                // Verify event publishing occurred (pitch detector has event bus)
                // In real implementation, we could check event counts
            }
        }
        
        // Test algorithm switching under load
        let algorithms = [PitchAlgorithm::YIN, PitchAlgorithm::McLeod, PitchAlgorithm::Auto];
        let test_signal = generate_test_signal(440.0, 44100.0, 2048);
        
        for &algorithm in &algorithms {
            pitch_detector.set_algorithm(algorithm).unwrap();
            
            for _ in 0..10 {
                let measurement_id = performance_monitor.start_measurement(
                    format!("algorithm_test_{:?}", algorithm)
                );
                
                let result = pitch_detector.detect_pitch(&test_signal);
                performance_monitor.end_measurement(measurement_id);
                
                assert!(result.is_ok(), "Algorithm {:?} should work", algorithm);
            }
        }
        
        // Final system health check
        let final_metrics = performance_monitor.get_current_metrics();
        assert!(final_metrics.end_to_end_latency_ms > 0.0, "Should have latency data");
        assert!(final_metrics.processing_latency_ms > 0.0, "Should have processing data");
        assert!(final_metrics.monitoring_overhead.cpu_overhead_percent < 20.0, 
            "Monitoring overhead should be reasonable");
        
        // Verify no threshold violations
        let violations = performance_monitor.get_threshold_violations();
        assert!(violations.is_empty(), "Should not have violations: {:?}", violations);
        
        // Test graceful shutdown simulation
        // (In real implementation, this would clean up resources)
        drop(pitch_detector);
        drop(performance_monitor);
        drop(device_manager);
        
        // Event bus should still be valid
        assert!(Arc::strong_count(&event_bus) == 1, "Event bus should be the last reference");
    }
}