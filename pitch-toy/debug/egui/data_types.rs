// Debug-specific data types for EGUI display
// These are display-oriented versions of engine data types

use crate::engine::audio::{MusicalNote, AudioWorkletState};

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
    pub rms_db: f32,
    pub peak_db: f32,
}

/// Pitch detection data for display
#[derive(Debug, Clone, PartialEq)]
pub struct PitchData {
    pub frequency: f32,
    pub confidence: f32,
    pub note: MusicalNote,
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
    pub last_update: f64,
}

impl Default for AudioWorkletStatus {
    fn default() -> Self {
        Self {
            state: AudioWorkletState::Uninitialized,
            processor_loaded: false,
            chunk_size: 128,
            chunks_processed: 0,
            last_update: 0.0,
        }
    }
}

// Conversion functions between debug types and audio types
impl From<crate::engine::audio::VolumeLevelData> for VolumeLevelData {
    fn from(audio_data: crate::engine::audio::VolumeLevelData) -> Self {
        Self {
            rms_db: audio_data.rms_db,
            peak_db: audio_data.peak_db,
        }
    }
}

impl From<crate::engine::audio::PitchData> for PitchData {
    fn from(audio_data: crate::engine::audio::PitchData) -> Self {
        Self {
            frequency: audio_data.frequency,
            confidence: audio_data.confidence,
            note: audio_data.note,
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
            last_update: audio_data.last_update,
        }
    }
}

// Adapter types for observable data conversion
pub struct PitchDataAdapter {
    inner: std::rc::Rc<dyn observable_data::DataSetter<Option<PitchData>>>,
}

impl PitchDataAdapter {
    pub fn new(inner: std::rc::Rc<dyn observable_data::DataSetter<Option<PitchData>>>) -> Self {
        Self { inner }
    }
}

unsafe impl Send for PitchDataAdapter {}
unsafe impl Sync for PitchDataAdapter {}

impl observable_data::DataSetter<Option<crate::engine::audio::PitchData>> for PitchDataAdapter {
    fn set(&self, data: Option<crate::engine::audio::PitchData>) {
        let converted = data.map(|d| d.into());
        self.inner.set(converted);
    }
}

pub struct VolumeDataAdapter {
    inner: std::rc::Rc<dyn observable_data::DataSetter<Option<VolumeLevelData>>>,
}

impl VolumeDataAdapter {
    pub fn new(inner: std::rc::Rc<dyn observable_data::DataSetter<Option<VolumeLevelData>>>) -> Self {
        Self { inner }
    }
}

unsafe impl Send for VolumeDataAdapter {}
unsafe impl Sync for VolumeDataAdapter {}

impl observable_data::DataSetter<Option<crate::engine::audio::VolumeLevelData>> for VolumeDataAdapter {
    fn set(&self, data: Option<crate::engine::audio::VolumeLevelData>) {
        let converted = data.map(|d| d.into());
        self.inner.set(converted);
    }
}

pub struct AudioWorkletStatusAdapter {
    inner: std::rc::Rc<dyn observable_data::DataSetter<AudioWorkletStatus>>,
}

impl AudioWorkletStatusAdapter {
    pub fn new(inner: std::rc::Rc<dyn observable_data::DataSetter<AudioWorkletStatus>>) -> Self {
        Self { inner }
    }
}

unsafe impl Send for AudioWorkletStatusAdapter {}
unsafe impl Sync for AudioWorkletStatusAdapter {}

impl observable_data::DataSetter<crate::engine::audio::AudioWorkletStatus> for AudioWorkletStatusAdapter {
    fn set(&self, data: crate::engine::audio::AudioWorkletStatus) {
        let converted = data.into();
        self.inner.set(converted);
    }
}