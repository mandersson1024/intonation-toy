
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
