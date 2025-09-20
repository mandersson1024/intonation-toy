#![cfg(target_arch = "wasm32")]

use std::collections::VecDeque;

/// Adaptive EMA that smooths small jitter strongly while staying responsive on larger moves.
///
/// Optionally includes:
/// - Median-of-3 prefilter for cheap jitter reduction
/// - Hampel outlier suppression to remove transient spikes
/// - Deadband for extra smoothing near stillness
/// - Hysteresis to reduce flicker near threshold
#[derive(Debug, Clone)]
pub struct AdaptiveEMA {
    /// Lower bound on EMA factor (0 < alpha_min < 1). Small value => strong smoothing
    alpha_min: f32,

    /// Upper bound on EMA factor (alpha_min <= alpha_max < 1). Large value => snappier on big moves
    alpha_max: f32,

    /// Soft threshold for "jitter size" in the sigmoid mapping (typical noise scale)
    d_base: f32,

    /// Softness of the transition (smaller => steeper)
    s: f32,

    /// If true, applies median-of-3 on the input stream before filtering
    use_median3: bool,

    /// Buffer for median-of-3 filter
    m3_buf: Option<VecDeque<f32>>,

    /// If true, applies Hampel outlier suppression before filtering
    use_hampel: bool,

    /// Odd window size for the Hampel filter (e.g., 5, 7)
    hampel_window: usize,

    /// Sensitivity for Hampel; larger => fewer points flagged as outliers
    hampel_nsigma: f32,

    /// Buffer for Hampel filter
    h_buf: Option<VecDeque<f32>>,

    /// When |x - y_prev| < deadband, force alpha_t = alpha_min (extra smoothing near stillness)
    deadband: Option<f32>,

    /// (d_down, d_up) two-threshold hysteresis to reduce flicker
    hysteresis: Option<(f32, f32)>,

    /// Current mode for hysteresis ("quiet" or "moving")
    mode: Mode,

    /// Last filtered value
    y: Option<f32>,

    /// Whether the filter has been initialized
    initialized: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    Quiet,
    Moving,
}

impl AdaptiveEMA {
    /// Create a new adaptive EMA filter with specified parameters
    pub fn new(
        alpha_min: f32,
        alpha_max: f32,
        d: f32,
        s: f32,
    ) -> Self {
        assert!(
            0.0 < alpha_min && alpha_min <= alpha_max && alpha_max < 1.0,
            "Invalid alpha range: 0 < alpha_min <= alpha_max < 1"
        );
        assert!(d > 0.0, "d must be positive");
        assert!(s > 0.0, "s must be positive");

        Self {
            alpha_min,
            alpha_max,
            d_base: d,
            s,
            use_median3: false,
            m3_buf: None,
            use_hampel: false,
            hampel_window: 7,
            hampel_nsigma: 3.0,
            h_buf: None,
            deadband: None,
            hysteresis: None,
            mode: Mode::Quiet,
            y: None,
            initialized: false,
        }
    }

    /// Enable median-of-3 prefilter
    pub fn with_median3(mut self, enable: bool) -> Self {
        self.use_median3 = enable;
        if enable {
            self.m3_buf = Some(VecDeque::with_capacity(3));
        } else {
            self.m3_buf = None;
        }
        self
    }

    /// Enable Hampel outlier suppression
    pub fn with_hampel(mut self, enable: bool, window: usize, nsigma: f32) -> Self {
        if enable {
            assert!(window >= 3, "Hampel window must be at least 3");
            assert!(window % 2 == 1, "Hampel window must be odd");
        }

        self.use_hampel = enable;
        self.hampel_window = window;
        self.hampel_nsigma = nsigma;

        if enable {
            self.h_buf = Some(VecDeque::with_capacity(window));
        } else {
            self.h_buf = None;
        }
        self
    }

    /// Set deadband threshold
    pub fn with_deadband(mut self, deadband: f32) -> Self {
        self.deadband = if deadband > 0.0 { Some(deadband) } else { None };
        self
    }

    /// Set hysteresis thresholds
    pub fn with_hysteresis(mut self, d_down: f32, d_up: f32) -> Self {
        assert!(d_down < d_up, "d_down must be less than d_up");
        self.hysteresis = Some((d_down, d_up));
        self
    }

    /// Calculate median of values
    fn median(vals: &[f32]) -> f32 {
        if vals.is_empty() {
            return 0.0;
        }

        let mut sorted = vals.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = sorted.len();
        let mid = n / 2;

        if n % 2 == 1 {
            sorted[mid]
        } else {
            0.5 * (sorted[mid - 1] + sorted[mid])
        }
    }

    /// Calculate Median Absolute Deviation
    fn mad(vals: &[f32], med: f32) -> f32 {
        let deviations: Vec<f32> = vals.iter().map(|v| (*v - med).abs()).collect();
        Self::median(&deviations)
    }

    /// Apply median-of-3 filter
    fn median3(&mut self, x: f32) -> f32 {
        if let Some(buf) = &mut self.m3_buf {
            buf.push_back(x);
            if buf.len() > 3 {
                buf.pop_front();
            }

            if buf.len() < 3 {
                x
            } else {
                let vals: Vec<f32> = buf.iter().copied().collect();
                Self::median(&vals)
            }
        } else {
            x
        }
    }

    /// Apply Hampel outlier suppression (causal, uses past-only window)
    fn hampel(&mut self, x: f32) -> f32 {
        if let Some(buf) = &mut self.h_buf {
            buf.push_back(x);
            if buf.len() > self.hampel_window {
                buf.pop_front();
            }

            let vals: Vec<f32> = buf.iter().copied().collect();
            let med = Self::median(&vals);
            let mad = Self::mad(&vals, med);

            // 1.4826 factor makes MAD ~ std for normal distribution
            let sigma = if mad > 0.0 { 1.4826 * mad } else { 0.0 };

            if sigma == 0.0 {
                // Not enough variation yet; don't flag
                x
            } else if (x - med).abs() > self.hampel_nsigma * sigma {
                // Replace outlier with median
                med
            } else {
                x
            }
        } else {
            x
        }
    }

    /// Compute adaptive alpha based on delta between x and previous y
    fn compute_alpha(&mut self, x: f32, y_prev: f32) -> f32 {
        let delta = (x - y_prev).abs();

        // Optional deadband override
        if let Some(deadband) = self.deadband {
            if delta < deadband {
                return self.alpha_min;
            }
        }

        // Optional hysteresis by shifting the effective threshold d
        let d_eff = if let Some((d_down, d_up)) = self.hysteresis {
            // Update mode first based on current delta
            match self.mode {
                Mode::Quiet if delta > d_up => {
                    self.mode = Mode::Moving;
                }
                Mode::Moving if delta < d_down => {
                    self.mode = Mode::Quiet;
                }
                _ => {}
            }

            // Bias 'd' depending on mode
            match self.mode {
                Mode::Quiet => d_down,
                Mode::Moving => d_up,
            }
        } else {
            self.d_base
        };

        // Sigmoid mapping from delta -> alpha
        let u = (delta - d_eff) / self.s;
        let sig = 1.0 / (1.0 + (-u).exp());
        let a = self.alpha_min + (self.alpha_max - self.alpha_min) * sig;

        // Safety clamp
        a.clamp(self.alpha_min, self.alpha_max)
    }

    /// Process a single sample and return the filtered value
    pub fn update(&mut self, mut x: f32) -> f32 {
        // Optional prefilters
        if self.use_median3 {
            x = self.median3(x);
        }

        if self.use_hampel {
            x = self.hampel(x);
        }

        if let Some(y_prev) = self.y {
            let a = self.compute_alpha(x, y_prev);
            let new_y = (1.0 - a) * y_prev + a * x;
            self.y = Some(new_y);
            self.initialized = true;
            new_y
        } else {
            self.y = Some(x);
            self.initialized = true;
            x
        }
    }

    /// Apply the same smoothing algorithm used in update() but renamed for compatibility
    pub fn apply(&mut self, value: f32) -> f32 {
        self.update(value)
    }

    /// Reset the filter state to initial conditions
    pub fn reset(&mut self) {
        self.y = None;
        self.initialized = false;
        self.mode = Mode::Quiet;

        if let Some(buf) = &mut self.m3_buf {
            buf.clear();
        }

        if let Some(buf) = &mut self.h_buf {
            buf.clear();
        }
    }

    /// Filter a sequence (batch mode). Resets internal state.
    pub fn filter_series(&mut self, xs: &[f32]) -> Vec<f32> {
        self.reset();
        xs.iter().map(|&x| self.update(x)).collect()
    }
}

/// Builder for creating AdaptiveEMA with sensible defaults for pitch detection
impl Default for AdaptiveEMA {
    fn default() -> Self {
        Self::new(0.02, 0.6, 0.3, 0.1)
            .with_median3(true)
            .with_hampel(true, 7, 3.0)
            .with_deadband(0.05)
            .with_hysteresis(0.25, 0.45)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_ema_basic() {
        let mut filter = AdaptiveEMA::new(0.02, 0.6, 0.5, 0.15);

        // First value should be passed through
        assert_eq!(filter.update(1.0), 1.0);

        // Small changes should be smoothed strongly
        let result = filter.update(1.1);
        assert!(result < 1.1 && result > 1.0);

        // Large changes should be more responsive
        let result = filter.update(5.0);
        assert!(result > 2.0); // Should move significantly toward 5.0
    }

    #[test]
    fn test_median3() {
        let mut filter = AdaptiveEMA::new(0.02, 0.6, 0.5, 0.15)
            .with_median3(true);

        // Need 3 values for median to work
        filter.update(1.0);
        filter.update(10.0); // spike
        let result = filter.update(2.0);

        // Median of [1.0, 10.0, 2.0] is 2.0, so spike should be filtered
        assert!(result < 5.0);
    }

    #[test]
    fn test_reset() {
        let mut filter = AdaptiveEMA::new(0.02, 0.6, 0.5, 0.15);

        filter.update(10.0);
        filter.update(10.0);
        filter.reset();

        // After reset, should start fresh
        assert_eq!(filter.update(1.0), 1.0);
    }
}