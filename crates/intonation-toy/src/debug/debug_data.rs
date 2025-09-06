use crate::debug::data_types::{PerformanceMetrics, VolumeLevelData, PitchData};
use crate::common::shared_types::{EngineUpdateResult, ModelUpdateResult, IntonationData};

#[derive(Clone)]
pub struct DebugData {
    pub performance_metrics: PerformanceMetrics,
    pub buffer_pool_stats: Option<crate::engine::audio::message_protocol::BufferPoolStats>,
    pub volume_level: Option<VolumeLevelData>,
    pub pitch_data: Option<PitchData>,
    pub intonation_data: Option<IntonationData>,
    pub audio_errors: Vec<crate::common::shared_types::Error>,
    pub interval_semitones: Option<i32>,
    pub tuning_fork_note: Option<crate::common::shared_types::MidiNote>,
}

impl Default for DebugData {
    fn default() -> Self {
        Self {
            performance_metrics: PerformanceMetrics::default(),
            buffer_pool_stats: None,
            volume_level: None,
            pitch_data: None,
            intonation_data: None,
            audio_errors: Vec::new(),
            interval_semitones: None,
            tuning_fork_note: None,
        }
    }
}

impl DebugData {
    pub fn update_from_layers(
        &mut self,
        engine_result: &EngineUpdateResult,
        model_result: Option<&ModelUpdateResult>,
    ) {
        self.audio_errors = engine_result.audio_errors.clone();
        
        if let Some(analysis) = &engine_result.audio_analysis {
            self.volume_level = Some(VolumeLevelData {
                peak_amplitude: analysis.volume_level.peak_amplitude,
                rms_amplitude: analysis.volume_level.rms_amplitude,
                fft_data: analysis.fft_data.clone(),
            });
            
            self.pitch_data = match &analysis.pitch {
                crate::common::shared_types::Pitch::Detected(frequency, clarity) => {
                    Some(PitchData {
                        frequency: *frequency,
                        clarity: *clarity,
                    })
                },
                crate::common::shared_types::Pitch::NotDetected => None,
            };
        } else {
            self.volume_level = None;
            self.pitch_data = None;
        }
        
        if let Some(model) = model_result {
            self.intonation_data = Some(crate::common::shared_types::IntonationData {
                closest_midi_note: model.closest_midi_note,
                cents_offset: model.cents_offset,
            });
            self.interval_semitones = Some(model.interval_semitones);
            self.tuning_fork_note = Some(model.tuning_fork_note);
        }
    }
    
    pub fn update_debug_data(
        &mut self,
        performance_metrics: PerformanceMetrics,
        buffer_pool_stats: Option<crate::engine::audio::message_protocol::BufferPoolStats>,
    ) {
        self.performance_metrics = performance_metrics;
        if let Some(stats) = buffer_pool_stats { self.buffer_pool_stats = Some(stats); }
    }

}