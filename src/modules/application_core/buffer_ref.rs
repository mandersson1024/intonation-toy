//! # Zero-Copy Buffer Reference Manager
//!
//! This module provides a zero-copy buffer reference system for efficient audio data sharing
//! between modules without performance-killing memory copies. The system uses Arc-based
//! reference counting for automatic memory management and lock-free access patterns
//! optimized for real-time audio processing.
//!
//! ## Key Components
//!
//! - [`BufferRef<T>`]: Shared reference to audio buffer data with metadata
//! - [`BufferMetadata`]: Audio properties and timing information
//! - [`BufferManager`]: Lifecycle management and buffer pool coordination
//! - [`BufferEvent`]: Events containing buffer metadata (no audio data)
//!
//! ## Usage Example
//!
//! ```rust
//! use crate::modules::application_core::buffer_ref::*;
//!
//! // Create audio buffer data
//! let samples: Vec<f32> = vec![0.0; 1024];
//! let metadata = BufferMetadata::new(44100, 1, 1024);
//! 
//! // Create buffer reference for sharing
//! let buffer_ref = BufferRef::new(samples, metadata);
//! 
//! // Clone reference for zero-copy sharing
//! let shared_ref = buffer_ref.clone();
//! 
//! // Access audio data through reference
//! let audio_data = buffer_ref.data();
//! assert_eq!(audio_data.len(), 1024);
//! ```

use std::sync::Arc;
use std::fmt;
use std::any::Any;
use super::event_bus::{Event, EventPriority, get_timestamp_ns};

/// Zero-copy buffer reference with automatic memory management.
///
/// BufferRef provides shared access to audio buffer data using Arc for reference counting.
/// Multiple BufferRef instances can reference the same underlying data without copying,
/// enabling efficient audio processing pipelines.
///
/// ## Performance Characteristics
///
/// - Zero-copy cloning through Arc reference counting
/// - Lock-free read access to buffer data
/// - Automatic cleanup when all references are dropped
/// - Cache-friendly memory layout for audio processing
///
/// ## Type Safety
///
/// The generic type parameter T allows for different audio data types:
/// - `f32`: Standard floating-point audio samples
/// - `i16`: 16-bit integer audio samples  
/// - `i32`: 24/32-bit integer audio samples
/// - Custom audio data structures
#[derive(Debug, Clone)]
pub struct BufferRef<T> {
    /// Shared reference to the actual audio data
    data: Arc<[T]>,
    /// Audio metadata and properties
    metadata: BufferMetadata,
    /// Unique buffer identifier for tracking
    buffer_id: BufferId,
}

impl<T> BufferRef<T> {
    /// Creates a new buffer reference from audio data and metadata.
    ///
    /// # Arguments
    /// * `data` - Audio sample data to be shared
    /// * `metadata` - Audio properties and timing information
    ///
    /// # Returns
    /// New BufferRef instance with unique buffer ID
    ///
    /// # Performance
    /// - Conversion to Arc<[T]> may copy data once during creation
    /// - All subsequent clones are zero-cost reference increments
    pub fn new(data: Vec<T>, metadata: BufferMetadata) -> Self {
        Self {
            data: Arc::from(data.into_boxed_slice()),
            metadata,
            buffer_id: BufferId::new(),
        }
    }
    
    /// Creates a buffer reference from existing Arc data.
    ///
    /// This is the true zero-copy constructor for cases where data is already
    /// in Arc format from external sources.
    ///
    /// # Arguments
    /// * `data` - Existing Arc reference to audio data
    /// * `metadata` - Audio properties and timing information
    pub fn from_arc(data: Arc<[T]>, metadata: BufferMetadata) -> Self {
        Self {
            data,
            metadata,
            buffer_id: BufferId::new(),
        }
    }
    
    /// Returns a reference to the audio data.
    ///
    /// This provides zero-copy access to the underlying audio samples.
    /// Multiple threads can safely read from the same buffer simultaneously.
    ///
    /// # Performance
    /// - O(1) access time
    /// - No memory allocation or copying
    /// - Cache-friendly sequential access pattern
    pub fn data(&self) -> &[T] {
        &self.data
    }
    
    /// Returns the buffer metadata.
    ///
    /// Metadata includes sample rate, channel count, timing information,
    /// and other audio properties needed for processing.
    pub fn metadata(&self) -> &BufferMetadata {
        &self.metadata
    }
    
    /// Returns the unique buffer identifier.
    ///
    /// Buffer IDs can be used for tracking, debugging, and correlation
    /// with buffer events in the event system.
    pub fn buffer_id(&self) -> BufferId {
        self.buffer_id
    }
    
    /// Returns the number of active references to this buffer.
    ///
    /// Useful for debugging memory usage and ensuring proper cleanup.
    /// When this returns 1, this is the only reference to the data.
    pub fn reference_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }
    
    /// Creates a buffer event for this buffer reference.
    ///
    /// Buffer events contain metadata and buffer ID but no audio data,
    /// enabling efficient event system communication about buffer state
    /// changes without copying large audio data.
    ///
    /// # Arguments
    /// * `event_type` - Type of buffer event (Created, Processed, Released)
    /// * `priority` - Event processing priority
    pub fn create_event(&self, event_type: BufferEventType, priority: EventPriority) -> BufferEvent {
        BufferEvent {
            buffer_id: self.buffer_id,
            event_type,
            metadata: self.metadata.clone(),
            timestamp: get_timestamp_ns(),
            priority,
        }
    }
    
    /// Returns the size of the buffer data in bytes.
    ///
    /// Useful for memory usage monitoring and buffer pool management.
    pub fn size_bytes(&self) -> usize {
        self.data.len() * std::mem::size_of::<T>()
    }
}

/// Audio buffer metadata and properties.
///
/// BufferMetadata contains all the information needed to interpret and process
/// audio buffer data, including format, timing, and source information.
///
/// ## Web Audio API Compatibility
///
/// The metadata fields are designed to be compatible with Web Audio API
/// buffer formats and can be easily converted to/from AudioBuffer properties.
#[derive(Debug, Clone, PartialEq)]
pub struct BufferMetadata {
    /// Sample rate in Hz (e.g., 44100, 48000)
    pub sample_rate: u32,
    /// Number of audio channels (1=mono, 2=stereo, etc.)
    pub channels: u8,
    /// Number of audio frames (samples per channel)
    pub frame_count: usize,
    /// Timestamp when buffer was created/captured (nanoseconds since epoch)
    pub timestamp: u64,
    /// Duration of audio data in nanoseconds
    pub duration_ns: u64,
    /// Optional source identifier for debugging
    pub source: Option<String>,
    /// Buffer format information
    pub format: AudioFormat,
}

impl BufferMetadata {
    /// Creates new buffer metadata with current timestamp.
    ///
    /// Duration is automatically calculated from sample rate and frame count.
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz
    /// * `channels` - Number of audio channels
    /// * `frame_count` - Number of audio frames
    pub fn new(sample_rate: u32, channels: u8, frame_count: usize) -> Self {
        let duration_ns = if sample_rate > 0 {
            (frame_count as u64 * 1_000_000_000) / sample_rate as u64
        } else {
            0
        };
        
        Self {
            sample_rate,
            channels,
            frame_count,
            timestamp: get_timestamp_ns(),
            duration_ns,
            source: None,
            format: AudioFormat::F32,
        }
    }
    
    /// Creates metadata with custom timestamp and source.
    ///
    /// Useful for buffers received from external sources or for testing.
    pub fn with_source(sample_rate: u32, channels: u8, frame_count: usize, source: String) -> Self {
        let mut metadata = Self::new(sample_rate, channels, frame_count);
        metadata.source = Some(source);
        metadata
    }
    
    /// Returns the total number of samples in the buffer.
    ///
    /// This is frame_count * channels, representing the total size
    /// of the audio data array.
    pub fn total_samples(&self) -> usize {
        self.frame_count * self.channels as usize
    }
    
    /// Returns the buffer duration in seconds as a floating-point value.
    pub fn duration_seconds(&self) -> f64 {
        self.duration_ns as f64 / 1_000_000_000.0
    }
    
    /// Validates the metadata for consistency and supported formats.
    ///
    /// # Returns
    /// * `Ok(())` - Metadata is valid
    /// * `Err(error)` - Validation failed with error description
    pub fn validate(&self) -> Result<(), String> {
        if self.sample_rate == 0 {
            return Err("Sample rate cannot be zero".to_string());
        }
        
        if self.channels == 0 {
            return Err("Channel count cannot be zero".to_string());
        }
        
        if self.channels > 32 {
            return Err("Channel count exceeds maximum of 32".to_string());
        }
        
        if self.frame_count == 0 {
            return Err("Frame count cannot be zero".to_string());
        }
        
        // Check for reasonable sample rates (8kHz to 192kHz)
        if self.sample_rate < 8000 || self.sample_rate > 192000 {
            return Err(format!("Sample rate {} is outside supported range (8000-192000 Hz)", self.sample_rate));
        }
        
        Ok(())
    }
}

/// Audio data format enumeration.
///
/// Defines the supported audio sample formats for buffer references.
/// This enables type-safe handling of different audio data types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    /// 32-bit floating-point samples (range: -1.0 to 1.0)
    F32,
    /// 64-bit floating-point samples (range: -1.0 to 1.0)  
    F64,
    /// 16-bit signed integer samples
    I16,
    /// 32-bit signed integer samples
    I32,
    /// 8-bit unsigned integer samples
    U8,
}

impl AudioFormat {
    /// Returns the size in bytes for one sample of this format.
    pub fn sample_size_bytes(&self) -> usize {
        match self {
            AudioFormat::F32 => 4,
            AudioFormat::F64 => 8,
            AudioFormat::I16 => 2,
            AudioFormat::I32 => 4,
            AudioFormat::U8 => 1,
        }
    }
    
    /// Returns the format name as a string.
    pub fn name(&self) -> &'static str {
        match self {
            AudioFormat::F32 => "f32",
            AudioFormat::F64 => "f64",
            AudioFormat::I16 => "i16",
            AudioFormat::I32 => "i32",
            AudioFormat::U8 => "u8",
        }
    }
}

impl fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Unique identifier for buffer instances.
///
/// Buffer IDs are used for tracking, correlation with events, and debugging.
/// Each BufferRef gets a unique ID that remains constant for the lifetime
/// of the buffer reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferId(u64);

impl BufferId {
    /// Creates a new unique buffer ID.
    ///
    /// Uses a simple atomic counter for ID generation. In a production system,
    /// this could be enhanced with node ID and timestamp for distributed uniqueness.
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
    
    /// Returns the numeric ID value.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for BufferId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "buf_{}", self.0)
    }
}

/// Buffer lifecycle event types.
///
/// Buffer events allow modules to track buffer state changes without
/// accessing the actual audio data, enabling efficient event-driven
/// buffer management.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferEventType {
    /// Buffer was created and is available for processing
    Created,
    /// Buffer processing started
    ProcessingStarted,
    /// Buffer processing completed successfully
    ProcessingCompleted,
    /// Buffer processing failed
    ProcessingFailed,
    /// Buffer reference was released (may not be final release)
    ReferenceReleased,
    /// All references released, buffer will be cleaned up
    BufferExpired,
}

impl fmt::Display for BufferEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            BufferEventType::Created => "Created",
            BufferEventType::ProcessingStarted => "ProcessingStarted",
            BufferEventType::ProcessingCompleted => "ProcessingCompleted",
            BufferEventType::ProcessingFailed => "ProcessingFailed",
            BufferEventType::ReferenceReleased => "ReferenceReleased",
            BufferEventType::BufferExpired => "BufferExpired",
        };
        write!(f, "{}", name)
    }
}

/// Event for buffer lifecycle notifications.
///
/// BufferEvent contains buffer metadata and state information but no actual
/// audio data, making it efficient to send through the event system for
/// buffer tracking and coordination.
#[derive(Debug, Clone)]  
pub struct BufferEvent {
    /// Buffer identifier for correlation
    pub buffer_id: BufferId,
    /// Type of buffer event
    pub event_type: BufferEventType,
    /// Buffer metadata (no audio data)
    pub metadata: BufferMetadata,
    /// Event timestamp
    pub timestamp: u64,
    /// Event processing priority
    pub priority: EventPriority,
}

impl Event for BufferEvent {
    fn event_type(&self) -> &'static str {
        "BufferEvent"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
    
    fn priority(&self) -> EventPriority {
        self.priority
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn metadata(&self) -> Option<&dyn Any> {
        Some(&self.metadata)
    }
}

impl BufferEvent {
    /// Creates a new buffer event.
    pub fn new(buffer_id: BufferId, event_type: BufferEventType, metadata: BufferMetadata, priority: EventPriority) -> Self {
        Self {
            buffer_id,
            event_type,
            metadata,
            timestamp: get_timestamp_ns(),
            priority,
        }
    }
}

/// Buffer manager for lifecycle coordination and memory management.
///
/// BufferManager provides centralized coordination for buffer creation,
/// tracking, and cleanup. It can implement buffer pooling, memory limits,
/// and performance monitoring for the buffer reference system.
#[derive(Debug)]
pub struct BufferManager {
    /// Active buffer tracking for monitoring
    active_buffers: std::collections::HashMap<BufferId, BufferInfo>,
    /// Buffer creation statistics
    total_buffers_created: u64,
    /// Total memory allocated for buffers
    total_memory_bytes: usize,
    /// Maximum allowed memory usage
    max_memory_bytes: usize,
}

/// Information about an active buffer for tracking
#[derive(Debug, Clone)]
pub struct BufferInfo {
    pub metadata: BufferMetadata,
    pub size_bytes: usize,
    pub created_at: u64,
    pub reference_count: usize,
}

impl BufferManager {
    /// Creates a new buffer manager with specified memory limit.
    ///
    /// # Arguments
    /// * `max_memory_bytes` - Maximum memory that can be allocated for buffers
    pub fn new(max_memory_bytes: usize) -> Self {
        Self {
            active_buffers: std::collections::HashMap::new(),
            total_buffers_created: 0,
            total_memory_bytes: 0,
            max_memory_bytes,
        }
    }
    
    /// Creates a new buffer with memory limit checking.
    ///
    /// # Arguments
    /// * `data` - Audio sample data
    /// * `metadata` - Audio properties and timing information
    ///
    /// # Returns
    /// * `Ok(buffer_ref)` - Buffer created successfully
    /// * `Err(error)` - Creation failed (memory limit, invalid metadata, etc.)
    pub fn create_buffer<T>(&mut self, data: Vec<T>, metadata: BufferMetadata) -> Result<BufferRef<T>, BufferManagerError> {
        // Validate metadata
        metadata.validate().map_err(|e| BufferManagerError::InvalidMetadata(e))?;
        
        // Check memory limits
        let buffer_size = data.len() * std::mem::size_of::<T>();
        if self.total_memory_bytes + buffer_size > self.max_memory_bytes {
            return Err(BufferManagerError::MemoryLimitExceeded);
        }
        
        // Create buffer reference
        let buffer_ref = BufferRef::new(data, metadata.clone());
        let buffer_id = buffer_ref.buffer_id();
        
        // Track the buffer
        let buffer_info = BufferInfo {
            metadata,
            size_bytes: buffer_size,
            created_at: get_timestamp_ns(),
            reference_count: 1,
        };
        
        self.active_buffers.insert(buffer_id, buffer_info);
        self.total_buffers_created += 1;
        self.total_memory_bytes += buffer_size;
        
        Ok(buffer_ref)
    }
    
    /// Updates reference count tracking for a buffer.
    ///
    /// Should be called when buffer references are cloned or dropped
    /// to maintain accurate tracking information.
    pub fn update_reference_count(&mut self, buffer_id: BufferId, new_count: usize) {
        if let Some(info) = self.active_buffers.get_mut(&buffer_id) {
            info.reference_count = new_count;
            
            // Remove from tracking if no references remain
            if new_count == 0 {
                self.total_memory_bytes -= info.size_bytes;
                self.active_buffers.remove(&buffer_id);
            }
        }
    }
    
    /// Returns buffer manager statistics.
    pub fn get_stats(&self) -> BufferManagerStats {
        BufferManagerStats {
            active_buffer_count: self.active_buffers.len(),
            total_buffers_created: self.total_buffers_created,
            total_memory_bytes: self.total_memory_bytes,
            max_memory_bytes: self.max_memory_bytes,
            memory_utilization_percent: (self.total_memory_bytes as f64 / self.max_memory_bytes as f64 * 100.0),
        }
    }
    
    /// Returns information about a specific buffer.
    pub fn get_buffer_info(&self, buffer_id: BufferId) -> Option<&BufferInfo> {
        self.active_buffers.get(&buffer_id)
    }
    
    /// Performs cleanup of expired buffer tracking.
    ///
    /// Should be called periodically to remove tracking information
    /// for buffers that are no longer referenced.
    pub fn cleanup_expired_buffers(&mut self) -> usize {
        let expired_count = self.active_buffers.len();
        self.active_buffers.retain(|_, info| info.reference_count > 0);
        let remaining_count = self.active_buffers.len();
        
        expired_count - remaining_count
    }
}

/// Buffer manager statistics for monitoring
#[derive(Debug, Clone)]
pub struct BufferManagerStats {
    pub active_buffer_count: usize,
    pub total_buffers_created: u64,
    pub total_memory_bytes: usize,
    pub max_memory_bytes: usize,
    pub memory_utilization_percent: f64,
}

/// Errors that can occur during buffer management operations
#[derive(Debug, Clone)]
pub enum BufferManagerError {
    /// Memory limit would be exceeded
    MemoryLimitExceeded,
    /// Buffer metadata is invalid
    InvalidMetadata(String),
    /// Buffer not found in tracking
    BufferNotFound,
    /// Internal error with context
    Internal(String),
}

impl fmt::Display for BufferManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BufferManagerError::MemoryLimitExceeded => write!(f, "Buffer memory limit exceeded"),
            BufferManagerError::InvalidMetadata(msg) => write!(f, "Invalid buffer metadata: {}", msg),
            BufferManagerError::BufferNotFound => write!(f, "Buffer not found in tracking"),
            BufferManagerError::Internal(msg) => write!(f, "Internal buffer manager error: {}", msg),
        }
    }
}

impl std::error::Error for BufferManagerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_ref_creation() {
        let data = vec![1.0f32, 2.0, 3.0, 4.0];
        let metadata = BufferMetadata::new(44100, 1, 4);
        let buffer_ref = BufferRef::new(data.clone(), metadata);
        
        assert_eq!(buffer_ref.data(), &[1.0f32, 2.0, 3.0, 4.0]);
        assert_eq!(buffer_ref.metadata().sample_rate, 44100);
        assert_eq!(buffer_ref.metadata().channels, 1);
        assert_eq!(buffer_ref.metadata().frame_count, 4);
        assert_eq!(buffer_ref.reference_count(), 1);
    }

    #[test]
    fn test_buffer_ref_zero_copy_clone() {
        let data = vec![1.0f32; 1024];
        let metadata = BufferMetadata::new(44100, 1, 1024);
        let buffer_ref1 = BufferRef::new(data, metadata);
        let buffer_ref2 = buffer_ref1.clone();
        
        // Both references should point to same data
        assert_eq!(buffer_ref1.buffer_id(), buffer_ref2.buffer_id());
        assert_eq!(buffer_ref1.reference_count(), 2);
        assert_eq!(buffer_ref2.reference_count(), 2);
        
        // Data should be identical
        assert_eq!(buffer_ref1.data().len(), buffer_ref2.data().len());
        assert_eq!(buffer_ref1.data()[0], buffer_ref2.data()[0]);
    }

    #[test]
    fn test_buffer_metadata_validation() {
        // Valid metadata
        let valid_metadata = BufferMetadata::new(44100, 2, 1024);
        assert!(valid_metadata.validate().is_ok());
        
        // Invalid sample rate
        let mut invalid_metadata = BufferMetadata::new(0, 2, 1024);
        assert!(invalid_metadata.validate().is_err());
        
        // Invalid channel count
        invalid_metadata = BufferMetadata::new(44100, 0, 1024);
        assert!(invalid_metadata.validate().is_err());
        
        // Too many channels
        invalid_metadata = BufferMetadata::new(44100, 64, 1024);
        assert!(invalid_metadata.validate().is_err());
    }

    #[test]
    fn test_buffer_metadata_calculations() {
        let metadata = BufferMetadata::new(44100, 2, 1024);
        
        assert_eq!(metadata.total_samples(), 2048); // 1024 frames * 2 channels
        assert!((metadata.duration_seconds() - 0.023219).abs() < 0.000001); // ~23.2ms
        assert!(metadata.duration_ns > 0);
    }

    #[test]
    fn test_buffer_event_creation() {
        let data = vec![0.5f32; 512];
        let metadata = BufferMetadata::new(48000, 1, 512);
        let buffer_ref = BufferRef::new(data, metadata);
        
        let event = buffer_ref.create_event(BufferEventType::Created, EventPriority::High);
        
        assert_eq!(event.buffer_id, buffer_ref.buffer_id());
        assert_eq!(event.event_type, BufferEventType::Created);
        assert_eq!(event.priority, EventPriority::High);
        assert_eq!(event.event_type(), "BufferEvent");
    }

    #[test]
    fn test_buffer_manager_creation() {
        let mut manager = BufferManager::new(1024 * 1024); // 1MB limit
        
        let data = vec![0.0f32; 256];
        let metadata = BufferMetadata::new(44100, 1, 256);
        
        let result = manager.create_buffer(data, metadata);
        assert!(result.is_ok());
        
        let stats = manager.get_stats();
        assert_eq!(stats.active_buffer_count, 1);
        assert_eq!(stats.total_buffers_created, 1);
        assert!(stats.total_memory_bytes > 0);
    }

    #[test]
    fn test_buffer_manager_memory_limit() {
        let mut manager = BufferManager::new(1000); // Very small limit
        
        let large_data = vec![0.0f32; 1000];
        let metadata = BufferMetadata::new(44100, 1, 1000);
        
        let result = manager.create_buffer(large_data, metadata);
        assert!(matches!(result, Err(BufferManagerError::MemoryLimitExceeded)));
    }

    #[test]
    fn test_audio_format_properties() {
        assert_eq!(AudioFormat::F32.sample_size_bytes(), 4);
        assert_eq!(AudioFormat::I16.sample_size_bytes(), 2);
        assert_eq!(AudioFormat::F32.name(), "f32");
        assert_eq!(AudioFormat::I16.to_string(), "i16");
    }

    #[test]
    fn test_buffer_id_uniqueness() {
        let id1 = BufferId::new();
        let id2 = BufferId::new();
        
        assert_ne!(id1, id2);
        assert_ne!(id1.as_u64(), id2.as_u64());
    }

    #[test] 
    fn test_buffer_size_calculation() {
        let data = vec![1.0f32; 1024];
        let metadata = BufferMetadata::new(44100, 1, 1024);
        let buffer_ref = BufferRef::new(data, metadata);
        
        assert_eq!(buffer_ref.size_bytes(), 1024 * 4); // 1024 samples * 4 bytes per f32
    }
}