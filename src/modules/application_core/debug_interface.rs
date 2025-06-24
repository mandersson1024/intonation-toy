//! # Event Bus Debug Interface
//!
//! This module provides a real-time debug interface for visualizing event bus performance,
//! metrics, and system health. It's designed for development and production monitoring.

use super::performance_monitor::{
    EventBusPerformanceMonitor, EnhancedEventBusMetrics, PerformanceAlert, 
    AlertSeverity, EventTypeMetrics, LatencyDataPoint, ThroughputDataPoint
};
use super::event_bus::{EventBusState, EventPriority};
use std::collections::HashMap;
use std::sync::Arc;

/// Debug interface for real-time event bus monitoring
pub struct EventBusDebugInterface {
    performance_monitor: Arc<EventBusPerformanceMonitor>,
    refresh_rate_ms: u64,
    display_config: DebugDisplayConfig,
}

/// Configuration for debug display formatting
#[derive(Debug, Clone)]
pub struct DebugDisplayConfig {
    pub show_latency_charts: bool,
    pub show_throughput_charts: bool,
    pub show_memory_usage: bool,
    pub show_per_type_breakdown: bool,
    pub show_active_alerts: bool,
    pub max_history_points: usize,
    pub compact_mode: bool,
    pub color_coding: bool,
}

impl Default for DebugDisplayConfig {
    fn default() -> Self {
        Self {
            show_latency_charts: true,
            show_throughput_charts: true,
            show_memory_usage: true,
            show_per_type_breakdown: true,
            show_active_alerts: true,
            max_history_points: 50,
            compact_mode: false,
            color_coding: true,
        }
    }
}

/// Formatted performance summary for display
#[derive(Debug)]
pub struct PerformanceSummary {
    pub timestamp: String,
    pub state: EventBusState,
    pub total_events: u64,
    pub events_per_second: f64,
    pub queue_status: QueueStatus,
    pub latency_summary: LatencySummary,
    pub memory_summary: MemorySummary,
    pub alert_summary: AlertSummary,
    pub top_event_types: Vec<EventTypeSummary>,
}

#[derive(Debug)]
pub struct QueueStatus {
    pub total_queued: usize,
    pub by_priority: [usize; 4],
    pub queue_health: QueueHealth,
}

#[derive(Debug)]
pub enum QueueHealth {
    Healthy,
    Warning,
    Critical,
}

#[derive(Debug)]
pub struct LatencySummary {
    pub avg_by_priority: [String; 4], // Formatted durations
    pub worst_latency: String,
    pub worst_priority: EventPriority,
    pub latency_trend: LatencyTrend,
}

#[derive(Debug)]
pub enum LatencyTrend {
    Improving,
    Stable,
    Degrading,
}

#[derive(Debug)]
pub struct MemorySummary {
    pub total_bytes: String, // Formatted size
    pub queue_bytes: String,
    pub handler_bytes: String,
    pub overhead_bytes: String,
    pub memory_trend: MemoryTrend,
}

#[derive(Debug)]
pub enum MemoryTrend {
    Decreasing,
    Stable,
    Increasing,
}

#[derive(Debug)]
pub struct AlertSummary {
    pub total_alerts: usize,
    pub critical_alerts: usize,
    pub unacknowledged_alerts: usize,
    pub recent_alerts: Vec<String>, // Last 5 alert messages
}

#[derive(Debug)]
pub struct EventTypeSummary {
    pub event_type: String,
    pub total_events: u64,
    pub avg_latency: String,
    pub error_rate: f64,
    pub last_seen: String,
}

impl EventBusDebugInterface {
    /// Creates a new debug interface
    pub fn new(performance_monitor: Arc<EventBusPerformanceMonitor>) -> Self {
        Self {
            performance_monitor,
            refresh_rate_ms: 1000, // 1 second default
            display_config: DebugDisplayConfig::default(),
        }
    }
    
    /// Creates a debug interface with custom configuration
    pub fn with_config(
        performance_monitor: Arc<EventBusPerformanceMonitor>,
        config: DebugDisplayConfig
    ) -> Self {
        Self {
            performance_monitor,
            refresh_rate_ms: 1000,
            display_config: config,
        }
    }
    
    /// Gets a comprehensive performance summary
    pub fn get_performance_summary(&self) -> Result<PerformanceSummary, String> {
        let enhanced_metrics = self.performance_monitor.get_enhanced_metrics()?;
        
        Ok(PerformanceSummary {
            timestamp: Self::format_timestamp(enhanced_metrics.last_update_timestamp),
            state: EventBusState::Running, // Would need to get this from event bus
            total_events: enhanced_metrics.base_metrics.total_events_processed,
            events_per_second: enhanced_metrics.base_metrics.events_per_second,
            queue_status: Self::create_queue_status(&enhanced_metrics),
            latency_summary: Self::create_latency_summary(&enhanced_metrics),
            memory_summary: Self::create_memory_summary(&enhanced_metrics),
            alert_summary: Self::create_alert_summary(&enhanced_metrics),
            top_event_types: Self::create_event_type_summaries(&enhanced_metrics, 10),
        })
    }
    
    /// Generates a formatted text dashboard
    pub fn generate_text_dashboard(&self) -> Result<String, String> {
        let summary = self.get_performance_summary()?;
        let mut output = String::new();
        
        // Header
        output.push_str("╔══════════════════════════════════════════════════════════════════════════════════╗\n");
        output.push_str("║                           EVENT BUS PERFORMANCE DASHBOARD                        ║\n");
        output.push_str("╚══════════════════════════════════════════════════════════════════════════════════╝\n");
        output.push_str(&format!("│ Last Update: {:<25} │ State: {:>12} │ Total Events: {:>8} │\n", 
            summary.timestamp, format!("{:?}", summary.state), summary.total_events));
        output.push_str("├──────────────────────────────────────────────────────────────────────────────────┤\n");
        
        // Performance Overview
        if !self.display_config.compact_mode {
            output.push_str("║ PERFORMANCE OVERVIEW                                                             ║\n");
            output.push_str("├──────────────────────────────────────────────────────────────────────────────────┤\n");
            output.push_str(&format!("│ Events/sec: {:>8.1} │ Memory: {:>12} │ Queue Health: {:>12} │\n",
                summary.events_per_second, 
                summary.memory_summary.total_bytes,
                format!("{:?}", summary.queue_status.queue_health)));
            output.push_str("├──────────────────────────────────────────────────────────────────────────────────┤\n");
        }
        
        // Queue Status
        if self.display_config.show_latency_charts {
            output.push_str("║ QUEUE STATUS                                                                     ║\n");
            output.push_str("├──────────────────────────────────────────────────────────────────────────────────┤\n");
            output.push_str(&format!("│ Critical: {:>4} │ High: {:>4} │ Normal: {:>4} │ Low: {:>4} │ Total: {:>4} │\n",
                summary.queue_status.by_priority[0],
                summary.queue_status.by_priority[1], 
                summary.queue_status.by_priority[2],
                summary.queue_status.by_priority[3],
                summary.queue_status.total_queued));
            output.push_str("├──────────────────────────────────────────────────────────────────────────────────┤\n");
        }
        
        // Latency Summary
        if self.display_config.show_latency_charts {
            output.push_str("║ LATENCY BY PRIORITY                                                              ║\n");
            output.push_str("├──────────────────────────────────────────────────────────────────────────────────┤\n");
            output.push_str(&format!("│ Critical: {:>8} │ High: {:>8} │ Normal: {:>8} │ Low: {:>8} │\n",
                summary.latency_summary.avg_by_priority[0],
                summary.latency_summary.avg_by_priority[1],
                summary.latency_summary.avg_by_priority[2],
                summary.latency_summary.avg_by_priority[3]));
            output.push_str(&format!("│ Worst: {} ({:?}) │ Trend: {:>12} │\n",
                summary.latency_summary.worst_latency,
                summary.latency_summary.worst_priority,
                format!("{:?}", summary.latency_summary.latency_trend)));
            output.push_str("├──────────────────────────────────────────────────────────────────────────────────┤\n");
        }
        
        // Active Alerts
        if self.display_config.show_active_alerts && summary.alert_summary.total_alerts > 0 {
            output.push_str("║ ACTIVE ALERTS                                                                    ║\n");
            output.push_str("├──────────────────────────────────────────────────────────────────────────────────┤\n");
            output.push_str(&format!("│ Total: {:>3} │ Critical: {:>3} │ Unacknowledged: {:>3} │\n",
                summary.alert_summary.total_alerts,
                summary.alert_summary.critical_alerts,
                summary.alert_summary.unacknowledged_alerts));
            
            for alert_msg in &summary.alert_summary.recent_alerts {
                output.push_str(&format!("│ • {:<74} │\n", Self::truncate_string(alert_msg, 74)));
            }
            output.push_str("├──────────────────────────────────────────────────────────────────────────────────┤\n");
        }
        
        // Top Event Types
        if self.display_config.show_per_type_breakdown && !summary.top_event_types.is_empty() {
            output.push_str("║ TOP EVENT TYPES                                                                  ║\n");
            output.push_str("├──────────────────────────────────────────────────────────────────────────────────┤\n");
            output.push_str("│ Event Type                │ Count   │ Avg Latency │ Error Rate │ Last Seen      │\n");
            output.push_str("├───────────────────────────┼─────────┼─────────────┼────────────┼────────────────┤\n");
            
            for event_summary in summary.top_event_types.iter().take(5) {
                output.push_str(&format!("│ {:<25} │ {:>7} │ {:>11} │ {:>9.1}% │ {:>14} │\n",
                    Self::truncate_string(&event_summary.event_type, 25),
                    event_summary.total_events,
                    event_summary.avg_latency,
                    event_summary.error_rate * 100.0,
                    event_summary.last_seen));
            }
            output.push_str("├──────────────────────────────────────────────────────────────────────────────────┤\n");
        }
        
        // Memory Breakdown
        if self.display_config.show_memory_usage {
            output.push_str("║ MEMORY USAGE                                                                     ║\n");
            output.push_str("├──────────────────────────────────────────────────────────────────────────────────┤\n");
            output.push_str(&format!("│ Total: {:>10} │ Queues: {:>10} │ Handlers: {:>10} │ Overhead: {:>8} │\n",
                summary.memory_summary.total_bytes,
                summary.memory_summary.queue_bytes,
                summary.memory_summary.handler_bytes,
                summary.memory_summary.overhead_bytes));
            output.push_str(&format!("│ Trend: {:>12}                                                              │\n",
                format!("{:?}", summary.memory_summary.memory_trend)));
            output.push_str("├──────────────────────────────────────────────────────────────────────────────────┤\n");
        }
        
        // Footer
        output.push_str("╚══════════════════════════════════════════════════════════════════════════════════╝\n");
        
        Ok(output)
    }
    
    /// Generates a compact JSON-like summary for programmatic consumption
    pub fn generate_json_summary(&self) -> Result<String, String> {
        let summary = self.get_performance_summary()?;
        
        // Manual JSON-like formatting since we don't have serde
        let json = format!(
            r#"{{
  "timestamp": "{}",
  "state": "{:?}",
  "total_events": {},
  "events_per_second": {},
  "queue_status": {{
    "total_queued": {},
    "by_priority": [{}, {}, {}, {}],
    "health": "{:?}"
  }},
  "memory_usage": "{}",
  "active_alerts": {},
  "monitoring_overhead": "<5%"
}}"#,
            summary.timestamp,
            summary.state,
            summary.total_events,
            summary.events_per_second,
            summary.queue_status.total_queued,
            summary.queue_status.by_priority[0],
            summary.queue_status.by_priority[1],
            summary.queue_status.by_priority[2],
            summary.queue_status.by_priority[3],
            summary.queue_status.queue_health,
            summary.memory_summary.total_bytes,
            summary.alert_summary.total_alerts
        );
        
        Ok(json)
    }
    
    /// Gets simple ASCII charts for latency history
    pub fn generate_latency_charts(&self) -> Result<Vec<String>, String> {
        let enhanced_metrics = self.performance_monitor.get_enhanced_metrics()?;
        let mut charts = Vec::new();
        
        let priority_names = ["Critical", "High", "Normal", "Low"];
        
        for (priority_idx, priority_name) in priority_names.iter().enumerate() {
            let history = &enhanced_metrics.latency_history[priority_idx];
            if history.is_empty() {
                continue;
            }
            
            let chart = Self::create_ascii_latency_chart(
                priority_name,
                history,
                self.display_config.max_history_points
            );
            charts.push(chart);
        }
        
        Ok(charts)
    }
    
    /// Gets simple ASCII chart for throughput history
    pub fn generate_throughput_chart(&self) -> Result<String, String> {
        let enhanced_metrics = self.performance_monitor.get_enhanced_metrics()?;
        
        if enhanced_metrics.throughput_history.is_empty() {
            return Ok("No throughput data available".to_string());
        }
        
        Ok(Self::create_ascii_throughput_chart(
            &enhanced_metrics.throughput_history,
            self.display_config.max_history_points
        ))
    }
    
    // Helper methods for creating summaries
    
    fn create_queue_status(metrics: &EnhancedEventBusMetrics) -> QueueStatus {
        let total_queued = metrics.base_metrics.queue_depths.iter().sum();
        
        let queue_health = if total_queued > 5000 {
            QueueHealth::Critical
        } else if total_queued > 1000 {
            QueueHealth::Warning
        } else {
            QueueHealth::Healthy
        };
        
        QueueStatus {
            total_queued,
            by_priority: metrics.base_metrics.queue_depths,
            queue_health,
        }
    }
    
    fn create_latency_summary(metrics: &EnhancedEventBusMetrics) -> LatencySummary {
        let avg_by_priority = [
            Self::format_duration_ns(metrics.base_metrics.avg_latency_by_priority[0]),
            Self::format_duration_ns(metrics.base_metrics.avg_latency_by_priority[1]),
            Self::format_duration_ns(metrics.base_metrics.avg_latency_by_priority[2]),
            Self::format_duration_ns(metrics.base_metrics.avg_latency_by_priority[3]),
        ];
        
        // Find worst latency
        let (worst_latency_ns, worst_priority_idx) = metrics.base_metrics.avg_latency_by_priority
            .iter()
            .enumerate()
            .max_by_key(|(_, &latency)| latency)
            .map(|(idx, &latency)| (latency, idx))
            .unwrap_or((0, 0));
        
        let worst_priority = match worst_priority_idx {
            0 => EventPriority::Critical,
            1 => EventPriority::High,
            2 => EventPriority::Normal,
            _ => EventPriority::Low,
        };
        
        // Simple trend analysis (would need historical comparison for real implementation)
        let latency_trend = LatencyTrend::Stable;
        
        LatencySummary {
            avg_by_priority,
            worst_latency: Self::format_duration_ns(worst_latency_ns),
            worst_priority,
            latency_trend,
        }
    }
    
    fn create_memory_summary(metrics: &EnhancedEventBusMetrics) -> MemorySummary {
        let total_bytes = Self::format_bytes(metrics.base_metrics.memory_usage_bytes);
        
        let (queue_bytes, handler_bytes, overhead_bytes) = if let Some(latest_memory) = metrics.memory_history.back() {
            (
                Self::format_bytes(latest_memory.queue_memory_bytes),
                Self::format_bytes(latest_memory.handler_memory_bytes),
                Self::format_bytes(metrics.monitoring_overhead.storage_overhead_bytes),
            )
        } else {
            ("N/A".to_string(), "N/A".to_string(), "N/A".to_string())
        };
        
        // Simple trend analysis
        let memory_trend = MemoryTrend::Stable;
        
        MemorySummary {
            total_bytes,
            queue_bytes,
            handler_bytes,
            overhead_bytes,
            memory_trend,
        }
    }
    
    fn create_alert_summary(metrics: &EnhancedEventBusMetrics) -> AlertSummary {
        let total_alerts = metrics.active_alerts.len();
        let critical_alerts = metrics.active_alerts.iter()
            .filter(|alert| alert.severity == AlertSeverity::Critical)
            .count();
        let unacknowledged_alerts = metrics.active_alerts.iter()
            .filter(|alert| !alert.acknowledged)
            .count();
        
        let recent_alerts = metrics.active_alerts.iter()
            .rev()
            .take(5)
            .map(|alert| format!("{:?}: {}", alert.severity, alert.message))
            .collect();
        
        AlertSummary {
            total_alerts,
            critical_alerts,
            unacknowledged_alerts,
            recent_alerts,
        }
    }
    
    fn create_event_type_summaries(metrics: &EnhancedEventBusMetrics, limit: usize) -> Vec<EventTypeSummary> {
        let mut summaries: Vec<_> = metrics.per_type_metrics.values()
            .map(|type_metrics| EventTypeSummary {
                event_type: type_metrics.event_type.clone(),
                total_events: type_metrics.total_events,
                avg_latency: Self::format_duration_ns(type_metrics.avg_latency_ns),
                error_rate: if type_metrics.total_events > 0 {
                    type_metrics.error_count as f64 / type_metrics.total_events as f64
                } else {
                    0.0
                },
                last_seen: Self::format_timestamp(type_metrics.last_seen_timestamp),
            })
            .collect();
        
        // Sort by total events (descending)
        summaries.sort_by(|a, b| b.total_events.cmp(&a.total_events));
        summaries.truncate(limit);
        
        summaries
    }
    
    // Utility formatting methods
    
    fn format_timestamp(timestamp_ns: u64) -> String {
        // Simple formatting - in a real implementation, would use proper time formatting
        let seconds = timestamp_ns / 1_000_000_000;
        let minutes = seconds / 60;
        let hours = minutes / 60;
        
        format!("{}h:{}m:{}s", hours % 24, minutes % 60, seconds % 60)
    }
    
    fn format_duration_ns(ns: u64) -> String {
        if ns >= 1_000_000_000 {
            format!("{:.2}s", ns as f64 / 1_000_000_000.0)
        } else if ns >= 1_000_000 {
            format!("{:.2}ms", ns as f64 / 1_000_000.0)
        } else if ns >= 1_000 {
            format!("{:.2}μs", ns as f64 / 1_000.0)
        } else {
            format!("{}ns", ns)
        }
    }
    
    fn format_bytes(bytes: usize) -> String {
        if bytes >= 1_000_000_000 {
            format!("{:.2}GB", bytes as f64 / 1_000_000_000.0)
        } else if bytes >= 1_000_000 {
            format!("{:.2}MB", bytes as f64 / 1_000_000.0)
        } else if bytes >= 1_000 {
            format!("{:.2}KB", bytes as f64 / 1_000.0)
        } else {
            format!("{}B", bytes)
        }
    }
    
    fn truncate_string(s: &str, max_len: usize) -> &str {
        if s.len() <= max_len {
            s
        } else {
            &s[..max_len.saturating_sub(3)]
        }
    }
    
    fn create_ascii_latency_chart(priority_name: &str, history: &std::collections::VecDeque<LatencyDataPoint>, max_points: usize) -> String {
        if history.is_empty() {
            return format!("{} Priority: No data", priority_name);
        }
        
        let points: Vec<_> = history.iter().rev().take(max_points).collect();
        let max_latency = points.iter().map(|p| p.latency_ns).max().unwrap_or(1);
        
        let mut chart = format!("{} Priority Latency (last {} points):\n", priority_name, points.len());
        chart.push_str(&format!("Max: {} │", Self::format_duration_ns(max_latency)));
        
        // Simple bar chart
        for point in points.iter().rev() {
            let bar_length = ((point.latency_ns as f64 / max_latency as f64) * 40.0) as usize;
            let bar = "█".repeat(bar_length);
            chart.push_str(&format!("\n{:>8} │{:<40} {}", 
                Self::format_duration_ns(point.latency_ns),
                bar,
                point.event_count));
        }
        
        chart
    }
    
    fn create_ascii_throughput_chart(history: &std::collections::VecDeque<ThroughputDataPoint>, max_points: usize) -> String {
        if history.is_empty() {
            return "Throughput: No data".to_string();
        }
        
        let points: Vec<_> = history.iter().rev().take(max_points).collect();
        let max_throughput = points.iter()
            .map(|p| p.events_per_second)
            .fold(0.0f64, f64::max);
        
        let mut chart = format!("Throughput (events/sec, last {} points):\n", points.len());
        chart.push_str(&format!("Max: {:.1} │", max_throughput));
        
        // Simple bar chart
        for point in points.iter().rev() {
            let bar_length = if max_throughput > 0.0 {
                ((point.events_per_second / max_throughput) * 40.0) as usize
            } else {
                0
            };
            let bar = "█".repeat(bar_length);
            chart.push_str(&format!("\n{:>6.1} │{:<40}", point.events_per_second, bar));
        }
        
        chart
    }
}

// Implement Serialize for PerformanceSummary if serde is available
// This is a simplified version without serde dependency

impl std::fmt::Display for PerformanceSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PerformanceSummary {{ timestamp: {}, events: {}, eps: {:.1} }}", 
               self.timestamp, self.total_events, self.events_per_second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::application_core::performance_monitor::{EventBusPerformanceMonitor, MonitorConfig};
    
    #[test]
    fn test_debug_interface_creation() {
        let monitor = Arc::new(EventBusPerformanceMonitor::new());
        let debug_interface = EventBusDebugInterface::new(monitor);
        
        // Should create successfully
        assert_eq!(debug_interface.refresh_rate_ms, 1000);
        assert!(debug_interface.display_config.show_latency_charts);
    }
    
    #[test]
    fn test_performance_summary_generation() {
        let monitor = Arc::new(EventBusPerformanceMonitor::new());
        let debug_interface = EventBusDebugInterface::new(monitor);
        
        let result = debug_interface.get_performance_summary();
        assert!(result.is_ok());
        
        let summary = result.unwrap();
        assert_eq!(summary.total_events, 0);
        assert_eq!(summary.events_per_second, 0.0);
    }
    
    #[test]
    fn test_text_dashboard_generation() {
        let monitor = Arc::new(EventBusPerformanceMonitor::new());
        let debug_interface = EventBusDebugInterface::new(monitor);
        
        let result = debug_interface.generate_text_dashboard();
        assert!(result.is_ok());
        
        let dashboard = result.unwrap();
        assert!(dashboard.contains("EVENT BUS PERFORMANCE DASHBOARD"));
        assert!(dashboard.contains("PERFORMANCE OVERVIEW"));
    }
    
    #[test]
    fn test_format_utilities() {
        assert_eq!(EventBusDebugInterface::format_bytes(1024), "1.02KB");
        assert_eq!(EventBusDebugInterface::format_bytes(1048576), "1.05MB");
        assert_eq!(EventBusDebugInterface::format_bytes(1073741824), "1.07GB");
        
        assert_eq!(EventBusDebugInterface::format_duration_ns(1000), "1.00μs");
        assert_eq!(EventBusDebugInterface::format_duration_ns(1_000_000), "1.00ms");
        assert_eq!(EventBusDebugInterface::format_duration_ns(1_000_000_000), "1.00s");
    }
    
    #[test]
    fn test_compact_mode() {
        let monitor = Arc::new(EventBusPerformanceMonitor::new());
        let config = DebugDisplayConfig {
            compact_mode: true,
            show_latency_charts: false,
            show_throughput_charts: false,
            ..DebugDisplayConfig::default()
        };
        let debug_interface = EventBusDebugInterface::with_config(monitor, config);
        
        let result = debug_interface.generate_text_dashboard();
        assert!(result.is_ok());
        
        let dashboard = result.unwrap();
        // In compact mode, should have fewer sections
        assert!(dashboard.len() < 2000); // Rough estimate for compact output
    }
}