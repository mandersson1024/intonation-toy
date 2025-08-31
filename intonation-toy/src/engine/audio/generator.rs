use web_sys::{OscillatorNode, GainNode};
use crate::common::dev_log;
use super::{
    AudioError,
    SignalGeneratorConfig,
    signal_generator::TuningForkConfig,
};

/// AudioGenerator manages all audio generation operations
/// 
/// This component is responsible for:
/// - Test signal generation and control
/// - Tuning fork generation and control
/// - Using oscillator and gain nodes from the AudioSignalFlow
/// - Managing generation state and parameters
/// 
/// The generator uses the oscillator nodes from the signal flow and provides
/// clean interfaces for controlling generated audio signals.
pub struct AudioGenerator {
    // Test signal nodes from signal flow
    test_signal_osc: OscillatorNode,
    test_signal_gain: GainNode,
    test_signal_state: TestSignalState,
    
    // Tuning fork nodes from signal flow
    tuning_fork_osc: OscillatorNode,
    tuning_fork_gain: GainNode,
    tuning_fork_state: TuningForkState,
}

/// State tracking for test signal generation
#[derive(Debug, Clone)]
struct TestSignalState {
    enabled: bool,
    frequency: f32,
    amplitude: f32,
    oscillator_started: bool,
}

impl Default for TestSignalState {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency: 440.0,
            amplitude: 0.0,
            oscillator_started: false,
        }
    }
}

/// State tracking for tuning fork generation
#[derive(Debug, Clone)]
struct TuningForkState {
    frequency: f32,
    volume: f32,
    oscillator_started: bool,
}

impl Default for TuningForkState {
    fn default() -> Self {
        Self {
            frequency: 440.0,
            volume: 0.0,
            oscillator_started: false,
        }
    }
}

impl AudioGenerator {
    /// Creates a new AudioGenerator
    /// 
    /// # Parameters
    /// - `test_signal_osc`: Test signal oscillator from AudioSignalFlow
    /// - `test_signal_gain`: Test signal gain node from AudioSignalFlow
    /// - `tuning_fork_osc`: Tuning fork oscillator from AudioSignalFlow
    /// - `tuning_fork_gain`: Tuning fork gain node from AudioSignalFlow
    /// 
    /// # Returns
    /// Result containing the configured AudioGenerator or error description
    pub fn new(
        test_signal_osc: OscillatorNode,
        test_signal_gain: GainNode,
        tuning_fork_osc: OscillatorNode,
        tuning_fork_gain: GainNode,
    ) -> Result<Self, String> {
        dev_log!("Creating AudioGenerator with signal flow oscillator nodes");
        
        Ok(Self {
            test_signal_osc,
            test_signal_gain,
            test_signal_state: TestSignalState::default(),
            tuning_fork_osc,
            tuning_fork_gain,
            tuning_fork_state: TuningForkState::default(),
        })
    }

    /// Update test signal configuration
    /// 
    /// This method handles enabling/disabling test signals and updating their parameters.
    /// It manages the oscillator lifecycle and ensures proper audio routing.
    /// 
    /// # Parameters
    /// - `config`: Test signal configuration including frequency, amplitude, and enabled state
    pub fn update_test_signal_config(&mut self, config: SignalGeneratorConfig) {
        dev_log!("Updating test signal config - enabled: {}, frequency: {} Hz, amplitude: {}", 
                config.enabled, config.frequency, config.amplitude);
        
        if config.enabled {
            // Configure oscillator parameters
            self.test_signal_osc.frequency().set_value(config.frequency);
            self.test_signal_gain.gain().set_value(config.amplitude);
            
            // Start the oscillator if not already started
            if !self.test_signal_state.oscillator_started {
                if let Err(e) = self.test_signal_osc.start() {
                    dev_log!("Test signal oscillator might already be started: {:?}", e);
                } else {
                    self.test_signal_state.oscillator_started = true;
                    dev_log!("✓ Test signal oscillator started");
                }
            }
            
            // Update state
            self.test_signal_state.enabled = true;
            self.test_signal_state.frequency = config.frequency;
            self.test_signal_state.amplitude = config.amplitude;
            
            dev_log!("✓ Test signal enabled: {} Hz, amplitude: {}", config.frequency, config.amplitude);
            
        } else {
            // Disable by setting gain to zero (keep oscillator running for performance)
            self.test_signal_gain.gain().set_value(0.0);
            self.test_signal_state.enabled = false;
            self.test_signal_state.amplitude = 0.0;
            
            dev_log!("✓ Test signal disabled");
        }
    }

    /// Update tuning fork configuration
    /// 
    /// This method handles tuning fork generation and parameter updates.
    /// It manages the oscillator lifecycle for the tuning fork audio.
    /// 
    /// # Parameters
    /// - `config`: Tuning fork configuration including frequency and volume
    pub fn update_tuning_fork_config(&mut self, config: TuningForkConfig) {
        dev_log!("Updating tuning fork config - frequency: {} Hz, volume: {}", 
                config.frequency, config.volume);
        
        // Configure oscillator parameters
        self.tuning_fork_osc.frequency().set_value(config.frequency);
        self.tuning_fork_gain.gain().set_value(config.volume);
        
        if config.volume > 0.0 {
            // Start the oscillator if not already started
            if !self.tuning_fork_state.oscillator_started {
                if let Err(e) = self.tuning_fork_osc.start() {
                    dev_log!("Tuning fork oscillator might already be started: {:?}", e);
                } else {
                    self.tuning_fork_state.oscillator_started = true;
                    dev_log!("✓ Tuning fork oscillator started");
                }
            }
            
            dev_log!("✓ Tuning fork enabled: {} Hz, volume: {}", config.frequency, config.volume);
            
        } else {
            dev_log!("✓ Tuning fork disabled (volume: 0.0)");
        }
        
        // Update state
        self.tuning_fork_state.frequency = config.frequency;
        self.tuning_fork_state.volume = config.volume;
    }

    /// Get current test signal state
    /// 
    /// Returns information about the current test signal configuration and state.
    /// 
    /// # Returns
    /// Current test signal state information
    pub fn get_test_signal_state(&self) -> &TestSignalState {
        &self.test_signal_state
    }

    /// Get current tuning fork state
    /// 
    /// Returns information about the current tuning fork configuration and state.
    /// 
    /// # Returns
    /// Current tuning fork state information  
    pub fn get_tuning_fork_state(&self) -> &TuningForkState {
        &self.tuning_fork_state
    }

    /// Check if test signal is currently enabled
    /// 
    /// # Returns
    /// True if test signal is enabled and generating audio
    pub fn is_test_signal_enabled(&self) -> bool {
        self.test_signal_state.enabled
    }

    /// Check if tuning fork is currently enabled
    /// 
    /// # Returns
    /// True if tuning fork volume is greater than 0
    pub fn is_tuning_fork_enabled(&self) -> bool {
        self.tuning_fork_state.volume > 0.0
    }

    /// Disable all audio generation
    /// 
    /// This is a convenience method to quickly disable both test signal and tuning fork
    /// without stopping the oscillators (for performance).
    pub fn disable_all_generation(&mut self) {
        // Disable test signal
        self.test_signal_gain.gain().set_value(0.0);
        self.test_signal_state.enabled = false;
        self.test_signal_state.amplitude = 0.0;
        
        // Disable tuning fork
        self.tuning_fork_gain.gain().set_value(0.0);
        self.tuning_fork_state.volume = 0.0;
        
        dev_log!("✓ All audio generation disabled");
    }

    /// Stop all oscillators and cleanup
    /// 
    /// This method stops all oscillators and prepares the generator for disposal.
    /// Note: Once stopped, oscillators cannot be restarted and must be recreated.
    /// 
    /// # Returns
    /// Result indicating success or failure of the cleanup
    pub fn disconnect(&mut self) -> Result<(), AudioError> {
        // Stop oscillators (this is final - they cannot be restarted)
        if self.test_signal_state.oscillator_started {
            let _ = self.test_signal_osc.stop();
            self.test_signal_state.oscillator_started = false;
            dev_log!("✓ Test signal oscillator stopped");
        }
        
        if self.tuning_fork_state.oscillator_started {
            let _ = self.tuning_fork_osc.stop();
            self.tuning_fork_state.oscillator_started = false;
            dev_log!("✓ Tuning fork oscillator stopped");
        }
        
        // Reset state
        self.test_signal_state.enabled = false;
        self.test_signal_state.amplitude = 0.0;
        self.tuning_fork_state.volume = 0.0;
        
        dev_log!("✓ AudioGenerator disconnected and cleaned up");
        Ok(())
    }

    /// Get reference to test signal oscillator node
    /// 
    /// Provides access to the underlying Web Audio API OscillatorNode for advanced use cases.
    /// 
    /// # Returns
    /// Reference to the test signal OscillatorNode from signal flow
    pub fn get_test_signal_oscillator(&self) -> &OscillatorNode {
        &self.test_signal_osc
    }

    /// Get reference to test signal gain node
    /// 
    /// Provides access to the underlying Web Audio API GainNode for advanced use cases.
    /// 
    /// # Returns
    /// Reference to the test signal GainNode from signal flow
    pub fn get_test_signal_gain(&self) -> &GainNode {
        &self.test_signal_gain
    }

    /// Get reference to tuning fork oscillator node
    /// 
    /// Provides access to the underlying Web Audio API OscillatorNode for advanced use cases.
    /// 
    /// # Returns
    /// Reference to the tuning fork OscillatorNode from signal flow
    pub fn get_tuning_fork_oscillator(&self) -> &OscillatorNode {
        &self.tuning_fork_osc
    }

    /// Get reference to tuning fork gain node
    /// 
    /// Provides access to the underlying Web Audio API GainNode for advanced use cases.
    /// 
    /// # Returns
    /// Reference to the tuning fork GainNode from signal flow
    pub fn get_tuning_fork_gain(&self) -> &GainNode {
        &self.tuning_fork_gain
    }
}

impl Drop for AudioGenerator {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}