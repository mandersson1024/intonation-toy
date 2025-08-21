use crate::engine::audio::AudioWorkletState;
use crate::app_config::{AUDIO_CHUNK_SIZE, BUFFER_SIZE};

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
}

#[derive(Debug, Clone, PartialEq)]
pub struct PitchData {
    pub frequency: f32,
    pub clarity: f32,
    pub timestamp: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioWorkletStatus {
    pub state: AudioWorkletState,
    pub processor_loaded: bool,
    pub chunk_size: u32,
    pub batch_size: u32,
    pub batches_processed: u32,
}

impl Default for AudioWorkletStatus {
    fn default() -> Self {
        Self {
            state: AudioWorkletState::Uninitialized,
            processor_loaded: false,
            chunk_size: AUDIO_CHUNK_SIZE as u32,
            batch_size: BUFFER_SIZE as u32,
            batches_processed: 0,
        }
    }
}

impl From<crate::engine::audio::VolumeLevelData> for VolumeLevelData {
    fn from(data: crate::engine::audio::VolumeLevelData) -> Self {
        Self { rms_amplitude: data.rms_amplitude, peak_amplitude: data.peak_amplitude }
    }
}

impl From<crate::engine::audio::PitchData> for PitchData {
    fn from(data: crate::engine::audio::PitchData) -> Self {
        Self { frequency: data.frequency, clarity: data.clarity, timestamp: data.timestamp }
    }
}

impl From<crate::engine::audio::AudioWorkletStatus> for AudioWorkletStatus {
    fn from(data: crate::engine::audio::AudioWorkletStatus) -> Self {
        Self {
            state: data.state,
            processor_loaded: data.processor_loaded,
            chunk_size: data.chunk_size,
            batch_size: data.batch_size,
            batches_processed: data.batches_processed,
        }
    }
}

