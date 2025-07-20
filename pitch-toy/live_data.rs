use observable_data::DataObserver;
use crate::engine::audio::{
    AudioPermission,
    AudioDevices,
};
use crate::debug::egui::live_data_panel::{PerformanceMetrics, VolumeLevelData, PitchData, AudioWorkletStatus};

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

/// Hybrid LiveData structure using interface observers for core data and direct observers for debug data
#[derive(Clone)]
pub struct HybridLiveData {
    // Core data via interface observers (read-only)
    pub audio_analysis: DataObserver<Option<crate::module_interfaces::engine_to_model::AudioAnalysis>>,
    pub permission_state: DataObserver<crate::module_interfaces::engine_to_model::PermissionState>,
    pub audio_errors: DataObserver<Vec<crate::module_interfaces::engine_to_model::AudioError>>,
    
    // Debug-specific data (direct access)
    pub audio_devices: DataObserver<AudioDevices>,
    pub performance_metrics: DataObserver<PerformanceMetrics>,
    pub audioworklet_status: DataObserver<AudioWorkletStatus>,
    pub buffer_pool_stats: DataObserver<Option<crate::engine::audio::message_protocol::BufferPoolStats>>,
}

impl HybridLiveData {
    /// Create new HybridLiveData from interface observers and debug-specific observers
    pub fn new(
        engine_to_model: &crate::module_interfaces::engine_to_model::EngineToModelInterface,
        audio_devices: DataObserver<AudioDevices>,
        performance_metrics: DataObserver<PerformanceMetrics>,
        audioworklet_status: DataObserver<AudioWorkletStatus>,
        buffer_pool_stats: DataObserver<Option<crate::engine::audio::message_protocol::BufferPoolStats>>,
    ) -> Self {
        Self {
            // Core data from interfaces
            audio_analysis: engine_to_model.audio_analysis_observer(),
            permission_state: engine_to_model.permission_state_observer(),
            audio_errors: engine_to_model.audio_errors_observer(),
            
            // Debug-specific data
            audio_devices,
            performance_metrics,
            audioworklet_status,
            buffer_pool_stats,
        }
    }

    /// Get volume level data from audio analysis interface
    pub fn get_volume_level(&self) -> Option<VolumeLevelData> {
        self.audio_analysis.get().map(|analysis| VolumeLevelData {
            rms_db: analysis.volume_level.rms,
            peak_db: analysis.volume_level.peak,
        })
    }

    /// Get pitch data from audio analysis interface
    pub fn get_pitch_data(&self) -> Option<PitchData> {
        self.audio_analysis.get().and_then(|analysis| {
            match analysis.pitch {
                crate::module_interfaces::engine_to_model::Pitch::Detected(frequency, clarity) => {
                    Some(PitchData {
                        frequency,
                        confidence: clarity, // Use clarity as confidence approximation
                        note: crate::engine::audio::MusicalNote::new(
                            crate::engine::audio::NoteName::A, // Placeholder
                            4, 
                            0.0, 
                            frequency
                        ),
                        clarity,
                        timestamp: analysis.timestamp,
                    })
                }
                crate::module_interfaces::engine_to_model::Pitch::NotDetected => None,
            }
        })
    }

    /// Convert permission state to legacy AudioPermission format
    pub fn get_microphone_permission(&self) -> AudioPermission {
        match self.permission_state.get() {
            crate::module_interfaces::engine_to_model::PermissionState::NotRequested => AudioPermission::Uninitialized,
            crate::module_interfaces::engine_to_model::PermissionState::Requested => AudioPermission::Requesting,
            crate::module_interfaces::engine_to_model::PermissionState::Granted => AudioPermission::Granted,
            crate::module_interfaces::engine_to_model::PermissionState::Denied => AudioPermission::Denied,
        }
    }
}