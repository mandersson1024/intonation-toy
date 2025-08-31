use web_sys::{AudioContext, AnalyserNode, AudioNode};
use super::{AudioError, data_types::VolumeAnalysis};
use crate::common::dev_log;

/// FFT size for the analyser node (fixed requirement)
const FFT_SIZE: u32 = 128;

/// Number of frequency bins (half of FFT size)
const FREQUENCY_BIN_COUNT: usize = (FFT_SIZE / 2) as usize;

/// Volume detector that uses Web Audio API's AnalyserNode for volume detection
/// 
/// This detector provides both volume analysis and FFT frequency data.
/// Peak amplitude is calculated directly from time domain data using getFloatTimeDomainData(),
/// which provides direct amplitude values in the -1.0 to 1.0 range.
/// The FFT data contains 64 frequency bins normalized to 0.0-1.0 range.
/// Bin 0 represents DC component, higher indices represent higher frequencies.
#[derive(Clone)]
pub struct VolumeDetector {
    /// The Web Audio API analyser node
    analyser_node: AnalyserNode,
    /// Pre-allocated buffer for frequency data to avoid reallocations
    frequency_data: Vec<u8>,
    /// Pre-allocated buffer for time domain data to avoid reallocations
    time_domain_data: Vec<f32>,
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
        // Initialize time domain data buffer with FFT size
        let time_domain_data = vec![0.0f32; FFT_SIZE as usize];
        
        dev_log!("VolumeDetector created with FFT size: {}", FFT_SIZE);
        
        Ok(Self {
            analyser_node,
            frequency_data,
            time_domain_data,
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
        
        // Get time domain data for direct amplitude calculation
        self.analyser_node.get_float_time_domain_data(&mut self.time_domain_data);
        
        // Calculate peak amplitude from time domain data
        let peak_amplitude = self.get_peak_amplitude_from_time_domain();
        
        // Calculate RMS amplitude from time domain data
        let rms_amplitude = self.calculate_rms_amplitude_from_time_domain();
        
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
    
    /// Calculates peak amplitude directly from time domain data
    /// 
    /// Uses getFloatTimeDomainData() which provides direct amplitude values
    /// in the range -1.0 to 1.0. Returns the absolute maximum value.
    fn get_peak_amplitude_from_time_domain(&self) -> f32 {
        // Find the maximum absolute value in time domain data
        self.time_domain_data
            .iter()
            .map(|&sample| sample.abs())
            .fold(0.0f32, f32::max)
    }
    
    /// Calculates RMS amplitude from time domain data
    /// 
    /// Root Mean Square provides the effective amplitude over the time window.
    fn calculate_rms_amplitude_from_time_domain(&self) -> f32 {
        // Calculate sum of squares
        let sum_of_squares: f32 = self.time_domain_data
            .iter()
            .map(|&sample| sample * sample)
            .sum();
        
        // Calculate RMS: square root of mean of squares
        (sum_of_squares / self.time_domain_data.len() as f32).sqrt()
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