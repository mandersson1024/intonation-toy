//! # Buffer Utilization Monitor Implementation
//!
//! This module provides real-time buffer utilization monitoring, metrics collection,
//! performance optimization recommendations, and memory efficiency tracking.
//! It enables buffer utilization visualization for development debugging.

use std::sync::{Arc, RwLock, Mutex};
use std::collections::VecDeque;
use std::time::{Instant, Duration};
use std::thread;

/// Buffer utilization monitor for real-time tracking
pub struct BufferUtilizationMonitor {
    /// Monitoring configuration
    config: MonitorConfig,
    /// Real-time metrics
    metrics: Arc<RwLock<UtilizationMetrics>>,
    /// Historical data for trends
    history: Arc<RwLock<VecDeque<UtilizationSnapshot>>>,
    /// Monitoring state
    state: Arc<Mutex<MonitorState>>,
    /// Monitoring interval
    interval_ms: u64,
}

/// Monitor configuration
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// Maximum history entries to keep
    pub max_history_entries: usize,
    /// Alert threshold for memory usage (percentage)
    pub memory_alert_threshold: f64,
    /// Alert threshold for allocation time (milliseconds)
    pub allocation_time_alert_threshold: f64,
    /// Enable performance recommendations
    pub recommendations_enabled: bool,
    /// Enable visualization data collection
    pub visualization_enabled: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            max_history_entries: 1000,
            memory_alert_threshold: 85.0,
            allocation_time_alert_threshold: 1.0,
            recommendations_enabled: true,
            visualization_enabled: true,
        }
    }
}

/// Real-time utilization metrics
#[derive(Debug, Clone)]
pub struct UtilizationMetrics {
    /// Current buffer count
    pub current_buffer_count: usize,
    /// Total memory usage in bytes
    pub total_memory_bytes: usize,
    /// Memory utilization percentage
    pub memory_utilization_percent: f64,
    /// Average allocation time in milliseconds
    pub avg_allocation_time_ms: f64,
    /// Maximum allocation time in milliseconds
    pub max_allocation_time_ms: f64,
    /// Buffer creation rate (buffers per second)
    pub creation_rate_bps: f64,
    /// Buffer cleanup rate (buffers per second)
    pub cleanup_rate_bps: f64,
    /// Pool hit rate percentage
    pub pool_hit_rate_percent: f64,
    /// Memory efficiency percentage
    pub memory_efficiency_percent: f64,
    /// Last update timestamp
    pub last_update: Instant,
}

impl Default for UtilizationMetrics {
    fn default() -> Self {
        Self {
            current_buffer_count: 0,
            total_memory_bytes: 0,
            memory_utilization_percent: 0.0,
            avg_allocation_time_ms: 0.0,
            max_allocation_time_ms: 0.0,
            creation_rate_bps: 0.0,
            cleanup_rate_bps: 0.0,
            pool_hit_rate_percent: 0.0,
            memory_efficiency_percent: 0.0,
            last_update: Instant::now(),
        }
    }
}

/// Historical utilization snapshot
#[derive(Debug, Clone)]
pub struct UtilizationSnapshot {
    /// Snapshot timestamp
    pub timestamp: Instant,
    /// Memory usage at snapshot
    pub memory_bytes: usize,
    /// Buffer count at snapshot
    pub buffer_count: usize,
    /// Allocation time at snapshot
    pub allocation_time_ms: f64,
    /// Pool hit rate at snapshot
    pub pool_hit_rate: f64,
}

/// Monitor state
#[derive(Debug, Clone, PartialEq)]
pub enum MonitorState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error(String),
}

/// Performance optimization recommendations
#[derive(Debug, Clone)]
pub struct OptimizationRecommendations {
    /// Recommended actions
    pub actions: Vec<RecommendationAction>,
    /// Priority level (1-5, 5 being highest)
    pub priority: u8,
    /// Expected performance improvement
    pub expected_improvement: String,
    /// Implementation complexity (Low/Medium/High)
    pub complexity: String,
}

/// Individual recommendation action
#[derive(Debug, Clone)]
pub struct RecommendationAction {
    /// Action description
    pub description: String,
    /// Action type
    pub action_type: ActionType,
    /// Expected impact
    pub impact: String,
}

/// Types of optimization actions
#[derive(Debug, Clone)]
pub enum ActionType {
    IncreasePoolSize,
    DecreasePoolSize,
    AdjustBufferSizes,
    EnableFragmentationPrevention,
    OptimizeAllocationStrategy,
    ReduceMemoryFootprint,
}

impl BufferUtilizationMonitor {
    /// Create a new buffer utilization monitor
    pub fn new(interval_ms: u64) -> Self {
        Self {
            config: MonitorConfig::default(),
            metrics: Arc::new(RwLock::new(UtilizationMetrics::default())),
            history: Arc::new(RwLock::new(VecDeque::new())),
            state: Arc::new(Mutex::new(MonitorState::Stopped)),
            interval_ms,
        }
    }

    /// Create monitor with custom configuration
    pub fn with_config(interval_ms: u64, config: MonitorConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(RwLock::new(UtilizationMetrics::default())),
            history: Arc::new(RwLock::new(VecDeque::new())),
            state: Arc::new(Mutex::new(MonitorState::Stopped)),
            interval_ms,
        }
    }

    /// Start monitoring
    pub fn start_monitoring(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut state) = self.state.lock() {
            if *state == MonitorState::Running {
                return Ok(()); // Already running
            }
            *state = MonitorState::Starting;
        }

        // In a real implementation, this would start a background thread
        // For now, we'll just mark as running
        if let Ok(mut state) = self.state.lock() {
            *state = MonitorState::Running;
        }

        Ok(())
    }

    /// Stop monitoring
    pub fn stop_monitoring(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut state) = self.state.lock() {
            *state = MonitorState::Stopping;
        }

        // Stop background monitoring thread
        // For now, just mark as stopped
        if let Ok(mut state) = self.state.lock() {
            *state = MonitorState::Stopped;
        }

        Ok(())
    }

    /// Update metrics with new data
    pub fn update_metrics(&self, buffer_count: usize, memory_bytes: usize, allocation_time_ms: f64, pool_hit_rate: f64) {
        let now = Instant::now();
        
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.current_buffer_count = buffer_count;
            metrics.total_memory_bytes = memory_bytes;
            metrics.avg_allocation_time_ms = allocation_time_ms;
            metrics.pool_hit_rate_percent = pool_hit_rate * 100.0;
            metrics.last_update = now;
            
            // Calculate memory utilization (assuming 256MB max)
            metrics.memory_utilization_percent = (memory_bytes as f64 / (256 * 1024 * 1024) as f64) * 100.0;
            
            // Update max allocation time
            if allocation_time_ms > metrics.max_allocation_time_ms {
                metrics.max_allocation_time_ms = allocation_time_ms;
            }
        }

        // Add to history if enabled
        if self.config.visualization_enabled {
            self.add_to_history(buffer_count, memory_bytes, allocation_time_ms, pool_hit_rate);
        }
    }

    /// Add snapshot to history
    fn add_to_history(&self, buffer_count: usize, memory_bytes: usize, allocation_time_ms: f64, pool_hit_rate: f64) {
        if let Ok(mut history) = self.history.write() {
            let snapshot = UtilizationSnapshot {
                timestamp: Instant::now(),
                memory_bytes,
                buffer_count,
                allocation_time_ms,
                pool_hit_rate,
            };
            
            history.push_back(snapshot);
            
            // Limit history size
            while history.len() > self.config.max_history_entries {
                history.pop_front();
            }
        }
    }

    /// Get current metrics
    pub fn get_current_metrics(&self) -> UtilizationMetrics {
        if let Ok(metrics) = self.metrics.read() {
            metrics.clone()
        } else {
            UtilizationMetrics::default()
        }
    }

    /// Get historical data for visualization
    pub fn get_history(&self) -> Vec<UtilizationSnapshot> {
        if let Ok(history) = self.history.read() {
            history.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Generate optimization recommendations
    pub fn generate_recommendations(&self) -> OptimizationRecommendations {
        let metrics = self.get_current_metrics();
        let mut actions = Vec::new();
        let mut priority = 1u8;

        // Memory usage recommendations
        if metrics.memory_utilization_percent > self.config.memory_alert_threshold {
            actions.push(RecommendationAction {
                description: "High memory usage detected. Consider reducing buffer pool size or enabling more aggressive cleanup.".to_string(),
                action_type: ActionType::ReduceMemoryFootprint,
                impact: "Reduce memory usage by 15-25%".to_string(),
            });
            priority = priority.max(4);
        }

        // Allocation time recommendations
        if metrics.avg_allocation_time_ms > self.config.allocation_time_alert_threshold {
            actions.push(RecommendationAction {
                description: "Allocation time exceeds target. Consider increasing buffer pool size or optimizing allocation strategy.".to_string(),
                action_type: ActionType::IncreasePoolSize,
                impact: "Reduce allocation time by 30-50%".to_string(),
            });
            priority = priority.max(5);
        }

        // Pool hit rate recommendations
        if metrics.pool_hit_rate_percent < 80.0 {
            actions.push(RecommendationAction {
                description: "Low pool hit rate. Consider increasing pool size or adjusting buffer size strategy.".to_string(),
                action_type: ActionType::OptimizeAllocationStrategy,
                impact: "Improve pool hit rate to 90%+".to_string(),
            });
            priority = priority.max(3);
        }

        // Default to no action needed
        if actions.is_empty() {
            actions.push(RecommendationAction {
                description: "Buffer utilization is within optimal range. No immediate action required.".to_string(),
                action_type: ActionType::OptimizeAllocationStrategy,
                impact: "Maintain current performance".to_string(),
            });
        }

        OptimizationRecommendations {
            actions,
            priority,
            expected_improvement: if priority >= 4 { "High" } else if priority >= 3 { "Medium" } else { "Low" }.to_string(),
            complexity: if priority >= 4 { "Medium" } else { "Low" }.to_string(),
        }
    }

    /// Check if alerts should be triggered
    pub fn check_alerts(&self) -> Vec<String> {
        let metrics = self.get_current_metrics();
        let mut alerts = Vec::new();

        if metrics.memory_utilization_percent > self.config.memory_alert_threshold {
            alerts.push(format!("High memory usage: {:.1}%", metrics.memory_utilization_percent));
        }

        if metrics.avg_allocation_time_ms > self.config.allocation_time_alert_threshold {
            alerts.push(format!("High allocation time: {:.2}ms", metrics.avg_allocation_time_ms));
        }

        if metrics.pool_hit_rate_percent < 70.0 {
            alerts.push(format!("Low pool hit rate: {:.1}%", metrics.pool_hit_rate_percent));
        }

        alerts
    }
}

impl Default for BufferUtilizationMonitor {
    fn default() -> Self {
        Self::new(100) // 100ms default interval
    }
} 