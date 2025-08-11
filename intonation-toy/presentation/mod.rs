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
pub use main_scene::{MainScene, TuningLines};

mod startup_scene;
pub use startup_scene::StartupScene;

mod smoothing;
pub use smoothing::EmaSmoother;

use std::rc::Rc;
use std::cell::RefCell;
use three_d::{RenderTarget, Context, Viewport};
use crate::shared_types::{ModelUpdateResult, TuningSystem, Scale, MidiNote, Pitch, PermissionState};

#[cfg(target_arch = "wasm32")]
use crate::web::main_scene_ui::{setup_main_scene_ui, cleanup_main_scene_ui, setup_event_listeners};

enum Scene {
    Startup(StartupScene),
    Main(MainScene),
}


/// Action structs for the new action collection system
/// 
/// These structs represent user actions that are collected by the presentation layer
/// and processed by the main loop. This provides a foundation for the new action flow
/// that moves away from direct action firing.


/// Request to change the tuning system
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeTuningSystem {
    pub tuning_system: TuningSystem,
}


/// Request to adjust the root note
#[derive(Debug, Clone, PartialEq)]
pub struct AdjustRootNote {
    pub root_note: MidiNote,
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
pub struct ConfigureRootNoteAudio {
    pub frequency: f32,
    pub volume: f32,
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
    pub scale_changes: Vec<ScaleChangeAction>,
    pub root_note_audio_configurations: Vec<ConfigureRootNoteAudio>,
}

impl PresentationLayerActions {
    /// Create a new instance with empty action collections
    pub fn new() -> Self {
        Self {
            tuning_system_changes: Vec::new(),
            root_note_adjustments: Vec::new(),
            scale_changes: Vec::new(),
            root_note_audio_configurations: Vec::new(),
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
}

#[cfg(debug_assertions)]
impl DebugLayerActions {
    /// Create a new instance with empty debug action collections
    pub(crate) fn new() -> Self {
        Self {
            test_signal_configurations: Vec::new(),
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
    
    /// Current scene (startup or main)
    scene: Scene,
    
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
    
    /// EMA smoother for interval position smoothing
    pub ema_smoother: EmaSmoother,
    
    /// Tracks whether the main scene UI is currently active
    /// Used to manage HTML UI lifecycle during scene transitions
    #[cfg(target_arch = "wasm32")]
    main_scene_ui_active: bool,

    /// Self-reference for passing to UI event handlers
    /// This enables UI elements to call back into the presenter
    #[cfg(target_arch = "wasm32")]
    self_reference: Option<Rc<RefCell<Self>>>,
    
    
    
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
            scene: Scene::Startup(StartupScene::new()),
            pending_user_actions: PresentationLayerActions::new(),
            #[cfg(debug_assertions)]
            pending_debug_actions: DebugLayerActions::new(),
            interval_position: 0.0,
            ema_smoother: EmaSmoother::new(0.1),
            #[cfg(target_arch = "wasm32")]
            main_scene_ui_active: true, // UI is now active from the start
            #[cfg(target_arch = "wasm32")]
            self_reference: None,
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
        
        // Set up event listeners now that we have the self-reference
        setup_event_listeners(self_ref);
    }

    /// No-op version for non-WASM targets
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_self_reference(&mut self, _self_ref: Rc<RefCell<Self>>) {
        // No-op for non-WASM targets
    }

    pub fn update_graphics(&mut self, viewport: Viewport, model_data: &ModelUpdateResult) {
        // Extract values we need before the match to avoid borrowing issues
        let interval_position = self.interval_position;
        let volume_peak = model_data.volume_peak;
        
        // Determine if pitch is detected and extract clarity
        let (pitch_detected, clarity) = match model_data.pitch {
            Pitch::Detected(_, clarity_value) => (true, Some(clarity_value)),
            Pitch::NotDetected => (false, None),
        };
        
        // Extract closest MIDI note from accuracy data when pitch is detected
        let closest_note = if pitch_detected {
            Some(model_data.accuracy.closest_midi_note)
        } else {
            None
        };
        
        // Get tuning line data for the active tuning system
        let tuning_line_data = if matches!(self.scene, Scene::Main(_)) {
            Self::get_tuning_line_positions(
                model_data.root_note,
                model_data.tuning_system,
                model_data.scale,
                viewport
            )
        } else {
            Vec::new()
        };
        
        match &mut self.scene {
            Scene::Startup(_) => {
                // No viewport updates needed for startup scene
            }
            Scene::Main(main_scene) => {
                main_scene.update_viewport(viewport);
                
                // Update volume peak state before updating pitch position
                main_scene.update_volume_peak(volume_peak);
                
                // Update tuning lines - MainScene doesn't know about music theory
                main_scene.update_tuning_lines(viewport, &tuning_line_data);
                
                main_scene.update_closest_note(closest_note);
                
                main_scene.update_pitch_position(viewport, interval_position, pitch_detected, clarity, model_data.accuracy.cents_offset);
            }
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
        
        // Calculate interval position with EMA smoothing for detected pitch
        let raw_interval_position = self.calculate_interval_position_from_frequency(&model_data.pitch, model_data.root_note);
        
        match model_data.pitch {
            Pitch::Detected(_, _) => {
                // Apply EMA smoothing when pitch is detected
                self.interval_position = self.ema_smoother.apply(raw_interval_position);
            }
            Pitch::NotDetected => {
                // Maintain existing behavior for undetected pitch (no smoothing)
                self.interval_position = raw_interval_position; // This will be 0.0
                // Reset EMA state for clean restart when pitch detection resumes
                self.ema_smoother.reset();
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
        self.pending_user_actions.tuning_system_changes.push(ChangeTuningSystem { tuning_system: tuning_system.clone() });
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

    /// Handle scale change action
    /// 
    /// # Arguments
    /// 
    /// * `scale` - The new scale to set
    pub fn on_scale_changed(&mut self, scale: Scale) {
        // Queue scale change action for collection by model layer
        self.pending_user_actions.scale_changes.push(ScaleChangeAction { scale: scale.clone() });
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
    #[cfg(debug_assertions)]
    pub fn on_test_signal_configured(&mut self, enabled: bool, frequency: f32, volume: f32) {
        self.pending_debug_actions.test_signal_configurations.push(ConfigureTestSignal {
            enabled,
            frequency,
            volume,
        });
    }

    /// Handle debug request to configure root note audio generation (debug builds only)
    /// 
    /// This method should be called by debug UI components to enable or disable
    /// root note audio generation for testing and audio reference.
    /// The frequency is automatically calculated from the current root note.
    /// 
    /// # Arguments
    /// 
    /// * `enabled` - Whether root note audio generation should be enabled
    /// * `root_note` - The MIDI note to use as the root note  
    /// * `volume_amplitude` - Volume as amplitude (0.0-1.0)
    pub fn on_root_note_audio_configured(&mut self, _enabled: bool, root_note: MidiNote, volume_amplitude: f32) {
        // Use the amplitude value directly
        let volume = volume_amplitude;
        
        crate::common::dev_log!("PRESENTER: Root note audio configured - root_note: {}, volume: {}", 
                                root_note, volume);
        let frequency = Self::midi_note_to_frequency(root_note);
        self.pending_user_actions.root_note_audio_configurations.push(ConfigureRootNoteAudio {
            frequency,
            volume,
        });
        crate::common::dev_log!("PRESENTER: Added action to pending_user_actions, total actions: {}", self.pending_user_actions.root_note_audio_configurations.len());
    }
    
    /// Handle root note audio configuration with volume control
    /// 
    /// This method should be called by UI components to configure root note audio
    /// with specific volume settings. The volume is provided in decibels for
    /// user-friendly control and converted to amplitude internally.
    /// 
    /// # Arguments
    /// 
    /// * `enabled` - Whether root note audio generation should be enabled
    /// * `root_note` - The MIDI note to use as the root note
    /// * `volume_amplitude` - Volume as amplitude (0.0-1.0)
    pub fn on_root_note_audio_configured_with_volume(&mut self, _enabled: bool, root_note: MidiNote, volume_amplitude: f32) {
        // Use the amplitude value directly
        let volume = volume_amplitude;
        
        crate::common::dev_log!("PRESENTER: Root note audio configured - root_note: {}, volume: {}", 
                                root_note, volume);
        
        let frequency = Self::midi_note_to_frequency(root_note);
        self.pending_user_actions.root_note_audio_configurations.push(ConfigureRootNoteAudio {
            frequency,
            volume,
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
    /// * `model_data` - The current model data containing root note, tuning system, and scale
    pub fn render(&mut self, context: &Context, screen: &mut RenderTarget, model_data: &ModelUpdateResult) {
        // Check if we need to switch from StartupScene to MainScene
        if matches!(self.scene, Scene::Startup(_)) && model_data.permission_state == PermissionState::Granted {
            // Permission was granted - switch to MainScene
            let viewport = screen.viewport();
            match MainScene::new(context, viewport) {
                Ok(main_scene) => self.scene = Scene::Main(main_scene),
                Err(e) => {
                    crate::common::dev_log!("Failed to create MainScene: {}", e);
                    return; // Stay in startup scene
                }
            }
            
            // Set up HTML UI for main scene
            #[cfg(target_arch = "wasm32")]
            {
                // UI was already set up during presenter creation
                
                // Set up event listeners if we have a self-reference
                if let Some(ref self_ref) = self.self_reference {
                    setup_event_listeners(self_ref.clone());
                } else {
                    crate::common::dev_log!("Warning: self_reference not set, UI event listeners not attached");
                }
                
                // Synchronize UI state with current model data values
                self.sync_html_ui(&model_data);
            }
        }
        
        // Delegate rendering to the active scene
        match &mut self.scene {
            Scene::Startup(startup_scene) => {
                startup_scene.render(screen);
            }
            Scene::Main(main_scene) => {
                main_scene.render(screen);
                
                // HTML UI synchronization is handled immediately when state changes,
                // not during render to prevent flickering
            }
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
    /// 
    /// Updates displays related to the current tuning system.
    /// 
    /// # Arguments
    /// 
    /// * `tuning_system` - Current tuning system from the model layer
    fn process_tuning_system(&mut self, tuning_system: &crate::shared_types::TuningSystem) {
        // Process tuning system without caching
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
        crate::theory::tuning::midi_note_to_standard_frequency(midi_note)
    }

    
    /// Get tuning line positions for the active tuning system
    /// Returns only the positions for intervals that are relevant to the current tuning system
    pub fn get_tuning_line_positions(
        root_note: MidiNote,
        tuning_system: TuningSystem,
        scale: Scale,
        viewport: Viewport
    ) -> Vec<(f32, MidiNote, f32)> {
        let root_frequency = crate::theory::tuning::midi_note_to_standard_frequency(root_note);
        
        // Helper function to determine line thickness based on semitone offset
        let get_thickness = |semitone: i32| -> f32 {
            // Octave lines (multiples of 12 semitones) get configurable thickness, others get regular thickness
            if semitone % 12 == 0 {
                crate::app_config::OCTAVE_LINE_THICKNESS
            } else {
                crate::app_config::REGULAR_LINE_THICKNESS
            }
        };
        
        // Show intervals from -12 to +12 semitones including root (0)
        let mut line_data = Vec::new();
        
        // Add center line (root note, 0 semitones)
        if crate::shared_types::semitone_in_scale(scale, 0) {
            // Root frequency stays at interval 0.0 (log2(1) = 0)
            let interval = 0.0;
            let y_position = crate::presentation::main_scene::interval_to_screen_y_position(
                interval,
                viewport.height as f32,
                crate::web::main_scene_ui::get_current_zoom_factor(),
            );
            let thickness = get_thickness(0);
            line_data.push((y_position, root_note, thickness));
        }
        
        // Add intervals above root: +1 to +12 semitones
        for semitone in 1..=12 {
            // Only show intervals that are in the current scale
            if crate::shared_types::semitone_in_scale(scale, semitone) {
                let frequency = crate::theory::tuning::interval_frequency(
                    tuning_system,
                    root_frequency,
                    semitone,
                );
                let interval = (frequency / root_frequency).log2();
                let y_position = crate::presentation::main_scene::interval_to_screen_y_position(
                    interval,
                    viewport.height as f32,
                    crate::web::main_scene_ui::get_current_zoom_factor(),
                );
                let midi_note = (root_note as i32 + semitone).clamp(0, 127) as MidiNote;
                let thickness = get_thickness(semitone);
                line_data.push((y_position, midi_note, thickness));
            }
        }
        
        // Add intervals below root: -12 to -1 semitones
        for semitone in -12..=-1 {
            // Only show intervals that are in the current scale
            if crate::shared_types::semitone_in_scale(scale, semitone) {
                let frequency = crate::theory::tuning::interval_frequency(
                    tuning_system,
                    root_frequency,
                    semitone,
                );
                let interval = (frequency / root_frequency).log2();
                let y_position = crate::presentation::main_scene::interval_to_screen_y_position(
                    interval,
                    viewport.height as f32,
                    crate::web::main_scene_ui::get_current_zoom_factor(),
                );
                let midi_note = (root_note as i32 + semitone).clamp(0, 127) as MidiNote;
                let thickness = get_thickness(semitone);
                line_data.push((y_position, midi_note, thickness));
            }
        }
        
        line_data
    }
    

    /// Convert MIDI note to frequency using specified tuning system and root note.
    /// 
    /// This method calculates frequency based on the tuning system and root note,
    /// enabling proper support for both Equal Temperament and Just Intonation.
    /// 
    /// # Arguments
    /// 
    /// * `midi_note` - The MIDI note number to convert
    /// * `root_note` - The root note for calculating intervals
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
    ///     60, // C as root
    ///     TuningSystem::EqualTemperament
    /// );
    /// ```
    pub fn midi_note_to_frequency_with_tuning(
        &self,
        midi_note: MidiNote,
        root_note: MidiNote,
        tuning_system: TuningSystem,
    ) -> f32 {
        let root_frequency = crate::theory::tuning::midi_note_to_standard_frequency(root_note);
        let interval_semitones = (midi_note as i32) - (root_note as i32);
        crate::theory::tuning::interval_frequency(tuning_system, root_frequency, interval_semitones)
    }


    /// Synchronize HTML UI with specified presenter state
    #[cfg(all(target_arch = "wasm32", debug_assertions))]
    fn sync_html_ui(&self, model_data: &ModelUpdateResult) {
        crate::web::main_scene_ui::sync_ui_with_presenter_state(model_data);
    }
    
    /// Synchronize HTML UI with specified presenter state (non-debug version)
    #[cfg(all(target_arch = "wasm32", not(debug_assertions)))]
    fn sync_html_ui(&self, model_data: &ModelUpdateResult) {
        crate::web::main_scene_ui::sync_ui_with_presenter_state(model_data);
    }

    /// No-op version for non-WASM targets
    #[cfg(not(target_arch = "wasm32"))]
    fn sync_html_ui(&self, _model_data: &ModelUpdateResult) {
        // No-op for non-WASM targets
    }
    
    /// Clean up HTML UI elements if they are currently active
    /// 
    /// This method ensures proper cleanup of DOM elements when the presenter
    /// is dropped or when transitioning away from the main scene.
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

