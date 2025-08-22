use super::AudioWorkletState;
use crate::app_config::{AUDIO_CHUNK_SIZE, BUFFER_SIZE};

/// Volume level data for external consumption
#[derive(Debug, Clone, PartialEq)]
pub struct VolumeLevelData {
    pub rms_amplitude: f32,
    pub peak_amplitude: f32,
}

/// Pitch detection data for external consumption
#[derive(Debug, Clone, PartialEq)]
pub struct PitchData {
    pub frequency: f32,
    pub clarity: f32,
    pub timestamp: f64,
}

/// AudioWorklet status for external consumption
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