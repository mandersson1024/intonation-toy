
#[derive(Debug, Clone, PartialEq)]
pub struct SignalGeneratorConfig {
    pub enabled: bool,
    pub frequency: f32,
    pub amplitude: f32,
    pub sample_rate: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TuningForkConfig {
    pub frequency: f32,
    pub volume: f32,
}

impl Default for TuningForkConfig {
    fn default() -> Self {
        Self {
            frequency: crate::common::music_theory::midi_note_to_standard_frequency(crate::app_config::DEFAULT_TUNING_FORK_NOTE),
            volume: 0.0,
        }
    }
}
