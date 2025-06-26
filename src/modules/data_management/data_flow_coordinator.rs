//! # Data Flow Coordinator Implementation
//!
//! This module provides comprehensive data flow coordination between modules, pipeline
//! registration, data flow routing, and performance monitoring. It handles flow control,
//! backpressure, error recovery, and real-time data transformation for seamless
//! inter-module data sharing.

use std::sync::{Arc, RwLock, Mutex};
use std::collections::HashMap;
use std::time::{Instant, Duration};
use std::sync::mpsc::{self, Sender, Receiver};
use serde::{Serialize, Deserialize};

/// Module identifier for data flow routing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleId {
    ApplicationCore,
    AudioFoundations,
    DataManagement,
    PlatformAbstraction,
    DevelopmentTools,
    PerformanceObservability,
}

/// Pipeline identifier for tracking data flows
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PipelineId(pub String);

/// Data formats supported by the flow coordinator
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataFormat {
    AudioBuffer,
    TypedEvent,
    MetricsData,
    ConfigurationData,
    BinaryData,
}

/// Flow data container for inter-module communication
#[derive(Debug, Clone)]
pub struct FlowData {
    pub format: DataFormat,
    pub data: Vec<u8>,
    pub metadata: HashMap<String, String>,
    pub timestamp: Instant,
}

/// Pipeline priority levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PipelinePriority {
    Critical,  // Real-time audio processing
    High,      // Performance monitoring
    Normal,    // General data flow
    Low,       // Background operations
}

/// Backpressure handling strategies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BackpressureStrategy {
    Drop,      // Drop oldest data
    Block,     // Block until space available
    Throttle,  // Reduce flow rate
    Buffer,    // Increase buffer size
}

/// Pipeline health status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PipelineHealth {
    Healthy,
    Degraded,
    Critical,
    Failed,
}

/// Flow error types
#[derive(Debug, Clone, PartialEq)]
pub enum FlowError {
    PipelineNotFound,
    InvalidConfiguration,
    BackpressureExceeded,
    TransformationFailed,
    RecoveryFailed,
    LatencyExceeded,
}

/// Transformation error types
#[derive(Debug, Clone, PartialEq)]
pub enum TransformError {
    UnsupportedFormat,
    InvalidData,
    ConversionFailed,
}

/// Data flow coordinator trait for inter-module data sharing
pub trait DataFlowCoordinator: Send + Sync {
    /// Register data flow pipeline between modules
    fn register_pipeline(&mut self, from: ModuleId, to: ModuleId, config: PipelineConfig) -> Result<PipelineId, FlowError>;
    
    /// Send data through registered pipeline
    fn send_data(&mut self, pipeline_id: PipelineId, data: FlowData) -> Result<(), FlowError>;
    
    /// Monitor data flow metrics
    fn get_flow_metrics(&self, pipeline_id: PipelineId) -> FlowMetrics;
    
    /// Handle backpressure situations
    fn handle_backpressure(&mut self, pipeline_id: PipelineId, strategy: BackpressureStrategy) -> Result<(), FlowError>;
    
    /// Transform data between module formats
    fn transform_data(&self, data: FlowData, from_format: DataFormat, to_format: DataFormat) -> Result<FlowData, TransformError>;
    
    /// Get pipeline health status
    fn get_pipeline_health(&self, pipeline_id: PipelineId) -> PipelineHealth;
    
    /// Start the coordinator
    fn start(&mut self) -> Result<(), FlowError>;
    
    /// Stop the coordinator
    fn stop(&mut self) -> Result<(), FlowError>;
}

/// Data flow coordinator implementation for inter-module data sharing
pub struct DataFlowCoordinatorImpl {
    /// Registered data flow pipelines
    pipelines: Arc<RwLock<HashMap<PipelineId, DataPipeline>>>,
    /// Flow control settings
    flow_control: Arc<RwLock<FlowControlSettings>>,
    /// Performance metrics
    metrics: Arc<RwLock<HashMap<PipelineId, FlowMetrics>>>,
    /// Coordinator state
    state: Arc<Mutex<CoordinatorState>>,
    /// Event sender for publishing flow events
    event_sender: Option<Sender<DataFlowEvent>>,
    /// Next pipeline ID counter
    next_pipeline_id: Arc<Mutex<u64>>,
}

/// Data pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub buffer_size: usize,
    pub max_latency_ms: f32,
    pub priority: PipelinePriority,
    pub backpressure_strategy: BackpressureStrategy,
    pub transformation_required: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1024,
            max_latency_ms: 2.0,
            priority: PipelinePriority::Normal,
            backpressure_strategy: BackpressureStrategy::Buffer,
            transformation_required: false,
        }
    }
}

/// Data pipeline runtime state
#[derive(Debug, Clone)]
pub struct DataPipeline {
    pub id: PipelineId,
    pub source_module: ModuleId,
    pub destination_module: ModuleId,
    pub config: PipelineConfig,
    pub created_at: Instant,
    pub active: bool,
    pub health: PipelineHealth,
    pub last_transfer: Option<Instant>,
    pub pending_data: Vec<FlowData>,
}

/// Flow control settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowControlSettings {
    pub max_concurrent_transfers: u32,
    pub backpressure_threshold: f64,
    pub error_recovery_enabled: bool,
    pub flow_monitoring_enabled: bool,
    pub max_retry_attempts: u32,
    pub retry_delay_ms: u64,
}

impl Default for FlowControlSettings {
    fn default() -> Self {
        Self {
            max_concurrent_transfers: 1000,
            backpressure_threshold: 0.8,
            error_recovery_enabled: true,
            flow_monitoring_enabled: true,
            max_retry_attempts: 3,
            retry_delay_ms: 100,
        }
    }
}

/// Flow metrics for monitoring pipeline performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowMetrics {
    pub throughput_ops_per_second: f32,
    pub average_latency_ms: f32,
    pub error_rate_percentage: f32,
    pub backpressure_events: u32,
    pub pipeline_health: PipelineHealth,
    pub total_transfers: u64,
    pub successful_transfers: u64,
    pub failed_transfers: u64,
    pub buffer_utilization: f32,
}

impl Default for FlowMetrics {
    fn default() -> Self {
        Self {
            throughput_ops_per_second: 0.0,
            average_latency_ms: 0.0,
            error_rate_percentage: 0.0,
            backpressure_events: 0,
            pipeline_health: PipelineHealth::Healthy,
            total_transfers: 0,
            successful_transfers: 0,
            failed_transfers: 0,
            buffer_utilization: 0.0,
        }
    }
}

/// Data flow events for publishing through TypedEventBus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataFlowEvent {
    PipelineRegistered {
        pipeline_id: PipelineId,
        source: ModuleId,
        destination: ModuleId,
    },
    DataFlowStarted {
        pipeline_id: PipelineId,
        data_size: usize,
    },
    BackpressureDetected {
        pipeline_id: PipelineId,
        buffer_utilization: f32,
    },
    PipelineFailure {
        pipeline_id: PipelineId,
        error: String,
    },
    FlowRecovery {
        pipeline_id: PipelineId,
        recovery_time_ms: f32,
    },
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

impl DataFlowCoordinatorImpl {
    /// Create a new data flow coordinator
    pub fn new() -> Self {
        Self {
            pipelines: Arc::new(RwLock::new(HashMap::new())),
            flow_control: Arc::new(RwLock::new(FlowControlSettings::default())),
            metrics: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(Mutex::new(CoordinatorState::Stopped)),
            event_sender: None,
            next_pipeline_id: Arc::new(Mutex::new(0)),
        }
    }
    
    /// Set event sender for publishing flow events
    pub fn set_event_sender(&mut self, sender: Sender<DataFlowEvent>) {
        self.event_sender = Some(sender);
    }
    
    /// Generate next pipeline ID
    fn generate_pipeline_id(&self) -> PipelineId {
        if let Ok(mut counter) = self.next_pipeline_id.lock() {
            *counter += 1;
            PipelineId(format!("pipeline_{}", *counter))
        } else {
            PipelineId(format!("pipeline_{}", std::process::id()))
        }
    }
    
    /// Publish data flow event
    fn publish_event(&self, event: DataFlowEvent) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event);
        }
    }
    
    /// Update pipeline metrics
    fn update_metrics(&self, pipeline_id: &PipelineId, success: bool, latency_ms: f32) {
        if let Ok(mut metrics) = self.metrics.write() {
            let metric = metrics.entry(pipeline_id.clone()).or_insert_with(FlowMetrics::default);
            
            metric.total_transfers += 1;
            if success {
                metric.successful_transfers += 1;
            } else {
                metric.failed_transfers += 1;
            }
            
            metric.error_rate_percentage = 
                (metric.failed_transfers as f32 / metric.total_transfers as f32) * 100.0;
            
            // Update average latency using exponential moving average
            metric.average_latency_ms = 
                0.9 * metric.average_latency_ms + 0.1 * latency_ms;
            
            // Calculate throughput (simplified)
            metric.throughput_ops_per_second = 
                metric.successful_transfers as f32 / 
                (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default().as_secs() as f32).max(1.0);
        }
    }

    /// Register a pipeline between Audio Foundations and other modules
    pub fn register_audio_foundations_pipeline(&mut self) -> Result<PipelineId, FlowError> {
        let config = PipelineConfig {
            buffer_size: 2048,
            max_latency_ms: 1.0, // Critical audio latency
            priority: PipelinePriority::Critical,
            backpressure_strategy: BackpressureStrategy::Drop,
            transformation_required: true,
        };
        
        self.register_pipeline(
            ModuleId::AudioFoundations,
            ModuleId::DataManagement,
            config
        )
    }

    /// Check if pipeline meets latency requirements
    fn check_latency_compliance(&self, pipeline_id: &PipelineId) -> bool {
        if let (Ok(pipelines), Ok(metrics)) = (self.pipelines.read(), self.metrics.read()) {
            if let (Some(pipeline), Some(metric)) = (pipelines.get(pipeline_id), metrics.get(pipeline_id)) {
                return metric.average_latency_ms <= pipeline.config.max_latency_ms;
            }
        }
        false
    }
    
    /// Handle pipeline recovery
    fn recover_pipeline(&mut self, pipeline_id: &PipelineId) -> Result<(), FlowError> {
        let recovery_start = Instant::now();
        
        // Clear pending data if necessary
        if let Ok(mut pipelines) = self.pipelines.write() {
            if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
                pipeline.pending_data.clear();
                pipeline.health = PipelineHealth::Healthy;
                pipeline.active = true;
            }
        }
        
        let recovery_time = recovery_start.elapsed().as_millis() as f32;
        
        self.publish_event(DataFlowEvent::FlowRecovery {
            pipeline_id: pipeline_id.clone(),
            recovery_time_ms: recovery_time,
        });
        
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
    
    /// Get all pipeline metrics
    pub fn get_all_metrics(&self) -> HashMap<PipelineId, FlowMetrics> {
        if let Ok(metrics) = self.metrics.read() {
            metrics.clone()
        } else {
            HashMap::new()
        }
    }
}

impl DataFlowCoordinator for DataFlowCoordinatorImpl {
    fn register_pipeline(&mut self, from: ModuleId, to: ModuleId, config: PipelineConfig) -> Result<PipelineId, FlowError> {
        let pipeline_id = self.generate_pipeline_id();
        
        let pipeline = DataPipeline {
            id: pipeline_id.clone(),
            source_module: from.clone(),
            destination_module: to.clone(),
            config,
            created_at: Instant::now(),
            active: false,
            health: PipelineHealth::Healthy,
            last_transfer: None,
            pending_data: Vec::new(),
        };
        
        if let Ok(mut pipelines) = self.pipelines.write() {
            pipelines.insert(pipeline_id.clone(), pipeline);
        } else {
            return Err(FlowError::InvalidConfiguration);
        }
        
        // Initialize metrics
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.insert(pipeline_id.clone(), FlowMetrics::default());
        }
        
        self.publish_event(DataFlowEvent::PipelineRegistered {
            pipeline_id: pipeline_id.clone(),
            source: from,
            destination: to,
        });
        
        Ok(pipeline_id)
    }
    
    fn send_data(&mut self, pipeline_id: PipelineId, data: FlowData) -> Result<(), FlowError> {
        let send_start = Instant::now();
        
        // Check if pipeline exists and is active
        let pipeline_exists = {
            if let Ok(pipelines) = self.pipelines.read() {
                pipelines.get(&pipeline_id)
                    .map(|p| p.active)
                    .unwrap_or(false)
            } else {
                false
            }
        };
        
        if !pipeline_exists {
            return Err(FlowError::PipelineNotFound);
        }
        
        // Check latency compliance
        if !self.check_latency_compliance(&pipeline_id) {
            return Err(FlowError::LatencyExceeded);
        }
        
        // Handle data flow
        let data_size = data.data.len();
        let result = self.process_data_flow(pipeline_id.clone(), data);
        
        let latency = send_start.elapsed().as_millis() as f32;
        self.update_metrics(&pipeline_id, result.is_ok(), latency);
        
        if result.is_ok() {
            self.publish_event(DataFlowEvent::DataFlowStarted {
                pipeline_id,
                data_size,
            });
        }
        
        result
    }
    
    fn get_flow_metrics(&self, pipeline_id: PipelineId) -> FlowMetrics {
        if let Ok(metrics) = self.metrics.read() {
            metrics.get(&pipeline_id).cloned().unwrap_or_default()
        } else {
            FlowMetrics::default()
        }
    }
    
    fn handle_backpressure(&mut self, pipeline_id: PipelineId, strategy: BackpressureStrategy) -> Result<(), FlowError> {
        if let Ok(mut pipelines) = self.pipelines.write() {
            if let Some(pipeline) = pipelines.get_mut(&pipeline_id) {
                match strategy {
                    BackpressureStrategy::Drop => {
                        pipeline.pending_data.clear();
                    },
                    BackpressureStrategy::Buffer => {
                        pipeline.config.buffer_size *= 2;
                    },
                    BackpressureStrategy::Throttle => {
                        pipeline.config.max_latency_ms *= 1.5;
                    },
                    BackpressureStrategy::Block => {
                        // Implemented in send_data logic
                    },
                }
                
                // Update metrics
                if let Ok(mut metrics) = self.metrics.write() {
                    if let Some(metric) = metrics.get_mut(&pipeline_id) {
                        metric.backpressure_events += 1;
                        metric.buffer_utilization = 
                            (pipeline.pending_data.len() as f32 / pipeline.config.buffer_size as f32) * 100.0;
                    }
                }
                
                self.publish_event(DataFlowEvent::BackpressureDetected {
                    pipeline_id,
                    buffer_utilization: (pipeline.pending_data.len() as f32 / pipeline.config.buffer_size as f32) * 100.0,
                });
                
                return Ok(());
            }
        }
        
        Err(FlowError::PipelineNotFound)
    }
    
    fn transform_data(&self, data: FlowData, from_format: DataFormat, to_format: DataFormat) -> Result<FlowData, TransformError> {
        if from_format == to_format {
            return Ok(data);
        }
        
        // Basic transformation logic - would be expanded based on actual formats
        let transformed_data = match (from_format, to_format) {
            (DataFormat::AudioBuffer, DataFormat::BinaryData) => {
                FlowData {
                    format: DataFormat::BinaryData,
                    data: data.data,
                    metadata: data.metadata,
                    timestamp: data.timestamp,
                }
            },
            _ => return Err(TransformError::UnsupportedFormat),
        };
        
        Ok(transformed_data)
    }
    
    fn get_pipeline_health(&self, pipeline_id: PipelineId) -> PipelineHealth {
        if let Ok(pipelines) = self.pipelines.read() {
            pipelines.get(&pipeline_id)
                .map(|p| p.health.clone())
                .unwrap_or(PipelineHealth::Failed)
        } else {
            PipelineHealth::Failed
        }
    }
    
    fn start(&mut self) -> Result<(), FlowError> {
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
    
    fn stop(&mut self) -> Result<(), FlowError> {
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
}

impl DataFlowCoordinatorImpl {
    /// Process data flow through pipeline
    fn process_data_flow(&mut self, pipeline_id: PipelineId, data: FlowData) -> Result<(), FlowError> {
        if let Ok(mut pipelines) = self.pipelines.write() {
            if let Some(pipeline) = pipelines.get_mut(&pipeline_id) {
                // Check buffer capacity
                if pipeline.pending_data.len() >= pipeline.config.buffer_size {
                    match pipeline.config.backpressure_strategy {
                        BackpressureStrategy::Drop => {
                            pipeline.pending_data.remove(0); // Drop oldest
                        },
                        BackpressureStrategy::Block => {
                            return Err(FlowError::BackpressureExceeded);
                        },
                        _ => {} // Other strategies handled in handle_backpressure
                    }
                }
                
                pipeline.pending_data.push(data);
                pipeline.last_transfer = Some(Instant::now());
                
                // Simulate data processing
                pipeline.pending_data.pop(); // Process and remove
                
                return Ok(());
            }
        }
        
        Err(FlowError::PipelineNotFound)
    }
}

impl Default for DataFlowCoordinatorImpl {
    fn default() -> Self {
        Self::new()
    }
} 