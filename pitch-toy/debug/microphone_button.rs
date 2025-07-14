// Microphone Permission Button for EGUI
// Central microphone button that requests permission and connects to audio pipeline

use std::sync::Arc;
use three_d::egui;
use observable_data::DataObserver;
use super::AudioPermission;



/// Callback type for microphone button clicks (must be synchronous for getUserMedia)
type ClickCallback = Arc<dyn Fn() + Send + Sync>;

/// Microphone button state and behavior
pub struct MicrophoneButton {
    microphone_permission: DataObserver<AudioPermission>,
    click_callback: Option<ClickCallback>,
}

impl MicrophoneButton {
    pub fn new(microphone_permission: DataObserver<AudioPermission>) -> Self {
        Self {
            microphone_permission,
            click_callback: None,
        }
    }

    /// Set callback for button clicks (called synchronously)
    pub fn set_click_callback<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.click_callback = Some(Arc::new(callback));
    }


    /// Render the microphone button in the center of the screen
    pub fn render_center_button(&mut self, ctx: &egui::Context) {
        // Show button for all states - users can see success/failure feedback
        
        // Get screen dimensions
        let screen_rect = ctx.screen_rect();
        let center_x = screen_rect.width() / 2.0;
        let center_y = screen_rect.height() / 2.0;
        
        // Create a fixed window in the center
        egui::Window::new("microphone_permission")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .fixed_pos([center_x - 100.0, center_y - 60.0])
            .fixed_size([200.0, 120.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    let permission_state = self.microphone_permission.get();
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
                            // Call the click callback immediately (synchronous with user gesture)
                            if let Some(callback) = &self.click_callback {
                                callback();
                            }
                        }
                    });
                });
            });
    }
}

