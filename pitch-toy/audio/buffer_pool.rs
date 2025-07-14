//! Audio Buffer Pool
//!
//! Provides a fixed collection of pre-allocated `CircularBuffer`s so that the
//! real-time audio pipeline can reuse memory and remain allocation-free once
//! running.  Each buffer is created up-front and lives for the lifetime of the
//! pool, completely avoiding expensive grow/shrink operations during audio
//! processing.
//!
//! The pool also keeps track of total memory usage and refuses to create if the
//! requested capacity would exceed the hard 50 MB GPU/Audio memory budget
//! defined by the architecture.
//!
//! # Example
//! ```rust
//! use pitch_toy::audio::{BufferPool, CircularBuffer};
//!
//! // Create a pool containing eight `CircularBuffer<f32>` instances, each able
//! // to store 1 024 samples (8 × 1 024 × 4 B ≈ 32 KB total).
//! let mut pool = BufferPool::<f32>::new(8, 1024).expect("failed to allocate buffer pool");
//!
//! // Write a 128-sample chunk to the first buffer.
//! if let Some(first) = pool.get_mut(0) {
//!     let samples = vec![0.0_f32; 128];
//!     first.write_chunk(&samples);
//! }
//!
//! // Read statistics.
//! println!("Total pool memory usage: {} bytes", pool.memory_usage_bytes());
//! println!("Total overflows so far: {}", pool.total_overflows());
//! ```
//!
//! For additional details see Story 2.2 / Task 3 in `docs/stories/2.2.story.md`.

use super::buffer::{validate_buffer_size_for_creation, CircularBuffer};
use std::mem::size_of;

/// Maximum GPU/Audio buffer memory usage allowed (50 MB)
pub const MAX_GPU_MEMORY_BYTES: usize = 50 * 1024 * 1024;

/// Pre-allocated pool of CircularBuffers for zero-allocation reuse
pub struct BufferPool<T> {
    buffers: Vec<CircularBuffer<T>>, // owned buffers
    capacity: usize,
}

impl<T> BufferPool<T>
where
    T: Clone + Default,
{
    /// Create a new buffer pool with `pool_size` buffers, each with `capacity` samples.
    pub fn new(pool_size: usize, capacity: usize) -> Result<Self, String> {
        if pool_size == 0 {
            return Err("Pool size must be > 0".to_string());
        }
        validate_buffer_size_for_creation(capacity)?;

        // Estimate memory usage and enforce limit
        let bytes_needed = pool_size * capacity * size_of::<T>();
        if bytes_needed > MAX_GPU_MEMORY_BYTES {
            return Err(format!(
                "Requested pool ({:.2} MB) exceeds memory limit ({:.2} MB)",
                bytes_needed as f64 / 1_048_576.0,
                MAX_GPU_MEMORY_BYTES as f64 / 1_048_576.0
            ));
        }

        let mut buffers = Vec::with_capacity(pool_size);
        for _ in 0..pool_size {
            // Safe unwrap: validation done above
            buffers.push(CircularBuffer::<T>::new(capacity).unwrap());
        }

        Ok(BufferPool { buffers, capacity })
    }

    /// Get immutable reference to buffer by index
    pub fn get(&self, idx: usize) -> Option<&CircularBuffer<T>> {
        self.buffers.get(idx)
    }

    /// Get mutable reference to buffer by index
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut CircularBuffer<T>> {
        self.buffers.get_mut(idx)
    }

    /// Number of buffers in the pool
    pub fn len(&self) -> usize {
        self.buffers.len()
    }

    /// Capacity of each buffer
    pub fn buffer_capacity(&self) -> usize {
        self.capacity
    }

    /// Total memory usage in bytes (host memory / WASM linear memory)
    pub fn memory_usage_bytes(&self) -> usize {
        self.buffers.len() * self.capacity * size_of::<T>()
    }

    /// Check if pool stays within memory limit
    pub fn within_memory_limit(&self) -> bool {
        self.memory_usage_bytes() <= MAX_GPU_MEMORY_BYTES
    }

    /// Aggregate overflow count across all buffers
    pub fn total_overflows(&self) -> usize {
        self.buffers.iter().map(|b| b.overflow_count()).sum()
    }

    /// Clear all buffers and reset overflow indicators
    pub fn reset_all(&mut self) {
        for buf in &mut self.buffers {
            buf.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_buffer_pool_creation_and_metrics() {
        let pool = BufferPool::<f32>::new(4, 256).unwrap();
        assert_eq!(pool.len(), 4);
        assert_eq!(pool.buffer_capacity(), 256);
        assert!(pool.within_memory_limit());
        // Expected bytes: 4 * 256 * 4 = 4096 bytes
        assert_eq!(pool.memory_usage_bytes(), 4 * 256 * 4);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_buffer_pool_overflow_tracking() {
        let mut pool = BufferPool::<f32>::new(2, 256).unwrap();
        // Trigger overflow in first buffer
        if let Some(buf) = pool.get_mut(0) {
            for i in 0..300 {
                buf.write(i as f32);
            }
        }
        assert!(pool.total_overflows() > 0);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_buffer_pool_memory_limit_enforcement() {
        // This should fail: 1000 buffers * 2048 samples * 4 bytes ≈ 7.8 MB (< 50MB) so pass
        assert!(BufferPool::<f32>::new(1000, 2048).is_ok());

        // excessive: 10000 * 2048 * 4 ≈ 78 MB > 50 MB
        assert!(BufferPool::<f32>::new(10_000, 2048).is_err());
    }
} 