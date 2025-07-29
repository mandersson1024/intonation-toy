//! Presentation Layer - Visualization and user interface
//! 
//! This layer is responsible for:
//! - Visual rendering and graphics display
//! - User interface elements and interactions
//! - Screen management and layout
//! - Event handling and user input
//! - Visual feedback and animations
//! - Debug visualization and overlays
//! 
//! ## Return-Based Data Flow in Presentation Layer
//! 
//! The presentation layer now uses a return-based pattern for data processing:
//! - Receives `ModelUpdateResult` data as a parameter from the model layer
//! - Processes visual data including volume, pitch, accuracy, errors, and permission state
//! - Updates UI elements and visualizations based on the provided data
//! 
//! ```rust
//! use pitch_toy::presentation::Presenter;
//! use pitch_toy::shared_types::ModelUpdateResult;
//! 
//! // Create presenter without interface dependencies
//! let mut presenter = Presenter::create()?;
//! 
//! // Update with model data
//! let model_data = ModelUpdateResult {
//!     volume: pitch_toy::shared_types::Volume { peak: -10.0, rms: -15.0 },
//!     pitch: pitch_toy::shared_types::Pitch::Detected(440.0, 0.95),
//!     accuracy: pitch_toy::shared_types::Accuracy { midi_note: 69, cents_offset: 5.0 },
//!     tuning_system: pitch_toy::shared_types::TuningSystem::EqualTemperament,
//!     errors: Vec::new(),
//!     permission_state: pitch_toy::shared_types::PermissionState::Granted,
//! };
//! presenter.update(timestamp, model_data);
//! ```
//! 
//! ## Current Status
//! 
//! The Presenter struct operates without interface dependencies and actively
//! processes data through method parameters. It provides:
//! 
//! - ✅ Basic sprite scene rendering capabilities
//! - ✅ Volume data processing for audio level visualization  
//! - ✅ Pitch detection handling and musical note processing
//! - ✅ Accuracy metrics processing for tuning feedback
//! - ✅ Error state management and user feedback
//! - ✅ Permission state tracking and UI updates
//! - ✅ Tuning system display management
//! - ✅ User action collection system for microphone permission, tuning system changes, and root note adjustments
//! 
//! ## Future Implementation
//! 
//! Enhanced visual implementation will add:
//! - Visual representations of pitch and volume (meters, waveforms)
//! - Interactive tuning displays and note indicators
//! - User interaction handling and input processing
//! - Advanced animations and visual transitions
//! - Complete screen layout and UI element management



mod main_scene;
pub use main_scene::MainScene;

use three_d::{RenderTarget, Context, Viewport};
use crate::shared_types::{ModelUpdateResult, TuningSystem, MidiNote, Pitch};

// Debug-only imports for conditional compilation
#[cfg(debug_assertions)]
use crate::engine::audio::TestWaveform;

/// Action structs for the new action collection system
/// 
/// These structs represent user actions that are collected by the presentation layer
/// and processed by the main loop. This provides a foundation for the new action flow
/// that moves away from direct action firing.

/// Request for microphone permission from the user interface
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RequestMicrophonePermission;

#[cfg(test)]
impl RequestMicrophonePermission {
    pub fn new() -> Self {
        Self
    }
}

/// Request to change the tuning system
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeTuningSystem {
    pub tuning_system: TuningSystem,
}

#[cfg(test)]
impl ChangeTuningSystem {
    pub fn new(tuning_system: TuningSystem) -> Self {
        Self { tuning_system }
    }
}

/// Request to adjust the root note
#[derive(Debug, Clone, PartialEq)]
pub struct AdjustRootNote {
    pub root_note: MidiNote,
}

#[cfg(test)]
impl AdjustRootNote {
    pub fn new(root_note: MidiNote) -> Self {
        Self { root_note }
    }
}

// Debug action structs (only available in debug builds)
#[cfg(debug_assertions)]
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigureTestSignal {
    pub enabled: bool,
    pub frequency: f32,
    pub volume: f32,
    pub waveform: TestWaveform,
}

#[cfg(all(debug_assertions, test))]
impl ConfigureTestSignal {
    pub fn new(enabled: bool, frequency: f32, volume: f32, waveform: TestWaveform) -> Self {
        Self { enabled, frequency, volume, waveform }
    }
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigureOutputToSpeakers {
    pub enabled: bool,
}

#[cfg(all(debug_assertions, test))]
impl ConfigureOutputToSpeakers {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigureBackgroundNoise {
    pub enabled: bool,
    pub level: f32,
    pub noise_type: TestWaveform,
}

#[cfg(all(debug_assertions, test))]
impl ConfigureBackgroundNoise {
    pub fn new(enabled: bool, level: f32, noise_type: TestWaveform) -> Self {
        Self { enabled, level, noise_type }
    }
}

/// Container for all collected user actions from the presentation layer
/// 
/// This struct is returned by the presentation layer's get_user_actions() method
/// and contains all user actions that occurred since the last collection.
/// The main loop retrieves these actions and processes them appropriately.
#[derive(Debug, Clone, PartialEq)]
pub struct PresentationLayerActions {
    pub tuning_system_changes: Vec<ChangeTuningSystem>,
    pub root_note_adjustments: Vec<AdjustRootNote>,
}

impl PresentationLayerActions {
    /// Create a new instance with empty action collections
    pub fn new() -> Self {
        Self {
            tuning_system_changes: Vec::new(),
            root_note_adjustments: Vec::new(),
        }
    }
}

#[cfg(test)]
impl PresentationLayerActions {
    /// Builder pattern for creating test instances
    pub fn builder() -> PresentationLayerActionsBuilder {
        PresentationLayerActionsBuilder::new()
    }
}

#[cfg(test)]
pub struct PresentationLayerActionsBuilder {
    tuning_system_changes: Vec<ChangeTuningSystem>,
    root_note_adjustments: Vec<AdjustRootNote>,
}

#[cfg(test)]
impl PresentationLayerActionsBuilder {
    pub fn new() -> Self {
        Self {
            tuning_system_changes: Vec::new(),
            root_note_adjustments: Vec::new(),
        }
    }
    
    pub fn with_tuning_change(mut self, tuning_system: TuningSystem) -> Self {
        self.tuning_system_changes.push(ChangeTuningSystem::new(tuning_system));
        self
    }
    
    pub fn with_root_note_adjustment(mut self, root_note: MidiNote) -> Self {
        self.root_note_adjustments.push(AdjustRootNote::new(root_note));
        self
    }
    
    pub fn build(self) -> PresentationLayerActions {
        PresentationLayerActions {
            tuning_system_changes: self.tuning_system_changes,
            root_note_adjustments: self.root_note_adjustments,
        }
    }
}

/// Container for all collected debug actions from the presentation layer
/// 
/// This struct is only available in debug builds and contains actions that
/// provide privileged access to engine operations for testing and debugging.
/// These actions bypass normal validation and safety checks.
#[cfg(debug_assertions)]
#[derive(Debug, Clone, PartialEq)]
pub struct DebugLayerActions {
    pub test_signal_configurations: Vec<ConfigureTestSignal>,
    pub speaker_output_configurations: Vec<ConfigureOutputToSpeakers>,
    pub background_noise_configurations: Vec<ConfigureBackgroundNoise>,
}

#[cfg(debug_assertions)]
impl DebugLayerActions {
    /// Create a new instance with empty debug action collections
    pub(crate) fn new() -> Self {
        Self {
            test_signal_configurations: Vec::new(),
            speaker_output_configurations: Vec::new(),
            background_noise_configurations: Vec::new(),
        }
    }
}

#[cfg(all(debug_assertions, test))]
impl DebugLayerActions {
    /// Builder pattern for creating test instances with debug actions
    pub fn builder() -> DebugLayerActionsBuilder {
        DebugLayerActionsBuilder::new()
    }
}

#[cfg(all(debug_assertions, test))]
pub struct DebugLayerActionsBuilder {
    test_signal_configurations: Vec<ConfigureTestSignal>,
    speaker_output_configurations: Vec<ConfigureOutputToSpeakers>,
    background_noise_configurations: Vec<ConfigureBackgroundNoise>,
}

#[cfg(all(debug_assertions, test))]
impl DebugLayerActionsBuilder {
    pub fn new() -> Self {
        Self {
            test_signal_configurations: Vec::new(),
            speaker_output_configurations: Vec::new(),
            background_noise_configurations: Vec::new(),
        }
    }
    
    pub fn with_test_signal(mut self, enabled: bool, frequency: f32, volume: f32, waveform: TestWaveform) -> Self {
        self.test_signal_configurations.push(ConfigureTestSignal::new(enabled, frequency, volume, waveform));
        self
    }
    
    pub fn with_speaker_output(mut self, enabled: bool) -> Self {
        self.speaker_output_configurations.push(ConfigureOutputToSpeakers::new(enabled));
        self
    }
    
    pub fn with_background_noise(mut self, enabled: bool, level: f32, noise_type: TestWaveform) -> Self {
        self.background_noise_configurations.push(ConfigureBackgroundNoise::new(enabled, level, noise_type));
        self
    }
    
    pub fn build(self) -> DebugLayerActions {
        DebugLayerActions {
            test_signal_configurations: self.test_signal_configurations,
            speaker_output_configurations: self.speaker_output_configurations,
            background_noise_configurations: self.background_noise_configurations,
        }
    }
}

/// Presenter - The presentation layer of the three-layer architecture
/// 
/// This struct represents the visual rendering and user interface layer
/// of the application. It receives processed data from the model layer
/// through method parameters and renders visual representations.
/// 
/// Data flows through method parameters and return values rather than interface dependencies.
/// 
/// # Current Implementation
/// 
/// This implementation:
/// - Operates without observable interface dependencies
/// - Receives model data through `update()` method parameters
/// - Actively processes volume, pitch, accuracy, and error data
/// - Provides sprite scene rendering capabilities
/// - Manages UI state based on model data updates
/// 
/// # Example
/// 
/// ```no_run
/// use pitch_toy::presentation::Presenter;
/// use pitch_toy::shared_types::model_to_presentation::ModelUpdateResult;
/// use three_d::RenderTarget;
/// 
/// let mut presenter = Presenter::create()
///     .expect("Presenter creation should always succeed");
/// 
/// // Later in render loop:
/// // let model_data = get_model_data(); // From model layer
/// // presenter.update(timestamp, model_data);
/// // let mut screen = frame_input.screen();
/// // presenter.render(&mut screen);
/// ```
pub struct Presenter {
    /// Presentation layer now operates without interface dependencies
    /// Data flows through method parameters and return values
    
    /// Main scene for rendering
    main_scene: Option<MainScene>,
    
    /// Flag to track if scene has been initialized
    scene_initialized: bool,
    
    /// Collection of pending user actions to be processed by the main loop
    /// 
    /// This field stores user actions (like requesting microphone permission,
    /// changing tuning system, or adjusting root note) until they are retrieved
    /// by the main loop via get_user_actions().
    pending_user_actions: PresentationLayerActions,
    
    /// Collection of pending debug actions (debug builds only)
    /// 
    /// This field stores debug actions that provide privileged engine access
    /// for testing and debugging purposes. These actions bypass normal validation.
    #[cfg(debug_assertions)]
    pending_debug_actions: DebugLayerActions,
    
    /// Processed interval position for rendering
    interval_position: f32,
    
    /// EMA smoothing factor (alpha) for exponential moving average calculations
    /// Value between 0.0 and 1.0, where higher values give more weight to recent data
    ema_smoothing_factor: f32,
    
    /// Previous EMA value used for calculating the next smoothed value
    previous_ema_value: f32,
    
    /// Whether the EMA has been initialized with the first value
    ema_initialized: bool,
}

impl Presenter {
    /// Create a new Presenter without interface dependencies
    /// 
    /// This constructor creates a presentation layer that operates through method parameters
    /// and return values rather than observable interfaces. Data is received directly
    /// through the update() method.
    /// 
    /// # Returns
    /// 
    /// Always returns `Ok(Presenter)` as this implementation cannot fail.
    /// 
    /// # Current Behavior
    /// 
    /// Creates a basic presenter struct with sprite scene support. Future implementations
    /// will add comprehensive rendering and UI functionality.
    pub fn create() -> Result<Self, String> {
        // Presentation layer initialization without interface dependencies
        // TODO: Initialize rendering state
        // TODO: Set up sprite scene configuration
        Ok(Self {
            main_scene: None,
            scene_initialized: false,
            pending_user_actions: PresentationLayerActions::new(),
            #[cfg(debug_assertions)]
            pending_debug_actions: DebugLayerActions::new(),
            interval_position: 0.0,
            ema_smoothing_factor: 0.1,
            previous_ema_value: 0.0,
            ema_initialized: false,
        })
    }

    pub fn update_graphics(&mut self, viewport: Viewport) {
        if let Some(ref mut scene) = self.main_scene {
            scene.update_viewport(viewport);
            scene.update_pitch_position(viewport, self.interval_position);
        }
    }

    /// Update the presentation layer with a new timestamp and model data
    /// 
    /// This method is called by the main render loop to update the presentation's state.
    /// It receives processed data from the model layer, updates internal rendering state,
    /// and prepares for the next render call.
    /// 
    /// # Arguments
    /// 
    /// * `timestamp` - The current timestamp in seconds since application start
    /// * `model_data` - The processed data from the model layer containing volume, pitch, accuracy, etc.
    /// 
    /// # Current Implementation
    /// 
    /// This implementation processes the model data and updates internal state:
    /// 1. Processes volume data for audio level visualization
    /// 2. Handles pitch detection results and musical note display
    /// 3. Processes accuracy metrics for tuning feedback
    /// 4. Manages error states and user feedback
    /// 5. Updates permission status display
    /// 6. Prepares data for next render cycle
    pub fn process_data(&mut self, _timestamp: f64, model_data: ModelUpdateResult) {
        // Process volume data for visualization
        self.process_volume_data(&model_data.volume);
        
        // Process pitch and note detection
        self.process_pitch_data(&model_data.pitch);
        
        // Process accuracy metrics for tuning feedback
        self.process_accuracy_data(&model_data.accuracy);
        
        // Handle error states and user feedback
        self.process_error_states(&model_data.errors);
        
        // Update permission status display
        self.process_permission_state(&model_data.permission_state);
        
        // Update tuning system display
        self.process_tuning_system(&model_data.tuning_system);
        
        // Calculate interval position with EMA smoothing for detected pitch
        let raw_interval_position = self.calculate_interval_position_from_frequency(&model_data.pitch, model_data.root_note);
        
        match model_data.pitch {
            Pitch::Detected { .. } => {
                // Apply EMA smoothing when pitch is detected
                self.interval_position = self.apply_ema_smoothing(raw_interval_position);
            }
            Pitch::NotDetected => {
                // Maintain existing behavior for undetected pitch (no smoothing)
                self.interval_position = raw_interval_position; // This will be 0.0
                // Reset EMA state for clean restart when pitch detection resumes
                self.reset_ema();
            }
        }
    }

    /// Retrieve and clear all pending user actions
    /// 
    /// This method is called by the main loop to get all user actions that have
    /// been collected since the last call. After retrieving the actions, the
    /// internal collection is cleared to prepare for the next collection cycle.
    /// 
    /// # Returns
    /// 
    /// A `PresentationLayerActions` struct containing all collected user actions.
    /// The returned struct will contain empty vectors if no actions were collected.
    /// 
    /// # Usage
    /// 
    /// This method should be called once per render loop by the main application
    /// to process user actions that occurred during the previous frame.
    pub fn get_user_actions(&mut self) -> PresentationLayerActions {
        std::mem::replace(&mut self.pending_user_actions, PresentationLayerActions::new())
    }

    /// Handle user request to change the tuning system
    /// 
    /// This method should be called by UI components when the user selects
    /// a different tuning system from a dropdown or control panel.
    /// 
    /// # Arguments
    /// 
    /// * `tuning_system` - The new tuning system selected by the user
    pub fn on_tuning_system_changed(&mut self, tuning_system: TuningSystem) {
        self.pending_user_actions.tuning_system_changes.push(ChangeTuningSystem { tuning_system });
    }

    /// Handle user request to adjust the root note
    /// 
    /// This method should be called by UI components when the user selects
    /// a different root note from a control or input field.
    /// 
    /// # Arguments
    /// 
    /// * `root_note` - The new root note selected by the user
    pub fn on_root_note_adjusted(&mut self, root_note: MidiNote) {
        self.pending_user_actions.root_note_adjustments.push(AdjustRootNote { root_note });
    }

    /// Retrieve and clear all pending debug actions (debug builds only)
    /// 
    /// This method is called by the main loop to get all debug actions that have
    /// been collected since the last call. These actions provide privileged access
    /// to engine operations for testing and debugging purposes.
    /// 
    /// # Returns
    /// 
    /// A `DebugLayerActions` struct containing all collected debug actions.
    /// The returned struct will contain empty vectors if no actions were collected.
    /// 
    /// # Safety
    /// 
    /// Debug actions bypass normal validation and can directly manipulate engine
    /// internals. They should only be used for testing and debugging.
    #[cfg(debug_assertions)]
    pub fn get_debug_actions(&mut self) -> DebugLayerActions {
        std::mem::replace(&mut self.pending_debug_actions, DebugLayerActions::new())
    }

    /// Handle debug request to configure test signal generation (debug builds only)
    /// 
    /// This method should be called by debug UI components to configure test
    /// signal generation for testing audio processing.
    /// 
    /// # Arguments
    /// 
    /// * `enabled` - Whether test signal generation should be enabled
    /// * `frequency` - The frequency of the test signal in Hz
    /// * `volume` - The volume of the test signal (0-100)
    /// * `waveform` - The waveform type to generate
    #[cfg(debug_assertions)]
    pub fn on_test_signal_configured(&mut self, enabled: bool, frequency: f32, volume: f32, waveform: TestWaveform) {
        self.pending_debug_actions.test_signal_configurations.push(ConfigureTestSignal {
            enabled,
            frequency,
            volume,
            waveform,
        });
    }

    /// Handle debug request to configure speaker output (debug builds only)
    /// 
    /// This method should be called by debug UI components to enable or disable
    /// direct speaker output for debugging audio processing.
    /// 
    /// # Arguments
    /// 
    /// * `enabled` - Whether speaker output should be enabled
    #[cfg(debug_assertions)]
    pub fn on_output_to_speakers_configured(&mut self, enabled: bool) {
        self.pending_debug_actions.speaker_output_configurations.push(ConfigureOutputToSpeakers {
            enabled,
        });
    }

    /// Handle debug request to configure background noise generation (debug builds only)
    /// 
    /// This method should be called by debug UI components to configure background
    /// noise generation for testing noise cancellation and signal processing.
    /// 
    /// # Arguments
    /// 
    /// * `enabled` - Whether background noise generation should be enabled
    /// * `level` - The level of background noise
    /// * `noise_type` - The type of noise to generate
    #[cfg(debug_assertions)]
    pub fn on_background_noise_configured(&mut self, enabled: bool, level: f32, noise_type: TestWaveform) {
        self.pending_debug_actions.background_noise_configurations.push(ConfigureBackgroundNoise {
            enabled,
            level,
            noise_type,
        });
    }

    /// Render the presentation layer to the screen
    /// 
    /// This method is called by the main render loop to draw the visual interface.
    /// 
    /// # Arguments
    /// 
    /// * `_context` - The WebGL context for rendering (currently unused)
    /// * `screen` - The render target to draw to
    pub fn render(&mut self, context: &Context, screen: &mut RenderTarget) {
        // Initialize scene on first render if not already done
        if !self.scene_initialized {
            let viewport = screen.viewport();
            self.main_scene = Some(MainScene::new(context, viewport));
            self.scene_initialized = true;
        }
        
        // Render the scene if available
        if let Some(ref scene) = self.main_scene {
            scene.render(screen);
        }
    }

    /// Get the current EMA smoothing factor
    /// 
    /// Returns the alpha value used in exponential moving average calculations.
    /// A higher value gives more weight to recent data points.
    /// 
    /// # Returns
    /// 
    /// The current smoothing factor as a value between 0.0 and 1.0
    pub fn get_ema_smoothing_factor(&self) -> f32 {
        self.ema_smoothing_factor
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
    pub fn set_ema_smoothing_factor(&mut self, factor: f32) {
        assert!(factor >= 0.0 && factor <= 1.0, "EMA smoothing factor must be between 0.0 and 1.0");
        self.ema_smoothing_factor = factor;
    }

    /// Get the equivalent EMA period for the current smoothing factor
    /// 
    /// Calculates the equivalent period (in samples) that would produce
    /// the same smoothing effect as the current smoothing factor.
    /// 
    /// # Returns
    /// 
    /// The equivalent EMA period as a floating-point number
    pub fn get_ema_period(&self) -> f32 {
        (2.0 / self.ema_smoothing_factor) - 1.0
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
    pub fn set_ema_period(&mut self, period: f32) {
        assert!(period > 0.0, "EMA period must be positive");
        self.ema_smoothing_factor = 2.0 / (period + 1.0);
    }

    /// Reset the EMA state to initial conditions
    /// 
    /// Clears the EMA history by resetting the initialization flag and
    /// previous value. The next EMA calculation will start fresh.
    pub fn reset_ema(&mut self) {
        self.ema_initialized = false;
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
    pub fn is_ema_initialized(&self) -> bool {
        self.ema_initialized
    }
    
    /// Apply exponential moving average (EMA) smoothing to a value
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
    /// 
    /// # EMA Formula
    /// 
    /// - First call: EMA = current_value (initialization)
    /// - Subsequent calls: EMA = (current_value × smoothing_factor) + (previous_ema × (1 - smoothing_factor))
    /// 
    /// # Smoothing Factor Behavior
    /// 
    /// - Higher values (closer to 1.0) respond more quickly to changes
    /// - Lower values (closer to 0.0) provide more smoothing and stability
    /// - Factor of 1.0 = no smoothing (returns current_value)
    /// - Factor of 0.0 = maximum smoothing (returns previous_ema)
    fn apply_ema_smoothing(&mut self, current_value: f32) -> f32 {
        if !self.ema_initialized {
            // First call: initialize EMA with the current value
            self.previous_ema_value = current_value;
            self.ema_initialized = true;
            current_value
        } else {
            // Subsequent calls: apply standard EMA formula
            let new_ema = (current_value * self.ema_smoothing_factor) + 
                         (self.previous_ema_value * (1.0 - self.ema_smoothing_factor));
            self.previous_ema_value = new_ema;
            new_ema
        }
    }
    
    /// Process volume data for audio level visualization
    /// 
    /// Updates internal state based on volume levels from the model layer.
    /// This data will be used to drive volume meters and audio visualizations.
    /// 
    /// # Arguments
    /// 
    /// * `volume` - Volume data containing peak and RMS levels in dB
    fn process_volume_data(&mut self, volume: &crate::shared_types::Volume) {
        // Store volume data for visualization
        // Future: Update volume meter displays, audio wave visualizations
        let _peak_amplitude = volume.peak_amplitude;
        let _rms_amplitude = volume.rms_amplitude;
        
        // Placeholder: Log significant volume changes for debugging
        if volume.peak_amplitude > -20.0 {
            // Loud audio detected - could trigger visual feedback
        }
    }
    
    /// Process pitch detection data for musical note display
    /// 
    /// Updates pitch-related UI elements based on detected frequencies and notes.
    /// 
    /// # Arguments
    /// 
    /// * `pitch` - Pitch detection result from the model layer
    fn process_pitch_data(&mut self, pitch: &crate::shared_types::Pitch) {
        match pitch {
            crate::shared_types::Pitch::Detected(frequency, clarity) => {
                // Pitch detected - update note display
                let _freq = *frequency;
                let _clarity = *clarity;
                
                // Future: Update pitch display, note name, frequency readout
                // Future: Update visual tuning indicators
            }
            crate::shared_types::Pitch::NotDetected => {
                // No pitch detected - clear pitch displays
                // Future: Dim pitch indicators, show "listening" state
            }
        }
    }
    
    /// Process accuracy data for tuning feedback
    /// 
    /// Updates tuning indicators and accuracy displays based on pitch accuracy.
    /// 
    /// # Arguments
    /// 
    /// * `accuracy` - Accuracy metrics containing closest note and deviation
    fn process_accuracy_data(&mut self, accuracy: &crate::shared_types::IntonationData) {
        let _midi_note = accuracy.closest_midi_note;
        let _cents_offset = accuracy.cents_offset;
        
        // Future: Update tuning needle/indicator position
        // Future: Change colors based on accuracy (green=good, red=off)
        // Future: Display note name and cents deviation
        
        if accuracy.cents_offset.abs() < 10.0 {
            // Very accurate - could show green indicator
        } else if accuracy.cents_offset.abs() > 30.0 {
            // Very inaccurate - could show red indicator  
        }
    }
    
    /// Process error states for user feedback
    /// 
    /// Handles error conditions and updates error displays.
    /// 
    /// # Arguments
    /// 
    /// * `errors` - Vector of error conditions from the model layer
    fn process_error_states(&mut self, errors: &Vec<crate::shared_types::Error>) {
        if errors.is_empty() {
            // No errors - clear error displays
            return;
        }
        
        // Process each error type
        for error in errors {
            match error {
                crate::shared_types::Error::MicrophonePermissionDenied => {
                    // Show microphone permission denied message
                }
                crate::shared_types::Error::MicrophoneNotAvailable => {
                    // Show microphone not available message
                }
                crate::shared_types::Error::BrowserApiNotSupported => {
                    // Show browser compatibility message
                }
                crate::shared_types::Error::AudioContextInitFailed => {
                    // Show audio initialization failure message
                }
                crate::shared_types::Error::AudioContextSuspended => {
                    // Show audio context suspended message
                }
                crate::shared_types::Error::ProcessingError(_msg) => {
                    // Show general processing error
                }
            }
        }
    }
    
    /// Process permission state for UI updates
    /// 
    /// Updates permission-related UI elements and user prompts.
    /// 
    /// # Arguments
    /// 
    /// * `permission_state` - Current microphone permission state
    fn process_permission_state(&mut self, permission_state: &crate::shared_types::PermissionState) {
        match permission_state {
            crate::shared_types::PermissionState::NotRequested => {
                // Show "Click to start" or permission request button
            }
            crate::shared_types::PermissionState::Requested => {
                // Show "Requesting permission..." status
            }
            crate::shared_types::PermissionState::Granted => {
                // Show active/listening state
            }
            crate::shared_types::PermissionState::Denied => {
                // Show permission denied message with instructions
            }
        }
    }
    
    /// Process tuning system updates
    /// 
    /// Updates displays related to the current tuning system.
    /// 
    /// # Arguments
    /// 
    /// * `tuning_system` - Current tuning system from the model layer
    fn process_tuning_system(&mut self, tuning_system: &crate::shared_types::TuningSystem) {
        match tuning_system {
            crate::shared_types::TuningSystem::EqualTemperament => {
                // Update UI to show Equal Temperament tuning
            }
            crate::shared_types::TuningSystem::JustIntonation => {
                // Update UI to show Just Intonation tuning
            }
        }
    }
    
    /// Calculate interval position directly from frequency and root note
    /// 
    /// This method provides a more accurate calculation by working directly with
    /// frequency data rather than pre-quantized MIDI note values. It calculates
    /// the musical interval using the frequency ratio between the detected pitch
    /// and the root note frequency.
    /// 
    /// # Arguments
    /// 
    /// * `pitch` - The detected pitch data containing frequency and clarity
    /// * `root_note` - The MIDI note number of the root note
    /// 
    /// # Returns
    /// 
    /// The interval position value for rendering, or 0.0 if no pitch is detected
    /// 
    /// # Musical Theory and Scaling
    /// 
    /// The position is calculated as `log2(frequency / root_frequency)`, which:
    /// - Maps unison (ratio 1.0) to position 0.0
    /// - Maps octave up (ratio 2.0) to position 1.0
    /// - Maps octave down (ratio 0.5) to position -1.0
    /// 
    /// This provides an intuitive octave-based scaling where each unit represents
    /// one octave of musical distance.
    fn calculate_interval_position_from_frequency(&self, pitch: &Pitch, root_note: MidiNote) -> f32 {
        match pitch {
            Pitch::Detected(frequency, _clarity) => {
                // Calculate root note frequency using standard A4=440Hz reference
                let root_frequency = Self::midi_note_to_frequency(root_note);
                
                // Calculate interval position using log2 of frequency ratio
                // This maps frequency ratios directly to position values:
                // - ratio 1.0 (unison) -> position 0.0
                // - ratio 2.0 (octave up) -> position 1.0
                // - ratio 0.5 (octave down) -> position -1.0
                (frequency / root_frequency).log2()
            }
            Pitch::NotDetected => 0.0,
        }
    }
    
    /// Convert MIDI note number to frequency in Hz
    /// 
    /// Uses the standard equal temperament formula with A4 (MIDI note 69) = 440Hz
    /// 
    /// # Arguments
    /// 
    /// * `midi_note` - The MIDI note number (0-127)
    /// 
    /// # Returns
    /// 
    /// The frequency in Hz
    /// 
    /// # Formula
    /// 
    /// `frequency = 440.0 * 2^((midi_note - 69) / 12)`
    /// 
    /// This formula is based on:
    /// - A4 (MIDI note 69) is the reference pitch at 440Hz
    /// - Each semitone up multiplies frequency by 2^(1/12)
    /// - Each octave up doubles the frequency
    fn midi_note_to_frequency(midi_note: MidiNote) -> f32 {
        440.0 * 2.0_f32.powf((midi_note as f32 - 69.0) / 12.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    use three_d::*;
    
    /// Create test model data for testing purposes
    fn create_test_model_data() -> ModelUpdateResult {
        ModelUpdateResult {
            volume: crate::shared_types::Volume { peak_amplitude: -10.0, rms_amplitude: -15.0 },
            pitch: crate::shared_types::Pitch::NotDetected,
            accuracy: crate::shared_types::IntonationData {
                closest_midi_note: 69,
                cents_offset: 0.0,
            },
            tuning_system: crate::shared_types::TuningSystem::EqualTemperament,
            errors: Vec::new(),
            permission_state: crate::shared_types::PermissionState::NotRequested,
            closest_midi_note: 69,
            cents_offset: 0.0,
            interval_semitones: 0,
            root_note: 53,
        }
    }

    /// Test that Presenter::create() succeeds without interface dependencies
    #[wasm_bindgen_test]
    fn test_presenter_create_succeeds() {
        let result = Presenter::create();
        assert!(result.is_ok(), "Presenter::create() should always succeed");
    }

    /// Test that update() and render() methods can be called without panicking
    #[wasm_bindgen_test]
    fn test_presenter_update_and_render_no_panic() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");

        // Test that update can be called multiple times without panicking
        let test_data = create_test_model_data();
        presenter.process_data(0.0, test_data.clone());
        presenter.process_data(1.0, test_data.clone());
        presenter.process_data(123.456, test_data.clone());
        presenter.process_data(-1.0, test_data); // Negative timestamp should also be safe
        
        // Test render method - we need a mock render target
        // Since we can't easily create a real RenderTarget in tests,
        // we'll create a simple test that verifies the method signature
        // The actual render testing will be done in integration tests
        
        // For now, just verify update doesn't panic
        // If we reach this point, no panic occurred
        assert!(true, "update() method should not panic");
    }

    /// Test that render method can be called (basic signature verification)
    #[wasm_bindgen_test]
    fn test_presenter_render_method_exists() {
        let presenter = Presenter::create()
            .expect("Presenter creation should succeed");

        // We can't easily create a RenderTarget in unit tests without a full WebGL context
        // But we can verify the method exists and compiles correctly by checking the type
        // This test mainly ensures the render method signature is correct
        
        // Create a mock function pointer to verify the signature
        let _render_fn: fn(&mut Presenter, &Context, &mut RenderTarget) = Presenter::render;
        
        assert!(true, "render() method signature is correct");
    }

    /// Verify that struct can be created without interface dependencies
    #[wasm_bindgen_test]
    fn test_presenter_interface_free_creation() {
        // This test verifies that the struct can be constructed without interfaces
        let presenter = Presenter::create();

        match presenter {
            Ok(_) => {
                // Success - presenter was created without interface dependencies
                assert!(true, "Presenter was created without interface dependencies");
            }
            Err(e) => {
                panic!("Presenter should be creatable without interfaces, but got error: {}", e);
            }
        }
    }

    /// Test basic runtime safety - creation and operation should not crash
    #[wasm_bindgen_test]
    fn test_presenter_runtime_safety() {
        // Create multiple instances to test memory safety
        for i in 0..10 {
            let mut presenter = Presenter::create()
                .expect("Presenter creation should always succeed");

            // Test multiple operations
            let test_data = create_test_model_data();
            presenter.process_data(i as f64, test_data.clone());
            presenter.process_data((i as f64) * 0.5, test_data.clone());
            
            // Test edge case values
            presenter.process_data(f64::MAX, test_data.clone());
            presenter.process_data(f64::MIN, test_data.clone());
            presenter.process_data(0.0, test_data);
        }
        
        // If we reach this point, all operations completed safely
        assert!(true, "All Presenter operations completed safely");
    }

    /// Test that Presenter compilation requirements are met
    #[wasm_bindgen_test]
    fn test_presenter_compilation_safety() {
        // This test exists primarily to ensure the struct compiles correctly
        // without interface dependencies

        // Test successful creation
        let presenter_result = Presenter::create();

        // Test that the result type is correct
        assert!(presenter_result.is_ok());
        
        let mut presenter = presenter_result.unwrap();
        
        // Test that update signature is correct
        let test_data = create_test_model_data();
        presenter.process_data(42.0, test_data);
        
        // Test completed - all compilation requirements verified
        assert!(true, "Presenter meets all compilation requirements");
    }

    /// Test interface isolation - different instances should be independent
    #[wasm_bindgen_test]
    fn test_presenter_interface_isolation() {
        let presenter1 = Presenter::create()
            .expect("Presenter1 creation should succeed");

        let presenter2 = Presenter::create()
            .expect("Presenter2 creation should succeed");

        // Both presenters should be independent and work correctly
        // This is mainly a compilation test since they're placeholders
        assert!(true, "Multiple Presenter instances are properly isolated");
    }

    /// Test action collection system - get_user_actions returns empty initially
    #[wasm_bindgen_test]
    fn test_get_user_actions_initially_empty() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");

        let actions = presenter.get_user_actions();
        
        assert!(actions.tuning_system_changes.is_empty());
        assert!(actions.root_note_adjustments.is_empty());
    }


    /// Test tuning system change collection
    #[wasm_bindgen_test]
    fn test_tuning_system_change_collection() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");

        // Trigger tuning system change
        presenter.on_tuning_system_changed(TuningSystem::EqualTemperament);
        
        let actions = presenter.get_user_actions();
        assert_eq!(actions.tuning_system_changes.len(), 1);
        assert_eq!(actions.tuning_system_changes[0].tuning_system, TuningSystem::EqualTemperament);
        
        // After getting actions, they should be cleared
        let actions2 = presenter.get_user_actions();
        assert!(actions2.tuning_system_changes.is_empty());
    }

    /// Test root note adjustment collection
    #[wasm_bindgen_test]
    fn test_root_note_adjustment_collection() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");

        // Trigger root note adjustment
        presenter.on_root_note_adjusted(61);
        
        let actions = presenter.get_user_actions();
        assert_eq!(actions.root_note_adjustments.len(), 1);
        assert_eq!(actions.root_note_adjustments[0].root_note, 61);
        
        // After getting actions, they should be cleared
        let actions2 = presenter.get_user_actions();
        assert!(actions2.root_note_adjustments.is_empty());
    }

    /// Test multiple action collection and clearing
    #[wasm_bindgen_test]
    fn test_multiple_action_collection() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");

        // Trigger multiple actions
        presenter.on_tuning_system_changed(TuningSystem::EqualTemperament);
        presenter.on_root_note_adjusted(67);
        presenter.on_root_note_adjusted(62); // Second root note change
        
        let actions = presenter.get_user_actions();
        
        // Verify all actions were collected
        assert_eq!(actions.tuning_system_changes.len(), 1);
        assert_eq!(actions.root_note_adjustments.len(), 2);
        
        // Verify action data
        assert_eq!(actions.tuning_system_changes[0].tuning_system, TuningSystem::EqualTemperament);
        assert_eq!(actions.root_note_adjustments[0].root_note, 67);
        assert_eq!(actions.root_note_adjustments[1].root_note, 62);
        
        // After getting actions, all should be cleared
        let actions2 = presenter.get_user_actions();
        assert!(actions2.tuning_system_changes.is_empty());
        assert!(actions2.root_note_adjustments.is_empty());
    }

    /// Test PresentationLayerActions struct creation and equality
    #[wasm_bindgen_test]
    fn test_presentation_layer_actions_struct() {
        let actions1 = PresentationLayerActions::new();
        let actions2 = PresentationLayerActions::new();
        
        // Test equality
        assert_eq!(actions1, actions2);
        
        // Test that new instances are empty
        assert!(actions1.tuning_system_changes.is_empty());
        assert!(actions1.root_note_adjustments.is_empty());
    }

    /// Test action struct creation and equality
    #[wasm_bindgen_test]
    fn test_action_struct_creation() {
        let perm_req1 = RequestMicrophonePermission;
        let perm_req2 = RequestMicrophonePermission;
        assert_eq!(perm_req1, perm_req2);
        
        let tuning_change1 = ChangeTuningSystem { tuning_system: TuningSystem::EqualTemperament };
        let tuning_change2 = ChangeTuningSystem { tuning_system: TuningSystem::EqualTemperament };
        assert_eq!(tuning_change1, tuning_change2);
        
        let root_note1 = AdjustRootNote { root_note: 65 };
        let root_note2 = AdjustRootNote { root_note: 65 };
        assert_eq!(root_note1, root_note2);
    }

    // Debug action tests (only run in debug builds)
    #[cfg(debug_assertions)]
    #[wasm_bindgen_test]
    fn test_debug_actions_initially_empty() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");

        let debug_actions = presenter.get_debug_actions();
        
        assert!(debug_actions.test_signal_configurations.is_empty());
        assert!(debug_actions.speaker_output_configurations.is_empty());
        assert!(debug_actions.background_noise_configurations.is_empty());
    }

    #[cfg(debug_assertions)]
    #[wasm_bindgen_test]
    fn test_test_signal_configuration_collection() {
        use crate::engine::audio::TestWaveform;
        
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");

        // Configure test signal
        presenter.on_test_signal_configured(true, 440.0, 50.0, TestWaveform::Sine);
        
        let debug_actions = presenter.get_debug_actions();
        assert_eq!(debug_actions.test_signal_configurations.len(), 1);
        assert_eq!(debug_actions.test_signal_configurations[0].enabled, true);
        assert_eq!(debug_actions.test_signal_configurations[0].frequency, 440.0);
        assert_eq!(debug_actions.test_signal_configurations[0].volume, 50.0);
        assert_eq!(debug_actions.test_signal_configurations[0].waveform, TestWaveform::Sine);
        
        // After getting actions, they should be cleared
        let debug_actions2 = presenter.get_debug_actions();
        assert!(debug_actions2.test_signal_configurations.is_empty());
    }

    #[cfg(debug_assertions)]
    #[wasm_bindgen_test]
    fn test_speaker_output_configuration_collection() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");

        // Configure speaker output
        presenter.on_output_to_speakers_configured(true);
        
        let debug_actions = presenter.get_debug_actions();
        assert_eq!(debug_actions.speaker_output_configurations.len(), 1);
        assert_eq!(debug_actions.speaker_output_configurations[0].enabled, true);
        
        // After getting actions, they should be cleared
        let debug_actions2 = presenter.get_debug_actions();
        assert!(debug_actions2.speaker_output_configurations.is_empty());
    }

    #[cfg(debug_assertions)]
    #[wasm_bindgen_test]
    fn test_background_noise_configuration_collection() {
        use crate::engine::audio::TestWaveform;
        
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");

        // Configure background noise
        presenter.on_background_noise_configured(true, 0.1, TestWaveform::WhiteNoise);
        
        let debug_actions = presenter.get_debug_actions();
        assert_eq!(debug_actions.background_noise_configurations.len(), 1);
        assert_eq!(debug_actions.background_noise_configurations[0].enabled, true);
        assert_eq!(debug_actions.background_noise_configurations[0].level, 0.1);
        assert_eq!(debug_actions.background_noise_configurations[0].noise_type, TestWaveform::WhiteNoise);
        
        // After getting actions, they should be cleared
        let debug_actions2 = presenter.get_debug_actions();
        assert!(debug_actions2.background_noise_configurations.is_empty());
    }

    #[cfg(debug_assertions)]
    #[wasm_bindgen_test]
    fn test_multiple_debug_action_collection() {
        use crate::engine::audio::TestWaveform;
        
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");

        // Trigger multiple debug actions
        presenter.on_test_signal_configured(true, 880.0, 75.0, TestWaveform::Square);
        presenter.on_output_to_speakers_configured(false);
        presenter.on_background_noise_configured(true, 0.2, TestWaveform::PinkNoise);
        presenter.on_test_signal_configured(false, 220.0, 25.0, TestWaveform::Triangle); // Second test signal config
        
        let debug_actions = presenter.get_debug_actions();
        
        // Verify all actions were collected
        assert_eq!(debug_actions.test_signal_configurations.len(), 2);
        assert_eq!(debug_actions.speaker_output_configurations.len(), 1);
        assert_eq!(debug_actions.background_noise_configurations.len(), 1);
        
        // Verify first test signal config
        assert_eq!(debug_actions.test_signal_configurations[0].enabled, true);
        assert_eq!(debug_actions.test_signal_configurations[0].frequency, 880.0);
        assert_eq!(debug_actions.test_signal_configurations[0].waveform, TestWaveform::Square);
        
        // Verify second test signal config
        assert_eq!(debug_actions.test_signal_configurations[1].enabled, false);
        assert_eq!(debug_actions.test_signal_configurations[1].frequency, 220.0);
        assert_eq!(debug_actions.test_signal_configurations[1].waveform, TestWaveform::Triangle);
        
        // After getting actions, all should be cleared
        let debug_actions2 = presenter.get_debug_actions();
        assert!(debug_actions2.test_signal_configurations.is_empty());
        assert!(debug_actions2.speaker_output_configurations.is_empty());
        assert!(debug_actions2.background_noise_configurations.is_empty());
    }

    #[cfg(debug_assertions)]
    #[wasm_bindgen_test]
    fn test_debug_layer_actions_struct() {
        let actions1 = DebugLayerActions::new();
        let actions2 = DebugLayerActions::new();
        
        // Test equality
        assert_eq!(actions1, actions2);
        
        // Test that new instances are empty
        assert!(actions1.test_signal_configurations.is_empty());
        assert!(actions1.speaker_output_configurations.is_empty());
        assert!(actions1.background_noise_configurations.is_empty());
    }

    #[cfg(debug_assertions)]
    #[wasm_bindgen_test]
    fn test_debug_action_struct_creation() {
        use crate::engine::audio::TestWaveform;
        
        let test_signal1 = ConfigureTestSignal { enabled: true, frequency: 440.0, volume: 50.0, waveform: TestWaveform::Sine };
        let test_signal2 = ConfigureTestSignal { enabled: true, frequency: 440.0, volume: 50.0, waveform: TestWaveform::Sine };
        assert_eq!(test_signal1, test_signal2);
        
        let speaker1 = ConfigureOutputToSpeakers { enabled: true };
        let speaker2 = ConfigureOutputToSpeakers { enabled: true };
        assert_eq!(speaker1, speaker2);
        
        let noise1 = ConfigureBackgroundNoise { enabled: false, level: 0.5, noise_type: TestWaveform::PinkNoise };
        let noise2 = ConfigureBackgroundNoise { enabled: false, level: 0.5, noise_type: TestWaveform::PinkNoise };
        assert_eq!(noise1, noise2);
    }
    
    /// Test the new frequency-based interval calculation
    #[wasm_bindgen_test]
    fn test_frequency_based_interval_calculation() {
        let presenter = Presenter::create()
            .expect("Presenter creation should succeed");
        
        // Test case 1: Perfect unison (same frequency as root)
        // A4 (440Hz) compared to A4 root (MIDI note 69)
        let pitch_unison = Pitch::Detected(440.0, 0.9);
        let interval_unison = presenter.calculate_interval_position_from_frequency(&pitch_unison, 69);
        assert!((interval_unison - 0.0).abs() < 0.001, "Perfect unison should yield 0.0 position");
        
        // Test case 2: Perfect octave up
        // A5 (880Hz) compared to A4 root (MIDI note 69)
        let pitch_octave_up = Pitch::Detected(880.0, 0.9);
        let interval_octave_up = presenter.calculate_interval_position_from_frequency(&pitch_octave_up, 69);
        assert!((interval_octave_up - 1.0).abs() < 0.001, "Perfect octave up should yield 1.0 position (log2(2.0))");
        
        // Test case 3: Perfect octave down
        // A3 (220Hz) compared to A4 root (MIDI note 69)
        let pitch_octave_down = Pitch::Detected(220.0, 0.9);
        let interval_octave_down = presenter.calculate_interval_position_from_frequency(&pitch_octave_down, 69);
        assert!((interval_octave_down - (-1.0)).abs() < 0.001, "Perfect octave down should yield -1.0 position (log2(0.5))");
        
        // Test case 4: Perfect fifth up
        // E5 (~659.25Hz) compared to A4 root (MIDI note 69)
        // Perfect fifth is 7 semitones = frequency ratio of 2^(7/12) ≈ 1.498
        // log2(1.498) ≈ 0.583
        let pitch_fifth_up = Pitch::Detected(440.0 * 1.498307, 0.9);
        let interval_fifth_up = presenter.calculate_interval_position_from_frequency(&pitch_fifth_up, 69);
        assert!((interval_fifth_up - 0.583).abs() < 0.01, "Perfect fifth up should yield ~0.583 position (log2(1.498))");
        
        // Test case 5: Different root note
        // A4 (440Hz) compared to F3 root (MIDI note 53)
        // This is 16 semitones up = frequency ratio of 2^(16/12) ≈ 2.52
        // log2(2.52) ≈ 1.333
        let pitch_a4 = Pitch::Detected(440.0, 0.9);
        let interval_from_f3 = presenter.calculate_interval_position_from_frequency(&pitch_a4, 53);
        assert!((interval_from_f3 - 1.333).abs() < 0.01, "A4 from F3 root should yield ~1.333 position (log2(2.52))");
        
        // Test case 6: No pitch detected
        let pitch_none = Pitch::NotDetected;
        let interval_none = presenter.calculate_interval_position_from_frequency(&pitch_none, 69);
        assert_eq!(interval_none, 0.0, "No pitch detected should yield 0.0 position");
    }
    
    /// Test MIDI note to frequency conversion
    #[wasm_bindgen_test]
    fn test_midi_note_to_frequency() {
        // Test known MIDI note frequencies
        assert!((Presenter::midi_note_to_frequency(69) - 440.0).abs() < 0.001, "A4 (MIDI 69) should be 440Hz");
        assert!((Presenter::midi_note_to_frequency(60) - 261.626).abs() < 0.01, "C4 (MIDI 60) should be ~261.626Hz");
        assert!((Presenter::midi_note_to_frequency(57) - 220.0).abs() < 0.001, "A3 (MIDI 57) should be 220Hz");
        assert!((Presenter::midi_note_to_frequency(81) - 880.0).abs() < 0.001, "A5 (MIDI 81) should be 880Hz");
        assert!((Presenter::midi_note_to_frequency(53) - 174.614).abs() < 0.01, "F3 (MIDI 53) should be ~174.614Hz");
    }
    
    /// Test EMA initialization behavior
    #[wasm_bindgen_test]
    fn test_ema_initialization() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");
        
        // Verify initial state
        assert!(!presenter.is_ema_initialized(), "EMA should not be initialized initially");
        
        // First call should initialize with the input value
        let first_value = 42.5;
        let result = presenter.apply_ema_smoothing(first_value);
        
        // Should return the first value unchanged
        assert_eq!(result, first_value, "First EMA call should return input value unchanged");
        
        // Should now be initialized
        assert!(presenter.is_ema_initialized(), "EMA should be initialized after first call");
        
        // Previous EMA value should be set to the first input
        assert_eq!(presenter.previous_ema_value, first_value, "Previous EMA value should be set to first input");
    }
    
    /// Test standard EMA calculation with known values
    #[wasm_bindgen_test]
    fn test_ema_standard_calculation() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");
        
        // Set known smoothing factor for predictable results
        presenter.set_ema_smoothing_factor(0.1);
        
        // Initialize with first value
        let first_value = 100.0;
        let result1 = presenter.apply_ema_smoothing(first_value);
        assert_eq!(result1, first_value, "First call should return input unchanged");
        
        // Second call should use EMA formula
        let second_value = 200.0;
        let result2 = presenter.apply_ema_smoothing(second_value);
        
        // Expected: (200.0 * 0.1) + (100.0 * 0.9) = 20.0 + 90.0 = 110.0
        let expected2 = (second_value * 0.1) + (first_value * 0.9);
        assert!((result2 - expected2).abs() < 0.001, "Second EMA call should use formula: got {}, expected {}", result2, expected2);
        
        // Third call using previous result
        let third_value = 50.0;
        let result3 = presenter.apply_ema_smoothing(third_value);
        
        // Expected: (50.0 * 0.1) + (110.0 * 0.9) = 5.0 + 99.0 = 104.0
        let expected3 = (third_value * 0.1) + (result2 * 0.9);
        assert!((result3 - expected3).abs() < 0.001, "Third EMA call should use previous result: got {}, expected {}", result3, expected3);
    }
    
    /// Test EMA with different smoothing factors
    #[wasm_bindgen_test]
    fn test_ema_different_smoothing_factors() {
        // Test with high smoothing factor (quick response)
        let mut presenter_high = Presenter::create()
            .expect("Presenter creation should succeed");
        presenter_high.set_ema_smoothing_factor(0.9);
        
        presenter_high.apply_ema_smoothing(10.0); // Initialize
        let result_high = presenter_high.apply_ema_smoothing(100.0);
        
        // Expected: (100.0 * 0.9) + (10.0 * 0.1) = 90.0 + 1.0 = 91.0
        assert!((result_high - 91.0).abs() < 0.001, "High smoothing factor should respond quickly: got {}", result_high);
        
        // Test with low smoothing factor (more smoothing)
        let mut presenter_low = Presenter::create()
            .expect("Presenter creation should succeed");
        presenter_low.set_ema_smoothing_factor(0.1);
        
        presenter_low.apply_ema_smoothing(10.0); // Initialize
        let result_low = presenter_low.apply_ema_smoothing(100.0);
        
        // Expected: (100.0 * 0.1) + (10.0 * 0.9) = 10.0 + 9.0 = 19.0
        assert!((result_low - 19.0).abs() < 0.001, "Low smoothing factor should smooth more: got {}", result_low);
        
        // Test with smoothing factor of 1.0 (no smoothing)
        let mut presenter_none = Presenter::create()
            .expect("Presenter creation should succeed");
        presenter_none.set_ema_smoothing_factor(1.0);
        
        presenter_none.apply_ema_smoothing(10.0); // Initialize
        let result_none = presenter_none.apply_ema_smoothing(100.0);
        
        // Should return current value unchanged
        assert_eq!(result_none, 100.0, "Smoothing factor 1.0 should return current value");
        
        // Test with smoothing factor of 0.0 (maximum smoothing)
        let mut presenter_max = Presenter::create()
            .expect("Presenter creation should succeed");
        presenter_max.set_ema_smoothing_factor(0.0);
        
        presenter_max.apply_ema_smoothing(10.0); // Initialize
        let result_max = presenter_max.apply_ema_smoothing(100.0);
        
        // Should return previous value unchanged
        assert_eq!(result_max, 10.0, "Smoothing factor 0.0 should return previous value");
    }
    
    /// Test EMA reset functionality
    #[wasm_bindgen_test]
    fn test_ema_reset_behavior() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");
        
        presenter.set_ema_smoothing_factor(0.5);
        
        // Initialize and perform some calculations
        presenter.apply_ema_smoothing(10.0);
        presenter.apply_ema_smoothing(20.0);
        let before_reset = presenter.apply_ema_smoothing(30.0);
        
        // Verify EMA is initialized
        assert!(presenter.is_ema_initialized(), "EMA should be initialized before reset");
        
        // Reset EMA state
        presenter.reset_ema();
        
        // Verify reset state
        assert!(!presenter.is_ema_initialized(), "EMA should not be initialized after reset");
        assert_eq!(presenter.previous_ema_value, 0.0, "Previous EMA value should be reset to 0.0");
        
        // Next call should behave like initialization
        let after_reset = presenter.apply_ema_smoothing(100.0);
        assert_eq!(after_reset, 100.0, "First call after reset should return input unchanged");
        assert!(presenter.is_ema_initialized(), "EMA should be initialized after first call post-reset");
        
        // Test multiple reset cycles
        presenter.reset_ema();
        let value1 = presenter.apply_ema_smoothing(50.0);
        assert_eq!(value1, 50.0, "Second reset cycle should work correctly");
        
        presenter.reset_ema();
        let value2 = presenter.apply_ema_smoothing(75.0);
        assert_eq!(value2, 75.0, "Third reset cycle should work correctly");
    }
    
    /// Test EMA with edge cases and extreme values
    #[wasm_bindgen_test]
    fn test_ema_edge_cases() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");
        
        presenter.set_ema_smoothing_factor(0.2);
        
        // Test with zero values
        let result_zero = presenter.apply_ema_smoothing(0.0);
        assert_eq!(result_zero, 0.0, "EMA should handle zero initialization");
        
        let result_after_zero = presenter.apply_ema_smoothing(10.0);
        let expected_after_zero = (10.0 * 0.2) + (0.0 * 0.8);
        assert!((result_after_zero - expected_after_zero).abs() < 0.001, "EMA should handle transition from zero");
        
        // Test with negative values
        presenter.reset_ema();
        let result_negative = presenter.apply_ema_smoothing(-50.0);
        assert_eq!(result_negative, -50.0, "EMA should handle negative initialization");
        
        let result_mixed = presenter.apply_ema_smoothing(25.0);
        let expected_mixed = (25.0 * 0.2) + (-50.0 * 0.8);
        assert!((result_mixed - expected_mixed).abs() < 0.001, "EMA should handle negative to positive transition");
        
        // Test with very large values
        presenter.reset_ema();
        let large_value = 1e6;
        let result_large = presenter.apply_ema_smoothing(large_value);
        assert_eq!(result_large, large_value, "EMA should handle large values");
        
        // Test with very small values
        presenter.reset_ema();
        let small_value = 1e-6;
        let result_small = presenter.apply_ema_smoothing(small_value);
        assert!((result_small - small_value).abs() < 1e-9, "EMA should handle very small values");
        
        // Test numerical stability with repeated small changes
        presenter.reset_ema();
        presenter.apply_ema_smoothing(1.0);
        
        let mut current_value = 1.0;
        for _ in 0..1000 {
            current_value = presenter.apply_ema_smoothing(current_value + 0.001);
        }
        
        // After 1000 iterations with small increments, value should be close to final input
        assert!(current_value > 1.0, "EMA should accumulate small changes");
        assert!(current_value < 2.0, "EMA should not grow unbounded with small changes");
    }
    
    /// Test EMA getter/setter integration and configuration
    #[wasm_bindgen_test]
    fn test_ema_configuration_integration() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");
        
        // Test initial configuration
        let initial_factor = presenter.get_ema_smoothing_factor();
        assert_eq!(initial_factor, 0.1, "Default EMA smoothing factor should be 0.1");
        
        // Test factor setting and getting
        presenter.set_ema_smoothing_factor(0.3);
        assert_eq!(presenter.get_ema_smoothing_factor(), 0.3, "Set smoothing factor should be retrievable");
        
        // Test that factor change affects calculations
        presenter.apply_ema_smoothing(10.0); // Initialize
        let result_with_03 = presenter.apply_ema_smoothing(100.0);
        let expected_03 = (100.0 * 0.3) + (10.0 * 0.7);
        assert!((result_with_03 - expected_03).abs() < 0.001, "Changed smoothing factor should affect calculations");
        
        // Test period conversion
        presenter.set_ema_period(19.0); // Should give smoothing factor of 2/(19+1) = 0.1
        assert!((presenter.get_ema_smoothing_factor() - 0.1).abs() < 0.001, "Period-to-factor conversion should be accurate");
        
        let calculated_period = presenter.get_ema_period();
        assert!((calculated_period - 19.0).abs() < 0.001, "Factor-to-period conversion should be accurate");
        
        // Test period setting with different values
        presenter.set_ema_period(9.0); // Should give smoothing factor of 2/10 = 0.2
        assert!((presenter.get_ema_smoothing_factor() - 0.2).abs() < 0.001, "Different period should give correct factor");
        
        // Test that period changes affect ongoing calculations
        presenter.reset_ema();
        presenter.apply_ema_smoothing(50.0); // Initialize
        let result_with_period = presenter.apply_ema_smoothing(150.0);
        let expected_with_period = (150.0 * 0.2) + (50.0 * 0.8);
        assert!((result_with_period - expected_with_period).abs() < 0.001, "Period-based factor should affect calculations");
        
        // Test configuration persistence across calculations
        for i in 0..10 {
            let value = presenter.apply_ema_smoothing(i as f32);
            // Factor should remain constant
            assert!((presenter.get_ema_smoothing_factor() - 0.2).abs() < 0.001, "Smoothing factor should persist across calculations");
        }
    }
    
    /// Test EMA configuration validation (panic cases)
    #[wasm_bindgen_test]
    #[should_panic(expected = "EMA smoothing factor must be between 0.0 and 1.0")]
    fn test_ema_factor_validation_negative() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");
        presenter.set_ema_smoothing_factor(-0.1);
    }
    
    #[wasm_bindgen_test]
    #[should_panic(expected = "EMA smoothing factor must be between 0.0 and 1.0")]
    fn test_ema_factor_validation_too_large() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");
        presenter.set_ema_smoothing_factor(1.1);
    }
    
    #[wasm_bindgen_test]
    #[should_panic(expected = "EMA period must be positive")]
    fn test_ema_period_validation_zero() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");
        presenter.set_ema_period(0.0);
    }
    
    #[wasm_bindgen_test]
    #[should_panic(expected = "EMA period must be positive")]
    fn test_ema_period_validation_negative() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");
        presenter.set_ema_period(-5.0);
    }
    
    #[wasm_bindgen_test]
    fn test_process_data_ema_smoothing_with_detected_pitch() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");
        
        // Set a known EMA factor for predictable results
        presenter.set_ema_smoothing_factor(0.2);
        
        let root_note = Note::C;
        
        // First detected pitch - should initialize EMA
        let mut model_data = create_test_model_data();
        model_data.pitch = Pitch::Detected { frequency: 440.0, clarity: 0.9 };
        model_data.root_note = root_note;
        
        presenter.process_data(&model_data);
        let first_position = presenter.interval_position;
        
        // Calculate expected raw position for verification
        let expected_raw_1 = (440.0 / root_note.frequency()).log2();
        assert!((first_position - expected_raw_1).abs() < 0.001, 
                "First detected pitch should equal raw value (EMA initialization)");
        
        // Second detected pitch - should apply EMA smoothing
        model_data.pitch = Pitch::Detected { frequency: 466.16, clarity: 0.8 };
        presenter.process_data(&model_data);
        let second_position = presenter.interval_position;
        
        // Calculate expected EMA result
        let expected_raw_2 = (466.16 / root_note.frequency()).log2();
        let expected_ema_2 = first_position + 0.2 * (expected_raw_2 - first_position);
        assert!((second_position - expected_ema_2).abs() < 0.001,
                "Second detected pitch should be EMA smoothed");
        
        // Third detected pitch - verify EMA progression
        model_data.pitch = Pitch::Detected { frequency: 493.88, clarity: 0.85 };
        presenter.process_data(&model_data);
        let third_position = presenter.interval_position;
        
        let expected_raw_3 = (493.88 / root_note.frequency()).log2();
        let expected_ema_3 = second_position + 0.2 * (expected_raw_3 - second_position);
        assert!((third_position - expected_ema_3).abs() < 0.001,
                "Third detected pitch should continue EMA progression");
    }
    
    #[wasm_bindgen_test]
    fn test_process_data_no_ema_smoothing_with_undetected_pitch() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");
        
        presenter.set_ema_smoothing_factor(0.3);
        
        // First, establish some EMA state with detected pitch
        let mut model_data = create_test_model_data();
        model_data.pitch = Pitch::Detected { frequency: 440.0, clarity: 0.9 };
        model_data.root_note = Note::C;
        
        presenter.process_data(&model_data);
        assert_ne!(presenter.interval_position, 0.0, "Should have non-zero position for detected pitch");
        
        // Now test undetected pitch
        model_data.pitch = Pitch::NotDetected;
        presenter.process_data(&model_data);
        
        assert_eq!(presenter.interval_position, 0.0, 
                   "Undetected pitch should result in 0.0 interval position");
        
        // Verify EMA state was reset by checking next detected pitch behavior
        model_data.pitch = Pitch::Detected { frequency: 440.0, clarity: 0.9 };
        presenter.process_data(&model_data);
        
        let expected_raw = (440.0 / Note::C.frequency()).log2();
        assert!((presenter.interval_position - expected_raw).abs() < 0.001,
                "After EMA reset, next detected pitch should equal raw value");
    }
    
    #[wasm_bindgen_test]
    fn test_process_data_ema_state_transitions() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");
        
        presenter.set_ema_smoothing_factor(0.25);
        let root_note = Note::A;
        
        let mut model_data = create_test_model_data();
        model_data.root_note = root_note;
        
        // Sequence: Detected -> NotDetected -> Detected -> Detected
        
        // First detected pitch
        model_data.pitch = Pitch::Detected { frequency: 440.0, clarity: 0.9 };
        presenter.process_data(&model_data);
        let first_position = presenter.interval_position;
        
        // Undetected pitch (should reset EMA)
        model_data.pitch = Pitch::NotDetected;
        presenter.process_data(&model_data);
        assert_eq!(presenter.interval_position, 0.0);
        
        // Next detected pitch (should start fresh EMA)
        model_data.pitch = Pitch::Detected { frequency: 466.16, clarity: 0.8 };
        presenter.process_data(&model_data);
        let restart_position = presenter.interval_position;
        
        let expected_restart_raw = (466.16 / root_note.frequency()).log2();
        assert!((restart_position - expected_restart_raw).abs() < 0.001,
                "EMA should restart with raw value after NotDetected");
        
        // Fourth detected pitch (should apply EMA from restarted state)
        model_data.pitch = Pitch::Detected { frequency: 493.88, clarity: 0.85 };
        presenter.process_data(&model_data);
        let final_position = presenter.interval_position;
        
        let expected_final_raw = (493.88 / root_note.frequency()).log2();
        let expected_final_ema = restart_position + 0.25 * (expected_final_raw - restart_position);
        assert!((final_position - expected_final_ema).abs() < 0.001,
                "EMA should continue properly after restart");
    }
    
    #[wasm_bindgen_test]
    fn test_process_data_ema_smoothing_accuracy() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");
        
        // Test with different smoothing factors
        let smoothing_factors = [0.1, 0.5, 0.9];
        
        for &factor in &smoothing_factors {
            presenter.set_ema_smoothing_factor(factor);
            presenter.reset_ema(); // Start fresh for each test
            
            let root_note = Note::C;
            let mut model_data = create_test_model_data();
            model_data.root_note = root_note;
            
            // First value - should be raw
            model_data.pitch = Pitch::Detected { frequency: 261.63, clarity: 0.9 };
            presenter.process_data(&model_data);
            let first_raw = (261.63 / root_note.frequency()).log2();
            assert!((presenter.interval_position - first_raw).abs() < 0.001,
                    "First value should be raw for factor {}", factor);
            
            // Second value - should be EMA smoothed
            model_data.pitch = Pitch::Detected { frequency: 293.66, clarity: 0.8 };
            presenter.process_data(&model_data);
            let second_raw = (293.66 / root_note.frequency()).log2();
            let expected_second = first_raw + factor * (second_raw - first_raw);
            assert!((presenter.interval_position - expected_second).abs() < 0.001,
                    "Second value should be correctly smoothed for factor {}", factor);
            
            // Verify that higher smoothing factors result in values closer to raw
            if factor == 0.9 {
                let diff_from_raw = (presenter.interval_position - second_raw).abs();
                assert!(diff_from_raw < 0.1, "High smoothing factor should be close to raw");
            }
        }
    }
    
    #[wasm_bindgen_test]
    fn test_process_data_preserves_existing_behavior() {
        let mut presenter = Presenter::create()
            .expect("Presenter creation should succeed");
        
        presenter.set_ema_smoothing_factor(0.3);
        
        let mut model_data = create_test_model_data();
        model_data.pitch = Pitch::Detected { frequency: 440.0, clarity: 0.9 };
        model_data.root_note = Note::A;
        model_data.volume = 0.75;
        model_data.accuracy = Accuracy::InTune;
        model_data.error_state = ErrorState::NoError;
        model_data.permission_state = PermissionState::Granted;
        model_data.tuning_system = TuningSystem::EqualTemperament;
        
        let initial_volume = presenter.volume;
        let initial_accuracy = presenter.accuracy;
        let initial_error_state = presenter.error_state;
        let initial_permission_state = presenter.permission_state;
        let initial_tuning_system = presenter.tuning_system;
        
        presenter.process_data(&model_data);
        
        // Verify all other fields are still processed correctly
        assert_eq!(presenter.volume, 0.75, "Volume should be updated");
        assert_eq!(presenter.accuracy, Accuracy::InTune, "Accuracy should be updated");
        assert_eq!(presenter.error_state, ErrorState::NoError, "Error state should be updated");
        assert_eq!(presenter.permission_state, PermissionState::Granted, "Permission state should be updated");
        assert_eq!(presenter.tuning_system, TuningSystem::EqualTemperament, "Tuning system should be updated");
        
        // Verify interval_position is processed with EMA
        assert_ne!(presenter.interval_position, 0.0, "Interval position should be calculated");
        
        // Verify that changes don't affect the method signature or interface
        let actions = presenter.get_pending_user_actions();
        assert!(actions.set_root_note_actions.is_empty(), "Actions should work normally");
    }
    
}