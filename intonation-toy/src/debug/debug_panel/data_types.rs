// Debug-specific data types for EGUI display
// These are display-oriented versions of engine data types

use crate::engine::audio::AudioWorkletState;
use crate::engine::audio::buffer::AUDIO_CHUNK_SIZE;

/// Performance metrics for display
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceMetrics {
    /// Frames per second (already implemented)
    pub fps: f64,
    /// JavaScript heap memory usage in megabytes (estimated via Performance API)
    /// Note: Memory metrics are estimates and may not be available on all browsers
    pub memory_usage_mb: f64,
    /// Percentage of allocated heap being used (estimated via Performance API)
    /// Note: Memory metrics are estimates and may not be available on all browsers
    pub memory_usage_percent: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            fps: 0.0,
            memory_usage_mb: 0.0,
            memory_usage_percent: 0.0,
        }
    }
}

/// Volume level data for display
#[derive(Debug, Clone, PartialEq)]
pub struct VolumeLevelData {
    pub rms_amplitude: f32,
    pub peak_amplitude: f32,
}

/// Pitch detection data for display
#[derive(Debug, Clone, PartialEq)]
pub struct PitchData {
    pub frequency: f32,
    pub clarity: f32,
    pub timestamp: f64,
}

/// AudioWorklet status for display
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
            batch_size: crate::engine::audio::buffer::BUFFER_SIZE as u32,
            batches_processed: 0,
        }
    }
}

// Conversion functions between debug types and audio types
impl From<crate::engine::audio::VolumeLevelData> for VolumeLevelData {
    fn from(audio_data: crate::engine::audio::VolumeLevelData) -> Self {
        Self {
            rms_amplitude: audio_data.rms_amplitude,
            peak_amplitude: audio_data.peak_amplitude,
        }
    }
}

impl From<crate::engine::audio::PitchData> for PitchData {
    fn from(audio_data: crate::engine::audio::PitchData) -> Self {
        Self {
            frequency: audio_data.frequency,
            clarity: audio_data.clarity,
            timestamp: audio_data.timestamp,
        }
    }
}

impl From<crate::engine::audio::AudioWorkletStatus> for AudioWorkletStatus {
    fn from(audio_data: crate::engine::audio::AudioWorkletStatus) -> Self {
        Self {
            state: audio_data.state,
            processor_loaded: audio_data.processor_loaded,
            chunk_size: audio_data.chunk_size,
            batch_size: audio_data.batch_size,
            batches_processed: audio_data.batches_processed,
        }
    }
}

