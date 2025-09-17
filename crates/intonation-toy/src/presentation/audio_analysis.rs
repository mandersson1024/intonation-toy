#![cfg(target_arch = "wasm32")]

#[derive(Default)]
pub struct AudioAnalysis {
    pub pitch_detected: bool,
    pub cents_offset: f32,
    pub interval: f32,
    pub clarity: Option<f32>,
    pub volume_peak: bool,
    pub frequency: f32,
}