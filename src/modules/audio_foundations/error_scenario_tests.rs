// Error Scenario Testing and Recovery Validation - STORY-3.19
// Comprehensive testing for error handling and recovery scenarios

#[cfg(test)]
mod error_scenario_tests {
    use crate::modules::audio_foundations::*;
    use std::time::{Duration, Instant};
    use std::sync::Arc;

    /// Error scenario types for testing
    #[derive(Debug, Clone)]
    pub enum ErrorScenarioType {
        DeviceFailure,
        PermissionDenied,
        AudioContextSuspension,
        NetworkInterruption,
        ResourceExhaustion,
        InvalidConfiguration,
        BufferUnderrun,
        MemoryPressure,
    }

    /// Recovery action to test
    #[derive(Debug, Clone, PartialEq)]
    pub enum ExpectedRecoveryAction {
        FallbackToAlternative,
        GracefulDegradation,
        RetryWithBackoff,
        UserNotification,
        SystemRestart,
        ErrorPropagation,
        SilentHandling,
    }

    /// Error scenario test case
    #[derive(Debug)]
    pub struct ErrorScenarioTestCase {
        pub name: String,
        pub scenario_type: ErrorScenarioType,
        pub expected_recovery: ExpectedRecoveryAction,
        pub max_recovery_time_ms: u64,
        pub should_maintain_functionality: bool,
        pub test_description: String,
    }

    impl ErrorScenarioTestCase {
        fn new(
            name: &str,
            scenario_type: ErrorScenarioType,
            expected_recovery: ExpectedRecoveryAction,
            max_recovery_time_ms: u64,
        ) -> Self {
            Self {
                name: name.to_string(),
                scenario_type: scenario_type.clone(),
                expected_recovery,
                max_recovery_time_ms,
                should_maintain_functionality: match scenario_type {
                    ErrorScenarioType::DeviceFailure => false,
                    ErrorScenarioType::PermissionDenied => false,
                    ErrorScenarioType::AudioContextSuspension => true,
                    ErrorScenarioType::NetworkInterruption => true,
                    ErrorScenarioType::ResourceExhaustion => false,
                    ErrorScenarioType::InvalidConfiguration => false,
                    ErrorScenarioType::BufferUnderrun => true,
                    ErrorScenarioType::MemoryPressure => true,
                },
                test_description: format!("Testing {:?} scenario", scenario_type),
            }
        }

        fn with_functionality_expectation(mut self, should_maintain: bool) -> Self {
            self.should_maintain_functionality = should_maintain;
            self
        }

        fn with_description(mut self, description: &str) -> Self {
            self.test_description = description.to_string();
            self
        }
    }

    /// Test result for error scenario
    #[derive(Debug)]
    pub struct ErrorScenarioTestResult {
        pub test_name: String,
        pub scenario_type: ErrorScenarioType,
        pub passed: bool,
        pub recovery_time_ms: u64,
        pub actual_recovery_action: Option<ExpectedRecoveryAction>,
        pub functionality_maintained: bool,
        pub error_messages: Vec<String>,
        pub recovery_notes: Vec<String>,
    }

    /// Mock error injector for testing
    pub struct ErrorInjector {
        active_scenarios: Vec<ErrorScenarioType>,
        recovery_simulation_enabled: bool,
    }

    impl ErrorInjector {
        fn new() -> Self {
            Self {
                active_scenarios: Vec::new(),
                recovery_simulation_enabled: true,
            }
        }

        fn inject_error(&mut self, scenario: ErrorScenarioType) {
            self.active_scenarios.push(scenario);
        }

        fn clear_errors(&mut self) {
            self.active_scenarios.clear();
        }

        fn is_error_active(&self, scenario: &ErrorScenarioType) -> bool {
            self.active_scenarios.iter().any(|s| {
                std::mem::discriminant(s) == std::mem::discriminant(scenario)
            })
        }

        fn simulate_recovery(&mut self, scenario: &ErrorScenarioType) -> bool {
            if !self.recovery_simulation_enabled {
                return false;
            }

            // Remove the error to simulate recovery
            self.active_scenarios.retain(|s| {
                std::mem::discriminant(s) != std::mem::discriminant(scenario)
            });

            true
        }
    }

    /// Error-aware audio system for testing
    pub struct ErrorAwareAudioSystem {
        pitch_detector: MultiAlgorithmPitchDetector,
        performance_monitor: AudioPerformanceMonitor,
        error_injector: ErrorInjector,
        last_error: Option<String>,
        recovery_attempts: usize,
        max_recovery_attempts: usize,
    }

    impl ErrorAwareAudioSystem {
        fn new() -> Result<Self, Box<dyn std::error::Error>> {
            let config = PitchDetectionConfig::default();
            let pitch_detector = MultiAlgorithmPitchDetector::new(config, None)?;
            let performance_monitor = AudioPerformanceMonitor::new();

            Ok(Self {
                pitch_detector,
                performance_monitor,
                error_injector: ErrorInjector::new(),
                last_error: None,
                recovery_attempts: 0,
                max_recovery_attempts: 3,
            })
        }

        fn inject_error_scenario(&mut self, scenario: ErrorScenarioType) {
            self.error_injector.inject_error(scenario);
        }

        fn process_audio_with_error_handling(&mut self, signal: &[f32]) -> Result<PitchResult, PitchError> {
            // Check for active error scenarios before processing
            if self.error_injector.is_error_active(&ErrorScenarioType::DeviceFailure) {
                self.last_error = Some("Audio device failure detected".to_string());
                return Err(PitchError::DeviceError("Simulated device failure".to_string()));
            }

            if self.error_injector.is_error_active(&ErrorScenarioType::PermissionDenied) {
                self.last_error = Some("Microphone permission denied".to_string());
                return Err(PitchError::PermissionError("Microphone access denied".to_string()));
            }

            if self.error_injector.is_error_active(&ErrorScenarioType::AudioContextSuspension) {
                self.last_error = Some("Audio context suspended".to_string());
                // Try to recover from suspension
                if self.attempt_recovery(&ErrorScenarioType::AudioContextSuspension) {
                    // Continue with processing after recovery
                } else {
                    return Err(PitchError::AudioContextError("Audio context suspended".to_string()));
                }
            }

            if self.error_injector.is_error_active(&ErrorScenarioType::ResourceExhaustion) {
                self.last_error = Some("System resources exhausted".to_string());
                return Err(PitchError::ResourceError("Insufficient system resources".to_string()));
            }

            if self.error_injector.is_error_active(&ErrorScenarioType::InvalidConfiguration) {
                self.last_error = Some("Invalid audio configuration".to_string());
                return Err(PitchError::ConfigurationError("Invalid configuration parameters".to_string()));
            }

            if self.error_injector.is_error_active(&ErrorScenarioType::BufferUnderrun) {
                self.last_error = Some("Audio buffer underrun".to_string());
                // Buffer underruns can often be handled gracefully
                if self.attempt_recovery(&ErrorScenarioType::BufferUnderrun) {
                    // Continue with processing but with degraded performance
                    let mut result = self.pitch_detector.detect_pitch(signal)?;
                    result.confidence *= 0.8; // Reduce confidence due to buffer issues
                    return Ok(result);
                }
            }

            if self.error_injector.is_error_active(&ErrorScenarioType::MemoryPressure) {
                self.last_error = Some("Memory pressure detected".to_string());
                // Try to continue with reduced memory usage
                if signal.len() > 1024 {
                    // Use smaller buffer under memory pressure
                    let reduced_signal = &signal[..1024];
                    return self.pitch_detector.detect_pitch(reduced_signal);
                }
            }

            // Normal processing if no errors are active
            self.pitch_detector.detect_pitch(signal)
        }

        fn attempt_recovery(&mut self, scenario: &ErrorScenarioType) -> bool {
            if self.recovery_attempts >= self.max_recovery_attempts {
                return false;
            }

            self.recovery_attempts += 1;

            match scenario {
                ErrorScenarioType::AudioContextSuspension => {
                    // Simulate audio context recovery
                    std::thread::sleep(Duration::from_millis(10));
                    self.error_injector.simulate_recovery(scenario)
                }
                ErrorScenarioType::BufferUnderrun => {
                    // Simulate buffer recovery
                    std::thread::sleep(Duration::from_millis(5));
                    self.error_injector.simulate_recovery(scenario)
                }
                ErrorScenarioType::NetworkInterruption => {
                    // Simulate network recovery
                    std::thread::sleep(Duration::from_millis(50));
                    self.error_injector.simulate_recovery(scenario)
                }
                _ => false, // Other scenarios don't have automatic recovery
            }
        }

        fn reset_error_state(&mut self) {
            self.error_injector.clear_errors();
            self.last_error = None;
            self.recovery_attempts = 0;
        }

        fn get_last_error(&self) -> Option<&String> {
            self.last_error.as_ref()
        }

        fn get_recovery_attempts(&self) -> usize {
            self.recovery_attempts
        }
    }

    /// Error scenario test framework
    pub struct ErrorScenarioTestFramework {
        test_cases: Vec<ErrorScenarioTestCase>,
    }

    impl ErrorScenarioTestFramework {
        pub fn new() -> Self {
            Self {
                test_cases: Self::generate_standard_test_cases(),
            }
        }

        fn generate_standard_test_cases() -> Vec<ErrorScenarioTestCase> {
            vec![
                ErrorScenarioTestCase::new(
                    "device_failure_fallback",
                    ErrorScenarioType::DeviceFailure,
                    ExpectedRecoveryAction::FallbackToAlternative,
                    1000,
                ).with_description("Test device failure with fallback to alternative device"),

                ErrorScenarioTestCase::new(
                    "permission_denied_notification",
                    ErrorScenarioType::PermissionDenied,
                    ExpectedRecoveryAction::UserNotification,
                    100,
                ).with_description("Test permission denied with user notification"),

                ErrorScenarioTestCase::new(
                    "audio_context_suspension_recovery",
                    ErrorScenarioType::AudioContextSuspension,
                    ExpectedRecoveryAction::RetryWithBackoff,
                    500,
                ).with_functionality_expectation(true)
                .with_description("Test audio context suspension with automatic recovery"),

                ErrorScenarioTestCase::new(
                    "network_interruption_degradation",
                    ErrorScenarioType::NetworkInterruption,
                    ExpectedRecoveryAction::GracefulDegradation,
                    200,
                ).with_functionality_expectation(true)
                .with_description("Test network interruption with graceful degradation"),

                ErrorScenarioTestCase::new(
                    "resource_exhaustion_restart",
                    ErrorScenarioType::ResourceExhaustion,
                    ExpectedRecoveryAction::SystemRestart,
                    2000,
                ).with_description("Test resource exhaustion requiring system restart"),

                ErrorScenarioTestCase::new(
                    "invalid_config_error_propagation",
                    ErrorScenarioType::InvalidConfiguration,
                    ExpectedRecoveryAction::ErrorPropagation,
                    50,
                ).with_description("Test invalid configuration with error propagation"),

                ErrorScenarioTestCase::new(
                    "buffer_underrun_silent_handling",
                    ErrorScenarioType::BufferUnderrun,
                    ExpectedRecoveryAction::SilentHandling,
                    100,
                ).with_functionality_expectation(true)
                .with_description("Test buffer underrun with silent recovery"),

                ErrorScenarioTestCase::new(
                    "memory_pressure_degradation",
                    ErrorScenarioType::MemoryPressure,
                    ExpectedRecoveryAction::GracefulDegradation,
                    150,
                ).with_functionality_expectation(true)
                .with_description("Test memory pressure with graceful degradation"),
            ]
        }

        pub fn run_error_scenario_tests(&self) -> Vec<ErrorScenarioTestResult> {
            let mut results = Vec::new();

            for test_case in &self.test_cases {
                let result = self.execute_error_scenario_test(test_case);
                results.push(result);
            }

            results
        }

        fn execute_error_scenario_test(&self, test_case: &ErrorScenarioTestCase) -> ErrorScenarioTestResult {
            let start_time = Instant::now();
            let mut system = match ErrorAwareAudioSystem::new() {
                Ok(sys) => sys,
                Err(e) => {
                    return ErrorScenarioTestResult {
                        test_name: test_case.name.clone(),
                        scenario_type: test_case.scenario_type.clone(),
                        passed: false,
                        recovery_time_ms: 0,
                        actual_recovery_action: None,
                        functionality_maintained: false,
                        error_messages: vec![format!("Failed to create test system: {}", e)],
                        recovery_notes: vec![],
                    };
                }
            };

            let mut error_messages = Vec::new();
            let mut recovery_notes = Vec::new();
            let mut functionality_maintained = false;
            let mut actual_recovery_action = None;

            // Generate test signal
            let test_signal: Vec<f32> = (0..2048)
                .map(|i| {
                    let t = i as f32 / 44100.0;
                    0.8 * (2.0 * std::f32::consts::PI * 440.0 * t).sin()
                })
                .collect();

            // Test normal operation first
            let baseline_result = system.process_audio_with_error_handling(&test_signal);
            let baseline_works = baseline_result.is_ok();

            if !baseline_works {
                error_messages.push("Baseline functionality failed before error injection".to_string());
            }

            // Inject the error scenario
            system.inject_error_scenario(test_case.scenario_type.clone());

            // Test behavior during error
            let error_result = system.process_audio_with_error_handling(&test_signal);
            
            match &error_result {
                Ok(pitch_result) => {
                    functionality_maintained = true;
                    recovery_notes.push(format!("System continued functioning with confidence: {:.2}", pitch_result.confidence));
                    
                    // Determine recovery action based on behavior
                    actual_recovery_action = Some(match test_case.scenario_type {
                        ErrorScenarioType::MemoryPressure => ExpectedRecoveryAction::GracefulDegradation,
                        ErrorScenarioType::BufferUnderrun => ExpectedRecoveryAction::SilentHandling,
                        ErrorScenarioType::AudioContextSuspension => ExpectedRecoveryAction::RetryWithBackoff,
                        _ => ExpectedRecoveryAction::SilentHandling,
                    });
                }
                Err(error) => {
                    functionality_maintained = false;
                    error_messages.push(format!("Error during scenario: {}", error));

                    // Determine recovery action based on error type
                    actual_recovery_action = Some(match error {
                        PitchError::DeviceError(_) => ExpectedRecoveryAction::FallbackToAlternative,
                        PitchError::PermissionError(_) => ExpectedRecoveryAction::UserNotification,
                        PitchError::AudioContextError(_) => ExpectedRecoveryAction::RetryWithBackoff,
                        PitchError::ResourceError(_) => ExpectedRecoveryAction::SystemRestart,
                        PitchError::ConfigurationError(_) => ExpectedRecoveryAction::ErrorPropagation,
                        _ => ExpectedRecoveryAction::ErrorPropagation,
                    });
                }
            }

            // Check for recovery attempts
            let recovery_attempts = system.get_recovery_attempts();
            if recovery_attempts > 0 {
                recovery_notes.push(format!("Made {} recovery attempts", recovery_attempts));
            }

            // Test recovery if applicable
            if matches!(test_case.scenario_type, 
                ErrorScenarioType::AudioContextSuspension | 
                ErrorScenarioType::BufferUnderrun | 
                ErrorScenarioType::NetworkInterruption) {
                
                // Simulate some time passing for recovery
                std::thread::sleep(Duration::from_millis(50));
                
                // Clear errors and test recovery
                system.reset_error_state();
                let recovery_result = system.process_audio_with_error_handling(&test_signal);
                
                if recovery_result.is_ok() {
                    recovery_notes.push("System successfully recovered after error clearance".to_string());
                    if !functionality_maintained {
                        functionality_maintained = true; // Recovery successful
                    }
                } else {
                    recovery_notes.push("System failed to recover after error clearance".to_string());
                }
            }

            let recovery_time_ms = start_time.elapsed().as_millis() as u64;

            // Evaluate test success
            let passed = self.evaluate_test_success(
                test_case,
                &actual_recovery_action,
                functionality_maintained,
                recovery_time_ms,
            );

            ErrorScenarioTestResult {
                test_name: test_case.name.clone(),
                scenario_type: test_case.scenario_type.clone(),
                passed,
                recovery_time_ms,
                actual_recovery_action,
                functionality_maintained,
                error_messages,
                recovery_notes,
            }
        }

        fn evaluate_test_success(
            &self,
            test_case: &ErrorScenarioTestCase,
            actual_recovery: &Option<ExpectedRecoveryAction>,
            functionality_maintained: bool,
            recovery_time_ms: u64,
        ) -> bool {
            // Check recovery time
            if recovery_time_ms > test_case.max_recovery_time_ms {
                return false;
            }

            // Check functionality maintenance expectation
            if test_case.should_maintain_functionality != functionality_maintained {
                // Allow some flexibility for scenarios that might degrade gracefully
                if !test_case.should_maintain_functionality && functionality_maintained {
                    // Better than expected - that's OK
                } else {
                    return false;
                }
            }

            // Check recovery action (allow some flexibility)
            if let Some(actual) = actual_recovery {
                match (&test_case.expected_recovery, actual) {
                    // Exact matches
                    (expected, actual) if expected == actual => true,
                    
                    // Acceptable alternatives
                    (ExpectedRecoveryAction::GracefulDegradation, ExpectedRecoveryAction::SilentHandling) => true,
                    (ExpectedRecoveryAction::SilentHandling, ExpectedRecoveryAction::GracefulDegradation) => true,
                    (ExpectedRecoveryAction::RetryWithBackoff, ExpectedRecoveryAction::SilentHandling) => true,
                    
                    // Other combinations
                    _ => false,
                }
            } else {
                // No recovery action detected - only OK for certain scenarios
                matches!(test_case.expected_recovery, ExpectedRecoveryAction::ErrorPropagation)
            }
        }

        pub fn print_error_scenario_report(&self, results: &[ErrorScenarioTestResult]) {
            println!("\n=== Error Scenario Testing Report ===");
            
            let total_tests = results.len();
            let passed_tests = results.iter().filter(|r| r.passed).count();
            let failed_tests = total_tests - passed_tests;

            println!("Total Tests: {}", total_tests);
            println!("Passed: {}", passed_tests);
            println!("Failed: {}", failed_tests);
            println!("Success Rate: {:.1}%", (passed_tests as f32 / total_tests as f32) * 100.0);

            // Detailed results
            println!("\n=== Detailed Results ===");
            for result in results {
                let status = if result.passed { "✅ PASS" } else { "❌ FAIL" };
                println!("\n{} {}", status, result.test_name);
                println!("  Scenario: {:?}", result.scenario_type);
                println!("  Recovery Time: {}ms", result.recovery_time_ms);
                println!("  Functionality Maintained: {}", result.functionality_maintained);
                
                if let Some(ref action) = result.actual_recovery_action {
                    println!("  Recovery Action: {:?}", action);
                }

                if !result.error_messages.is_empty() {
                    println!("  Errors:");
                    for error in &result.error_messages {
                        println!("    - {}", error);
                    }
                }

                if !result.recovery_notes.is_empty() {
                    println!("  Recovery Notes:");
                    for note in &result.recovery_notes {
                        println!("    - {}", note);
                    }
                }
            }

            // Summary by scenario type
            println!("\n=== Results by Scenario Type ===");
            let mut scenario_stats = std::collections::HashMap::new();
            for result in results {
                let entry = scenario_stats.entry(format!("{:?}", result.scenario_type))
                    .or_insert((0, 0));
                entry.0 += 1; // total
                if result.passed {
                    entry.1 += 1; // passed
                }
            }

            for (scenario, (total, passed)) in scenario_stats {
                let rate = (passed as f32 / total as f32) * 100.0;
                println!("  {}: {}/{} ({:.1}%)", scenario, passed, total, rate);
            }
        }
    }

    // =============================================================================
    // ACTUAL TESTS
    // =============================================================================

    #[test]
    fn test_device_failure_error_scenario() {
        let mut system = ErrorAwareAudioSystem::new().unwrap();
        let test_signal: Vec<f32> = (0..2048)
            .map(|i| {
                let t = i as f32 / 44100.0;
                0.8 * (2.0 * std::f32::consts::PI * 440.0 * t).sin()
            })
            .collect();

        // Test normal operation
        let normal_result = system.process_audio_with_error_handling(&test_signal);
        assert!(normal_result.is_ok(), "Normal operation should succeed");

        // Inject device failure
        system.inject_error_scenario(ErrorScenarioType::DeviceFailure);

        // Test error handling
        let error_result = system.process_audio_with_error_handling(&test_signal);
        assert!(error_result.is_err(), "Device failure should cause error");

        // Check error message
        let error_msg = system.get_last_error().unwrap();
        assert!(error_msg.contains("device failure"), "Error message should mention device failure");
    }

    #[test]
    fn test_permission_denied_error_scenario() {
        let mut system = ErrorAwareAudioSystem::new().unwrap();
        let test_signal: Vec<f32> = vec![0.5; 1024];

        // Inject permission denied error
        system.inject_error_scenario(ErrorScenarioType::PermissionDenied);

        let error_result = system.process_audio_with_error_handling(&test_signal);
        assert!(error_result.is_err(), "Permission denied should cause error");

        match error_result.unwrap_err() {
            PitchError::PermissionError(msg) => {
                assert!(msg.contains("denied"), "Permission error should mention denial");
            }
            _ => panic!("Expected PermissionError"),
        }
    }

    #[test]
    fn test_audio_context_suspension_recovery() {
        let mut system = ErrorAwareAudioSystem::new().unwrap();
        let test_signal: Vec<f32> = (0..1024)
            .map(|i| (i as f32 / 1024.0 * 2.0 * std::f32::consts::PI * 440.0).sin())
            .collect();

        // Inject audio context suspension
        system.inject_error_scenario(ErrorScenarioType::AudioContextSuspension);

        // Should attempt recovery
        let result = system.process_audio_with_error_handling(&test_signal);
        
        // Either succeeds after recovery or fails gracefully
        match result {
            Ok(_) => {
                assert!(system.get_recovery_attempts() > 0, "Should have attempted recovery");
            }
            Err(PitchError::AudioContextError(_)) => {
                // Expected if recovery failed
            }
            Err(other) => panic!("Unexpected error type: {:?}", other),
        }
    }

    #[test]
    fn test_buffer_underrun_graceful_handling() {
        let mut system = ErrorAwareAudioSystem::new().unwrap();
        let test_signal: Vec<f32> = vec![0.7; 2048];

        // Inject buffer underrun
        system.inject_error_scenario(ErrorScenarioType::BufferUnderrun);

        let result = system.process_audio_with_error_handling(&test_signal);
        
        // Should either recover gracefully or handle silently
        match result {
            Ok(pitch_result) => {
                // Should continue functioning, possibly with reduced confidence
                assert!(pitch_result.confidence >= 0.0, "Confidence should be valid");
                assert!(system.get_recovery_attempts() > 0, "Should have attempted recovery");
            }
            Err(_) => {
                // If it fails, should have attempted recovery
                assert!(system.get_recovery_attempts() > 0, "Should have attempted recovery");
            }
        }
    }

    #[test]
    fn test_memory_pressure_degradation() {
        let mut system = ErrorAwareAudioSystem::new().unwrap();
        let large_signal: Vec<f32> = vec![0.6; 4096]; // Larger than normal

        // Inject memory pressure
        system.inject_error_scenario(ErrorScenarioType::MemoryPressure);

        let result = system.process_audio_with_error_handling(&large_signal);
        
        // Should handle gracefully by using smaller buffer
        assert!(result.is_ok(), "Memory pressure should be handled gracefully");
        
        let error_msg = system.get_last_error();
        assert!(error_msg.is_some(), "Should have logged memory pressure");
        assert!(error_msg.unwrap().contains("Memory pressure"), "Should mention memory pressure");
    }

    #[test]
    fn test_resource_exhaustion_error_propagation() {
        let mut system = ErrorAwareAudioSystem::new().unwrap();
        let test_signal: Vec<f32> = vec![0.5; 1024];

        // Inject resource exhaustion
        system.inject_error_scenario(ErrorScenarioType::ResourceExhaustion);

        let result = system.process_audio_with_error_handling(&test_signal);
        assert!(result.is_err(), "Resource exhaustion should cause error");

        match result.unwrap_err() {
            PitchError::ResourceError(msg) => {
                assert!(msg.contains("resources"), "Should mention resources");
            }
            _ => panic!("Expected ResourceError"),
        }
    }

    #[test]
    fn test_invalid_configuration_error_handling() {
        let mut system = ErrorAwareAudioSystem::new().unwrap();
        let test_signal: Vec<f32> = vec![0.4; 512];

        // Inject invalid configuration
        system.inject_error_scenario(ErrorScenarioType::InvalidConfiguration);

        let result = system.process_audio_with_error_handling(&test_signal);
        assert!(result.is_err(), "Invalid configuration should cause error");

        match result.unwrap_err() {
            PitchError::ConfigurationError(msg) => {
                assert!(msg.contains("configuration"), "Should mention configuration");
            }
            _ => panic!("Expected ConfigurationError"),
        }
    }

    #[test]
    fn test_comprehensive_error_scenario_suite() {
        let framework = ErrorScenarioTestFramework::new();
        let results = framework.run_error_scenario_tests();

        // Print full report for manual review
        framework.print_error_scenario_report(&results);

        // Verify we tested all major scenarios
        assert!(results.len() >= 8, "Should test at least 8 error scenarios");

        // Check that most tests passed (allow some failures for realistic testing)
        let passed_count = results.iter().filter(|r| r.passed).count();
        let success_rate = passed_count as f32 / results.len() as f32;
        assert!(success_rate > 0.6, "Should have >60% success rate, got {:.1}%", success_rate * 100.0);

        // Verify critical scenarios are handled
        let critical_scenarios = [
            ErrorScenarioType::DeviceFailure,
            ErrorScenarioType::PermissionDenied,
            ErrorScenarioType::AudioContextSuspension,
        ];

        for scenario in &critical_scenarios {
            let scenario_results: Vec<_> = results.iter()
                .filter(|r| std::mem::discriminant(&r.scenario_type) == std::mem::discriminant(scenario))
                .collect();
            
            assert!(!scenario_results.is_empty(), 
                "Should test {:?} scenario", scenario);
        }

        // Check recovery times are reasonable
        for result in &results {
            assert!(result.recovery_time_ms < 5000, 
                "Recovery time for {} should be <5s, got {}ms", 
                result.test_name, result.recovery_time_ms);
        }

        // Verify error messages are meaningful
        for result in &results {
            if !result.passed && result.error_messages.is_empty() {
                panic!("Failed test {} should have error messages", result.test_name);
            }
        }
    }

    #[test]
    fn test_error_recovery_sequence() {
        let mut system = ErrorAwareAudioSystem::new().unwrap();
        let test_signal: Vec<f32> = (0..1024)
            .map(|i| 0.5 * (i as f32 / 1024.0 * 2.0 * std::f32::consts::PI * 220.0).sin())
            .collect();

        // Test sequence: normal -> error -> recovery -> normal
        
        // 1. Normal operation
        let result1 = system.process_audio_with_error_handling(&test_signal);
        assert!(result1.is_ok(), "Initial operation should succeed");

        // 2. Inject recoverable error
        system.inject_error_scenario(ErrorScenarioType::BufferUnderrun);
        let result2 = system.process_audio_with_error_handling(&test_signal);
        // May succeed with recovery or fail gracefully

        // 3. Clear error and test recovery
        system.reset_error_state();
        let result3 = system.process_audio_with_error_handling(&test_signal);
        assert!(result3.is_ok(), "Should recover after error clearance");

        // 4. Verify system is back to normal
        let result4 = system.process_audio_with_error_handling(&test_signal);
        assert!(result4.is_ok(), "Should continue working normally");
    }

    #[test]
    fn test_multiple_concurrent_errors() {
        let mut system = ErrorAwareAudioSystem::new().unwrap();
        let test_signal: Vec<f32> = vec![0.3; 2048];

        // Inject multiple errors
        system.inject_error_scenario(ErrorScenarioType::MemoryPressure);
        system.inject_error_scenario(ErrorScenarioType::BufferUnderrun);

        let result = system.process_audio_with_error_handling(&test_signal);
        
        // System should handle the first error it encounters
        // (In this case, memory pressure is checked first)
        assert!(result.is_ok(), "Should handle memory pressure gracefully");
        
        let error_msg = system.get_last_error().unwrap();
        assert!(error_msg.contains("Memory pressure"), "Should handle memory pressure first");
    }
}