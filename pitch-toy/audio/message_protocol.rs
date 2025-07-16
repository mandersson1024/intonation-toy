// Structured message protocol for AudioWorklet communication
// Provides type-safe message construction and parsing for cross-thread communication

use crate::audio::test_signal_generator::{TestSignalGeneratorConfig, BackgroundNoiseConfig};

/// Message types sent from main thread to AudioWorklet
#[derive(Debug, Clone, PartialEq)]
pub enum ToWorkletMessage {
    /// Start audio processing
    StartProcessing,
    
    /// Stop audio processing
    StopProcessing,
    
    /// Update test signal configuration
    UpdateTestSignalConfig {
        config: TestSignalGeneratorConfig,
    },
    
    /// Update batch processing configuration
    UpdateBatchConfig {
        config: BatchConfig,
    },
    
    /// Update background noise configuration
    UpdateBackgroundNoiseConfig {
        config: BackgroundNoiseConfig,
    },
}

/// Message types sent from AudioWorklet to main thread
#[derive(Debug, Clone, PartialEq)]
pub enum FromWorkletMessage {
    /// AudioWorklet processor is ready
    ProcessorReady {
        batch_size: Option<usize>,
    },
    
    /// Processing has started
    ProcessingStarted,
    
    /// Processing has stopped
    ProcessingStopped,
    
    /// Audio data batch with transferable buffer
    AudioDataBatch {
        data: AudioDataBatch,
    },
    
    /// Processing error occurred
    ProcessingError {
        error: WorkletError,
    },
    
    /// Processor status update
    StatusUpdate {
        status: ProcessorStatus,
    },
}

/// Audio data batch structure for transferable buffer communication
#[derive(Debug, Clone, PartialEq)]
pub struct AudioDataBatch {
    /// Sample rate of the audio data
    pub sample_rate: f64,
    
    /// Number of samples in the batch
    pub sample_count: usize,
    
    /// Buffer length in bytes
    pub buffer_length: usize,
    
    /// Timestamp when batch was created
    pub timestamp: f64,
    
    /// Optional batch sequence number
    pub sequence_number: Option<u32>,
}

/// Processor status information
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessorStatus {
    /// Whether processor is currently active
    pub active: bool,
    
    /// Current sample rate
    pub sample_rate: f64,
    
    /// Current buffer size
    pub buffer_size: usize,
    
    /// Number of processed batches
    pub processed_batches: u32,
    
    /// Average processing time in milliseconds
    pub avg_processing_time_ms: f64,
    
    /// Memory usage information
    pub memory_usage: Option<MemoryUsage>,
}

/// Memory usage information
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryUsage {
    /// Heap size in bytes
    pub heap_size: usize,
    
    /// Used heap in bytes
    pub used_heap: usize,
    
    /// Number of active buffers
    pub active_buffers: usize,
}

/// Batch processing configuration
#[derive(Debug, Clone, PartialEq)]
pub struct BatchConfig {
    /// Size of each batch in samples
    pub batch_size: usize,
    
    /// Maximum number of batches to queue
    pub max_queue_size: usize,
    
    /// Timeout for batch processing in milliseconds
    pub timeout_ms: u32,
    
    /// Enable batch compression
    pub enable_compression: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 1024,
            max_queue_size: 8,
            timeout_ms: 100,
            enable_compression: false,
        }
    }
}

/// Structured error information for worklet errors
#[derive(Debug, Clone, PartialEq)]
pub struct WorkletError {
    /// Error code for categorization
    pub code: WorkletErrorCode,
    
    /// Human-readable error message
    pub message: String,
    
    /// Additional context information
    pub context: Option<ErrorContext>,
    
    /// Timestamp when error occurred
    pub timestamp: f64,
}

/// Error codes for worklet errors
#[derive(Debug, Clone, PartialEq)]
pub enum WorkletErrorCode {
    /// Initialization failed
    InitializationFailed,
    
    /// Processing failed
    ProcessingFailed,
    
    /// Buffer overflow
    BufferOverflow,
    
    /// Invalid configuration
    InvalidConfiguration,
    
    /// Memory allocation failed
    MemoryAllocationFailed,
    
    /// Generic error
    Generic,
}

/// Error context for additional debugging information
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorContext {
    /// Function or module where error occurred
    pub location: String,
    
    /// System state at time of error
    pub system_state: Option<String>,
    
    /// Additional debug information
    pub debug_info: Option<String>,
}

/// Message envelope with correlation and timing information
#[derive(Debug, Clone, PartialEq)]
pub struct MessageEnvelope<T> {
    /// Unique message identifier for correlation
    pub message_id: u32,
    
    /// Timestamp when message was created
    pub timestamp: f64,
    
    /// The actual message payload
    pub payload: T,
}

/// Unified message type for main thread to worklet communication
pub type ToWorkletEnvelope = MessageEnvelope<ToWorkletMessage>;

/// Unified message type for worklet to main thread communication
pub type FromWorkletEnvelope = MessageEnvelope<FromWorkletMessage>;

impl<T> MessageEnvelope<T> {
    /// Create a new message envelope with the given payload
    pub fn new(payload: T) -> Self {
        Self {
            message_id: generate_message_id(),
            timestamp: get_current_timestamp(),
            payload,
        }
    }
    
    /// Create a new message envelope with a specific message ID
    pub fn with_id(payload: T, message_id: u32) -> Self {
        Self {
            message_id,
            timestamp: get_current_timestamp(),
            payload,
        }
    }
}

/// Generate a unique message ID
fn generate_message_id() -> u32 {
    // Simple counter-based ID generation
    // In a real implementation, this would use atomic operations
    static mut COUNTER: u32 = 0;
    unsafe {
        COUNTER += 1;
        COUNTER
    }
}

/// Get current timestamp in milliseconds
fn get_current_timestamp() -> f64 {
    // Use performance.now() for high-resolution timing
    js_sys::Date::now()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_message_envelope_creation() {
        let payload = ToWorkletMessage::StartProcessing;
        let envelope = MessageEnvelope::new(payload.clone());
        
        assert_eq!(envelope.payload, payload);
        assert!(envelope.message_id > 0);
        assert!(envelope.timestamp > 0.0);
    }

    #[wasm_bindgen_test]
    fn test_message_envelope_with_id() {
        let payload = FromWorkletMessage::ProcessorReady { batch_size: Some(1024) };
        let envelope = MessageEnvelope::with_id(payload.clone(), 42);
        
        assert_eq!(envelope.payload, payload);
        assert_eq!(envelope.message_id, 42);
        assert!(envelope.timestamp > 0.0);
    }

    #[wasm_bindgen_test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        
        assert_eq!(config.batch_size, 1024);
        assert_eq!(config.max_queue_size, 8);
        assert_eq!(config.timeout_ms, 100);
        assert_eq!(config.enable_compression, false);
    }

    #[wasm_bindgen_test]
    fn test_worklet_error_creation() {
        let error = WorkletError {
            code: WorkletErrorCode::BufferOverflow,
            message: "Buffer overflow detected".to_string(),
            context: Some(ErrorContext {
                location: "audio_processor.rs:123".to_string(),
                system_state: Some("processing=true, queue_size=10".to_string()),
                debug_info: None,
            }),
            timestamp: get_current_timestamp(),
        };
        
        assert_eq!(error.code, WorkletErrorCode::BufferOverflow);
        assert_eq!(error.message, "Buffer overflow detected");
        assert!(error.context.is_some());
        assert!(error.timestamp > 0.0);
    }

    #[wasm_bindgen_test]
    fn test_audio_data_batch_creation() {
        let batch = AudioDataBatch {
            sample_rate: 48000.0,
            sample_count: 1024,
            buffer_length: 4096,
            timestamp: get_current_timestamp(),
            sequence_number: Some(42),
        };
        
        assert_eq!(batch.sample_rate, 48000.0);
        assert_eq!(batch.sample_count, 1024);
        assert_eq!(batch.buffer_length, 4096);
        assert_eq!(batch.sequence_number, Some(42));
        assert!(batch.timestamp > 0.0);
    }

    #[wasm_bindgen_test]
    fn test_processor_status_creation() {
        let status = ProcessorStatus {
            active: true,
            sample_rate: 48000.0,
            buffer_size: 1024,
            processed_batches: 100,
            avg_processing_time_ms: 5.2,
            memory_usage: Some(MemoryUsage {
                heap_size: 1024 * 1024,
                used_heap: 512 * 1024,
                active_buffers: 8,
            }),
        };
        
        assert_eq!(status.active, true);
        assert_eq!(status.sample_rate, 48000.0);
        assert_eq!(status.buffer_size, 1024);
        assert_eq!(status.processed_batches, 100);
        assert_eq!(status.avg_processing_time_ms, 5.2);
        assert!(status.memory_usage.is_some());
    }

    #[wasm_bindgen_test]
    fn test_message_types_enum_variants() {
        // Test ToWorkletMessage variants
        let start_msg = ToWorkletMessage::StartProcessing;
        let stop_msg = ToWorkletMessage::StopProcessing;
        assert_ne!(start_msg, stop_msg);
        
        // Test FromWorkletMessage variants
        let ready_msg = FromWorkletMessage::ProcessorReady { batch_size: Some(1024) };
        let started_msg = FromWorkletMessage::ProcessingStarted;
        assert_ne!(ready_msg, started_msg);
    }
}