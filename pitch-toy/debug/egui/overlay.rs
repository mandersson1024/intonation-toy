// EGUI Microphone Button
// Manages microphone button rendering in three-d + egui context

use three_d::egui;
use super::super::microphone_button::MicrophoneButton;
use observable_data::DataObserver;
use crate::audio::AudioPermission;

/// EGUI microphone button wrapper for three-d + egui rendering
pub struct EguiMicrophoneButton {
    microphone_button: MicrophoneButton,
}

impl EguiMicrophoneButton {
    /// Create new EGUI microphone button with permission observer and action trigger
    pub fn new(
        permission_observer: DataObserver<AudioPermission>,
        microphone_trigger: action::ActionTrigger<crate::MicrophonePermissionAction>,
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
        }
    }
    
    /// Render the microphone button
    pub fn render(&mut self, gui_context: &egui::Context) {
        self.microphone_button.render_center_button(gui_context);
    }
}