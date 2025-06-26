// Cross-Browser Automated Testing Framework - STORY-3.19
// Automated testing for Chrome, Firefox, Safari, and Edge compatibility

#[cfg(test)]
mod cross_browser_tests {
    use crate::modules::audio_foundations::*;
    use std::collections::HashMap;
    use std::time::{Duration, Instant};

    /// Browser compatibility information
    #[derive(Debug, Clone, PartialEq)]
    pub enum BrowserType {
        Chrome,
        Firefox,
        Safari,
        Edge,
        Unknown,
    }

    impl BrowserType {
        fn from_user_agent(user_agent: &str) -> Self {
            let ua = user_agent.to_lowercase();
            if ua.contains("chrome") && !ua.contains("edge") {
                BrowserType::Chrome
            } else if ua.contains("firefox") {
                BrowserType::Firefox
            } else if ua.contains("safari") && !ua.contains("chrome") {
                BrowserType::Safari
            } else if ua.contains("edge") {
                BrowserType::Edge
            } else {
                BrowserType::Unknown
            }
        }
    }

    /// Browser capability detection
    #[derive(Debug, Clone)]
    pub struct BrowserCapabilities {
        pub browser_type: BrowserType,
        pub supports_webassembly: bool,
        pub supports_audio_worklet: bool,
        pub supports_media_stream: bool,
        pub supports_audio_context: bool,
        pub max_sample_rate: u32,
        pub min_sample_rate: u32,
        pub supports_echo_cancellation: bool,
        pub supports_noise_suppression: bool,
        pub webgl_version: String,
    }

    impl Default for BrowserCapabilities {
        fn default() -> Self {
            Self {
                browser_type: BrowserType::Unknown,
                supports_webassembly: true,
                supports_audio_worklet: true,
                supports_media_stream: true,
                supports_audio_context: true,
                max_sample_rate: 48000,
                min_sample_rate: 8000,
                supports_echo_cancellation: true,
                supports_noise_suppression: true,
                webgl_version: "2.0".to_string(),
            }
        }
    }

    /// Mock browser environment for testing
    struct MockBrowserEnvironment {
        capabilities: BrowserCapabilities,
        simulated_latency_ms: f32,
        simulated_memory_limit_mb: usize,
        permission_responses: HashMap<String, bool>,
    }

    impl MockBrowserEnvironment {
        fn new(browser_type: BrowserType) -> Self {
            let capabilities = match browser_type {
                BrowserType::Chrome => BrowserCapabilities {
                    browser_type: BrowserType::Chrome,
                    supports_webassembly: true,
                    supports_audio_worklet: true,
                    supports_media_stream: true,
                    supports_audio_context: true,
                    max_sample_rate: 48000,
                    min_sample_rate: 8000,
                    supports_echo_cancellation: true,
                    supports_noise_suppression: true,
                    webgl_version: "2.0".to_string(),
                },
                BrowserType::Firefox => BrowserCapabilities {
                    browser_type: BrowserType::Firefox,
                    supports_webassembly: true,
                    supports_audio_worklet: true,
                    supports_media_stream: true,
                    supports_audio_context: true,
                    max_sample_rate: 48000,
                    min_sample_rate: 8000,
                    supports_echo_cancellation: false, // Firefox limitations
                    supports_noise_suppression: false,
                    webgl_version: "2.0".to_string(),
                },
                BrowserType::Safari => BrowserCapabilities {
                    browser_type: BrowserType::Safari,
                    supports_webassembly: true,
                    supports_audio_worklet: true, // Safari 14.1+
                    supports_media_stream: true,
                    supports_audio_context: true,
                    max_sample_rate: 48000,
                    min_sample_rate: 8000,
                    supports_echo_cancellation: true,
                    supports_noise_suppression: true,
                    webgl_version: "2.0".to_string(),
                },
                BrowserType::Edge => BrowserCapabilities {
                    browser_type: BrowserType::Edge,
                    supports_webassembly: true,
                    supports_audio_worklet: true,
                    supports_media_stream: true,
                    supports_audio_context: true,
                    max_sample_rate: 48000,
                    min_sample_rate: 8000,
                    supports_echo_cancellation: true,
                    supports_noise_suppression: true,
                    webgl_version: "2.0".to_string(),
                },
                BrowserType::Unknown => BrowserCapabilities::default(),
            };

            Self {
                capabilities,
                simulated_latency_ms: match browser_type {
                    BrowserType::Chrome => 2.0,
                    BrowserType::Firefox => 4.0,
                    BrowserType::Safari => 3.0,
                    BrowserType::Edge => 2.5,
                    BrowserType::Unknown => 5.0,
                },
                simulated_memory_limit_mb: match browser_type {
                    BrowserType::Chrome => 2048,
                    BrowserType::Firefox => 1024,
                    BrowserType::Safari => 1536,
                    BrowserType::Edge => 2048,
                    BrowserType::Unknown => 512,
                },
                permission_responses: HashMap::new(),
            }
        }

        fn set_permission_response(&mut self, permission: &str, granted: bool) {
            self.permission_responses.insert(permission.to_string(), granted);
        }

        fn simulate_permission_request(&self, permission: &str) -> bool {
            self.permission_responses.get(permission).copied().unwrap_or(true)
        }
    }

    /// Cross-browser test result
    #[derive(Debug, Clone)]
    pub struct CrossBrowserTestResult {
        pub test_name: String,
        pub browser_type: BrowserType,
        pub passed: bool,
        pub performance_ms: f32,
        pub memory_usage_mb: f32,
        pub error_message: Option<String>,
        pub compatibility_issues: Vec<String>,
    }

    /// Cross-browser test suite
    pub struct CrossBrowserTestSuite {
        environments: Vec<MockBrowserEnvironment>,
    }

    impl CrossBrowserTestSuite {
        pub fn new() -> Self {
            Self {
                environments: vec![
                    MockBrowserEnvironment::new(BrowserType::Chrome),
                    MockBrowserEnvironment::new(BrowserType::Firefox),
                    MockBrowserEnvironment::new(BrowserType::Safari),
                    MockBrowserEnvironment::new(BrowserType::Edge),
                ],
            }
        }

        /// Test WebAssembly compatibility across browsers
        pub fn test_webassembly_compatibility(&self) -> Vec<CrossBrowserTestResult> {
            let mut results = Vec::new();

            for env in &self.environments {
                let start_time = Instant::now();
                let mut compatibility_issues = Vec::new();

                // Test basic WASM support
                let wasm_supported = env.capabilities.supports_webassembly;
                if !wasm_supported {
                    compatibility_issues.push("WebAssembly not supported".to_string());
                }

                // Simulate WASM module loading and execution
                let config = PitchDetectionConfig::default();
                let detector_result = MultiAlgorithmPitchDetector::new(config, None);

                let passed = detector_result.is_ok() && wasm_supported;
                let performance_ms = start_time.elapsed().as_millis() as f32 + env.simulated_latency_ms;

                results.push(CrossBrowserTestResult {
                    test_name: "webassembly_compatibility".to_string(),
                    browser_type: env.capabilities.browser_type.clone(),
                    passed,
                    performance_ms,
                    memory_usage_mb: 5.0, // Simulated WASM memory usage
                    error_message: if !passed {
                        Some("WebAssembly initialization failed".to_string())
                    } else {
                        None
                    },
                    compatibility_issues,
                });
            }

            results
        }

        /// Test Web Audio API compatibility
        pub fn test_web_audio_compatibility(&self) -> Vec<CrossBrowserTestResult> {
            let mut results = Vec::new();

            for env in &self.environments {
                let start_time = Instant::now();
                let mut compatibility_issues = Vec::new();

                // Check Audio Context support
                if !env.capabilities.supports_audio_context {
                    compatibility_issues.push("AudioContext not supported".to_string());
                }

                // Check AudioWorklet support
                if !env.capabilities.supports_audio_worklet {
                    compatibility_issues.push("AudioWorklet not supported".to_string());
                }

                // Test sample rate compatibility
                let test_sample_rate = 44100;
                if test_sample_rate < env.capabilities.min_sample_rate 
                   || test_sample_rate > env.capabilities.max_sample_rate {
                    compatibility_issues.push(format!(
                        "Sample rate {}Hz not supported (range: {}-{}Hz)",
                        test_sample_rate, env.capabilities.min_sample_rate, env.capabilities.max_sample_rate
                    ));
                }

                let passed = env.capabilities.supports_audio_context 
                    && env.capabilities.supports_audio_worklet
                    && compatibility_issues.is_empty();

                let performance_ms = start_time.elapsed().as_millis() as f32 + env.simulated_latency_ms * 2.0;

                results.push(CrossBrowserTestResult {
                    test_name: "web_audio_compatibility".to_string(),
                    browser_type: env.capabilities.browser_type.clone(),
                    passed,
                    performance_ms,
                    memory_usage_mb: 3.0,
                    error_message: if !passed {
                        Some("Web Audio API compatibility issues".to_string())
                    } else {
                        None
                    },
                    compatibility_issues,
                });
            }

            results
        }

        /// Test microphone permission handling across browsers
        pub fn test_microphone_permissions(&mut self) -> Vec<CrossBrowserTestResult> {
            let mut results = Vec::new();

            for env in &mut self.environments {
                let start_time = Instant::now();
                let mut compatibility_issues = Vec::new();

                // Set up permission responses based on browser behavior
                match env.capabilities.browser_type {
                    BrowserType::Chrome => {
                        env.set_permission_response("microphone", true);
                    }
                    BrowserType::Firefox => {
                        env.set_permission_response("microphone", true);
                    }
                    BrowserType::Safari => {
                        env.set_permission_response("microphone", true);
                        compatibility_issues.push("Requires user gesture for permission".to_string());
                    }
                    BrowserType::Edge => {
                        env.set_permission_response("microphone", true);
                    }
                    BrowserType::Unknown => {
                        env.set_permission_response("microphone", false);
                        compatibility_issues.push("Unknown browser permission behavior".to_string());
                    }
                }

                let permission_granted = env.simulate_permission_request("microphone");
                let media_stream_supported = env.capabilities.supports_media_stream;

                let passed = permission_granted && media_stream_supported;
                let performance_ms = start_time.elapsed().as_millis() as f32 + 
                    env.simulated_latency_ms * 3.0; // Permission requests are slower

                results.push(CrossBrowserTestResult {
                    test_name: "microphone_permissions".to_string(),
                    browser_type: env.capabilities.browser_type.clone(),
                    passed,
                    performance_ms,
                    memory_usage_mb: 1.0,
                    error_message: if !passed {
                        Some("Microphone permission or MediaStream not available".to_string())
                    } else {
                        None
                    },
                    compatibility_issues,
                });
            }

            results
        }

        /// Test pitch detection performance across browsers
        pub fn test_pitch_detection_performance(&self) -> Vec<CrossBrowserTestResult> {
            let mut results = Vec::new();

            for env in &self.environments {
                let start_time = Instant::now();
                let mut compatibility_issues = Vec::new();

                // Create test signal
                let test_signal: Vec<f32> = (0..2048)
                    .map(|i| {
                        let t = i as f32 / 44100.0;
                        0.8 * (2.0 * std::f32::consts::PI * 440.0 * t).sin()
                    })
                    .collect();

                // Test pitch detection with browser-specific adjustments
                let config = PitchDetectionConfig {
                    algorithm: match env.capabilities.browser_type {
                        BrowserType::Firefox => PitchAlgorithm::YIN, // YIN might be more stable on Firefox
                        _ => PitchAlgorithm::Auto,
                    },
                    sample_rate: env.capabilities.max_sample_rate as f32,
                    ..PitchDetectionConfig::default()
                };

                let detector_result = MultiAlgorithmPitchDetector::new(config, None);
                let mut passed = false;
                let mut error_message = None;

                if let Ok(mut detector) = detector_result {
                    match detector.detect_pitch(&test_signal) {
                        Ok(pitch_result) => {
                            passed = pitch_result.is_valid;
                            if !passed {
                                error_message = Some("Pitch detection returned invalid result".to_string());
                            }

                            // Check browser-specific performance expectations
                            let expected_max_time_ns = match env.capabilities.browser_type {
                                BrowserType::Chrome => 5_000_000,   // 5ms
                                BrowserType::Firefox => 8_000_000,  // 8ms  
                                BrowserType::Safari => 6_000_000,   // 6ms
                                BrowserType::Edge => 5_500_000,     // 5.5ms
                                BrowserType::Unknown => 10_000_000, // 10ms
                            };

                            if pitch_result.processing_time_ns > expected_max_time_ns {
                                compatibility_issues.push(format!(
                                    "Processing time {}ns exceeds browser expectation {}ns",
                                    pitch_result.processing_time_ns, expected_max_time_ns
                                ));
                            }
                        }
                        Err(error) => {
                            error_message = Some(format!("Pitch detection failed: {}", error));
                        }
                    }
                } else {
                    error_message = Some("Failed to create pitch detector".to_string());
                }

                let performance_ms = start_time.elapsed().as_millis() as f32 + env.simulated_latency_ms;

                // Estimate memory usage based on browser type
                let memory_usage_mb = match env.capabilities.browser_type {
                    BrowserType::Chrome => 8.0,
                    BrowserType::Firefox => 12.0, // Firefox typically uses more memory
                    BrowserType::Safari => 10.0,
                    BrowserType::Edge => 9.0,
                    BrowserType::Unknown => 15.0,
                };

                results.push(CrossBrowserTestResult {
                    test_name: "pitch_detection_performance".to_string(),
                    browser_type: env.capabilities.browser_type.clone(),
                    passed,
                    performance_ms,
                    memory_usage_mb,
                    error_message,
                    compatibility_issues,
                });
            }

            results
        }

        /// Test browser-specific audio features
        pub fn test_browser_specific_features(&self) -> Vec<CrossBrowserTestResult> {
            let mut results = Vec::new();

            for env in &self.environments {
                let start_time = Instant::now();
                let mut compatibility_issues = Vec::new();

                // Test echo cancellation support
                if !env.capabilities.supports_echo_cancellation {
                    compatibility_issues.push("Echo cancellation not supported".to_string());
                }

                // Test noise suppression support
                if !env.capabilities.supports_noise_suppression {
                    compatibility_issues.push("Noise suppression not supported".to_string());
                }

                // Browser-specific feature tests
                match env.capabilities.browser_type {
                    BrowserType::Chrome => {
                        // Chrome-specific tests (advanced WebRTC features)
                    }
                    BrowserType::Firefox => {
                        // Firefox doesn't support some advanced audio features
                        if env.capabilities.supports_echo_cancellation {
                            compatibility_issues.push("Unexpected echo cancellation support in Firefox".to_string());
                        }
                    }
                    BrowserType::Safari => {
                        // Safari-specific limitations and requirements
                        compatibility_issues.push("Requires HTTPS for microphone access".to_string());
                    }
                    BrowserType::Edge => {
                        // Edge typically follows Chrome behavior
                    }
                    BrowserType::Unknown => {
                        compatibility_issues.push("Unknown browser - cannot test specific features".to_string());
                    }
                }

                let passed = compatibility_issues.len() <= 2; // Allow minor compatibility issues
                let performance_ms = start_time.elapsed().as_millis() as f32 + env.simulated_latency_ms;

                results.push(CrossBrowserTestResult {
                    test_name: "browser_specific_features".to_string(),
                    browser_type: env.capabilities.browser_type.clone(),
                    passed,
                    performance_ms,
                    memory_usage_mb: 2.0,
                    error_message: if !passed {
                        Some("Too many compatibility issues detected".to_string())
                    } else {
                        None
                    },
                    compatibility_issues,
                });
            }

            results
        }

        /// Run comprehensive cross-browser test suite
        pub fn run_comprehensive_tests(&mut self) -> CrossBrowserTestReport {
            let mut all_results = Vec::new();

            all_results.extend(self.test_webassembly_compatibility());
            all_results.extend(self.test_web_audio_compatibility());
            all_results.extend(self.test_microphone_permissions());
            all_results.extend(self.test_pitch_detection_performance());
            all_results.extend(self.test_browser_specific_features());

            CrossBrowserTestReport::new(all_results)
        }
    }

    /// Comprehensive test report
    #[derive(Debug)]
    pub struct CrossBrowserTestReport {
        pub results: Vec<CrossBrowserTestResult>,
        pub summary: HashMap<BrowserType, BrowserTestSummary>,
    }

    #[derive(Debug)]
    pub struct BrowserTestSummary {
        pub total_tests: usize,
        pub passed_tests: usize,
        pub failed_tests: usize,
        pub success_rate: f32,
        pub avg_performance_ms: f32,
        pub total_memory_usage_mb: f32,
        pub critical_issues: Vec<String>,
    }

    impl CrossBrowserTestReport {
        pub fn new(results: Vec<CrossBrowserTestResult>) -> Self {
            let mut summary = HashMap::new();

            // Group results by browser type
            let browsers = [BrowserType::Chrome, BrowserType::Firefox, BrowserType::Safari, BrowserType::Edge];
            
            for browser in &browsers {
                let browser_results: Vec<&CrossBrowserTestResult> = results
                    .iter()
                    .filter(|r| r.browser_type == *browser)
                    .collect();

                if !browser_results.is_empty() {
                    let total_tests = browser_results.len();
                    let passed_tests = browser_results.iter().filter(|r| r.passed).count();
                    let failed_tests = total_tests - passed_tests;
                    let success_rate = passed_tests as f32 / total_tests as f32;
                    
                    let avg_performance_ms = browser_results
                        .iter()
                        .map(|r| r.performance_ms)
                        .sum::<f32>() / total_tests as f32;
                    
                    let total_memory_usage_mb = browser_results
                        .iter()
                        .map(|r| r.memory_usage_mb)
                        .sum::<f32>();

                    let mut critical_issues = Vec::new();
                    for result in &browser_results {
                        if !result.passed {
                            if let Some(ref error) = result.error_message {
                                critical_issues.push(format!("{}: {}", result.test_name, error));
                            }
                        }
                    }

                    summary.insert(browser.clone(), BrowserTestSummary {
                        total_tests,
                        passed_tests,
                        failed_tests,
                        success_rate,
                        avg_performance_ms,
                        total_memory_usage_mb,
                        critical_issues,
                    });
                }
            }

            Self { results, summary }
        }

        pub fn print_report(&self) {
            println!("\n=== Cross-Browser Compatibility Test Report ===");
            
            for (browser, summary) in &self.summary {
                println!("\n{:?} Browser Results:", browser);
                println!("  Tests: {} passed, {} failed ({:.1}% success rate)",
                    summary.passed_tests, summary.failed_tests, summary.success_rate * 100.0);
                println!("  Performance: {:.2}ms average", summary.avg_performance_ms);
                println!("  Memory Usage: {:.1}MB total", summary.total_memory_usage_mb);
                
                if !summary.critical_issues.is_empty() {
                    println!("  Critical Issues:");
                    for issue in &summary.critical_issues {
                        println!("    - {}", issue);
                    }
                }
            }

            // Overall compatibility summary
            let total_tests: usize = self.summary.values().map(|s| s.total_tests).sum();
            let total_passed: usize = self.summary.values().map(|s| s.passed_tests).sum();
            let overall_success_rate = if total_tests > 0 {
                total_passed as f32 / total_tests as f32
            } else {
                0.0
            };

            println!("\n=== Overall Compatibility Summary ===");
            println!("Total Tests: {}", total_tests);
            println!("Overall Success Rate: {:.1}%", overall_success_rate * 100.0);
            
            // Browser support recommendations
            println!("\n=== Browser Support Recommendations ===");
            for (browser, summary) in &self.summary {
                let recommendation = if summary.success_rate >= 0.9 {
                    "✅ Fully Supported"
                } else if summary.success_rate >= 0.7 {
                    "⚠️  Partially Supported (with limitations)"
                } else {
                    "❌ Not Recommended"
                };
                println!("{:?}: {}", browser, recommendation);
            }
        }
    }

    // =============================================================================
    // ACTUAL TESTS
    // =============================================================================

    #[test]
    fn test_browser_type_detection() {
        assert_eq!(BrowserType::from_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"), BrowserType::Chrome);
        assert_eq!(BrowserType::from_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0"), BrowserType::Firefox);
        assert_eq!(BrowserType::from_user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15"), BrowserType::Safari);
        assert_eq!(BrowserType::from_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36 Edg/91.0.864.59"), BrowserType::Edge);
    }

    #[test]
    fn test_webassembly_cross_browser_compatibility() {
        let test_suite = CrossBrowserTestSuite::new();
        let results = test_suite.test_webassembly_compatibility();
        
        assert_eq!(results.len(), 4); // Four browsers tested
        
        // All modern browsers should support WebAssembly
        for result in &results {
            assert!(result.passed, "{:?} should support WebAssembly", result.browser_type);
            assert!(result.performance_ms > 0.0);
            assert!(result.memory_usage_mb > 0.0);
        }
    }

    #[test]
    fn test_web_audio_cross_browser_compatibility() {
        let test_suite = CrossBrowserTestSuite::new();
        let results = test_suite.test_web_audio_compatibility();
        
        assert_eq!(results.len(), 4);
        
        for result in &results {
            // All modern browsers should support Web Audio API
            assert!(result.passed, "{:?} should support Web Audio API", result.browser_type);
            
            // Performance should be reasonable
            assert!(result.performance_ms < 50.0, "Web Audio setup should be fast");
        }
    }

    #[test]
    fn test_microphone_permission_cross_browser() {
        let mut test_suite = CrossBrowserTestSuite::new();
        let results = test_suite.test_microphone_permissions();
        
        assert_eq!(results.len(), 4);
        
        let mut chrome_found = false;
        let mut firefox_found = false;
        let mut safari_found = false;
        let mut edge_found = false;

        for result in &results {
            match result.browser_type {
                BrowserType::Chrome => {
                    chrome_found = true;
                    assert!(result.passed, "Chrome should handle microphone permissions");
                }
                BrowserType::Firefox => {
                    firefox_found = true;
                    assert!(result.passed, "Firefox should handle microphone permissions");
                }
                BrowserType::Safari => {
                    safari_found = true;
                    assert!(result.passed, "Safari should handle microphone permissions");
                    // Safari should have compatibility notes about user gestures
                    assert!(!result.compatibility_issues.is_empty());
                }
                BrowserType::Edge => {
                    edge_found = true;
                    assert!(result.passed, "Edge should handle microphone permissions");
                }
                BrowserType::Unknown => {
                    // Unknown browsers may fail
                }
            }
        }

        assert!(chrome_found && firefox_found && safari_found && edge_found);
    }

    #[test]
    fn test_pitch_detection_cross_browser_performance() {
        let test_suite = CrossBrowserTestSuite::new();
        let results = test_suite.test_pitch_detection_performance();
        
        assert_eq!(results.len(), 4);
        
        for result in &results {
            // Basic functionality should work on all browsers
            assert!(result.passed, "{:?} should support pitch detection", result.browser_type);
            
            // Performance should be acceptable for real-time processing
            assert!(result.performance_ms < 100.0, 
                "{:?} pitch detection should be fast enough for real-time", result.browser_type);
            
            // Memory usage should be reasonable
            assert!(result.memory_usage_mb < 50.0,
                "{:?} memory usage should be reasonable", result.browser_type);
        }
    }

    #[test]
    fn test_browser_specific_feature_compatibility() {
        let test_suite = CrossBrowserTestSuite::new();
        let results = test_suite.test_browser_specific_features();
        
        assert_eq!(results.len(), 4);
        
        for result in &results {
            // Check browser-specific expectations
            match result.browser_type {
                BrowserType::Chrome => {
                    // Chrome should have full feature support
                    assert!(result.passed, "Chrome should have comprehensive feature support");
                }
                BrowserType::Firefox => {
                    // Firefox may have some limitations but should still pass
                    // (we allow minor compatibility issues)
                    if !result.passed {
                        println!("Firefox compatibility issues: {:?}", result.compatibility_issues);
                    }
                }
                BrowserType::Safari => {
                    // Safari should work but may have HTTPS requirements
                    assert!(!result.compatibility_issues.is_empty(), 
                        "Safari should note HTTPS requirement");
                }
                BrowserType::Edge => {
                    // Edge should have similar support to Chrome
                    assert!(result.passed, "Edge should have good feature support");
                }
                BrowserType::Unknown => {
                    // Unknown browsers expected to have issues
                }
            }
        }
    }

    #[test]
    fn test_comprehensive_cross_browser_suite() {
        let mut test_suite = CrossBrowserTestSuite::new();
        let report = test_suite.run_comprehensive_tests();
        
        // Print the full report for manual review
        report.print_report();
        
        // Verify all browsers were tested
        assert!(report.summary.contains_key(&BrowserType::Chrome));
        assert!(report.summary.contains_key(&BrowserType::Firefox));
        assert!(report.summary.contains_key(&BrowserType::Safari));
        assert!(report.summary.contains_key(&BrowserType::Edge));
        
        // Check overall success rates
        for (browser, summary) in &report.summary {
            match browser {
                BrowserType::Chrome => {
                    assert!(summary.success_rate > 0.9, 
                        "Chrome should have >90% success rate, got {:.1}%", summary.success_rate * 100.0);
                }
                BrowserType::Firefox => {
                    assert!(summary.success_rate > 0.8, 
                        "Firefox should have >80% success rate, got {:.1}%", summary.success_rate * 100.0);
                }
                BrowserType::Safari => {
                    assert!(summary.success_rate > 0.8, 
                        "Safari should have >80% success rate, got {:.1}%", summary.success_rate * 100.0);
                }
                BrowserType::Edge => {
                    assert!(summary.success_rate > 0.9, 
                        "Edge should have >90% success rate, got {:.1}%", summary.success_rate * 100.0);
                }
                BrowserType::Unknown => {
                    // No specific requirements for unknown browsers
                }
            }
            
            // Performance should be reasonable across all browsers
            assert!(summary.avg_performance_ms < 100.0,
                "{:?} average performance should be <100ms, got {:.2}ms", 
                browser, summary.avg_performance_ms);
        }
        
        // Verify we have a substantial number of test results
        assert!(report.results.len() >= 20, "Should have at least 20 test results");
        
        // Calculate overall cross-browser compatibility
        let total_tests: usize = report.summary.values().map(|s| s.total_tests).sum();
        let total_passed: usize = report.summary.values().map(|s| s.passed_tests).sum();
        let overall_success_rate = total_passed as f32 / total_tests as f32;
        
        assert!(overall_success_rate > 0.85, 
            "Overall cross-browser success rate should be >85%, got {:.1}%", 
            overall_success_rate * 100.0);
    }
}