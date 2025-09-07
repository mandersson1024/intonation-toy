
#![cfg(target_arch = "wasm32")]

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PerformanceMetrics {
    pub fps: f64,
    pub memory_usage_mb: f64,
    pub memory_usage_percent: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VolumeLevelData {
    pub rms_amplitude: f32,
    pub peak_amplitude: f32,
    pub fft_data: Option<Vec<f32>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PitchData {
    pub frequency: f32,
    pub clarity: f32,
}


impl From<crate::engine::audio::VolumeLevelData> for VolumeLevelData {
    fn from(data: crate::engine::audio::VolumeLevelData) -> Self {
        Self { 
            rms_amplitude: data.rms_amplitude, 
            peak_amplitude: data.peak_amplitude,
            fft_data: data.fft_data.clone(),
        }
    }
}



