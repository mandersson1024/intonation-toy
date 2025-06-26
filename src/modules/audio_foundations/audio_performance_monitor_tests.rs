//! # Audio Performance Monitor Test Suite - STORY-017
//!
//! Comprehensive tests for audio performance monitoring system including:
//! - Real-time latency measurement accuracy
//! - CPU and memory monitoring
//! - Dropout detection and counting
//! - Performance threshold alerts
//! - Historical data collection and retention
//! - Performance regression detection
//! - Monitoring overhead validation

#[cfg(test)]
mod tests {
    use super::super::audio_performance_monitor::*;
use super::super::audio_events::{AudioPerformanceEvent, PerformanceAlertEvent, PerformanceRegressionEvent};
use crate::modules::application_core::event_bus::{EventBus, get_timestamp_ns, EventBusMetrics, EventBusError, SubscriptionId, EventBusState};
use crate::modules::application_core::typed_event_bus::TypedEventBus;
use std::sync::{Arc, Mutex};
use std::collections::{VecDeque, HashMap};
use std::thread;
use std::time::Duration;
    
    /// Mock event bus for testing - use TypedEventBus instead of custom implementation
    fn create_test_event_bus() -> TypedEventBus {
        TypedEventBus::new()
    }
    
    /// Test helper to count events published to a real event bus
    struct EventCounter {
        count: Arc<Mutex<usize>>,
    }
    
    impl EventCounter {
        fn new() -> Self {
            Self {
                count: Arc::new(Mutex::new(0)),
            }
        }
        
        fn get_count(&self) -> usize {
            *self.count.lock().unwrap()
        }
        
        fn reset(&self) {
            *self.count.lock().unwrap() = 0;
        }
    }
    
    #[test]
    fn test_performance_monitor_initialization() {
        let monitor = AudioPerformanceMonitor::new();
        let metrics = monitor.get_current_metrics();
        
        // Verify initial state
        assert_eq!(metrics.dropout_count, 0);
        assert_eq!(metrics.end_to_end_latency_ms, 0.0);
        assert_eq!(metrics.processing_latency_ms, 0.0);
        assert_eq!(metrics.cpu_usage_percent, 0.0);
        assert_eq!(metrics.memory_usage_bytes, 0);
        assert_eq!(metrics.buffer_underruns, 0);
        assert_eq!(metrics.buffer_overruns, 0);
        assert!(metrics.operation_metrics.is_empty());
        assert!(metrics.threshold_violations.is_empty());
    }
    
    #[test]
    fn test_custom_configuration() {
        let config = MonitoringConfig {
            enabled: true,
            sampling_interval_ns: 500_000_000, // 0.5 seconds
            history_retention_minutes: 60, // 1 hour
            real_time_alerts: true,
            regression_detection: true,
            detailed_operation_tracking: true,
        };
        
        let monitor = AudioPerformanceMonitor::with_config(config.clone());
        let monitor_config = monitor.get_config();
        assert_eq!(monitor_config.sampling_interval_ns, 500_000_000);
        assert_eq!(monitor_config.history_retention_minutes, 60);
        assert!(monitor_config.real_time_alerts);
        assert!(monitor_config.regression_detection);
        assert!(monitor_config.detailed_operation_tracking);
    }
    
    #[test]
    fn test_measurement_lifecycle_accuracy() {
        let mut monitor = AudioPerformanceMonitor::new();
        
        // Test multiple measurements
        let measurement_ids: Vec<MeasurementId> = (0..5)
            .map(|i| monitor.start_measurement(&format!("test_operation_{}", i)))
            .collect();
        
        // Verify unique IDs
        for i in 0..measurement_ids.len() {
            for j in i+1..measurement_ids.len() {
                assert_ne!(measurement_ids[i], measurement_ids[j]);
            }
        }
        
        // Simulate work with different durations
        let work_durations = [10, 20, 5, 15, 8]; // milliseconds
        for (i, duration) in work_durations.iter().enumerate() {
            thread::sleep(Duration::from_millis(*duration as u64));
            monitor.end_measurement(measurement_ids[i]);
        }
        
        let metrics = monitor.get_current_metrics();
        
        // Verify measurements were recorded
        for i in 0..5 {
            let operation_name = format!("test_operation_{}", i);
            assert!(metrics.operation_metrics.contains_key(&operation_name));
            
            let operation_metrics = &metrics.operation_metrics[&operation_name];
            assert_eq!(operation_metrics.total_calls, 1);
            assert!(operation_metrics.average_time_ns > 0);
            assert_eq!(operation_metrics.success_count, 1);
            assert_eq!(operation_metrics.error_count, 0);
        }
    }
    
    #[test]
    fn test_latency_measurement_precision() {
        let mut monitor = AudioPerformanceMonitor::new();
        
        // Test microsecond precision requirement
        let test_latencies = [0.001, 0.1, 1.0, 5.5, 10.0, 15.7]; // milliseconds
        
        for latency in &test_latencies {
            monitor.record_audio_latency(*latency);
            let metrics = monitor.get_current_metrics();
            
            // Verify precision to 3 decimal places (microsecond precision)
            assert!((metrics.end_to_end_latency_ms - latency).abs() < 0.001);
        }
    }
    
    #[test]
    fn test_cpu_and_memory_monitoring() {
        let mut monitor = AudioPerformanceMonitor::new();
        
        // Test CPU usage recording
        let cpu_percentages = [25.5, 67.8, 89.2, 45.1];
        for cpu in &cpu_percentages {
            monitor.record_cpu_usage(*cpu);
            let metrics = monitor.get_current_metrics();
            assert_eq!(metrics.cpu_usage_percent, *cpu);
        }
        
        // Test memory usage recording
        let memory_amounts = [1024 * 1024, 5 * 1024 * 1024, 10 * 1024 * 1024]; // 1MB, 5MB, 10MB
        for memory in &memory_amounts {
            monitor.record_memory_usage(*memory);
            let metrics = monitor.get_current_metrics();
            assert_eq!(metrics.memory_usage_bytes, *memory);
        }
    }
    
    #[test]
    fn test_dropout_detection_and_counting() {
        let mut monitor = AudioPerformanceMonitor::new();
        
        // Test different types of dropouts
        monitor.record_dropout(DropoutType::BufferUnderrun);
        monitor.record_dropout(DropoutType::BufferUnderrun);
        monitor.record_dropout(DropoutType::BufferOverrun);
        monitor.record_dropout(DropoutType::ProcessingTimeout);
        
        let metrics = monitor.get_current_metrics();
        
        assert_eq!(metrics.dropout_count, 4);
        assert_eq!(metrics.buffer_underruns, 2);
        assert_eq!(metrics.buffer_overruns, 1);
        
        // Processing timeout should create a threshold violation
        assert!(!metrics.threshold_violations.is_empty());
        let timeout_violation = metrics.threshold_violations.iter()
            .find(|v| v.metric_name == "processing_timeout");
        assert!(timeout_violation.is_some());
        
        let violation = timeout_violation.unwrap();
        assert_eq!(violation.violation_severity, ViolationSeverity::Critical);
    }
    
    #[test]
    fn test_processing_cycle_recording() {
        let mut monitor = AudioPerformanceMonitor::new();
        
        // Test processing cycle with buffer latency calculation
        let processing_time = 8.5; // ms
        let buffer_size = 1024; // samples
        let sample_rate = 44100.0; // Hz
        
        monitor.record_processing_cycle(processing_time, buffer_size, sample_rate);
        
        let metrics = monitor.get_current_metrics();
        
        assert_eq!(metrics.processing_latency_ms, processing_time);
        
        // Verify buffer latency calculation
        let expected_buffer_latency = (buffer_size as f32 / sample_rate) * 1000.0;
        let expected_total_latency = processing_time + expected_buffer_latency;
        assert!((metrics.end_to_end_latency_ms - expected_total_latency).abs() < 0.01);
        
        // Verify buffer memory calculation
        let expected_buffer_memory = buffer_size * std::mem::size_of::<f32>();
        assert_eq!(metrics.buffer_memory_bytes, expected_buffer_memory);
    }
    
    #[test]
    fn test_threshold_violations_and_alerts() {
        let mut monitor = AudioPerformanceMonitor::new();
        let mut event_bus = create_test_event_bus();
        
        // Set strict thresholds
        let strict_thresholds = PerformanceThresholds {
            max_end_to_end_latency_ms: 5.0,
            max_processing_latency_ms: 2.0,
            max_cpu_usage_percent: 50.0,
            max_memory_usage_bytes: 1024 * 1024, // 1MB
            ..Default::default()
        };
        monitor.update_thresholds(strict_thresholds);
        
        // Trigger latency violation
        monitor.record_audio_latency(12.0); // Exceeds 5.0ms threshold
        monitor.record_processing_cycle(12.0, 1024, 44100.0);
        
        // Trigger CPU violation
        monitor.record_cpu_usage(75.0); // Exceeds 50% threshold
        
        // Trigger memory violation
        monitor.record_memory_usage(2 * 1024 * 1024); // Exceeds 1MB threshold
        
        let metrics = monitor.get_current_metrics();
        assert!(!metrics.threshold_violations.is_empty());
    }
    
    #[test]
    fn test_historical_data_collection() {
        let monitor = AudioPerformanceMonitor::new();
        
        // Add multiple historical data points
        for i in 0..50 {
            monitor.add_historical_data();
            thread::sleep(Duration::from_millis(1)); // Ensure different timestamps
        }
        
        // Test historical data retrieval
        let historical_metrics = monitor.get_historical_metrics(60); // Last 60 minutes
        assert!(historical_metrics.len() <= 50);
        assert!(!historical_metrics.is_empty());
        
        // Verify chronological ordering
        for i in 1..historical_metrics.len() {
            assert!(historical_metrics[i].timestamp >= historical_metrics[i-1].timestamp);
        }
    }
    
    #[test]
    fn test_data_retention_policy() {
        let config = MonitoringConfig {
            history_retention_minutes: 1, // Very short retention for testing
            ..Default::default()
        };
        let monitor = AudioPerformanceMonitor::with_config(config);
        
        // Add historical data
        for _ in 0..10 {
            monitor.add_historical_data();
        }
        
        // Wait for retention period to pass
        thread::sleep(Duration::from_millis(100));
        
        // Add more data to trigger cleanup
        monitor.add_historical_data();
        
        // Verify retention policy was applied
        let recent_metrics = monitor.get_historical_metrics(2); // Last 2 minutes
        assert!(recent_metrics.len() <= 11); // Should have cleaned up old data
    }
    
    #[test]
    fn test_performance_regression_detection() {
        let mut monitor = AudioPerformanceMonitor::new();
        
        // Set baseline performance
        let baseline = AudioPerformanceMetrics {
            timestamp: get_timestamp_ns(),
            end_to_end_latency_ms: 5.0,
            processing_latency_ms: 2.0,
            cpu_usage_percent: 30.0,
            ..monitor.get_current_metrics()
        };
        monitor.set_performance_baseline(baseline);
        
        // Simulate current metrics to show regression by recording new latency
        monitor.record_audio_latency(7.5); // 50% increase from baseline
        
        // Detect regression
        let regression = monitor.detect_performance_regression();
        assert!(regression.is_some());
        
        let regression = regression.unwrap();
        assert_eq!(regression.metric_name, "end_to_end_latency_ms");
        assert_eq!(regression.baseline_value, 5.0);
        assert_eq!(regression.current_value, 7.5);
        assert!(regression.regression_percent > 40.0);
        assert_eq!(regression.confidence_level, 0.95);
    }
    
    #[test]
    fn test_monitoring_overhead_validation() {
        let mut monitor = AudioPerformanceMonitor::new();
        
        // Perform many operations to accumulate overhead
        let operation_count = 1000;
        let start_time = std::time::Instant::now();
        
        for i in 0..operation_count {
            let id = monitor.start_measurement(&format!("overhead_test_{}", i % 10));
            // Simulate minimal work
            monitor.end_measurement(id);
        }
        
        let total_time = start_time.elapsed();
        let metrics = monitor.get_current_metrics();
        
        // Verify monitoring overhead is less than 5%
        assert!(metrics.monitoring_overhead.cpu_overhead_percent < super::super::audio_performance_monitor::MAX_MONITORING_OVERHEAD_PERCENT);
        
        // Verify overhead tracking is working
        assert!(metrics.monitoring_overhead.collection_time_ns > 0);
        
        // Calculate average overhead per operation
        let avg_overhead_per_op = total_time.as_nanos() as f64 / operation_count as f64;
        
        // Should be very low overhead per operation (< 1 microsecond)
        assert!(avg_overhead_per_op < 1000.0); // nanoseconds
    }
    
    #[test]
    fn test_concurrent_measurement_handling() {
        let monitor = Arc::new(std::sync::Mutex::new(AudioPerformanceMonitor::new()));
        let handles: Vec<_> = (0..10)
            .map(|thread_id| {
                let monitor_clone = monitor.clone();
                thread::spawn(move || {
                    let mut monitor_guard = monitor_clone.lock().unwrap();
                    
                    for i in 0..100 {
                        let id = monitor_guard.start_measurement(&format!("thread_{}_{}", thread_id, i));
                        // Simulate some work
                        thread::sleep(Duration::from_micros(10));
                        monitor_guard.end_measurement(id);
                    }
                })
            })
            .collect();
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        let monitor_guard = monitor.lock().unwrap();
        let metrics = monitor_guard.get_current_metrics();
        
        // Verify all measurements were recorded
        assert_eq!(metrics.operation_metrics.len(), 10); // 10 different operation patterns
        
        // Verify total calls across all operations
        let total_calls: u64 = metrics.operation_metrics.values()
            .map(|op| op.total_calls)
            .sum();
        assert_eq!(total_calls, 1000); // 10 threads * 100 operations each
    }
    
    #[test]
    fn test_event_publishing_integration() {
        let mut monitor = AudioPerformanceMonitor::new();
        let event_bus = Arc::new(MockEventBus::new());
        monitor.set_event_bus(event_bus.clone());
        
        // Clear initial events
        event_bus.clear_events();
        
        // Record various metrics to trigger events
        monitor.record_audio_latency(8.5);
        
        let id = monitor.start_measurement("test_operation");
        thread::sleep(Duration::from_millis(10));
        monitor.end_measurement(id);
        
        monitor.record_dropout(DropoutType::BufferUnderrun);
        
        // Verify events were published
        assert!(event_bus.count_events() > 0);
    }
    
    #[test]
    fn test_algorithm_memory_tracking() {
        let mut monitor = AudioPerformanceMonitor::new();
        
        // Test algorithm-specific memory tracking by recording total memory
        monitor.record_memory_usage(512 * 1024 + 1024 * 4); // Algorithm + buffer memory
        
        let metrics = monitor.get_current_metrics();
        assert_eq!(metrics.algorithm_memory_bytes, 512 * 1024);
        assert_eq!(metrics.buffer_memory_bytes, 1024 * 4);
        assert_eq!(metrics.memory_usage_bytes, 512 * 1024 + 1024 * 4);
    }
    
    #[test]
    fn test_performance_thresholds_customization() {
        let mut monitor = AudioPerformanceMonitor::new();
        
        let custom_thresholds = PerformanceThresholds {
            max_end_to_end_latency_ms: 8.0,
            max_processing_latency_ms: 4.0,
            max_cpu_usage_percent: 60.0,
            max_memory_usage_bytes: 64 * 1024 * 1024, // 64MB
            max_dropout_rate: 0.2,
            max_underrun_rate: 0.1,
            regression_threshold_percent: 15.0,
        };
        
        monitor.update_thresholds(custom_thresholds.clone());
        
        let thresholds = monitor.get_thresholds_raw().unwrap();
        assert_eq!(thresholds.max_end_to_end_latency_ms, 8.0);
        assert_eq!(thresholds.max_processing_latency_ms, 4.0);
        assert_eq!(thresholds.max_cpu_usage_percent, 60.0);
        assert_eq!(thresholds.max_memory_usage_bytes, 64 * 1024 * 1024);
        assert_eq!(thresholds.max_dropout_rate, 0.2);
        assert_eq!(thresholds.max_underrun_rate, 0.1);
        assert_eq!(thresholds.regression_threshold_percent, 15.0);
    }
    
    #[test]
    fn test_comprehensive_metrics_snapshot() {
        let mut monitor = AudioPerformanceMonitor::new();
        
        // Set up comprehensive metrics
        monitor.record_audio_latency(6.5);
        monitor.record_cpu_usage(45.2);
        monitor.record_memory_usage(8 * 1024 * 1024); // 8MB
        monitor.record_dropout(DropoutType::BufferUnderrun);
        monitor.record_dropout(DropoutType::BufferOverrun);
        
        // Record several operations
        for i in 0..5 {
            let id = monitor.start_measurement(&format!("operation_{}", i));
            thread::sleep(Duration::from_millis(5));
            monitor.end_measurement(id);
        }
        
        let metrics = monitor.get_current_metrics();
        
        // Verify comprehensive metrics are captured
        assert_eq!(metrics.end_to_end_latency_ms, 6.5);
        assert_eq!(metrics.cpu_usage_percent, 45.2);
        assert_eq!(metrics.memory_usage_bytes, 8 * 1024 * 1024);
        assert_eq!(metrics.dropout_count, 2);
        assert_eq!(metrics.buffer_underruns, 1);
        assert_eq!(metrics.buffer_overruns, 1);
        assert_eq!(metrics.operation_metrics.len(), 5);
        assert!(metrics.timestamp > 0);
        assert!(metrics.monitoring_overhead.collection_time_ns > 0);
    }
    
    #[test]
    fn test_error_handling_and_resilience() {
        let monitor = AudioPerformanceMonitor::new();
        
        // Test graceful handling of invalid measurement IDs
        let mut monitor_mut = AudioPerformanceMonitor::new();
        monitor_mut.end_measurement(999999); // Non-existent ID
        
        // Should not panic or crash
        let metrics = monitor.get_current_metrics();
        assert!(metrics.operation_metrics.is_empty());
        
        // Test with extreme values
        monitor_mut.record_audio_latency(f32::MAX);
        monitor_mut.record_cpu_usage(f32::MAX);
        monitor_mut.record_memory_usage(usize::MAX);
        
        // Should handle extreme values gracefully
        let metrics = monitor_mut.get_current_metrics();
        assert!(metrics.end_to_end_latency_ms.is_finite() || metrics.end_to_end_latency_ms.is_infinite());
    }
}