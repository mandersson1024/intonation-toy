//! # Audio Data Pipeline Implementation
//!
//! This module provides specialized real-time audio data pipeline management,
//! optimized for low-latency audio processing with sub-2ms coordination overhead.
//! It includes audio buffer flow coordination, priority management, and
//! performance monitoring specifically designed for audio processing workflows.

use std::sync::{Arc, RwLock, Mutex};
use std::collections::{HashMap, VecDeque};
use std::time::{Instant, Duration};
use std::sync::mpsc::{self, Sender, Receiver};
use serde::{Serialize, Deserialize};

use super::data_flow_coordinator::{
    DataFlowCoordinator, DataFlowCoordinatorImpl, FlowData, FlowMetrics,
    PipelineId, ModuleId, PipelinePriority, BackpressureStrategy,
    PipelineHealth, FlowError, DataFlowEvent
};

/// Audio-specific data formats
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioDataFormat {
    F32Array,        // Raw f32 audio samples
    I16Array,        // 16-bit integer samples
    TypedBuffer,     // Typed audio buffer with metadata
    JSCompatible,    // JavaScript-compatible format
    WASMOptimized,   // WASM-optimized binary format
}

/// Audio buffer metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioBufferMetadata {
    pub sample_rate: f32,
    pub channels: u32,
    pub buffer_size: usize,
    pub timestamp: u64,
    pub sequence_number: u64,
    pub latency_budget_ms: f32,
}

/// Real-time audio constraint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeConstraints {
    pub max_latency_ms: f32,
    pub max_jitter_ms: f32,
    pub priority_level: AudioPipelinePriority,
    pub drop_threshold_ms: f32,
    pub recovery_strategy: AudioRecoveryStrategy,
}

impl Default for RealtimeConstraints {
    fn default() -> Self {
        Self {
            max_latency_ms: 2.0,
            max_jitter_ms: 0.5,
            priority_level: AudioPipelinePriority::Critical,
            drop_threshold_ms: 5.0,
            recovery_strategy: AudioRecoveryStrategy::DropAndRecover,
        }
    }
}

/// Audio pipeline priority levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioPipelinePriority {
    Critical,    // Real-time audio processing (<1ms)
    High,        // Audio analysis and effects (<2ms)
    Normal,      // Audio visualization (<5ms)
    Background,  // Audio recording and storage (<10ms)
}

/// Audio recovery strategies for pipeline failures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioRecoveryStrategy {
    DropAndRecover,      // Drop current buffer and recover
    RetryWithFallback,   // Retry with degraded quality
    SilentFill,          // Fill with silence to maintain timing
    EmergencyBypass,     // Bypass processing for emergency recovery
}

/// Audio data pipeline specialized for real-time processing
pub struct AudioDataPipeline {
    /// Base data flow coordinator
    coordinator: DataFlowCoordinatorImpl,
    /// Audio-specific pipeline configurations
    audio_pipelines: Arc<RwLock<HashMap<PipelineId, AudioPipelineConfig>>>,
    /// Real-time constraint enforcement
    constraints: Arc<RwLock<RealtimeConstraints>>,
    /// Audio staging buffers
    staging_buffers: Arc<RwLock<HashMap<PipelineId, VecDeque<AudioStageBuffer>>>>,
    /// Priority queue for audio processing
    priority_queue: Arc<Mutex<Vec<AudioProcessingTask>>>,
    /// Audio performance metrics
    audio_metrics: Arc<RwLock<HashMap<PipelineId, AudioPerformanceMetrics>>>,
    /// Alert system for constraint violations
    alert_sender: Option<Sender<AudioPipelineAlert>>,
}

/// Audio pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPipelineConfig {
    pub pipeline_id: PipelineId,
    pub audio_format: AudioDataFormat,
    pub constraints: RealtimeConstraints,
    pub buffer_count: usize,
    pub prefill_count: usize,
    pub monitoring_enabled: bool,
}

/// Audio staging buffer for real-time processing
#[derive(Debug, Clone)]
pub struct AudioStageBuffer {
    pub data: Vec<f32>,
    pub metadata: AudioBufferMetadata,
    pub arrival_time: Instant,
    pub priority: AudioPipelinePriority,
    pub processing_deadline: Instant,
}

/// Audio processing task for priority queue
#[derive(Debug, Clone)]
pub struct AudioProcessingTask {
    pub pipeline_id: PipelineId,
    pub buffer_id: u64,
    pub priority: AudioPipelinePriority,
    pub deadline: Instant,
    pub retry_count: u32,
}

impl PartialEq for AudioProcessingTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.deadline == other.deadline
    }
}

impl Eq for AudioProcessingTask {}

impl PartialOrd for AudioProcessingTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AudioProcessingTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then earlier deadline
        match self.priority.cmp(&other.priority) {
            std::cmp::Ordering::Equal => other.deadline.cmp(&self.deadline),
            other => other,
        }
    }
}

impl PartialEq for AudioPipelinePriority {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl PartialOrd for AudioPipelinePriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AudioPipelinePriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_val = match self {
            AudioPipelinePriority::Critical => 4,
            AudioPipelinePriority::High => 3,
            AudioPipelinePriority::Normal => 2,
            AudioPipelinePriority::Background => 1,
        };
        let other_val = match other {
            AudioPipelinePriority::Critical => 4,
            AudioPipelinePriority::High => 3,
            AudioPipelinePriority::Normal => 2,
            AudioPipelinePriority::Background => 1,
        };
        self_val.cmp(&other_val)
    }
}

/// Audio performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPerformanceMetrics {
    pub average_latency_ms: f32,
    pub max_latency_ms: f32,
    pub jitter_ms: f32,
    pub drops_per_second: f32,
    pub underruns: u64,
    pub overruns: u64,
    pub constraint_violations: u64,
    pub recovery_events: u64,
    pub throughput_buffers_per_second: f32,
}

impl Default for AudioPerformanceMetrics {
    fn default() -> Self {
        Self {
            average_latency_ms: 0.0,
            max_latency_ms: 0.0,
            jitter_ms: 0.0,
            drops_per_second: 0.0,
            underruns: 0,
            overruns: 0,
            constraint_violations: 0,
            recovery_events: 0,
            throughput_buffers_per_second: 0.0,
        }
    }
}

/// Audio pipeline alerts for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioPipelineAlert {
    LatencyViolation {
        pipeline_id: PipelineId,
        actual_latency_ms: f32,
        max_allowed_ms: f32,
    },
    UnderrunDetected {
        pipeline_id: PipelineId,
        buffer_count: usize,
    },
    OverrunDetected {
        pipeline_id: PipelineId,
        dropped_buffers: usize,
    },
    ConstraintViolation {
        pipeline_id: PipelineId,
        constraint_type: String,
        violation_severity: AlertSeverity,
    },
    RecoveryActivated {
        pipeline_id: PipelineId,
        strategy: AudioRecoveryStrategy,
    },
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

impl AudioDataPipeline {
    /// Create a new audio data pipeline
    pub fn new() -> Self {
        Self {
            coordinator: DataFlowCoordinatorImpl::new(),
            audio_pipelines: Arc::new(RwLock::new(HashMap::new())),
            constraints: Arc::new(RwLock::new(RealtimeConstraints::default())),
            staging_buffers: Arc::new(RwLock::new(HashMap::new())),
            priority_queue: Arc::new(Mutex::new(Vec::new())),
            audio_metrics: Arc::new(RwLock::new(HashMap::new())),
            alert_sender: None,
        }
    }
    
    /// Set alert sender for pipeline monitoring
    pub fn set_alert_sender(&mut self, sender: Sender<AudioPipelineAlert>) {
        self.alert_sender = Some(sender);
    }
    
    /// Register audio pipeline with real-time constraints
    pub fn register_audio_pipeline(
        &mut self,
        source: ModuleId,
        destination: ModuleId,
        audio_format: AudioDataFormat,
        constraints: RealtimeConstraints,
    ) -> Result<PipelineId, FlowError> {
        // Create base pipeline configuration
        let base_config = super::data_flow_coordinator::PipelineConfig {
            buffer_size: 2048,
            max_latency_ms: constraints.max_latency_ms,
            priority: match constraints.priority_level {
                AudioPipelinePriority::Critical => PipelinePriority::Critical,
                AudioPipelinePriority::High => PipelinePriority::High,
                AudioPipelinePriority::Normal => PipelinePriority::Normal,
                AudioPipelinePriority::Background => PipelinePriority::Low,
            },
            backpressure_strategy: match constraints.recovery_strategy {
                AudioRecoveryStrategy::DropAndRecover => BackpressureStrategy::Drop,
                AudioRecoveryStrategy::RetryWithFallback => BackpressureStrategy::Throttle,
                AudioRecoveryStrategy::SilentFill => BackpressureStrategy::Buffer,
                AudioRecoveryStrategy::EmergencyBypass => BackpressureStrategy::Drop,
            },
            transformation_required: true,
        };
        
        // Register with base coordinator
        let pipeline_id = self.coordinator.register_pipeline(source, destination, base_config)?;
        
        // Create audio-specific configuration
        let audio_config = AudioPipelineConfig {
            pipeline_id: pipeline_id.clone(),
            audio_format,
            constraints,
            buffer_count: 4,
            prefill_count: 2,
            monitoring_enabled: true,
        };
        
        // Store audio configuration
        if let Ok(mut pipelines) = self.audio_pipelines.write() {
            pipelines.insert(pipeline_id.clone(), audio_config);
        }
        
        // Initialize staging buffers
        if let Ok(mut buffers) = self.staging_buffers.write() {
            buffers.insert(pipeline_id.clone(), VecDeque::with_capacity(4));
        }
        
        // Initialize audio metrics
        if let Ok(mut metrics) = self.audio_metrics.write() {
            metrics.insert(pipeline_id.clone(), AudioPerformanceMetrics::default());
        }
        
        Ok(pipeline_id)
    }
    
    /// Send audio buffer through pipeline with real-time constraints
    pub fn send_audio_buffer(
        &mut self,
        pipeline_id: PipelineId,
        audio_data: Vec<f32>,
        metadata: AudioBufferMetadata,
    ) -> Result<(), FlowError> {
        let send_start = Instant::now();
        
        // Check if pipeline meets real-time constraints
        if !self.check_realtime_constraints(&pipeline_id)? {
            self.send_alert(AudioPipelineAlert::LatencyViolation {
                pipeline_id: pipeline_id.clone(),
                actual_latency_ms: self.get_current_latency(&pipeline_id),
                max_allowed_ms: self.get_max_latency(&pipeline_id),
            });
            return Err(FlowError::LatencyExceeded);
        }
        
        // Create staging buffer
        let deadline = send_start + Duration::from_millis(metadata.latency_budget_ms as u64);
        let stage_buffer = AudioStageBuffer {
            data: audio_data,
            metadata,
            arrival_time: send_start,
            priority: self.get_pipeline_priority(&pipeline_id),
            processing_deadline: deadline,
        };
        
        // Stage the buffer
        self.stage_audio_buffer(pipeline_id.clone(), stage_buffer)?;
        
        // Process staged buffers
        self.process_staged_buffers()?;
        
        // Update metrics
        let processing_time = send_start.elapsed().as_millis() as f32;
        self.update_audio_metrics(&pipeline_id, processing_time, true);
        
        Ok(())
    }
    
    /// Check if pipeline meets real-time constraints
    fn check_realtime_constraints(&self, pipeline_id: &PipelineId) -> Result<bool, FlowError> {
        if let Ok(metrics) = self.audio_metrics.read() {
            if let Some(metric) = metrics.get(pipeline_id) {
                let max_latency = self.get_max_latency(pipeline_id);
                return Ok(metric.average_latency_ms <= max_latency);
            }
        }
        Err(FlowError::PipelineNotFound)
    }
    
    /// Get current latency for pipeline
    fn get_current_latency(&self, pipeline_id: &PipelineId) -> f32 {
        if let Ok(metrics) = self.audio_metrics.read() {
            metrics.get(pipeline_id)
                .map(|m| m.average_latency_ms)
                .unwrap_or(0.0)
        } else {
            0.0
        }
    }
    
    /// Get maximum allowed latency for pipeline
    fn get_max_latency(&self, pipeline_id: &PipelineId) -> f32 {
        if let Ok(pipelines) = self.audio_pipelines.read() {
            pipelines.get(pipeline_id)
                .map(|p| p.constraints.max_latency_ms)
                .unwrap_or(2.0)
        } else {
            2.0
        }
    }
    
    /// Get pipeline priority
    fn get_pipeline_priority(&self, pipeline_id: &PipelineId) -> AudioPipelinePriority {
        if let Ok(pipelines) = self.audio_pipelines.read() {
            pipelines.get(pipeline_id)
                .map(|p| p.constraints.priority_level.clone())
                .unwrap_or(AudioPipelinePriority::Normal)
        } else {
            AudioPipelinePriority::Normal
        }
    }
    
    /// Stage audio buffer for processing
    fn stage_audio_buffer(
        &mut self,
        pipeline_id: PipelineId,
        buffer: AudioStageBuffer,
    ) -> Result<(), FlowError> {
        if let Ok(mut staging) = self.staging_buffers.write() {
            if let Some(pipeline_buffers) = staging.get_mut(&pipeline_id) {
                // Check for overrun
                if pipeline_buffers.len() >= 8 { // Max staging buffers
                    self.send_alert(AudioPipelineAlert::OverrunDetected {
                        pipeline_id: pipeline_id.clone(),
                        dropped_buffers: 1,
                    });
                    
                    // Drop oldest buffer
                    pipeline_buffers.pop_front();
                }
                
                pipeline_buffers.push_back(buffer);
                return Ok(());
            }
        }
        
        Err(FlowError::PipelineNotFound)
    }
    
    /// Process staged buffers according to priority
    fn process_staged_buffers(&mut self) -> Result<(), FlowError> {
        let mut tasks = Vec::new();
        
        // Collect processing tasks
        if let Ok(staging) = self.staging_buffers.read() {
            for (pipeline_id, buffers) in staging.iter() {
                for (idx, buffer) in buffers.iter().enumerate() {
                    tasks.push(AudioProcessingTask {
                        pipeline_id: pipeline_id.clone(),
                        buffer_id: idx as u64,
                        priority: buffer.priority.clone(),
                        deadline: buffer.processing_deadline,
                        retry_count: 0,
                    });
                }
            }
        }
        
        // Sort by priority and deadline
        tasks.sort();
        
        // Process high-priority tasks first
        for task in tasks.iter().take(10) { // Process up to 10 buffers per cycle
            self.process_audio_task(task)?;
        }
        
        Ok(())
    }
    
    /// Process individual audio task
    fn process_audio_task(&mut self, task: &AudioProcessingTask) -> Result<(), FlowError> {
        let process_start = Instant::now();
        
        // Check deadline
        if process_start > task.deadline {
            self.send_alert(AudioPipelineAlert::LatencyViolation {
                pipeline_id: task.pipeline_id.clone(),
                actual_latency_ms: process_start.duration_since(task.deadline).as_millis() as f32,
                max_allowed_ms: self.get_max_latency(&task.pipeline_id),
            });
            return Err(FlowError::LatencyExceeded);
        }
        
        // Simulate audio processing
        std::thread::sleep(Duration::from_micros(100)); // Simulate processing time
        
        // Remove processed buffer from staging
        if let Ok(mut staging) = self.staging_buffers.write() {
            if let Some(pipeline_buffers) = staging.get_mut(&task.pipeline_id) {
                if !pipeline_buffers.is_empty() {
                    pipeline_buffers.pop_front();
                }
            }
        }
        
        Ok(())
    }
    
    /// Update audio performance metrics
    fn update_audio_metrics(&self, pipeline_id: &PipelineId, latency_ms: f32, success: bool) {
        if let Ok(mut metrics) = self.audio_metrics.write() {
            if let Some(metric) = metrics.get_mut(pipeline_id) {
                // Update latency metrics
                metric.average_latency_ms = 0.9 * metric.average_latency_ms + 0.1 * latency_ms;
                metric.max_latency_ms = metric.max_latency_ms.max(latency_ms);
                
                // Update jitter
                let jitter = (latency_ms - metric.average_latency_ms).abs();
                metric.jitter_ms = 0.9 * metric.jitter_ms + 0.1 * jitter;
                
                // Update throughput
                metric.throughput_buffers_per_second += 1.0;
                
                if !success {
                    metric.constraint_violations += 1;
                }
            }
        }
    }
    
    /// Send alert if sender is configured
    fn send_alert(&self, alert: AudioPipelineAlert) {
        if let Some(sender) = &self.alert_sender {
            let _ = sender.send(alert);
        }
    }
    
    /// Get audio performance metrics for pipeline
    pub fn get_audio_metrics(&self, pipeline_id: &PipelineId) -> Option<AudioPerformanceMetrics> {
        if let Ok(metrics) = self.audio_metrics.read() {
            metrics.get(pipeline_id).cloned()
        } else {
            None
        }
    }
    
    /// Start audio pipeline processing
    pub fn start(&mut self) -> Result<(), FlowError> {
        self.coordinator.start()
    }
    
    /// Stop audio pipeline processing
    pub fn stop(&mut self) -> Result<(), FlowError> {
        self.coordinator.stop()
    }
}

impl Default for AudioDataPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audio_pipeline_creation() {
        let pipeline = AudioDataPipeline::new();
        assert!(pipeline.audio_pipelines.read().unwrap().is_empty());
    }
    
    #[test]
    fn test_audio_pipeline_registration() {
        let mut pipeline = AudioDataPipeline::new();
        let constraints = RealtimeConstraints::default();
        
        let result = pipeline.register_audio_pipeline(
            ModuleId::AudioFoundations,
            ModuleId::DataManagement,
            AudioDataFormat::F32Array,
            constraints,
        );
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_audio_buffer_metadata() {
        let metadata = AudioBufferMetadata {
            sample_rate: 44100.0,
            channels: 2,
            buffer_size: 1024,
            timestamp: 12345,
            sequence_number: 1,
            latency_budget_ms: 2.0,
        };
        
        assert_eq!(metadata.sample_rate, 44100.0);
        assert_eq!(metadata.channels, 2);
    }
    
    #[test]
    fn test_priority_ordering() {
        let critical = AudioPipelinePriority::Critical;
        let high = AudioPipelinePriority::High;
        
        assert!(critical > high);
    }
}