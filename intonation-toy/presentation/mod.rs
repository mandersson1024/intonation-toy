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
//! use intonation_toy::presentation::Presenter;
//! use intonation_toy::shared_types::ModelUpdateResult;
//! 
//! // Create presenter without interface dependencies
//! let mut presenter = Presenter::create()?;
//! 
//! // Update with model data
//! let model_data = ModelUpdateResult {
//!     volume: intonation_toy::shared_types::Volume { peak: -10.0, rms: -15.0 },
//!     pitch: intonation_toy::shared_types::Pitch::Detected(440.0, 0.95),
//!     accuracy: intonation_toy::shared_types::Accuracy { midi_note: 69, cents_offset: 5.0 },
//!     tuning_system: intonation_toy::shared_types::TuningSystem::EqualTemperament,
//!     errors: Vec::new(),
//!     permission_state: intonation_toy::shared_types::PermissionState::Granted,
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
//! - ✅ User action collection system for microphone permission, tuning system changes, and tuning fork adjustments
//! 
//! ## Future Implementation
//! 
//! Enhanced visual implementation will add:
//! - Visual representations of pitch and volume (meters, waveforms)
//! - Interactive tuning displays and note indicators
//! - User interaction handling and input processing
//! - Advanced animations and visual transitions
//! - Complete screen layout and UI element management

mod audio_analysis;
mod main_scene;
mod tuning_lines;
mod egui_text_backend;
mod user_pitch_line;
pub use audio_analysis::AudioAnalysis;
pub use main_scene::MainScene;
pub use tuning_lines::TuningLines;
pub use egui_text_backend::EguiTextBackend;
pub use user_pitch_line::UserPitchLine;


use std::rc::Rc;
use std::cell::RefCell;
use three_d::{RenderTarget, Context, Viewport};
use crate::shared_types::{ModelUpdateResult, TuningSystem, Scale, MidiNote, Pitch};

#[cfg(target_arch = "wasm32")]
use crate::web::main_scene_ui::{setup_main_scene_ui, cleanup_main_scene_ui, setup_event_listeners};


/// Action structs for the new action collection system
/// 
/// These structs represent user actions that are collected by the presentation layer
/// and processed by the main loop. This provides a foundation for the new action flow
/// that moves away from direct action firing.
///
/// Request to change the tuning system
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeTuningSystem {
    pub tuning_system: TuningSystem,
}

/// Request to adjust the tuning fork
#[derive(Debug, Clone, PartialEq)]
pub struct AdjustTuningFork {
    pub note: MidiNote,
}

/// Action for changing the active scale
/// 
/// This action represents a user request to change the musical scale used
/// for note filtering and intonation calculations.
#[derive(Debug, Clone, PartialEq)]
pub struct ScaleChangeAction {
    pub scale: Scale,
}

// Debug action structs (only available in debug builds)
#[cfg(debug_assertions)]
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigureTestSignal {
    pub enabled: bool,
    pub frequency: f32,
    pub volume: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigureTuningFork {
    pub frequency: f32,
    pub volume: f32,
}

/// Container for all collected user actions from the presentation layer
/// 
/// This struct is returned by the presentation layer's get_user_actions() method
/// and contains all user actions that occurred since the last collection.
/// The main loop retrieves these actions and processes them appropriately.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PresentationLayerActions {
    pub tuning_system_changes: Vec<ChangeTuningSystem>,
    pub tuning_fork_adjustments: Vec<AdjustTuningFork>,
    pub scale_changes: Vec<ScaleChangeAction>,
    pub tuning_fork_configurations: Vec<ConfigureTuningFork>,
}


/// Container for all collected debug actions from the presentation layer
/// 
/// This struct is only available in debug builds and contains actions that
/// provide privileged access to engine operations for testing and debugging.
/// These actions bypass normal validation and safety checks.
#[cfg(debug_assertions)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DebugLayerActions {
    pub test_signal_configurations: Vec<ConfigureTestSignal>,
}

#[cfg(debug_assertions)]
impl DebugLayerActions {
    /// Create a new instance with empty debug action collections
    pub(crate) fn new() -> Self {
        Self::default()
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
/// use intonation_toy::presentation::Presenter;
/// use intonation_toy::shared_types::model_to_presentation::ModelUpdateResult;
/// use three_d::RenderTarget;
/// 
/// let mut presenter = Presenter::create()
///     .expect("Presenter creation should always succeed");
/// 
/// // Later in render loop:
/// // presenter.update(timestamp, model_data);
/// // let mut screen = frame_input.screen();
/// // presenter.render(&mut screen);
/// ```
pub struct Presenter {
    /// Presentation layer now operates without interface dependencies
    /// Data flows through method parameters and return values
    
    /// Main scene (created once graphics context is available)
    main_scene: Option<Box<MainScene>>,
    
    /// Collection of pending user actions to be processed by the main loop
    /// 
    /// This field stores user actions (like requesting microphone permission,
    /// changing tuning system, or adjusting tuning fork) until they are retrieved
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
    
    /// Tracks whether the main scene UI is currently active
    /// Used to manage HTML UI lifecycle during scene transitions
    #[cfg(target_arch = "wasm32")]
    main_scene_ui_active: bool,

    /// Self-reference for passing to UI event handlers
    /// This enables UI elements to call back into the presenter
    #[cfg(target_arch = "wasm32")]
    self_reference: Option<Rc<RefCell<Self>>>,
    
    /// Tracks whether UI event listeners have been attached
    /// Prevents duplicate listener registration
    #[cfg(target_arch = "wasm32")]
    ui_listeners_attached: bool,
    
    
    
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
        
        // Set up HTML UI for sidebar immediately so it's available during startup scene
        #[cfg(target_arch = "wasm32")]
        {
            setup_main_scene_ui();
        }
        
        Ok(Self {
            main_scene: None,
            pending_user_actions: PresentationLayerActions::default(),
            #[cfg(debug_assertions)]
            pending_debug_actions: DebugLayerActions::new(),
            interval_position: 0.0,
            #[cfg(target_arch = "wasm32")]
            main_scene_ui_active: true, // UI is now active from the start
            #[cfg(target_arch = "wasm32")]
            self_reference: None,
            #[cfg(target_arch = "wasm32")]
            ui_listeners_attached: false,
        })
    }

    /// Set the self-reference for UI event handling
    /// 
    /// This method should be called after creating the presenter and wrapping it
    /// in Rc<RefCell<>> to enable UI event handlers to call back into the presenter.
    /// 
    /// # Arguments
    /// 
    /// * `self_ref` - The Rc<RefCell<>> wrapped presenter reference
    #[cfg(target_arch = "wasm32")]
    pub fn set_self_reference(&mut self, self_ref: Rc<RefCell<Self>>) {
        self.self_reference = Some(self_ref.clone());
        
        // Set up event listeners if not already attached
        if !self.ui_listeners_attached {
            setup_event_listeners(self_ref);
            self.ui_listeners_attached = true;
        }
    }

    /// No-op version for non-WASM targets
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_self_reference(&mut self, _self_ref: Rc<RefCell<Self>>) {
        // No-op for non-WASM targets
    }

    pub fn update_graphics(&mut self, viewport: Viewport, model_data: &ModelUpdateResult) {
        let (pitch_detected, clarity) = match model_data.pitch {
            Pitch::Detected(_, clarity_value) => (true, Some(clarity_value)),
            Pitch::NotDetected => (false, None),
        };
        
        
        if let Some(main_scene) = &mut self.main_scene {
            main_scene.update_presentation_context(&crate::shared_types::PresentationContext {
                tuning_fork_note: model_data.tuning_fork_note,
                tuning_system: model_data.tuning_system,
                current_scale: model_data.scale,
            }, viewport);
            
            main_scene.update_audio_analysis(AudioAnalysis {
                pitch_detected,
                cents_offset: model_data.accuracy.cents_offset,
                interval: self.interval_position,
                clarity,
                volume_peak: model_data.volume_peak,
            });
            
            main_scene.update_pitch_position(viewport);
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
        
        // Update tuning system display
        self.process_tuning_system(&model_data.tuning_system);
        
        // Sync HTML UI with updated state
        self.sync_html_ui(&model_data);
        
        self.interval_position = self.calculate_interval_position_from_frequency(&model_data.pitch, model_data.tuning_fork_note);
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
        std::mem::take(&mut self.pending_user_actions)
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

    /// Handle user request to adjust the tuning fork
    /// 
    /// This method should be called by UI components when the user selects
    /// a different tuning fork from a control or input field.
    /// 
    /// # Arguments
    /// 
    /// * `tuning_fork` - The new tuning fork selected by the user
    pub fn on_tuning_fork_adjusted(&mut self, note: MidiNote) {
        self.pending_user_actions.tuning_fork_adjustments.push(AdjustTuningFork { note });
    }

    /// Handle scale change action
    /// 
    /// # Arguments
    /// 
    /// * `scale` - The new scale to set
    pub fn on_scale_changed(&mut self, scale: Scale) {
        // Queue scale change action for collection by model layer
        self.pending_user_actions.scale_changes.push(ScaleChangeAction { scale });
    }

    /// Retrieve and clear all pending debug actions (debug builds only)
    #[cfg(debug_assertions)]
    pub fn get_debug_actions(&mut self) -> DebugLayerActions {
        std::mem::take(&mut self.pending_debug_actions)
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
    #[cfg(debug_assertions)]
    pub fn on_test_signal_configured(&mut self, enabled: bool, frequency: f32, volume: f32) {
        self.pending_debug_actions.test_signal_configurations.push(ConfigureTestSignal {
            enabled,
            frequency,
            volume,
        });
    }

    /// Handle debug request to configure tuning fork audio generation (debug builds only)
    pub fn on_tuning_fork_configured(&mut self, _enabled: bool, note: MidiNote, volume_amplitude: f32) {
        crate::common::dev_log!("PRESENTER: Tuning fork audio configured - tuning_fork: {}, volume: {}", 
                                note, volume_amplitude);
        
        self.pending_user_actions.tuning_fork_configurations.push(ConfigureTuningFork {
            frequency: Self::midi_note_to_frequency(note),
            volume: volume_amplitude,
        });
        crate::common::dev_log!("PRESENTER: Added action to pending_user_actions, total actions: {}", self.pending_user_actions.tuning_fork_configurations.len());
    }
    
    /// Handle tuning fork audio configuration with volume control
    pub fn on_tuning_fork_audio_configured_with_volume(&mut self, _enabled: bool, note: MidiNote, volume_amplitude: f32) {
        crate::common::dev_log!("PRESENTER: Tuning fork audio configured - tuning_fork: {}, volume: {}", 
                                note, volume_amplitude);
        
        self.pending_user_actions.tuning_fork_configurations.push(ConfigureTuningFork {
            frequency: Self::midi_note_to_frequency(note),
            volume: volume_amplitude,
        });
        crate::common::dev_log!("PRESENTER: Added action to pending_user_actions with volume control");
    }

    /// Render the presentation layer to the screen
    /// 
    /// This method is called by the main render loop to draw the visual interface.
    /// 
    /// # Arguments
    /// 
    /// * `_context` - The WebGL context for rendering (currently unused)
    /// * `screen` - The render target to draw to
    /// * `model_data` - The current model data containing tuning fork, tuning system, and scale
    pub fn render(&mut self, context: &Context, screen: &mut RenderTarget, model_data: &ModelUpdateResult) {
        if self.main_scene.is_none() {
            let main_scene = match MainScene::new(context, screen.viewport()) {
                Ok(scene) => scene,
                Err(e) => {
                    crate::common::dev_log!("Failed to create MainScene: {}", e);
                    screen.clear(three_d::ClearState::color(0.0, 0.0, 0.0, 1.0));
                    return;
                }
            };
            
            self.main_scene = Some(Box::new(main_scene));
            self.update_graphics(screen.viewport(), model_data);
            
            #[cfg(target_arch = "wasm32")]
            self.sync_html_ui(model_data);
        }
        
        let viewport = screen.viewport();
        
        if self.main_scene.is_some() {
            self.update_graphics(viewport, model_data);
        }
        
        if let Some(main_scene) = &mut self.main_scene {
            main_scene.render(screen, viewport);
        } else {
            screen.clear(three_d::ClearState::color(0.0, 0.0, 0.0, 1.0));
        }
    }

    
    /// Process volume data for audio level visualization
    fn process_volume_data(&mut self, volume: &crate::shared_types::Volume) {
        if volume.peak_amplitude > -20.0 {
            // Loud audio detected - could trigger visual feedback
        }
    }
    
    /// Process pitch detection data for musical note display
    fn process_pitch_data(&mut self, _pitch: &crate::shared_types::Pitch) {
        // Future: Update pitch display, note name, frequency readout
        // Future: Update visual tuning indicators
    }
    
    /// Process accuracy data for tuning feedback
    fn process_accuracy_data(&mut self, accuracy: &crate::shared_types::IntonationData) {
        if accuracy.cents_offset.abs() < crate::app_config::INTONATION_ACCURACY_THRESHOLD {
            // Very accurate - main scene will show accent color for user pitch line
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
                    // Show microphone not available message - this is a critical error
                    crate::web::error_message_box::show_error(&crate::shared_types::Error::MicrophoneNotAvailable);
                }
                crate::shared_types::Error::BrowserApiNotSupported => {
                    // Show browser compatibility message
                }
                crate::shared_types::Error::ProcessingError(msg) => {
                    // Show general processing error
                    crate::common::error_log!("🔥 PROCESSING ERROR: {}", msg);
                }
                crate::shared_types::Error::MobileDeviceNotSupported => {
                    // Show mobile device not supported message
                    crate::web::error_message_box::show_error(&crate::shared_types::Error::MobileDeviceNotSupported);
                }
                crate::shared_types::Error::BrowserError => {
                    // Show browser error message
                    crate::web::error_message_box::show_error(&crate::shared_types::Error::BrowserError);
                }
            }
        }
    }
    
    /// Process tuning system updates
    fn process_tuning_system(&mut self, _tuning_system: &crate::shared_types::TuningSystem) {
        // Future: Update UI to show current tuning system
    }
    
    /// Calculate interval position directly from frequency and tuning fork
    /// 
    /// This method provides a more accurate calculation by working directly with
    /// frequency data rather than pre-quantized MIDI note values. It calculates
    /// the musical interval using the frequency ratio between the detected pitch
    /// and the tuning fork frequency.
    /// 
    /// # Arguments
    /// 
    /// * `pitch` - The detected pitch data containing frequency and clarity
    /// * `tuning_fork` - The MIDI note number of the tuning fork
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
    fn calculate_interval_position_from_frequency(&self, pitch: &Pitch, note: MidiNote) -> f32 {
        match pitch {
            Pitch::Detected(frequency, _clarity) => {
                // Calculate tuning fork frequency using standard A4=440Hz reference
                let tuning_fork_frequency = Self::midi_note_to_frequency(note);
                
                // Calculate interval position using log2 of frequency ratio
                // This maps frequency ratios directly to position values:
                // - ratio 1.0 (unison) -> position 0.0
                // - ratio 2.0 (octave up) -> position 1.0
                // - ratio 0.5 (octave down) -> position -1.0
                (frequency / tuning_fork_frequency).log2()
            }
            Pitch::NotDetected => 0.0,
        }
    }
    
    /// Convert MIDI note number to frequency in Hz
    fn midi_note_to_frequency(midi_note: MidiNote) -> f32 {
        crate::music_theory::midi_note_to_standard_frequency(midi_note)
    }

    
    

    /// Convert MIDI note to frequency using specified tuning system and tuning fork.
    /// 
    /// This method calculates frequency based on the tuning system and tuning fork,
    /// enabling proper support for both Equal Temperament and Just Intonation.
    /// 
    /// # Arguments
    /// 
    /// * `midi_note` - The MIDI note number to convert
    /// * `tuning_fork` - The tuning fork for calculating intervals
    /// * `tuning_system` - The tuning system to use (Equal Temperament or Just Intonation)
    /// 
    /// # Returns
    /// 
    /// The frequency in Hz according to the specified tuning system
    /// 
    /// # Example
    /// 
    /// ```ignore
    /// let frequency = presenter.midi_note_to_frequency_with_tuning(
    ///     60, // Middle C
    ///     60, // C as tuning fork
    ///     TuningSystem::EqualTemperament
    /// );
    /// ```
    pub fn midi_note_to_frequency_with_tuning(
        &self,
        midi_note: MidiNote,
        note: MidiNote,
        tuning_system: TuningSystem,
    ) -> f32 {
        let tuning_fork_frequency = crate::music_theory::midi_note_to_standard_frequency(note);
        let interval_semitones = (midi_note as i32) - (note as i32);
        crate::music_theory::interval_frequency(tuning_system, tuning_fork_frequency, interval_semitones)
    }

    /// Synchronize HTML UI with specified presenter state
    #[cfg(target_arch = "wasm32")]
    fn sync_html_ui(&self, model_data: &ModelUpdateResult) {
        crate::web::main_scene_ui::sync_ui_with_presenter_state(model_data);
    }

    /// No-op version for non-WASM targets
    #[cfg(not(target_arch = "wasm32"))]
    fn sync_html_ui(&self, _model_data: &ModelUpdateResult) {
        // No-op for non-WASM targets
    }
    
    /// Clean up HTML UI elements if they are currently active
    #[cfg(target_arch = "wasm32")]
    fn cleanup_main_scene_ui_if_active(&mut self) {
        if self.main_scene_ui_active {
            cleanup_main_scene_ui();
            self.main_scene_ui_active = false;
        }
    }

    /// No-op version for non-WASM targets
    #[cfg(not(target_arch = "wasm32"))]
    fn cleanup_main_scene_ui_if_active(&mut self) {
        // No-op for non-WASM targets
    }
}

impl Drop for Presenter {
    fn drop(&mut self) {
        // Clean up HTML UI elements if active
        self.cleanup_main_scene_ui_if_active();
    }
}

