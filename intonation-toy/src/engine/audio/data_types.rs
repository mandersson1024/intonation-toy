use super::AudioWorkletState;
use crate::app_config::{AUDIO_CHUNK_SIZE, BUFFER_SIZE};

/// Volume level data for external consumption
/// 
/// Contains RMS and peak amplitude measurements, and optionally FFT frequency data.
/// The FFT data is normalized to 0.0-1.0 range and contains frequency bin magnitudes.
/// FFT data is None for traditional volume detection and Some(Vec<f32>) for analyser-based detection.
#[derive(Debug, Clone, PartialEq)]
pub struct VolumeLevelData {
    pub rms_amplitude: f32,
    pub peak_amplitude: f32,
    pub fft_data: Option<Vec<f32>>,
}

/// Internal volume analysis result from volume detection
/// 
/// Contains raw amplitude measurements and optional FFT frequency data from volume detection.
/// This is used internally by volume detectors before converting to VolumeLevelData for external consumption.
/// 
/// The fft_data contains normalized frequency bin data (0.0-1.0 range) when available:
/// - Some(Vec<f32>) for FFT-based detection (64 frequency bins when using 128 FFT size)
/// - None for traditional sample-based detection
#[derive(Debug, Clone)]
pub struct VolumeAnalysis {
    pub rms_amplitude: f32,
    pub peak_amplitude: f32,
    pub fft_data: Option<Vec<f32>>,
}

/// Pitch detection data for external consumption
#[derive(Debug, Clone, PartialEq)]
pub struct PitchData {
    pub frequency: f32,
    pub clarity: f32,
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