// Audio buffer management for real-time processing
// Implements circular buffers for efficient audio streaming with zero-allocation operations

use std::collections::VecDeque;

pub const AUDIO_CHUNK_SIZE: usize = 128;                // AudioWorklet fixed chunk size
pub const BUFFER_SIZE: usize = AUDIO_CHUNK_SIZE * 32;   // 4096 - IMPORTANT: Also update BUFFER_SIZE in static/audio-processor.js
pub const STANDARD_SAMPLE_RATE: u32 = 44100;            // Standard consumer audio sample rate (44.1 kHz)

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
    /// Create a new circular buffer with fixed BUFFER_SIZE capacity
    pub fn new() -> Self {
        CircularBuffer {
            buffer: VecDeque::with_capacity(BUFFER_SIZE),
            capacity: BUFFER_SIZE,
            state: BufferState::Empty,
            write_pos: 0,
            read_pos: 0,
            has_wrapped: false,
            overflow_count: 0,
        }
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
        Self::new()
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

