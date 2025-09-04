use web_sys::{AnalyserNode};
use super::{AudioError, data_types::VolumeAnalysis};
use crate::common::dev_log;
use super::analysis;

/// FFT size for the analyser node (fixed requirement)
const FFT_SIZE: u32 = 128;

/// Volume detector that uses AudioPipeline's AnalyserNode for volume detection
/// 
/// This detector provides volume analysis by calculating peak and RMS amplitude
/// from time domain data using getFloatTimeDomainData(),
/// which provides direct amplitude values in the -1.0 to 1.0 range.
pub struct VolumeDetector {
    /// Reference to the AudioPipeline's analyser node
    analyser_node: AnalyserNode,
    /// Pre-allocated buffer for time domain data to avoid reallocations
    time_domain_data: Vec<f32>,
}

impl VolumeDetector {
    /// Creates a new VolumeDetector using the provided AnalyserNode
    pub fn new(analyser_node: AnalyserNode) -> Self {
        // Initialize time domain data buffer with FFT size
        let time_domain_data = vec![0.0f32; FFT_SIZE as usize];
        
        dev_log!("VolumeDetector created using shared analyser node");
        
        Self {
            analyser_node,
            time_domain_data,
        }
    }
    
    /// Analyzes current audio and returns volume levels
    /// 
    /// Returns VolumeAnalysis with amplitude measurements.
    pub fn analyze(&mut self) -> Result<VolumeAnalysis, AudioError> {
        // Get time domain data for direct amplitude calculation
        self.analyser_node.get_float_time_domain_data(&mut self.time_domain_data);
        
        // Perform the analysis with the filled buffers
        Ok(analysis::analyze_volume(&self.time_domain_data))
    }
    
}

impl Drop for VolumeDetector {
    fn drop(&mut self) {
        dev_log!("VolumeDetector dropped");
    }
}