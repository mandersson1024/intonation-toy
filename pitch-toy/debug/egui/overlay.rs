// EGUI Microphone Button
// Manages microphone button rendering in three-d + egui context

use three_d::egui;
use super::super::microphone_button::MicrophoneButton;
use observable_data::DataObserver;
use crate::audio::AudioPermission;

/// EGUI microphone button wrapper for three-d + egui rendering
pub struct EguiMicrophoneButton {
    microphone_button: MicrophoneButton,
    output_to_speakers: bool,
    prev_output_to_speakers: bool,
    output_to_speakers_trigger: action::ActionTrigger<crate::OutputToSpeakersAction>,
}

impl EguiMicrophoneButton {
    /// Create new EGUI microphone button with permission observer and action trigger
    pub fn new(
        permission_observer: DataObserver<AudioPermission>,
        microphone_trigger: action::ActionTrigger<crate::MicrophonePermissionAction>,
        output_to_speakers_trigger: action::ActionTrigger<crate::OutputToSpeakersAction>,
    ) -> Self {
        let mut microphone_button = MicrophoneButton::new(permission_observer);
        
        // Set up microphone button click callback to trigger permission request action
        microphone_button.set_click_callback(move || {
            let action = crate::MicrophonePermissionAction {
                request_permission: true,
            };
            microphone_trigger.fire(action);
        });
        
        Self {
            microphone_button,
            output_to_speakers: false,
            prev_output_to_speakers: false,
            output_to_speakers_trigger,
        }
    }
    
    /// Render the microphone button and output to speakers control
    pub fn render(&mut self, gui_context: &egui::Context) {
        // Get screen dimensions
        let screen_rect = gui_context.screen_rect();
        let center_x = screen_rect.width() / 2.0;
        let center_y = screen_rect.height() / 2.0;
        
        // Create a fixed window in the center with more height for both controls
        egui::Window::new("audio_controls")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .fixed_pos([center_x - 120.0, center_y - 80.0])
            .fixed_size([240.0, 160.0])
            .show(gui_context, |ui| {
                ui.vertical_centered(|ui| {
                    // Render microphone permission button
                    let permission_state = self.microphone_button.get_permission_state();
                    let button_text = match permission_state {
                        AudioPermission::Uninitialized => "Request Permission",
                        AudioPermission::Requesting => "Requesting...",
                        AudioPermission::Granted => "Granted",
                        AudioPermission::Denied => "Denied",
                        AudioPermission::Unavailable => "Unknown Error",
                    };
                    let button_enabled = matches!(permission_state, 
                        AudioPermission::Uninitialized | AudioPermission::Denied | AudioPermission::Unavailable
                    );
                    
                    ui.add_enabled_ui(button_enabled, |ui| {
                        if ui.button(button_text).clicked() {
                            self.microphone_button.trigger_click();
                        }
                    });
                    
                    ui.add_space(10.0);
                    
                    // Render output to speakers checkbox
                    if ui.checkbox(&mut self.output_to_speakers, "Output to Speakers").changed() {
                        let action = crate::OutputToSpeakersAction {
                            enabled: self.output_to_speakers,
                        };
                        self.output_to_speakers_trigger.fire(action);
                    }
                });
            });
    }
}