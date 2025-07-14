// EGUI Live Data Panel
// Real-time data visualization and monitoring for egui interface

use three_d::egui::{self, Color32, Vec2, Ui};
use std::rc::Rc;

use crate::audio::{
    AudioPermission, MusicalNote, VolumeLevel,
    AudioWorkletState, ConsoleAudioServiceImpl, TestWaveform,
    BackgroundNoiseConfig,
};
use crate::live_data::LiveData;

/// Performance metrics for display
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceMetrics {
    pub fps: f64,
    pub memory_usage: f64,
    pub audio_latency: f64,
    pub cpu_usage: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            fps: 0.0,
            memory_usage: 0.0,
            audio_latency: 0.0,
            cpu_usage: 0.0,
        }
    }
}

/// Volume level data for display
#[derive(Debug, Clone, PartialEq)]
pub struct VolumeLevelData {
    pub rms_db: f32,
    pub peak_db: f32,
    pub peak_fast_db: f32,
    pub peak_slow_db: f32,
    pub level: VolumeLevel,
    pub confidence_weight: f32,
    pub timestamp: f64,
}

/// Pitch detection data for display
#[derive(Debug, Clone, PartialEq)]
pub struct PitchData {
    pub frequency: f32,
    pub confidence: f32,
    pub note: MusicalNote,
    pub clarity: f32,
    pub timestamp: f64,
}

/// AudioWorklet status for display
#[derive(Debug, Clone, PartialEq)]
pub struct AudioWorkletStatus {
    pub state: AudioWorkletState,
    pub processor_loaded: bool,
    pub chunk_size: u32,
    pub chunks_processed: u32,
    pub last_update: f64,
}

impl Default for AudioWorkletStatus {
    fn default() -> Self {
        Self {
            state: AudioWorkletState::Uninitialized,
            processor_loaded: false,
            chunk_size: 128,
            chunks_processed: 0,
            last_update: 0.0,
        }
    }
}

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

/// EGUI Live Data Panel - Real-time audio monitoring and control interface
pub struct EguiLiveDataPanel {
    /// Audio service for device operations
    audio_service: Rc<ConsoleAudioServiceImpl>,
    
    live_data: LiveData,
    
    /// Test signal configuration
    test_signal_config: TestSignalConfig,
    background_noise_config: BackgroundNoiseConfig,
    output_to_speakers: bool,
    
    /// UI state
    last_metrics_update: f64,
}

impl EguiLiveDataPanel {
    /// Create new EGUI Live Data Panel
    pub fn new(
        audio_service: Rc<ConsoleAudioServiceImpl>,
        live_data: LiveData,
    ) -> Self {
        Self {
            audio_service,
            live_data,
            test_signal_config: TestSignalConfig::default(),
            background_noise_config: BackgroundNoiseConfig::default(),
            output_to_speakers: false,
            last_metrics_update: 0.0,
        }
    }
    
    /// Render the live data panel
    pub fn render(&mut self, gui_context: &egui::Context) {
        egui::Window::new("Live Data Panel")
            .default_size(Vec2::new(400.0, 600.0))
            .resizable(true)
            .show(gui_context, |ui| {
                self.render_content(ui);
            });
    }
    
    /// Render panel content
    fn render_content(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            // Audio Devices Section
            self.render_audio_devices_section(ui);
            ui.separator();
            
            // AudioWorklet Status Section
            self.render_audioworklet_status_section(ui);
            ui.separator();
            
            // Performance Metrics Section
            self.render_performance_metrics_section(ui);
            ui.separator();
            
            // Volume Level Section
            self.render_volume_level_section(ui);
            ui.separator();
            
            // Pitch Detection Section
            self.render_pitch_detection_section(ui);
            ui.separator();
            
            // Test Signal Controls Section
            self.render_test_signal_controls_section(ui);
            ui.separator();
            
            // Global Audio Controls Section
            self.render_global_audio_controls_section(ui);
        });
    }
    
    /// Render audio devices section
    fn render_audio_devices_section(&self, ui: &mut Ui) {
        ui.heading("Audio Devices");
        
        let permission = self.live_data.microphone_permission.get();
        let devices = self.live_data.audio_devices.get();
        
        ui.horizontal(|ui| {
            ui.label("Microphone Permission:");
            let (color, text) = match permission {
                AudioPermission::Uninitialized => (Color32::GRAY, "Uninitialized"),
                AudioPermission::Requesting => (Color32::YELLOW, "Requesting"),
                AudioPermission::Granted => (Color32::GREEN, "Granted"),
                AudioPermission::Denied => (Color32::RED, "Denied"),
                AudioPermission::Unavailable => (Color32::RED, "Unavailable"),
            };
            ui.colored_label(color, text);
        });
        
        ui.label(format!("Input Devices: {}", devices.input_devices.len()));
        for device in &devices.input_devices {
            ui.indent("input_device", |ui| {
                ui.label(format!("• {}", device.1));
            });
        }
        
        ui.label(format!("Output Devices: {}", devices.output_devices.len()));
        for device in &devices.output_devices {
            ui.indent("output_device", |ui| {
                ui.label(format!("• {}", device.1));
            });
        }
    }
    
    /// Render AudioWorklet status section
    fn render_audioworklet_status_section(&self, ui: &mut Ui) {
        ui.heading("AudioWorklet Status");
        
        let status = self.live_data.audioworklet_status.get();
        
        ui.horizontal(|ui| {
            ui.label("State:");
            let (color, text) = match status.state {
                AudioWorkletState::Uninitialized => (Color32::GRAY, "Uninitialized"),
                AudioWorkletState::Initializing => (Color32::YELLOW, "Initializing"),
                AudioWorkletState::Ready => (Color32::GREEN, "Ready"),
                AudioWorkletState::Processing => (Color32::GREEN, "Processing"),
                AudioWorkletState::Stopped => (Color32::YELLOW, "Stopped"),
                AudioWorkletState::Failed => (Color32::RED, "Failed"),
            };
            ui.colored_label(color, text);
        });
        
        ui.horizontal(|ui| {
            ui.label("Processor Loaded:");
            let color = if status.processor_loaded { Color32::GREEN } else { Color32::RED };
            ui.colored_label(color, status.processor_loaded.to_string());
        });
        
        ui.label(format!("Chunk Size: {} samples", status.chunk_size));
        ui.label(format!("Chunks Processed: {}", status.chunks_processed));
        
        if status.last_update > 0.0 {
            let now = js_sys::Date::now() / 1000.0; // Convert from ms to seconds
            let age = now - status.last_update;
            ui.label(format!("Last Update: {:.1}s ago", age));
        }
    }
    
    /// Render performance metrics section
    fn render_performance_metrics_section(&mut self, ui: &mut Ui) {
        ui.heading("Performance Metrics");
        
        let metrics = self.live_data.performance_metrics.get();
        
        // Update metrics periodically
        let now = js_sys::Date::now() / 1000.0; // Convert from ms to seconds
        if now - self.last_metrics_update > 1.0 {
            self.last_metrics_update = now;
            // TODO: Trigger metrics update through setter
        }
        
        ui.horizontal(|ui| {
            ui.label("FPS:");
            let color = if metrics.fps >= 50.0 { Color32::GREEN } 
                       else if metrics.fps >= 30.0 { Color32::YELLOW } 
                       else { Color32::RED };
            ui.colored_label(color, format!("{:.1}", metrics.fps));
        });
        
        ui.horizontal(|ui| {
            ui.label("Memory:");
            let color = if metrics.memory_usage < 50.0 { Color32::GREEN } 
                       else if metrics.memory_usage < 80.0 { Color32::YELLOW } 
                       else { Color32::RED };
            ui.colored_label(color, format!("{:.1}%", metrics.memory_usage));
        });
        
        ui.horizontal(|ui| {
            ui.label("Audio Latency:");
            let color = if metrics.audio_latency < 20.0 { Color32::GREEN } 
                       else if metrics.audio_latency < 50.0 { Color32::YELLOW } 
                       else { Color32::RED };
            ui.colored_label(color, format!("{:.1}ms", metrics.audio_latency));
        });
        
        ui.horizontal(|ui| {
            ui.label("CPU:");
            let color = if metrics.cpu_usage < 50.0 { Color32::GREEN } 
                       else if metrics.cpu_usage < 80.0 { Color32::YELLOW } 
                       else { Color32::RED };
            ui.colored_label(color, format!("{:.1}%", metrics.cpu_usage));
        });
    }
    
    /// Render volume level section
    fn render_volume_level_section(&self, ui: &mut Ui) {
        ui.heading("Volume Level");
        
        if let Some(volume) = self.live_data.volume_level.get() {
            ui.horizontal(|ui| {
                ui.label("Level:");
                let (color, text) = match volume.level {
                    VolumeLevel::Silent => (Color32::GRAY, "Silent"),
                    VolumeLevel::Low => (Color32::BLUE, "Low"),
                    VolumeLevel::Optimal => (Color32::GREEN, "Optimal"),
                    VolumeLevel::High => (Color32::YELLOW, "High"),
                    VolumeLevel::Clipping => (Color32::RED, "Clipping"),
                };
                ui.colored_label(color, text);
            });
            
            ui.label(format!("RMS: {:.1} dB", volume.rms_db));
            ui.label(format!("Peak: {:.1} dB", volume.peak_db));
            ui.label(format!("Peak (Fast): {:.1} dB", volume.peak_fast_db));
            ui.label(format!("Peak (Slow): {:.1} dB", volume.peak_slow_db));
            ui.label(format!("Confidence: {:.2}", volume.confidence_weight));
            
            // Volume bar visualization
            let bar_width = ui.available_width() - 100.0;
            let bar_height = 20.0;
            
            // Normalize peak_db from -60 to 0 dB
            let normalized = ((volume.peak_db + 60.0) / 60.0).clamp(0.0, 1.0);
            let bar_color = match volume.level {
                VolumeLevel::Silent => Color32::GRAY,
                VolumeLevel::Low => Color32::BLUE,
                VolumeLevel::Optimal => Color32::GREEN,
                VolumeLevel::High => Color32::YELLOW,
                VolumeLevel::Clipping => Color32::RED,
            };
            
            let (rect, _response) = ui.allocate_exact_size(Vec2::new(bar_width, bar_height), egui::Sense::hover());
            ui.painter().rect_filled(rect, 2.0, Color32::from_gray(40));
            
            let filled_width = rect.width() * normalized;
            let filled_rect = egui::Rect::from_min_size(rect.min, Vec2::new(filled_width, rect.height()));
            ui.painter().rect_filled(filled_rect, 2.0, bar_color);
            
        } else {
            ui.label("No volume data available");
        }
    }
    
    /// Render pitch detection section
    fn render_pitch_detection_section(&self, ui: &mut Ui) {
        ui.heading("Pitch Detection");
        
        if let Some(pitch) = self.live_data.pitch_data.get() {
            ui.label(format!("Frequency: {:.2} Hz", pitch.frequency));
            ui.label(format!("Note: {} ({})", pitch.note.note, pitch.note.octave));
            ui.label(format!("Cents: {:+.1}", pitch.note.cents));
            ui.label(format!("Confidence: {:.2}", pitch.confidence));
            ui.label(format!("Clarity: {:.2}", pitch.clarity));
            
            let now = js_sys::Date::now() / 1000.0; // Convert from ms to seconds
            let age = now - pitch.timestamp;
            ui.label(format!("Age: {:.1}s", age));
            
        } else {
            ui.label("No pitch data available");
        }
    }
    
    /// Render test signal controls section
    fn render_test_signal_controls_section(&mut self, ui: &mut Ui) {
        ui.heading("Test Signal Generator");
        
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.test_signal_config.enabled, "Enable Test Signal");
        });
        
        if self.test_signal_config.enabled {
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
        }
        
        ui.horizontal(|ui| {
            ui.label("Background Noise:");
            ui.add(egui::Slider::new(&mut self.background_noise_config.level, 0.0..=100.0)
                .suffix("%"));
        });
    }
    
    /// Render global audio controls section
    fn render_global_audio_controls_section(&mut self, ui: &mut Ui) {
        ui.heading("Global Audio Controls");
        
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.output_to_speakers, "Output to Speakers");
        });
        
    }
}