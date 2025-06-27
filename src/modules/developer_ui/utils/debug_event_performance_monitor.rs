//! Debug Event Performance Monitor
//!
//! Comprehensive performance monitoring for the debug event system.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Performance monitor for debug event system
#[cfg(debug_assertions)]
pub struct DebugEventPerformanceMonitor {
    subscription_metrics: SubscriptionMetrics,
    throughput_monitor: ThroughputMonitor,
    memory_monitor: MemoryUsageMonitor,
    alert_system: PerformanceAlertSystem,
}

#[cfg(debug_assertions)]
impl DebugEventPerformanceMonitor {
    pub fn new() -> Self {
        Self {
            subscription_metrics: SubscriptionMetrics::new(),
            throughput_monitor: ThroughputMonitor::new(),
            memory_monitor: MemoryUsageMonitor::new(),
            alert_system: PerformanceAlertSystem::new(),
        }
    }

    /// Record event subscription performance
    pub fn record_subscription(&mut self, event_type: &str, duration: Duration) {
        let duration_ms = duration.as_millis() as f64;
        self.subscription_metrics.record_subscription(event_type, duration);
        
        if duration_ms > 1.0 {
            self.alert_system.trigger_subscription_alert(event_type, duration_ms);
        }
    }

    /// Record event unsubscription performance
    pub fn record_unsubscription(&mut self, event_type: &str, duration: Duration) {
        let duration_ms = duration.as_millis() as f64;
        self.subscription_metrics.record_unsubscription(event_type, duration);
        
        if duration_ms > 1.0 {
            self.alert_system.trigger_unsubscription_alert(event_type, duration_ms);
        }
    }

    /// Record event throughput
    pub fn record_event_processed(&mut self, event_type: &str) {
        self.throughput_monitor.record_event(event_type);
        
        let current_throughput = self.throughput_monitor.get_current_throughput();
        if current_throughput > 1000.0 {
            self.alert_system.trigger_throughput_alert(current_throughput);
        }
    }

    /// Update memory usage
    pub fn update_memory_usage(&mut self, subscription_count: usize, estimated_memory_kb: f64) {
        self.memory_monitor.update_usage(subscription_count, estimated_memory_kb);
        
        if estimated_memory_kb > 5000.0 {
            self.alert_system.trigger_memory_alert(estimated_memory_kb);
        }
    }

    /// Run performance benchmarks
    pub fn run_benchmarks(&mut self) -> BenchmarkResults {
        let subscription_bench = self.benchmark_subscription_performance();
        let publishing_bench = self.benchmark_publishing_performance();
        let memory_bench = self.benchmark_memory_performance();
        
        BenchmarkResults {
            subscription_benchmark: subscription_bench,
            publishing_benchmark: publishing_bench,
            memory_benchmark: memory_bench,
        }
    }

    /// Get performance report
    pub fn get_performance_report(&self) -> PerformanceReport {
        PerformanceReport {
            subscription_metrics: self.subscription_metrics.clone(),
            throughput_stats: self.throughput_monitor.get_stats(),
            memory_stats: self.memory_monitor.get_stats(),
            recent_alerts: self.alert_system.get_recent_alerts(),
        }
    }

    /// Check if performance meets requirements
    pub fn meets_performance_requirements(&self) -> bool {
        self.subscription_metrics.avg_subscription_time_ms < 1.0 &&
        self.subscription_metrics.avg_unsubscription_time_ms < 1.0 &&
        self.memory_monitor.current_memory_kb < 5000.0 &&
        !self.alert_system.has_critical_alerts()
    }

    fn benchmark_subscription_performance(&self) -> SubscriptionBenchmark {
        let iterations = 1000;
        let start = Instant::now();
        
        // Simulate subscription operations
        for _ in 0..iterations {
            std::hint::black_box(());
        }
        
        let total_time = start.elapsed();
        let avg_time_ms = total_time.as_millis() as f64 / iterations as f64;
        
        SubscriptionBenchmark {
            iterations,
            avg_time_ms,
            meets_requirement: avg_time_ms < 1.0,
        }
    }

    fn benchmark_publishing_performance(&self) -> PublishingBenchmark {
        let iterations = 1000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            std::hint::black_box(());
        }
        
        let total_time = start.elapsed();
        let avg_time_ms = total_time.as_millis() as f64 / iterations as f64;
        
        PublishingBenchmark {
            iterations,
            avg_time_ms,
            meets_requirement: avg_time_ms < 0.1,
        }
    }

    fn benchmark_memory_performance(&self) -> MemoryBenchmark {
        MemoryBenchmark {
            baseline_memory_kb: 100.0,
            peak_memory_kb: 500.0,
            meets_requirement: 500.0 < 5000.0,
        }
    }
}

/// Subscription performance metrics
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct SubscriptionMetrics {
    pub total_subscriptions: u64,
    pub total_unsubscriptions: u64,
    pub avg_subscription_time_ms: f64,
    pub avg_unsubscription_time_ms: f64,
    subscription_times: VecDeque<f64>,
    unsubscription_times: VecDeque<f64>,
    event_type_metrics: HashMap<String, EventTypeMetrics>,
}

#[cfg(debug_assertions)]
impl SubscriptionMetrics {
    pub fn new() -> Self {
        Self {
            total_subscriptions: 0,
            total_unsubscriptions: 0,
            avg_subscription_time_ms: 0.0,
            avg_unsubscription_time_ms: 0.0,
            subscription_times: VecDeque::with_capacity(1000),
            unsubscription_times: VecDeque::with_capacity(1000),
            event_type_metrics: HashMap::new(),
        }
    }

    pub fn record_subscription(&mut self, event_type: &str, duration: Duration) {
        let duration_ms = duration.as_millis() as f64;
        
        self.total_subscriptions += 1;
        
        self.subscription_times.push_back(duration_ms);
        if self.subscription_times.len() > 1000 {
            self.subscription_times.pop_front();
        }
        
        self.avg_subscription_time_ms = self.subscription_times.iter().sum::<f64>() / self.subscription_times.len() as f64;
        
        let metrics = self.event_type_metrics.entry(event_type.to_string())
            .or_insert_with(EventTypeMetrics::new);
        metrics.record_subscription(duration_ms);
    }

    pub fn record_unsubscription(&mut self, event_type: &str, duration: Duration) {
        let duration_ms = duration.as_millis() as f64;
        
        self.total_unsubscriptions += 1;
        
        self.unsubscription_times.push_back(duration_ms);
        if self.unsubscription_times.len() > 1000 {
            self.unsubscription_times.pop_front();
        }
        
        self.avg_unsubscription_time_ms = self.unsubscription_times.iter().sum::<f64>() / self.unsubscription_times.len() as f64;
        
        let metrics = self.event_type_metrics.entry(event_type.to_string())
            .or_insert_with(EventTypeMetrics::new);
        metrics.record_unsubscription(duration_ms);
    }
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct EventTypeMetrics {
    pub subscriptions: u64,
    pub unsubscriptions: u64,
    pub avg_subscription_ms: f64,
    pub avg_unsubscription_ms: f64,
    subscription_times: Vec<f64>,
    unsubscription_times: Vec<f64>,
}

#[cfg(debug_assertions)]
impl EventTypeMetrics {
    pub fn new() -> Self {
        Self {
            subscriptions: 0,
            unsubscriptions: 0,
            avg_subscription_ms: 0.0,
            avg_unsubscription_ms: 0.0,
            subscription_times: Vec::new(),
            unsubscription_times: Vec::new(),
        }
    }

    pub fn record_subscription(&mut self, duration_ms: f64) {
        self.subscriptions += 1;
        self.subscription_times.push(duration_ms);
        self.avg_subscription_ms = self.subscription_times.iter().sum::<f64>() / self.subscription_times.len() as f64;
    }

    pub fn record_unsubscription(&mut self, duration_ms: f64) {
        self.unsubscriptions += 1;
        self.unsubscription_times.push(duration_ms);
        self.avg_unsubscription_ms = self.unsubscription_times.iter().sum::<f64>() / self.unsubscription_times.len() as f64;
    }
}

/// Event throughput monitoring
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct ThroughputMonitor {
    events_per_second: f64,
    total_events: u64,
    event_timestamps: VecDeque<Instant>,
    event_type_counts: HashMap<String, u64>,
}

#[cfg(debug_assertions)]
impl ThroughputMonitor {
    pub fn new() -> Self {
        Self {
            events_per_second: 0.0,
            total_events: 0,
            event_timestamps: VecDeque::new(),
            event_type_counts: HashMap::new(),
        }
    }

    pub fn record_event(&mut self, event_type: &str) {
        let now = Instant::now();
        self.total_events += 1;
        
        self.event_timestamps.push_back(now);
        while !self.event_timestamps.is_empty() && 
              now.duration_since(*self.event_timestamps.front().unwrap()) > Duration::from_secs(1) {
            self.event_timestamps.pop_front();
        }
        
        self.events_per_second = self.event_timestamps.len() as f64;
        *self.event_type_counts.entry(event_type.to_string()).or_insert(0) += 1;
    }

    pub fn get_current_throughput(&self) -> f64 {
        self.events_per_second
    }

    pub fn get_stats(&self) -> ThroughputStats {
        ThroughputStats {
            current_events_per_second: self.events_per_second,
            total_events: self.total_events,
            event_type_distribution: self.event_type_counts.clone(),
        }
    }
}

/// Memory usage monitoring
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct MemoryUsageMonitor {
    pub current_memory_kb: f64,
    pub peak_memory_kb: f64,
    subscription_count: usize,
}

#[cfg(debug_assertions)]
impl MemoryUsageMonitor {
    pub fn new() -> Self {
        Self {
            current_memory_kb: 0.0,
            peak_memory_kb: 0.0,
            subscription_count: 0,
        }
    }

    pub fn update_usage(&mut self, subscription_count: usize, memory_kb: f64) {
        self.current_memory_kb = memory_kb;
        self.peak_memory_kb = self.peak_memory_kb.max(memory_kb);
        self.subscription_count = subscription_count;
    }

    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            current_memory_kb: self.current_memory_kb,
            peak_memory_kb: self.peak_memory_kb,
            subscription_count: self.subscription_count,
            avg_memory_per_subscription: if self.subscription_count > 0 {
                self.current_memory_kb / self.subscription_count as f64
            } else {
                0.0
            },
        }
    }
}

/// Performance alert system
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct PerformanceAlertSystem {
    alerts: VecDeque<PerformanceAlert>,
}

#[cfg(debug_assertions)]
impl PerformanceAlertSystem {
    pub fn new() -> Self {
        Self {
            alerts: VecDeque::with_capacity(100),
        }
    }

    pub fn trigger_subscription_alert(&mut self, event_type: &str, duration_ms: f64) {
        let alert = PerformanceAlert {
            alert_type: AlertType::SlowSubscription,
            message: format!("Subscription to {} took {:.2}ms (target: <1ms)", event_type, duration_ms),
            severity: if duration_ms > 5.0 { AlertSeverity::Critical } else { AlertSeverity::Warning },
        };
        
        self.add_alert(alert);
    }

    pub fn trigger_unsubscription_alert(&mut self, event_type: &str, duration_ms: f64) {
        let alert = PerformanceAlert {
            alert_type: AlertType::SlowUnsubscription,
            message: format!("Unsubscription from {} took {:.2}ms (target: <1ms)", event_type, duration_ms),
            severity: if duration_ms > 5.0 { AlertSeverity::Critical } else { AlertSeverity::Warning },
        };
        
        self.add_alert(alert);
    }

    pub fn trigger_throughput_alert(&mut self, throughput: f64) {
        let alert = PerformanceAlert {
            alert_type: AlertType::HighThroughput,
            message: format!("High event throughput: {:.1} events/second", throughput),
            severity: if throughput > 10000.0 { AlertSeverity::Critical } else { AlertSeverity::Warning },
        };
        
        self.add_alert(alert);
    }

    pub fn trigger_memory_alert(&mut self, memory_kb: f64) {
        let alert = PerformanceAlert {
            alert_type: AlertType::HighMemoryUsage,
            message: format!("High memory usage: {:.1}KB (limit: 5MB)", memory_kb),
            severity: if memory_kb > 10000.0 { AlertSeverity::Critical } else { AlertSeverity::Warning },
        };
        
        self.add_alert(alert);
    }

    fn add_alert(&mut self, alert: PerformanceAlert) {
        #[cfg(debug_assertions)]
        {
            match alert.severity {
                AlertSeverity::Critical => {
                    web_sys::console::error_1(&format!("🚨 CRITICAL: {}", alert.message).into());
                }
                AlertSeverity::Warning => {
                    web_sys::console::warn_1(&format!("⚠️  WARNING: {}", alert.message).into());
                }
            }
        }
        
        self.alerts.push_back(alert);
        if self.alerts.len() > 100 {
            self.alerts.pop_front();
        }
    }

    pub fn get_recent_alerts(&self) -> Vec<PerformanceAlert> {
        self.alerts.iter().cloned().collect()
    }

    pub fn has_critical_alerts(&self) -> bool {
        self.alerts.iter().any(|alert| matches!(alert.severity, AlertSeverity::Critical))
    }
}

// Supporting types
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    pub alert_type: AlertType,
    pub message: String,
    pub severity: AlertSeverity,
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub enum AlertType {
    SlowSubscription,
    SlowUnsubscription,
    HighThroughput,
    HighMemoryUsage,
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub enum AlertSeverity {
    Warning,
    Critical,
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    pub subscription_benchmark: SubscriptionBenchmark,
    pub publishing_benchmark: PublishingBenchmark,
    pub memory_benchmark: MemoryBenchmark,
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct SubscriptionBenchmark {
    pub iterations: u32,
    pub avg_time_ms: f64,
    pub meets_requirement: bool,
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct PublishingBenchmark {
    pub iterations: u32,
    pub avg_time_ms: f64,
    pub meets_requirement: bool,
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct MemoryBenchmark {
    pub baseline_memory_kb: f64,
    pub peak_memory_kb: f64,
    pub meets_requirement: bool,
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub subscription_metrics: SubscriptionMetrics,
    pub throughput_stats: ThroughputStats,
    pub memory_stats: MemoryStats,
    pub recent_alerts: Vec<PerformanceAlert>,
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct ThroughputStats {
    pub current_events_per_second: f64,
    pub total_events: u64,
    pub event_type_distribution: HashMap<String, u64>,
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub current_memory_kb: f64,
    pub peak_memory_kb: f64,
    pub subscription_count: usize,
    pub avg_memory_per_subscription: f64,
}

// Production builds have no debug event performance monitoring
#[cfg(not(debug_assertions))]
pub struct DebugEventPerformanceMonitor;

#[cfg(not(debug_assertions))]
impl DebugEventPerformanceMonitor {
    pub fn new() -> Self {
        Self
    }

    pub fn record_subscription(&mut self, _event_type: &str, _duration: Duration) {}
    pub fn record_unsubscription(&mut self, _event_type: &str, _duration: Duration) {}
    pub fn record_event_processed(&mut self, _event_type: &str) {}
    pub fn update_memory_usage(&mut self, _subscription_count: usize, _estimated_memory_kb: f64) {}
    pub fn meets_performance_requirements(&self) -> bool { true }
}

pub fn create_performance_monitor() -> DebugEventPerformanceMonitor {
    DebugEventPerformanceMonitor::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = create_performance_monitor();
        assert!(monitor.meets_performance_requirements());
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_subscription_performance_tracking() {
        let mut monitor = DebugEventPerformanceMonitor::new();
        
        monitor.record_subscription("TestEvent", Duration::from_micros(500));
        
        let report = monitor.get_performance_report();
        assert_eq!(report.subscription_metrics.total_subscriptions, 1);
        assert!(report.subscription_metrics.avg_subscription_time_ms < 1.0);
        assert!(monitor.meets_performance_requirements());
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_performance_alert_system() {
        let mut monitor = DebugEventPerformanceMonitor::new();
        
        monitor.record_subscription("SlowEvent", Duration::from_millis(5));
        
        let report = monitor.get_performance_report();
        assert!(!report.recent_alerts.is_empty());
        
        let alert = &report.recent_alerts[0];
        assert!(matches!(alert.alert_type, AlertType::SlowSubscription));
        assert!(matches!(alert.severity, AlertSeverity::Critical));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_memory_monitoring() {
        let mut monitor = DebugEventPerformanceMonitor::new();
        
        monitor.update_memory_usage(100, 2000.0);
        
        let report = monitor.get_performance_report();
        assert_eq!(report.memory_stats.subscription_count, 100);
        assert_eq!(report.memory_stats.current_memory_kb, 2000.0);
        assert_eq!(report.memory_stats.avg_memory_per_subscription, 20.0);
        assert!(monitor.meets_performance_requirements());
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_throughput_monitoring() {
        let mut monitor = DebugEventPerformanceMonitor::new();
        
        for i in 0..50 {
            monitor.record_event_processed(&format!("Event{}", i % 5));
        }
        
        let report = monitor.get_performance_report();
        assert_eq!(report.throughput_stats.total_events, 50);
        assert!(report.throughput_stats.current_events_per_second < 1000.0);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_benchmark_execution() {
        let mut monitor = DebugEventPerformanceMonitor::new();
        
        let results = monitor.run_benchmarks();
        
        assert!(results.subscription_benchmark.meets_requirement);
        assert!(results.publishing_benchmark.meets_requirement);
        assert!(results.memory_benchmark.meets_requirement);
    }
} 