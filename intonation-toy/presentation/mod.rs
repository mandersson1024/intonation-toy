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
//! 

mod audio_analysis;
mod renderer;
mod tuning_lines;
mod egui_text_backend;
mod user_pitch_line;
pub use audio_analysis::AudioAnalysis;
pub use renderer::Renderer;
pub use tuning_lines::TuningLines;
pub use egui_text_backend::EguiTextBackend;
pub use user_pitch_line::UserPitchLine;


use std::rc::Rc;
use std::cell::RefCell;
use three_d::{RenderTarget, Context, Viewport};
use crate::shared_types::{ModelUpdateResult, TuningSystem, Scale, MidiNote, Pitch};

#[cfg(target_arch = "wasm32")]
use crate::web::sidebar_controls::{setup_sidebar_controls, cleanup_sidebar_controls, setup_event_listeners};


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
#[derive(Debug, Clone, PartialEq)]
pub struct ScaleChangeAction {
    pub scale: Scale,
}

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
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PresentationLayerActions {
    pub tuning_system_changes: Vec<ChangeTuningSystem>,
    pub tuning_fork_adjustments: Vec<AdjustTuningFork>,
    pub scale_changes: Vec<ScaleChangeAction>,
    pub tuning_fork_configurations: Vec<ConfigureTuningFork>,
}


/// Container for all collected debug actions from the presentation layer
#[cfg(debug_assertions)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DebugLayerActions {
    pub test_signal_configurations: Vec<ConfigureTestSignal>,
}

#[cfg(debug_assertions)]
impl DebugLayerActions {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

/// Presenter - The presentation layer of the three-layer architecture
/// 
pub struct Presenter {
    renderer: Option<Box<Renderer>>,
    pending_user_actions: PresentationLayerActions,
    
    #[cfg(debug_assertions)]
    pending_debug_actions: DebugLayerActions,
    
    interval_position: f32,
    
    #[cfg(target_arch = "wasm32")]
    sidebar_ui_active: bool,

    #[cfg(target_arch = "wasm32")]
    self_reference: Option<Rc<RefCell<Self>>>,
    
    #[cfg(target_arch = "wasm32")]
    ui_listeners_attached: bool,
}

impl Presenter {
    /// Create a new Presenter
    pub fn create() -> Result<Self, String> {
        #[cfg(target_arch = "wasm32")]
        {
            setup_sidebar_controls();
        }
        
        Ok(Self {
            renderer: None,
            pending_user_actions: PresentationLayerActions::default(),
            #[cfg(debug_assertions)]
            pending_debug_actions: DebugLayerActions::new(),
            interval_position: 0.0,
            #[cfg(target_arch = "wasm32")]
            sidebar_ui_active: true,
            #[cfg(target_arch = "wasm32")]
            self_reference: None,
            #[cfg(target_arch = "wasm32")]
            ui_listeners_attached: false,
        })
    }

    /// Set the self-reference for UI event handling
    #[cfg(target_arch = "wasm32")]
    pub fn set_self_reference(&mut self, self_ref: Rc<RefCell<Self>>) {
        self.self_reference = Some(self_ref.clone());
        
        if !self.ui_listeners_attached {
            setup_event_listeners(self_ref);
            self.ui_listeners_attached = true;
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_self_reference(&mut self, _self_ref: Rc<RefCell<Self>>) {
    }

    pub fn update_graphics(&mut self, viewport: Viewport, model_data: &ModelUpdateResult) {
        let (pitch_detected, clarity) = match model_data.pitch {
            Pitch::Detected(_, clarity_value) => (true, Some(clarity_value)),
            Pitch::NotDetected => (false, None),
        };
        
        
        if let Some(renderer) = &mut self.renderer {
            renderer.update_presentation_context(&crate::shared_types::PresentationContext {
                tuning_fork_note: model_data.tuning_fork_note,
                tuning_system: model_data.tuning_system,
                current_scale: model_data.scale,
            }, viewport);
            
            renderer.update_audio_analysis(AudioAnalysis {
                pitch_detected,
                cents_offset: model_data.accuracy.cents_offset,
                interval: self.interval_position,
                clarity,
                volume_peak: model_data.volume_peak,
            });
            
            renderer.update_pitch_position(viewport);
        }
    }

    /// Update the presentation layer with model data
    pub fn process_data(&mut self, _timestamp: f64, model_data: ModelUpdateResult) {
        self.process_volume_data(&model_data.volume);
        self.process_pitch_data(&model_data.pitch);
        self.process_accuracy_data(&model_data.accuracy);
        self.process_error_states(&model_data.errors);
        self.process_tuning_system(&model_data.tuning_system);
        self.sync_sidebar_ui(&model_data);
        
        self.interval_position = self.calculate_interval_position_from_frequency(&model_data.pitch, model_data.tuning_fork_note);
    }

    /// Retrieve and clear all pending user actions
    pub fn get_user_actions(&mut self) -> PresentationLayerActions {
        std::mem::take(&mut self.pending_user_actions)
    }

    /// Handle user request to change the tuning system
    pub fn on_tuning_system_changed(&mut self, tuning_system: TuningSystem) {
        self.pending_user_actions.tuning_system_changes.push(ChangeTuningSystem { tuning_system });
    }

    /// Handle user request to adjust the tuning fork
    pub fn on_tuning_fork_adjusted(&mut self, note: MidiNote) {
        self.pending_user_actions.tuning_fork_adjustments.push(AdjustTuningFork { note });
    }

    /// Handle scale change action
    pub fn on_scale_changed(&mut self, scale: Scale) {
        self.pending_user_actions.scale_changes.push(ScaleChangeAction { scale });
    }

    #[cfg(debug_assertions)]
    pub fn get_debug_actions(&mut self) -> DebugLayerActions {
        std::mem::take(&mut self.pending_debug_actions)
    }

    #[cfg(debug_assertions)]
    pub fn on_test_signal_configured(&mut self, enabled: bool, frequency: f32, volume: f32) {
        self.pending_debug_actions.test_signal_configurations.push(ConfigureTestSignal {
            enabled,
            frequency,
            volume,
        });
    }
    pub fn on_tuning_fork_configured(&mut self, _enabled: bool, note: MidiNote, volume_amplitude: f32) {
        crate::common::dev_log!("PRESENTER: Tuning fork audio configured - tuning_fork: {}, volume: {}", 
                                note, volume_amplitude);
        
        self.pending_user_actions.tuning_fork_configurations.push(ConfigureTuningFork {
            frequency: Self::midi_note_to_frequency(note),
            volume: volume_amplitude,
        });
        crate::common::dev_log!("PRESENTER: Added action to pending_user_actions, total actions: {}", self.pending_user_actions.tuning_fork_configurations.len());
    }
    
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
    pub fn render(&mut self, context: &Context, screen: &mut RenderTarget, model_data: &ModelUpdateResult) {
        if self.renderer.is_none() {
            let renderer = match Renderer::new(context, screen.viewport()) {
                Ok(scene) => scene,
                Err(e) => {
                    crate::common::dev_log!("Failed to create Renderer: {}", e);
                    screen.clear(three_d::ClearState::color(0.0, 0.0, 0.0, 1.0));
                    return;
                }
            };
            
            self.renderer = Some(Box::new(renderer));
            self.update_graphics(screen.viewport(), model_data);
            
            #[cfg(target_arch = "wasm32")]
            self.sync_sidebar_ui(model_data);
        }
        
        let viewport = screen.viewport();
        
        if self.renderer.is_some() {
            self.update_graphics(viewport, model_data);
        }
        
        if let Some(renderer) = &mut self.renderer {
            renderer.render(screen, viewport);
        } else {
            screen.clear(three_d::ClearState::color(0.0, 0.0, 0.0, 1.0));
        }
    }

    
    fn process_volume_data(&mut self, volume: &crate::shared_types::Volume) {
        if volume.peak_amplitude > -20.0 {
        }
    }
    
    fn process_pitch_data(&mut self, _pitch: &crate::shared_types::Pitch) {
    }
    
    fn process_accuracy_data(&mut self, accuracy: &crate::shared_types::IntonationData) {
        if accuracy.cents_offset.abs() < crate::app_config::INTONATION_ACCURACY_THRESHOLD {
        } else if accuracy.cents_offset.abs() > 30.0 {
        }
    }
    
    fn process_error_states(&mut self, errors: &Vec<crate::shared_types::Error>) {
        if errors.is_empty() {
            return;
        }
        
        for error in errors {
            match error {
                crate::shared_types::Error::MicrophonePermissionDenied => {
                }
                crate::shared_types::Error::MicrophoneNotAvailable => {
                    crate::web::error_message_box::show_error(&crate::shared_types::Error::MicrophoneNotAvailable);
                }
                crate::shared_types::Error::BrowserApiNotSupported => {
                }
                crate::shared_types::Error::ProcessingError(msg) => {
                    crate::common::error_log!("🔥 PROCESSING ERROR: {}", msg);
                }
                crate::shared_types::Error::MobileDeviceNotSupported => {
                    crate::web::error_message_box::show_error(&crate::shared_types::Error::MobileDeviceNotSupported);
                }
                crate::shared_types::Error::BrowserError => {
                    crate::web::error_message_box::show_error(&crate::shared_types::Error::BrowserError);
                }
            }
        }
    }
    
    fn process_tuning_system(&mut self, _tuning_system: &crate::shared_types::TuningSystem) {
    }
    
    /// Calculate interval position from frequency and tuning fork
    fn calculate_interval_position_from_frequency(&self, pitch: &Pitch, note: MidiNote) -> f32 {
        match pitch {
            Pitch::Detected(frequency, _clarity) => {
                let tuning_fork_frequency = Self::midi_note_to_frequency(note);
                (frequency / tuning_fork_frequency).log2()
            }
            Pitch::NotDetected => 0.0,
        }
    }
    
    fn midi_note_to_frequency(midi_note: MidiNote) -> f32 {
        crate::music_theory::midi_note_to_standard_frequency(midi_note)
    }
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

    #[cfg(target_arch = "wasm32")]
    fn sync_sidebar_ui(&self, model_data: &ModelUpdateResult) {
        crate::web::sidebar_controls::sync_sidebar_with_presenter_state(model_data);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn sync_sidebar_ui(&self, _model_data: &ModelUpdateResult) {
    }
    
    #[cfg(target_arch = "wasm32")]
    fn cleanup_sidebar_ui_if_active(&mut self) {
        if self.sidebar_ui_active {
            cleanup_sidebar_controls();
            self.sidebar_ui_active = false;
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn cleanup_sidebar_ui_if_active(&mut self) {
    }
}

impl Drop for Presenter {
    fn drop(&mut self) {
        self.cleanup_sidebar_ui_if_active();
    }
}

