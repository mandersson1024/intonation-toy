
#[derive(Default)]
pub struct FpsCounter {
    last_timestamp: f64,
}

impl FpsCounter {
    pub fn update(&mut self, timestamp: f64) -> f64 {
        let delta = timestamp - self.last_timestamp;
        self.last_timestamp = timestamp;
        1000.0 / delta
    }
}
