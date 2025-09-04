use web_sys::{AnalyserNode};
use super::{AudioError, data_types::VolumeAnalysis};
use super::analysis;

const VOLUME_DATA_LENGTH: u32 = 128;

pub struct VolumeDetector {
    analyser_node: AnalyserNode,
    sample_data: Vec<f32>,
}

impl VolumeDetector {
    pub fn new(analyser_node: AnalyserNode) -> Self {
        let data = vec![0.0f32; VOLUME_DATA_LENGTH as usize];
        
        Self {
            analyser_node,
            sample_data: data,
        }
    }
    
    pub fn analyze(&mut self) -> Result<VolumeAnalysis, AudioError> {
        self.analyser_node.get_float_time_domain_data(&mut self.sample_data);
        Ok(analysis::analyze_volume(&self.sample_data))
    }
    
}
