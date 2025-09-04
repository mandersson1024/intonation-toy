use web_sys::AnalyserNode;
use super::data_types::VolumeAnalysis;
use super::analysis;

const NUM_SAMPLES: usize = 512;

pub struct VolumeDetector {
    node: AnalyserNode,
    data: Vec<f32>,
}

impl VolumeDetector {
    pub fn new(analyser_node: AnalyserNode) -> Self {
        Self {
            node: analyser_node,
            data: vec![0.0f32; NUM_SAMPLES],
        }
    }
    
    pub fn analyze(&mut self) -> VolumeAnalysis {
        self.node.get_float_time_domain_data(&mut self.data);
        analysis::analyze_volume(&self.data)
    }
    
}
