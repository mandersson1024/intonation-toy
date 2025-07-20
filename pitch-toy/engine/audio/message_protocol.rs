// Structured message protocol for AudioWorklet communication
// Provides type-safe message construction and parsing for cross-thread communication
//
// Features:
// - ReturnBuffer messages for ping-pong buffer recycling
// - Structured message envelopes with IDs and timestamps
// - Serialization/deserialization to/from JavaScript objects
// - Message validation and error handling
//
// Usage:
//   let factory = AudioWorkletMessageFactory::new();
//   let return_msg = factory.return_buffer(buffer_id)?;
//   let serializer = MessageSerializer::new();
//   let js_message = serializer.serialize_envelope(&return_msg)?;

use crate::engine::audio::test_signal_generator::{TestSignalGeneratorConfig, BackgroundNoiseConfig};
use js_sys::{Object, Reflect};
use wasm_bindgen::{JsValue, JsCast};

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
    
    /// Return buffer to worklet for recycling
    ReturnBuffer {
        buffer_id: u32,
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
    
    /// Buffer ID for ping-pong pattern
    pub buffer_id: Option<u32>,
    
    /// Buffer pool statistics bundled with the audio data
    pub buffer_pool_stats: Option<BufferPoolStats>,
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
    
    /// Buffer pool statistics
    pub buffer_pool_stats: Option<BufferPoolStats>,
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

/// Buffer pool statistics
#[derive(Debug, Clone, PartialEq)]
pub struct BufferPoolStats {
    /// Total number of buffers in the pool
    pub pool_size: u32,
    
    /// Number of available buffers
    pub available_buffers: u32,
    
    /// Number of buffers currently in use
    pub in_use_buffers: u32,
    
    /// Total number of buffers
    pub total_buffers: u32,
    
    /// Number of acquire attempts
    pub acquire_count: u32,
    
    /// Number of successful transfers
    pub transfer_count: u32,
    
    /// Number of times pool was exhausted
    pub pool_exhausted_count: u32,
    
    /// Consecutive pool failures
    pub consecutive_pool_failures: u32,
    
    /// Pool hit rate percentage
    pub pool_hit_rate: f32,
    
    /// Pool efficiency percentage
    pub pool_efficiency: f32,
    
    /// Buffer utilization percentage
    pub buffer_utilization_percent: f32,
    
    /// Total megabytes transferred
    pub total_megabytes_transferred: f32,
    
    /// Average acquisition time in milliseconds
    pub avg_acquisition_time_ms: f32,
    
    /// Fastest acquisition time in milliseconds
    pub fastest_acquisition_time_ms: f32,
    
    /// Slowest acquisition time in milliseconds
    pub slowest_acquisition_time_ms: f32,
    
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

impl std::fmt::Display for WorkletError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::fmt::Display for WorkletErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkletErrorCode::InitializationFailed => write!(f, "Initialization failed"),
            WorkletErrorCode::ProcessingFailed => write!(f, "Processing failed"),
            WorkletErrorCode::BufferOverflow => write!(f, "Buffer overflow"),
            WorkletErrorCode::InvalidConfiguration => write!(f, "Invalid configuration"),
            WorkletErrorCode::MemoryAllocationFailed => write!(f, "Memory allocation failed"),
            WorkletErrorCode::Generic => write!(f, "Generic error"),
        }
    }
}

impl std::error::Error for WorkletError {}

/// Enhanced error context for detailed debugging information
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorContext {
    /// Function or module where error occurred
    pub location: String,
    
    /// Stack trace information (when available)
    pub stack_trace: Option<Vec<String>>,
    
    /// Message context information
    pub message_context: Option<MessageContext>,
    
    /// System state at time of error
    pub system_state: Option<SystemState>,
    
    /// Additional debug information
    pub debug_info: Option<String>,
    
    /// Error timestamp (high precision)
    pub timestamp: f64,
    
    /// Thread or context identifier
    pub thread_id: Option<String>,
}

/// Message context information for error reporting
#[derive(Debug, Clone, PartialEq)]
pub struct MessageContext {
    /// Message type or identifier
    pub message_type: String,
    
    /// Message direction (ToWorklet, FromWorklet)
    pub direction: MessageDirection,
    
    /// Message ID if available
    pub message_id: Option<u32>,
    
    /// Message timestamp
    pub message_timestamp: Option<f64>,
    
    /// Message size in bytes
    pub message_size: Option<usize>,
}

/// Message direction for context
#[derive(Debug, Clone, PartialEq)]
pub enum MessageDirection {
    /// Message sent to worklet
    ToWorklet,
    /// Message sent from worklet
    FromWorklet,
    /// Internal message processing
    Internal,
}

/// System state information for error context
#[derive(Debug, Clone, PartialEq)]
pub struct SystemState {
    /// Current memory usage in bytes
    pub memory_usage: Option<usize>,
    
    /// Message queue depth
    pub queue_depth: Option<usize>,
    
    /// Active buffer count
    pub active_buffers: Option<usize>,
    
    /// Audio processing status
    pub audio_processing_active: Option<bool>,
    
    /// Sample rate
    pub sample_rate: Option<f64>,
    
    /// Buffer size
    pub buffer_size: Option<usize>,
    
    /// Processor load percentage (0-100)
    pub processor_load: Option<f32>,
    
    /// Available heap memory
    pub available_heap: Option<usize>,
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
            message_id: generate_unique_message_id(),
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


/// Get current timestamp in milliseconds
fn get_current_timestamp() -> f64 {
    // Use performance.now() for high-resolution timing
    js_sys::Date::now()
}

// ================================
// Serialization/Deserialization System
// ================================

/// Result type for serialization operations
pub type SerializationResult<T> = Result<T, SerializationError>;

/// Serialization error types
#[derive(Debug, Clone, PartialEq)]
pub enum SerializationError {
    /// Failed to create JavaScript object
    ObjectCreationFailed(String),
    /// Failed to set property on JavaScript object
    PropertySetFailed(String),
    /// Failed to get property from JavaScript object
    PropertyGetFailed(String),
    /// Invalid property type
    InvalidPropertyType(String),
    /// Missing required property
    MissingProperty(String),
    /// Validation failed
    ValidationFailed(String),
    /// Buffer transfer failed
    BufferTransferFailed(String),
}

impl std::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializationError::ObjectCreationFailed(msg) => write!(f, "Object creation failed: {}", msg),
            SerializationError::PropertySetFailed(msg) => write!(f, "Property set failed: {}", msg),
            SerializationError::PropertyGetFailed(msg) => write!(f, "Property get failed: {}", msg),
            SerializationError::InvalidPropertyType(msg) => write!(f, "Invalid property type: {}", msg),
            SerializationError::MissingProperty(msg) => write!(f, "Missing property: {}", msg),
            SerializationError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            SerializationError::BufferTransferFailed(msg) => write!(f, "Buffer transfer failed: {}", msg),
        }
    }
}

impl std::error::Error for SerializationError {}

/// Protocol-specific error types for message handling
#[derive(Debug, Clone, PartialEq)]
pub enum MessageProtocolError {
    /// Serialization error
    Serialization(SerializationError),
    /// Message validation error
    Validation(ValidationError),
    /// Buffer transfer error
    Transfer(TransferError),
    /// Message construction error
    Construction(MessageConstructionError),
    /// Worklet processing error
    Worklet(WorkletError),
}

impl std::fmt::Display for MessageProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageProtocolError::Serialization(err) => write!(f, "Serialization error: {}", err),
            MessageProtocolError::Validation(err) => write!(f, "Validation error: {}", err),
            MessageProtocolError::Transfer(err) => write!(f, "Transfer error: {}", err),
            MessageProtocolError::Construction(err) => write!(f, "Construction error: {}", err),
            MessageProtocolError::Worklet(err) => write!(f, "Worklet error: {}", err),
        }
    }
}

impl std::error::Error for MessageProtocolError {}

impl From<SerializationError> for MessageProtocolError {
    fn from(err: SerializationError) -> Self {
        MessageProtocolError::Serialization(err)
    }
}

impl From<ValidationError> for MessageProtocolError {
    fn from(err: ValidationError) -> Self {
        MessageProtocolError::Validation(err)
    }
}

impl From<TransferError> for MessageProtocolError {
    fn from(err: TransferError) -> Self {
        MessageProtocolError::Transfer(err)
    }
}

impl From<MessageConstructionError> for MessageProtocolError {
    fn from(err: MessageConstructionError) -> Self {
        MessageProtocolError::Construction(err)
    }
}

impl From<WorkletError> for MessageProtocolError {
    fn from(err: WorkletError) -> Self {
        MessageProtocolError::Worklet(err)
    }
}

/// Validation error types for message validation
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Field validation failed
    FieldValidation { field: String, reason: String },
    /// Value out of range
    ValueOutOfRange { field: String, value: String, min: Option<String>, max: Option<String> },
    /// Invalid message type
    InvalidMessageType(String),
    /// Missing required field
    MissingRequiredField(String),
    /// Conflicting configuration
    ConflictingConfiguration(String),
    /// Unsupported message version
    UnsupportedVersion { expected: String, received: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::FieldValidation { field, reason } => 
                write!(f, "Field '{}' validation failed: {}", field, reason),
            ValidationError::ValueOutOfRange { field, value, min, max } => {
                let range = match (min, max) {
                    (Some(min), Some(max)) => format!(" (expected {} to {})", min, max),
                    (Some(min), None) => format!(" (expected >= {})", min),
                    (None, Some(max)) => format!(" (expected <= {})", max),
                    (None, None) => String::new(),
                };
                write!(f, "Field '{}' value '{}' out of range{}", field, value, range)
            },
            ValidationError::InvalidMessageType(msg_type) => 
                write!(f, "Invalid message type: {}", msg_type),
            ValidationError::MissingRequiredField(field) => 
                write!(f, "Missing required field: {}", field),
            ValidationError::ConflictingConfiguration(msg) => 
                write!(f, "Conflicting configuration: {}", msg),
            ValidationError::UnsupportedVersion { expected, received } => 
                write!(f, "Unsupported message version: expected {}, received {}", expected, received),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Transfer error types for buffer transfer operations
#[derive(Debug, Clone, PartialEq)]
pub enum TransferError {
    /// Buffer allocation failed
    BufferAllocation { size: usize, reason: String },
    /// Buffer transfer failed
    BufferTransfer { buffer_id: Option<String>, reason: String },
    /// Buffer validation failed
    BufferValidation { buffer_id: Option<String>, reason: String },
    /// Buffer size mismatch
    BufferSizeMismatch { expected: usize, actual: usize },
    /// Transferable object creation failed
    TransferableCreation(String),
    /// Buffer pool exhausted
    BufferPoolExhausted { requested_size: usize, available_memory: Option<usize> },
    /// Buffer ownership violation
    BufferOwnershipViolation { buffer_id: String, current_owner: String },
}

impl std::fmt::Display for TransferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferError::BufferAllocation { size, reason } => 
                write!(f, "Buffer allocation failed for {} bytes: {}", size, reason),
            TransferError::BufferTransfer { buffer_id, reason } => {
                let id_str = buffer_id.as_deref().unwrap_or("unknown");
                write!(f, "Buffer transfer failed for buffer '{}': {}", id_str, reason)
            },
            TransferError::BufferValidation { buffer_id, reason } => {
                let id_str = buffer_id.as_deref().unwrap_or("unknown");
                write!(f, "Buffer validation failed for buffer '{}': {}", id_str, reason)
            },
            TransferError::BufferSizeMismatch { expected, actual } => 
                write!(f, "Buffer size mismatch: expected {} bytes, got {} bytes", expected, actual),
            TransferError::TransferableCreation(reason) => 
                write!(f, "Transferable object creation failed: {}", reason),
            TransferError::BufferPoolExhausted { requested_size, available_memory } => {
                match available_memory {
                    Some(available) => write!(f, "Buffer pool exhausted: requested {} bytes, {} bytes available", requested_size, available),
                    None => write!(f, "Buffer pool exhausted: requested {} bytes", requested_size),
                }
            },
            TransferError::BufferOwnershipViolation { buffer_id, current_owner } => 
                write!(f, "Buffer ownership violation: buffer '{}' is owned by '{}'", buffer_id, current_owner),
        }
    }
}

impl std::error::Error for TransferError {}

/// Result type for message protocol operations
pub type MessageProtocolResult<T> = Result<T, MessageProtocolError>;

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Result type for transfer operations
pub type TransferResult<T> = Result<T, TransferError>;

/// Trait for converting Rust types to JavaScript objects
pub trait ToJsMessage {
    /// Convert to JavaScript object
    fn to_js_object(&self) -> SerializationResult<Object>;
    
    /// Convert to JavaScript value
    fn to_js_value(&self) -> SerializationResult<JsValue> {
        self.to_js_object().map(|obj| obj.into())
    }
}

/// Trait for converting JavaScript objects to Rust types
pub trait FromJsMessage: Sized {
    /// Convert from JavaScript object
    fn from_js_object(obj: &Object) -> SerializationResult<Self>;
    
    /// Convert from JavaScript value
    fn from_js_value(value: &JsValue) -> SerializationResult<Self> {
        let obj = value.dyn_ref::<Object>()
            .ok_or_else(|| SerializationError::InvalidPropertyType("Expected object".to_string()))?;
        Self::from_js_object(obj)
    }
}

/// Trait for message validation
pub trait MessageValidator {
    /// Validate message structure and contents
    fn validate(&self) -> SerializationResult<()>;
}

/// Message serializer for efficient serialization
pub struct MessageSerializer;

impl MessageSerializer {
    /// Create a new message serializer
    pub fn new() -> Self {
        Self
    }
    
    /// Serialize a message envelope to JavaScript
    pub fn serialize_envelope<T: ToJsMessage + MessageValidator>(
        &self,
        envelope: &MessageEnvelope<T>,
    ) -> SerializationResult<Object> {
        // Validate the message first
        envelope.payload.validate()?;
        
        let obj = Object::new();
        
        // Set envelope metadata
        self.set_property(&obj, "messageId", &envelope.message_id.into())?;
        self.set_property(&obj, "timestamp", &envelope.timestamp.into())?;
        
        // Serialize the payload
        let payload_obj = envelope.payload.to_js_object()?;
        self.set_property(&obj, "payload", &payload_obj.into())?;
        
        Ok(obj)
    }
    
    /// Helper method to set object properties
    fn set_property(&self, obj: &Object, key: &str, value: &JsValue) -> SerializationResult<()> {
        Reflect::set(obj, &key.into(), value)
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set '{}': {:?}", key, e)))?;
        Ok(())
    }
}

/// Message deserializer for efficient deserialization
pub struct MessageDeserializer;

impl MessageDeserializer {
    /// Create a new message deserializer
    pub fn new() -> Self {
        Self
    }
    
    /// Deserialize a message envelope from JavaScript
    pub fn deserialize_envelope<T: FromJsMessage + MessageValidator>(
        &self,
        obj: &Object,
    ) -> SerializationResult<MessageEnvelope<T>> {
        let message_id = self.get_u32_property(obj, "messageId")?;
        let timestamp = self.get_f64_property(obj, "timestamp")?;
        
        let payload_obj = self.get_object_property(obj, "payload")?;
        let payload = T::from_js_object(&payload_obj)?;
        
        // Validate the deserialized message
        payload.validate()?;
        
        Ok(MessageEnvelope {
            message_id,
            timestamp,
            payload,
        })
    }
    
    /// Helper method to get u32 property
    fn get_u32_property(&self, obj: &Object, key: &str) -> SerializationResult<u32> {
        let value = self.get_property(obj, key)?;
        value.as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType(format!("Property '{}' is not a number", key)))
            .map(|n| n as u32)
    }
    
    /// Helper method to get f64 property
    fn get_f64_property(&self, obj: &Object, key: &str) -> SerializationResult<f64> {
        let value = self.get_property(obj, key)?;
        value.as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType(format!("Property '{}' is not a number", key)))
    }
    
    
    /// Helper method to get object property
    fn get_object_property(&self, obj: &Object, key: &str) -> SerializationResult<Object> {
        let value = self.get_property(obj, key)?;
        value.dyn_into::<Object>()
            .map_err(|_| SerializationError::InvalidPropertyType(format!("Property '{}' is not an object", key)))
    }
    
    
    /// Helper method to get property from object
    fn get_property(&self, obj: &Object, key: &str) -> SerializationResult<JsValue> {
        Reflect::get(obj, &key.into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get '{}': {:?}", key, e)))
            .and_then(|value| {
                if value.is_undefined() {
                    Err(SerializationError::MissingProperty(key.to_string()))
                } else {
                    Ok(value)
                }
            })
    }
    
}

// ================================
// Message Type Implementations
// ================================

// ToWorkletMessage implementations
impl ToJsMessage for ToWorkletMessage {
    fn to_js_object(&self) -> SerializationResult<Object> {
        let obj = Object::new();
        
        match self {
            ToWorkletMessage::StartProcessing => {
                Reflect::set(&obj, &"type".into(), &"startProcessing".into())
                    .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set type: {:?}", e)))?;
            }
            ToWorkletMessage::StopProcessing => {
                Reflect::set(&obj, &"type".into(), &"stopProcessing".into())
                    .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set type: {:?}", e)))?;
            }
            ToWorkletMessage::UpdateTestSignalConfig { config } => {
                Reflect::set(&obj, &"type".into(), &"updateTestSignalConfig".into())
                    .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set type: {:?}", e)))?;
                let config_obj = config.to_js_object()?;
                Reflect::set(&obj, &"config".into(), &config_obj.into())
                    .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set config: {:?}", e)))?;
            }
            ToWorkletMessage::UpdateBatchConfig { config } => {
                Reflect::set(&obj, &"type".into(), &"updateBatchConfig".into())
                    .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set type: {:?}", e)))?;
                let config_obj = config.to_js_object()?;
                Reflect::set(&obj, &"config".into(), &config_obj.into())
                    .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set config: {:?}", e)))?;
            }
            ToWorkletMessage::UpdateBackgroundNoiseConfig { config } => {
                Reflect::set(&obj, &"type".into(), &"updateBackgroundNoiseConfig".into())
                    .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set type: {:?}", e)))?;
                let config_obj = config.to_js_object()?;
                Reflect::set(&obj, &"config".into(), &config_obj.into())
                    .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set config: {:?}", e)))?;
            }
            ToWorkletMessage::ReturnBuffer { buffer_id } => {
                Reflect::set(&obj, &"type".into(), &"returnBuffer".into())
                    .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set type: {:?}", e)))?;
                Reflect::set(&obj, &"bufferId".into(), &(*buffer_id).into())
                    .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set bufferId: {:?}", e)))?;
            }
        }
        
        Ok(obj)
    }
}

impl FromJsMessage for ToWorkletMessage {
    fn from_js_object(obj: &Object) -> SerializationResult<Self> {
        let msg_type = Reflect::get(obj, &"type".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get type: {:?}", e)))?
            .as_string()
            .ok_or_else(|| SerializationError::InvalidPropertyType("type must be string".to_string()))?;
        
        match msg_type.as_str() {
            "startProcessing" => Ok(ToWorkletMessage::StartProcessing),
            "stopProcessing" => Ok(ToWorkletMessage::StopProcessing),
            "updateTestSignalConfig" => {
                let config_obj = Reflect::get(obj, &"config".into())
                    .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get config: {:?}", e)))?
                    .dyn_into::<Object>()
                    .map_err(|_| SerializationError::InvalidPropertyType("config must be object".to_string()))?;
                let config = TestSignalGeneratorConfig::from_js_object(&config_obj)?;
                Ok(ToWorkletMessage::UpdateTestSignalConfig { config })
            }
            "updateBatchConfig" => {
                let config_obj = Reflect::get(obj, &"config".into())
                    .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get config: {:?}", e)))?
                    .dyn_into::<Object>()
                    .map_err(|_| SerializationError::InvalidPropertyType("config must be object".to_string()))?;
                let config = BatchConfig::from_js_object(&config_obj)?;
                Ok(ToWorkletMessage::UpdateBatchConfig { config })
            }
            "updateBackgroundNoiseConfig" => {
                let config_obj = Reflect::get(obj, &"config".into())
                    .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get config: {:?}", e)))?
                    .dyn_into::<Object>()
                    .map_err(|_| SerializationError::InvalidPropertyType("config must be object".to_string()))?;
                let config = BackgroundNoiseConfig::from_js_object(&config_obj)?;
                Ok(ToWorkletMessage::UpdateBackgroundNoiseConfig { config })
            }
            "returnBuffer" => {
                let buffer_id = Reflect::get(obj, &"bufferId".into())
                    .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get bufferId: {:?}", e)))?
                    .as_f64()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("bufferId must be number".to_string()))?;
                Ok(ToWorkletMessage::ReturnBuffer { buffer_id: buffer_id as u32 })
            }
            _ => Err(SerializationError::InvalidPropertyType(format!("Unknown message type: {}", msg_type))),
        }
    }
}

impl MessageValidator for ToWorkletMessage {
    fn validate(&self) -> SerializationResult<()> {
        match self {
            ToWorkletMessage::StartProcessing | ToWorkletMessage::StopProcessing => Ok(()),
            ToWorkletMessage::UpdateTestSignalConfig { config } => config.validate(),
            ToWorkletMessage::UpdateBatchConfig { config } => config.validate(),
            ToWorkletMessage::UpdateBackgroundNoiseConfig { config } => config.validate(),
            ToWorkletMessage::ReturnBuffer { buffer_id: _ } => Ok(()),
        }
    }
}

// FromWorkletMessage implementations
impl ToJsMessage for FromWorkletMessage {
    fn to_js_object(&self) -> SerializationResult<Object> {
        let obj = Object::new();
        
        match self {
            FromWorkletMessage::ProcessorReady { batch_size } => {
                Reflect::set(&obj, &"type".into(), &"processorReady".into())
                    .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set type: {:?}", e)))?;
                if let Some(size) = batch_size {
                    Reflect::set(&obj, &"batchSize".into(), &(*size as f64).into())
                        .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set batchSize: {:?}", e)))?;
                }
            }
            FromWorkletMessage::ProcessingStarted => {
                Reflect::set(&obj, &"type".into(), &"processingStarted".into())
                    .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set type: {:?}", e)))?;
            }
            FromWorkletMessage::ProcessingStopped => {
                Reflect::set(&obj, &"type".into(), &"processingStopped".into())
                    .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set type: {:?}", e)))?;
            }
            FromWorkletMessage::AudioDataBatch { data } => {
                Reflect::set(&obj, &"type".into(), &"audioDataBatch".into())
                    .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set type: {:?}", e)))?;
                let data_obj = data.to_js_object()?;
                Reflect::set(&obj, &"data".into(), &data_obj.into())
                    .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set data: {:?}", e)))?;
            }
            FromWorkletMessage::ProcessingError { error } => {
                Reflect::set(&obj, &"type".into(), &"processingError".into())
                    .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set type: {:?}", e)))?;
                let error_obj = error.to_js_object()?;
                Reflect::set(&obj, &"error".into(), &error_obj.into())
                    .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set error: {:?}", e)))?;
            }
        }
        
        Ok(obj)
    }
}

impl FromJsMessage for FromWorkletMessage {
    fn from_js_object(obj: &Object) -> SerializationResult<Self> {
        let msg_type = Reflect::get(obj, &"type".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get type: {:?}", e)))?
            .as_string()
            .ok_or_else(|| SerializationError::InvalidPropertyType("type must be string".to_string()))?;
        
        match msg_type.as_str() {
            "processorReady" => {
                let batch_size = match Reflect::get(obj, &"batchSize".into()) {
                    Ok(value) if !value.is_undefined() => {
                        Some(value.as_f64()
                            .ok_or_else(|| SerializationError::InvalidPropertyType("batchSize must be number".to_string()))?
                            as usize)
                    }
                    _ => None,
                };
                Ok(FromWorkletMessage::ProcessorReady { batch_size })
            }
            "processingStarted" => Ok(FromWorkletMessage::ProcessingStarted),
            "processingStopped" => Ok(FromWorkletMessage::ProcessingStopped),
            "audioDataBatch" => {
                let data_obj = Reflect::get(obj, &"data".into())
                    .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get data: {:?}", e)))?
                    .dyn_into::<Object>()
                    .map_err(|_| SerializationError::InvalidPropertyType("data must be object".to_string()))?;
                let data = AudioDataBatch::from_js_object(&data_obj)?;
                Ok(FromWorkletMessage::AudioDataBatch { data })
            }
            "processingError" => {
                let error_obj = Reflect::get(obj, &"error".into())
                    .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get error: {:?}", e)))?
                    .dyn_into::<Object>()
                    .map_err(|_| SerializationError::InvalidPropertyType("error must be object".to_string()))?;
                let error = WorkletError::from_js_object(&error_obj)?;
                Ok(FromWorkletMessage::ProcessingError { error })
            }
            _ => Err(SerializationError::InvalidPropertyType(format!("Unknown message type: {}", msg_type))),
        }
    }
}

impl MessageValidator for FromWorkletMessage {
    fn validate(&self) -> SerializationResult<()> {
        match self {
            FromWorkletMessage::ProcessorReady { batch_size } => {
                if let Some(size) = batch_size {
                    if *size == 0 {
                        return Err(SerializationError::ValidationFailed("batch_size cannot be zero".to_string()));
                    }
                }
                Ok(())
            }
            FromWorkletMessage::ProcessingStarted | FromWorkletMessage::ProcessingStopped => Ok(()),
            FromWorkletMessage::AudioDataBatch { data } => data.validate(),
            FromWorkletMessage::ProcessingError { error } => error.validate(),
        }
    }
}

// Data structure implementations
impl ToJsMessage for AudioDataBatch {
    fn to_js_object(&self) -> SerializationResult<Object> {
        let obj = Object::new();
        
        Reflect::set(&obj, &"sampleRate".into(), &self.sample_rate.into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set sampleRate: {:?}", e)))?;
        Reflect::set(&obj, &"sampleCount".into(), &(self.sample_count as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set sampleCount: {:?}", e)))?;
        Reflect::set(&obj, &"bufferLength".into(), &(self.buffer_length as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set bufferLength: {:?}", e)))?;
        Reflect::set(&obj, &"timestamp".into(), &self.timestamp.into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set timestamp: {:?}", e)))?;
        
        if let Some(seq_num) = self.sequence_number {
            Reflect::set(&obj, &"sequenceNumber".into(), &(seq_num as f64).into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set sequenceNumber: {:?}", e)))?;
        }
        
        if let Some(buffer_id) = self.buffer_id {
            Reflect::set(&obj, &"bufferId".into(), &(buffer_id as f64).into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set bufferId: {:?}", e)))?;
        }
        
        if let Some(buffer_pool_stats) = &self.buffer_pool_stats {
            let stats_obj = buffer_pool_stats.to_js_object()?;
            Reflect::set(&obj, &"bufferPoolStats".into(), &stats_obj.into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set bufferPoolStats: {:?}", e)))?;
        }
        
        Ok(obj)
    }
}

impl FromJsMessage for AudioDataBatch {
    fn from_js_object(obj: &Object) -> SerializationResult<Self> {
        let sample_rate = Reflect::get(obj, &"sampleRate".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get sampleRate: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("sampleRate must be number".to_string()))?;
        
        let sample_count = Reflect::get(obj, &"sampleCount".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get sampleCount: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("sampleCount must be number".to_string()))?
            as usize;
        
        let buffer_length = Reflect::get(obj, &"bufferLength".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get bufferLength: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("bufferLength must be number".to_string()))?
            as usize;
        
        let timestamp = Reflect::get(obj, &"timestamp".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get timestamp: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("timestamp must be number".to_string()))?;
        
        let sequence_number = match Reflect::get(obj, &"sequenceNumber".into()) {
            Ok(value) if !value.is_undefined() => {
                Some(value.as_f64()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("sequenceNumber must be number".to_string()))?
                    as u32)
            }
            _ => None,
        };
        
        let buffer_id = match Reflect::get(obj, &"bufferId".into()) {
            Ok(value) if !value.is_undefined() => {
                Some(value.as_f64()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("bufferId must be number".to_string()))?
                    as u32)
            }
            _ => None,
        };
        
        let buffer_pool_stats = match Reflect::get(obj, &"bufferPoolStats".into()) {
            Ok(value) if !value.is_undefined() => {
                let stats_obj = value.dyn_into::<Object>()
                    .map_err(|_| SerializationError::InvalidPropertyType("bufferPoolStats must be object".to_string()))?;
                Some(BufferPoolStats::from_js_object(&stats_obj)?)
            }
            _ => None,
        };
        
        Ok(AudioDataBatch {
            sample_rate,
            sample_count,
            buffer_length,
            timestamp,
            sequence_number,
            buffer_id,
            buffer_pool_stats,
        })
    }
}

impl MessageValidator for AudioDataBatch {
    fn validate(&self) -> SerializationResult<()> {
        if self.sample_rate <= 0.0 {
            return Err(SerializationError::ValidationFailed("sample_rate must be positive".to_string()));
        }
        if self.sample_count == 0 {
            return Err(SerializationError::ValidationFailed("sample_count cannot be zero".to_string()));
        }
        if self.buffer_length == 0 {
            return Err(SerializationError::ValidationFailed("buffer_length cannot be zero".to_string()));
        }
        if self.timestamp < 0.0 {
            return Err(SerializationError::ValidationFailed("timestamp cannot be negative".to_string()));
        }
        Ok(())
    }
}

impl ToJsMessage for ProcessorStatus {
    fn to_js_object(&self) -> SerializationResult<Object> {
        let obj = Object::new();
        
        Reflect::set(&obj, &"active".into(), &self.active.into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set active: {:?}", e)))?;
        Reflect::set(&obj, &"sampleRate".into(), &self.sample_rate.into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set sampleRate: {:?}", e)))?;
        Reflect::set(&obj, &"bufferSize".into(), &(self.buffer_size as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set bufferSize: {:?}", e)))?;
        Reflect::set(&obj, &"processedBatches".into(), &(self.processed_batches as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set processedBatches: {:?}", e)))?;
        Reflect::set(&obj, &"avgProcessingTimeMs".into(), &self.avg_processing_time_ms.into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set avgProcessingTimeMs: {:?}", e)))?;
        
        if let Some(memory_usage) = &self.memory_usage {
            let memory_obj = memory_usage.to_js_object()?;
            Reflect::set(&obj, &"memoryUsage".into(), &memory_obj.into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set memoryUsage: {:?}", e)))?;
        }
        
        if let Some(buffer_pool_stats) = &self.buffer_pool_stats {
            let buffer_pool_obj = buffer_pool_stats.to_js_object()?;
            Reflect::set(&obj, &"buffer_pool_stats".into(), &buffer_pool_obj.into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set buffer_pool_stats: {:?}", e)))?;
        }
        
        Ok(obj)
    }
}

impl FromJsMessage for ProcessorStatus {
    fn from_js_object(obj: &Object) -> SerializationResult<Self> {
        let active = Reflect::get(obj, &"active".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get active: {:?}", e)))?
            .as_bool()
            .ok_or_else(|| SerializationError::InvalidPropertyType("active must be boolean".to_string()))?;
        
        let sample_rate = Reflect::get(obj, &"sampleRate".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get sampleRate: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("sampleRate must be number".to_string()))?;
        
        let buffer_size = Reflect::get(obj, &"bufferSize".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get bufferSize: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("bufferSize must be number".to_string()))?
            as usize;
        
        let processed_batches = Reflect::get(obj, &"processedBatches".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get processedBatches: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("processedBatches must be number".to_string()))?
            as u32;
        
        let avg_processing_time_ms = Reflect::get(obj, &"avgProcessingTimeMs".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get avgProcessingTimeMs: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("avgProcessingTimeMs must be number".to_string()))?;
        
        let memory_usage = match Reflect::get(obj, &"memoryUsage".into()) {
            Ok(value) if !value.is_undefined() => {
                let memory_obj = value.dyn_into::<Object>()
                    .map_err(|_| SerializationError::InvalidPropertyType("memoryUsage must be object".to_string()))?;
                Some(MemoryUsage::from_js_object(&memory_obj)?)
            }
            _ => None,
        };
        
        let buffer_pool_stats = match Reflect::get(obj, &"buffer_pool_stats".into()) {
            Ok(value) if !value.is_undefined() => {
                let buffer_pool_obj = value.dyn_into::<Object>()
                    .map_err(|_| SerializationError::InvalidPropertyType("buffer_pool_stats must be object".to_string()))?;
                Some(BufferPoolStats::from_js_object(&buffer_pool_obj)?)
            }
            _ => None,
        };
        
        Ok(ProcessorStatus {
            active,
            sample_rate,
            buffer_size,
            processed_batches,
            avg_processing_time_ms,
            memory_usage,
            buffer_pool_stats,
        })
    }
}

impl MessageValidator for ProcessorStatus {
    fn validate(&self) -> SerializationResult<()> {
        if self.sample_rate <= 0.0 {
            return Err(SerializationError::ValidationFailed("sample_rate must be positive".to_string()));
        }
        if self.buffer_size == 0 {
            return Err(SerializationError::ValidationFailed("buffer_size cannot be zero".to_string()));
        }
        if self.avg_processing_time_ms < 0.0 {
            return Err(SerializationError::ValidationFailed("avg_processing_time_ms cannot be negative".to_string()));
        }
        if let Some(memory_usage) = &self.memory_usage {
            memory_usage.validate()?;
        }
        Ok(())
    }
}

impl ToJsMessage for MemoryUsage {
    fn to_js_object(&self) -> SerializationResult<Object> {
        let obj = Object::new();
        
        Reflect::set(&obj, &"heapSize".into(), &(self.heap_size as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set heapSize: {:?}", e)))?;
        Reflect::set(&obj, &"usedHeap".into(), &(self.used_heap as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set usedHeap: {:?}", e)))?;
        Reflect::set(&obj, &"activeBuffers".into(), &(self.active_buffers as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set activeBuffers: {:?}", e)))?;
        
        Ok(obj)
    }
}

impl FromJsMessage for MemoryUsage {
    fn from_js_object(obj: &Object) -> SerializationResult<Self> {
        let heap_size = Reflect::get(obj, &"heapSize".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get heapSize: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("heapSize must be number".to_string()))?
            as usize;
        
        let used_heap = Reflect::get(obj, &"usedHeap".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get usedHeap: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("usedHeap must be number".to_string()))?
            as usize;
        
        let active_buffers = Reflect::get(obj, &"activeBuffers".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get activeBuffers: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("activeBuffers must be number".to_string()))?
            as usize;
        
        Ok(MemoryUsage {
            heap_size,
            used_heap,
            active_buffers,
        })
    }
}

impl MessageValidator for MemoryUsage {
    fn validate(&self) -> SerializationResult<()> {
        if self.used_heap > self.heap_size {
            return Err(SerializationError::ValidationFailed("used_heap cannot exceed heap_size".to_string()));
        }
        Ok(())
    }
}

impl ToJsMessage for BufferPoolStats {
    fn to_js_object(&self) -> SerializationResult<Object> {
        let obj = Object::new();
        
        Reflect::set(&obj, &"pool_size".into(), &(self.pool_size as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set pool_size: {:?}", e)))?;
        Reflect::set(&obj, &"available_buffers".into(), &(self.available_buffers as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set available_buffers: {:?}", e)))?;
        Reflect::set(&obj, &"in_use_buffers".into(), &(self.in_use_buffers as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set in_use_buffers: {:?}", e)))?;
        Reflect::set(&obj, &"total_buffers".into(), &(self.total_buffers as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set total_buffers: {:?}", e)))?;
        Reflect::set(&obj, &"acquire_count".into(), &(self.acquire_count as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set acquire_count: {:?}", e)))?;
        Reflect::set(&obj, &"transfer_count".into(), &(self.transfer_count as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set transfer_count: {:?}", e)))?;
        Reflect::set(&obj, &"pool_exhausted_count".into(), &(self.pool_exhausted_count as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set pool_exhausted_count: {:?}", e)))?;
        Reflect::set(&obj, &"consecutive_pool_failures".into(), &(self.consecutive_pool_failures as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set consecutive_pool_failures: {:?}", e)))?;
        Reflect::set(&obj, &"pool_hit_rate".into(), &(self.pool_hit_rate as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set pool_hit_rate: {:?}", e)))?;
        Reflect::set(&obj, &"pool_efficiency".into(), &(self.pool_efficiency as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set pool_efficiency: {:?}", e)))?;
        Reflect::set(&obj, &"buffer_utilization_percent".into(), &(self.buffer_utilization_percent as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set buffer_utilization_percent: {:?}", e)))?;
        Reflect::set(&obj, &"total_megabytes_transferred".into(), &(self.total_megabytes_transferred as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set total_megabytes_transferred: {:?}", e)))?;
        Reflect::set(&obj, &"avg_acquisition_time_ms".into(), &(self.avg_acquisition_time_ms as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set avg_acquisition_time_ms: {:?}", e)))?;
        Reflect::set(&obj, &"fastest_acquisition_time_ms".into(), &(self.fastest_acquisition_time_ms as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set fastest_acquisition_time_ms: {:?}", e)))?;
        Reflect::set(&obj, &"slowest_acquisition_time_ms".into(), &(self.slowest_acquisition_time_ms as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set slowest_acquisition_time_ms: {:?}", e)))?;
        
        Ok(obj)
    }
}

impl FromJsMessage for BufferPoolStats {
    fn from_js_object(obj: &Object) -> SerializationResult<Self> {
        let pool_size = Reflect::get(obj, &"pool_size".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get pool_size: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("pool_size must be number".to_string()))?
            as u32;
        
        let available_buffers = Reflect::get(obj, &"available_buffers".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get available_buffers: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("available_buffers must be number".to_string()))?
            as u32;
        
        let in_use_buffers = Reflect::get(obj, &"in_use_buffers".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get in_use_buffers: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("in_use_buffers must be number".to_string()))?
            as u32;
        
        let total_buffers = Reflect::get(obj, &"total_buffers".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get total_buffers: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("total_buffers must be number".to_string()))?
            as u32;
        
        let acquire_count = Reflect::get(obj, &"acquire_count".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get acquire_count: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("acquire_count must be number".to_string()))?
            as u32;
        
        let transfer_count = Reflect::get(obj, &"transfer_count".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get transfer_count: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("transfer_count must be number".to_string()))?
            as u32;
        
        let pool_exhausted_count = Reflect::get(obj, &"pool_exhausted_count".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get pool_exhausted_count: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("pool_exhausted_count must be number".to_string()))?
            as u32;
        
        let consecutive_pool_failures = Reflect::get(obj, &"consecutive_pool_failures".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get consecutive_pool_failures: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("consecutive_pool_failures must be number".to_string()))?
            as u32;
        
        let pool_hit_rate = Reflect::get(obj, &"pool_hit_rate".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get pool_hit_rate: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("pool_hit_rate must be number".to_string()))?
            as f32;
        
        let pool_efficiency = Reflect::get(obj, &"pool_efficiency".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get pool_efficiency: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("pool_efficiency must be number".to_string()))?
            as f32;
        
        let buffer_utilization_percent = Reflect::get(obj, &"buffer_utilization_percent".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get buffer_utilization_percent: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("buffer_utilization_percent must be number".to_string()))?
            as f32;
        
        let total_megabytes_transferred = Reflect::get(obj, &"total_megabytes_transferred".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get total_megabytes_transferred: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("total_megabytes_transferred must be number".to_string()))?
            as f32;
        
        let avg_acquisition_time_ms = Reflect::get(obj, &"avg_acquisition_time_ms".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get avg_acquisition_time_ms: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("avg_acquisition_time_ms must be number".to_string()))?
            as f32;
        
        let fastest_acquisition_time_ms = Reflect::get(obj, &"fastest_acquisition_time_ms".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get fastest_acquisition_time_ms: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("fastest_acquisition_time_ms must be number".to_string()))?
            as f32;
        
        let slowest_acquisition_time_ms = Reflect::get(obj, &"slowest_acquisition_time_ms".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get slowest_acquisition_time_ms: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("slowest_acquisition_time_ms must be number".to_string()))?
            as f32;
        
        
        Ok(BufferPoolStats {
            pool_size,
            available_buffers,
            in_use_buffers,
            total_buffers,
            acquire_count,
            transfer_count,
            pool_exhausted_count,
            consecutive_pool_failures,
            pool_hit_rate,
            pool_efficiency,
            buffer_utilization_percent,
            total_megabytes_transferred,
            avg_acquisition_time_ms,
            fastest_acquisition_time_ms,
            slowest_acquisition_time_ms,
        })
    }
}

impl MessageValidator for BufferPoolStats {
    fn validate(&self) -> SerializationResult<()> {
        if self.pool_hit_rate < 0.0 || self.pool_hit_rate > 100.0 {
            return Err(SerializationError::ValidationFailed("pool_hit_rate must be between 0 and 100".to_string()));
        }
        if self.pool_efficiency < 0.0 || self.pool_efficiency > 100.0 {
            return Err(SerializationError::ValidationFailed("pool_efficiency must be between 0 and 100".to_string()));
        }
        if self.buffer_utilization_percent < 0.0 || self.buffer_utilization_percent > 100.0 {
            return Err(SerializationError::ValidationFailed("buffer_utilization_percent must be between 0 and 100".to_string()));
        }
        if self.in_use_buffers + self.available_buffers != self.total_buffers {
            return Err(SerializationError::ValidationFailed("in_use_buffers + available_buffers must equal total_buffers".to_string()));
        }
        Ok(())
    }
}

impl ToJsMessage for BatchConfig {
    fn to_js_object(&self) -> SerializationResult<Object> {
        let obj = Object::new();
        
        Reflect::set(&obj, &"batchSize".into(), &(self.batch_size as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set batchSize: {:?}", e)))?;
        Reflect::set(&obj, &"maxQueueSize".into(), &(self.max_queue_size as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set maxQueueSize: {:?}", e)))?;
        Reflect::set(&obj, &"timeoutMs".into(), &(self.timeout_ms as f64).into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set timeoutMs: {:?}", e)))?;
        Reflect::set(&obj, &"enableCompression".into(), &self.enable_compression.into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set enableCompression: {:?}", e)))?;
        
        Ok(obj)
    }
}

impl FromJsMessage for BatchConfig {
    fn from_js_object(obj: &Object) -> SerializationResult<Self> {
        let batch_size = Reflect::get(obj, &"batchSize".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get batchSize: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("batchSize must be number".to_string()))?
            as usize;
        
        let max_queue_size = Reflect::get(obj, &"maxQueueSize".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get maxQueueSize: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("maxQueueSize must be number".to_string()))?
            as usize;
        
        let timeout_ms = Reflect::get(obj, &"timeoutMs".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get timeoutMs: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("timeoutMs must be number".to_string()))?
            as u32;
        
        let enable_compression = Reflect::get(obj, &"enableCompression".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get enableCompression: {:?}", e)))?
            .as_bool()
            .ok_or_else(|| SerializationError::InvalidPropertyType("enableCompression must be boolean".to_string()))?;
        
        Ok(BatchConfig {
            batch_size,
            max_queue_size,
            timeout_ms,
            enable_compression,
        })
    }
}

impl MessageValidator for BatchConfig {
    fn validate(&self) -> SerializationResult<()> {
        if self.batch_size == 0 {
            return Err(SerializationError::ValidationFailed("batch_size cannot be zero".to_string()));
        }
        if self.max_queue_size == 0 {
            return Err(SerializationError::ValidationFailed("max_queue_size cannot be zero".to_string()));
        }
        if self.timeout_ms == 0 {
            return Err(SerializationError::ValidationFailed("timeout_ms cannot be zero".to_string()));
        }
        Ok(())
    }
}

// Configuration type implementations
impl ToJsMessage for TestSignalGeneratorConfig {
    fn to_js_object(&self) -> SerializationResult<Object> {
        let obj = Object::new();
        
        Reflect::set(&obj, &"enabled".into(), &self.enabled.into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set enabled: {:?}", e)))?;
        Reflect::set(&obj, &"frequency".into(), &self.frequency.into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set frequency: {:?}", e)))?;
        Reflect::set(&obj, &"amplitude".into(), &self.amplitude.into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set amplitude: {:?}", e)))?;
        Reflect::set(&obj, &"sampleRate".into(), &self.sample_rate.into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set sampleRate: {:?}", e)))?;
        
        let waveform_str = match self.waveform {
            crate::engine::audio::test_signal_generator::TestWaveform::Sine => "sine",
            crate::engine::audio::test_signal_generator::TestWaveform::Square => "square",
            crate::engine::audio::test_signal_generator::TestWaveform::Sawtooth => "sawtooth",
            crate::engine::audio::test_signal_generator::TestWaveform::Triangle => "triangle",
            crate::engine::audio::test_signal_generator::TestWaveform::WhiteNoise => "whiteNoise",
            crate::engine::audio::test_signal_generator::TestWaveform::PinkNoise => "pinkNoise",
        };
        Reflect::set(&obj, &"waveform".into(), &waveform_str.into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set waveform: {:?}", e)))?;
        
        Ok(obj)
    }
}

impl FromJsMessage for TestSignalGeneratorConfig {
    fn from_js_object(obj: &Object) -> SerializationResult<Self> {
        let enabled = Reflect::get(obj, &"enabled".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get enabled: {:?}", e)))?
            .as_bool()
            .ok_or_else(|| SerializationError::InvalidPropertyType("enabled must be boolean".to_string()))?;
        
        let frequency = Reflect::get(obj, &"frequency".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get frequency: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("frequency must be number".to_string()))?
            as f32;
        
        let amplitude = Reflect::get(obj, &"amplitude".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get amplitude: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("amplitude must be number".to_string()))?
            as f32;
        
        let sample_rate = Reflect::get(obj, &"sampleRate".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get sampleRate: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("sampleRate must be number".to_string()))?
            as f32;
        
        let waveform_str = Reflect::get(obj, &"waveform".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get waveform: {:?}", e)))?
            .as_string()
            .ok_or_else(|| SerializationError::InvalidPropertyType("waveform must be string".to_string()))?;
        
        let waveform = match waveform_str.as_str() {
            "sine" => crate::engine::audio::test_signal_generator::TestWaveform::Sine,
            "square" => crate::engine::audio::test_signal_generator::TestWaveform::Square,
            "sawtooth" => crate::engine::audio::test_signal_generator::TestWaveform::Sawtooth,
            "triangle" => crate::engine::audio::test_signal_generator::TestWaveform::Triangle,
            "whiteNoise" => crate::engine::audio::test_signal_generator::TestWaveform::WhiteNoise,
            "pinkNoise" => crate::engine::audio::test_signal_generator::TestWaveform::PinkNoise,
            _ => return Err(SerializationError::InvalidPropertyType(format!("Unknown waveform: {}", waveform_str))),
        };
        
        Ok(TestSignalGeneratorConfig {
            enabled,
            frequency,
            amplitude,
            waveform,
            sample_rate,
        })
    }
}

impl MessageValidator for TestSignalGeneratorConfig {
    fn validate(&self) -> SerializationResult<()> {
        if self.frequency <= 0.0 {
            return Err(SerializationError::ValidationFailed("frequency must be positive".to_string()));
        }
        if self.amplitude < 0.0 || self.amplitude > 1.0 {
            return Err(SerializationError::ValidationFailed("amplitude must be between 0.0 and 1.0".to_string()));
        }
        if self.sample_rate <= 0.0 {
            return Err(SerializationError::ValidationFailed("sample_rate must be positive".to_string()));
        }
        Ok(())
    }
}

impl ToJsMessage for BackgroundNoiseConfig {
    fn to_js_object(&self) -> SerializationResult<Object> {
        let obj = Object::new();
        
        Reflect::set(&obj, &"enabled".into(), &self.enabled.into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set enabled: {:?}", e)))?;
        Reflect::set(&obj, &"level".into(), &self.level.into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set level: {:?}", e)))?;
        
        let noise_type_str = match self.noise_type {
            crate::engine::audio::test_signal_generator::TestWaveform::WhiteNoise => "whiteNoise",
            crate::engine::audio::test_signal_generator::TestWaveform::PinkNoise => "pinkNoise",
            _ => "whiteNoise", // Default to white noise for non-noise types
        };
        Reflect::set(&obj, &"noiseType".into(), &noise_type_str.into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set noiseType: {:?}", e)))?;
        
        Ok(obj)
    }
}

impl FromJsMessage for BackgroundNoiseConfig {
    fn from_js_object(obj: &Object) -> SerializationResult<Self> {
        let enabled = Reflect::get(obj, &"enabled".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get enabled: {:?}", e)))?
            .as_bool()
            .ok_or_else(|| SerializationError::InvalidPropertyType("enabled must be boolean".to_string()))?;
        
        let level = Reflect::get(obj, &"level".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get level: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("level must be number".to_string()))?
            as f32;
        
        let noise_type_str = Reflect::get(obj, &"noiseType".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get noiseType: {:?}", e)))?
            .as_string()
            .ok_or_else(|| SerializationError::InvalidPropertyType("noiseType must be string".to_string()))?;
        
        let noise_type = match noise_type_str.as_str() {
            "whiteNoise" => crate::engine::audio::test_signal_generator::TestWaveform::WhiteNoise,
            "pinkNoise" => crate::engine::audio::test_signal_generator::TestWaveform::PinkNoise,
            _ => crate::engine::audio::test_signal_generator::TestWaveform::WhiteNoise, // Default to white noise
        };
        
        Ok(BackgroundNoiseConfig {
            enabled,
            level,
            noise_type,
        })
    }
}

impl MessageValidator for BackgroundNoiseConfig {
    fn validate(&self) -> SerializationResult<()> {
        if self.level < 0.0 || self.level > 1.0 {
            return Err(SerializationError::ValidationFailed("level must be between 0.0 and 1.0".to_string()));
        }
        Ok(())
    }
}

impl ToJsMessage for WorkletError {
    fn to_js_object(&self) -> SerializationResult<Object> {
        let obj = Object::new();
        
        let code_str = match self.code {
            WorkletErrorCode::InitializationFailed => "initializationFailed",
            WorkletErrorCode::ProcessingFailed => "processingFailed",
            WorkletErrorCode::BufferOverflow => "bufferOverflow",
            WorkletErrorCode::InvalidConfiguration => "invalidConfiguration",
            WorkletErrorCode::MemoryAllocationFailed => "memoryAllocationFailed",
            WorkletErrorCode::Generic => "generic",
        };
        Reflect::set(&obj, &"code".into(), &code_str.into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set code: {:?}", e)))?;
        
        Reflect::set(&obj, &"message".into(), &self.message.clone().into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set message: {:?}", e)))?;
        
        Reflect::set(&obj, &"timestamp".into(), &self.timestamp.into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set timestamp: {:?}", e)))?;
        
        if let Some(context) = &self.context {
            let context_obj = context.to_js_object()?;
            Reflect::set(&obj, &"context".into(), &context_obj.into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set context: {:?}", e)))?;
        }
        
        Ok(obj)
    }
}

impl FromJsMessage for WorkletError {
    fn from_js_object(obj: &Object) -> SerializationResult<Self> {
        let code_str = Reflect::get(obj, &"code".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get code: {:?}", e)))?
            .as_string()
            .ok_or_else(|| SerializationError::InvalidPropertyType("code must be string".to_string()))?;
        
        let code = match code_str.as_str() {
            "initializationFailed" => WorkletErrorCode::InitializationFailed,
            "processingFailed" => WorkletErrorCode::ProcessingFailed,
            "bufferOverflow" => WorkletErrorCode::BufferOverflow,
            "invalidConfiguration" => WorkletErrorCode::InvalidConfiguration,
            "memoryAllocationFailed" => WorkletErrorCode::MemoryAllocationFailed,
            "generic" => WorkletErrorCode::Generic,
            _ => return Err(SerializationError::InvalidPropertyType(format!("Unknown error code: {}", code_str))),
        };
        
        let message = Reflect::get(obj, &"message".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get message: {:?}", e)))?
            .as_string()
            .ok_or_else(|| SerializationError::InvalidPropertyType("message must be string".to_string()))?;
        
        let timestamp = Reflect::get(obj, &"timestamp".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get timestamp: {:?}", e)))?
            .as_f64()
            .ok_or_else(|| SerializationError::InvalidPropertyType("timestamp must be number".to_string()))?;
        
        let context = match Reflect::get(obj, &"context".into()) {
            Ok(value) if !value.is_undefined() => {
                let context_obj = value.dyn_into::<Object>()
                    .map_err(|_| SerializationError::InvalidPropertyType("context must be object".to_string()))?;
                Some(ErrorContext::from_js_object(&context_obj)?)
            }
            _ => None,
        };
        
        Ok(WorkletError {
            code,
            message,
            timestamp,
            context,
        })
    }
}

impl MessageValidator for WorkletError {
    fn validate(&self) -> SerializationResult<()> {
        if self.message.is_empty() {
            return Err(SerializationError::ValidationFailed("message cannot be empty".to_string()));
        }
        if self.timestamp < 0.0 {
            return Err(SerializationError::ValidationFailed("timestamp cannot be negative".to_string()));
        }
        if let Some(context) = &self.context {
            context.validate()?;
        }
        Ok(())
    }
}

impl ToJsMessage for ErrorContext {
    fn to_js_object(&self) -> SerializationResult<Object> {
        let obj = Object::new();
        
        Reflect::set(&obj, &"location".into(), &self.location.clone().into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set location: {:?}", e)))?;
        
        Reflect::set(&obj, &"timestamp".into(), &self.timestamp.into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set timestamp: {:?}", e)))?;
        
        if let Some(stack_trace) = &self.stack_trace {
            let js_array = js_sys::Array::new();
            for trace in stack_trace {
                js_array.push(&trace.clone().into());
            }
            Reflect::set(&obj, &"stackTrace".into(), &js_array.into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set stackTrace: {:?}", e)))?;
        }
        
        if let Some(message_context) = &self.message_context {
            let context_obj = message_context.to_js_object()?;
            Reflect::set(&obj, &"messageContext".into(), &context_obj.into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set messageContext: {:?}", e)))?;
        }
        
        if let Some(system_state) = &self.system_state {
            let state_obj = system_state.to_js_object()?;
            Reflect::set(&obj, &"systemState".into(), &state_obj.into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set systemState: {:?}", e)))?;
        }
        
        if let Some(debug_info) = &self.debug_info {
            Reflect::set(&obj, &"debugInfo".into(), &debug_info.clone().into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set debugInfo: {:?}", e)))?;
        }
        
        if let Some(thread_id) = &self.thread_id {
            Reflect::set(&obj, &"threadId".into(), &thread_id.clone().into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set threadId: {:?}", e)))?;
        }
        
        Ok(obj)
    }
}

impl FromJsMessage for ErrorContext {
    fn from_js_object(obj: &Object) -> SerializationResult<Self> {
        let location = Reflect::get(obj, &"location".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get location: {:?}", e)))?
            .as_string()
            .ok_or_else(|| SerializationError::InvalidPropertyType("location must be string".to_string()))?;
        
        let timestamp = Reflect::get(obj, &"timestamp".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get timestamp: {:?}", e)))?
            .as_f64()
            .unwrap_or_else(|| js_sys::Date::now());
        
        let stack_trace = match Reflect::get(obj, &"stackTrace".into()) {
            Ok(value) if !value.is_undefined() => {
                let array = value.dyn_ref::<js_sys::Array>()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("stackTrace must be array".to_string()))?;
                let mut trace_vec = Vec::new();
                for i in 0..array.length() {
                    if let Some(trace_str) = array.get(i).as_string() {
                        trace_vec.push(trace_str);
                    }
                }
                Some(trace_vec)
            }
            _ => None,
        };
        
        let message_context = match Reflect::get(obj, &"messageContext".into()) {
            Ok(value) if !value.is_undefined() => {
                let context_obj = value.dyn_ref::<Object>()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("messageContext must be object".to_string()))?;
                Some(MessageContext::from_js_object(context_obj)?)
            }
            _ => None,
        };
        
        let system_state = match Reflect::get(obj, &"systemState".into()) {
            Ok(value) if !value.is_undefined() => {
                let state_obj = value.dyn_ref::<Object>()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("systemState must be object".to_string()))?;
                Some(SystemState::from_js_object(state_obj)?)
            }
            _ => None,
        };
        
        let debug_info = match Reflect::get(obj, &"debugInfo".into()) {
            Ok(value) if !value.is_undefined() => {
                Some(value.as_string()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("debugInfo must be string".to_string()))?)
            }
            _ => None,
        };
        
        let thread_id = match Reflect::get(obj, &"threadId".into()) {
            Ok(value) if !value.is_undefined() => {
                Some(value.as_string()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("threadId must be string".to_string()))?)
            }
            _ => None,
        };
        
        Ok(ErrorContext {
            location,
            stack_trace,
            message_context,
            system_state,
            debug_info,
            timestamp,
            thread_id,
        })
    }
}

impl MessageValidator for ErrorContext {
    fn validate(&self) -> SerializationResult<()> {
        if self.location.is_empty() {
            return Err(SerializationError::ValidationFailed("location cannot be empty".to_string()));
        }
        Ok(())
    }
}

impl ToJsMessage for MessageContext {
    fn to_js_object(&self) -> SerializationResult<Object> {
        let obj = Object::new();
        
        Reflect::set(&obj, &"messageType".into(), &self.message_type.clone().into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set messageType: {:?}", e)))?;
        
        let direction_str = match self.direction {
            MessageDirection::ToWorklet => "toWorklet",
            MessageDirection::FromWorklet => "fromWorklet", 
            MessageDirection::Internal => "internal",
        };
        Reflect::set(&obj, &"direction".into(), &direction_str.into())
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set direction: {:?}", e)))?;
        
        if let Some(message_id) = self.message_id {
            Reflect::set(&obj, &"messageId".into(), &message_id.into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set messageId: {:?}", e)))?;
        }
        
        if let Some(timestamp) = self.message_timestamp {
            Reflect::set(&obj, &"messageTimestamp".into(), &timestamp.into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set messageTimestamp: {:?}", e)))?;
        }
        
        if let Some(size) = self.message_size {
            Reflect::set(&obj, &"messageSize".into(), &(size as f64).into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set messageSize: {:?}", e)))?;
        }
        
        Ok(obj)
    }
}

impl FromJsMessage for MessageContext {
    fn from_js_object(obj: &Object) -> SerializationResult<Self> {
        let message_type = Reflect::get(obj, &"messageType".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get messageType: {:?}", e)))?
            .as_string()
            .ok_or_else(|| SerializationError::InvalidPropertyType("messageType must be string".to_string()))?;
        
        let direction_str = Reflect::get(obj, &"direction".into())
            .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get direction: {:?}", e)))?
            .as_string()
            .ok_or_else(|| SerializationError::InvalidPropertyType("direction must be string".to_string()))?;
        
        let direction = match direction_str.as_str() {
            "toWorklet" => MessageDirection::ToWorklet,
            "fromWorklet" => MessageDirection::FromWorklet,
            "internal" => MessageDirection::Internal,
            _ => return Err(SerializationError::InvalidPropertyType(format!("Invalid direction: {}", direction_str))),
        };
        
        let message_id = match Reflect::get(obj, &"messageId".into()) {
            Ok(value) if !value.is_undefined() => {
                Some(value.as_f64()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("messageId must be number".to_string()))? as u32)
            }
            _ => None,
        };
        
        let message_timestamp = match Reflect::get(obj, &"messageTimestamp".into()) {
            Ok(value) if !value.is_undefined() => {
                Some(value.as_f64()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("messageTimestamp must be number".to_string()))?)
            }
            _ => None,
        };
        
        let message_size = match Reflect::get(obj, &"messageSize".into()) {
            Ok(value) if !value.is_undefined() => {
                Some(value.as_f64()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("messageSize must be number".to_string()))? as usize)
            }
            _ => None,
        };
        
        Ok(MessageContext {
            message_type,
            direction,
            message_id,
            message_timestamp,
            message_size,
        })
    }
}

impl ToJsMessage for SystemState {
    fn to_js_object(&self) -> SerializationResult<Object> {
        let obj = Object::new();
        
        if let Some(memory_usage) = self.memory_usage {
            Reflect::set(&obj, &"memoryUsage".into(), &(memory_usage as f64).into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set memoryUsage: {:?}", e)))?;
        }
        
        if let Some(queue_depth) = self.queue_depth {
            Reflect::set(&obj, &"queueDepth".into(), &(queue_depth as f64).into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set queueDepth: {:?}", e)))?;
        }
        
        if let Some(active_buffers) = self.active_buffers {
            Reflect::set(&obj, &"activeBuffers".into(), &(active_buffers as f64).into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set activeBuffers: {:?}", e)))?;
        }
        
        if let Some(audio_processing_active) = self.audio_processing_active {
            Reflect::set(&obj, &"audioProcessingActive".into(), &audio_processing_active.into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set audioProcessingActive: {:?}", e)))?;
        }
        
        if let Some(sample_rate) = self.sample_rate {
            Reflect::set(&obj, &"sampleRate".into(), &sample_rate.into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set sampleRate: {:?}", e)))?;
        }
        
        if let Some(buffer_size) = self.buffer_size {
            Reflect::set(&obj, &"bufferSize".into(), &(buffer_size as f64).into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set bufferSize: {:?}", e)))?;
        }
        
        if let Some(processor_load) = self.processor_load {
            Reflect::set(&obj, &"processorLoad".into(), &(processor_load as f64).into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set processorLoad: {:?}", e)))?;
        }
        
        if let Some(available_heap) = self.available_heap {
            Reflect::set(&obj, &"availableHeap".into(), &(available_heap as f64).into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set availableHeap: {:?}", e)))?;
        }
        
        Ok(obj)
    }
}

impl FromJsMessage for SystemState {
    fn from_js_object(obj: &Object) -> SerializationResult<Self> {
        let memory_usage = match Reflect::get(obj, &"memoryUsage".into()) {
            Ok(value) if !value.is_undefined() => {
                Some(value.as_f64()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("memoryUsage must be number".to_string()))? as usize)
            }
            _ => None,
        };
        
        let queue_depth = match Reflect::get(obj, &"queueDepth".into()) {
            Ok(value) if !value.is_undefined() => {
                Some(value.as_f64()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("queueDepth must be number".to_string()))? as usize)
            }
            _ => None,
        };
        
        let active_buffers = match Reflect::get(obj, &"activeBuffers".into()) {
            Ok(value) if !value.is_undefined() => {
                Some(value.as_f64()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("activeBuffers must be number".to_string()))? as usize)
            }
            _ => None,
        };
        
        let audio_processing_active = match Reflect::get(obj, &"audioProcessingActive".into()) {
            Ok(value) if !value.is_undefined() => {
                Some(value.as_bool()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("audioProcessingActive must be boolean".to_string()))?)
            }
            _ => None,
        };
        
        let sample_rate = match Reflect::get(obj, &"sampleRate".into()) {
            Ok(value) if !value.is_undefined() => {
                Some(value.as_f64()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("sampleRate must be number".to_string()))?)
            }
            _ => None,
        };
        
        let buffer_size = match Reflect::get(obj, &"bufferSize".into()) {
            Ok(value) if !value.is_undefined() => {
                Some(value.as_f64()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("bufferSize must be number".to_string()))? as usize)
            }
            _ => None,
        };
        
        let processor_load = match Reflect::get(obj, &"processorLoad".into()) {
            Ok(value) if !value.is_undefined() => {
                Some(value.as_f64()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("processorLoad must be number".to_string()))? as f32)
            }
            _ => None,
        };
        
        let available_heap = match Reflect::get(obj, &"availableHeap".into()) {
            Ok(value) if !value.is_undefined() => {
                Some(value.as_f64()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("availableHeap must be number".to_string()))? as usize)
            }
            _ => None,
        };
        
        Ok(SystemState {
            memory_usage,
            queue_depth,
            active_buffers,
            audio_processing_active,
            sample_rate,
            buffer_size,
            processor_load,
            available_heap,
        })
    }
}


// ================================
// Message Construction Utilities
// ================================

/// Result type for message construction operations
pub type MessageConstructionResult<T> = Result<T, MessageConstructionError>;

/// Error types for message construction
#[derive(Debug, Clone, PartialEq)]
pub enum MessageConstructionError {
    /// Invalid parameter value
    InvalidParameter(String),
    /// Missing required parameter
    MissingParameter(String),
    /// Validation failed during construction
    ValidationFailed(String),
    /// Message ID generation failed
    IdGenerationFailed(String),
}

impl std::fmt::Display for MessageConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageConstructionError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            MessageConstructionError::MissingParameter(msg) => write!(f, "Missing parameter: {}", msg),
            MessageConstructionError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            MessageConstructionError::IdGenerationFailed(msg) => write!(f, "ID generation failed: {}", msg),
        }
    }
}

impl std::error::Error for MessageConstructionError {}

/// Enhanced message ID generator with atomicity support
#[derive(Clone)]
pub struct MessageIdGenerator {
    counter: std::rc::Rc<std::cell::RefCell<u32>>,
}

impl MessageIdGenerator {
    /// Create a new message ID generator
    pub fn new() -> Self {
        Self {
            counter: std::rc::Rc::new(std::cell::RefCell::new(0)),
        }
    }
    
    /// Generate a unique message ID
    pub fn next_id(&self) -> u32 {
        let mut counter = self.counter.borrow_mut();
        *counter = counter.wrapping_add(1);
        *counter
    }
    
    /// Reset the counter (for testing)
    pub fn reset(&self) {
        *self.counter.borrow_mut() = 0;
    }
}

impl Default for MessageIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// Global message ID generator instance
// TODO: FUTURE REFACTORING - Remove this global variable and replace with dependency injection through context.
// This is a planned future task. Do NOT refactor this during unrelated work.
// See docs/global_variables_refactoring_guide.md for refactoring strategy.
thread_local! {
    static MESSAGE_ID_GENERATOR: MessageIdGenerator = MessageIdGenerator::new();
}

/// Generate a unique message ID using the global generator
pub fn generate_unique_message_id() -> u32 {
    MESSAGE_ID_GENERATOR.with(|generator| generator.next_id())
}

/// Get current high-resolution timestamp
pub fn get_high_resolution_timestamp() -> f64 {
    // Use performance.now() for high-resolution timing
    js_sys::Date::now()
}

/// Message construction utilities
pub struct MessageBuilder;

impl MessageBuilder {
    /// Create a new message envelope with auto-generated ID and timestamp
    pub fn envelope<T: MessageValidator>(payload: T) -> MessageConstructionResult<MessageEnvelope<T>> {
        payload.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        
        Ok(MessageEnvelope {
            message_id: generate_unique_message_id(),
            timestamp: get_high_resolution_timestamp(),
            payload,
        })
    }
    
    /// Create a new message envelope with specific ID
    pub fn envelope_with_id<T: MessageValidator>(payload: T, message_id: u32) -> MessageConstructionResult<MessageEnvelope<T>> {
        payload.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        
        Ok(MessageEnvelope {
            message_id,
            timestamp: get_high_resolution_timestamp(),
            payload,
        })
    }
}

// Constructor implementations for message types
impl ToWorkletMessage {
    /// Create a start processing message
    pub fn start_processing() -> Self {
        Self::StartProcessing
    }
    
    /// Create a stop processing message
    pub fn stop_processing() -> Self {
        Self::StopProcessing
    }
    
    /// Create an update test signal config message
    pub fn update_test_signal_config(config: TestSignalGeneratorConfig) -> MessageConstructionResult<Self> {
        config.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        Ok(Self::UpdateTestSignalConfig { config })
    }
    
    /// Create an update batch config message
    pub fn update_batch_config(config: BatchConfig) -> MessageConstructionResult<Self> {
        config.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        Ok(Self::UpdateBatchConfig { config })
    }
    
    /// Create an update background noise config message
    pub fn update_background_noise_config(config: BackgroundNoiseConfig) -> MessageConstructionResult<Self> {
        config.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        Ok(Self::UpdateBackgroundNoiseConfig { config })
    }
    
    /// Create a return buffer message
    pub fn return_buffer(buffer_id: u32) -> Self {
        Self::ReturnBuffer { buffer_id }
    }
    
}

impl FromWorkletMessage {
    /// Create a processor ready message
    pub fn processor_ready(batch_size: Option<usize>) -> MessageConstructionResult<Self> {
        if let Some(size) = batch_size {
            if size == 0 {
                return Err(MessageConstructionError::InvalidParameter("batch_size cannot be zero".to_string()));
            }
        }
        Ok(Self::ProcessorReady { batch_size })
    }
    
    /// Create a processing started message
    pub fn processing_started() -> Self {
        Self::ProcessingStarted
    }
    
    /// Create a processing stopped message
    pub fn processing_stopped() -> Self {
        Self::ProcessingStopped
    }
    
    /// Create an audio data batch message
    pub fn audio_data_batch(data: AudioDataBatch) -> MessageConstructionResult<Self> {
        data.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        Ok(Self::AudioDataBatch { data })
    }
    
    /// Create a processing error message
    pub fn processing_error(error: WorkletError) -> MessageConstructionResult<Self> {
        error.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        Ok(Self::ProcessingError { error })
    }
    
}

impl AudioDataBatch {
    /// Create a new audio data batch
    pub fn new(
        sample_rate: f64,
        sample_count: usize,
        buffer_length: usize,
        sequence_number: Option<u32>,
    ) -> MessageConstructionResult<Self> {
        let batch = Self {
            sample_rate,
            sample_count,
            buffer_length,
            timestamp: get_high_resolution_timestamp(),
            sequence_number,
            buffer_id: None,
            buffer_pool_stats: None,
        };
        
        batch.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        Ok(batch)
    }
    
    /// Create a new audio data batch with current timestamp
    pub fn with_timestamp(
        sample_rate: f64,
        sample_count: usize,
        buffer_length: usize,
        timestamp: f64,
        sequence_number: Option<u32>,
    ) -> MessageConstructionResult<Self> {
        let batch = Self {
            sample_rate,
            sample_count,
            buffer_length,
            timestamp,
            sequence_number,
            buffer_id: None,
            buffer_pool_stats: None,
        };
        
        batch.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        Ok(batch)
    }
}

impl ProcessorStatus {
    /// Create a new processor status
    pub fn new(
        active: bool,
        sample_rate: f64,
        buffer_size: usize,
        processed_batches: u32,
        avg_processing_time_ms: f64,
        memory_usage: Option<MemoryUsage>,
    ) -> MessageConstructionResult<Self> {
        let status = Self {
            active,
            sample_rate,
            buffer_size,
            processed_batches,
            avg_processing_time_ms,
            memory_usage,
            buffer_pool_stats: None,
        };
        
        status.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        Ok(status)
    }
    
    /// Create a new processor status with buffer pool statistics
    pub fn with_buffer_pool_stats(
        active: bool,
        sample_rate: f64,
        buffer_size: usize,
        processed_batches: u32,
        avg_processing_time_ms: f64,
        memory_usage: Option<MemoryUsage>,
        buffer_pool_stats: Option<BufferPoolStats>,
    ) -> MessageConstructionResult<Self> {
        let status = Self {
            active,
            sample_rate,
            buffer_size,
            processed_batches,
            avg_processing_time_ms,
            memory_usage,
            buffer_pool_stats,
        };
        
        status.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        Ok(status)
    }
}

impl MemoryUsage {
    /// Create a new memory usage info
    pub fn new(heap_size: usize, used_heap: usize, active_buffers: usize) -> MessageConstructionResult<Self> {
        let usage = Self {
            heap_size,
            used_heap,
            active_buffers,
        };
        
        usage.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        Ok(usage)
    }
}

impl BatchConfig {
    /// Create a new batch config
    pub fn new(
        batch_size: usize,
        max_queue_size: usize,
        timeout_ms: u32,
        enable_compression: bool,
    ) -> MessageConstructionResult<Self> {
        let config = Self {
            batch_size,
            max_queue_size,
            timeout_ms,
            enable_compression,
        };
        
        config.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        Ok(config)
    }
}

impl WorkletError {
    /// Create a new worklet error
    pub fn new(
        code: WorkletErrorCode,
        message: String,
        context: Option<ErrorContext>,
    ) -> MessageConstructionResult<Self> {
        let error = Self {
            code,
            message,
            timestamp: get_high_resolution_timestamp(),
            context,
        };
        
        error.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        Ok(error)
    }
    
    /// Create a new worklet error with custom timestamp
    pub fn with_timestamp(
        code: WorkletErrorCode,
        message: String,
        timestamp: f64,
        context: Option<ErrorContext>,
    ) -> MessageConstructionResult<Self> {
        let error = Self {
            code,
            message,
            timestamp,
            context,
        };
        
        error.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        Ok(error)
    }
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(location: String) -> Self {
        Self {
            location,
            stack_trace: None,
            message_context: None,
            system_state: None,
            debug_info: None,
            timestamp: js_sys::Date::now(),
            thread_id: None,
        }
    }

    /// Create a new error context with full information
    pub fn new_full(
        location: String,
        stack_trace: Option<Vec<String>>,
        message_context: Option<MessageContext>,
        system_state: Option<SystemState>,
        debug_info: Option<String>,
        thread_id: Option<String>,
    ) -> MessageConstructionResult<Self> {
        let context = Self {
            location,
            stack_trace,
            message_context,
            system_state,
            debug_info,
            timestamp: js_sys::Date::now(),
            thread_id,
        };
        
        context.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        Ok(context)
    }

    /// Add stack trace information
    pub fn with_stack_trace(mut self, stack_trace: Vec<String>) -> Self {
        self.stack_trace = Some(stack_trace);
        self
    }

    /// Add message context information  
    pub fn with_message_context(mut self, message_context: MessageContext) -> Self {
        self.message_context = Some(message_context);
        self
    }

    /// Add system state information
    pub fn with_system_state(mut self, system_state: SystemState) -> Self {
        self.system_state = Some(system_state);
        self
    }

    /// Add debug information
    pub fn with_debug_info(mut self, debug_info: String) -> Self {
        self.debug_info = Some(debug_info);
        self
    }

    /// Add thread identifier
    pub fn with_thread_id(mut self, thread_id: String) -> Self {
        self.thread_id = Some(thread_id);
        self
    }
}

impl MessageContext {
    /// Create a new message context
    pub fn new(message_type: String, direction: MessageDirection) -> Self {
        Self {
            message_type,
            direction,
            message_id: None,
            message_timestamp: None,
            message_size: None,
        }
    }

    /// Add message ID
    pub fn with_message_id(mut self, message_id: u32) -> Self {
        self.message_id = Some(message_id);
        self
    }

    /// Add message timestamp
    pub fn with_timestamp(mut self, timestamp: f64) -> Self {
        self.message_timestamp = Some(timestamp);
        self
    }

    /// Add message size
    pub fn with_size(mut self, size: usize) -> Self {
        self.message_size = Some(size);
        self
    }
}

impl SystemState {
    /// Create a new empty system state
    pub fn new() -> Self {
        Self {
            memory_usage: None,
            queue_depth: None,
            active_buffers: None,
            audio_processing_active: None,
            sample_rate: None,
            buffer_size: None,
            processor_load: None,
            available_heap: None,
        }
    }

    /// Create system state with basic information
    pub fn basic(
        memory_usage: Option<usize>,
        queue_depth: Option<usize>,
        audio_processing_active: Option<bool>,
    ) -> Self {
        Self {
            memory_usage,
            queue_depth,
            active_buffers: None,
            audio_processing_active,
            sample_rate: None,
            buffer_size: None,
            processor_load: None,
            available_heap: None,
        }
    }

    /// Add memory usage information
    pub fn with_memory_usage(mut self, memory_usage: usize) -> Self {
        self.memory_usage = Some(memory_usage);
        self
    }

    /// Add queue depth information
    pub fn with_queue_depth(mut self, queue_depth: usize) -> Self {
        self.queue_depth = Some(queue_depth);
        self
    }

    /// Add active buffer count
    pub fn with_active_buffers(mut self, active_buffers: usize) -> Self {
        self.active_buffers = Some(active_buffers);
        self
    }

    /// Add audio processing status
    pub fn with_audio_processing_active(mut self, active: bool) -> Self {
        self.audio_processing_active = Some(active);
        self
    }

    /// Add sample rate information
    pub fn with_sample_rate(mut self, sample_rate: f64) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }

    /// Add buffer size information
    pub fn with_buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = Some(buffer_size);
        self
    }

    /// Add processor load information
    pub fn with_processor_load(mut self, processor_load: f32) -> Self {
        self.processor_load = Some(processor_load);
        self
    }

    /// Add available heap information
    pub fn with_available_heap(mut self, available_heap: usize) -> Self {
        self.available_heap = Some(available_heap);
        self
    }
}

impl TestSignalGeneratorConfig {
    /// Create a new test signal generator config
    pub fn new(
        enabled: bool,
        frequency: f32,
        amplitude: f32,
        waveform: crate::engine::audio::test_signal_generator::TestWaveform,
        sample_rate: f32,
    ) -> MessageConstructionResult<Self> {
        let config = Self {
            enabled,
            frequency,
            amplitude,
            waveform,
            sample_rate,
        };
        
        config.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        Ok(config)
    }
}

impl BackgroundNoiseConfig {
    /// Create a new background noise config
    pub fn new(
        enabled: bool,
        level: f32,
        noise_type: crate::engine::audio::test_signal_generator::TestWaveform,
    ) -> MessageConstructionResult<Self> {
        let config = Self {
            enabled,
            level,
            noise_type,
        };
        
        config.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        Ok(config)
    }
}

// ================================
// AudioWorklet Message Factory
// ================================

/// Centralized message factory for AudioWorklet communication
#[derive(Clone)]
pub struct AudioWorkletMessageFactory {
    id_generator: MessageIdGenerator,
}

impl AudioWorkletMessageFactory {
    /// Create a new message factory
    pub fn new() -> Self {
        Self {
            id_generator: MessageIdGenerator::new(),
        }
    }
    
    /// Create a message factory with custom ID generator
    pub fn with_id_generator(id_generator: MessageIdGenerator) -> Self {
        Self {
            id_generator,
        }
    }
    
    /// Generate a unique message ID
    pub fn generate_id(&self) -> u32 {
        self.id_generator.next_id()
    }
    
    /// Reset the ID generator (for testing)
    pub fn reset_id_generator(&self) {
        self.id_generator.reset();
    }
    
    // ToWorkletMessage factory methods
    
    /// Create a start processing message envelope
    pub fn start_processing(&self) -> MessageConstructionResult<ToWorkletEnvelope> {
        let message = ToWorkletMessage::start_processing();
        Ok(MessageEnvelope {
            message_id: self.generate_id(),
            timestamp: get_high_resolution_timestamp(),
            payload: message,
        })
    }
    
    /// Create a stop processing message envelope
    pub fn stop_processing(&self) -> MessageConstructionResult<ToWorkletEnvelope> {
        let message = ToWorkletMessage::stop_processing();
        Ok(MessageEnvelope {
            message_id: self.generate_id(),
            timestamp: get_high_resolution_timestamp(),
            payload: message,
        })
    }
    
    /// Create an update test signal config message envelope
    pub fn update_test_signal_config(&self, config: TestSignalGeneratorConfig) -> MessageConstructionResult<ToWorkletEnvelope> {
        let message = ToWorkletMessage::update_test_signal_config(config)?;
        Ok(MessageEnvelope {
            message_id: self.generate_id(),
            timestamp: get_high_resolution_timestamp(),
            payload: message,
        })
    }
    
    /// Create an update batch config message envelope
    pub fn update_batch_config(&self, config: BatchConfig) -> MessageConstructionResult<ToWorkletEnvelope> {
        let message = ToWorkletMessage::update_batch_config(config)?;
        Ok(MessageEnvelope {
            message_id: self.generate_id(),
            timestamp: get_high_resolution_timestamp(),
            payload: message,
        })
    }
    
    /// Create an update background noise config message envelope
    pub fn update_background_noise_config(&self, config: BackgroundNoiseConfig) -> MessageConstructionResult<ToWorkletEnvelope> {
        let message = ToWorkletMessage::update_background_noise_config(config)?;
        Ok(MessageEnvelope {
            message_id: self.generate_id(),
            timestamp: get_high_resolution_timestamp(),
            payload: message,
        })
    }
    
    /// Create a return buffer message envelope
    pub fn return_buffer(&self, buffer_id: u32) -> MessageConstructionResult<ToWorkletEnvelope> {
        let message = ToWorkletMessage::return_buffer(buffer_id);
        Ok(MessageEnvelope {
            message_id: self.generate_id(),
            timestamp: get_high_resolution_timestamp(),
            payload: message,
        })
    }
    
    
    // FromWorkletMessage factory methods
    
    /// Create a processor ready message envelope
    pub fn processor_ready(&self, batch_size: Option<usize>) -> MessageConstructionResult<FromWorkletEnvelope> {
        let message = FromWorkletMessage::processor_ready(batch_size)?;
        Ok(MessageEnvelope {
            message_id: self.generate_id(),
            timestamp: get_high_resolution_timestamp(),
            payload: message,
        })
    }
    
    /// Create a processing started message envelope
    pub fn processing_started(&self) -> MessageConstructionResult<FromWorkletEnvelope> {
        let message = FromWorkletMessage::processing_started();
        Ok(MessageEnvelope {
            message_id: self.generate_id(),
            timestamp: get_high_resolution_timestamp(),
            payload: message,
        })
    }
    
    /// Create a processing stopped message envelope
    pub fn processing_stopped(&self) -> MessageConstructionResult<FromWorkletEnvelope> {
        let message = FromWorkletMessage::processing_stopped();
        Ok(MessageEnvelope {
            message_id: self.generate_id(),
            timestamp: get_high_resolution_timestamp(),
            payload: message,
        })
    }
    
    /// Create an audio data batch message envelope
    pub fn audio_data_batch(&self, data: AudioDataBatch) -> MessageConstructionResult<FromWorkletEnvelope> {
        let message = FromWorkletMessage::audio_data_batch(data)?;
        Ok(MessageEnvelope {
            message_id: self.generate_id(),
            timestamp: get_high_resolution_timestamp(),
            payload: message,
        })
    }
    
    /// Create a processing error message envelope
    pub fn processing_error(&self, error: WorkletError) -> MessageConstructionResult<FromWorkletEnvelope> {
        let message = FromWorkletMessage::processing_error(error)?;
        Ok(MessageEnvelope {
            message_id: self.generate_id(),
            timestamp: get_high_resolution_timestamp(),
            payload: message,
        })
    }
    
    
    // Convenience methods for common patterns
    
    /// Create an audio data batch with metadata
    pub fn create_audio_data_batch(&self, 
        sample_rate: f64, 
        sample_count: usize, 
        buffer_length: usize,
        sequence_number: Option<u32>
    ) -> MessageConstructionResult<FromWorkletEnvelope> {
        let data = AudioDataBatch::new(sample_rate, sample_count, buffer_length, sequence_number)?;
        self.audio_data_batch(data)
    }
    
    /// Create a worklet error with context
    pub fn create_worklet_error(&self, 
        code: WorkletErrorCode, 
        message: String, 
        location: String,
        system_state: Option<String>
    ) -> MessageConstructionResult<FromWorkletEnvelope> {
        let context = ErrorContext::new(location);
        let error = WorkletError::new(code, message, Some(context))?;
        self.processing_error(error)
    }
    
    
    /// Create memory usage info
    pub fn create_memory_usage(&self, 
        heap_size: usize, 
        used_heap: usize, 
        active_buffers: usize
    ) -> MessageConstructionResult<MemoryUsage> {
        MemoryUsage::new(heap_size, used_heap, active_buffers)
    }
    
    /// Create a test signal config
    pub fn create_test_signal_config(&self,
        enabled: bool,
        frequency: f32,
        amplitude: f32,
        waveform: crate::engine::audio::test_signal_generator::TestWaveform,
        sample_rate: f32
    ) -> MessageConstructionResult<ToWorkletEnvelope> {
        let config = TestSignalGeneratorConfig::new(enabled, frequency, amplitude, waveform, sample_rate)?;
        self.update_test_signal_config(config)
    }
    
    /// Create a background noise config
    pub fn create_background_noise_config(&self,
        enabled: bool,
        level: f32,
        noise_type: crate::engine::audio::test_signal_generator::TestWaveform
    ) -> MessageConstructionResult<ToWorkletEnvelope> {
        let config = BackgroundNoiseConfig::new(enabled, level, noise_type)?;
        self.update_background_noise_config(config)
    }
    
    /// Create a batch config
    pub fn create_batch_config(&self,
        batch_size: usize,
        max_queue_size: usize,
        timeout_ms: u32,
        enable_compression: bool
    ) -> MessageConstructionResult<ToWorkletEnvelope> {
        let config = BatchConfig::new(batch_size, max_queue_size, timeout_ms, enable_compression)?;
        self.update_batch_config(config)
    }
    
    // Request/response correlation support
    
    /// Create a response message with correlation to a request
    pub fn create_response<T: MessageValidator>(&self, request_id: u32, payload: T) -> MessageConstructionResult<MessageEnvelope<T>> {
        payload.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        
        Ok(MessageEnvelope {
            message_id: request_id, // Use same ID for correlation
            timestamp: get_high_resolution_timestamp(),
            payload,
        })
    }
    
    /// Create a correlated processor ready response
    pub fn processor_ready_response(&self, request_id: u32, batch_size: Option<usize>) -> MessageConstructionResult<FromWorkletEnvelope> {
        let message = FromWorkletMessage::processor_ready(batch_size)?;
        self.create_response(request_id, message)
    }
    
    /// Create a correlated error response
    pub fn error_response(&self, request_id: u32, error: WorkletError) -> MessageConstructionResult<FromWorkletEnvelope> {
        let message = FromWorkletMessage::processing_error(error)?;
        self.create_response(request_id, message)
    }
}

impl Default for AudioWorkletMessageFactory {
    fn default() -> Self {
        Self::new()
    }
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
            context: Some(ErrorContext::new("audio_processor.rs:123".to_string())
                .with_system_state(SystemState::basic(
                    Some(1024),
                    Some(10), 
                    Some(true)
                ))),
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
            buffer_id: None,
            buffer_pool_stats: None,
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
            buffer_pool_stats: None,
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

    #[wasm_bindgen_test]
    fn test_serialization_round_trip_to_worklet() {
        // Test simple message
        let msg = ToWorkletMessage::StartProcessing;
        let obj = msg.to_js_object().unwrap();
        let deserialized = ToWorkletMessage::from_js_object(&obj).unwrap();
        assert_eq!(msg, deserialized);
        
        // Test message with config
        let config = BatchConfig::default();
        let msg = ToWorkletMessage::UpdateBatchConfig { config: config.clone() };
        let obj = msg.to_js_object().unwrap();
        let deserialized = ToWorkletMessage::from_js_object(&obj).unwrap();
        assert_eq!(msg, deserialized);
    }

    #[wasm_bindgen_test]
    fn test_serialization_round_trip_from_worklet() {
        // Test simple message
        let msg = FromWorkletMessage::ProcessingStarted;
        let obj = msg.to_js_object().unwrap();
        let deserialized = FromWorkletMessage::from_js_object(&obj).unwrap();
        assert_eq!(msg, deserialized);
        
        // Test message with data
        let data = AudioDataBatch {
            sample_rate: 48000.0,
            sample_count: 1024,
            buffer_length: 4096,
            timestamp: 12345.0,
            sequence_number: Some(42),
            buffer_id: None,
            buffer_pool_stats: None,
        };
        let msg = FromWorkletMessage::AudioDataBatch { data: data.clone() };
        let obj = msg.to_js_object().unwrap();
        let deserialized = FromWorkletMessage::from_js_object(&obj).unwrap();
        assert_eq!(msg, deserialized);
    }

    #[wasm_bindgen_test]
    fn test_envelope_serialization() {
        let payload = ToWorkletMessage::StartProcessing;
        let envelope = MessageEnvelope::new(payload);
        
        let serializer = MessageSerializer::new();
        let obj = serializer.serialize_envelope(&envelope).unwrap();
        
        let deserializer = MessageDeserializer::new();
        let deserialized = deserializer.deserialize_envelope::<ToWorkletMessage>(&obj).unwrap();
        
        assert_eq!(envelope.payload, deserialized.payload);
        assert_eq!(envelope.message_id, deserialized.message_id);
        assert_eq!(envelope.timestamp, deserialized.timestamp);
    }

    #[wasm_bindgen_test]
    fn test_validation_errors() {
        // Test invalid batch config
        let invalid_config = BatchConfig {
            batch_size: 0, // Invalid
            max_queue_size: 8,
            timeout_ms: 100,
            enable_compression: false,
        };
        assert!(invalid_config.validate().is_err());
        
        // Test invalid audio data batch
        let invalid_data = AudioDataBatch {
            sample_rate: -1.0, // Invalid
            sample_count: 1024,
            buffer_length: 4096,
            timestamp: 12345.0,
            sequence_number: None,
            buffer_id: None,
            buffer_pool_stats: None,
        };
        assert!(invalid_data.validate().is_err());
    }

    #[wasm_bindgen_test]
    fn test_serialization_errors() {
        // Test invalid message type deserialization
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &"type".into(), &"invalidType".into()).unwrap();
        
        let result = ToWorkletMessage::from_js_object(&obj);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            SerializationError::InvalidPropertyType(msg) => {
                assert!(msg.contains("Unknown message type"));
            }
            _ => panic!("Expected InvalidPropertyType error"),
        }
    }

    #[wasm_bindgen_test]
    fn test_test_signal_config_serialization() {
        use crate::engine::audio::test_signal_generator::TestWaveform;
        
        let config = TestSignalGeneratorConfig {
            enabled: true,
            frequency: 440.0,
            amplitude: 0.5,
            waveform: TestWaveform::Sine,
            sample_rate: 48000.0,
        };
        
        let obj = config.to_js_object().unwrap();
        let deserialized = TestSignalGeneratorConfig::from_js_object(&obj).unwrap();
        
        assert_eq!(config, deserialized);
    }

    #[wasm_bindgen_test]
    fn test_worklet_error_serialization() {
        let error = WorkletError {
            code: WorkletErrorCode::BufferOverflow,
            message: "Buffer overflow occurred".to_string(),
            timestamp: 12345.0,
            context: Some(ErrorContext::new("audio_processor.rs:123".to_string())
                .with_system_state(SystemState::basic(
                    None,
                    None, 
                    Some(true)
                ))),
        };
        
        let obj = error.to_js_object().unwrap();
        let deserialized = WorkletError::from_js_object(&obj).unwrap();
        
        assert_eq!(error, deserialized);
    }

    #[wasm_bindgen_test]
    fn test_message_construction_utilities() {
        // Test message ID generation
        let generator = MessageIdGenerator::new();
        let id1 = generator.next_id();
        let id2 = generator.next_id();
        assert_ne!(id1, id2);
        assert!(id2 > id1);
        
        // Test global ID generation
        let global_id1 = generate_unique_message_id();
        let global_id2 = generate_unique_message_id();
        assert_ne!(global_id1, global_id2);
        
        // Test timestamp generation
        let timestamp1 = get_high_resolution_timestamp();
        let timestamp2 = get_high_resolution_timestamp();
        assert!(timestamp2 >= timestamp1);
    }

    #[wasm_bindgen_test]
    fn test_message_builder() {
        let payload = ToWorkletMessage::StartProcessing;
        let envelope = MessageBuilder::envelope(payload.clone()).unwrap();
        
        assert_eq!(envelope.payload, payload);
        assert!(envelope.message_id > 0);
        assert!(envelope.timestamp > 0.0);
        
        // Test with specific ID
        let specific_id = 42;
        let envelope_with_id = MessageBuilder::envelope_with_id(payload.clone(), specific_id).unwrap();
        assert_eq!(envelope_with_id.message_id, specific_id);
        assert_eq!(envelope_with_id.payload, payload);
    }

    #[wasm_bindgen_test]
    fn test_constructor_validation() {
        use crate::engine::audio::test_signal_generator::TestWaveform;
        
        // Test valid construction
        let valid_config = TestSignalGeneratorConfig::new(
            true,
            440.0,
            0.5,
            TestWaveform::Sine,
            48000.0
        );
        assert!(valid_config.is_ok());
        
        // Test invalid construction
        let invalid_config = TestSignalGeneratorConfig::new(
            true,
            -440.0, // Invalid frequency
            0.5,
            TestWaveform::Sine,
            48000.0
        );
        assert!(invalid_config.is_err());
    }

    #[wasm_bindgen_test]
    fn test_message_factory() {
        let factory = AudioWorkletMessageFactory::new();
        
        // Test start processing message
        let start_msg = factory.start_processing().unwrap();
        assert!(matches!(start_msg.payload, ToWorkletMessage::StartProcessing));
        assert!(start_msg.message_id > 0);
        assert!(start_msg.timestamp > 0.0);
        
        // Test processor ready message
        let ready_msg = factory.processor_ready(Some(1024)).unwrap();
        assert!(matches!(ready_msg.payload, FromWorkletMessage::ProcessorReady { batch_size: Some(1024) }));
        
        // Test error message creation
        let error_msg = factory.create_worklet_error(
            WorkletErrorCode::BufferOverflow,
            "Test error".to_string(),
            "test_location".to_string(),
            Some("test_state".to_string())
        ).unwrap();
        assert!(matches!(error_msg.payload, FromWorkletMessage::ProcessingError { .. }));
    }

    #[wasm_bindgen_test]
    fn test_factory_convenience_methods() {
        let factory = AudioWorkletMessageFactory::new();
        
        // Test audio data batch creation
        let batch_msg = factory.create_audio_data_batch(
            48000.0,
            1024,
            4096,
            Some(42)
        ).unwrap();
        
        if let FromWorkletMessage::AudioDataBatch { data } = &batch_msg.payload {
            assert_eq!(data.sample_rate, 48000.0);
            assert_eq!(data.sample_count, 1024);
            assert_eq!(data.buffer_length, 4096);
            assert_eq!(data.sequence_number, Some(42));
        } else {
            panic!("Expected AudioDataBatch message");
        }
        
    }

    #[wasm_bindgen_test]
    fn test_request_response_correlation() {
        let factory = AudioWorkletMessageFactory::new();
        
        let request_id = 123;
        let response = factory.processor_ready_response(request_id, Some(1024)).unwrap();
        
        assert_eq!(response.message_id, request_id);
        assert!(matches!(response.payload, FromWorkletMessage::ProcessorReady { batch_size: Some(1024) }));
    }

    #[wasm_bindgen_test]
    fn test_construction_error_handling() {
        // Test invalid parameter
        let result = FromWorkletMessage::processor_ready(Some(0));
        assert!(result.is_err());
        
        match result.unwrap_err() {
            MessageConstructionError::InvalidParameter(msg) => {
                assert!(msg.contains("batch_size cannot be zero"));
            }
            _ => panic!("Expected InvalidParameter error"),
        }
        
        // Test validation failure
        let invalid_memory = MemoryUsage::new(1024, 2048, 8); // used > total
        assert!(invalid_memory.is_err());
        
        match invalid_memory.unwrap_err() {
            MessageConstructionError::ValidationFailed(msg) => {
                assert!(msg.contains("used_heap cannot exceed heap_size"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[wasm_bindgen_test]
    fn test_factory_id_generation() {
        let factory = AudioWorkletMessageFactory::new();
        
        // Reset for predictable testing
        factory.reset_id_generator();
        
        let msg1 = factory.start_processing().unwrap();
        let msg2 = factory.stop_processing().unwrap();
        
        assert_ne!(msg1.message_id, msg2.message_id);
        assert!(msg2.message_id > msg1.message_id);
    }
}