mod audio_context_state;
mod audio_context_config;
mod audio_devices;
mod audio_context_manager;
mod audio_system_context;

pub use audio_context_state::AudioContextState;
pub use audio_context_config::AudioContextConfig;
pub use audio_devices::AudioDevices;
pub use audio_context_manager::AudioContextManager;
pub use audio_system_context::AudioSystemContext;

// Re-export the conversion functions that were in the original context.rs
pub fn convert_volume_data(volume_data: Option<super::data_types::VolumeLevelData>) -> Option<crate::shared_types::Volume> {
    volume_data.map(|data| crate::shared_types::Volume {
        peak_amplitude: data.peak_amplitude,
        rms_amplitude: data.rms_amplitude,
    })
}

pub fn convert_pitch_data(pitch_data: Option<super::data_types::PitchData>) -> Option<crate::shared_types::Pitch> {
    pitch_data.map(|data| {
        if data.frequency > 0.0 {
            crate::shared_types::Pitch::Detected(data.frequency, data.clarity)
        } else {
            crate::shared_types::Pitch::NotDetected
        }
    })
}

pub fn merge_audio_analysis(
    volume: Option<crate::shared_types::Volume>,
    pitch: Option<crate::shared_types::Pitch>,
    timestamp: f64
) -> Option<crate::shared_types::AudioAnalysis> {
    (volume.is_some() || pitch.is_some()).then(|| crate::shared_types::AudioAnalysis {
        volume_level: volume.unwrap_or(crate::shared_types::Volume { peak_amplitude: -60.0, rms_amplitude: -60.0 }),
        pitch: pitch.unwrap_or(crate::shared_types::Pitch::NotDetected),
        fft_data: None,
        timestamp: timestamp.max(js_sys::Date::now()),
    })
}