// Debug-specific data types for EGUI display
// These are display-oriented versions of engine data types

use crate::engine::audio::AudioWorkletState;

/// Performance metrics for display
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceMetrics {
    pub fps: f64,
    pub memory_usage: f64,
    pub audio_latency: f64,
    pub cpu_usage: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            fps: 0.0,
            memory_usage: 0.0,
            audio_latency: 0.0,
            cpu_usage: 0.0,
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
    pub chunks_processed: u32,
}

impl Default for AudioWorkletStatus {
    fn default() -> Self {
        Self {
            state: AudioWorkletState::Uninitialized,
            processor_loaded: false,
            chunk_size: 128,
            chunks_processed: 0,
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
            chunks_processed: audio_data.chunks_processed,
        }
    }
}

