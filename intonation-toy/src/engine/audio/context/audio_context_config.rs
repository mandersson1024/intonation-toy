use crate::app_config::STANDARD_SAMPLE_RATE;

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
            buffer_size: 1024,    // Production buffer size
            max_recreation_attempts: 3,
        }
    }
}

impl AudioContextConfig {
    pub fn with_buffer_size(mut self, buffer_size: u32) -> Self {
        self.buffer_size = buffer_size;
        self
    }
}