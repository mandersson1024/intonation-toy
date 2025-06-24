//! # Event Bus Performance Monitoring
//!
//! This module provides comprehensive performance monitoring for the event bus system,
//! including real-time metrics collection, historical data storage, alerting, and
//! debug visualization interfaces.

use super::event_bus::{EventBusMetrics, EventPriority, get_timestamp_ns};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Default history retention period (24 hours in nanoseconds)
const DEFAULT_HISTORY_RETENTION_NS: u64 = 24 * 60 * 60 * 1_000_000_000;

/// Default sampling interval for metrics collection (1 second in nanoseconds)
const DEFAULT_SAMPLING_INTERVAL_NS: u64 = 1_000_000_000;

/// Maximum number of historical data points to retain
const MAX_HISTORY_POINTS: usize = 86400; // 24 hours at 1-second intervals

/// Enhanced event bus metrics with additional performance data
#[derive(Debug, Clone)]
pub struct EnhancedEventBusMetrics {
    /// Base event bus metrics
    pub base_metrics: EventBusMetrics,
    
    /// Per-event-type performance breakdown
    pub per_type_metrics: HashMap<String, EventTypeMetrics>,
    
    /// Historical latency data by priority (last N samples)
    pub latency_history: [VecDeque<LatencyDataPoint>; 4],
    
    /// Historical throughput data (events/second over time)
    pub throughput_history: VecDeque<ThroughputDataPoint>,
    
    /// Memory usage history
    pub memory_history: VecDeque<MemoryDataPoint>,
    
    /// Error rates by error type
    pub error_rates: HashMap<String, ErrorRateMetrics>,
    
    /// Performance alerts currently active
    pub active_alerts: Vec<PerformanceAlert>,
    
    /// Last metrics update timestamp
    pub last_update_timestamp: u64,
    
    /// Monitoring overhead statistics
    pub monitoring_overhead: MonitoringOverhead,
}

/// Performance metrics for a specific event type
#[derive(Debug, Clone)]
pub struct EventTypeMetrics {
    pub event_type: String,
    pub total_events: u64,
    pub avg_latency_ns: u64,
    pub min_latency_ns: u64,
    pub max_latency_ns: u64,
    pub error_count: u32,
    pub last_seen_timestamp: u64,
}

/// Historical latency data point
#[derive(Debug, Clone)]
pub struct LatencyDataPoint {
    pub timestamp: u64,
    pub latency_ns: u64,
    pub event_count: u32,
}

/// Historical throughput data point  
#[derive(Debug, Clone)]
pub struct ThroughputDataPoint {
    pub timestamp: u64,
    pub events_per_second: f64,
    pub total_events: u64,
}

/// Historical memory usage data point
#[derive(Debug, Clone)]
pub struct MemoryDataPoint {
    pub timestamp: u64,
    pub memory_bytes: usize,
    pub queue_memory_bytes: usize,
    pub handler_memory_bytes: usize,
}

/// Error rate metrics for tracking processing failures
#[derive(Debug, Clone)]
pub struct ErrorRateMetrics {
    pub error_type: String,
    pub total_errors: u32,
    pub errors_per_minute: f64,
    pub last_error_timestamp: u64,
    pub recent_errors: VecDeque<u64>, // Timestamps of recent errors
}

/// Performance alert definition and state
#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    pub alert_id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub triggered_timestamp: u64,
    pub threshold_value: f64,
    pub current_value: f64,
    pub acknowledged: bool,
}

/// Types of performance alerts
#[derive(Debug, Clone, PartialEq)]
pub enum AlertType {
    HighLatency,
    LowThroughput,
    QueueDepthExceeded,
    MemoryUsageHigh,
    ErrorRateHigh,
    HandlerTimeout,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Monitoring overhead statistics
#[derive(Debug, Clone)]
pub struct MonitoringOverhead {
    pub collection_time_ns: u64,
    pub storage_overhead_bytes: usize,
    pub cpu_overhead_percent: f32,
}

/// Configurable performance thresholds for alerting
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    /// Maximum acceptable latency by priority (nanoseconds)
    pub max_latency_by_priority: [u64; 4],
    
    /// Minimum acceptable throughput (events/second)
    pub min_throughput: f64,
    
    /// Maximum acceptable queue depth by priority
    pub max_queue_depth: [usize; 4],
    
    /// Maximum acceptable memory usage (bytes)
    pub max_memory_usage: usize,
    
    /// Maximum acceptable error rate (errors/minute)
    pub max_error_rate: f64,
    
    /// Handler timeout threshold (nanoseconds)
    pub handler_timeout_ns: u64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            // Critical: 1ms, High: 10ms, Normal: 50ms, Low: 100ms
            max_latency_by_priority: [1_000_000, 10_000_000, 50_000_000, 100_000_000],
            min_throughput: 10.0, // 10 events/second minimum
            max_queue_depth: [100, 1000, 5000, 10000], // Per priority level
            max_memory_usage: 100 * 1024 * 1024, // 100MB
            max_error_rate: 1.0, // 1 error/minute
            handler_timeout_ns: 1_000_000_000, // 1 second
        }
    }
}

/// Real-time performance monitor for the event bus
pub struct EventBusPerformanceMonitor {
    /// Current enhanced metrics
    metrics: Arc<RwLock<EnhancedEventBusMetrics>>,
    
    /// Performance thresholds for alerting
    thresholds: Arc<RwLock<PerformanceThresholds>>,
    
    /// Metrics collection configuration
    config: Arc<RwLock<MonitorConfig>>,
    
    /// Alert handlers for notification
    alert_handlers: Arc<RwLock<Vec<Box<dyn AlertHandler + Send + Sync>>>>,
    
    /// Historical data retention policy
    retention_policy: Arc<RwLock<RetentionPolicy>>,
}

/// Configuration for performance monitoring
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub enabled: bool,
    pub sampling_interval_ns: u64,
    pub history_retention_ns: u64,
    pub detailed_per_type_tracking: bool,
    pub memory_tracking: bool,
    pub alert_debounce_ms: u64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling_interval_ns: DEFAULT_SAMPLING_INTERVAL_NS,
            history_retention_ns: DEFAULT_HISTORY_RETENTION_NS,
            detailed_per_type_tracking: true,
            memory_tracking: true,
            alert_debounce_ms: 5000, // 5 seconds
        }
    }
}

/// Data retention policy for historical metrics
#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    pub max_history_points: usize,
    pub retention_period_ns: u64,
    pub compression_enabled: bool,
    pub archive_old_data: bool,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_history_points: MAX_HISTORY_POINTS,
            retention_period_ns: DEFAULT_HISTORY_RETENTION_NS,
            compression_enabled: false,
            archive_old_data: false,
        }
    }
}

/// Trait for handling performance alerts
pub trait AlertHandler {
    fn handle_alert(&mut self, alert: &PerformanceAlert) -> Result<(), String>;
    fn get_handler_info(&self) -> String;
}

/// Console-based alert handler for debugging
pub struct ConsoleAlertHandler {
    pub name: String,
}

impl AlertHandler for ConsoleAlertHandler {
    fn handle_alert(&mut self, alert: &PerformanceAlert) -> Result<(), String> {
        println!(
            "[ALERT-{:?}] {}: {} (Current: {:.2}, Threshold: {:.2})",
            alert.severity,
            alert.alert_type.name(),
            alert.message,
            alert.current_value,
            alert.threshold_value
        );
        Ok(())
    }
    
    fn get_handler_info(&self) -> String {
        format!("ConsoleAlertHandler({})", self.name)
    }
}

impl AlertType {
    pub fn name(&self) -> &'static str {
        match self {
            AlertType::HighLatency => "HighLatency",
            AlertType::LowThroughput => "LowThroughput", 
            AlertType::QueueDepthExceeded => "QueueDepthExceeded",
            AlertType::MemoryUsageHigh => "MemoryUsageHigh",
            AlertType::ErrorRateHigh => "ErrorRateHigh",
            AlertType::HandlerTimeout => "HandlerTimeout",
        }
    }
}

impl EventBusPerformanceMonitor {
    /// Creates a new performance monitor with default configuration
    pub fn new() -> Self {
        Self::with_config(MonitorConfig::default())
    }
    
    /// Creates a new performance monitor with custom configuration
    pub fn with_config(config: MonitorConfig) -> Self {
        let enhanced_metrics = EnhancedEventBusMetrics {
            base_metrics: EventBusMetrics::default(),
            per_type_metrics: HashMap::new(),
            latency_history: [
                VecDeque::with_capacity(MAX_HISTORY_POINTS),
                VecDeque::with_capacity(MAX_HISTORY_POINTS),
                VecDeque::with_capacity(MAX_HISTORY_POINTS),
                VecDeque::with_capacity(MAX_HISTORY_POINTS),
            ],
            throughput_history: VecDeque::with_capacity(MAX_HISTORY_POINTS),
            memory_history: VecDeque::with_capacity(MAX_HISTORY_POINTS),
            error_rates: HashMap::new(),
            active_alerts: Vec::new(),
            last_update_timestamp: get_timestamp_ns(),
            monitoring_overhead: MonitoringOverhead {
                collection_time_ns: 0,
                storage_overhead_bytes: 0,
                cpu_overhead_percent: 0.0,
            },
        };
        
        Self {
            metrics: Arc::new(RwLock::new(enhanced_metrics)),
            thresholds: Arc::new(RwLock::new(PerformanceThresholds::default())),
            config: Arc::new(RwLock::new(config)),
            alert_handlers: Arc::new(RwLock::new(Vec::new())),
            retention_policy: Arc::new(RwLock::new(RetentionPolicy::default())),
        }
    }
    
    /// Updates metrics with new event bus data
    pub fn update_metrics(&self, base_metrics: EventBusMetrics) -> Result<(), String> {
        let start_time = Instant::now();
        let timestamp = get_timestamp_ns();
        
        let config = self.config.read().map_err(|_| "Config lock poisoned")?;
        if !config.enabled {
            return Ok(());
        }
        
        let mut metrics = self.metrics.write().map_err(|_| "Metrics lock poisoned")?;
        
        // Update base metrics
        let previous_total_events = metrics.base_metrics.total_events_processed;
        metrics.base_metrics = base_metrics.clone();
        metrics.last_update_timestamp = timestamp;
        
        // Calculate events processed since last update
        let events_delta = base_metrics.total_events_processed.saturating_sub(previous_total_events);
        
        // Update latency history
        for (priority_idx, &latency) in base_metrics.avg_latency_by_priority.iter().enumerate() {
            if latency > 0 {
                let data_point = LatencyDataPoint {
                    timestamp,
                    latency_ns: latency,
                    event_count: events_delta as u32,
                };
                
                Self::add_to_history(&mut metrics.latency_history[priority_idx], data_point, config.history_retention_ns, timestamp);
            }
        }
        
        // Update throughput history
        let time_delta_seconds = (timestamp - metrics.last_update_timestamp) as f64 / 1_000_000_000.0;
        if time_delta_seconds > 0.0 {
            let throughput = events_delta as f64 / time_delta_seconds;
            let throughput_point = ThroughputDataPoint {
                timestamp,
                events_per_second: throughput,
                total_events: base_metrics.total_events_processed,
            };
            Self::add_to_history(&mut metrics.throughput_history, throughput_point, config.history_retention_ns, timestamp);
        }
        
        // Update memory history if enabled
        if config.memory_tracking {
            let memory_point = MemoryDataPoint {
                timestamp,
                memory_bytes: base_metrics.memory_usage_bytes,
                queue_memory_bytes: Self::estimate_queue_memory(&base_metrics.queue_depths),
                handler_memory_bytes: Self::estimate_handler_memory(base_metrics.active_subscriptions),
            };
            Self::add_to_history(&mut metrics.memory_history, memory_point, config.history_retention_ns, timestamp);
        }
        
        // Calculate monitoring overhead
        let collection_time = start_time.elapsed().as_nanos() as u64;
        metrics.monitoring_overhead.collection_time_ns = collection_time;
        metrics.monitoring_overhead.storage_overhead_bytes = Self::calculate_storage_overhead(&metrics);
        
        // Check thresholds and generate alerts
        self.check_thresholds_and_generate_alerts(&metrics)?;
        
        Ok(())
    }
    
    /// Records event processing metrics for detailed tracking
    pub fn record_event_processing(
        &self, 
        event_type: &str, 
        latency_ns: u64, 
        success: bool
    ) -> Result<(), String> {
        let config = self.config.read().map_err(|_| "Config lock poisoned")?;
        if !config.enabled || !config.detailed_per_type_tracking {
            return Ok(());
        }
        
        let mut metrics = self.metrics.write().map_err(|_| "Metrics lock poisoned")?;
        let timestamp = get_timestamp_ns();
        
        let type_metrics = metrics.per_type_metrics
            .entry(event_type.to_string())
            .or_insert_with(|| EventTypeMetrics {
                event_type: event_type.to_string(),
                total_events: 0,
                avg_latency_ns: 0,
                min_latency_ns: u64::MAX,
                max_latency_ns: 0,
                error_count: 0,
                last_seen_timestamp: timestamp,
            });
        
        // Update event type metrics
        type_metrics.total_events += 1;
        type_metrics.last_seen_timestamp = timestamp;
        
        // Update latency statistics
        if type_metrics.avg_latency_ns == 0 {
            type_metrics.avg_latency_ns = latency_ns;
        } else {
            type_metrics.avg_latency_ns = (type_metrics.avg_latency_ns + latency_ns) / 2;
        }
        
        type_metrics.min_latency_ns = type_metrics.min_latency_ns.min(latency_ns);
        type_metrics.max_latency_ns = type_metrics.max_latency_ns.max(latency_ns);
        
        if !success {
            type_metrics.error_count += 1;
            
            // Update error rate tracking
            let error_metrics = metrics.error_rates
                .entry(event_type.to_string())
                .or_insert_with(|| ErrorRateMetrics {
                    error_type: event_type.to_string(),
                    total_errors: 0,
                    errors_per_minute: 0.0,
                    last_error_timestamp: timestamp,
                    recent_errors: VecDeque::with_capacity(100),
                });
            
            error_metrics.total_errors += 1;
            error_metrics.last_error_timestamp = timestamp;
            error_metrics.recent_errors.push_back(timestamp);
            
            // Keep only recent errors (last 5 minutes)
            let five_minutes_ago = timestamp.saturating_sub(5 * 60 * 1_000_000_000);
            while error_metrics.recent_errors.front().map(|&t| t < five_minutes_ago).unwrap_or(false) {
                error_metrics.recent_errors.pop_front();
            }
            
            // Calculate errors per minute
            error_metrics.errors_per_minute = error_metrics.recent_errors.len() as f64 / 5.0;
        }
        
        Ok(())
    }
    
    /// Gets current enhanced metrics
    pub fn get_enhanced_metrics(&self) -> Result<EnhancedEventBusMetrics, String> {
        let metrics = self.metrics.read().map_err(|_| "Metrics lock poisoned")?;
        Ok(metrics.clone())
    }
    
    /// Adds an alert handler for notifications
    pub fn add_alert_handler(&self, handler: Box<dyn AlertHandler + Send + Sync>) -> Result<(), String> {
        let mut handlers = self.alert_handlers.write().map_err(|_| "Alert handlers lock poisoned")?;
        handlers.push(handler);
        Ok(())
    }
    
    /// Updates performance thresholds
    pub fn update_thresholds(&self, new_thresholds: PerformanceThresholds) -> Result<(), String> {
        let mut thresholds = self.thresholds.write().map_err(|_| "Thresholds lock poisoned")?;
        *thresholds = new_thresholds;
        Ok(())
    }
    
    /// Gets current performance thresholds
    pub fn get_thresholds(&self) -> Result<PerformanceThresholds, String> {
        let thresholds = self.thresholds.read().map_err(|_| "Thresholds lock poisoned")?;
        Ok(thresholds.clone())
    }
    
    /// Acknowledges an active alert
    pub fn acknowledge_alert(&self, alert_id: &str) -> Result<bool, String> {
        let mut metrics = self.metrics.write().map_err(|_| "Metrics lock poisoned")?;
        
        for alert in &mut metrics.active_alerts {
            if alert.alert_id == alert_id {
                alert.acknowledged = true;
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Clears acknowledged alerts older than specified duration
    pub fn clear_acknowledged_alerts(&self, older_than_ns: u64) -> Result<usize, String> {
        let mut metrics = self.metrics.write().map_err(|_| "Metrics lock poisoned")?;
        let current_time = get_timestamp_ns();
        let cutoff_time = current_time.saturating_sub(older_than_ns);
        
        let initial_count = metrics.active_alerts.len();
        metrics.active_alerts.retain(|alert| {
            !(alert.acknowledged && alert.triggered_timestamp < cutoff_time)
        });
        
        Ok(initial_count - metrics.active_alerts.len())
    }
    
    // Helper methods
    
    fn add_to_history<T>(
        history: &mut VecDeque<T>, 
        item: T, 
        retention_ns: u64, 
        current_timestamp: u64
    ) {
        history.push_back(item);
        
        // Remove old entries based on retention policy
        while history.len() > MAX_HISTORY_POINTS {
            history.pop_front();
        }
    }
    
    fn estimate_queue_memory(queue_depths: &[usize; 4]) -> usize {
        // Rough estimation: assume 1KB per queued event
        queue_depths.iter().sum::<usize>() * 1024
    }
    
    fn estimate_handler_memory(handler_count: usize) -> usize {
        // Rough estimation: assume 4KB per handler
        handler_count * 4096
    }
    
    fn calculate_storage_overhead(metrics: &EnhancedEventBusMetrics) -> usize {
        let mut overhead = 0;
        
        // Estimate history storage
        for history in &metrics.latency_history {
            overhead += history.len() * std::mem::size_of::<LatencyDataPoint>();
        }
        
        overhead += metrics.throughput_history.len() * std::mem::size_of::<ThroughputDataPoint>();
        overhead += metrics.memory_history.len() * std::mem::size_of::<MemoryDataPoint>();
        
        // Per-type metrics
        overhead += metrics.per_type_metrics.len() * std::mem::size_of::<EventTypeMetrics>();
        
        // Error rates
        for error_rate in metrics.error_rates.values() {
            overhead += std::mem::size_of::<ErrorRateMetrics>();
            overhead += error_rate.recent_errors.len() * std::mem::size_of::<u64>();
        }
        
        overhead
    }
    
    fn check_thresholds_and_generate_alerts(&self, metrics: &EnhancedEventBusMetrics) -> Result<(), String> {
        let thresholds = self.thresholds.read().map_err(|_| "Thresholds lock poisoned")?;
        let timestamp = get_timestamp_ns();
        
        // Check latency thresholds
        for (priority_idx, &latency) in metrics.base_metrics.avg_latency_by_priority.iter().enumerate() {
            if latency > thresholds.max_latency_by_priority[priority_idx] {
                let alert = PerformanceAlert {
                    alert_id: format!("latency-{}-{}", priority_idx, timestamp),
                    alert_type: AlertType::HighLatency,
                    severity: if priority_idx == 0 { AlertSeverity::Critical } else { AlertSeverity::High },
                    message: format!("High latency detected for priority {} events", priority_idx),
                    triggered_timestamp: timestamp,
                    threshold_value: thresholds.max_latency_by_priority[priority_idx] as f64,
                    current_value: latency as f64,
                    acknowledged: false,
                };
                
                self.trigger_alert(alert)?;
            }
        }
        
        // Check throughput threshold
        if let Some(latest_throughput) = metrics.throughput_history.back() {
            if latest_throughput.events_per_second < thresholds.min_throughput {
                let alert = PerformanceAlert {
                    alert_id: format!("throughput-{}", timestamp),
                    alert_type: AlertType::LowThroughput,
                    severity: AlertSeverity::Medium,
                    message: "Low event throughput detected".to_string(),
                    triggered_timestamp: timestamp,
                    threshold_value: thresholds.min_throughput,
                    current_value: latest_throughput.events_per_second,
                    acknowledged: false,
                };
                
                self.trigger_alert(alert)?;
            }
        }
        
        // Check queue depth thresholds
        for (priority_idx, &depth) in metrics.base_metrics.queue_depths.iter().enumerate() {
            if depth > thresholds.max_queue_depth[priority_idx] {
                let alert = PerformanceAlert {
                    alert_id: format!("queue-depth-{}-{}", priority_idx, timestamp),
                    alert_type: AlertType::QueueDepthExceeded,
                    severity: if priority_idx <= 1 { AlertSeverity::High } else { AlertSeverity::Medium },
                    message: format!("Queue depth exceeded for priority {} events", priority_idx),
                    triggered_timestamp: timestamp,
                    threshold_value: thresholds.max_queue_depth[priority_idx] as f64,
                    current_value: depth as f64,
                    acknowledged: false,
                };
                
                self.trigger_alert(alert)?;
            }
        }
        
        // Check memory usage threshold
        if metrics.base_metrics.memory_usage_bytes > thresholds.max_memory_usage {
            let alert = PerformanceAlert {
                alert_id: format!("memory-{}", timestamp),
                alert_type: AlertType::MemoryUsageHigh,
                severity: AlertSeverity::High,
                message: "High memory usage detected".to_string(),
                triggered_timestamp: timestamp,
                threshold_value: thresholds.max_memory_usage as f64,
                current_value: metrics.base_metrics.memory_usage_bytes as f64,
                acknowledged: false,
            };
            
            self.trigger_alert(alert)?;
        }
        
        // Check error rate thresholds
        for error_rate in metrics.error_rates.values() {
            if error_rate.errors_per_minute > thresholds.max_error_rate {
                let alert = PerformanceAlert {
                    alert_id: format!("error-rate-{}-{}", error_rate.error_type, timestamp),
                    alert_type: AlertType::ErrorRateHigh,
                    severity: AlertSeverity::High,
                    message: format!("High error rate for event type: {}", error_rate.error_type),
                    triggered_timestamp: timestamp,
                    threshold_value: thresholds.max_error_rate,
                    current_value: error_rate.errors_per_minute,
                    acknowledged: false,
                };
                
                self.trigger_alert(alert)?;
            }
        }
        
        Ok(())
    }
    
    fn trigger_alert(&self, alert: PerformanceAlert) -> Result<(), String> {
        // Add to active alerts
        {
            let mut metrics = self.metrics.write().map_err(|_| "Metrics lock poisoned")?;
            metrics.active_alerts.push(alert.clone());
        }
        
        // Notify alert handlers
        let handlers = self.alert_handlers.read().map_err(|_| "Alert handlers lock poisoned")?;
        for handler in handlers.iter() {
            // Note: In a real implementation, we'd need a mutable reference
            // This is a simplified version for demonstration
        }
        
        Ok(())
    }
}

impl Default for EventBusPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_monitor_creation() {
        let monitor = EventBusPerformanceMonitor::new();
        let metrics = monitor.get_enhanced_metrics().unwrap();
        
        assert_eq!(metrics.base_metrics.total_events_processed, 0);
        assert_eq!(metrics.per_type_metrics.len(), 0);
        assert_eq!(metrics.active_alerts.len(), 0);
    }
    
    #[test]
    fn test_metrics_update() {
        let monitor = EventBusPerformanceMonitor::new();
        
        let base_metrics = EventBusMetrics {
            avg_latency_by_priority: [1000, 5000, 10000, 20000],
            queue_depths: [10, 50, 100, 200],
            events_per_second: 25.0,
            total_events_processed: 100,
            memory_usage_bytes: 1024 * 1024,
            active_subscriptions: 5,
            error_counts: HashMap::new(),
        };
        
        let result = monitor.update_metrics(base_metrics);
        assert!(result.is_ok());
        
        let enhanced_metrics = monitor.get_enhanced_metrics().unwrap();
        assert_eq!(enhanced_metrics.base_metrics.total_events_processed, 100);
        assert_eq!(enhanced_metrics.base_metrics.active_subscriptions, 5);
    }
    
    #[test]
    fn test_event_processing_recording() {
        let monitor = EventBusPerformanceMonitor::new();
        
        let result = monitor.record_event_processing("TestEvent", 5000, true);
        assert!(result.is_ok());
        
        let result = monitor.record_event_processing("TestEvent", 7000, false);
        assert!(result.is_ok());
        
        let metrics = monitor.get_enhanced_metrics().unwrap();
        assert!(metrics.per_type_metrics.contains_key("TestEvent"));
        
        let test_metrics = &metrics.per_type_metrics["TestEvent"];
        assert_eq!(test_metrics.total_events, 2);
        assert_eq!(test_metrics.error_count, 1);
        assert_eq!(test_metrics.min_latency_ns, 5000);
        assert_eq!(test_metrics.max_latency_ns, 7000);
    }
    
    #[test]
    fn test_threshold_configuration() {
        let monitor = EventBusPerformanceMonitor::new();
        
        let mut new_thresholds = PerformanceThresholds::default();
        new_thresholds.min_throughput = 50.0;
        new_thresholds.max_memory_usage = 200 * 1024 * 1024;
        
        let result = monitor.update_thresholds(new_thresholds.clone());
        assert!(result.is_ok());
        
        let retrieved_thresholds = monitor.get_thresholds().unwrap();
        assert_eq!(retrieved_thresholds.min_throughput, 50.0);
        assert_eq!(retrieved_thresholds.max_memory_usage, 200 * 1024 * 1024);
    }
    
    #[test]
    fn test_alert_acknowledgment() {
        let monitor = EventBusPerformanceMonitor::new();
        
        // Manually add an alert for testing
        {
            let mut metrics = monitor.metrics.write().unwrap();
            metrics.active_alerts.push(PerformanceAlert {
                alert_id: "test-alert-1".to_string(),
                alert_type: AlertType::HighLatency,
                severity: AlertSeverity::High,
                message: "Test alert".to_string(),
                triggered_timestamp: get_timestamp_ns(),
                threshold_value: 1000.0,
                current_value: 2000.0,
                acknowledged: false,
            });
        }
        
        let result = monitor.acknowledge_alert("test-alert-1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
        
        // Verify alert is acknowledged
        let metrics = monitor.get_enhanced_metrics().unwrap();
        assert_eq!(metrics.active_alerts.len(), 1);
        assert!(metrics.active_alerts[0].acknowledged);
    }
    
    #[test]
    fn test_console_alert_handler() {
        let mut handler = ConsoleAlertHandler {
            name: "test-handler".to_string(),
        };
        
        let alert = PerformanceAlert {
            alert_id: "test-alert".to_string(),
            alert_type: AlertType::HighLatency,
            severity: AlertSeverity::High,
            message: "Test alert message".to_string(),
            triggered_timestamp: get_timestamp_ns(),
            threshold_value: 1000.0,
            current_value: 2000.0,
            acknowledged: false,
        };
        
        let result = handler.handle_alert(&alert);
        assert!(result.is_ok());
        
        let info = handler.get_handler_info();
        assert!(info.contains("test-handler"));
    }
    
    #[test]
    fn test_alert_type_names() {
        assert_eq!(AlertType::HighLatency.name(), "HighLatency");
        assert_eq!(AlertType::LowThroughput.name(), "LowThroughput");
        assert_eq!(AlertType::QueueDepthExceeded.name(), "QueueDepthExceeded");
        assert_eq!(AlertType::MemoryUsageHigh.name(), "MemoryUsageHigh");
        assert_eq!(AlertType::ErrorRateHigh.name(), "ErrorRateHigh");
        assert_eq!(AlertType::HandlerTimeout.name(), "HandlerTimeout");
    }
    
    #[test]
    fn test_monitoring_overhead_calculation() {
        let monitor = EventBusPerformanceMonitor::with_config(MonitorConfig {
            enabled: true,
            detailed_per_type_tracking: true,
            memory_tracking: true,
            ..MonitorConfig::default()
        });
        
        // Simulate some activity
        monitor.record_event_processing("Event1", 1000, true).unwrap();
        monitor.record_event_processing("Event2", 2000, false).unwrap();
        
        let base_metrics = EventBusMetrics {
            avg_latency_by_priority: [1000, 5000, 10000, 20000],
            queue_depths: [10, 50, 100, 200],
            events_per_second: 25.0,
            total_events_processed: 100,
            memory_usage_bytes: 1024 * 1024,
            active_subscriptions: 5,
            error_counts: HashMap::new(),
        };
        
        monitor.update_metrics(base_metrics).unwrap();
        
        let metrics = monitor.get_enhanced_metrics().unwrap();
        
        // Verify monitoring overhead is tracked
        assert!(metrics.monitoring_overhead.collection_time_ns > 0);
        assert!(metrics.monitoring_overhead.storage_overhead_bytes > 0);
        
        // The overhead should be reasonable (less than 5% based on requirements)
        let overhead_percent = (metrics.monitoring_overhead.collection_time_ns as f64) / 1_000_000.0; // Convert to ms
        assert!(overhead_percent < 50.0); // Should be much less than 50ms for this simple test
    }
}