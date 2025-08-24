use web_sys::{AudioContext, AnalyserNode, AudioNode};
use super::{AudioError, VolumeAnalysis};
use crate::common::dev_log;

/// FFT size for the analyser node (fixed requirement)
const FFT_SIZE: u32 = 128;

/// Number of frequency bins (half of FFT size)
const FREQUENCY_BIN_COUNT: usize = (FFT_SIZE / 2) as usize;

/// Volume detector that uses Web Audio API's AnalyserNode for FFT-based volume detection
pub struct AnalyserVolumeDetector {
    /// The Web Audio API analyser node
    analyser_node: AnalyserNode,
    /// Pre-allocated buffer for frequency data to avoid reallocations
    frequency_data: Vec<u8>,
}

impl AnalyserVolumeDetector {
    /// Creates a new AnalyserVolumeDetector with configured AnalyserNode
    pub fn new(audio_context: &AudioContext) -> Result<Self, AudioError> {
        // Create analyser node
        let analyser_node = audio_context
            .create_analyser()
            .map_err(|e| AudioError::NotSupported(format!("Failed to create analyser node: {:?}", e)))?;
        
        // Set FFT size to 128
        analyser_node.set_fft_size(FFT_SIZE);
        
        // Set smoothing time constant to 0.0 for real-time analysis
        analyser_node.set_smoothing_time_constant(0.0);
        
        // Initialize frequency data buffer with correct size
        let frequency_data = vec![0u8; FREQUENCY_BIN_COUNT];
        
        dev_log!("AnalyserVolumeDetector created with FFT size: {}", FFT_SIZE);
        
        Ok(Self {
            analyser_node,
            frequency_data,
        })
    }
    
    /// Connects an audio source to the analyser node
    pub fn connect_source(&self, source: &AudioNode) -> Result<(), AudioError> {
        source
            .connect_with_audio_node(&self.analyser_node)
            .map_err(|e| AudioError::Generic(format!("Failed to connect source to analyser: {:?}", e)))?;
        
        dev_log!("Audio source connected to AnalyserVolumeDetector");
        Ok(())
    }
    
    /// Analyzes current audio and returns volume levels
    pub fn analyze(&mut self) -> Result<VolumeAnalysis, AudioError> {
        // Verify FFT size hasn't changed - this should never happen during normal operation
        let expected = self.analyser_node.frequency_bin_count() as usize;
        if self.frequency_data.len() != expected {
            panic!(
                "FFT size changed unexpectedly! Buffer size: {}, Expected: {}. \
                This indicates external modification of the AnalyserNode's fft_size.",
                self.frequency_data.len(),
                expected
            );
        }
        
        // Get frequency data from analyser node directly into our buffer
        self.analyser_node.get_byte_frequency_data(&mut self.frequency_data);
        
        // Calculate peak amplitude from frequency bins
        let peak_amplitude = self.calculate_peak_from_bins();
        
        // RMS is not needed for this implementation
        let rms_amplitude = 0.0;
        
        Ok(VolumeAnalysis {
            rms_amplitude,
            peak_amplitude,
        })
    }
    
    /// Calculates peak amplitude from frequency bin data
    fn calculate_peak_from_bins(&self) -> f32 {
        // Find maximum value in frequency bins
        let max_value = self.frequency_data
            .iter()
            .max()
            .copied()
            .unwrap_or(0);
        
        // Normalize to 0.0-1.0 range
        max_value as f32 / 255.0
    }
    
    /// Returns the underlying AnalyserNode for direct access if needed
    /// 
    /// WARNING: Do not modify the fft_size of the returned AnalyserNode.
    /// Changing the FFT size will cause a panic in the analyze() method.
    pub fn node(&self) -> &AnalyserNode {
        &self.analyser_node
    }
}

impl Drop for AnalyserVolumeDetector {
    fn drop(&mut self) {
        dev_log!("AnalyserVolumeDetector dropped");
    }
}