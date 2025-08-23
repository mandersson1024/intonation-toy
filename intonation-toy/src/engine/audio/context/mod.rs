mod audio_context_state;
mod audio_devices;
mod audio_context_manager;
mod audio_system_context;

pub use audio_context_state::AudioContextState;
pub use audio_devices::AudioDevices;
pub use audio_context_manager::AudioContextManager;
pub use audio_system_context::AudioSystemContext;

use crate::common::shared_types::{Volume, Pitch, AudioAnalysis};
use super::data_types::{VolumeLevelData, PitchData};

pub(super) fn convert_volume_data(volume_data: Option<VolumeLevelData>) -> Option<Volume> {
    volume_data.map(|data| Volume {
        peak_amplitude: data.peak_amplitude,
        rms_amplitude: data.rms_amplitude,
    })
}

pub(super) fn convert_pitch_data(pitch_data: Option<PitchData>) -> Option<Pitch> {
    pitch_data.map(|data| {
        if data.frequency > 0.0 {
            Pitch::Detected(data.frequency, data.clarity)
        } else {
            Pitch::NotDetected
        }
    })
}

pub(super) fn merge_audio_analysis(
    volume: Option<Volume>,
    pitch: Option<Pitch>,
    timestamp: f64
) -> Option<AudioAnalysis> {
    (volume.is_some() || pitch.is_some()).then(|| AudioAnalysis {
        volume_level: volume.unwrap_or(Volume { peak_amplitude: 0.0, rms_amplitude: 0.0 }),
        pitch: pitch.unwrap_or(Pitch::NotDetected),
        fft_data: None,
        timestamp: timestamp.max(js_sys::Date::now()),
    })
}