//! # Audio Performance Monitoring System - STORY-017
//!
//! Comprehensive performance monitoring for audio processing with:
//! - Real-time latency measurement (end-to-end, processing, context)
//! - CPU usage monitoring for audio processing threads
//! - Memory usage tracking for audio buffers and algorithms
//! - Dropout detection and counting
//! - Performance threshold alerts and warnings
//! - Historical performance data collection
//! - Performance regression detection system

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use crate::modules::application_core::event_bus::{Event, get_timestamp_ns};
use crate::modules::application_core::typed_event_bus::TypedEventBus;
use crate::modules::audio_foundations::audio_events::{
    AudioPerformanceEvent, PerformanceAlertEvent, PerformanceRegressionEvent
};

/// Maximum number of historical data points to retain
const MAX_HISTORY_POINTS: usize = 86400; // 24 hours at 1-second intervals

/// Default sampling interval (1 second in nanoseconds)
const DEFAULT_SAMPLING_INTERVAL_NS: u64 = 1_000_000_000;

/// Performance monitoring overhead target (<5%)
const MAX_MONITORING_OVERHEAD_PERCENT: f32 = 5.0;

/// Unique identifier for measurements
pub type MeasurementId = u64;

/// Core performance monitoring trait
pub trait PerformanceMonitor: Send + Sync {
    /// Start a performance measurement for a specific operation
    fn start_measurement(&mut self, operation: &str) -> MeasurementId;
    
    /// End a performance measurement
    fn end_measurement(&mut self, id: MeasurementId);
    
    /// Record audio latency measurement
    fn record_audio_latency(&mut self, latency_ms: f32);
    
    /// Record CPU usage percentage
    fn record_cpu_usage(&mut self, usage_percent: f32);
    
    /// Record memory usage in bytes
    fn record_memory_usage(&mut self, bytes: usize);
    
    /// Detect and record audio dropout
    fn detect_dropout(&mut self);
    
    /// Get current performance metrics snapshot
    fn get_current_metrics(&self) -> AudioPerformanceMetrics;
    
    /// Get historical performance data
    fn get_historical_metrics(&self, duration_minutes: u32) -> Vec<AudioPerformanceMetrics>;
    
    /// Check for performance regressions
    fn detect_performance_regression(&self) -> Option<PerformanceRegression>;
    
    /// Update performance thresholds
    fn update_thresholds(&mut self, thresholds: PerformanceThresholds);
}

/// Comprehensive audio performance metrics
#[derive(Debug, Clone)]
pub struct AudioPerformanceMetrics {
    /// Timestamp of measurement
    pub timestamp: u64,
    
    /// End-to-end audio latency (microphone to output)
    pub end_to_end_latency_ms: f32,
    
    /// Audio processing latency (algorithm execution time)
    pub processing_latency_ms: f32,
    
    /// Audio context latency (Web Audio API overhead)
    pub context_latency_ms: f32,
    
    /// CPU usage percentage for audio processing
    pub cpu_usage_percent: f32,
    
    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
    
    /// Audio buffer memory usage
    pub buffer_memory_bytes: usize,
    
    /// Algorithm-specific memory usage
    pub algorithm_memory_bytes: usize,
    
    /// Number of audio dropouts detected
    pub dropout_count: u32,
    
    /// Number of buffer underruns
    pub buffer_underruns: u32,
    
    /// Number of buffer overruns
    pub buffer_overruns: u32,
    
    /// Performance metrics by operation type
    pub operation_metrics: HashMap<String, OperationMetrics>,
    
    /// Performance threshold violations
    pub threshold_violations: Vec<ThresholdViolation>,
    
    /// Monitoring overhead statistics
    pub monitoring_overhead: MonitoringOverhead,
}

/// Performance metrics for specific operations
#[derive(Debug, Clone)]
pub struct OperationMetrics {
    pub operation_name: String,
    pub total_calls: u64,
    pub total_time_ns: u64,
    pub average_time_ns: u64,
    pub min_time_ns: u64,
    pub max_time_ns: u64,
    pub success_count: u64,
    pub error_count: u32,
}

/// Performance threshold violation
#[derive(Debug, Clone)]
pub struct ThresholdViolation {
    pub metric_name: String,
    pub threshold_value: f32,
    pub actual_value: f32,
    pub violation_severity: ViolationSeverity,
    pub timestamp: u64,
}

/// Severity levels for threshold violations
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ViolationSeverity {
    Warning,
    Alert,
    Critical,
}

/// Performance regression detection result
#[derive(Debug, Clone)]
pub struct PerformanceRegression {
    pub metric_name: String,
    pub baseline_value: f32,
    pub current_value: f32,
    pub regression_percent: f32,
    pub confidence_level: f32,
    pub detected_timestamp: u64,
}

/// Configurable performance thresholds
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    /// Maximum acceptable end-to-end latency (ms)
    pub max_end_to_end_latency_ms: f32,
    
    /// Maximum acceptable processing latency (ms)
    pub max_processing_latency_ms: f32,
    
    /// Maximum acceptable CPU usage (%)
    pub max_cpu_usage_percent: f32,
    
    /// Maximum acceptable memory usage (bytes)
    pub max_memory_usage_bytes: usize,
    
    /// Maximum acceptable dropout rate (dropouts/second)
    pub max_dropout_rate: f32,
    
    /// Maximum acceptable buffer underrun rate
    pub max_underrun_rate: f32,
    
    /// Performance regression threshold (%)
    pub regression_threshold_percent: f32,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_end_to_end_latency_ms: 10.0,    // <10ms requirement
            max_processing_latency_ms: 5.0,      // <5ms for processing
            max_cpu_usage_percent: 70.0,         // <70% CPU usage
            max_memory_usage_bytes: 50 * 1024 * 1024, // 50MB memory limit
            max_dropout_rate: 0.1,               // <0.1 dropouts/second
            max_underrun_rate: 0.05,             // <0.05 underruns/second  
            regression_threshold_percent: 10.0,  // 10% regression threshold
        }
    }
}

/// Monitoring overhead statistics
#[derive(Debug, Clone)]
pub struct MonitoringOverhead {
    pub collection_time_ns: u64,
    pub storage_overhead_bytes: usize,
    pub cpu_overhead_percent: f32,
    pub memory_overhead_bytes: usize,
}

/// Active performance measurement
#[derive(Debug)]
struct ActiveMeasurement {
    id: MeasurementId,
    operation: String,
    start_time: Instant,
    start_timestamp: u64,
}

/// Historical data point for trend analysis
#[derive(Debug, Clone)]
struct HistoricalDataPoint {
    timestamp: u64,
    metrics: AudioPerformanceMetrics,
}

/// Audio performance monitor implementation
pub struct AudioPerformanceMonitor {
    /// Current performance metrics
    current_metrics: Arc<RwLock<AudioPerformanceMetrics>>,
    
    /// Performance thresholds for alerting
    thresholds: Arc<RwLock<PerformanceThresholds>>,
    
    /// Historical performance data
    historical_data: Arc<RwLock<VecDeque<HistoricalDataPoint>>>,
    
    /// Active measurements by ID
    active_measurements: Arc<Mutex<HashMap<MeasurementId, ActiveMeasurement>>>,
    
    /// Next measurement ID
    next_measurement_id: Arc<Mutex<MeasurementId>>,
    
    /// Event bus for publishing alerts
    event_bus: Option<Arc<TypedEventBus>>,
    
    /// Monitoring configuration
    config: MonitoringConfig,
    
    /// Performance baseline for regression detection
    performance_baseline: Arc<RwLock<Option<AudioPerformanceMetrics>>>,
    
    /// Monitoring overhead tracking
    overhead_tracker: Arc<Mutex<OverheadTracker>>,
}

/// Monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub sampling_interval_ns: u64,
    pub history_retention_minutes: u32,
    pub real_time_alerts: bool,
    pub regression_detection: bool,
    pub detailed_operation_tracking: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling_interval_ns: DEFAULT_SAMPLING_INTERVAL_NS,
            history_retention_minutes: 1440, // 24 hours
            real_time_alerts: true,
            regression_detection: true,
            detailed_operation_tracking: true,
        }
    }
}

/// Overhead tracking for monitoring system
#[derive(Debug)]
struct OverheadTracker {
    monitoring_start_time: Option<Instant>,
    total_monitoring_time_ns: u64,
    total_processing_time_ns: u64,
    memory_baseline: usize,
}

impl AudioPerformanceMonitor {
    /// Create new performance monitor
    pub fn new() -> Self {
        Self::with_config(MonitoringConfig::default())
    }
    
    /// Create new performance monitor with custom configuration
    pub fn with_config(config: MonitoringConfig) -> Self {
        let current_time = get_timestamp_ns();
        
        let initial_metrics = AudioPerformanceMetrics {
            timestamp: current_time,
            end_to_end_latency_ms: 0.0,
            processing_latency_ms: 0.0,
            context_latency_ms: 0.0,
            cpu_usage_percent: 0.0,
            memory_usage_bytes: 0,
            buffer_memory_bytes: 0,
            algorithm_memory_bytes: 0,
            dropout_count: 0,
            buffer_underruns: 0,
            buffer_overruns: 0,
            operation_metrics: HashMap::new(),
            threshold_violations: Vec::new(),
            monitoring_overhead: MonitoringOverhead {
                collection_time_ns: 0,
                storage_overhead_bytes: 0,
                cpu_overhead_percent: 0.0,
                memory_overhead_bytes: 0,
            },
        };
        
        Self {
            current_metrics: Arc::new(RwLock::new(initial_metrics)),
            thresholds: Arc::new(RwLock::new(PerformanceThresholds::default())),
            historical_data: Arc::new(RwLock::new(VecDeque::new())),
            active_measurements: Arc::new(Mutex::new(HashMap::new())),
            next_measurement_id: Arc::new(Mutex::new(1)),
            event_bus: None,
            config,
            performance_baseline: Arc::new(RwLock::new(None)),
            overhead_tracker: Arc::new(Mutex::new(OverheadTracker {
                monitoring_start_time: None,
                total_monitoring_time_ns: 0,
                total_processing_time_ns: 0,
                memory_baseline: 0,
            })),
        }
    }
    
    /// Set event bus for publishing performance events
    pub fn set_event_bus(&mut self, event_bus: Arc<TypedEventBus>) {
        self.event_bus = Some(event_bus);
    }
    
    /// Get current monitoring configuration (for testing)
    #[cfg(test)]
    pub fn get_config(&self) -> &MonitoringConfig {
        &self.config
    }
    
    /// Get access to current metrics for testing
    #[cfg(test)]
    pub fn get_current_metrics_raw(&self) -> Result<AudioPerformanceMetrics, String> {
        self.current_metrics.read()
            .map(|metrics| metrics.clone())
            .map_err(|_| "Failed to read metrics".to_string())
    }
    
    /// Get access to thresholds for testing
    #[cfg(test)]
    pub fn get_thresholds_raw(&self) -> Result<PerformanceThresholds, String> {
        self.thresholds.read()
            .map(|thresholds| thresholds.clone())
            .map_err(|_| "Failed to read thresholds".to_string())
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: MonitoringConfig) {
        self.config = config;
    }
    
    /// Set performance baseline for regression detection
    pub fn set_performance_baseline(&self, baseline: AudioPerformanceMetrics) {
        if let Ok(mut baseline_guard) = self.performance_baseline.write() {
            *baseline_guard = Some(baseline);
        }
    }
    
    /// Record audio processing cycle completion
    pub fn record_processing_cycle(&mut self, 
        processing_time_ms: f32, 
        buffer_size: usize,
        sample_rate: f32) {
        
        let overhead_start = Instant::now();
        
        // Update processing latency
        if let Ok(mut metrics) = self.current_metrics.write() {
            metrics.processing_latency_ms = processing_time_ms;
            metrics.timestamp = get_timestamp_ns();
            
            // Calculate buffer contribution to latency
            let buffer_latency_ms = (buffer_size as f32 / sample_rate) * 1000.0;
            metrics.end_to_end_latency_ms = metrics.processing_latency_ms + 
                                          metrics.context_latency_ms + 
                                          buffer_latency_ms;
            
            // Update buffer memory usage
            metrics.buffer_memory_bytes = buffer_size * std::mem::size_of::<f32>();
        }
        
        // Record overhead
        let overhead_time = overhead_start.elapsed();
        self.record_monitoring_overhead(overhead_time);
        
        // Check thresholds and publish alerts if needed
        self.check_thresholds_and_alert();
    }
    
    /// Record audio dropout detection
    pub fn record_dropout(&mut self, dropout_type: DropoutType) {
        if let Ok(mut metrics) = self.current_metrics.write() {
            metrics.dropout_count += 1;
            
            match dropout_type {
                DropoutType::BufferUnderrun => metrics.buffer_underruns += 1,
                DropoutType::BufferOverrun => metrics.buffer_overruns += 1,
                DropoutType::ProcessingTimeout => {
                    // Create threshold violation for processing timeout
                    let violation = ThresholdViolation {
                        metric_name: "processing_timeout".to_string(),
                        threshold_value: 0.0,
                        actual_value: 1.0,
                        violation_severity: ViolationSeverity::Critical,
                        timestamp: get_timestamp_ns(),
                    };
                    metrics.threshold_violations.push(violation);
                }
            }
        }
        
        // Publish dropout event
        if let Some(ref event_bus) = self.event_bus {
            let event = AudioPerformanceEvent {
                metric_type: "dropout_detected".to_string(),
                value: 1.0,
                unit: "count".to_string(),
                timestamp: get_timestamp_ns(),
                operation_context: Some(format!("{:?}", dropout_type)),
            };
            let _ = event_bus.publish_high(event);
        }
    }
    
    /// Add historical data point and manage retention
    pub fn add_historical_data(&self) {
        if let (Ok(metrics), Ok(mut history)) = (
            self.current_metrics.read(),
            self.historical_data.write()
        ) {
            let data_point = HistoricalDataPoint {
                timestamp: metrics.timestamp,
                metrics: metrics.clone(),
            };
            
            history.push_back(data_point);
            
            // Apply retention policy
            let retention_cutoff = get_timestamp_ns() - 
                (self.config.history_retention_minutes as u64 * 60 * 1_000_000_000);
            
            while let Some(front) = history.front() {
                if front.timestamp < retention_cutoff {
                    history.pop_front();
                } else {
                    break;
                }
            }
            
            // Limit maximum history points
            while history.len() > MAX_HISTORY_POINTS {
                history.pop_front();
            }
        }
    }
    
    /// Check thresholds and generate alerts
    fn check_thresholds_and_alert(&self) {
        if !self.config.real_time_alerts {
            return;
        }
        
        if let (Ok(metrics), Ok(thresholds)) = (
            self.current_metrics.read(),
            self.thresholds.read()
        ) {
            let mut violations = Vec::new();
            
            // Check latency thresholds
            if metrics.end_to_end_latency_ms > thresholds.max_end_to_end_latency_ms {
                violations.push(ThresholdViolation {
                    metric_name: "end_to_end_latency_ms".to_string(),
                    threshold_value: thresholds.max_end_to_end_latency_ms,
                    actual_value: metrics.end_to_end_latency_ms,
                    violation_severity: if metrics.end_to_end_latency_ms > thresholds.max_end_to_end_latency_ms * 2.0 {
                        ViolationSeverity::Critical
                    } else {
                        ViolationSeverity::Alert
                    },
                    timestamp: get_timestamp_ns(),
                });
            }
            
            // Check CPU usage thresholds
            if metrics.cpu_usage_percent > thresholds.max_cpu_usage_percent {
                violations.push(ThresholdViolation {
                    metric_name: "cpu_usage_percent".to_string(),
                    threshold_value: thresholds.max_cpu_usage_percent,
                    actual_value: metrics.cpu_usage_percent,
                    violation_severity: if metrics.cpu_usage_percent > 90.0 {
                        ViolationSeverity::Critical
                    } else if metrics.cpu_usage_percent > 80.0 {
                        ViolationSeverity::Alert
                    } else {
                        ViolationSeverity::Warning
                    },
                    timestamp: get_timestamp_ns(),
                });
            }
            
            // Check memory usage thresholds
            if metrics.memory_usage_bytes > thresholds.max_memory_usage_bytes {
                violations.push(ThresholdViolation {
                    metric_name: "memory_usage_bytes".to_string(),
                    threshold_value: thresholds.max_memory_usage_bytes as f32,
                    actual_value: metrics.memory_usage_bytes as f32,
                    violation_severity: ViolationSeverity::Alert,
                    timestamp: get_timestamp_ns(),
                });
            }
            
            // Publish alerts for violations
            for violation in violations {
                if let Some(ref event_bus) = self.event_bus {
                    let alert_event = PerformanceAlertEvent {
                        alert_type: violation.metric_name.clone(),
                        severity: format!("{:?}", violation.violation_severity),
                        threshold_value: violation.threshold_value,
                        actual_value: violation.actual_value,
                        message: format!(
                            "Performance threshold exceeded: {} = {:.2} (threshold: {:.2})",
                            violation.metric_name, violation.actual_value, violation.threshold_value
                        ),
                        timestamp: violation.timestamp,
                        requires_attention: violation.violation_severity >= ViolationSeverity::Alert,
                    };
                    let _ = event_bus.publish_high(alert_event);
                }
            }
        }
    }
    
    /// Record monitoring overhead for self-monitoring
    fn record_monitoring_overhead(&self, overhead_time: Duration) {
        if let Ok(mut tracker) = self.overhead_tracker.lock() {
            let overhead_ns = overhead_time.as_nanos() as u64;
            tracker.total_monitoring_time_ns += overhead_ns;
            
            // Update overhead metrics
            if let Ok(mut metrics) = self.current_metrics.write() {
                let total_time = tracker.total_monitoring_time_ns + tracker.total_processing_time_ns;
                if total_time > 0 {
                    metrics.monitoring_overhead.cpu_overhead_percent = 
                        (tracker.total_monitoring_time_ns as f32 / total_time as f32) * 100.0;
                }
                metrics.monitoring_overhead.collection_time_ns = overhead_ns;
            }
        }
    }
}

/// Types of audio dropouts
#[derive(Debug, Clone)]
pub enum DropoutType {
    BufferUnderrun,
    BufferOverrun,
    ProcessingTimeout,
}

impl PerformanceMonitor for AudioPerformanceMonitor {
    fn start_measurement(&mut self, operation: &str) -> MeasurementId {
        let overhead_start = Instant::now();
        
        if let (Ok(mut measurements), Ok(mut id_counter)) = (
            self.active_measurements.lock(),
            self.next_measurement_id.lock()
        ) {
            let measurement_id = *id_counter;
            *id_counter += 1;
            
            let measurement = ActiveMeasurement {
                id: measurement_id,
                operation: operation.to_string(),
                start_time: Instant::now(),
                start_timestamp: get_timestamp_ns(),
            };
            
            measurements.insert(measurement_id, measurement);
            
            // Record overhead
            let overhead_time = overhead_start.elapsed();
            self.record_monitoring_overhead(overhead_time);
            
            measurement_id
        } else {
            0 // Error case
        }
    }
    
    fn end_measurement(&mut self, id: MeasurementId) {
        let overhead_start = Instant::now();
        let end_time = Instant::now();
        let end_timestamp = get_timestamp_ns();
        
        if let Ok(mut measurements) = self.active_measurements.lock() {
            if let Some(measurement) = measurements.remove(&id) {
                let duration = end_time.duration_since(measurement.start_time);
                let duration_ns = duration.as_nanos() as u64;
                
                // Update operation metrics
                if self.config.detailed_operation_tracking {
                    if let Ok(mut metrics) = self.current_metrics.write() {
                        let operation_metrics = metrics.operation_metrics
                            .entry(measurement.operation.clone())
                            .or_insert(OperationMetrics {
                                operation_name: measurement.operation.clone(),
                                total_calls: 0,
                                total_time_ns: 0,
                                average_time_ns: 0,
                                min_time_ns: u64::MAX,
                                max_time_ns: 0,
                                success_count: 0,
                                error_count: 0,
                            });
                        
                        operation_metrics.total_calls += 1;
                        operation_metrics.total_time_ns += duration_ns;
                        operation_metrics.average_time_ns = 
                            operation_metrics.total_time_ns / operation_metrics.total_calls;
                        operation_metrics.min_time_ns = 
                            operation_metrics.min_time_ns.min(duration_ns);
                        operation_metrics.max_time_ns = 
                            operation_metrics.max_time_ns.max(duration_ns);
                        operation_metrics.success_count += 1;
                    }
                }
                
                // Publish performance event
                if let Some(ref event_bus) = self.event_bus {
                    let event = AudioPerformanceEvent {
                        metric_type: format!("operation_{}", measurement.operation),
                        value: duration_ns as f32 / 1_000_000.0, // Convert to milliseconds
                        unit: "ms".to_string(),
                        timestamp: end_timestamp,
                        operation_context: Some(measurement.operation),
                    };
                    let _ = event_bus.publish_normal(event);
                }
            }
        }
        
        // Record overhead
        let overhead_time = overhead_start.elapsed();
        self.record_monitoring_overhead(overhead_time);
    }
    
    fn record_audio_latency(&mut self, latency_ms: f32) {
        if let Ok(mut metrics) = self.current_metrics.write() {
            metrics.end_to_end_latency_ms = latency_ms;
            metrics.timestamp = get_timestamp_ns();
        }
        
        // Publish latency event
        if let Some(ref event_bus) = self.event_bus {
            let event = AudioPerformanceEvent {
                metric_type: "audio_latency".to_string(),
                value: latency_ms,
                unit: "ms".to_string(),
                timestamp: get_timestamp_ns(),
                operation_context: None,
            };
            let _ = event_bus.publish_normal(event);
        }
    }
    
    fn record_cpu_usage(&mut self, usage_percent: f32) {
        if let Ok(mut metrics) = self.current_metrics.write() {
            metrics.cpu_usage_percent = usage_percent;
            metrics.timestamp = get_timestamp_ns();
        }
    }
    
    fn record_memory_usage(&mut self, bytes: usize) {
        if let Ok(mut metrics) = self.current_metrics.write() {
            metrics.memory_usage_bytes = bytes;
            metrics.timestamp = get_timestamp_ns();
        }
    }
    
    fn detect_dropout(&mut self) {
        self.record_dropout(DropoutType::ProcessingTimeout);
    }
    
    fn get_current_metrics(&self) -> AudioPerformanceMetrics {
        if let Ok(metrics) = self.current_metrics.read() {
            metrics.clone()
        } else {
            // Return default metrics if lock fails
            AudioPerformanceMetrics {
                timestamp: get_timestamp_ns(),
                end_to_end_latency_ms: 0.0,
                processing_latency_ms: 0.0,
                context_latency_ms: 0.0,
                cpu_usage_percent: 0.0,
                memory_usage_bytes: 0,
                buffer_memory_bytes: 0,
                algorithm_memory_bytes: 0,
                dropout_count: 0,
                buffer_underruns: 0,
                buffer_overruns: 0,
                operation_metrics: HashMap::new(),
                threshold_violations: Vec::new(),
                monitoring_overhead: MonitoringOverhead {
                    collection_time_ns: 0,
                    storage_overhead_bytes: 0,
                    cpu_overhead_percent: 0.0,
                    memory_overhead_bytes: 0,
                },
            }
        }
    }
    
    fn get_historical_metrics(&self, duration_minutes: u32) -> Vec<AudioPerformanceMetrics> {
        if let Ok(history) = self.historical_data.read() {
            let cutoff_time = get_timestamp_ns() - (duration_minutes as u64 * 60 * 1_000_000_000);
            
            history.iter()
                .filter(|data_point| data_point.timestamp >= cutoff_time)
                .map(|data_point| data_point.metrics.clone())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    fn detect_performance_regression(&self) -> Option<PerformanceRegression> {
        if !self.config.regression_detection {
            return None;
        }
        
        if let (Ok(current_metrics), Ok(baseline_guard)) = (
            self.current_metrics.read(),
            self.performance_baseline.read()
        ) {
            if let Some(ref baseline) = *baseline_guard {
                let thresholds = self.thresholds.read().ok()?;
                
                // Check latency regression
                let latency_regression = (current_metrics.end_to_end_latency_ms - 
                                        baseline.end_to_end_latency_ms) / 
                                       baseline.end_to_end_latency_ms * 100.0;
                
                if latency_regression > thresholds.regression_threshold_percent {
                    let regression = PerformanceRegression {
                        metric_name: "end_to_end_latency_ms".to_string(),
                        baseline_value: baseline.end_to_end_latency_ms,
                        current_value: current_metrics.end_to_end_latency_ms,
                        regression_percent: latency_regression,
                        confidence_level: 0.95, // High confidence for latency measurements
                        detected_timestamp: get_timestamp_ns(),
                    };
                    
                    // Publish regression event
                    if let Some(ref event_bus) = self.event_bus {
                        let regression_event = PerformanceRegressionEvent {
                            metric_name: regression.metric_name.clone(),
                            baseline_value: regression.baseline_value,
                            current_value: regression.current_value,
                            regression_percent: regression.regression_percent,
                            confidence_level: regression.confidence_level,
                            impact_assessment: if latency_regression > 50.0 {
                                "High impact - significant latency increase".to_string()
                            } else if latency_regression > 25.0 {
                                "Medium impact - noticeable latency increase".to_string()
                            } else {
                                "Low impact - minor latency increase".to_string()
                            },
                            timestamp: regression.detected_timestamp,
                        };
                        let _ = event_bus.publish_high(regression_event);
                    }
                    
                    return Some(regression);
                }
            }
        }
        
        None
    }
    
    fn update_thresholds(&mut self, thresholds: PerformanceThresholds) {
        if let Ok(mut current_thresholds) = self.thresholds.write() {
            *current_thresholds = thresholds;
        }
    }
}

impl Default for AudioPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_performance_monitor_creation() {
        let monitor = AudioPerformanceMonitor::new();
        let metrics = monitor.get_current_metrics();
        
        assert_eq!(metrics.dropout_count, 0);
        assert_eq!(metrics.end_to_end_latency_ms, 0.0);
        assert_eq!(metrics.cpu_usage_percent, 0.0);
    }
    
    #[test]
    fn test_measurement_lifecycle() {
        let mut monitor = AudioPerformanceMonitor::new();
        
        let measurement_id = monitor.start_measurement("test_operation");
        assert!(measurement_id > 0);
        
        // Simulate some work
        thread::sleep(Duration::from_millis(10));
        
        monitor.end_measurement(measurement_id);
        
        let metrics = monitor.get_current_metrics();
        assert!(metrics.operation_metrics.contains_key("test_operation"));
        
        let operation_metrics = &metrics.operation_metrics["test_operation"];
        assert_eq!(operation_metrics.total_calls, 1);
        assert!(operation_metrics.average_time_ns > 0);
    }
    
    #[test]
    fn test_latency_recording() {
        let mut monitor = AudioPerformanceMonitor::new();
        
        monitor.record_audio_latency(15.5);
        
        let metrics = monitor.get_current_metrics();
        assert_eq!(metrics.end_to_end_latency_ms, 15.5);
    }
    
    #[test]
    fn test_dropout_detection() {
        let mut monitor = AudioPerformanceMonitor::new();
        
        monitor.record_dropout(DropoutType::BufferUnderrun);
        monitor.record_dropout(DropoutType::BufferOverrun);
        
        let metrics = monitor.get_current_metrics();
        assert_eq!(metrics.dropout_count, 2);
        assert_eq!(metrics.buffer_underruns, 1);
        assert_eq!(metrics.buffer_overruns, 1);
    }
    
    #[test]
    fn test_threshold_configuration() {
        let mut monitor = AudioPerformanceMonitor::new();
        
        let custom_thresholds = PerformanceThresholds {
            max_end_to_end_latency_ms: 5.0,
            max_processing_latency_ms: 2.0,
            max_cpu_usage_percent: 50.0,
            ..Default::default()
        };
        
        monitor.update_thresholds(custom_thresholds.clone());
        
        if let Ok(thresholds) = monitor.thresholds.read() {
            assert_eq!(thresholds.max_end_to_end_latency_ms, 5.0);
            assert_eq!(thresholds.max_processing_latency_ms, 2.0);
            assert_eq!(thresholds.max_cpu_usage_percent, 50.0);
        }
    }
    
    #[test]
    fn test_processing_cycle_recording() {
        let mut monitor = AudioPerformanceMonitor::new();
        
        monitor.record_processing_cycle(8.5, 1024, 44100.0);
        
        let metrics = monitor.get_current_metrics();
        assert_eq!(metrics.processing_latency_ms, 8.5);
        assert!(metrics.end_to_end_latency_ms > 8.5); // Should include buffer latency
        assert_eq!(metrics.buffer_memory_bytes, 1024 * 4); // f32 = 4 bytes
    }
    
    #[test]
    fn test_historical_data_retention() {
        let monitor = AudioPerformanceMonitor::new();
        
        // Add some historical data points
        for i in 0..10 {
            monitor.add_historical_data();
            thread::sleep(Duration::from_millis(1));
        }
        
        let historical_metrics = monitor.get_historical_metrics(60); // Last 60 minutes
        assert!(historical_metrics.len() <= 10);
    }
    
    #[test]
    fn test_performance_regression_detection() {
        let monitor = AudioPerformanceMonitor::new();
        
        // Set baseline
        let baseline = AudioPerformanceMetrics {
            timestamp: get_timestamp_ns(),
            end_to_end_latency_ms: 5.0,
            ..monitor.get_current_metrics()
        };
        monitor.set_performance_baseline(baseline);
        
        // Simulate current metrics with regression
        if let Ok(mut current_metrics) = monitor.current_metrics.write() {
            current_metrics.end_to_end_latency_ms = 7.0; // 40% increase
        }
        
        let regression = monitor.detect_performance_regression();
        assert!(regression.is_some());
        
        let regression = regression.unwrap();
        assert_eq!(regression.metric_name, "end_to_end_latency_ms");
        assert_eq!(regression.baseline_value, 5.0);
        assert_eq!(regression.current_value, 7.0);
        assert!(regression.regression_percent > 35.0);
    }
    
    #[test]
    fn test_monitoring_overhead() {
        let mut monitor = AudioPerformanceMonitor::new();
        
        // Perform several operations to accumulate overhead
        for i in 0..100 {
            let id = monitor.start_measurement(&format!("operation_{}", i));
            monitor.end_measurement(id);
        }
        
        let metrics = monitor.get_current_metrics();
        
        // Monitoring overhead should be less than 5%
        assert!(metrics.monitoring_overhead.cpu_overhead_percent < MAX_MONITORING_OVERHEAD_PERCENT);
        assert!(metrics.monitoring_overhead.collection_time_ns > 0);
    }
    
    #[test]
    fn test_cpu_and_memory_recording() {
        let mut monitor = AudioPerformanceMonitor::new();
        
        monitor.record_cpu_usage(65.5);
        monitor.record_memory_usage(1024 * 1024); // 1MB
        
        let metrics = monitor.get_current_metrics();
        assert_eq!(metrics.cpu_usage_percent, 65.5);
        assert_eq!(metrics.memory_usage_bytes, 1024 * 1024);
    }
}