// Debug Data Panel
// Real-time data visualization and monitoring

use three_d::egui::{self, Color32, Vec2, Ui};
use crate::engine::audio::{
    AudioWorkletState,
    TestWaveform,
    buffer::AUDIO_CHUNK_SIZE,
};
use crate::debug::debug_data::DebugData;
use crate::shared_types::{TuningSystem, MidiNote, increment_midi_note, decrement_midi_note};

/// Convert MIDI note to display name for debugging
fn midi_note_to_display_name(midi_note: MidiNote) -> &'static str {
    match midi_note % 12 {
        0 => "C",
        1 => "C#",
        2 => "D",
        3 => "D#",
        4 => "E",
        5 => "F",
        6 => "F#",
        7 => "G",
        8 => "G#",
        9 => "A",
        10 => "A#",
        11 => "B",
        _ => unreachable!(),
    }
}
use std::rc::Rc;
use std::cell::RefCell;

pub struct DebugPanel {
    debug_data: DebugData,
    presenter: Rc<RefCell<crate::presentation::Presenter>>,
    last_metrics_update: f64,
    
    // UI state for debug controls
    test_signal_enabled: bool,
    test_signal_frequency: f32,
    test_signal_volume: f32,
    test_signal_waveform: TestWaveform,
    output_to_speakers_enabled: bool,
}

impl DebugPanel {
    pub fn new(
        debug_data: DebugData,
        presenter: Rc<RefCell<crate::presentation::Presenter>>,
    ) -> Self {
        Self {
            debug_data,
            presenter,
            last_metrics_update: 0.0,
            
            // Initialize UI state
            test_signal_enabled: false,
            test_signal_frequency: 220.0, // A3
            test_signal_volume: 50.0,
            test_signal_waveform: TestWaveform::Sine,
            output_to_speakers_enabled: false,
            
        }
    }
    
    pub fn update_data(
        &mut self,
        engine_result: &crate::shared_types::EngineUpdateResult,
        model_result: Option<&crate::shared_types::ModelUpdateResult>,
    ) {
        self.debug_data.update_from_layers(engine_result, model_result);
    }
    
    /// Update debug-specific data  
    pub fn update_debug_data(
        &mut self,
        audio_devices: Option<crate::engine::audio::AudioDevices>,
        performance_metrics: Option<crate::debug::debug_panel::data_types::PerformanceMetrics>,
        audioworklet_status: Option<crate::debug::debug_panel::data_types::AudioWorkletStatus>,
        buffer_pool_stats: Option<crate::engine::audio::message_protocol::BufferPoolStats>,
    ) {
        self.debug_data.update_debug_data(audio_devices, performance_metrics, audioworklet_status, buffer_pool_stats);
    }
    
    /// Render the live data panel
    pub fn render(&mut self, gui_context: &egui::Context, model_data: &crate::shared_types::ModelUpdateResult) {
        let screen_rect = gui_context.screen_rect();
        egui::Window::new("Debug Data")
            .default_pos([0.0, 0.0])
            .default_size(Vec2::new(400.0, screen_rect.height()))
            .resizable(true)
            .show(gui_context, |ui| {
                self.render_content(ui, model_data);
            });
    }
    
    /// Render panel content
    fn render_content(&mut self, ui: &mut Ui, model_data: &crate::shared_types::ModelUpdateResult) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical(|ui| {                
                // Root Note Audio Controls Section (debug actions)
                self.render_root_note_audio_controls(ui, model_data);

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
                
                // Accuracy Section (core data via interface)
                self.render_accuracy_section(ui);
                ui.separator();
                
                // User Actions Section (debug actions)
                self.render_user_actions_section(ui, model_data.root_note, model_data.tuning_system, model_data.root_note_audio_enabled);
                ui.separator();
                
                // Test Signal Controls Section (debug actions)
                self.render_test_signal_controls(ui);
                ui.separator();
                
                // Output to Speakers Controls Section (debug actions)
                self.render_output_to_speakers_controls(ui);
                ui.separator();
            });
        });
    }
    
    /// Render audio devices section (debug-specific data)
    fn render_audio_devices_section(&self, ui: &mut Ui) {
        let devices = &self.debug_data.audio_devices;
        
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
                let status = &self.debug_data.audioworklet_status;
                
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
                let metrics = &self.debug_data.performance_metrics;
                
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
                let stats = &self.debug_data.buffer_pool_stats;
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
                if let Some(volume) = self.debug_data.get_volume_level() {
                    ui.label(format!("RMS: {:.3}", volume.rms_amplitude));
                    ui.label(format!("Peak: {:.1}", volume.peak_amplitude));
                } else {
                    ui.label("RMS: --");
                    ui.label("Peak: --");
                }
                
                // Volume bar visualization (always present)
                let bar_width = ui.available_width() - 100.0;
                let bar_height = 20.0;
                
                let (normalized, bar_color) = if let Some(volume) = self.debug_data.get_volume_level() {
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
                if let Some(pitch) = self.debug_data.get_pitch_data() {
                    ui.label(format!("Frequency: {:.2} Hz", pitch.frequency));
                    ui.label(format!("Clarity: {:.2}", pitch.clarity));
                } else {
                    ui.label("Frequency: -- Hz");
                    ui.label("Clarity: --");
                }
            });
    }
    
    /// Render accuracy section (core data via interface)
    fn render_accuracy_section(&self, ui: &mut Ui) {
        egui::CollapsingHeader::new("Intonation")
            .default_open(true)
            .show(ui, |ui| {
                // Always reserve space for consistent height
                if let Some(intonation) = self.debug_data.get_intonation_data() {
                    // Display closest MIDI note
                    let note_name = midi_note_to_display_name(intonation.closest_midi_note);
                    let octave = (intonation.closest_midi_note as i16 / 12) - 1;
                    ui.label(format!("Closest Note: {}{}", note_name, octave));
                    
                    // Display cents offset with color coding
                    ui.horizontal(|ui| {
                        ui.label("Cents Offset:");
                        let cents = intonation.cents_offset;
                        let (color, display_text) = if cents.abs() <= 5.0 {
                            (Color32::GREEN, format!("{:+.1}", cents))
                        } else if cents.abs() <= 20.0 {
                            (Color32::YELLOW, format!("{:+.1}", cents))
                        } else {
                            (Color32::RED, format!("{:+.1}", cents))
                        };
                        ui.colored_label(color, display_text);
                    });
                    
                    // Display interval information
                    ui.horizontal(|ui| {
                        ui.label("Interval:");
                        if let (Some(interval_semitones), Some(_)) = 
                            (self.debug_data.get_interval_semitones(), self.debug_data.get_root_note()) {
                            let interval_name = crate::shared_types::interval_name_from_semitones(interval_semitones);
                            let (color, display_text) = if interval_semitones == 0 {
                                (Color32::GREEN, format!("{} ({:+} st)", interval_name, interval_semitones))
                            } else if interval_semitones.abs() == 12 || interval_semitones.abs() == 7 || interval_semitones.abs() == 5 {
                                (Color32::GREEN, format!("{} ({:+} st)", interval_name, interval_semitones))
                            } else if interval_semitones.abs() <= 12 {
                                (Color32::YELLOW, format!("{} ({:+} st)", interval_name, interval_semitones))
                            } else {
                                (Color32::from_rgb(255, 255, 255), format!("{} ({:+} st)", interval_name, interval_semitones))
                            };
                            ui.colored_label(color, display_text);
                        } else {
                            ui.label("--");
                        }
                    });
                    
                    // Display root note
                    ui.horizontal(|ui| {
                        ui.label("Root Note:");
                        if let Some(root_note) = self.debug_data.get_root_note() {
                            let root_name = midi_note_to_display_name(root_note);
                            let root_octave = (root_note as i16 / 12) - 1;
                            ui.label(format!("{}{}", root_name, root_octave));
                        } else {
                            ui.label("--");
                        }
                    });
                } else {
                    ui.label("Closest Note: --");
                    ui.label("Cents Offset: --");
                    ui.label("Interval: --");
                    ui.label("Root Note: --");
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
                    if ui.add(egui::Slider::new(&mut self.test_signal_frequency, 110.0..=440.0).suffix(" Hz")).changed() {
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
    
    /// Render root note audio controls (debug actions)
    fn render_root_note_audio_controls(&mut self, ui: &mut Ui, model_data: &crate::shared_types::ModelUpdateResult) {
        egui::CollapsingHeader::new("Root Note Audio Controls")
            .default_open(true)
            .show(ui, |ui| {
                let mut root_note_audio_enabled = model_data.root_note_audio_enabled;
                if ui.checkbox(&mut root_note_audio_enabled, "Enable Root Note Audio (plays current root note)").changed() {
                    self.send_root_note_audio_action(model_data.root_note, root_note_audio_enabled);
                }
                
                // Display current frequency when enabled
                if model_data.root_note_audio_enabled {
                    let frequency = Self::midi_note_to_frequency(model_data.root_note);
                    ui.label(format!("Frequency: {:.2} Hz", frequency));
                }
                
                ui.label("Note: Audio frequency automatically follows the root note from User Actions section");
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
    fn send_root_note_audio_action(&self, root_note: crate::shared_types::MidiNote, enabled: bool) {
        if let Ok(mut presenter) = self.presenter.try_borrow_mut() {
            presenter.on_root_note_audio_configured(enabled, root_note);
        }
    }
    
    /// Render user actions section (debug actions)
    fn render_user_actions_section(&mut self, ui: &mut Ui, current_root_note: crate::shared_types::MidiNote, current_tuning_system: crate::shared_types::TuningSystem, root_note_audio_enabled: bool) {
        egui::CollapsingHeader::new("User Actions")
            .default_open(true)
            .show(ui, |ui| {
                
                // Root Note Selection
                ui.horizontal(|ui| {
                    ui.label("Root Note:");
                    
                    // Decrement button
                    if ui.add_enabled(current_root_note > 0, egui::Button::new("-")).clicked() {
                        if let Some(new_note) = decrement_midi_note(current_root_note) {
                            self.send_root_note_action(new_note, root_note_audio_enabled);
                        }
                    }
                    
                    // Current note display
                    ui.label(format!("{}{}", midi_note_to_display_name(current_root_note), (current_root_note as i16 / 12) - 1));
                    
                    // Increment button
                    if ui.add_enabled(current_root_note < 127, egui::Button::new("+")).clicked() {
                        if let Some(new_note) = increment_midi_note(current_root_note) {
                            self.send_root_note_action(new_note, root_note_audio_enabled);
                        }
                    }
                });
                
                // Tuning System Selection
                ui.horizontal(|ui| {
                    ui.label("Tuning System:");
                    ui.push_id("tuning", |ui| {
                        let mut selected_tuning = current_tuning_system;
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{:?}", selected_tuning))
                            .show_ui(ui, |ui| {
                                let tuning_systems = [
                                    TuningSystem::EqualTemperament,
                                    TuningSystem::JustIntonation,
                                ];
                                
                                for system in &tuning_systems {
                                    if ui.selectable_value(&mut selected_tuning, system.clone(), format!("{:?}", system)).clicked() {
                                        self.send_tuning_system_action(selected_tuning.clone());
                                    }
                                }
                            })
                        });
                });
            });
    }
    
    #[cfg(debug_assertions)]
    fn send_root_note_action(&self, root_note: MidiNote, root_note_audio_enabled: bool) {
        crate::common::dev_log!("DEBUG PANEL: send_root_note_action called - root_note: {}, audio_enabled: {}", root_note, root_note_audio_enabled);
        if let Ok(mut presenter) = self.presenter.try_borrow_mut() {
            presenter.on_root_note_adjusted(root_note);
            
            // Also update root note audio frequency if it's currently enabled
            if root_note_audio_enabled {
                crate::common::dev_log!("DEBUG PANEL: Updating root note audio frequency to {} Hz", Self::midi_note_to_frequency(root_note));
                presenter.on_root_note_audio_configured(true, root_note);
            } else {
                crate::common::dev_log!("DEBUG PANEL: Root note audio is disabled, not updating frequency");
            }
        }
    }
    
    #[cfg(debug_assertions)]
    fn send_tuning_system_action(&self, tuning_system: TuningSystem) {
        if let Ok(mut presenter) = self.presenter.try_borrow_mut() {
            presenter.on_tuning_system_changed(tuning_system);
        }
    }
    
    /// Convert MIDI note number to frequency in Hz
    /// 
    /// Uses the same formula as the presentation layer for consistency
    /// 
    /// # Arguments
    /// 
    /// * `midi_note` - The MIDI note number (0-127)
    /// 
    /// # Returns
    /// 
    /// The frequency in Hz
    fn midi_note_to_frequency(midi_note: MidiNote) -> f32 {
        crate::theory::tuning::midi_note_to_standard_frequency(midi_note)
    }
}