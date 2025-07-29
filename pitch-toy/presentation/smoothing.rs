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
/// use pitch_toy::presentation::smoothing::EmaSmoother;
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
        assert!(smoothing_factor >= 0.0 && smoothing_factor <= 1.0, 
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
        assert!(factor >= 0.0 && factor <= 1.0, "EMA smoothing factor must be between 0.0 and 1.0");
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

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    
    /// Test EMA smoother creation and basic functionality
    #[wasm_bindgen_test]
    fn test_ema_smoother_creation() {
        let smoother = EmaSmoother::new(0.2);
        assert_eq!(smoother.get_smoothing_factor(), 0.2);
        assert!(!smoother.is_initialized());
    }
    
    /// Test EMA smoother creation from period
    #[wasm_bindgen_test]
    fn test_ema_smoother_from_period() {
        let smoother = EmaSmoother::from_period(19.0);
        // Period 19 should give smoothing factor of 2/(19+1) = 0.1
        assert!((smoother.get_smoothing_factor() - 0.1).abs() < 0.001);
    }
    
    /// Test default EMA smoother
    #[wasm_bindgen_test]
    fn test_ema_smoother_default() {
        let smoother = EmaSmoother::default();
        assert_eq!(smoother.get_smoothing_factor(), 0.1);
        assert!(!smoother.is_initialized());
    }
    
    /// Test EMA initialization behavior
    #[wasm_bindgen_test]
    fn test_ema_initialization() {
        let mut smoother = EmaSmoother::new(0.3);
        
        // Verify initial state
        assert!(!smoother.is_initialized());
        
        // First call should initialize with the input value
        let first_value = 42.5;
        let result = smoother.apply(first_value);
        
        // Should return the first value unchanged
        assert_eq!(result, first_value);
        
        // Should now be initialized
        assert!(smoother.is_initialized());
    }
    
    /// Test standard EMA calculation with known values
    #[wasm_bindgen_test]
    fn test_ema_standard_calculation() {
        let mut smoother = EmaSmoother::new(0.1);
        
        // Initialize with first value
        let first_value = 100.0;
        let result1 = smoother.apply(first_value);
        assert_eq!(result1, first_value);
        
        // Second call should use EMA formula
        let second_value = 200.0;
        let result2 = smoother.apply(second_value);
        
        // Expected: (200.0 * 0.1) + (100.0 * 0.9) = 20.0 + 90.0 = 110.0
        let expected2 = (second_value * 0.1) + (first_value * 0.9);
        assert!((result2 - expected2).abs() < 0.001);
        
        // Third call using previous result
        let third_value = 50.0;
        let result3 = smoother.apply(third_value);
        
        // Expected: (50.0 * 0.1) + (110.0 * 0.9) = 5.0 + 99.0 = 104.0
        let expected3 = (third_value * 0.1) + (result2 * 0.9);
        assert!((result3 - expected3).abs() < 0.001);
    }
    
    /// Test EMA with different smoothing factors
    #[wasm_bindgen_test]
    fn test_ema_different_smoothing_factors() {
        // Test with high smoothing factor (quick response)
        let mut smoother_high = EmaSmoother::new(0.9);
        smoother_high.apply(10.0); // Initialize
        let result_high = smoother_high.apply(100.0);
        
        // Expected: (100.0 * 0.9) + (10.0 * 0.1) = 90.0 + 1.0 = 91.0
        assert!((result_high - 91.0).abs() < 0.001);
        
        // Test with low smoothing factor (more smoothing)
        let mut smoother_low = EmaSmoother::new(0.1);
        smoother_low.apply(10.0); // Initialize
        let result_low = smoother_low.apply(100.0);
        
        // Expected: (100.0 * 0.1) + (10.0 * 0.9) = 10.0 + 9.0 = 19.0
        assert!((result_low - 19.0).abs() < 0.001);
        
        // Test with smoothing factor of 1.0 (no smoothing)
        let mut smoother_none = EmaSmoother::new(1.0);
        smoother_none.apply(10.0); // Initialize
        let result_none = smoother_none.apply(100.0);
        
        // Should return current value unchanged
        assert_eq!(result_none, 100.0);
        
        // Test with smoothing factor of 0.0 (maximum smoothing)
        let mut smoother_max = EmaSmoother::new(0.0);
        smoother_max.apply(10.0); // Initialize
        let result_max = smoother_max.apply(100.0);
        
        // Should return previous value unchanged
        assert_eq!(result_max, 10.0);
    }
    
    /// Test EMA reset functionality
    #[wasm_bindgen_test]
    fn test_ema_reset_behavior() {
        let mut smoother = EmaSmoother::new(0.5);
        
        // Initialize and perform some calculations
        smoother.apply(10.0);
        smoother.apply(20.0);
        smoother.apply(30.0);
        
        // Verify EMA is initialized
        assert!(smoother.is_initialized());
        
        // Reset EMA state
        smoother.reset();
        
        // Verify reset state
        assert!(!smoother.is_initialized());
        
        // Next call should behave like initialization
        let after_reset = smoother.apply(100.0);
        assert_eq!(after_reset, 100.0);
        assert!(smoother.is_initialized());
    }
    
    /// Test EMA configuration methods
    #[wasm_bindgen_test]
    fn test_ema_configuration() {
        let mut smoother = EmaSmoother::new(0.3);
        
        // Test factor setting and getting
        assert_eq!(smoother.get_smoothing_factor(), 0.3);
        
        smoother.set_smoothing_factor(0.2);
        assert_eq!(smoother.get_smoothing_factor(), 0.2);
        
        // Test period conversion
        smoother.set_period(19.0); // Should give smoothing factor of 2/(19+1) = 0.1
        assert!((smoother.get_smoothing_factor() - 0.1).abs() < 0.001);
        
        let calculated_period = smoother.get_period();
        assert!((calculated_period - 19.0).abs() < 0.001);
    }
    
    /// Test EMA with edge cases and extreme values
    #[wasm_bindgen_test]
    fn test_ema_edge_cases() {
        let mut smoother = EmaSmoother::new(0.2);
        
        // Test with zero values
        let result_zero = smoother.apply(0.0);
        assert_eq!(result_zero, 0.0);
        
        let result_after_zero = smoother.apply(10.0);
        let expected_after_zero = (10.0 * 0.2) + (0.0 * 0.8);
        assert!((result_after_zero - expected_after_zero).abs() < 0.001);
        
        // Test with negative values
        smoother.reset();
        let result_negative = smoother.apply(-50.0);
        assert_eq!(result_negative, -50.0);
        
        let result_mixed = smoother.apply(25.0);
        let expected_mixed = (25.0 * 0.2) + (-50.0 * 0.8);
        assert!((result_mixed - expected_mixed).abs() < 0.001);
        
        // Test with very large values
        smoother.reset();
        let large_value = 1e6;
        let result_large = smoother.apply(large_value);
        assert_eq!(result_large, large_value);
        
        // Test with very small values
        smoother.reset();
        let small_value = 1e-6;
        let result_small = smoother.apply(small_value);
        assert!((result_small - small_value).abs() < 1e-9);
    }
    
    /// Test EMA validation (panic cases)
    #[wasm_bindgen_test]
    #[should_panic(expected = "EMA smoothing factor must be between 0.0 and 1.0")]
    fn test_ema_factor_validation_negative() {
        EmaSmoother::new(-0.1);
    }
    
    #[wasm_bindgen_test]
    #[should_panic(expected = "EMA smoothing factor must be between 0.0 and 1.0")]
    fn test_ema_factor_validation_too_large() {
        EmaSmoother::new(1.1);
    }
    
    #[wasm_bindgen_test]
    #[should_panic(expected = "EMA period must be positive")]
    fn test_ema_period_validation_zero() {
        EmaSmoother::from_period(0.0);
    }
    
    #[wasm_bindgen_test]
    #[should_panic(expected = "EMA period must be positive")]
    fn test_ema_period_validation_negative() {
        EmaSmoother::from_period(-5.0);
    }
    
    #[wasm_bindgen_test]
    #[should_panic(expected = "EMA smoothing factor must be between 0.0 and 1.0")]
    fn test_ema_set_factor_validation_negative() {
        let mut smoother = EmaSmoother::new(0.5);
        smoother.set_smoothing_factor(-0.1);
    }
    
    #[wasm_bindgen_test]
    #[should_panic(expected = "EMA smoothing factor must be between 0.0 and 1.0")]
    fn test_ema_set_factor_validation_too_large() {
        let mut smoother = EmaSmoother::new(0.5);
        smoother.set_smoothing_factor(1.1);
    }
    
    #[wasm_bindgen_test]
    #[should_panic(expected = "EMA period must be positive")]
    fn test_ema_set_period_validation_zero() {
        let mut smoother = EmaSmoother::new(0.5);
        smoother.set_period(0.0);
    }
    
    #[wasm_bindgen_test]
    #[should_panic(expected = "EMA period must be positive")]
    fn test_ema_set_period_validation_negative() {
        let mut smoother = EmaSmoother::new(0.5);
        smoother.set_period(-5.0);
    }
}