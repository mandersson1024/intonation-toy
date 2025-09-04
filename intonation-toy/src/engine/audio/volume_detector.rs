use web_sys::{AudioContext, AnalyserNode, AudioNode};
use super::{AudioError, data_types::VolumeAnalysis};
use crate::common::dev_log;
use super::analysis;

/// FFT size for the analyser node (fixed requirement)
const FFT_SIZE: u32 = 128;

/// Volume detector that uses Web Audio API's AnalyserNode for volume detection
/// 
/// This detector provides volume analysis by calculating peak and RMS amplitude
/// from time domain data using getFloatTimeDomainData(),
/// which provides direct amplitude values in the -1.0 to 1.0 range.
#[derive(Clone)]
pub struct VolumeDetector {
    /// The Web Audio API analyser node
    analyser_node: AnalyserNode,
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
        
        // Initialize time domain data buffer with FFT size
        let time_domain_data = vec![0.0f32; FFT_SIZE as usize];
        
        dev_log!("VolumeDetector created with FFT size: {}", FFT_SIZE);
        
        Ok(Self {
            analyser_node,
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
    
    /// Analyzes current audio and returns volume levels
    /// 
    /// Returns VolumeAnalysis with amplitude measurements.
    pub fn analyze(&mut self) -> Result<VolumeAnalysis, AudioError> {
        // Get time domain data for direct amplitude calculation
        self.analyser_node.get_float_time_domain_data(&mut self.time_domain_data);
        
        // Perform the analysis with the filled buffers
        Ok(analysis::analyze(&self.time_domain_data))
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