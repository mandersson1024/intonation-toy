// Type-safe message protocol for AudioWorklet communication

use crate::engine::audio::signal_generator::SignalGeneratorConfig;
use crate::common::utils;
use js_sys::{Object, Reflect};
use wasm_bindgen::{JsValue, JsCast};

/// Message types sent from main thread to AudioWorklet
#[derive(Debug, Clone, PartialEq)]
pub enum ToWorkletMessage {
    StartProcessing,
    StopProcessing,
    UpdateBatchConfig { config: BatchConfig },
    ReturnBuffer { buffer_id: u32 },
    
}

/// Message types sent from AudioWorklet to main thread
#[derive(Debug, Clone, PartialEq)]
pub enum FromWorkletMessage {
    ProcessorReady { batch_size: Option<usize> },
    ProcessingStarted,
    ProcessingStopped,
    AudioDataBatch { data: AudioDataBatch },
    ProcessingError { error: WorkletError },
    BatchConfigUpdated { config: BatchConfig },
    
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioDataBatch {
    pub sample_rate: u32,
    pub sample_count: usize,
    pub buffer_length: usize,
    pub timestamp: f64,
    pub sequence_number: Option<u32>,
    pub buffer_id: Option<u32>,
    pub buffer_pool_stats: Option<BufferPoolStats>,
}


#[derive(Debug, Clone, PartialEq)]
pub struct MemoryUsage {
    pub heap_size: usize,
    pub used_heap: usize,
    pub active_buffers: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BufferPoolStats {
    pub pool_size: u32,
    pub available_buffers: u32,
    pub in_use_buffers: u32,
    pub total_buffers: u32,
    pub acquire_count: u32,
    pub transfer_count: u32,
    pub pool_exhausted_count: u32,
    pub consecutive_pool_failures: u32,
    pub pool_hit_rate: f32,
    pub pool_efficiency: f32,
    pub buffer_utilization_percent: f32,
    pub total_megabytes_transferred: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchConfig {
    pub batch_size: usize,
    pub max_queue_size: usize,
    pub timeout_ms: u32,
    pub enable_compression: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: crate::app_config::BUFFER_SIZE,
            max_queue_size: 8,
            timeout_ms: 100,
            enable_compression: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkletError {
    pub code: WorkletErrorCode,
    pub message: String,
    pub context: Option<ErrorContext>,
    pub timestamp: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkletErrorCode {
    InitializationFailed,
    ProcessingFailed,
    BufferOverflow,
    InvalidConfiguration,
    MemoryAllocationFailed,
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

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorContext {
    pub location: String,
    pub stack_trace: Option<Vec<String>>,
    pub message_context: Option<MessageContext>,
    pub system_state: Option<SystemState>,
    pub debug_info: Option<String>,
    pub timestamp: f64,
    pub thread_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageContext {
    pub message_type: String,
    pub direction: MessageDirection,
    pub message_id: Option<u32>,
    pub message_timestamp: Option<f64>,
    pub message_size: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageDirection {
    ToWorklet,
    FromWorklet,
    Internal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemState {
    pub memory_usage: Option<usize>,
    pub queue_depth: Option<usize>,
    pub active_buffers: Option<usize>,
    pub audio_processing_active: Option<bool>,
    pub sample_rate: Option<f64>,
    pub buffer_size: Option<usize>,
    pub processor_load: Option<f32>,
    pub available_heap: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageEnvelope<T> {
    pub message_id: u32,
    pub timestamp: f64,
    pub payload: T,
}

pub type ToWorkletEnvelope = MessageEnvelope<ToWorkletMessage>;
pub type FromWorkletEnvelope = MessageEnvelope<FromWorkletMessage>;

impl<T> MessageEnvelope<T> {
    pub fn new(payload: T) -> Self {
        Self {
            message_id: generate_unique_message_id(),
            timestamp: get_current_timestamp(),
            payload,
        }
    }
    
}

fn get_current_timestamp() -> f64 {
    js_sys::Date::now()
}

pub type SerializationResult<T> = Result<T, SerializationError>;

/// Serialization error types
#[derive(Debug, Clone, PartialEq)]
pub enum SerializationError {
    ObjectCreationFailed(String),
    PropertySetFailed(String),
    PropertyGetFailed(String),
    InvalidPropertyType(String),
    MissingProperty(String),
    ValidationFailed(String),
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





pub trait ToJsMessage {
    fn to_js_object(&self) -> SerializationResult<Object>;
    
    fn to_js_value(&self) -> SerializationResult<JsValue> {
        self.to_js_object().map(|obj| obj.into())
    }
}

pub trait FromJsMessage: Sized {
    fn from_js_object(obj: &Object) -> SerializationResult<Self>;
    
    fn from_js_value(value: &JsValue) -> SerializationResult<Self> {
        let obj = value.dyn_ref::<Object>()
            .ok_or_else(|| SerializationError::InvalidPropertyType("Expected object".to_string()))?;
        Self::from_js_object(obj)
    }
}

pub trait MessageValidator {
    fn validate(&self) -> SerializationResult<()>;
}

#[derive(Default)]
pub struct MessageSerializer;

impl MessageSerializer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn serialize_envelope<T: ToJsMessage + MessageValidator>(
        &self,
        envelope: &MessageEnvelope<T>,
    ) -> SerializationResult<Object> {
        envelope.payload.validate()?;
        
        let obj = Object::new();
        
        self.set_property(&obj, "messageId", &envelope.message_id.into())?;
        self.set_property(&obj, "timestamp", &envelope.timestamp.into())?;
        
        let payload_obj = envelope.payload.to_js_object()?;
        self.set_property(&obj, "payload", &payload_obj.into())?;
        
        Ok(obj)
    }
    
    fn set_property(&self, obj: &Object, key: &str, value: &JsValue) -> SerializationResult<()> {
        Reflect::set(obj, &key.into(), value)
            .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set '{}': {:?}", key, e)))?;
        Ok(())
    }
}


// Message Type Implementations
impl ToJsMessage for ToWorkletMessage {
    fn to_js_object(&self) -> SerializationResult<Object> {
        let obj = Object::new();
        let set = |k: &str, v: JsValue| {
            Reflect::set(&obj, &k.into(), &v)
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set {}: {:?}", k, e)))
        };
        
        match self {
            ToWorkletMessage::StartProcessing => {
                set("type", "startProcessing".into())?;
            }
            ToWorkletMessage::StopProcessing => {
                set("type", "stopProcessing".into())?;
            }
            ToWorkletMessage::UpdateBatchConfig { config } => {
                set("type", "updateBatchConfig".into())?;
                set("config", config.to_js_object()?.into())?;
            }
            ToWorkletMessage::ReturnBuffer { buffer_id } => {
                set("type", "returnBuffer".into())?;
                set("bufferId", (*buffer_id).into())?;
            }
        }
        
        Ok(obj)
    }
}

impl FromJsMessage for ToWorkletMessage {
    fn from_js_object(obj: &Object) -> SerializationResult<Self> {
        let get = |k: &str| {
            Reflect::get(obj, &k.into())
                .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get {}: {:?}", k, e)))
        };
        
        let msg_type = get("type")?
            .as_string()
            .ok_or_else(|| SerializationError::InvalidPropertyType("type must be string".to_string()))?;
        
        match msg_type.as_str() {
            "startProcessing" => Ok(ToWorkletMessage::StartProcessing),
            "stopProcessing" => Ok(ToWorkletMessage::StopProcessing),
            "updateBatchConfig" => {
                let config_obj = get("config")?
                    .dyn_into::<Object>()
                    .map_err(|_| SerializationError::InvalidPropertyType("config must be object".to_string()))?;
                Ok(ToWorkletMessage::UpdateBatchConfig { 
                    config: BatchConfig::from_js_object(&config_obj)? 
                })
            }
            "returnBuffer" => {
                let buffer_id = get("bufferId")?
                    .as_f64()
                    .ok_or_else(|| SerializationError::InvalidPropertyType("bufferId must be number".to_string()))? as u32;
                Ok(ToWorkletMessage::ReturnBuffer { buffer_id })
            }
            _ => Err(SerializationError::InvalidPropertyType(format!("Unknown message type: {}", msg_type))),
        }
    }
}

impl MessageValidator for ToWorkletMessage {
    fn validate(&self) -> SerializationResult<()> {
        match self {
            ToWorkletMessage::StartProcessing | ToWorkletMessage::StopProcessing => Ok(()),
            ToWorkletMessage::UpdateBatchConfig { config } => config.validate(),
            ToWorkletMessage::ReturnBuffer { buffer_id: _ } => Ok(()),
        }
    }
}

impl ToJsMessage for FromWorkletMessage {
    fn to_js_object(&self) -> SerializationResult<Object> {
        let obj = Object::new();
        let set = |k: &str, v: JsValue| {
            Reflect::set(&obj, &k.into(), &v)
                .map_err(|e| SerializationError::PropertySetFailed(format!("Failed to set {}: {:?}", k, e)))
        };
        
        match self {
            FromWorkletMessage::ProcessorReady { batch_size } => {
                set("type", "processorReady".into())?;
                if let Some(size) = batch_size {
                    set("batchSize", (*size as f64).into())?;
                }
            }
            FromWorkletMessage::ProcessingStarted => {
                set("type", "processingStarted".into())?;
            }
            FromWorkletMessage::ProcessingStopped => {
                set("type", "processingStopped".into())?;
            }
            FromWorkletMessage::AudioDataBatch { data } => {
                set("type", "audioDataBatch".into())?;
                set("data", data.to_js_object()?.into())?;
            }
            FromWorkletMessage::ProcessingError { error } => {
                set("type", "processingError".into())?;
                set("error", error.to_js_object()?.into())?;
            }
            FromWorkletMessage::BatchConfigUpdated { config } => {
                set("type", "batchConfigUpdated".into())?;
                set("config", config.to_js_object()?.into())?;
            }
        }
        
        Ok(obj)
    }
}

impl FromJsMessage for FromWorkletMessage {
    fn from_js_object(obj: &Object) -> SerializationResult<Self> {
        let get = |k: &str| {
            Reflect::get(obj, &k.into())
                .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get {}: {:?}", k, e)))
        };
        
        let msg_type = get("type")?
            .as_string()
            .ok_or_else(|| SerializationError::InvalidPropertyType("type must be string".to_string()))?;
        
        match msg_type.as_str() {
            "processorReady" => {
                let batch_size = match get("batchSize") {
                    Ok(value) if !value.is_undefined() => {
                        Some(value.as_f64()
                            .ok_or_else(|| SerializationError::InvalidPropertyType("batchSize must be number".to_string()))? as usize)
                    }
                    _ => None,
                };
                Ok(FromWorkletMessage::ProcessorReady { batch_size })
            }
            "processingStarted" => Ok(FromWorkletMessage::ProcessingStarted),
            "processingStopped" => Ok(FromWorkletMessage::ProcessingStopped),
            "audioDataBatch" => {
                let data_obj = get("data")?
                    .dyn_into::<Object>()
                    .map_err(|_| SerializationError::InvalidPropertyType("data must be object".to_string()))?;
                Ok(FromWorkletMessage::AudioDataBatch { 
                    data: AudioDataBatch::from_js_object(&data_obj)? 
                })
            }
            "processingError" => {
                let error_obj = get("error")?
                    .dyn_into::<Object>()
                    .map_err(|_| SerializationError::InvalidPropertyType("error must be object".to_string()))?;
                Ok(FromWorkletMessage::ProcessingError { 
                    error: WorkletError::from_js_object(&error_obj)? 
                })
            }
            "batchConfigUpdated" => {
                let config_obj = get("config")?
                    .dyn_into::<Object>()
                    .map_err(|_| SerializationError::InvalidPropertyType("config must be object".to_string()))?;
                Ok(FromWorkletMessage::BatchConfigUpdated { 
                    config: BatchConfig::from_js_object(&config_obj)? 
                })
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
            FromWorkletMessage::BatchConfigUpdated { config } => config.validate(),
        }
    }
}

// Helper macro for simpler property getting
macro_rules! get_optional {
    ($obj:expr, $key:expr, $convert:expr) => {
        match Reflect::get($obj, &$key.into()) {
            Ok(value) if !value.is_undefined() => Some($convert(value)?),
            _ => None,
        }
    };
}

// Data structure implementations
impl ToJsMessage for AudioDataBatch {
    fn to_js_object(&self) -> SerializationResult<Object> {
        let obj = Object::new();
        
        Reflect::set(&obj, &"sampleRate".into(), &(self.sample_rate as f64).into())
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
        let get = |k: &str| {
            Reflect::get(obj, &k.into())
                .map_err(|e| SerializationError::PropertyGetFailed(format!("Failed to get {}: {:?}", k, e)))
        };
        
        let get_num = |k: &str| -> SerializationResult<f64> {
            get(k)?.as_f64()
                .ok_or_else(|| SerializationError::InvalidPropertyType(format!("{} must be number", k)))
        };
        
        Ok(AudioDataBatch {
            sample_rate: get_num("sampleRate")? as u32,
            sample_count: get_num("sampleCount")? as usize,
            buffer_length: get_num("bufferLength")? as usize,
            timestamp: get_num("timestamp")?,
            sequence_number: get_optional!(obj, "sequenceNumber", |v: JsValue| 
                v.as_f64().ok_or_else(|| SerializationError::InvalidPropertyType("sequenceNumber must be number".to_string())).map(|n| n as u32)),
            buffer_id: get_optional!(obj, "bufferId", |v: JsValue|
                v.as_f64().ok_or_else(|| SerializationError::InvalidPropertyType("bufferId must be number".to_string())).map(|n| n as u32)),
            buffer_pool_stats: get_optional!(obj, "bufferPoolStats", |v: JsValue| {
                let stats_obj = v.dyn_into::<Object>()
                    .map_err(|_| SerializationError::InvalidPropertyType("bufferPoolStats must be object".to_string()))?;
                BufferPoolStats::from_js_object(&stats_obj)
            }),
        })
    }
}

impl MessageValidator for AudioDataBatch {
    fn validate(&self) -> SerializationResult<()> {
        if self.sample_rate == 0 {
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
impl ToJsMessage for SignalGeneratorConfig {
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
        
        
        Ok(obj)
    }
}

impl FromJsMessage for SignalGeneratorConfig {
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
            as u32;
        
        Ok(SignalGeneratorConfig {
            enabled,
            frequency,
            amplitude,
            sample_rate,
        })
    }
}

impl MessageValidator for SignalGeneratorConfig {
    fn validate(&self) -> SerializationResult<()> {
        if self.frequency <= 0.0 {
            return Err(SerializationError::ValidationFailed("frequency must be positive".to_string()));
        }
        if self.amplitude < 0.0 || self.amplitude > 1.0 {
            return Err(SerializationError::ValidationFailed("amplitude must be between 0.0 and 1.0".to_string()));
        }
        if self.sample_rate == 0 {
            return Err(SerializationError::ValidationFailed("sample_rate must be positive".to_string()));
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
            .unwrap_or_else(js_sys::Date::now);
        
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

pub type MessageConstructionResult<T> = Result<T, MessageConstructionError>;

#[derive(Debug, Clone, PartialEq)]
pub enum MessageConstructionError {
    InvalidParameter(String),
    MissingParameter(String),
    ValidationFailed(String),
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

#[derive(Clone, Default)]
pub struct MessageIdGenerator {
    counter: std::rc::Rc<std::cell::RefCell<u32>>,
}

impl MessageIdGenerator {
    pub fn new() -> Self {
        Self {
            counter: std::rc::Rc::new(std::cell::RefCell::new(0)),
        }
    }
    
    pub fn next_id(&self) -> u32 {
        let mut counter = self.counter.borrow_mut();
        *counter = counter.wrapping_add(1);
        *counter
    }
}

thread_local! {
    static MESSAGE_ID_GENERATOR: MessageIdGenerator = MessageIdGenerator::new();
}

pub fn generate_unique_message_id() -> u32 {
    MESSAGE_ID_GENERATOR.with(|generator| generator.next_id())
}

pub fn get_high_resolution_timestamp() -> f64 {
    utils::get_high_resolution_time()
}


impl ToWorkletMessage {
    pub fn start_processing() -> Self {
        Self::StartProcessing
    }
    
    pub fn stop_processing() -> Self {
        Self::StopProcessing
    }
    
    pub fn update_batch_config(config: BatchConfig) -> MessageConstructionResult<Self> {
        config.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        Ok(Self::UpdateBatchConfig { config })
    }
    
    pub fn return_buffer(buffer_id: u32) -> Self {
        Self::ReturnBuffer { buffer_id }
    }
}

impl FromWorkletMessage {
    pub fn processor_ready(batch_size: Option<usize>) -> MessageConstructionResult<Self> {
        if let Some(size) = batch_size {
            if size == 0 {
                return Err(MessageConstructionError::InvalidParameter("batch_size cannot be zero".to_string()));
            }
        }
        Ok(Self::ProcessorReady { batch_size })
    }
    
    pub fn processing_started() -> Self {
        Self::ProcessingStarted
    }
    
    pub fn processing_stopped() -> Self {
        Self::ProcessingStopped
    }
    
    pub fn audio_data_batch(data: AudioDataBatch) -> MessageConstructionResult<Self> {
        data.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        Ok(Self::AudioDataBatch { data })
    }
    
    pub fn processing_error(error: WorkletError) -> MessageConstructionResult<Self> {
        error.validate().map_err(|e| MessageConstructionError::ValidationFailed(e.to_string()))?;
        Ok(Self::ProcessingError { error })
    }
}






impl ErrorContext {
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

}

impl MessageContext {
    pub fn new(message_type: String, direction: MessageDirection) -> Self {
        Self {
            message_type,
            direction,
            message_id: None,
            message_timestamp: None,
            message_size: None,
        }
    }
}

impl Default for SystemState {
    fn default() -> Self {
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
}

impl SystemState {
    pub fn new() -> Self {
        Self::default()
    }

}


#[derive(Clone, Default)]
pub struct AudioWorkletMessageFactory {
    id_generator: MessageIdGenerator,
}

impl AudioWorkletMessageFactory {
    pub fn new() -> Self {
        Self {
            id_generator: MessageIdGenerator::new(),
        }
    }
    
    pub fn generate_id(&self) -> u32 {
        self.id_generator.next_id()
    }
    
    fn create_envelope<T>(&self, message: T) -> MessageEnvelope<T> {
        MessageEnvelope {
            message_id: self.generate_id(),
            timestamp: get_high_resolution_timestamp(),
            payload: message,
        }
    }
    
    // ToWorkletMessage factory methods
    pub fn start_processing(&self) -> MessageConstructionResult<ToWorkletEnvelope> {
        Ok(self.create_envelope(ToWorkletMessage::start_processing()))
    }
    
    pub fn stop_processing(&self) -> MessageConstructionResult<ToWorkletEnvelope> {
        Ok(self.create_envelope(ToWorkletMessage::stop_processing()))
    }
    
    pub fn update_batch_config(&self, config: BatchConfig) -> MessageConstructionResult<ToWorkletEnvelope> {
        Ok(self.create_envelope(ToWorkletMessage::update_batch_config(config)?))
    }
    
    pub fn return_buffer(&self, buffer_id: u32) -> MessageConstructionResult<ToWorkletEnvelope> {
        Ok(self.create_envelope(ToWorkletMessage::return_buffer(buffer_id)))
    }
    
    // FromWorkletMessage factory methods
    pub fn processor_ready(&self, batch_size: Option<usize>) -> MessageConstructionResult<FromWorkletEnvelope> {
        Ok(self.create_envelope(FromWorkletMessage::processor_ready(batch_size)?))
    }
    
    pub fn processing_started(&self) -> MessageConstructionResult<FromWorkletEnvelope> {
        Ok(self.create_envelope(FromWorkletMessage::processing_started()))
    }
    
    pub fn processing_stopped(&self) -> MessageConstructionResult<FromWorkletEnvelope> {
        Ok(self.create_envelope(FromWorkletMessage::processing_stopped()))
    }
    
    pub fn audio_data_batch(&self, data: AudioDataBatch) -> MessageConstructionResult<FromWorkletEnvelope> {
        Ok(self.create_envelope(FromWorkletMessage::audio_data_batch(data)?))
    }
    
    pub fn processing_error(&self, error: WorkletError) -> MessageConstructionResult<FromWorkletEnvelope> {
        Ok(self.create_envelope(FromWorkletMessage::processing_error(error)?))
    }
}

