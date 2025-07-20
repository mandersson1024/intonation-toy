// Debug Actions Interface
// Dedicated interface for debug GUI specific controls and actions

use action::{Action, ActionTrigger, ActionListener};
use crate::engine::audio::TestWaveform;

/// Test signal configuration action
#[derive(Debug, Clone, PartialEq)]
pub struct TestSignalAction {
    pub enabled: bool,
    pub frequency: f32,
    pub volume: f32, // 0-100 percentage
    pub waveform: TestWaveform,
}

/// Output to speakers configuration action
#[derive(Debug, Clone, PartialEq)]
pub struct OutputToSpeakersAction {
    pub enabled: bool,
}

/// Background noise configuration action
#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundNoiseAction {
    pub enabled: bool,
    pub level: f32,
    pub noise_type: TestWaveform, // Using TestWaveform for noise types (WhiteNoise, PinkNoise)
}

/// Debug Actions Interface
/// 
/// This interface provides action triggers for debug-specific controls that should not
/// be part of the core application logic. These actions are typically used by debug
/// GUI components for testing and development purposes.
pub struct DebugActionsInterface {
    test_signal: Action<TestSignalAction>,
    output_to_speakers: Action<OutputToSpeakersAction>,
    background_noise: Action<BackgroundNoiseAction>,
}

impl DebugActionsInterface {
    /// Create a new debug actions interface
    pub fn new() -> Self {
        Self {
            test_signal: Action::new(),
            output_to_speakers: Action::new(),
            background_noise: Action::new(),
        }
    }

    // Test Signal Action Methods
    
    /// Get trigger for test signal actions
    pub fn test_signal_trigger(&self) -> ActionTrigger<TestSignalAction> {
        self.test_signal.trigger()
    }
    
    /// Get listener for test signal actions
    pub fn test_signal_listener(&self) -> ActionListener<TestSignalAction> {
        self.test_signal.listener()
    }

    // Output to Speakers Action Methods
    
    /// Get trigger for output to speakers actions
    pub fn output_to_speakers_trigger(&self) -> ActionTrigger<OutputToSpeakersAction> {
        self.output_to_speakers.trigger()
    }
    
    /// Get listener for output to speakers actions
    pub fn output_to_speakers_listener(&self) -> ActionListener<OutputToSpeakersAction> {
        self.output_to_speakers.listener()
    }

    // Background Noise Action Methods
    
    /// Get trigger for background noise actions
    pub fn background_noise_trigger(&self) -> ActionTrigger<BackgroundNoiseAction> {
        self.background_noise.trigger()
    }
    
    /// Get listener for background noise actions
    pub fn background_noise_listener(&self) -> ActionListener<BackgroundNoiseAction> {
        self.background_noise.listener()
    }
}

impl Default for DebugActionsInterface {
    fn default() -> Self {
        Self::new()
    }
}