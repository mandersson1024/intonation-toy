//! # Backpressure Controller Implementation
//!
//! This module provides comprehensive flow control and backpressure handling for
//! data flow pipelines. It includes backpressure detection, handling strategies,
//! adaptive flow control, and performance monitoring to ensure system stability
//! under high load conditions.

use std::sync::{Arc, RwLock, Mutex};
use std::collections::{HashMap, VecDeque};
use std::time::{Instant, Duration};
use std::sync::mpsc::{self, Sender, Receiver};
use serde::{Serialize, Deserialize};

use super::data_flow_coordinator::{PipelineId, FlowError, BackpressureStrategy};

/// Backpressure controller for flow control management
pub struct BackpressureController {
    /// Pipeline congestion states
    congestion_states: Arc<RwLock<HashMap<PipelineId, CongestionState>>>,
    /// Flow control configurations
    flow_configs: Arc<RwLock<HashMap<PipelineId, FlowControlConfig>>>,
    /// Backpressure metrics
    metrics: Arc<RwLock<BackpressureMetrics>>,
    /// Event notifications
    event_sender: Option<Sender<BackpressureEvent>>,
    /// Adaptive control settings
    adaptive_control: Arc<RwLock<AdaptiveControlSettings>>,
    /// Detection algorithms
    detection_algorithms: Vec<BackpressureDetectionAlgorithm>,
}

/// Pipeline congestion state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CongestionState {
    pub pipeline_id: PipelineId,
    pub congestion_level: CongestionLevel,
    pub buffer_utilization: f32,
    pub throughput_ratio: f32,
    pub latency_increase: f32,
    pub last_update: Instant,
    pub consecutive_violations: u32,
    pub recovery_attempts: u32,
}

/// Congestion severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CongestionLevel {
    None,      // Normal operation
    Light,     // Minor congestion
    Moderate,  // Noticeable congestion
    Heavy,     // Significant congestion
    Critical,  // System overload
}

/// Flow control configuration per pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowControlConfig {
    pub pipeline_id: PipelineId,
    pub buffer_threshold: f32,        // Buffer utilization threshold (0.0-1.0)
    pub latency_threshold_ms: f32,    // Maximum acceptable latency
    pub throughput_threshold: f32,    // Minimum acceptable throughput ratio
    pub strategy: BackpressureStrategy,
    pub max_retries: u32,
    pub recovery_timeout_ms: u64,
    pub adaptive_enabled: bool,
}

impl Default for FlowControlConfig {
    fn default() -> Self {
        Self {
            pipeline_id: PipelineId("default".to_string()),
            buffer_threshold: 0.8,
            latency_threshold_ms: 5.0,
            throughput_threshold: 0.7,
            strategy: BackpressureStrategy::Throttle,
            max_retries: 3,
            recovery_timeout_ms: 1000,
            adaptive_enabled: true,
        }
    }
}

/// Adaptive control settings for dynamic flow control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveControlSettings {
    pub learning_rate: f32,           // How quickly to adapt (0.0-1.0)
    pub performance_window_ms: u64,   // Performance measurement window
    pub adaptation_threshold: f32,    // Threshold for triggering adaptation
    pub max_adaptation_rate: f32,     // Maximum rate of change per adaptation
    pub stability_factor: f32,        // Factor for maintaining stability
}

impl Default for AdaptiveControlSettings {
    fn default() -> Self {
        Self {
            learning_rate: 0.1,
            performance_window_ms: 5000,
            adaptation_threshold: 0.1,
            max_adaptation_rate: 0.2,
            stability_factor: 0.9,
        }
    }
}

/// Backpressure detection algorithms
#[derive(Debug, Clone)]
pub enum BackpressureDetectionAlgorithm {
    BufferUtilization,     // Monitor buffer fill levels
    LatencyIncrease,       // Detect latency increases
    ThroughputDegradation, // Monitor throughput drops
    QueueLength,           // Monitor queue lengths
    AdaptiveThreshold,     // Dynamic threshold adjustment
}

/// Backpressure performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpressureMetrics {
    pub total_backpressure_events: u64,
    pub backpressure_events_by_level: HashMap<String, u64>,
    pub average_recovery_time_ms: f32,
    pub max_recovery_time_ms: f32,
    pub successful_recoveries: u64,
    pub failed_recoveries: u64,
    pub adaptive_adjustments: u64,
    pub dropped_data_bytes: u64,
    pub throttled_operations: u64,
}

impl Default for BackpressureMetrics {
    fn default() -> Self {
        Self {
            total_backpressure_events: 0,
            backpressure_events_by_level: HashMap::new(),
            average_recovery_time_ms: 0.0,
            max_recovery_time_ms: 0.0,
            successful_recoveries: 0,
            failed_recoveries: 0,
            adaptive_adjustments: 0,
            dropped_data_bytes: 0,
            throttled_operations: 0,
        }
    }
}

/// Backpressure events for monitoring and notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackpressureEvent {
    CongestionDetected {
        pipeline_id: PipelineId,
        level: CongestionLevel,
        metrics: CongestionMetrics,
    },
    BackpressureApplied {
        pipeline_id: PipelineId,
        strategy: BackpressureStrategy,
        severity: f32,
    },
    RecoveryStarted {
        pipeline_id: PipelineId,
        recovery_strategy: RecoveryStrategy,
    },
    RecoveryCompleted {
        pipeline_id: PipelineId,
        recovery_time_ms: f32,
        success: bool,
    },
    AdaptiveAdjustment {
        pipeline_id: PipelineId,
        parameter: String,
        old_value: f32,
        new_value: f32,
    },
    ThresholdViolation {
        pipeline_id: PipelineId,
        threshold_type: String,
        violation_severity: f32,
    },
}

/// Congestion metrics for detailed monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CongestionMetrics {
    pub buffer_utilization: f32,
    pub latency_ms: f32,
    pub throughput_ops_per_sec: f32,
    pub queue_length: usize,
    pub drop_rate: f32,
}

/// Recovery strategies for backpressure situations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    GradualThrottling,     // Gradually reduce flow rate
    ImmediateDrop,         // Drop excess data immediately
    BufferExpansion,       // Temporarily increase buffer size
    LoadRedistribution,    // Redistribute load to other pipelines
    EmergencyBypass,       // Bypass non-critical processing
}

impl BackpressureController {
    /// Create a new backpressure controller
    pub fn new() -> Self {
        Self {
            congestion_states: Arc::new(RwLock::new(HashMap::new())),
            flow_configs: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(BackpressureMetrics::default())),
            event_sender: None,
            adaptive_control: Arc::new(RwLock::new(AdaptiveControlSettings::default())),
            detection_algorithms: vec![
                BackpressureDetectionAlgorithm::BufferUtilization,
                BackpressureDetectionAlgorithm::LatencyIncrease,
                BackpressureDetectionAlgorithm::ThroughputDegradation,
                BackpressureDetectionAlgorithm::AdaptiveThreshold,
            ],
        }
    }
    
    /// Set event sender for backpressure notifications
    pub fn set_event_sender(&mut self, sender: Sender<BackpressureEvent>) {
        self.event_sender = Some(sender);
    }
    
    /// Register pipeline for backpressure monitoring
    pub fn register_pipeline(
        &mut self,
        pipeline_id: PipelineId,
        config: FlowControlConfig,
    ) -> Result<(), FlowError> {
        // Store flow control configuration
        if let Ok(mut configs) = self.flow_configs.write() {
            configs.insert(pipeline_id.clone(), config);
        }
        
        // Initialize congestion state
        let initial_state = CongestionState {
            pipeline_id: pipeline_id.clone(),
            congestion_level: CongestionLevel::None,
            buffer_utilization: 0.0,
            throughput_ratio: 1.0,
            latency_increase: 0.0,
            last_update: Instant::now(),
            consecutive_violations: 0,
            recovery_attempts: 0,
        };
        
        if let Ok(mut states) = self.congestion_states.write() {
            states.insert(pipeline_id, initial_state);
        }
        
        Ok(())
    }
    
    /// Detect backpressure conditions for a pipeline
    pub fn detect_backpressure(
        &mut self,
        pipeline_id: &PipelineId,
        current_metrics: CongestionMetrics,
    ) -> Result<CongestionLevel, FlowError> {
        let mut detection_results = Vec::new();
        
        // Run all detection algorithms
        for algorithm in &self.detection_algorithms {
            let result = self.run_detection_algorithm(algorithm, pipeline_id, &current_metrics)?;
            detection_results.push(result);
        }
        
        // Aggregate detection results
        let aggregated_level = self.aggregate_detection_results(detection_results);
        
        // Update congestion state
        self.update_congestion_state(pipeline_id, aggregated_level, current_metrics)?;
        
        // Publish event if congestion detected
        if aggregated_level != CongestionLevel::None {
            self.publish_event(BackpressureEvent::CongestionDetected {
                pipeline_id: pipeline_id.clone(),
                level: aggregated_level.clone(),
                metrics: current_metrics,
            });
        }
        
        Ok(aggregated_level)
    }
    
    /// Apply backpressure strategy for a pipeline
    pub fn apply_backpressure(
        &mut self,
        pipeline_id: &PipelineId,
        strategy: BackpressureStrategy,
    ) -> Result<BackpressureResult, FlowError> {
        let apply_start = Instant::now();
        
        let result = match strategy {
            BackpressureStrategy::Drop => {
                self.apply_drop_strategy(pipeline_id)?
            },
            BackpressureStrategy::Block => {
                self.apply_block_strategy(pipeline_id)?
            },
            BackpressureStrategy::Throttle => {
                self.apply_throttle_strategy(pipeline_id)?
            },
            BackpressureStrategy::Buffer => {
                self.apply_buffer_strategy(pipeline_id)?
            },
        };
        
        // Update metrics
        self.update_backpressure_metrics(pipeline_id, &strategy, apply_start.elapsed(), result.success);
        
        // Publish event
        self.publish_event(BackpressureEvent::BackpressureApplied {
            pipeline_id: pipeline_id.clone(),
            strategy,
            severity: result.severity,
        });
        
        Ok(result)
    }
    
    /// Run specific detection algorithm
    fn run_detection_algorithm(
        &self,
        algorithm: &BackpressureDetectionAlgorithm,
        pipeline_id: &PipelineId,
        metrics: &CongestionMetrics,
    ) -> Result<CongestionLevel, FlowError> {
        let config = self.get_flow_config(pipeline_id)?;
        
        match algorithm {
            BackpressureDetectionAlgorithm::BufferUtilization => {
                if metrics.buffer_utilization > 0.9 {
                    Ok(CongestionLevel::Critical)
                } else if metrics.buffer_utilization > config.buffer_threshold {
                    Ok(CongestionLevel::Heavy)
                } else if metrics.buffer_utilization > config.buffer_threshold * 0.7 {
                    Ok(CongestionLevel::Moderate)
                } else {
                    Ok(CongestionLevel::None)
                }
            },
            BackpressureDetectionAlgorithm::LatencyIncrease => {
                if metrics.latency_ms > config.latency_threshold_ms * 2.0 {
                    Ok(CongestionLevel::Critical)
                } else if metrics.latency_ms > config.latency_threshold_ms {
                    Ok(CongestionLevel::Heavy)
                } else if metrics.latency_ms > config.latency_threshold_ms * 0.8 {
                    Ok(CongestionLevel::Moderate)
                } else {
                    Ok(CongestionLevel::None)
                }
            },
            BackpressureDetectionAlgorithm::ThroughputDegradation => {
                if metrics.throughput_ops_per_sec < config.throughput_threshold * 0.5 {
                    Ok(CongestionLevel::Critical)
                } else if metrics.throughput_ops_per_sec < config.throughput_threshold {
                    Ok(CongestionLevel::Heavy)
                } else if metrics.throughput_ops_per_sec < config.throughput_threshold * 1.2 {
                    Ok(CongestionLevel::Moderate)
                } else {
                    Ok(CongestionLevel::None)
                }
            },
            BackpressureDetectionAlgorithm::QueueLength => {
                if metrics.queue_length > 1000 {
                    Ok(CongestionLevel::Critical)
                } else if metrics.queue_length > 500 {
                    Ok(CongestionLevel::Heavy)
                } else if metrics.queue_length > 100 {
                    Ok(CongestionLevel::Moderate)
                } else {
                    Ok(CongestionLevel::None)
                }
            },
            BackpressureDetectionAlgorithm::AdaptiveThreshold => {
                // Adaptive detection based on historical performance
                self.adaptive_detection(pipeline_id, metrics)
            },
        }
    }
    
    /// Adaptive detection algorithm
    fn adaptive_detection(
        &self,
        pipeline_id: &PipelineId,
        metrics: &CongestionMetrics,
    ) -> Result<CongestionLevel, FlowError> {
        // Simplified adaptive detection
        // In a real implementation, this would use machine learning or statistical analysis
        
        if let Ok(states) = self.congestion_states.read() {
            if let Some(state) = states.get(pipeline_id) {
                let performance_score = 
                    (1.0 - metrics.buffer_utilization) * 0.4 +
                    (1.0 / metrics.latency_ms.max(1.0)) * 0.3 +
                    (metrics.throughput_ops_per_sec / 1000.0).min(1.0) * 0.3;
                
                if performance_score < 0.3 {
                    Ok(CongestionLevel::Critical)
                } else if performance_score < 0.5 {
                    Ok(CongestionLevel::Heavy)
                } else if performance_score < 0.7 {
                    Ok(CongestionLevel::Moderate)
                } else {
                    Ok(CongestionLevel::None)
                }
            } else {
                Ok(CongestionLevel::None)
            }
        } else {
            Ok(CongestionLevel::None)
        }
    }
    
    /// Aggregate multiple detection results
    fn aggregate_detection_results(&self, results: Vec<CongestionLevel>) -> CongestionLevel {
        let mut critical_count = 0;
        let mut heavy_count = 0;
        let mut moderate_count = 0;
        
        for result in results {
            match result {
                CongestionLevel::Critical => critical_count += 1,
                CongestionLevel::Heavy => heavy_count += 1,
                CongestionLevel::Moderate => moderate_count += 1,
                _ => {},
            }
        }
        
        // Majority voting with bias toward higher severity
        if critical_count >= 2 {
            CongestionLevel::Critical
        } else if heavy_count >= 2 || (critical_count >= 1 && heavy_count >= 1) {
            CongestionLevel::Heavy
        } else if moderate_count >= 2 || (heavy_count >= 1 && moderate_count >= 1) {
            CongestionLevel::Moderate
        } else if moderate_count >= 1 {
            CongestionLevel::Light
        } else {
            CongestionLevel::None
        }
    }
    
    /// Apply drop backpressure strategy
    fn apply_drop_strategy(&mut self, pipeline_id: &PipelineId) -> Result<BackpressureResult, FlowError> {
        // Simulate dropping data
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.dropped_data_bytes += 1024; // Simulate dropped data
        }
        
        Ok(BackpressureResult {
            strategy: BackpressureStrategy::Drop,
            success: true,
            severity: 0.5,
            data_affected_bytes: 1024,
            recovery_time_ms: 0.0,
        })
    }
    
    /// Apply block backpressure strategy
    fn apply_block_strategy(&mut self, pipeline_id: &PipelineId) -> Result<BackpressureResult, FlowError> {
        // Simulate blocking operation
        std::thread::sleep(Duration::from_millis(10));
        
        Ok(BackpressureResult {
            strategy: BackpressureStrategy::Block,
            success: true,
            severity: 0.8,
            data_affected_bytes: 0,
            recovery_time_ms: 10.0,
        })
    }
    
    /// Apply throttle backpressure strategy
    fn apply_throttle_strategy(&mut self, pipeline_id: &PipelineId) -> Result<BackpressureResult, FlowError> {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.throttled_operations += 1;
        }
        
        Ok(BackpressureResult {
            strategy: BackpressureStrategy::Throttle,
            success: true,
            severity: 0.3,
            data_affected_bytes: 0,
            recovery_time_ms: 5.0,
        })
    }
    
    /// Apply buffer expansion strategy
    fn apply_buffer_strategy(&mut self, pipeline_id: &PipelineId) -> Result<BackpressureResult, FlowError> {
        // Simulate buffer expansion
        Ok(BackpressureResult {
            strategy: BackpressureStrategy::Buffer,
            success: true,
            severity: 0.2,
            data_affected_bytes: 0,
            recovery_time_ms: 1.0,
        })
    }
    
    /// Update congestion state for pipeline
    fn update_congestion_state(
        &mut self,
        pipeline_id: &PipelineId,
        level: CongestionLevel,
        metrics: CongestionMetrics,
    ) -> Result<(), FlowError> {
        if let Ok(mut states) = self.congestion_states.write() {
            if let Some(state) = states.get_mut(pipeline_id) {
                let previous_level = state.congestion_level.clone();
                state.congestion_level = level.clone();
                state.buffer_utilization = metrics.buffer_utilization;
                state.throughput_ratio = metrics.throughput_ops_per_sec / 1000.0; // Normalize
                state.latency_increase = metrics.latency_ms;
                state.last_update = Instant::now();
                
                if level != CongestionLevel::None {
                    if level == previous_level {
                        state.consecutive_violations += 1;
                    } else {
                        state.consecutive_violations = 1;
                    }
                } else {
                    state.consecutive_violations = 0;
                }
            }
        }
        
        Ok(())
    }
    
    /// Get flow control configuration for pipeline
    fn get_flow_config(&self, pipeline_id: &PipelineId) -> Result<FlowControlConfig, FlowError> {
        if let Ok(configs) = self.flow_configs.read() {
            configs.get(pipeline_id)
                .cloned()
                .ok_or(FlowError::PipelineNotFound)
        } else {
            Err(FlowError::PipelineNotFound)
        }
    }
    
    /// Update backpressure metrics
    fn update_backpressure_metrics(
        &mut self,
        pipeline_id: &PipelineId,
        strategy: &BackpressureStrategy,
        duration: Duration,
        success: bool,
    ) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.total_backpressure_events += 1;
            
            let level_key = format!("{:?}", strategy);
            *metrics.backpressure_events_by_level.entry(level_key).or_insert(0) += 1;
            
            let recovery_time_ms = duration.as_millis() as f32;
            metrics.average_recovery_time_ms = 
                0.9 * metrics.average_recovery_time_ms + 0.1 * recovery_time_ms;
            metrics.max_recovery_time_ms = 
                metrics.max_recovery_time_ms.max(recovery_time_ms);
            
            if success {
                metrics.successful_recoveries += 1;
            } else {
                metrics.failed_recoveries += 1;
            }
        }
    }
    
    /// Publish backpressure event
    fn publish_event(&self, event: BackpressureEvent) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event);
        }
    }
    
    /// Get backpressure metrics
    pub fn get_metrics(&self) -> Option<BackpressureMetrics> {
        if let Ok(metrics) = self.metrics.read() {
            Some(metrics.clone())
        } else {
            None
        }
    }
    
    /// Get congestion state for pipeline
    pub fn get_congestion_state(&self, pipeline_id: &PipelineId) -> Option<CongestionState> {
        if let Ok(states) = self.congestion_states.read() {
            states.get(pipeline_id).cloned()
        } else {
            None
        }
    }
    
    /// Perform adaptive adjustment of flow control parameters
    pub fn adapt_flow_control(&mut self, pipeline_id: &PipelineId) -> Result<(), FlowError> {
        if let Ok(adaptive_settings) = self.adaptive_control.read() {
            if let Ok(mut configs) = self.flow_configs.write() {
                if let Some(config) = configs.get_mut(pipeline_id) {
                    if config.adaptive_enabled {
                        // Simple adaptive adjustment
                        let state = self.get_congestion_state(pipeline_id)
                            .ok_or(FlowError::PipelineNotFound)?;
                        
                        let adjustment_factor = adaptive_settings.learning_rate;
                        
                        match state.congestion_level {
                            CongestionLevel::Heavy | CongestionLevel::Critical => {
                                // Tighten thresholds
                                config.buffer_threshold *= (1.0 - adjustment_factor);
                                config.latency_threshold_ms *= (1.0 - adjustment_factor);
                            },
                            CongestionLevel::None => {
                                // Relax thresholds slightly
                                config.buffer_threshold = 
                                    (config.buffer_threshold * (1.0 + adjustment_factor * 0.1)).min(0.95);
                                config.latency_threshold_ms *= (1.0 + adjustment_factor * 0.1);
                            },
                            _ => {}
                        }
                        
                        if let Ok(mut metrics) = self.metrics.write() {
                            metrics.adaptive_adjustments += 1;
                        }
                        
                        self.publish_event(BackpressureEvent::AdaptiveAdjustment {
                            pipeline_id: pipeline_id.clone(),
                            parameter: "buffer_threshold".to_string(),
                            old_value: state.buffer_utilization,
                            new_value: config.buffer_threshold,
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Result of applying backpressure strategy
#[derive(Debug, Clone)]
pub struct BackpressureResult {
    pub strategy: BackpressureStrategy,
    pub success: bool,
    pub severity: f32,
    pub data_affected_bytes: u64,
    pub recovery_time_ms: f32,
}

impl Default for BackpressureController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_backpressure_controller_creation() {
        let controller = BackpressureController::new();
        assert_eq!(controller.detection_algorithms.len(), 4);
    }
    
    #[test]
    fn test_pipeline_registration() {
        let mut controller = BackpressureController::new();
        let pipeline_id = PipelineId("test".to_string());
        let config = FlowControlConfig::default();
        
        let result = controller.register_pipeline(pipeline_id.clone(), config);
        assert!(result.is_ok());
        
        let state = controller.get_congestion_state(&pipeline_id);
        assert!(state.is_some());
        assert_eq!(state.unwrap().congestion_level, CongestionLevel::None);
    }
    
    #[test]
    fn test_backpressure_detection() {
        let mut controller = BackpressureController::new();
        let pipeline_id = PipelineId("test".to_string());
        let config = FlowControlConfig::default();
        
        controller.register_pipeline(pipeline_id.clone(), config).unwrap();
        
        let metrics = CongestionMetrics {
            buffer_utilization: 0.9,
            latency_ms: 10.0,
            throughput_ops_per_sec: 500.0,
            queue_length: 100,
            drop_rate: 0.0,
        };
        
        let result = controller.detect_backpressure(&pipeline_id, metrics);
        assert!(result.is_ok());
        assert_ne!(result.unwrap(), CongestionLevel::None);
    }
    
    #[test]
    fn test_backpressure_application() {
        let mut controller = BackpressureController::new();
        let pipeline_id = PipelineId("test".to_string());
        let config = FlowControlConfig::default();
        
        controller.register_pipeline(pipeline_id.clone(), config).unwrap();
        
        let result = controller.apply_backpressure(&pipeline_id, BackpressureStrategy::Throttle);
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }
    
    #[test]
    fn test_congestion_level_ordering() {
        assert!(CongestionLevel::Critical != CongestionLevel::None);
        assert!(CongestionLevel::Heavy != CongestionLevel::Light);
    }
}
