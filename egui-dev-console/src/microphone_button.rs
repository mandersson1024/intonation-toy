// Microphone Permission Button for EGUI
// Central microphone button that requests permission and connects to audio pipeline

use std::sync::Arc;
use three_d::egui;
use observable_data::ObservableData;

/// Audio permission states trait - implementors can define their own permission types
pub trait AudioPermissionState: Clone + Send + Sync + 'static {
    fn is_uninitialized(&self) -> bool;
    fn is_requesting(&self) -> bool;
    fn is_granted(&self) -> bool;
    fn is_denied(&self) -> bool;
    fn is_unavailable(&self) -> bool;
    
    fn get_icon_text(&self) -> &'static str {
        if self.is_granted() { "🎤 ✓" }
        else if self.is_denied() { "🎤 ✗" }
        else if self.is_requesting() { "🎤 ..." }
        else if self.is_unavailable() { "🎤 ⚠" }
        else { "🎤" }
    }
    
    fn get_status_text(&self) -> &'static str {
        if self.is_uninitialized() { "Click to enable microphone" }
        else if self.is_requesting() { "Requesting permission..." }
        else if self.is_granted() { "Microphone enabled" }
        else if self.is_denied() { "Permission denied" }
        else { "Microphone unavailable" }
    }
    
    fn get_button_text(&self) -> &'static str {
        if self.is_uninitialized() { "Enable Microphone" }
        else if self.is_requesting() { "Requesting..." }
        else if self.is_granted() { "Microphone Ready" }
        else if self.is_denied() { "Try Again" }
        else { "Check Device" }
    }
    
    fn is_button_enabled(&self) -> bool {
        self.is_uninitialized() || self.is_denied() || self.is_unavailable()
    }
}

/// Callback type for permission state changes
pub type PermissionCallback<T> = Arc<dyn Fn(T) + Send + Sync>;

/// Callback type for microphone button clicks (must be synchronous for getUserMedia)
pub type ClickCallback = Arc<dyn Fn() + Send + Sync>;

/// Microphone button state and behavior
pub struct MicrophoneButton<T: AudioPermissionState> {
    microphone_permission: ObservableData<T>,
    error_message: Option<String>,
    permission_callback: Option<PermissionCallback<T>>,
    click_callback: Option<ClickCallback>,
}

impl<T: AudioPermissionState> MicrophoneButton<T> {
    pub fn new(microphone_permission: ObservableData<T>) -> Self {
        Self {
            microphone_permission,
            error_message: None,
            permission_callback: None,
            click_callback: None,
        }
    }

    /// Set callback for permission state changes
    pub fn set_permission_callback<F>(&mut self, callback: F)
    where
        F: Fn(T) + Send + Sync + 'static,
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


    /// Set error message
    pub fn set_error(&mut self, error: Option<String>) {
        self.error_message = error;
    }


    /// Render the microphone button in the center of the screen
    /// Returns true if clicked
    pub fn render_center_button(&mut self, ctx: &egui::Context) -> bool {
        // Show button for all states - users can see success/failure feedback
        
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
                    
                    // Get current permission state from observable
                    let permission_state = self.microphone_permission.get();
                    
                    // Microphone icon (using text for now)
                    let mic_text = permission_state.get_icon_text();
                    
                    ui.heading(mic_text);
                    ui.add_space(5.0);
                    
                    // Status text
                    let status_text = permission_state.get_status_text();
                    
                    ui.label(status_text);
                    ui.add_space(5.0);
                    
                    // Button
                    let button_text = permission_state.get_button_text();
                    let button_enabled = permission_state.is_button_enabled();
                    
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

