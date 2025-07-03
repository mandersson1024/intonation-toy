// Audio buffer analyzer for sequential block processing without overlap
// Supports optional windowing functions (Hamming, Blackman)

use super::buffer::{CircularBuffer, validate_buffer_size};

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
} 