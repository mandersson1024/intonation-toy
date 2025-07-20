// EGUI Microphone Button
// Manages microphone button rendering in three-d + egui context

use three_d::egui;
use super::super::microphone_button::MicrophoneButton;
use observable_data::DataObserver;
use crate::engine::audio::{AudioPermission, TestWaveform, test_signal_generator::BackgroundNoiseConfig};

/// Test signal configuration
#[derive(Debug, Clone, PartialEq)]
pub struct TestSignalConfig {
    pub enabled: bool,
    pub waveform: TestWaveform,
    pub frequency: f32,
    pub volume: f32,
}

impl Default for TestSignalConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            waveform: TestWaveform::Sine,
            frequency: 440.0,
            volume: 50.0,
        }
    }
}

/// EGUI microphone button wrapper for three-d + egui rendering
pub struct EguiMicrophoneButton {
    microphone_button: MicrophoneButton,
    output_to_speakers: bool,
    prev_output_to_speakers: bool,
    output_to_speakers_trigger: action::ActionTrigger<crate::OutputToSpeakersAction>,
    test_signal_config: TestSignalConfig,
    prev_test_signal_config: TestSignalConfig,
    background_noise_config: BackgroundNoiseConfig,
    prev_background_noise_config: BackgroundNoiseConfig,
    test_signal_trigger: action::ActionTrigger<crate::TestSignalAction>,
    background_noise_trigger: action::ActionTrigger<crate::BackgroundNoiseAction>,
}

impl EguiMicrophoneButton {
    /// Create new EGUI microphone button with permission observer and action trigger
    pub fn new(
        permission_observer: DataObserver<AudioPermission>,
        microphone_trigger: action::ActionTrigger<crate::MicrophonePermissionAction>,
        output_to_speakers_trigger: action::ActionTrigger<crate::OutputToSpeakersAction>,
        test_signal_trigger: action::ActionTrigger<crate::TestSignalAction>,
        background_noise_trigger: action::ActionTrigger<crate::BackgroundNoiseAction>,
    ) -> Self {
        let mut microphone_button = MicrophoneButton::new(permission_observer);
        
        // Set up microphone button click callback to trigger permission request action
        microphone_button.set_click_callback(move || {
            let action = crate::MicrophonePermissionAction {
                request_permission: true,
            };
            microphone_trigger.fire(action);
        });
        
        let test_signal_config = TestSignalConfig::default();
        let background_noise_config = BackgroundNoiseConfig::default();
        
        Self {
            microphone_button,
            output_to_speakers: false,
            prev_output_to_speakers: false,
            output_to_speakers_trigger,
            test_signal_config: test_signal_config.clone(),
            prev_test_signal_config: test_signal_config,
            background_noise_config: background_noise_config.clone(),
            prev_background_noise_config: background_noise_config,
            test_signal_trigger,
            background_noise_trigger,
        }
    }
    
    /// Apply test signal configuration to audio system
    fn apply_test_signal_config(&self, config: &TestSignalConfig) {
        let action = crate::TestSignalAction {
            enabled: config.enabled,
            waveform: config.waveform.clone(),
            frequency: config.frequency,
            volume: config.volume,
        };
        self.test_signal_trigger.fire(action);
    }
    
    /// Apply background noise configuration to audio system
    fn apply_background_noise_config(&self, config: &BackgroundNoiseConfig) {
        let action = crate::BackgroundNoiseAction {
            enabled: config.enabled,
            level: config.level,
            noise_type: config.noise_type.clone(),
        };
        self.background_noise_trigger.fire(action);
    }
    
    /// Check for configuration changes and apply them
    fn check_and_apply_changes(&mut self) {
        // Check test signal config changes
        if self.test_signal_config != self.prev_test_signal_config {
            self.apply_test_signal_config(&self.test_signal_config);
            self.prev_test_signal_config = self.test_signal_config.clone();
        }
        
        // Check background noise config changes
        if self.background_noise_config != self.prev_background_noise_config {
            self.apply_background_noise_config(&self.background_noise_config);
            self.prev_background_noise_config = self.background_noise_config.clone();
        }
    }
    
    /// Render test signal controls section
    fn render_test_signal_controls_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Test Signal Generator");
        
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.test_signal_config.enabled, "Enable Test Signal");
        });
        
        // Always show all controls but disable them when test signal is disabled
        ui.add_enabled_ui(self.test_signal_config.enabled, |ui| {
            ui.horizontal(|ui| {
                ui.label("Waveform:");
                egui::ComboBox::from_id_salt("waveform")
                    .selected_text(format!("{:?}", self.test_signal_config.waveform))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.test_signal_config.waveform, TestWaveform::Sine, "Sine");
                        ui.selectable_value(&mut self.test_signal_config.waveform, TestWaveform::Square, "Square");
                        ui.selectable_value(&mut self.test_signal_config.waveform, TestWaveform::Triangle, "Triangle");
                        ui.selectable_value(&mut self.test_signal_config.waveform, TestWaveform::Sawtooth, "Sawtooth");
                        ui.selectable_value(&mut self.test_signal_config.waveform, TestWaveform::WhiteNoise, "White Noise");
                        ui.selectable_value(&mut self.test_signal_config.waveform, TestWaveform::PinkNoise, "Pink Noise");
                    });
            });
            
            ui.horizontal(|ui| {
                ui.label("Frequency:");
                ui.add(egui::Slider::new(&mut self.test_signal_config.frequency, 20.0..=2000.0)
                    .suffix(" Hz")
                    .logarithmic(true));
            });
            
            ui.horizontal(|ui| {
                ui.label("Volume:");
                ui.add(egui::Slider::new(&mut self.test_signal_config.volume, 0.0..=100.0)
                    .suffix("%"));
            });
        });
        
        ui.horizontal(|ui| {
            ui.label("Background Noise:");
            let mut level = self.background_noise_config.level * 100.0; // Convert to percentage
            if ui.add(egui::Slider::new(&mut level, 0.0..=100.0)
                .suffix("%")).changed() {
                self.background_noise_config.level = level / 100.0; // Convert back to 0-1 range
                self.background_noise_config.enabled = level > 0.0; // Auto-enable when level > 0
            }
        });
    }
    
    /// Render the microphone button and all audio controls
    pub fn render(&mut self, gui_context: &egui::Context) {
        // Get screen dimensions
        let screen_rect = gui_context.screen_rect();
        let center_x = screen_rect.width() / 2.0;
        let center_y = screen_rect.height() / 2.0;
        
        // Create a larger window in the center for all audio controls
        egui::Window::new("audio_controls")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .fixed_pos([center_x - 200.0, center_y - 75.0])
            .fixed_size([400.0, 500.0])
            .show(gui_context, |ui| {
                ui.vertical_centered(|ui| {
                    // Render microphone permission status
                    let permission_state = self.microphone_button.get_permission_state();
                    
                    ui.horizontal(|ui| {
                        ui.label("Microphone Permission:");
                        let (color, text) = match permission_state {
                            AudioPermission::Uninitialized => (three_d::egui::Color32::GRAY, "Uninitialized"),
                            AudioPermission::Requesting => (three_d::egui::Color32::YELLOW, "Requesting"),
                            AudioPermission::Granted => (three_d::egui::Color32::GREEN, "Granted"),
                            AudioPermission::Denied => (three_d::egui::Color32::RED, "Denied"),
                            AudioPermission::Unavailable => (three_d::egui::Color32::RED, "Unavailable"),
                        };
                        ui.colored_label(color, text);
                    });
                    
                    ui.add_space(10.0);
                    
                    // Render microphone permission button
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
                    
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);
                    
                    // Render test signal controls
                    self.render_test_signal_controls_section(ui);
                    
                    // Check for changes and apply them after rendering
                    self.check_and_apply_changes();
                });
            });
    }
}