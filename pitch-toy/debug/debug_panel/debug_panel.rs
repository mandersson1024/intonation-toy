// Hybrid EGUI Live Data Panel
// Real-time data visualization and monitoring for egui interface using hybrid architecture

use three_d::egui::{self, Color32, Vec2, Ui};
use crate::engine::audio::{
    AudioWorkletState,
    TestWaveform,
    buffer::AUDIO_CHUNK_SIZE,
};
use crate::debug::debug_data::DebugData;
use crate::shared_types::{NoteName, TuningSystem};
use std::rc::Rc;
use std::cell::RefCell;

/// Hybrid EGUI Live Data Panel - Real-time audio monitoring and control interface using hybrid architecture
pub struct DebugPanel {
    hybrid_data: DebugData,
    presenter: Rc<RefCell<crate::presentation::Presenter>>,
    last_metrics_update: f64,
    
    // UI state for debug controls
    test_signal_enabled: bool,
    test_signal_frequency: f32,
    test_signal_volume: f32,
    test_signal_waveform: TestWaveform,
    output_to_speakers_enabled: bool,
    background_noise_enabled: bool,
    background_noise_level: f32,
    background_noise_type: TestWaveform,
    
    // UI state for user actions
    selected_root_note: NoteName,
    selected_tuning_system: TuningSystem,
}

impl DebugPanel {
    /// Create new Hybrid EGUI Live Data Panel
    pub fn new(
        hybrid_data: DebugData,
        presenter: Rc<RefCell<crate::presentation::Presenter>>,
    ) -> Self {
        Self {
            hybrid_data,
            presenter,
            last_metrics_update: 0.0,
            
            // Initialize UI state
            test_signal_enabled: false,
            test_signal_frequency: 440.0,
            test_signal_volume: 50.0,
            test_signal_waveform: TestWaveform::Sine,
            output_to_speakers_enabled: false,
            background_noise_enabled: false,
            background_noise_level: 0.1,
            background_noise_type: TestWaveform::WhiteNoise,
            
            // Initialize user action state
            selected_root_note: NoteName::A,
            selected_tuning_system: TuningSystem::EqualTemperament,
        }
    }
    
    /// Update the hybrid data with engine and model results
    pub fn update_data(
        &mut self,
        engine_result: &crate::shared_types::EngineUpdateResult,
        model_result: Option<&crate::shared_types::ModelUpdateResult>,
    ) {
        self.hybrid_data.update_from_layers(engine_result, model_result);
    }
    
    /// Update debug-specific data  
    pub fn update_debug_data(
        &mut self,
        audio_devices: Option<crate::engine::audio::AudioDevices>,
        performance_metrics: Option<crate::debug::debug_panel::data_types::PerformanceMetrics>,
        audioworklet_status: Option<crate::debug::debug_panel::data_types::AudioWorkletStatus>,
        buffer_pool_stats: Option<crate::engine::audio::message_protocol::BufferPoolStats>,
    ) {
        self.hybrid_data.update_debug_data(audio_devices, performance_metrics, audioworklet_status, buffer_pool_stats);
    }
    
    /// Render the live data panel
    pub fn render(&mut self, gui_context: &egui::Context) {
        let screen_rect = gui_context.screen_rect();
        egui::Window::new("Hybrid Live Data Panel")
            .default_pos([0.0, 0.0])
            .default_size(Vec2::new(400.0, screen_rect.height()))
            .resizable(true)
            .show(gui_context, |ui| {
                self.render_content(ui);
            });
    }
    
    /// Render panel content
    fn render_content(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical(|ui| {
                // Audio Devices Section (debug-specific data)
                self.render_audio_devices_section(ui);
                ui.separator();
                
                // AudioWorklet Status Section (debug-specific data)
                self.render_audioworklet_status_section(ui);
                ui.separator();
                
                // Performance Metrics Section (debug-specific data)
                self.render_performance_metrics_section(ui);
                ui.separator();
                
                // Buffer Pool Statistics Section (debug-specific data)
                self.render_buffer_pool_stats_section(ui);
                ui.separator();
                
                // Volume Level Section (core data via interface)
                self.render_volume_level_section(ui);
                ui.separator();
                
                // Pitch Detection Section (core data via interface)
                self.render_pitch_detection_section(ui);
                ui.separator();
                
                // User Actions Section (debug actions)
                self.render_user_actions_section(ui);
                ui.separator();
                
                // Test Signal Controls Section (debug actions)
                self.render_test_signal_controls(ui);
                ui.separator();
                
                // Output to Speakers Controls Section (debug actions)
                self.render_output_to_speakers_controls(ui);
                ui.separator();
                
                // Background Noise Controls Section (debug actions)
                self.render_background_noise_controls(ui);
            });
        });
    }
    
    /// Render audio devices section (debug-specific data)
    fn render_audio_devices_section(&self, ui: &mut Ui) {
        let devices = &self.hybrid_data.audio_devices;
        
        egui::CollapsingHeader::new("Audio Devices")
            .default_open(true)
            .show(ui, |ui| {
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
            });
    }
    
    /// Render AudioWorklet status section (debug-specific data)
    fn render_audioworklet_status_section(&self, ui: &mut Ui) {
        egui::CollapsingHeader::new("AudioWorklet Status")
            .default_open(true)
            .show(ui, |ui| {
                let status = &self.hybrid_data.audioworklet_status;
                
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
                
                ui.label(format!("Batch Size: {} samples ({} chunks of {})", status.batch_size, status.batch_size / AUDIO_CHUNK_SIZE as u32, AUDIO_CHUNK_SIZE));
                ui.label(format!("Batches Processed: {}", status.batches_processed));
            });
    }
    
    /// Render performance metrics section (debug-specific data)
    fn render_performance_metrics_section(&mut self, ui: &mut Ui) {
        egui::CollapsingHeader::new("Performance Metrics")
            .default_open(true)
            .show(ui, |ui| {
                let metrics = &self.hybrid_data.performance_metrics;
                
                // Update metrics periodically
                let now = js_sys::Date::now() / 1000.0; // Convert from ms to seconds
                if now - self.last_metrics_update > 1.0 {
                    self.last_metrics_update = now;
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
                    let color = if metrics.memory_usage_mb < 100.0 { Color32::GREEN } 
                               else if metrics.memory_usage_mb < 200.0 { Color32::YELLOW } 
                               else { Color32::RED };
                    ui.colored_label(color, format!("{:.1} MB", metrics.memory_usage_mb));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Heap Usage:");
                    let color = if metrics.memory_usage_percent < 50.0 { Color32::GREEN } 
                               else if metrics.memory_usage_percent < 80.0 { Color32::YELLOW } 
                               else { Color32::RED };
                    ui.colored_label(color, format!("{:.1}%", metrics.memory_usage_percent));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Audio Latency:");
                    let color = if metrics.audio_latency < 20.0 { Color32::GREEN } 
                               else if metrics.audio_latency < 50.0 { Color32::YELLOW } 
                               else { Color32::RED };
                    ui.colored_label(color, format!("{:.1}ms", metrics.audio_latency));
                });
            });
    }
    
    /// Render buffer pool statistics section (debug-specific data)
    fn render_buffer_pool_stats_section(&self, ui: &mut Ui) {
        egui::CollapsingHeader::new("Buffer Pool Statistics")
            .default_open(true)
            .show(ui, |ui| {
                let stats = &self.hybrid_data.buffer_pool_stats;
                if let Some(stats) = stats {
                    // Pool status
                    ui.horizontal(|ui| {
                        ui.label("Pool Status:");
                        let status_color = if stats.available_buffers > 0 {
                            Color32::GREEN
                        } else {
                            Color32::RED
                        };
                        ui.colored_label(status_color, format!("{}/{} available", 
                                                              stats.available_buffers, 
                                                              stats.pool_size));
                    });
                    
                    // Pool allocation failures
                    ui.horizontal(|ui| {
                        ui.label("Allocation Failures:");
                        let fail_color = if stats.pool_exhausted_count == 0 {
                            Color32::GREEN
                        } else if stats.pool_exhausted_count < 10 {
                            Color32::YELLOW
                        } else {
                            Color32::RED
                        };
                        ui.colored_label(fail_color, format!("{}", stats.pool_exhausted_count));
                    });
                    
                    // Additional stats
                    ui.label(format!("Data Transferred: {:.2} MB", stats.total_megabytes_transferred));
                } else {
                    ui.label("No buffer pool statistics available");
                }
            });
    }
    
    /// Render volume level section (core data via interface)
    fn render_volume_level_section(&self, ui: &mut Ui) {
        egui::CollapsingHeader::new("Volume Level")
            .default_open(true)
            .show(ui, |ui| {
                // Always reserve space for consistent height
                if let Some(volume) = self.hybrid_data.get_volume_level() {
                    ui.label(format!("RMS: {:.3}", volume.rms_amplitude));
                    ui.label(format!("Peak: {:.1}", volume.peak_amplitude));
                } else {
                    ui.label("RMS: --");
                    ui.label("Peak: --");
                }
                
                // Volume bar visualization (always present)
                let bar_width = ui.available_width() - 100.0;
                let bar_height = 20.0;
                
                let (normalized, bar_color) = if let Some(volume) = self.hybrid_data.get_volume_level() {
                    // Use peak amplitude directly (0.0 to 1.0)
                    let amplitude = volume.peak_amplitude;
                    
                    // Clamp to 0-1 range
                    let normalized = amplitude.clamp(0.0, 1.0);
                    
                    // Color based on amplitude level
                    let bar_color = if normalized > 0.9 {
                        Color32::RED  // Near clipping
                    } else if normalized > 0.7 {
                        Color32::YELLOW  // High level
                    } else {
                        Color32::GREEN  // Normal level
                    };
                    
                    (normalized, bar_color)
                } else {
                    (0.0, Color32::GRAY)
                };
                
                let (rect, _response) = ui.allocate_exact_size(Vec2::new(bar_width, bar_height), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, Color32::from_gray(40));
                
                let filled_width = rect.width() * normalized;
                let filled_rect = egui::Rect::from_min_size(rect.min, Vec2::new(filled_width, rect.height()));
                ui.painter().rect_filled(filled_rect, 2.0, bar_color);
            });
    }
    
    /// Render pitch detection section (core data via interface)
    fn render_pitch_detection_section(&self, ui: &mut Ui) {
        egui::CollapsingHeader::new("Pitch Detection")
            .default_open(true)
            .show(ui, |ui| {
                // Always reserve space for consistent height
                if let Some(pitch) = self.hybrid_data.get_pitch_data() {
                    ui.label(format!("Frequency: {:.2} Hz", pitch.frequency));
                    ui.label(format!("Clarity: {:.2}", pitch.clarity));
                } else {
                    ui.label("Frequency: -- Hz");
                    ui.label("Clarity: --");
                }
            });
    }
    
    /// Render test signal controls (debug actions)
    fn render_test_signal_controls(&mut self, ui: &mut Ui) {
        egui::CollapsingHeader::new("Test Signal Controls")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.test_signal_enabled, "Enable Test Signal").changed() {
                        self.send_test_signal_action();
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Frequency:");
                    if ui.add(egui::Slider::new(&mut self.test_signal_frequency, 50.0..=8000.0).suffix(" Hz")).changed() {
                        if self.test_signal_enabled {
                            self.send_test_signal_action();
                        }
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Volume:");
                    if ui.add(egui::Slider::new(&mut self.test_signal_volume, 0.0..=100.0).suffix("%")).changed() {
                        if self.test_signal_enabled {
                            self.send_test_signal_action();
                        }
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Waveform:");
                    egui::ComboBox::from_label("")
                        .selected_text(format!("{:?}", self.test_signal_waveform))
                        .show_ui(ui, |ui| {
                            let waveforms = [
                                TestWaveform::Sine,
                                TestWaveform::Square,
                                TestWaveform::Triangle,
                                TestWaveform::Sawtooth,
                            ];
                            
                            for waveform in &waveforms {
                                if ui.selectable_value(&mut self.test_signal_waveform, waveform.clone(), format!("{:?}", waveform)).clicked() {
                                    if self.test_signal_enabled {
                                        self.send_test_signal_action();
                                    }
                                }
                            }
                        });
                });
            });
    }
    
    /// Render output to speakers controls (debug actions)
    fn render_output_to_speakers_controls(&mut self, ui: &mut Ui) {
        egui::CollapsingHeader::new("Output to Speakers")
            .default_open(true)
            .show(ui, |ui| {
                if ui.checkbox(&mut self.output_to_speakers_enabled, "Enable Output to Speakers").changed() {
                    self.send_output_to_speakers_action();
                }
            });
    }
    
    /// Render background noise controls (debug actions)
    fn render_background_noise_controls(&mut self, ui: &mut Ui) {
        egui::CollapsingHeader::new("Background Noise Controls")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.background_noise_enabled, "Enable Background Noise").changed() {
                        self.send_background_noise_action();
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Level:");
                    if ui.add(egui::Slider::new(&mut self.background_noise_level, 0.0..=1.0)).changed() {
                        if self.background_noise_enabled {
                            self.send_background_noise_action();
                        }
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_label("")
                        .selected_text(format!("{:?}", self.background_noise_type))
                        .show_ui(ui, |ui| {
                            let noise_types = [
                                TestWaveform::WhiteNoise,
                                TestWaveform::PinkNoise,
                            ];
                            
                            for noise_type in &noise_types {
                                if ui.selectable_value(&mut self.background_noise_type, noise_type.clone(), format!("{:?}", noise_type)).clicked() {
                                    if self.background_noise_enabled {
                                        self.send_background_noise_action();
                                    }
                                }
                            }
                        });
                });
            });
    }
    
    // Debug action helper methods
    
    #[cfg(debug_assertions)]
    fn send_test_signal_action(&self) {
        if let Ok(mut presenter) = self.presenter.try_borrow_mut() {
            presenter.on_test_signal_configured(
                self.test_signal_enabled,
                self.test_signal_frequency,
                self.test_signal_volume,
                self.test_signal_waveform.clone(),
            );
        }
    }
    
    #[cfg(debug_assertions)]
    fn send_output_to_speakers_action(&self) {
        if let Ok(mut presenter) = self.presenter.try_borrow_mut() {
            presenter.on_output_to_speakers_configured(self.output_to_speakers_enabled);
        }
    }
    
    #[cfg(debug_assertions)]
    fn send_background_noise_action(&self) {
        if let Ok(mut presenter) = self.presenter.try_borrow_mut() {
            presenter.on_background_noise_configured(
                self.background_noise_enabled,
                self.background_noise_level,
                self.background_noise_type.clone(),
            );
        }
    }
    
    /// Render user actions section (debug actions)
    fn render_user_actions_section(&mut self, ui: &mut Ui) {
        egui::CollapsingHeader::new("User Actions")
            .default_open(true)
            .show(ui, |ui| {
                // Root Note Selection
                ui.horizontal(|ui| {
                    ui.label("Root Note:");
                    ui.push_id("note", |ui| {
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{:?}", self.selected_root_note))
                            .show_ui(ui, |ui| {
                                let notes = [
                                    NoteName::C,
                                    NoteName::DFlat,
                                    NoteName::D,
                                    NoteName::EFlat,
                                    NoteName::E,
                                    NoteName::F,
                                    NoteName::FSharp,
                                    NoteName::G,
                                    NoteName::AFlat,
                                    NoteName::A,
                                    NoteName::BFlat,
                                    NoteName::B,
                                ];
                                
                                for note in &notes {
                                    if ui.selectable_value(&mut self.selected_root_note, note.clone(), format!("{:?}", note)).clicked() {
                                        self.send_root_note_action();
                                    }
                                }
                            })
                        });
                });
                
                // Tuning System Selection
                ui.horizontal(|ui| {
                    ui.label("Tuning System:");
                    ui.push_id("tuning", |ui| {
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{:?}", self.selected_tuning_system))
                            .show_ui(ui, |ui| {
                                let tuning_systems = [
                                    TuningSystem::EqualTemperament,
                                    TuningSystem::JustIntonation,
                                ];
                                
                                for system in &tuning_systems {
                                    if ui.selectable_value(&mut self.selected_tuning_system, system.clone(), format!("{:?}", system)).clicked() {
                                        self.send_tuning_system_action();
                                    }
                                }
                            })
                        });
                });
            });
    }
    
    #[cfg(debug_assertions)]
    fn send_root_note_action(&self) {
        if let Ok(mut presenter) = self.presenter.try_borrow_mut() {
            presenter.on_root_note_adjusted(self.selected_root_note.clone());
        }
    }
    
    #[cfg(debug_assertions)]
    fn send_tuning_system_action(&self) {
        if let Ok(mut presenter) = self.presenter.try_borrow_mut() {
            presenter.on_tuning_system_changed(self.selected_tuning_system.clone());
        }
    }
}