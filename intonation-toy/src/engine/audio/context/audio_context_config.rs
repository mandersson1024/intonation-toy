use crate::app_config::{BUFFER_SIZE, STANDARD_SAMPLE_RATE};

#[derive(Debug, Clone)]
pub struct AudioContextConfig {
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub max_recreation_attempts: u32,
}

impl Default for AudioContextConfig {
    fn default() -> Self {
        Self {
            sample_rate: STANDARD_SAMPLE_RATE,
            buffer_size: BUFFER_SIZE as u32,
            max_recreation_attempts: 3,
        }
    }
}