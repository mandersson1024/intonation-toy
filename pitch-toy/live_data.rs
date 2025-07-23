use observable_data::DataObserver;
use crate::engine::audio::{
    AudioPermission,
    AudioDevices,
};
use crate::debug::egui::data_types::{PerformanceMetrics, VolumeLevelData, PitchData, AudioWorkletStatus};

/// Legacy LiveData structure for backward compatibility
#[derive(Clone)]
pub struct LiveData {
    pub microphone_permission: DataObserver<AudioPermission>,
    pub audio_devices: DataObserver<AudioDevices>,
    pub performance_metrics: DataObserver<PerformanceMetrics>,
    pub volume_level: DataObserver<Option<VolumeLevelData>>,
    pub pitch_data: DataObserver<Option<PitchData>>,
    pub audioworklet_status: DataObserver<AudioWorkletStatus>,
    pub buffer_pool_stats: DataObserver<Option<crate::engine::audio::message_protocol::BufferPoolStats>>,
}

/// Hybrid LiveData structure using direct observers for debug data (interface-free)
#[derive(Clone)]
pub struct HybridLiveData {
    // Debug-specific data (direct access)
    pub audio_devices: DataObserver<AudioDevices>,
    pub performance_metrics: DataObserver<PerformanceMetrics>,
    pub audioworklet_status: DataObserver<AudioWorkletStatus>,
    pub buffer_pool_stats: DataObserver<Option<crate::engine::audio::message_protocol::BufferPoolStats>>,
}

impl HybridLiveData {
    /// Create new HybridLiveData from debug-specific observers (interface-free)
    pub fn new(
        audio_devices: DataObserver<AudioDevices>,
        performance_metrics: DataObserver<PerformanceMetrics>,
        audioworklet_status: DataObserver<AudioWorkletStatus>,
        buffer_pool_stats: DataObserver<Option<crate::engine::audio::message_protocol::BufferPoolStats>>,
    ) -> Self {
        Self {
            // Debug-specific data
            audio_devices,
            performance_metrics,
            audioworklet_status,
            buffer_pool_stats,
        }
    }

    /// Get volume level data (placeholder implementation)
    pub fn get_volume_level(&self) -> Option<VolumeLevelData> {
        // TODO: Implement direct volume data access when debug layer update pattern is implemented
        None
    }

    /// Get pitch data (placeholder implementation)
    pub fn get_pitch_data(&self) -> Option<PitchData> {
        // TODO: Implement direct pitch data access when debug layer update pattern is implemented
        None
    }

    /// Get microphone permission (placeholder implementation)
    pub fn get_microphone_permission(&self) -> AudioPermission {
        // TODO: Implement direct permission state access when debug layer update pattern is implemented
        AudioPermission::Uninitialized
    }
}