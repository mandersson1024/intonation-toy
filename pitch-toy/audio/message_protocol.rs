// Structured message protocol for AudioWorklet communication
// Provides type-safe message construction and parsing for cross-thread communication

use crate::audio::test_signal_generator::{TestSignalGeneratorConfig, BackgroundNoiseConfig};
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
    
    /// Helper method to get string property
    fn get_string_property(&self, obj: &Object, key: &str) -> SerializationResult<String> {
        let value = self.get_property(obj, key)?;
        value.as_string()
            .ok_or_else(|| SerializationError::InvalidPropertyType(format!("Property '{}' is not a string", key)))
    }
    
    /// Helper method to get object property
    fn get_object_property(&self, obj: &Object, key: &str) -> SerializationResult<Object> {
        let value = self.get_property(obj, key)?;
        value.dyn_into::<Object>()
            .map_err(|_| SerializationError::InvalidPropertyType(format!("Property '{}' is not an object", key)))
    }
    
    /// Helper method to get optional object property
    fn get_optional_object_property(&self, obj: &Object, key: &str) -> SerializationResult<Option<Object>> {
        match Reflect::get(obj, &key.into()) {
            Ok(value) if value.is_undefined() || value.is_null() => Ok(None),
            Ok(value) => {
                let obj = value.dyn_into::<Object>()
                    .map_err(|_| SerializationError::InvalidPropertyType(format!("Property '{}' is not an object", key)))?;
                Ok(Some(obj))
            }
            Err(e) => Err(SerializationError::PropertyGetFailed(format!("Failed to get '{}': {:?}", key, e))),
        }
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
    
    /// Helper method to get optional property from object
    fn get_optional_property(&self, obj: &Object, key: &str) -> SerializationResult<Option<JsValue>> {
        match Reflect::get(obj, &key.into()) {
            Ok(value) if value.is_undefined() || value.is_null() => Ok(None),
            Ok(value) => Ok(Some(value)),
            Err(e) => Err(SerializationError::PropertyGetFailed(format!("Failed to get '{}': {:?}", key, e))),
        }
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
            FromWorkletMessage::StatusUpdate { status } => {
                Reflect::set(&obj, &"type".into(), &"statusUpdate".into())
                    .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set type: {:?}", e)))?;
                let status_obj = status.to_js_object()?;
                Reflect::set(&obj, &"status".into(), &status_obj.into())
                    .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set status: {:?}", e)))?;
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
            "statusUpdate" => {
                let status_obj = Reflect::get(obj, &"status".into())
                    .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get status: {:?}", e)))?
                    .dyn_into::<Object>()
                    .map_err(|_| SerializationError::InvalidPropertyType("status must be object".to_string()))?;
                let status = ProcessorStatus::from_js_object(&status_obj)?;
                Ok(FromWorkletMessage::StatusUpdate { status })
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
            FromWorkletMessage::StatusUpdate { status } => status.validate(),
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
        
        Ok(AudioDataBatch {
            sample_rate,
            sample_count,
            buffer_length,
            timestamp,
            sequence_number,
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
        
        Ok(ProcessorStatus {
            active,
            sample_rate,
            buffer_size,
            processed_batches,
            avg_processing_time_ms,
            memory_usage,
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
            crate::audio::test_signal_generator::TestWaveform::Sine => "sine",
            crate::audio::test_signal_generator::TestWaveform::Square => "square",
            crate::audio::test_signal_generator::TestWaveform::Sawtooth => "sawtooth",
            crate::audio::test_signal_generator::TestWaveform::Triangle => "triangle",
            crate::audio::test_signal_generator::TestWaveform::WhiteNoise => "whiteNoise",
            crate::audio::test_signal_generator::TestWaveform::PinkNoise => "pinkNoise",
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
            "sine" => crate::audio::test_signal_generator::TestWaveform::Sine,
            "square" => crate::audio::test_signal_generator::TestWaveform::Square,
            "sawtooth" => crate::audio::test_signal_generator::TestWaveform::Sawtooth,
            "triangle" => crate::audio::test_signal_generator::TestWaveform::Triangle,
            "whiteNoise" => crate::audio::test_signal_generator::TestWaveform::WhiteNoise,
            "pinkNoise" => crate::audio::test_signal_generator::TestWaveform::PinkNoise,
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
            crate::audio::test_signal_generator::TestWaveform::WhiteNoise => "whiteNoise",
            crate::audio::test_signal_generator::TestWaveform::PinkNoise => "pinkNoise",
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
            "whiteNoise" => crate::audio::test_signal_generator::TestWaveform::WhiteNoise,
            "pinkNoise" => crate::audio::test_signal_generator::TestWaveform::PinkNoise,
            _ => crate::audio::test_signal_generator::TestWaveform::WhiteNoise, // Default to white noise
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
        
        if let Some(system_state) = &self.system_state {
            Reflect::set(&obj, &"systemState".into(), &system_state.clone().into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set systemState: {:?}", e)))?;
        }
        
        if let Some(debug_info) = &self.debug_info {
            Reflect::set(&obj, &"debugInfo".into(), &debug_info.clone().into())
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set debugInfo: {:?}", e)))?;
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
        
        let system_state = match Reflect::get(obj, &"systemState".into()) {
            Ok(value) if !value.is_undefined() => {
                Some(value.as_string()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("systemState must be string".to_string()))?)
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
        
        Ok(ErrorContext {
            location,
            system_state,
            debug_info,
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
        use crate::audio::test_signal_generator::TestWaveform;
        
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
            context: Some(ErrorContext {
                location: "audio_processor.rs:123".to_string(),
                system_state: Some("processing=true".to_string()),
                debug_info: None,
            }),
        };
        
        let obj = error.to_js_object().unwrap();
        let deserialized = WorkletError::from_js_object(&obj).unwrap();
        
        assert_eq!(error, deserialized);
    }
}