// Audio buffer management for real-time processing
// Implements circular buffers for efficient audio streaming with zero-allocation operations

use std::collections::VecDeque;

/// Buffer size constants as multiples of 128-sample AudioWorklet chunks
pub const PRODUCTION_BUFFER_SIZE: usize = 4096;  // 32 chunks - sufficient for pitch detection
pub const DEV_BUFFER_SIZE_MIN: usize = 256;      // 2 chunks of 128 samples each  
pub const DEV_BUFFER_SIZE_MAX: usize = 4096;     // 32 chunks - accommodate pitch detection window
pub const DEV_BUFFER_SIZE_DEFAULT: usize = 4096; // Default to max for pitch detection accuracy
pub const AUDIO_CHUNK_SIZE: usize = 128;         // AudioWorklet fixed chunk size

/// Determines the buffer size based on build configuration
pub fn get_buffer_size() -> usize {
    if cfg!(debug_assertions) {
        DEV_BUFFER_SIZE_DEFAULT
    } else {
        PRODUCTION_BUFFER_SIZE
    }
}

/// Validates that buffer size is a multiple of 128
pub fn validate_buffer_size(size: usize) -> Result<(), String> {
    if size % AUDIO_CHUNK_SIZE != 0 {
        return Err(format!("Buffer size {} must be a multiple of {}", size, AUDIO_CHUNK_SIZE));
    }
    
    Ok(())
}

/// Validates buffer size for the buffer creation with recommended limits
pub fn validate_buffer_size_for_creation(size: usize) -> Result<(), String> {
    validate_buffer_size(size)?;
    
    if cfg!(debug_assertions) {
        if size < DEV_BUFFER_SIZE_MIN || size > DEV_BUFFER_SIZE_MAX {
            return Err(format!("Development buffer size {} must be between {} and {}", 
                size, DEV_BUFFER_SIZE_MIN, DEV_BUFFER_SIZE_MAX));
        }
    } else {
        if size != PRODUCTION_BUFFER_SIZE {
            return Err(format!("Production buffer size must be {}", PRODUCTION_BUFFER_SIZE));
        }
    }
    
    Ok(())
}

/// Buffer state tracking for efficient processing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BufferState {
    /// Buffer is empty, ready for initial data
    Empty,
    /// Buffer is being filled with data
    Filling,
    /// Buffer is full and ready for processing
    Full,
    /// Buffer has overflowed, oldest data will be evicted
    Overflow,
    /// Buffer is being processed (read-only state)
    Processing,
}

impl std::fmt::Display for BufferState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BufferState::Empty => write!(f, "Empty"),
            BufferState::Filling => write!(f, "Filling"),
            BufferState::Full => write!(f, "Full"),
            BufferState::Overflow => write!(f, "Overflow"),
            BufferState::Processing => write!(f, "Processing"),
        }
    }
}

/// High-performance circular buffer for real-time audio streaming
/// Provides zero-allocation operations during steady-state processing
pub struct CircularBuffer<T> {
    /// Internal buffer storage using VecDeque for efficient push/pop operations
    buffer: VecDeque<T>,
    /// Maximum capacity of the buffer
    capacity: usize,
    /// Current state of the buffer
    state: BufferState,
    /// Write position for sequential access
    write_pos: usize,
    /// Read position for sequential access
    read_pos: usize,
    /// Track if buffer has been used (for overflow detection)
    has_wrapped: bool,
    /// Count number of times overflow occurred
    overflow_count: usize,
}

impl<T> CircularBuffer<T>
where
    T: Clone + Default,
{
    /// Create a new circular buffer with specified capacity
    /// Capacity must be a multiple of 128 for AudioWorklet compatibility
    pub fn new(capacity: usize) -> Result<Self, String> {
        validate_buffer_size_for_creation(capacity)?;
        
        Ok(CircularBuffer {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
            state: BufferState::Empty,
            write_pos: 0,
            read_pos: 0,
            has_wrapped: false,
            overflow_count: 0,
        })
    }

    /// Create a new circular buffer with default capacity based on build configuration
    pub fn new_default() -> Result<Self, String> {
        Self::new(get_buffer_size())
    }

    /// Get the current buffer state
    pub fn state(&self) -> BufferState {
        self.state
    }

    /// Get the current number of elements in the buffer
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Check if the buffer is full
    pub fn is_full(&self) -> bool {
        self.buffer.len() >= self.capacity
    }

    /// Get the buffer capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the number of available slots in the buffer
    pub fn available(&self) -> usize {
        self.capacity - self.buffer.len()
    }

    /// Write a single sample to the buffer
    /// Returns true if successful, false if buffer is full
    pub fn write(&mut self, sample: T) -> bool {
        if self.is_full() {
            // Handle overflow by evicting oldest data
            self.buffer.pop_front();
            self.has_wrapped = true;
            self.state = BufferState::Overflow;
            self.overflow_count += 1;
        }

        self.buffer.push_back(sample);
        self.write_pos = (self.write_pos + 1) % self.capacity;
        
        self.update_state();
        true
    }

    /// Write multiple samples to the buffer
    /// Returns the number of samples actually written
    pub fn write_chunk(&mut self, samples: &[T]) -> usize {
        let mut written = 0;
        
        for sample in samples {
            if self.write(sample.clone()) {
                written += 1;
            } else {
                break;
            }
        }
        
        written
    }

    /// Read a single sample from the buffer
    /// Returns None if buffer is empty
    pub fn read(&mut self) -> Option<T> {
        if let Some(sample) = self.buffer.pop_front() {
            self.read_pos = (self.read_pos + 1) % self.capacity;
            self.update_state();
            Some(sample)
        } else {
            None
        }
    }

    /// Read multiple samples from the buffer
    /// Returns the actual number of samples read
    pub fn read_chunk(&mut self, output: &mut [T]) -> usize {
        let mut read_count = 0;
        
        for i in 0..output.len() {
            if let Some(sample) = self.read() {
                output[i] = sample;
                read_count += 1;
            } else {
                break;
            }
        }
        
        read_count
    }

    /// Peek at samples without removing them from the buffer
    /// Returns a slice of available samples up to the requested count
    pub fn peek(&self, count: usize) -> Vec<T> {
        let available = std::cmp::min(count, self.buffer.len());
        let mut result = Vec::with_capacity(available);
        
        for i in 0..available {
            if let Some(sample) = self.buffer.get(i) {
                result.push(sample.clone());
            }
        }
        
        result
    }

    /// Get a sequential block of samples for analysis
    /// Returns None if not enough samples are available
    pub fn get_sequential_block(&self, size: usize) -> Option<Vec<T>> {
        if self.buffer.len() < size {
            return None;
        }
        
        let mut block = Vec::with_capacity(size);
        for i in 0..size {
            if let Some(sample) = self.buffer.get(i) {
                block.push(sample.clone());
            }
        }
        
        Some(block)
    }

    /// Non-destructive read of multiple samples starting at offset
    /// Returns the actual number of samples read into output buffer
    /// Preserves data in the circular buffer for sliding window processing
    pub fn peek_chunk(&self, offset: usize, output: &mut [T]) -> usize {
        let mut read_count = 0;
        
        for i in 0..output.len() {
            if let Some(sample) = self.buffer.get(offset + i) {
                output[i] = sample.clone();
                read_count += 1;
            } else {
                break;
            }
        }
        
        read_count
    }

    /// Non-destructive read of multiple samples starting at offset
    /// Returns a Vec containing the requested samples
    /// More convenient than peek_chunk when you don't have a pre-allocated buffer
    pub fn peek_chunk_vec(&self, offset: usize, count: usize) -> Vec<T> {
        let available = std::cmp::min(count, self.buffer.len().saturating_sub(offset));
        let mut result = Vec::with_capacity(available);
        
        for i in offset..offset + available {
            if let Some(sample) = self.buffer.get(i) {
                result.push(sample.clone());
            }
        }
        
        result
    }

    /// Check if enough samples are available for a sliding window operation
    /// Returns true if we can read 'window_size' samples starting at 'offset'
    pub fn can_read_window(&self, offset: usize, window_size: usize) -> bool {
        self.buffer.len() >= offset + window_size
    }

    /// Clear the buffer and reset state
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.state = BufferState::Empty;
        self.write_pos = 0;
        self.read_pos = 0;
        self.has_wrapped = false;
        self.overflow_count = 0;
    }

    /// Check if the buffer has experienced overflow
    pub fn has_overflowed(&self) -> bool {
        self.has_wrapped || self.state == BufferState::Overflow
    }

    /// Return total number of overflows experienced since creation/reset
    pub fn overflow_count(&self) -> usize {
        self.overflow_count
    }

    /// Reset overflow indicators and counter (recovery)
    pub fn reset_overflow(&mut self) {
        self.has_wrapped = false;
        self.overflow_count = 0;
        self.update_state();
    }

    /// Update buffer state based on current conditions
    fn update_state(&mut self) {
        if self.has_wrapped {
            self.state = BufferState::Overflow;
        } else if self.buffer.is_empty() {
            self.state = BufferState::Empty;
        } else if self.is_full() {
            self.state = BufferState::Full;
        } else {
            self.state = BufferState::Filling;
        }
    }
}

impl<T> Default for CircularBuffer<T>
where
    T: Clone + Default,
{
    fn default() -> Self {
        Self::new_default().expect("Default buffer size should be valid")
    }
}

// Implement Debug for CircularBuffer
impl<T> std::fmt::Debug for CircularBuffer<T>
where
    T: Clone + Default,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CircularBuffer")
            .field("capacity", &self.capacity)
            .field("len", &self.len())
            .field("state", &self.state)
            .field("write_pos", &self.write_pos)
            .field("read_pos", &self.read_pos)
            .field("has_wrapped", &self.has_wrapped)
            .field("overflow_count", &self.overflow_count)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_buffer_size_constants() {
        assert_eq!(PRODUCTION_BUFFER_SIZE, 4096);
        assert_eq!(DEV_BUFFER_SIZE_MIN, 256);
        assert_eq!(DEV_BUFFER_SIZE_MAX, 4096);
        assert_eq!(DEV_BUFFER_SIZE_DEFAULT, 4096);
        assert_eq!(AUDIO_CHUNK_SIZE, 128);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_buffer_size_validation() {
        // Valid sizes (multiples of 128)
        assert!(validate_buffer_size(128).is_ok());
        assert!(validate_buffer_size(256).is_ok());
        assert!(validate_buffer_size(1024).is_ok());
        
        // Invalid sizes (not multiples of 128)
        assert!(validate_buffer_size(100).is_err());
        assert!(validate_buffer_size(200).is_err());
        assert!(validate_buffer_size(1000).is_err());
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_buffer_size_validation_for_creation() {
        // Valid sizes for creation in debug mode
        assert!(validate_buffer_size_for_creation(256).is_ok());
        assert!(validate_buffer_size_for_creation(512).is_ok());
        assert!(validate_buffer_size_for_creation(1024).is_ok());
        
        // Invalid sizes for creation in debug mode (too small or too large)
        if cfg!(debug_assertions) {
            assert!(validate_buffer_size_for_creation(128).is_err()); // Too small (< 256)
            assert!(validate_buffer_size_for_creation(8192).is_err()); // Too large (> 4096)
        }
        
        // Invalid sizes (not multiples of 128)
        assert!(validate_buffer_size_for_creation(100).is_err());
        assert!(validate_buffer_size_for_creation(200).is_err());
        assert!(validate_buffer_size_for_creation(1000).is_err());
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_get_buffer_size() {
        let size = get_buffer_size();
        assert!(validate_buffer_size(size).is_ok());
        
        if cfg!(debug_assertions) {
            assert_eq!(size, DEV_BUFFER_SIZE_DEFAULT);
        } else {
            assert_eq!(size, PRODUCTION_BUFFER_SIZE);
        }
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_buffer_state_display() {
        assert_eq!(BufferState::Empty.to_string(), "Empty");
        assert_eq!(BufferState::Filling.to_string(), "Filling");
        assert_eq!(BufferState::Full.to_string(), "Full");
        assert_eq!(BufferState::Overflow.to_string(), "Overflow");
        assert_eq!(BufferState::Processing.to_string(), "Processing");
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_circular_buffer_creation() {
        let buffer = CircularBuffer::<f32>::new(256).unwrap();
        assert_eq!(buffer.capacity(), 256);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());
        assert_eq!(buffer.state(), BufferState::Empty);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_circular_buffer_invalid_size() {
        let result = CircularBuffer::<f32>::new(100);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be a multiple of 128"));
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_circular_buffer_write_read() {
        let mut buffer = CircularBuffer::<f32>::new(256).unwrap();
        
        // Write some samples
        assert!(buffer.write(1.0));
        assert!(buffer.write(2.0));
        assert!(buffer.write(3.0));
        
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.state(), BufferState::Filling);
        
        // Read samples back
        assert_eq!(buffer.read(), Some(1.0));
        assert_eq!(buffer.read(), Some(2.0));
        assert_eq!(buffer.read(), Some(3.0));
        assert_eq!(buffer.read(), None);
        
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.state(), BufferState::Empty);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_circular_buffer_overflow() {
        let mut buffer = CircularBuffer::<f32>::new(256).unwrap();
        
        // Fill buffer to capacity
        for i in 0..256 {
            assert!(buffer.write(i as f32));
        }
        
        assert!(buffer.is_full());
        assert_eq!(buffer.state(), BufferState::Full);
        
        // Write one more sample to trigger overflow
        assert!(buffer.write(256.0));
        assert_eq!(buffer.state(), BufferState::Overflow);
        assert!(buffer.has_overflowed());
        
        // Should still have 256 samples, but first sample should be evicted
        assert_eq!(buffer.len(), 256);
        assert_eq!(buffer.read(), Some(1.0)); // First sample (0.0) was evicted
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_circular_buffer_chunk_operations() {
        let mut buffer = CircularBuffer::<f32>::new(256).unwrap();
        
        // Write a chunk
        let input = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let written = buffer.write_chunk(&input);
        assert_eq!(written, 5);
        assert_eq!(buffer.len(), 5);
        
        // Read a chunk
        let mut output = vec![0.0; 3];
        let read = buffer.read_chunk(&mut output);
        assert_eq!(read, 3);
        assert_eq!(output, vec![1.0, 2.0, 3.0]);
        assert_eq!(buffer.len(), 2);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_circular_buffer_peek() {
        let mut buffer = CircularBuffer::<f32>::new(256).unwrap();
        
        // Write some samples
        buffer.write(1.0);
        buffer.write(2.0);
        buffer.write(3.0);
        
        // Peek at samples without removing them
        let peeked = buffer.peek(2);
        assert_eq!(peeked, vec![1.0, 2.0]);
        assert_eq!(buffer.len(), 3); // Length unchanged
        
        // Peek at more samples than available
        let peeked = buffer.peek(5);
        assert_eq!(peeked, vec![1.0, 2.0, 3.0]);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_circular_buffer_sequential_block() {
        let mut buffer = CircularBuffer::<f32>::new(256).unwrap();
        
        // Write some samples
        for i in 0..10 {
            buffer.write(i as f32);
        }
        
        // Get a sequential block
        let block = buffer.get_sequential_block(5);
        assert_eq!(block, Some(vec![0.0, 1.0, 2.0, 3.0, 4.0]));
        
        // Try to get more samples than available
        let block = buffer.get_sequential_block(15);
        assert_eq!(block, None);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_circular_buffer_clear() {
        let mut buffer = CircularBuffer::<f32>::new(256).unwrap();
        
        // Write some samples
        buffer.write(1.0);
        buffer.write(2.0);
        buffer.write(3.0);
        
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.state(), BufferState::Filling);
        
        // Clear the buffer
        buffer.clear();
        
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.state(), BufferState::Empty);
        assert!(!buffer.has_overflowed());
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_circular_buffer_default() {
        let buffer = CircularBuffer::<f32>::default();
        assert_eq!(buffer.capacity(), get_buffer_size());
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.state(), BufferState::Empty);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_circular_buffer_overflow_count_and_reset() {
        let mut buffer = CircularBuffer::<f32>::new(256).unwrap();

        // Trigger two overflows
        for _ in 0..(256 + 10) {
            buffer.write(1.0);
        }
        for _ in 0..(10) {
            buffer.write(2.0);
        }

        assert!(buffer.has_overflowed());
        assert!(buffer.overflow_count() >= 2);

        // Reset overflow status
        buffer.reset_overflow();
        assert!(!buffer.has_overflowed());
        assert_eq!(buffer.overflow_count(), 0);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_circular_buffer_peek_chunk() {
        let mut buffer = CircularBuffer::<f32>::new(256).unwrap();
        
        // Write some samples
        for i in 0..10 {
            buffer.write(i as f32);
        }
        
        // Peek at samples without removing them
        let mut output = vec![0.0; 5];
        let read = buffer.peek_chunk(2, &mut output);
        assert_eq!(read, 5);
        assert_eq!(output, vec![2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(buffer.len(), 10); // Length unchanged
        
        // Peek beyond available samples
        let mut output = vec![0.0; 5];
        let read = buffer.peek_chunk(8, &mut output);
        assert_eq!(read, 2);
        assert_eq!(output[0..2], vec![8.0, 9.0]);
        
        // Peek at offset beyond buffer
        let mut output = vec![0.0; 5];
        let read = buffer.peek_chunk(20, &mut output);
        assert_eq!(read, 0);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_circular_buffer_peek_chunk_vec() {
        let mut buffer = CircularBuffer::<f32>::new(256).unwrap();
        
        // Write some samples
        for i in 0..10 {
            buffer.write(i as f32);
        }
        
        // Peek at samples without removing them
        let peeked = buffer.peek_chunk_vec(3, 4);
        assert_eq!(peeked, vec![3.0, 4.0, 5.0, 6.0]);
        assert_eq!(buffer.len(), 10); // Length unchanged
        
        // Peek beyond available samples
        let peeked = buffer.peek_chunk_vec(8, 5);
        assert_eq!(peeked, vec![8.0, 9.0]);
        
        // Peek at offset beyond buffer
        let peeked = buffer.peek_chunk_vec(20, 5);
        assert_eq!(peeked, Vec::<f32>::new());
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_circular_buffer_can_read_window() {
        let mut buffer = CircularBuffer::<f32>::new(256).unwrap();
        
        // Write some samples
        for i in 0..10 {
            buffer.write(i as f32);
        }
        
        // Check valid window reads
        assert!(buffer.can_read_window(0, 5));
        assert!(buffer.can_read_window(5, 5));
        assert!(buffer.can_read_window(0, 10));
        
        // Check invalid window reads
        assert!(!buffer.can_read_window(0, 11));
        assert!(!buffer.can_read_window(5, 6));
        assert!(!buffer.can_read_window(10, 1));
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_sliding_window_simulation() {
        let mut buffer = CircularBuffer::<f32>::new(1024).unwrap();
        
        // Write 800 samples (less than window size to fit in buffer)
        for i in 0..800 {
            buffer.write(i as f32);
        }
        
        // Simulate sliding window with 50% overlap
        let window_size = 512;
        let hop_size = 256;
        
        // First window at offset 0
        assert!(buffer.can_read_window(0, window_size));
        let window1 = buffer.peek_chunk_vec(0, window_size);
        assert_eq!(window1.len(), window_size);
        assert_eq!(window1[0], 0.0);
        assert_eq!(window1[511], 511.0);
        
        // Second window at offset 256 (50% overlap)
        assert!(buffer.can_read_window(hop_size, window_size));
        let window2 = buffer.peek_chunk_vec(hop_size, window_size);
        assert_eq!(window2.len(), window_size);
        assert_eq!(window2[0], 256.0);
        assert_eq!(window2[511], 767.0);
        
        // Verify overlap: last 256 samples of window1 == first 256 samples of window2
        assert_eq!(window1[256..512], window2[0..256]);
    }
}