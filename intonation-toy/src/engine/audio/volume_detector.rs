use web_sys::{AudioContext, AnalyserNode, AudioNode};
use super::{AudioError, data_types::VolumeAnalysis};
use crate::common::dev_log;

/// FFT size for the analyser node (fixed requirement)
const FFT_SIZE: u32 = 128;

/// Number of frequency bins (half of FFT size)
const FREQUENCY_BIN_COUNT: usize = (FFT_SIZE / 2) as usize;

/// Volume detector that uses Web Audio API's AnalyserNode for FFT-based volume detection
/// 
/// This detector provides both volume analysis and FFT frequency data.
/// The FFT data contains 64 frequency bins normalized to 0.0-1.0 range.
/// Bin 0 represents DC component, higher indices represent higher frequencies.
#[derive(Clone)]
pub struct VolumeDetector {
    /// The Web Audio API analyser node
    analyser_node: AnalyserNode,
    /// Pre-allocated buffer for frequency data to avoid reallocations
    frequency_data: Vec<u8>,
}

impl VolumeDetector {
    /// Creates a new VolumeDetector with configured AnalyserNode
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
        
        dev_log!("VolumeDetector created with FFT size: {}", FFT_SIZE);
        
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
        
        dev_log!("Audio source connected to VolumeDetector");
        Ok(())
    }
    
    /// Analyzes current audio and returns volume levels with FFT data
    /// 
    /// Returns VolumeAnalysis with populated fft_data field containing
    /// normalized frequency bin magnitudes (0.0-1.0 range).
    pub fn analyze(&mut self) -> Result<VolumeAnalysis, AudioError> {
        // Verify FFT size hasn't changed - this should never happen during normal operation
        let expected = self.analyser_node.frequency_bin_count() as usize;
        if self.frequency_data.len() != expected {
            return Err(AudioError::Generic(format!(
                "FFT size changed unexpectedly! Buffer size: {}, Expected: {}",
                self.frequency_data.len(),
                expected
            )));
        }
        
        // Get frequency data from analyser node directly into our buffer
        self.analyser_node.get_byte_frequency_data(&mut self.frequency_data);
        
        // Calculate peak amplitude from frequency bins
        let peak_amplitude = self.calculate_peak_from_bins();
        
        // RMS is not needed for this implementation
        let rms_amplitude = 0.0;
        
        // Convert byte frequency data to normalized f32 values (0.0-1.0)
        let fft_data: Vec<f32> = self.frequency_data
            .iter()
            .map(|&byte| byte as f32 / 255.0)
            .collect();
        
        Ok(VolumeAnalysis {
            rms_amplitude,
            peak_amplitude,
            fft_data: Some(fft_data),
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
    
    /// Gets FFT data directly without calculating volume metrics
    /// 
    /// Updates the internal frequency buffer and returns normalized FFT data.
    /// Useful for cases where only FFT data is needed without volume calculation.
    /// Returns 64 frequency bins normalized to 0.0-1.0 range.
    pub fn get_fft_data(&mut self) -> Result<Vec<f32>, AudioError> {
        // Verify FFT size hasn't changed
        let expected = self.analyser_node.frequency_bin_count() as usize;
        if self.frequency_data.len() != expected {
            return Err(AudioError::Generic(format!(
                "FFT size changed unexpectedly! Buffer size: {}, Expected: {}",
                self.frequency_data.len(),
                expected
            )));
        }
        
        // Get frequency data from analyser node
        self.analyser_node.get_byte_frequency_data(&mut self.frequency_data);
        
        // Convert to normalized f32 values
        Ok(self.frequency_data
            .iter()
            .map(|&byte| byte as f32 / 255.0)
            .collect())
    }
    
    /// Disconnects the analyser node from all connected inputs
    pub fn disconnect(&self) -> Result<(), AudioError> {
        let _ = self.analyser_node.disconnect();
        dev_log!("VolumeDetector disconnected from audio sources");
        Ok(())
    }
}

impl Drop for VolumeDetector {
    fn drop(&mut self) {
        dev_log!("VolumeDetector dropped");
    }
}