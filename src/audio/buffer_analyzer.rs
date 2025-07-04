// Audio buffer analyzer for sequential block processing without overlap
// Supports optional windowing functions (Hamming, Blackman)

use super::buffer::{CircularBuffer, validate_buffer_size};

/// Processing strategy for buffer analysis
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessingStrategy {
    /// Sequential processing without overlap (current implementation)
    Sequential,
    /// Sliding window processing with configurable overlap
    SlidingWindow { overlap_ratio: f32 },
}

/// Result of buffer processing operation
#[derive(Debug, PartialEq)]
pub enum ProcessingResult {
    /// Block was successfully processed
    BlockReady(Vec<f32>),
    /// Insufficient data for processing
    InsufficientData,
    /// Processing completed (no more data)
    Completed,
}

/// Abstract buffer processor trait for different processing strategies
pub trait BufferProcessor {
    /// Process the next available data
    fn process_next(&mut self) -> ProcessingResult;
    
    /// Zero-allocation variant that fills a pre-allocated buffer
    fn process_next_into(&mut self, output: &mut [f32]) -> bool;
    
    /// Check if processor can produce a block
    fn can_process(&self) -> bool;
    
    /// Get the block size this processor produces
    fn block_size(&self) -> usize;
    
    /// Get the processing strategy being used
    fn strategy(&self) -> ProcessingStrategy;
}

/// Supported windowing functions for spectral analysis
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowFunction {
    /// No windowing (raw samples)
    None,
    /// Hamming window
    Hamming,
    /// Blackman window
    Blackman,
}

/// Generate window coefficients for a given size and window function
fn generate_window(size: usize, window_fn: WindowFunction) -> Vec<f32> {
    match window_fn {
        WindowFunction::None => vec![1.0; size],
        WindowFunction::Hamming => {
            // w[n] = 0.54 - 0.46 * cos(2*pi*n/(N-1))
            let n_minus_1 = (size - 1) as f32;
            (0..size)
                .map(|n| {
                    0.54 - 0.46 * ((2.0 * std::f32::consts::PI * n as f32) / n_minus_1).cos()
                })
                .collect()
        }
        WindowFunction::Blackman => {
            // w[n] = 0.42 - 0.5*cos(2*pi*n/(N-1)) + 0.08*cos(4*pi*n/(N-1))
            let n_minus_1 = (size - 1) as f32;
            (0..size)
                .map(|n| {
                    let a = (2.0 * std::f32::consts::PI * n as f32) / n_minus_1;
                    0.42 - 0.5 * a.cos() + 0.08 * (2.0 * a).cos()
                })
                .collect()
        }
    }
}

/// Analyzer that reads sequential blocks from a circular buffer without overlap
/// Applies an optional windowing function to the returned block.
pub struct BufferAnalyzer<'a> {
    buffer: &'a mut CircularBuffer<f32>,
    block_size: usize,
    window_fn: WindowFunction,
    window_coeffs: Vec<f32>,
}

impl<'a> BufferAnalyzer<'a> {
    /// Create a new BufferAnalyzer
    ///
    /// * `buffer` - mutable reference to an existing CircularBuffer
    /// * `block_size` - size of each analysis block (must be multiple of 128)
    /// * `window_fn` - windowing function to apply
    pub fn new(
        buffer: &'a mut CircularBuffer<f32>,
        block_size: usize,
        window_fn: WindowFunction,
    ) -> Result<Self, String> {
        // Ensure block_size is valid and compatible with AudioWorklet chunk size
        validate_buffer_size(block_size)?;
        if block_size > buffer.capacity() {
            return Err("Block size cannot exceed buffer capacity".to_string());
        }

        let window_coeffs = generate_window(block_size, window_fn);

        Ok(BufferAnalyzer {
            buffer,
            block_size,
            window_fn,
            window_coeffs,
        })
    }

    /// Attempt to retrieve the next analysis block.
    ///
    /// Returns `Some<Vec<f32>>` containing `block_size` samples if enough data is available.
    /// Returns `None` if the buffer does not yet contain enough samples.
    pub fn next_block(&mut self) -> Option<Vec<f32>> {
        if self.buffer.len() < self.block_size {
            return None;
        }

        // Read samples from the buffer (consuming them to avoid overlap)
        let mut block = vec![0.0f32; self.block_size];
        let read = self.buffer.read_chunk(&mut block);
        if read != self.block_size {
            // This should not happen due to the earlier length check
            return None;
        }

        // Apply windowing in-place
        match self.window_fn {
            WindowFunction::None => {},
            _ => {
                for (sample, coeff) in block.iter_mut().zip(self.window_coeffs.iter()) {
                    *sample *= *coeff;
                }
            }
        }

        Some(block)
    }

    /// Zero-allocation variant of `next_block`. The caller supplies a mutable
    /// slice (`output`) that will be filled with `block_size` samples. Returns
    /// `true` when the slice has been filled, `false` when insufficient data is
    /// currently available in the underlying buffer.
    ///
    /// This method performs **no heap allocations** during steady-state
    /// processing, satisfying Story 2.2 AC 6 (zero-allocation operations).
    pub fn next_block_into(&mut self, output: &mut [f32]) -> bool {
        if output.len() != self.block_size {
            // Mismatched slice size; treat as programmer error
            panic!("output slice length {} does not match analyzer block_size {}", output.len(), self.block_size);
        }

        if self.buffer.len() < self.block_size {
            return false;
        }

        // Read samples directly into caller-provided slice
        let read = self.buffer.read_chunk(output);
        if read != self.block_size {
            // Should not happen because we pre-checked len()
            return false;
        }

        // Apply windowing coefficients in-place
        match self.window_fn {
            WindowFunction::None => {},
            _ => {
                for (sample, coeff) in output.iter_mut().zip(self.window_coeffs.iter()) {
                    *sample *= *coeff;
                }
            }
        }

        true
    }
}

impl<'a> BufferProcessor for BufferAnalyzer<'a> {
    fn process_next(&mut self) -> ProcessingResult {
        match self.next_block() {
            Some(block) => ProcessingResult::BlockReady(block),
            None => ProcessingResult::InsufficientData,
        }
    }
    
    fn process_next_into(&mut self, output: &mut [f32]) -> bool {
        self.next_block_into(output)
    }
    
    fn can_process(&self) -> bool {
        self.buffer.len() >= self.block_size
    }
    
    fn block_size(&self) -> usize {
        self.block_size
    }
    
    fn strategy(&self) -> ProcessingStrategy {
        ProcessingStrategy::Sequential
    }
}

/// Sliding window buffer processor for overlapping analysis
/// Maintains internal state for sliding window position and overlap management
pub struct SlidingWindowProcessor<'a> {
    buffer: &'a CircularBuffer<f32>,
    block_size: usize,
    overlap_ratio: f32,
    hop_size: usize,
    window_fn: WindowFunction,
    window_coeffs: Vec<f32>,
    current_offset: usize,
    processed_samples: usize,
}

impl<'a> SlidingWindowProcessor<'a> {
    /// Create a new SlidingWindowProcessor
    ///
    /// * `buffer` - reference to an existing CircularBuffer
    /// * `block_size` - size of each analysis block (must be multiple of 128)
    /// * `overlap_ratio` - fraction of overlap between windows (0.0 to 0.75)
    /// * `window_fn` - windowing function to apply
    pub fn new(
        buffer: &'a CircularBuffer<f32>,
        block_size: usize,
        overlap_ratio: f32,
        window_fn: WindowFunction,
    ) -> Result<Self, String> {
        // Validate parameters
        validate_buffer_size(block_size)?;
        if block_size > buffer.capacity() {
            return Err("Block size cannot exceed buffer capacity".to_string());
        }
        if overlap_ratio < 0.0 || overlap_ratio >= 1.0 {
            return Err("Overlap ratio must be between 0.0 and 1.0".to_string());
        }
        
        // Calculate hop size ensuring it's a multiple of 128 for AudioWorklet compatibility
        let ideal_hop_size = (block_size as f32 * (1.0 - overlap_ratio)) as usize;
        let hop_size = if ideal_hop_size >= 128 {
            (ideal_hop_size / 128) * 128
        } else {
            // For small block sizes or high overlap ratios, use smaller hop sizes
            // Find the largest divisor of 128 that's <= ideal_hop_size
            let divisors = [64, 32, 16, 8, 4, 2, 1];
            divisors.iter()
                .find(|&&d| d <= ideal_hop_size)
                .copied()
                .unwrap_or(1)
        };
        
        if hop_size == 0 || hop_size >= block_size {
            return Err("Invalid hop size calculation".to_string());
        }
        
        let window_coeffs = generate_window(block_size, window_fn);
        
        Ok(SlidingWindowProcessor {
            buffer,
            block_size,
            overlap_ratio,
            hop_size,
            window_fn,
            window_coeffs,
            current_offset: 0,
            processed_samples: 0,
        })
    }
    
    /// Reset the processor to start from the beginning
    pub fn reset(&mut self) {
        self.current_offset = 0;
        self.processed_samples = 0;
    }
    
    /// Get the current window position
    pub fn current_offset(&self) -> usize {
        self.current_offset
    }
    
    /// Get the hop size (distance between window starts)
    pub fn hop_size(&self) -> usize {
        self.hop_size
    }
}

impl<'a> BufferProcessor for SlidingWindowProcessor<'a> {
    fn process_next(&mut self) -> ProcessingResult {
        if !self.can_process() {
            return ProcessingResult::InsufficientData;
        }
        
        // Get the windowed block
        let mut block = self.buffer.peek_chunk_vec(self.current_offset, self.block_size);
        
        // Apply windowing function
        match self.window_fn {
            WindowFunction::None => {},
            _ => {
                for (sample, coeff) in block.iter_mut().zip(self.window_coeffs.iter()) {
                    *sample *= *coeff;
                }
            }
        }
        
        // Advance window position
        self.current_offset += self.hop_size;
        self.processed_samples += self.hop_size;
        
        ProcessingResult::BlockReady(block)
    }
    
    fn process_next_into(&mut self, output: &mut [f32]) -> bool {
        if output.len() != self.block_size {
            panic!("output slice length {} does not match processor block_size {}", 
                   output.len(), self.block_size);
        }
        
        if !self.can_process() {
            return false;
        }
        
        // Read samples into output buffer
        let read = self.buffer.peek_chunk(self.current_offset, output);
        if read != self.block_size {
            return false;
        }
        
        // Apply windowing function
        match self.window_fn {
            WindowFunction::None => {},
            _ => {
                for (sample, coeff) in output.iter_mut().zip(self.window_coeffs.iter()) {
                    *sample *= *coeff;
                }
            }
        }
        
        // Advance window position
        self.current_offset += self.hop_size;
        self.processed_samples += self.hop_size;
        
        true
    }
    
    fn can_process(&self) -> bool {
        self.buffer.can_read_window(self.current_offset, self.block_size)
    }
    
    fn block_size(&self) -> usize {
        self.block_size
    }
    
    fn strategy(&self) -> ProcessingStrategy {
        ProcessingStrategy::SlidingWindow { 
            overlap_ratio: self.overlap_ratio 
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::buffer::CircularBuffer;

    #[test]
    fn test_generate_window_sizes() {
        let hamming = generate_window(4, WindowFunction::Hamming);
        assert_eq!(hamming.len(), 4);
        let blackman = generate_window(8, WindowFunction::Blackman);
        assert_eq!(blackman.len(), 8);
    }

    #[test]
    fn test_analyzer_sequential_blocks_no_overlap() {
        let mut circ = CircularBuffer::<f32>::new(512).unwrap();
        // Fill buffer with 256 samples (0.0 .. 255.0)
        for i in 0..256 {
            circ.write(i as f32);
        }

        let mut analyzer = BufferAnalyzer::new(&mut circ, 128, WindowFunction::None).unwrap();

        // First block should contain samples 0..127
        let b1 = analyzer.next_block().unwrap();
        assert_eq!(b1.len(), 128);
        assert_eq!(b1, (0..128).map(|i| i as f32).collect::<Vec<_>>());

        // Second block should contain samples 128..255 (no overlap)
        let b2 = analyzer.next_block().unwrap();
        assert_eq!(b2, (128..256).map(|i| i as f32).collect::<Vec<_>>());

        // No more data available
        assert!(analyzer.next_block().is_none());
    }

    #[test]
    fn test_windowing_application() {
        let mut circ = CircularBuffer::<f32>::new(256).unwrap();
        // Fill buffer with 128 samples of value 1.0
        for _ in 0..128 {
            circ.write(1.0);
        }

        let mut analyzer = BufferAnalyzer::new(&mut circ, 128, WindowFunction::Hamming).unwrap();
        let block = analyzer.next_block().unwrap();
        // Each sample should now equal its window coefficient (since original sample = 1.0)
        let coeffs = generate_window(128, WindowFunction::Hamming);
        assert_eq!(block, coeffs);
    }

    #[test]
    fn test_zero_allocation_next_block_into() {
        let mut circ = CircularBuffer::<f32>::new(512).unwrap();
        // Fill with 256 samples of value 1.0
        for _ in 0..256 {
            circ.write(1.0);
        }

        let mut analyzer = BufferAnalyzer::new(&mut circ, 128, WindowFunction::None).unwrap();
        let mut output = vec![0.0_f32; 128];

        // Call next_block_into; should return true and fill slice without allocating
        let filled = analyzer.next_block_into(&mut output);
        assert!(filled);
        assert_eq!(output, vec![1.0_f32; 128]);

        // Second call should also work
        assert!(analyzer.next_block_into(&mut output));

        // Third call should return false (not enough samples left)
        assert!(!analyzer.next_block_into(&mut output));
    }

    #[test]
    fn test_buffer_processor_trait_for_sequential() {
        let mut circ = CircularBuffer::<f32>::new(512).unwrap();
        for i in 0..256 {
            circ.write(i as f32);
        }

        let mut analyzer = BufferAnalyzer::new(&mut circ, 128, WindowFunction::None).unwrap();
        
        // Test trait methods
        assert_eq!(analyzer.strategy(), ProcessingStrategy::Sequential);
        assert_eq!(analyzer.block_size(), 128);
        assert!(analyzer.can_process());
        
        // Test process_next
        match analyzer.process_next() {
            ProcessingResult::BlockReady(block) => {
                assert_eq!(block.len(), 128);
                assert_eq!(block[0], 0.0);
                assert_eq!(block[127], 127.0);
            }
            _ => panic!("Expected BlockReady result"),
        }
        
        // Test process_next_into
        let mut output = vec![0.0; 128];
        assert!(analyzer.process_next_into(&mut output));
        assert_eq!(output[0], 128.0);
        assert_eq!(output[127], 255.0);
    }

    #[test]
    fn test_sliding_window_processor_creation() {
        let circ = CircularBuffer::<f32>::new(512).unwrap();
        
        // Valid parameters
        let processor = SlidingWindowProcessor::new(&circ, 256, 0.5, WindowFunction::None);
        assert!(processor.is_ok());
        let processor = processor.unwrap();
        assert_eq!(processor.block_size(), 256);
        assert_eq!(processor.hop_size(), 128); // 50% overlap = 128 hop size
        
        // Invalid overlap ratio
        assert!(SlidingWindowProcessor::new(&circ, 256, 1.0, WindowFunction::None).is_err());
        assert!(SlidingWindowProcessor::new(&circ, 256, -0.1, WindowFunction::None).is_err());
        
        // Invalid block size
        assert!(SlidingWindowProcessor::new(&circ, 1000, 0.5, WindowFunction::None).is_err());
    }

    #[test]
    fn test_sliding_window_processor_overlapping_windows() {
        let mut circ = CircularBuffer::<f32>::new(512).unwrap();
        // Fill with sequential values
        for i in 0..400 {
            circ.write(i as f32);
        }
        
        let mut processor = SlidingWindowProcessor::new(&circ, 256, 0.5, WindowFunction::None).unwrap();
        
        // Test trait methods
        assert_eq!(processor.strategy(), ProcessingStrategy::SlidingWindow { overlap_ratio: 0.5 });
        assert_eq!(processor.block_size(), 256);
        assert_eq!(processor.hop_size(), 128);
        assert!(processor.can_process());
        
        // First window: samples 0-255
        match processor.process_next() {
            ProcessingResult::BlockReady(block) => {
                assert_eq!(block.len(), 256);
                assert_eq!(block[0], 0.0);
                assert_eq!(block[255], 255.0);
            }
            _ => panic!("Expected BlockReady result"),
        }
        
        // Second window: samples 128-383 (50% overlap)
        match processor.process_next() {
            ProcessingResult::BlockReady(block) => {
                assert_eq!(block.len(), 256);
                assert_eq!(block[0], 128.0);
                assert_eq!(block[255], 383.0);
            }
            _ => panic!("Expected BlockReady result"),
        }
        
        // Third window: samples 256-511, but we only have up to 399
        assert_eq!(processor.process_next(), ProcessingResult::InsufficientData);
    }

    #[test]
    fn test_sliding_window_processor_zero_allocation() {
        let mut circ = CircularBuffer::<f32>::new(512).unwrap();
        for i in 0..300 {
            circ.write(i as f32);
        }
        
        let mut processor = SlidingWindowProcessor::new(&circ, 128, 0.5, WindowFunction::None).unwrap();
        let mut output = vec![0.0; 128];
        
        // First window
        assert!(processor.process_next_into(&mut output));
        assert_eq!(output[0], 0.0);
        assert_eq!(output[127], 127.0);
        
        // Second window (64 samples forward due to 50% overlap)
        assert!(processor.process_next_into(&mut output));
        assert_eq!(output[0], 64.0);
        assert_eq!(output[127], 191.0);
    }

    #[test]
    fn test_sliding_window_processor_reset() {
        let mut circ = CircularBuffer::<f32>::new(512).unwrap();
        for i in 0..200 {
            circ.write(i as f32);
        }
        
        let mut processor = SlidingWindowProcessor::new(&circ, 128, 0.5, WindowFunction::None).unwrap();
        
        // Process one window
        assert!(processor.can_process());
        let _ = processor.process_next();
        assert_eq!(processor.current_offset(), 64); // hop size for 50% overlap with 128 samples
        
        // Reset and verify
        processor.reset();
        assert_eq!(processor.current_offset(), 0);
        
        // Should be able to process from beginning again
        match processor.process_next() {
            ProcessingResult::BlockReady(block) => {
                assert_eq!(block[0], 0.0);
                assert_eq!(block[127], 127.0);
            }
            _ => panic!("Expected BlockReady result"),
        }
    }

    #[test]
    fn test_audioworklet_compatibility() {
        let circ = CircularBuffer::<f32>::new(512).unwrap();
        
        // Test that hop sizes are multiples of 128 for AudioWorklet compatibility
        let processor = SlidingWindowProcessor::new(&circ, 512, 0.5, WindowFunction::None).unwrap();
        assert_eq!(processor.hop_size(), 256); // 512 * 0.5 = 256 (multiple of 128)
        
        let processor = SlidingWindowProcessor::new(&circ, 384, 0.5, WindowFunction::None).unwrap();
        assert_eq!(processor.hop_size(), 128); // 384 * 0.5 = 192, rounded down to 128
        
        // Test with 75% overlap
        let processor = SlidingWindowProcessor::new(&circ, 512, 0.75, WindowFunction::None).unwrap();
        assert_eq!(processor.hop_size(), 128); // 512 * 0.25 = 128
    }
} 