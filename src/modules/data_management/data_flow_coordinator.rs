//! # Data Flow Coordinator Implementation
//!
//! This module provides data flow coordination between modules, pipeline registration,
//! data flow routing, and performance monitoring. It handles flow control, backpressure,
//! and error recovery for seamless inter-module data sharing.

use std::sync::{Arc, RwLock, Mutex};
use std::collections::HashMap;
use std::time::{Instant, Duration};

/// Data flow coordinator for inter-module data sharing
pub struct DataFlowCoordinator {
    /// Registered data flow pipelines
    pipelines: Arc<RwLock<HashMap<String, DataPipeline>>>,
    /// Flow control settings
    flow_control: Arc<RwLock<FlowControlSettings>>,
    /// Performance metrics
    metrics: Arc<RwLock<DataFlowMetrics>>,
    /// Coordinator state
    state: Arc<Mutex<CoordinatorState>>,
}

/// Data pipeline configuration
#[derive(Debug, Clone)]
pub struct DataPipeline {
    pub name: String,
    pub source_module: String,
    pub destination_module: String,
    pub buffer_size: usize,
    pub flow_rate_limit: Option<u32>,
    pub backpressure_enabled: bool,
    pub created_at: Instant,
    pub active: bool,
}

/// Flow control settings
#[derive(Debug, Clone)]
pub struct FlowControlSettings {
    pub max_concurrent_transfers: u32,
    pub backpressure_threshold: f64,
    pub error_recovery_enabled: bool,
    pub flow_monitoring_enabled: bool,
}

impl Default for FlowControlSettings {
    fn default() -> Self {
        Self {
            max_concurrent_transfers: 100,
            backpressure_threshold: 0.8,
            error_recovery_enabled: true,
            flow_monitoring_enabled: true,
        }
    }
}

/// Data flow performance metrics
#[derive(Debug, Clone, Default)]
pub struct DataFlowMetrics {
    pub total_transfers: u64,
    pub successful_transfers: u64,
    pub failed_transfers: u64,
    pub avg_transfer_time_ms: f64,
    pub throughput_mbps: f64,
    pub active_pipelines: u32,
    pub backpressure_events: u64,
    pub error_recovery_events: u64,
}

/// Coordinator state
#[derive(Debug, Clone, PartialEq)]
pub enum CoordinatorState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error(String),
}

impl DataFlowCoordinator {
    /// Create a new data flow coordinator
    pub fn new() -> Self {
        Self {
            pipelines: Arc::new(RwLock::new(HashMap::new())),
            flow_control: Arc::new(RwLock::new(FlowControlSettings::default())),
            metrics: Arc::new(RwLock::new(DataFlowMetrics::default())),
            state: Arc::new(Mutex::new(CoordinatorState::Stopped)),
        }
    }

    /// Register a pipeline between Audio Foundations and other modules
    pub fn register_audio_foundations_pipeline(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pipeline = DataPipeline {
            name: "audio-foundations-pipeline".to_string(),
            source_module: "audio-foundations".to_string(),
            destination_module: "data-management".to_string(),
            buffer_size: 2048,
            flow_rate_limit: Some(1000), // 1000 buffers/second
            backpressure_enabled: true,
            created_at: Instant::now(),
            active: false,
        };

        if let Ok(mut pipelines) = self.pipelines.write() {
            pipelines.insert(pipeline.name.clone(), pipeline);
        }

        Ok(())
    }

    /// Start the data flow coordinator
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut state) = self.state.lock() {
            *state = CoordinatorState::Starting;
        }

        // Activate all registered pipelines
        if let Ok(mut pipelines) = self.pipelines.write() {
            for pipeline in pipelines.values_mut() {
                pipeline.active = true;
            }
        }

        if let Ok(mut state) = self.state.lock() {
            *state = CoordinatorState::Running;
        }

        Ok(())
    }

    /// Stop the data flow coordinator
    pub fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut state) = self.state.lock() {
            *state = CoordinatorState::Stopping;
        }

        // Deactivate all pipelines
        if let Ok(mut pipelines) = self.pipelines.write() {
            for pipeline in pipelines.values_mut() {
                pipeline.active = false;
            }
        }

        if let Ok(mut state) = self.state.lock() {
            *state = CoordinatorState::Stopped;
        }

        Ok(())
    }

    /// Get the number of registered pipelines
    pub fn get_pipeline_count(&self) -> u32 {
        if let Ok(pipelines) = self.pipelines.read() {
            pipelines.len() as u32
        } else {
            0
        }
    }

    /// Get data flow metrics
    pub fn get_metrics(&self) -> DataFlowMetrics {
        if let Ok(metrics) = self.metrics.read() {
            metrics.clone()
        } else {
            DataFlowMetrics::default()
        }
    }
}

impl Default for DataFlowCoordinator {
    fn default() -> Self {
        Self::new()
    }
} 