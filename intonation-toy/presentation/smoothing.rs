//! EMA (Exponential Moving Average) Smoothing Module
//! 
//! This module provides exponential moving average functionality for smoothing
//! noisy data over time. It's particularly useful for smoothing pitch detection
//! and other audio-related data that may have rapid fluctuations.
//! 
//! ## EMA Formula
//! 
//! The standard EMA formula used is:
//! - First call: EMA = current_value (initialization)
//! - Subsequent calls: EMA = (current_value × smoothing_factor) + (previous_ema × (1 - smoothing_factor))
//! 
//! ## Smoothing Factor Behavior
//! 
//! - Higher values (closer to 1.0) respond more quickly to changes
//! - Lower values (closer to 0.0) provide more smoothing and stability
//! - Factor of 1.0 = no smoothing (returns current_value)
//! - Factor of 0.0 = maximum smoothing (returns previous_ema)

/// EMA (Exponential Moving Average) smoother for data smoothing over time
/// 
/// This struct implements exponential moving average smoothing to reduce noise
/// in time-series data. It maintains state between calls to provide continuous
/// smoothing of incoming data points.
/// 
/// # Examples
/// 
/// ```rust
/// use intonation_toy::presentation::smoothing::EmaSmoother;
/// 
/// let mut smoother = EmaSmoother::new(0.1);
/// 
/// // First value initializes the EMA
/// let smoothed1 = smoother.apply(100.0);
/// assert_eq!(smoothed1, 100.0);
/// 
/// // Subsequent values are smoothed
/// let smoothed2 = smoother.apply(200.0);
/// // Result will be: (200.0 * 0.1) + (100.0 * 0.9) = 110.0
/// ```
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
    /// 
    /// # Arguments
    /// 
    /// * `smoothing_factor` - The smoothing factor between 0.0 and 1.0
    /// 
    /// # Panics
    /// 
    /// Panics if the smoothing factor is not between 0.0 and 1.0 (inclusive)
    pub fn new(smoothing_factor: f32) -> Self {
        assert!((0.0..=1.0).contains(&smoothing_factor), 
                "EMA smoothing factor must be between 0.0 and 1.0");
        
        Self {
            smoothing_factor,
            previous_ema_value: 0.0,
            initialized: false,
        }
    }
    
    /// Create a new EMA smoother using a period instead of smoothing factor
    /// 
    /// The smoothing factor is calculated using the standard formula: 2.0 / (period + 1.0)
    /// 
    /// # Arguments
    /// 
    /// * `period` - The EMA period in samples (must be positive)
    /// 
    /// # Panics
    /// 
    /// Panics if the period is not positive
    pub fn from_period(period: f32) -> Self {
        assert!(period > 0.0, "EMA period must be positive");
        let smoothing_factor = 2.0 / (period + 1.0);
        Self::new(smoothing_factor)
    }
    
    /// Apply exponential moving average smoothing to a value
    /// 
    /// This method implements the standard EMA formula to smooth noisy data over time.
    /// On the first call (initialization), it uses the current value as the first EMA value.
    /// For subsequent calls, it applies the formula: new_ema = (current_value × k) + (previous_ema × (1 - k))
    /// where k is the smoothing factor (alpha).
    /// 
    /// # Arguments
    /// 
    /// * `current_value` - The new data point to be smoothed
    /// 
    /// # Returns
    /// 
    /// The smoothed value after applying EMA
    pub fn apply(&mut self, current_value: f32) -> f32 {
        if !self.initialized {
            // First call: initialize EMA with the current value
            self.previous_ema_value = current_value;
            self.initialized = true;
            current_value
        } else {
            // Subsequent calls: apply standard EMA formula
            let new_ema = (current_value * self.smoothing_factor) + 
                         (self.previous_ema_value * (1.0 - self.smoothing_factor));
            self.previous_ema_value = new_ema;
            new_ema
        }
    }
    
    /// Reset the EMA state to initial conditions
    /// 
    /// Clears the EMA history by resetting the initialization flag and
    /// previous value. The next EMA calculation will start fresh.
    pub fn reset(&mut self) {
        self.initialized = false;
        self.previous_ema_value = 0.0;
    }
    
    /// Check whether the EMA has been initialized
    /// 
    /// Returns true if at least one EMA calculation has been performed,
    /// false if the EMA is still in its initial state.
    /// 
    /// # Returns
    /// 
    /// True if EMA has been initialized, false otherwise
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Get the current EMA smoothing factor
    /// 
    /// Returns the alpha value used in exponential moving average calculations.
    /// A higher value gives more weight to recent data points.
    /// 
    /// # Returns
    /// 
    /// The current smoothing factor as a value between 0.0 and 1.0
    pub fn get_smoothing_factor(&self) -> f32 {
        self.smoothing_factor
    }
    
    /// Set the EMA smoothing factor directly
    /// 
    /// Sets the alpha value used in exponential moving average calculations.
    /// The smoothing factor determines how much weight recent values have
    /// compared to historical data.
    /// 
    /// # Arguments
    /// 
    /// * `factor` - The smoothing factor between 0.0 and 1.0
    /// 
    /// # Panics
    /// 
    /// Panics if the factor is not between 0.0 and 1.0 (inclusive)
    pub fn set_smoothing_factor(&mut self, factor: f32) {
        assert!((0.0..=1.0).contains(&factor), "EMA smoothing factor must be between 0.0 and 1.0");
        self.smoothing_factor = factor;
    }
    
    /// Get the equivalent EMA period for the current smoothing factor
    /// 
    /// Calculates the equivalent period (in samples) that would produce
    /// the same smoothing effect as the current smoothing factor.
    /// 
    /// # Returns
    /// 
    /// The equivalent EMA period as a floating-point number
    pub fn get_period(&self) -> f32 {
        (2.0 / self.smoothing_factor) - 1.0
    }
    
    /// Set the EMA period and calculate the corresponding smoothing factor
    /// 
    /// Sets the EMA configuration by specifying the period (in samples)
    /// rather than the smoothing factor directly. The smoothing factor
    /// is automatically calculated using the standard formula.
    /// 
    /// # Arguments
    /// 
    /// * `period` - The EMA period in samples (must be positive)
    /// 
    /// # Panics
    /// 
    /// Panics if the period is not positive
    pub fn set_period(&mut self, period: f32) {
        assert!(period > 0.0, "EMA period must be positive");
        self.smoothing_factor = 2.0 / (period + 1.0);
    }
}

impl Default for EmaSmoother {
    /// Create a default EMA smoother with a smoothing factor of 0.1
    /// 
    /// This corresponds to an EMA period of 19 samples, providing moderate smoothing.
    fn default() -> Self {
        Self::new(0.1)
    }
}

