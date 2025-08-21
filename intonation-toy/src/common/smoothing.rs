/// EMA smoother for data smoothing over time
#[derive(Debug, Clone, PartialEq)]
pub struct EmaSmoother {
    /// EMA smoothing factor (alpha) for exponential moving average calculations
    /// Value between 0.0 and 1.0, where higher values give more weight to recent data
    smoothing_factor: f32,
    
    /// Previous EMA value used for calculating the next smoothed value
    previous_ema_value: f32,
    
    /// Whether the EMA has been initialized with the first value
    initialized: bool,
}

impl EmaSmoother {
    /// Create a new EMA smoother with the specified smoothing factor
    pub fn new(smoothing_factor: f32) -> Self {
        assert!((0.0..=1.0).contains(&smoothing_factor), 
                "EMA smoothing factor must be between 0.0 and 1.0");
        
        Self {
            smoothing_factor,
            previous_ema_value: 0.0,
            initialized: false,
        }
    }
    
    
    /// Apply exponential moving average smoothing to a value
    pub fn apply(&mut self, current_value: f32) -> f32 {
        if !self.initialized {
            self.previous_ema_value = current_value;
            self.initialized = true;
            current_value
        } else {
            let new_ema = (current_value * self.smoothing_factor) + 
                         (self.previous_ema_value * (1.0 - self.smoothing_factor));
            self.previous_ema_value = new_ema;
            new_ema
        }
    }
    
    /// Reset the EMA state to initial conditions
    pub fn reset(&mut self) {
        self.initialized = false;
        self.previous_ema_value = 0.0;
    }
    
}

impl Default for EmaSmoother {
    /// Create a default EMA smoother with a smoothing factor of 0.1
    fn default() -> Self {
        Self::new(0.1)
    }
}

