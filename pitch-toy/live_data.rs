use observable_data::DataObserver;
use crate::audio::{
    AudioPermission,
    AudioDevices,
    AudioContextState,
};
use crate::debug::egui::live_data_panel::{PerformanceMetrics, VolumeLevelData, PitchData, AudioWorkletStatus};

pub struct LiveData {
    pub microphone_permission: DataObserver<AudioPermission>,
    pub audio_devices: DataObserver<AudioDevices>,
    pub audio_context_state: DataObserver<AudioContextState>,
    pub performance_metrics: DataObserver<PerformanceMetrics>,
    pub volume_level: DataObserver<Option<VolumeLevelData>>,
    pub pitch_data: DataObserver<Option<PitchData>>,
    pub audioworklet_status: DataObserver<AudioWorkletStatus>,
}