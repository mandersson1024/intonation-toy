// Microphone Permission Button for EGUI
// Central microphone button that requests permission and connects to audio pipeline

use std::sync::Arc;
use three_d::egui;

/// Audio permission states (re-exported from main crate or defined here for compatibility)
#[derive(Debug, Clone, PartialEq)]
pub enum AudioPermission {
    Uninitialized,
    Requesting, 
    Granted,
    Denied,
    Unavailable,
}

impl std::fmt::Display for AudioPermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioPermission::Uninitialized => write!(f, "Uninitialized"),
            AudioPermission::Requesting => write!(f, "Requesting"),
            AudioPermission::Granted => write!(f, "Granted"),
            AudioPermission::Denied => write!(f, "Denied"),
            AudioPermission::Unavailable => write!(f, "Unavailable"),
        }
    }
}

/// Callback type for permission state changes
pub type PermissionCallback = Arc<dyn Fn(AudioPermission) + Send + Sync>;

/// Callback type for microphone button clicks (must be synchronous for getUserMedia)
pub type ClickCallback = Arc<dyn Fn() + Send + Sync>;

/// Microphone button state and behavior
pub struct MicrophoneButton {
    permission_state: AudioPermission,
    error_message: Option<String>,
    permission_callback: Option<PermissionCallback>,
    click_callback: Option<ClickCallback>,
}

impl MicrophoneButton {
    pub fn new() -> Self {
        Self {
            permission_state: AudioPermission::Uninitialized,
            error_message: None,
            permission_callback: None,
            click_callback: None,
        }
    }

    /// Set callback for permission state changes
    pub fn set_permission_callback<F>(&mut self, callback: F)
    where
        F: Fn(AudioPermission) + Send + Sync + 'static,
    {
        self.permission_callback = Some(Arc::new(callback));
    }

    /// Set callback for button clicks (called synchronously)
    pub fn set_click_callback<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.click_callback = Some(Arc::new(callback));
    }

    /// Update permission state
    pub fn update_permission_state(&mut self, state: AudioPermission) {
        self.permission_state = state.clone();
        if let Some(callback) = &self.permission_callback {
            callback(state);
        }
    }

    /// Set error message
    pub fn set_error(&mut self, error: Option<String>) {
        self.error_message = error;
    }

    /// Get current permission state
    pub fn permission_state(&self) -> &AudioPermission {
        &self.permission_state
    }

    /// Render the microphone button in the center of the screen
    /// Returns true if clicked, but only renders if permission is not granted
    pub fn render_center_button(&mut self, ctx: &egui::Context) -> bool {
        // Only show the button if permission is not granted
        if matches!(self.permission_state, AudioPermission::Granted) {
            return false;
        }
        
        let mut clicked = false;
        
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
                    ui.add_space(10.0);
                    
                    // Microphone icon (using text for now)
                    let mic_text = match self.permission_state {
                        AudioPermission::Granted => "🎤 ✓",
                        AudioPermission::Denied => "🎤 ✗",
                        AudioPermission::Requesting => "🎤 ...",
                        AudioPermission::Unavailable => "🎤 ⚠",
                        AudioPermission::Uninitialized => "🎤",
                    };
                    
                    ui.heading(mic_text);
                    ui.add_space(5.0);
                    
                    // Status text
                    let status_text = match self.permission_state {
                        AudioPermission::Uninitialized => "Click to enable microphone",
                        AudioPermission::Requesting => "Requesting permission...",
                        AudioPermission::Granted => "Microphone enabled",
                        AudioPermission::Denied => "Permission denied",
                        AudioPermission::Unavailable => "Microphone unavailable",
                    };
                    
                    ui.label(status_text);
                    ui.add_space(5.0);
                    
                    // Button
                    let button_text = match self.permission_state {
                        AudioPermission::Uninitialized => "Enable Microphone",
                        AudioPermission::Requesting => "Requesting...",
                        AudioPermission::Granted => "Microphone Ready",
                        AudioPermission::Denied => "Try Again",
                        AudioPermission::Unavailable => "Check Device",
                    };
                    
                    let button_enabled = matches!(
                        self.permission_state,
                        AudioPermission::Uninitialized | AudioPermission::Denied | AudioPermission::Unavailable
                    );
                    
                    ui.add_enabled_ui(button_enabled, |ui| {
                        if ui.button(button_text).clicked() {
                            clicked = true;
                            // Call the click callback immediately (synchronous with user gesture)
                            if let Some(callback) = &self.click_callback {
                                callback();
                            }
                        }
                    });
                    
                    // Error message
                    if let Some(error) = &self.error_message {
                        ui.add_space(5.0);
                        ui.colored_label(egui::Color32::RED, error);
                    }
                });
            });
        
        clicked
    }
}

impl Default for MicrophoneButton {
    fn default() -> Self {
        Self::new()
    }
}