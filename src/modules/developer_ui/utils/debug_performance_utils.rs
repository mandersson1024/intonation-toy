//! Debug Performance Utilities
//!
//! General performance utilities for debug components.

use std::time::{Duration, Instant};

/// Simple performance measurement utility
pub struct PerformanceMeasurement {
    start_time: Instant,
    label: String,
}

impl PerformanceMeasurement {
    pub fn new(label: &str) -> Self {
        Self {
            start_time: Instant::now(),
            label: label.to_string(),
        }
    }
    
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    pub fn elapsed_ms(&self) -> f64 {
        self.start_time.elapsed().as_millis() as f64
    }
    
    pub fn finish(self) -> Duration {
        let duration = self.elapsed();
        
        #[cfg(debug_assertions)]
        web_sys::console::log_1(&format!(
            "Performance measurement '{}': {:.2}ms",
            self.label,
            duration.as_millis()
        ).into());
        
        duration
    }
}

/// Helper macro for timing code blocks
#[macro_export]
macro_rules! time_block {
    ($label:expr, $block:block) => {{
        let _measurement = crate::modules::developer_ui::utils::debug_performance_utils::PerformanceMeasurement::new($label);
        let result = $block;
        _measurement.finish();
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_measurement() {
        let measurement = PerformanceMeasurement::new("test");
        std::thread::sleep(Duration::from_millis(1));
        let elapsed = measurement.elapsed();
        assert!(elapsed >= Duration::from_millis(1));
    }
} 