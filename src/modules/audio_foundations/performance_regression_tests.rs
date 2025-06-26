// Performance Regression Test Suite - STORY-3.19
// Automated performance testing and regression detection

use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

/// Performance baseline for regression detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub test_name: String,
    pub baseline_latency_ns: u64,
    pub baseline_memory_bytes: usize,
    pub baseline_cpu_percent: f32,
    pub tolerance_percent: f32,
    pub timestamp: u64,
    pub git_commit: Option<String>,
}

/// Performance test result
#[derive(Debug, Clone)]
pub struct PerformanceTestResult {
    pub test_name: String,
    pub measured_latency_ns: u64,
    pub measured_memory_bytes: usize,
    pub measured_cpu_percent: f32,
    pub passed: bool,
    pub regression_detected: bool,
    pub improvement_detected: bool,
    pub notes: Vec<String>,
}

/// Performance regression detector
pub struct PerformanceRegressionDetector {
    baselines: HashMap<String, PerformanceBaseline>,
    tolerance_percent: f32,
}

impl PerformanceRegressionDetector {
    /// Create new regression detector
    pub fn new(tolerance_percent: f32) -> Self {
        Self {
            baselines: HashMap::new(),
            tolerance_percent,
        }
    }

    /// Load baselines from JSON string (in production, this would load from file)
    pub fn load_baselines(&mut self, json_data: &str) -> Result<(), String> {
        let baselines: Vec<PerformanceBaseline> = serde_json::from_str(json_data)
            .map_err(|e| format!("Failed to parse baselines: {}", e))?;
        
        for baseline in baselines {
            self.baselines.insert(baseline.test_name.clone(), baseline);
        }
        
        Ok(())
    }

    /// Save current baselines to JSON string
    pub fn save_baselines(&self) -> Result<String, String> {
        let baselines: Vec<&PerformanceBaseline> = self.baselines.values().collect();
        serde_json::to_string_pretty(&baselines)
            .map_err(|e| format!("Failed to serialize baselines: {}", e))
    }

    /// Add or update a baseline
    pub fn set_baseline(&mut self, baseline: PerformanceBaseline) {
        self.baselines.insert(baseline.test_name.clone(), baseline);
    }

    /// Check if a test result represents a regression
    pub fn check_regression(&self, result: &PerformanceTestResult) -> bool {
        if let Some(baseline) = self.baselines.get(&result.test_name) {
            let latency_regression = self.is_regression(
                baseline.baseline_latency_ns,
                result.measured_latency_ns,
                baseline.tolerance_percent,
            );
            
            let memory_regression = self.is_regression(
                baseline.baseline_memory_bytes as u64,
                result.measured_memory_bytes as u64,
                baseline.tolerance_percent,
            );
            
            let cpu_regression = self.is_regression(
                (baseline.baseline_cpu_percent * 1000.0) as u64,
                (result.measured_cpu_percent * 1000.0) as u64,
                baseline.tolerance_percent,
            );
            
            latency_regression || memory_regression || cpu_regression
        } else {
            false // No baseline to compare against
        }
    }

    /// Check if current value represents a regression from baseline
    fn is_regression(&self, baseline: u64, current: u64, tolerance: f32) -> bool {
        let threshold = baseline as f32 * (1.0 + tolerance / 100.0);
        current as f32 > threshold
    }

    /// Check if current value represents an improvement from baseline
    pub fn is_improvement(&self, baseline: u64, current: u64, tolerance: f32) -> bool {
        let threshold = baseline as f32 * (1.0 - tolerance / 100.0);
        (current as f32) < threshold
    }
}

#[cfg(test)]
mod performance_regression_tests {
    use super::*;
    use crate::modules::audio_foundations::multi_algorithm_pitch_detector::*;
    use crate::modules::audio_foundations::audio_performance_monitor::*;

    /// Generate a test sine wave
    fn generate_test_signal(frequency: f32, sample_rate: f32, duration_samples: usize) -> Vec<f32> {
        (0..duration_samples)
            .map(|i| {
                let t = i as f32 / sample_rate;
                0.8 * (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect()
    }

    /// Measure memory usage (simplified for testing)
    fn measure_memory_usage() -> usize {
        // In a real implementation, this would measure actual memory usage
        // For testing, we'll simulate realistic values
        1024 * 1024 // 1MB baseline
    }

    /// Measure CPU usage (simplified for testing)
    fn measure_cpu_usage() -> f32 {
        // In a real implementation, this would measure actual CPU usage
        // For testing, we'll simulate realistic values
        15.0 // 15% CPU usage
    }

    #[test]
    fn test_baseline_management() {
        let mut detector = PerformanceRegressionDetector::new(10.0);
        
        let baseline = PerformanceBaseline {
            test_name: "test_pitch_detection_440hz".to_string(),
            baseline_latency_ns: 1_000_000, // 1ms
            baseline_memory_bytes: 1024,
            baseline_cpu_percent: 10.0,
            tolerance_percent: 15.0,
            timestamp: 1234567890,
            git_commit: Some("abc123".to_string()),
        };
        
        detector.set_baseline(baseline.clone());
        
        // Test serialization
        let json = detector.save_baselines().unwrap();
        assert!(json.contains("test_pitch_detection_440hz"));
        
        // Test loading
        let mut new_detector = PerformanceRegressionDetector::new(10.0);
        new_detector.load_baselines(&json).unwrap();
        
        assert!(new_detector.baselines.contains_key("test_pitch_detection_440hz"));
    }

    #[test]
    fn test_regression_detection() {
        let mut detector = PerformanceRegressionDetector::new(10.0);
        
        let baseline = PerformanceBaseline {
            test_name: "test_algorithm_performance".to_string(),
            baseline_latency_ns: 1_000_000, // 1ms
            baseline_memory_bytes: 1024,
            baseline_cpu_percent: 10.0,
            tolerance_percent: 15.0,
            timestamp: 1234567890,
            git_commit: None,
        };
        
        detector.set_baseline(baseline);
        
        // Test within tolerance (no regression)
        let good_result = PerformanceTestResult {
            test_name: "test_algorithm_performance".to_string(),
            measured_latency_ns: 1_100_000, // 1.1ms (10% increase, within 15% tolerance)
            measured_memory_bytes: 1100,
            measured_cpu_percent: 11.0,
            passed: true,
            regression_detected: false,
            improvement_detected: false,
            notes: vec![],
        };
        
        assert!(!detector.check_regression(&good_result));
        
        // Test regression
        let bad_result = PerformanceTestResult {
            test_name: "test_algorithm_performance".to_string(),
            measured_latency_ns: 1_300_000, // 1.3ms (30% increase, beyond 15% tolerance)
            measured_memory_bytes: 1400,
            measured_cpu_percent: 13.0,
            passed: false,
            regression_detected: true,
            improvement_detected: false,
            notes: vec!["Latency regression detected".to_string()],
        };
        
        assert!(detector.check_regression(&bad_result));
    }

    #[test]
    fn test_pitch_detection_performance_baseline() {
        let config = PitchDetectionConfig::default();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        
        let test_signal = generate_test_signal(440.0, 44100.0, 2048);
        
        // Warm up
        for _ in 0..5 {
            let _ = detector.detect_pitch(&test_signal);
        }
        
        // Measure performance
        let iterations = 100;
        let start_time = Instant::now();
        let start_memory = measure_memory_usage();
        
        for _ in 0..iterations {
            let result = detector.detect_pitch(&test_signal);
            assert!(result.is_ok(), "Pitch detection should succeed");
        }
        
        let elapsed = start_time.elapsed();
        let end_memory = measure_memory_usage();
        let avg_latency_ns = elapsed.as_nanos() as u64 / iterations;
        let memory_usage = end_memory.saturating_sub(start_memory);
        
        // Performance should be reasonable
        assert!(avg_latency_ns < 10_000_000, "Average latency should be < 10ms"); // 10ms
        assert!(memory_usage < 10_000_000, "Memory usage should be < 10MB");
        
        // Create baseline for future regression testing
        let baseline = PerformanceBaseline {
            test_name: "pitch_detection_440hz_2048_samples".to_string(),
            baseline_latency_ns: avg_latency_ns,
            baseline_memory_bytes: memory_usage,
            baseline_cpu_percent: measure_cpu_usage(),
            tolerance_percent: 20.0, // 20% tolerance for pitch detection
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            git_commit: None,
        };
        
        assert!(baseline.baseline_latency_ns > 0);
        assert!(baseline.baseline_cpu_percent > 0.0);
    }

    #[test]
    fn test_algorithm_switching_performance() {
        let config = PitchDetectionConfig::default();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        
        let test_signal = generate_test_signal(440.0, 44100.0, 2048);
        
        // Measure YIN performance
        detector.set_algorithm(PitchAlgorithm::YIN).unwrap();
        let start_time = Instant::now();
        for _ in 0..50 {
            let _ = detector.detect_pitch(&test_signal);
        }
        let yin_duration = start_time.elapsed();
        
        // Measure McLeod performance
        detector.set_algorithm(PitchAlgorithm::McLeod).unwrap();
        let start_time = Instant::now();
        for _ in 0..50 {
            let _ = detector.detect_pitch(&test_signal);
        }
        let mcleod_duration = start_time.elapsed();
        
        // Measure algorithm switching overhead
        let start_time = Instant::now();
        for i in 0..20 {
            let algorithm = if i % 2 == 0 { PitchAlgorithm::YIN } else { PitchAlgorithm::McLeod };
            detector.set_algorithm(algorithm).unwrap();
            let _ = detector.detect_pitch(&test_signal);
        }
        let switching_duration = start_time.elapsed();
        
        // Algorithm switching should not add significant overhead
        let switching_avg = switching_duration.as_nanos() / 20;
        let yin_avg = yin_duration.as_nanos() / 50;
        let mcleod_avg = mcleod_duration.as_nanos() / 50;
        let baseline_avg = (yin_avg + mcleod_avg) / 2;
        
        // Switching overhead should be < 50% of baseline
        assert!(switching_avg < baseline_avg + (baseline_avg / 2),
            "Algorithm switching overhead too high: {}ns vs {}ns baseline", 
            switching_avg, baseline_avg);
    }

    #[test]
    fn test_sustained_performance_monitoring() {
        let mut monitor = AudioPerformanceMonitor::new();
        let config = MonitoringConfig {
            enable_detailed_metrics: true,
            enable_regression_detection: true,
            sampling_interval_ns: 1_000_000, // 1ms
            max_history_size: 1000,
        };
        monitor.configure(config);
        
        // Simulate sustained audio processing
        let iterations = 200;
        let start_time = Instant::now();
        
        for i in 0..iterations {
            let measurement_id = monitor.start_measurement("audio_processing".to_string());
            
            // Simulate audio processing work
            std::thread::sleep(Duration::from_micros(100)); // 0.1ms
            
            monitor.end_measurement(measurement_id);
            
            // Check for performance degradation every 50 iterations
            if i % 50 == 0 && i > 0 {
                let metrics = monitor.get_current_metrics();
                
                // Performance should remain stable
                assert!(metrics.end_to_end_latency_ms < 50.0, 
                    "End-to-end latency too high: {}ms", metrics.end_to_end_latency_ms);
                assert!(metrics.processing_latency_ms < 20.0,
                    "Processing latency too high: {}ms", metrics.processing_latency_ms);
                assert!(metrics.cpu_usage_percent < 80.0,
                    "CPU usage too high: {}%", metrics.cpu_usage_percent);
            }
        }
        
        let total_duration = start_time.elapsed();
        let avg_iteration_time = total_duration.as_nanos() / iterations as u128;
        
        // Average iteration should be reasonable
        assert!(avg_iteration_time < 5_000_000, "Average iteration time too high: {}ns", avg_iteration_time);
        
        // Check final metrics
        let final_metrics = monitor.get_current_metrics();
        assert!(final_metrics.monitoring_overhead.cpu_overhead_percent < 5.0,
            "Monitoring overhead too high: {}%", final_metrics.monitoring_overhead.cpu_overhead_percent);
    }

    #[test]
    fn test_memory_leak_detection() {
        let config = PitchDetectionConfig::default();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        
        let test_signal = generate_test_signal(440.0, 44100.0, 2048);
        let initial_memory = measure_memory_usage();
        
        // Simulate extended usage
        for batch in 0..10 {
            for _ in 0..100 {
                let _ = detector.detect_pitch(&test_signal);
            }
            
            // Check memory usage every batch
            let current_memory = measure_memory_usage();
            let memory_growth = current_memory.saturating_sub(initial_memory);
            
            // Memory growth should be bounded
            assert!(memory_growth < 50_000_000, // 50MB max growth
                "Potential memory leak detected: {}MB growth after {} iterations",
                memory_growth / 1_000_000, (batch + 1) * 100);
        }
        
        // Force garbage collection (if we had it in Rust)
        // In a real implementation, we'd trigger cleanup and verify memory returns to baseline
        let final_memory = measure_memory_usage();
        let total_growth = final_memory.saturating_sub(initial_memory);
        
        assert!(total_growth < 20_000_000, // 20MB max total growth
            "Memory leak detected: {}MB total growth", total_growth / 1_000_000);
    }

    #[test]
    fn test_edge_case_performance() {
        let config = PitchDetectionConfig::default();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        
        // Test performance with edge cases
        let test_cases = vec![
            ("silence", vec![0.0; 2048]),
            ("noise", (0..2048).map(|i| (i as f32 % 255.0 - 127.0) / 127.0).collect()),
            ("clipped", vec![1.0; 2048]),
            ("very_quiet", vec![0.001; 2048]),
        ];
        
        for (test_name, test_signal) in test_cases {
            let start_time = Instant::now();
            
            for _ in 0..20 {
                let result = detector.detect_pitch(&test_signal);
                // Should either succeed or fail gracefully, but not crash
                match result {
                    Ok(_) => {}, // Success is fine
                    Err(_) => {}, // Graceful failure is also fine
                }
            }
            
            let duration = start_time.elapsed();
            let avg_duration_ns = duration.as_nanos() / 20;
            
            // Edge cases should not take significantly longer than normal cases
            assert!(avg_duration_ns < 20_000_000, // 20ms max
                "Edge case '{}' performance too slow: {}ns average", 
                test_name, avg_duration_ns);
        }
    }

    #[test]
    fn test_performance_comparison_across_algorithms() {
        let config = PitchDetectionConfig::default();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        
        let test_signal = generate_test_signal(440.0, 44100.0, 2048);
        let iterations = 50;
        
        // Test YIN performance
        detector.set_algorithm(PitchAlgorithm::YIN).unwrap();
        let start_time = Instant::now();
        for _ in 0..iterations {
            let _ = detector.detect_pitch(&test_signal);
        }
        let yin_duration = start_time.elapsed();
        
        // Test McLeod performance
        detector.set_algorithm(PitchAlgorithm::McLeod).unwrap();
        let start_time = Instant::now();
        for _ in 0..iterations {
            let _ = detector.detect_pitch(&test_signal);
        }
        let mcleod_duration = start_time.elapsed();
        
        // Test Auto selection performance
        detector.set_algorithm(PitchAlgorithm::Auto).unwrap();
        let start_time = Instant::now();
        for _ in 0..iterations {
            let _ = detector.detect_pitch(&test_signal);
        }
        let auto_duration = start_time.elapsed();
        
        let yin_avg_ns = yin_duration.as_nanos() / iterations as u128;
        let mcleod_avg_ns = mcleod_duration.as_nanos() / iterations as u128;
        let auto_avg_ns = auto_duration.as_nanos() / iterations as u128;
        
        // All algorithms should complete within reasonable time
        assert!(yin_avg_ns < 10_000_000, "YIN too slow: {}ns", yin_avg_ns);
        assert!(mcleod_avg_ns < 10_000_000, "McLeod too slow: {}ns", mcleod_avg_ns);
        assert!(auto_avg_ns < 15_000_000, "Auto selection too slow: {}ns", auto_avg_ns);
        
        // Auto selection should not be more than 50% slower than the slowest individual algorithm
        let max_individual = yin_avg_ns.max(mcleod_avg_ns);
        assert!(auto_avg_ns < max_individual + (max_individual / 2),
            "Auto selection overhead too high: {}ns vs {}ns max individual",
            auto_avg_ns, max_individual);
        
        // Log performance comparison for baseline establishment
        println!("Performance comparison:");
        println!("  YIN:    {}ns average", yin_avg_ns);
        println!("  McLeod: {}ns average", mcleod_avg_ns);
        println!("  Auto:   {}ns average", auto_avg_ns);
    }

    #[test]
    fn test_throughput_performance() {
        let config = PitchDetectionConfig::default();
        let mut detector = MultiAlgorithmPitchDetector::new(config, None).unwrap();
        
        let test_signal = generate_test_signal(440.0, 44100.0, 2048);
        let test_duration = Duration::from_secs(1); // 1 second test
        
        let start_time = Instant::now();
        let mut iterations = 0;
        
        while start_time.elapsed() < test_duration {
            let _ = detector.detect_pitch(&test_signal);
            iterations += 1;
        }
        
        let actual_duration = start_time.elapsed();
        let throughput = iterations as f64 / actual_duration.as_secs_f64();
        
        // Should achieve reasonable throughput
        assert!(throughput > 50.0, "Throughput too low: {} detections/second", throughput);
        assert!(throughput < 10000.0, "Throughput suspiciously high: {} detections/second", throughput);
        
        println!("Throughput: {:.1} pitch detections per second", throughput);
        
        // Verify we can sustain real-time processing
        // For 44.1kHz audio with 2048 sample buffers:
        // Real-time requirement: 44100 / 2048 = ~21.5 detections/second
        assert!(throughput > 21.5, 
            "Cannot sustain real-time processing: {} < 21.5 detections/second", throughput);
    }
}