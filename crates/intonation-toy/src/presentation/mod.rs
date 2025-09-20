#![cfg(target_arch = "wasm32")]

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
mod background_shader;
mod renderer;
mod tuning_lines;
mod egui_text_backend;
mod user_pitch_line;
pub use audio_analysis::AudioAnalysis;
pub use background_shader::BackgroundShader;
pub use renderer::Renderer;
pub use tuning_lines::TuningLines;
pub use egui_text_backend::EguiTextBackend;
pub use user_pitch_line::UserPitchLine;

use std::rc::Rc;
use std::cell::RefCell;
use three_d::{RenderTarget, Context, Viewport};
use crate::common::shared_types::{ModelUpdateResult, TuningSystem, Scale, MidiNote, Pitch};

use crate::web::sidebar_controls::{setup_sidebar_controls, cleanup_sidebar_controls, setup_event_listeners};

/// Request to change the tuning system
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeTuningSystem {
    pub tuning_system: TuningSystem,
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
pub struct ConfigureTonalCenter {
    pub note: MidiNote,
    pub volume: f32,
}

/// Container for all collected user actions from the presentation layer
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PresentationLayerActions {
    pub tuning_system_change: Option<ChangeTuningSystem>,
    pub scale_change: Option<ScaleChangeAction>,
    pub tonal_center_configuration: Option<ConfigureTonalCenter>,
}

impl PresentationLayerActions {
    /// Check if there are any actions to process
    pub fn has_actions(&self) -> bool {
        self.tuning_system_change.is_some() ||
        self.scale_change.is_some() ||
        self.tonal_center_configuration.is_some()
    }
}

/// Container for all collected debug actions from the presentation layer
#[cfg(debug_assertions)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DebugLayerActions {
    pub test_signal_configuration: Option<ConfigureTestSignal>,
}

/// Presenter - The presentation layer of the three-layer architecture
/// 
pub struct Presenter {
    renderer: Option<Box<Renderer>>,
    pending_user_actions: PresentationLayerActions,
    
    #[cfg(debug_assertions)]
    pending_debug_actions: DebugLayerActions,
    
    interval_position: f32,
    
    sidebar_ui_active: bool,

    self_reference: Option<Rc<RefCell<Self>>>,
    
    ui_listeners_attached: bool,
}

impl Presenter {
    /// Create a new Presenter wrapped in Rc<RefCell>
    pub fn create() -> Result<Rc<RefCell<Self>>, String> {
        setup_sidebar_controls();
        
        let presenter = Self {
            renderer: None,
            pending_user_actions: PresentationLayerActions::default(),
            #[cfg(debug_assertions)]
            pending_debug_actions: DebugLayerActions::default(),
            interval_position: 0.0,
            sidebar_ui_active: true,
            self_reference: None,
            ui_listeners_attached: false,
        };
        
        let presenter_rc = Rc::new(RefCell::new(presenter));
        
        presenter_rc.borrow_mut().self_reference = Some(presenter_rc.clone());
        setup_event_listeners(presenter_rc.clone());
        presenter_rc.borrow_mut().ui_listeners_attached = true;
        
        Ok(presenter_rc)
    }

    /// Update the presentation layer with model data and graphics
    pub fn update(&mut self, viewport: Viewport, model_data: &ModelUpdateResult) {
        self.process_data(model_data);
        self.update_graphics(viewport, model_data);
    }

    fn update_graphics(&mut self, viewport: Viewport, model_data: &ModelUpdateResult) {
        let (pitch_detected, clarity, frequency) = match model_data.pitch {
            Pitch::Detected(freq, clarity_value) => (true, Some(clarity_value), freq),
            Pitch::NotDetected => (false, None, 0.0),
        };
        
        
        if let Some(renderer) = &mut self.renderer {
            renderer.update_presentation_context(&crate::common::shared_types::PresentationContext {
                tonal_center_note: model_data.tonal_center_note,
                tuning_system: model_data.tuning_system,
                current_scale: model_data.scale,
                display_range: crate::app_config::DEFAULT_DISPLAY_RANGE,
            }, viewport);
            
            let tonal_center_frequency = crate::common::music_theory::midi_note_to_standard_frequency(model_data.tonal_center_note);

            renderer.update_audio_analysis(AudioAnalysis {
                pitch_detected,
                cents_offset: model_data.cents_offset,
                interval: self.interval_position,
                clarity,
                volume_peak: model_data.is_peaking,
                frequency,
                tonal_center_frequency,
            });
            
            renderer.update_pitch_position(viewport);
        }
    }

    /// Update the presentation layer with model data
    fn process_data(&mut self, model_data: &ModelUpdateResult) {
        self.process_tuning_system(&model_data.tuning_system);
        self.sync_sidebar_ui(model_data);
        
        self.interval_position = self.calculate_interval_position_from_frequency(&model_data.pitch, model_data.tonal_center_note);
    }

    /// Retrieve and clear all pending user actions
    pub fn get_user_actions(&mut self) -> PresentationLayerActions {
        std::mem::take(&mut self.pending_user_actions)
    }

    /// Handle user request to change the tuning system
    pub fn on_tuning_system_changed(&mut self, tuning_system: TuningSystem) {
        self.pending_user_actions.tuning_system_change = Some(ChangeTuningSystem { tuning_system });
    }

    /// Handle scale change action
    pub fn on_scale_changed(&mut self, scale: Scale) {
        self.pending_user_actions.scale_change = Some(ScaleChangeAction { scale });
    }

    #[cfg(debug_assertions)]
    pub fn get_debug_actions(&mut self) -> DebugLayerActions {
        std::mem::take(&mut self.pending_debug_actions)
    }

    #[cfg(debug_assertions)]
    pub fn on_test_signal_configured(&mut self, enabled: bool, frequency: f32, volume: f32) {
        self.pending_debug_actions.test_signal_configuration = Some(ConfigureTestSignal {
            enabled,
            frequency,
            volume,
        });
    }
    pub fn on_tonal_center_configured(&mut self, _enabled: bool, note: MidiNote, volume_amplitude: f32) {
        crate::common::dev_log!("PRESENTER: Tonal center audio configured - tonal_center: {}, volume: {}", 
                                note, volume_amplitude);
        
        self.pending_user_actions.tonal_center_configuration = Some(ConfigureTonalCenter {
            note,
            volume: volume_amplitude,
        });
        crate::common::dev_log!("PRESENTER: Set tonal center configuration action");
    }
    

    /// Render the presentation layer to the screen
    pub fn render(&mut self, context: &Context, screen: &mut RenderTarget, model_data: &ModelUpdateResult) {
        if self.renderer.is_none() {
            let renderer = match Renderer::new(context, screen.viewport()) {
                Ok(scene) => scene,
                Err(_e) => {
                    crate::common::dev_log!("Failed to create Renderer: {}", _e);
                    screen.clear(three_d::ClearState::color(0.0, 0.0, 0.0, 1.0));
                    return;
                }
            };
            
            self.renderer = Some(Box::new(renderer));
            self.update_graphics(screen.viewport(), model_data);
            
            self.sync_sidebar_ui(model_data);
        }
        
        let viewport = screen.viewport();
        
        if self.renderer.is_some() {
            self.update_graphics(viewport, model_data);
        }
        
        if let Some(renderer) = &mut self.renderer {
            crate::profile!("renderer_render", renderer.render(screen, viewport));
        } else {
            screen.clear(three_d::ClearState::color(0.0, 0.0, 0.0, 1.0));
        }
    }
    
    fn process_tuning_system(&mut self, _tuning_system: &crate::common::shared_types::TuningSystem) {
    }
    
    /// Calculate interval position from frequency and tonal center
    fn calculate_interval_position_from_frequency(&self, pitch: &Pitch, note: MidiNote) -> f32 {
        match pitch {
            Pitch::Detected(frequency, _clarity) => {
                let tonal_center_frequency = Self::midi_note_to_frequency(note);
                (frequency / tonal_center_frequency).log2()
            }
            Pitch::NotDetected => 0.0,
        }
    }
    
    fn midi_note_to_frequency(midi_note: MidiNote) -> f32 {
        crate::common::music_theory::midi_note_to_standard_frequency(midi_note)
    }
    pub fn midi_note_to_frequency_with_tuning(
        &self,
        midi_note: MidiNote,
        note: MidiNote,
        tuning_system: TuningSystem,
    ) -> f32 {
        let tonal_center_frequency = crate::common::music_theory::midi_note_to_standard_frequency(note);
        let interval_semitones = (midi_note as i32) - (note as i32);
        crate::common::music_theory::interval_frequency(tuning_system, tonal_center_frequency, interval_semitones)
    }

    fn sync_sidebar_ui(&self, model_data: &ModelUpdateResult) {
        crate::web::sidebar_controls::sync_sidebar_with_presenter_state(model_data);
    }
    
    fn cleanup_sidebar_ui_if_active(&mut self) {
        if self.sidebar_ui_active {
            cleanup_sidebar_controls();
            self.sidebar_ui_active = false;
        }
    }
}

impl Drop for Presenter {
    fn drop(&mut self) {
        self.cleanup_sidebar_ui_if_active();
    }
}

