// EGUI Microphone Button
// Manages microphone button rendering in three-d + egui context

use three_d::egui;
use crate::engine::audio::{TestWaveform, test_signal_generator::BackgroundNoiseConfig};

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

/// EGUI debug controls wrapper for three-d + egui rendering
pub struct EguiDebugControls {
    output_to_speakers: bool,
    prev_output_to_speakers: bool,
    test_signal_config: TestSignalConfig,
    prev_test_signal_config: TestSignalConfig,
    background_noise_config: BackgroundNoiseConfig,
    prev_background_noise_config: BackgroundNoiseConfig,
}

impl EguiDebugControls {
    /// Create new EGUI debug controls
    pub fn new() -> Self {
        let test_signal_config = TestSignalConfig::default();
        let background_noise_config = BackgroundNoiseConfig::default();
        
        Self {
            output_to_speakers: false,
            prev_output_to_speakers: false,
            test_signal_config: test_signal_config.clone(),
            prev_test_signal_config: test_signal_config,
            background_noise_config: background_noise_config.clone(),
            prev_background_noise_config: background_noise_config,
        }
    }
    
    /// Apply test signal configuration to audio system
    #[cfg(debug_assertions)]
    fn apply_test_signal_config(&self, config: &TestSignalConfig, presenter: &mut crate::presentation::Presenter) {
        presenter.on_test_signal_configured(
            config.enabled,
            config.frequency,
            config.volume,
            config.waveform.clone(),
        );
    }
    
    /// Apply background noise configuration to audio system
    #[cfg(debug_assertions)]
    fn apply_background_noise_config(&self, config: &BackgroundNoiseConfig, presenter: &mut crate::presentation::Presenter) {
        presenter.on_background_noise_configured(
            config.enabled,
            config.level,
            config.noise_type.clone(),
        );
    }
    
    /// Check for configuration changes and apply them
    #[cfg(debug_assertions)]
    fn check_and_apply_changes(&mut self, presenter: &mut crate::presentation::Presenter) {
        // Check test signal config changes
        if self.test_signal_config != self.prev_test_signal_config {
            self.apply_test_signal_config(&self.test_signal_config, presenter);
            self.prev_test_signal_config = self.test_signal_config.clone();
        }
        
        // Check background noise config changes
        if self.background_noise_config != self.prev_background_noise_config {
            self.apply_background_noise_config(&self.background_noise_config, presenter);
            self.prev_background_noise_config = self.background_noise_config.clone();
        }
        
        // Check output to speakers changes
        if self.output_to_speakers != self.prev_output_to_speakers {
            presenter.on_output_to_speakers_configured(self.output_to_speakers);
            self.prev_output_to_speakers = self.output_to_speakers;
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
    
    /// Render debug controls inline within an existing UI context
    #[cfg(debug_assertions)]
    pub fn render_inline(&mut self, ui: &mut egui::Ui, presenter: &mut crate::presentation::Presenter) {
        // Render debug controls (test signal, background noise, output to speakers)
        
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.output_to_speakers, "Output to Speakers");
        });
        
        // Check and apply any configuration changes
        self.check_and_apply_changes(presenter);
    }
}