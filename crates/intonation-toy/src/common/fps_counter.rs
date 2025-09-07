
#![cfg(target_arch = "wasm32")]

pub struct FpsCounter {
    last_timestamp: Option<f64>,
    fps_buffer: Option<Vec<f64>>,
    buffer_index: usize,
    frame_count: u32,
}

impl FpsCounter {
    pub fn new(frame_count: u32) -> Self {
        Self {
            last_timestamp: None,
            fps_buffer: None,
            buffer_index: 0,
            frame_count,
        }
    }

    pub fn update(&mut self, timestamp: f64) -> f64 {
        if let Some(last_ts) = self.last_timestamp {
            let delta = timestamp - last_ts;
            let current_fps = if delta > 0.0 { 1000.0 / delta } else { 0.0 };
            
            // Second call - initialize buffer with first FPS value
            if self.fps_buffer.is_none() {
                self.fps_buffer = Some(vec![current_fps; self.frame_count as usize]);
            } else if let Some(ref mut buffer) = self.fps_buffer {
                // Subsequent calls - update circular buffer
                buffer[self.buffer_index] = current_fps;
                self.buffer_index = (self.buffer_index + 1) % buffer.len();
            }
            
            self.last_timestamp = Some(timestamp);
            
            if let Some(ref buffer) = self.fps_buffer {
                let sum: f64 = buffer.iter().sum();
                sum / buffer.len() as f64
            } else {
                0.0
            }
        } else {
            // First call - just store timestamp and return 0
            self.last_timestamp = Some(timestamp);
            0.0
        }
    }
}
