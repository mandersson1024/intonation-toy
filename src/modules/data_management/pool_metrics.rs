//! # Pool Metrics Collection and Monitoring
//!
//! This module provides real-time pool efficiency monitoring with <90% hit rate alerting,
//! memory overhead tracking (<3% overhead target), and JavaScript GC pressure reduction measurement.

use std::sync::{Arc, RwLock, Mutex};
use std::collections::HashMap;
use std::time::{Instant, Duration};
use super::buffer_recycling_pool::{PoolMetrics, PoolError};

/// Pool metrics monitor with real-time efficiency tracking
pub struct PoolMetricsMonitor {
    /// Current metrics snapshot
    current_metrics: Arc<RwLock<PoolMetrics>>,
    /// Historical metrics for trend analysis
    metrics_history: Arc<Mutex<Vec<MetricsSnapshot>>>,
    /// Alert thresholds
    alert_config: AlertConfig,
    /// Performance tracking
    monitoring_start_time: Instant,
    /// Metrics update interval
    update_interval: Duration,
}

/// Configuration for performance alerting
#[derive(Debug, Clone)]
pub struct AlertConfig {
    /// Minimum hit rate threshold (default: 90%)
    pub min_hit_rate_percent: f32,
    /// Maximum memory overhead threshold (default: 3%)
    pub max_memory_overhead_percent: f32,
    /// Maximum fragmentation threshold (default: 5%)
    pub max_fragmentation_percent: f32,
    /// Minimum GC pressure reduction target (default: 10%)
    pub min_gc_pressure_reduction_percent: f32,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            min_hit_rate_percent: 90.0,
            max_memory_overhead_percent: 3.0,
            max_fragmentation_percent: 5.0,
            min_gc_pressure_reduction_percent: 10.0,
        }
    }
}

/// Metrics snapshot with timestamp
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub metrics: PoolMetrics,
    pub timestamp: Instant,
    pub alerts: Vec<PerformanceAlert>,
}

/// Performance alert types
#[derive(Debug, Clone)]
pub enum PerformanceAlert {
    LowHitRate {
        actual: f32,
        threshold: f32,
        severity: AlertSeverity,
    },
    HighMemoryOverhead {
        actual: f32,
        threshold: f32,
        severity: AlertSeverity,
    },
    HighFragmentation {
        actual: f32,
        threshold: f32,
        severity: AlertSeverity,
    },
    LowGCPressureReduction {
        actual: f32,
        threshold: f32,
        severity: AlertSeverity,
    },
}

#[derive(Debug, Clone)]
pub enum AlertSeverity {
    Warning,
    Critical,
}

impl PoolMetricsMonitor {
    /// Create a new pool metrics monitor
    pub fn new() -> Self {
        Self {
            current_metrics: Arc::new(RwLock::new(PoolMetrics::default())),
            metrics_history: Arc::new(Mutex::new(Vec::new())),
            alert_config: AlertConfig::default(),
            monitoring_start_time: Instant::now(),
            update_interval: Duration::from_millis(250), // 4 times per second
        }
    }
    
    /// Create monitor with custom alert configuration
    pub fn with_alert_config(alert_config: AlertConfig) -> Self {
        Self {
            current_metrics: Arc::new(RwLock::new(PoolMetrics::default())),
            metrics_history: Arc::new(Mutex::new(Vec::new())),
            alert_config,
            monitoring_start_time: Instant::now(),
            update_interval: Duration::from_millis(250),
        }
    }
    
    /// Update metrics and check for alerts
    pub fn update_metrics(&self, new_metrics: PoolMetrics) -> Result<Vec<PerformanceAlert>, PoolError> {
        // Update current metrics
        {
            let mut current = self.current_metrics.write()
                .map_err(|_| PoolError::Internal("Failed to update current metrics".to_string()))?;
            *current = new_metrics.clone();
        }
        
        // Check for performance alerts
        let alerts = self.check_performance_alerts(&new_metrics);
        
        // Add to history
        {
            let mut history = self.metrics_history.lock()
                .map_err(|_| PoolError::Internal("Failed to update metrics history".to_string()))?;
            
            let snapshot = MetricsSnapshot {
                metrics: new_metrics,
                timestamp: Instant::now(),
                alerts: alerts.clone(),
            };
            
            history.push(snapshot);
            
            // Keep only last 1000 snapshots to prevent memory growth
            if history.len() > 1000 {
                history.remove(0);
            }
        }
        
        Ok(alerts)
    }
    
    /// Check for performance alerts based on current metrics
    fn check_performance_alerts(&self, metrics: &PoolMetrics) -> Vec<PerformanceAlert> {
        let mut alerts = Vec::new();
        
        // Check hit rate
        if metrics.hit_rate_percentage < self.alert_config.min_hit_rate_percent {
            let severity = if metrics.hit_rate_percentage < 70.0 {
                AlertSeverity::Critical
            } else {
                AlertSeverity::Warning
            };
            
            alerts.push(PerformanceAlert::LowHitRate {
                actual: metrics.hit_rate_percentage,
                threshold: self.alert_config.min_hit_rate_percent,
                severity,
            });
        }
        
        // Check memory overhead
        let memory_overhead_percent = if metrics.total_allocations > 0 {
            (metrics.memory_overhead_bytes as f32 / (metrics.total_allocations * 4096) as f32) * 100.0
        } else {
            0.0
        };
        
        if memory_overhead_percent > self.alert_config.max_memory_overhead_percent {
            let severity = if memory_overhead_percent > 5.0 {
                AlertSeverity::Critical
            } else {
                AlertSeverity::Warning
            };
            
            alerts.push(PerformanceAlert::HighMemoryOverhead {
                actual: memory_overhead_percent,
                threshold: self.alert_config.max_memory_overhead_percent,
                severity,
            });
        }
        
        // Check fragmentation
        if metrics.fragmentation_percentage > self.alert_config.max_fragmentation_percent {
            let severity = if metrics.fragmentation_percentage > 10.0 {
                AlertSeverity::Critical
            } else {
                AlertSeverity::Warning
            };
            
            alerts.push(PerformanceAlert::HighFragmentation {
                actual: metrics.fragmentation_percentage,
                threshold: self.alert_config.max_fragmentation_percent,
                severity,
            });
        }
        
        // Check GC pressure reduction
        if metrics.js_gc_pressure_reduction < self.alert_config.min_gc_pressure_reduction_percent {
            alerts.push(PerformanceAlert::LowGCPressureReduction {
                actual: metrics.js_gc_pressure_reduction,
                threshold: self.alert_config.min_gc_pressure_reduction_percent,
                severity: AlertSeverity::Warning,
            });
        }
        
        alerts
    }
    
    /// Get current metrics snapshot
    pub fn get_current_metrics(&self) -> Result<PoolMetrics, PoolError> {
        self.current_metrics.read()
            .map(|metrics| metrics.clone())
            .map_err(|_| PoolError::Internal("Failed to read current metrics".to_string()))
    }
    
    /// Get efficiency report with trend analysis
    pub fn get_efficiency_report(&self) -> Result<PoolEfficiencyReport, PoolError> {
        let current = self.get_current_metrics()?;
        
        let history = self.metrics_history.lock()
            .map_err(|_| PoolError::Internal("Failed to read metrics history".to_string()))?;
        
        let runtime_duration = self.monitoring_start_time.elapsed();
        
        // Calculate trends from recent history
        let recent_snapshots: Vec<_> = history.iter()
            .rev()
            .take(20) // Last 20 snapshots (5 seconds at 250ms intervals)
            .collect();
        
        let hit_rate_trend = if recent_snapshots.len() >= 2 {
            let first = recent_snapshots.last().unwrap().metrics.hit_rate_percentage;
            let last = recent_snapshots.first().unwrap().metrics.hit_rate_percentage;
            last - first
        } else {
            0.0
        };
        
        let fragmentation_trend = if recent_snapshots.len() >= 2 {
            let first = recent_snapshots.last().unwrap().metrics.fragmentation_percentage;
            let last = recent_snapshots.first().unwrap().metrics.fragmentation_percentage;
            last - first
        } else {
            0.0
        };
        
        // Count recent alerts
        let recent_alerts = recent_snapshots.iter()
            .flat_map(|snapshot| &snapshot.alerts)
            .count();
        
        Ok(PoolEfficiencyReport {
            current_metrics: current,
            runtime_seconds: runtime_duration.as_secs(),
            hit_rate_trend,
            fragmentation_trend,
            recent_alert_count: recent_alerts,
            total_snapshots: history.len(),
            meets_performance_targets: self.meets_performance_targets(&current),
        })
    }
    
    /// Check if current metrics meet all performance targets
    fn meets_performance_targets(&self, metrics: &PoolMetrics) -> bool {
        metrics.hit_rate_percentage >= self.alert_config.min_hit_rate_percent &&
        metrics.fragmentation_percentage <= self.alert_config.max_fragmentation_percent &&
        metrics.js_gc_pressure_reduction >= self.alert_config.min_gc_pressure_reduction_percent
    }
    
    /// Get metrics visualization data for development debugging
    pub fn get_visualization_data(&self) -> Result<MetricsVisualizationData, PoolError> {
        let history = self.metrics_history.lock()
            .map_err(|_| PoolError::Internal("Failed to read metrics history".to_string()))?;
        
        let hit_rates: Vec<f32> = history.iter()
            .map(|snapshot| snapshot.metrics.hit_rate_percentage)
            .collect();
        
        let fragmentation_levels: Vec<f32> = history.iter()
            .map(|snapshot| snapshot.metrics.fragmentation_percentage)
            .collect();
        
        let gc_pressure_reductions: Vec<f32> = history.iter()
            .map(|snapshot| snapshot.metrics.js_gc_pressure_reduction)
            .collect();
        
        let timestamps: Vec<u64> = history.iter()
            .map(|snapshot| snapshot.timestamp.elapsed().as_millis() as u64)
            .collect();
        
        Ok(MetricsVisualizationData {
            hit_rates,
            fragmentation_levels,
            gc_pressure_reductions,
            timestamps,
            alert_thresholds: self.alert_config.clone(),
        })
    }
}

/// Comprehensive efficiency report with trend analysis
#[derive(Debug, Clone)]
pub struct PoolEfficiencyReport {
    pub current_metrics: PoolMetrics,
    pub runtime_seconds: u64,
    pub hit_rate_trend: f32, // Positive = improving
    pub fragmentation_trend: f32, // Negative = improving
    pub recent_alert_count: usize,
    pub total_snapshots: usize,
    pub meets_performance_targets: bool,
}

/// Data structure for metrics visualization
#[derive(Debug, Clone)]
pub struct MetricsVisualizationData {
    pub hit_rates: Vec<f32>,
    pub fragmentation_levels: Vec<f32>,
    pub gc_pressure_reductions: Vec<f32>,
    pub timestamps: Vec<u64>,
    pub alert_thresholds: AlertConfig,
}

impl Default for PoolMetricsMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_monitor_creation() {
        let monitor = PoolMetricsMonitor::new();
        let metrics = monitor.get_current_metrics().unwrap();
        assert_eq!(metrics.hit_rate_percentage, 0.0);
    }
    
    #[test]
    fn test_performance_alerts() {
        let monitor = PoolMetricsMonitor::new();
        
        let poor_metrics = PoolMetrics {
            hit_rate_percentage: 50.0, // Below 90% threshold
            fragmentation_percentage: 8.0, // Above 5% threshold
            js_gc_pressure_reduction: 5.0, // Below 10% threshold
            ..Default::default()
        };
        
        let alerts = monitor.update_metrics(poor_metrics).unwrap();
        assert!(!alerts.is_empty());
        
        // Should have alerts for low hit rate, high fragmentation, and low GC pressure reduction
        assert!(alerts.len() >= 3);
    }
    
    #[test]
    fn test_efficiency_report() {
        let monitor = PoolMetricsMonitor::new();
        
        // Add some test metrics
        let test_metrics = PoolMetrics {
            hit_rate_percentage: 95.0,
            fragmentation_percentage: 2.0,
            js_gc_pressure_reduction: 15.0,
            total_allocations: 1000,
            pool_hits: 950,
            pool_misses: 50,
            ..Default::default()
        };
        
        monitor.update_metrics(test_metrics).unwrap();
        
        let report = monitor.get_efficiency_report().unwrap();
        assert!(report.meets_performance_targets);
        assert_eq!(report.current_metrics.hit_rate_percentage, 95.0);
    }
}