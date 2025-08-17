// EGUI Debug Interface Module
// Handles the three-d + egui rendering for debug components
// 
// Note: Microphone permission is now handled directly from lib.rs
// Debug controls use the new presenter action collection system
// All debug functionality is only available in debug builds

pub(crate) mod data_types;

// Debug Data Panel
// Real-time data visualization and monitoring

use three_d::egui::{self, Color32, Vec2, Ui};
use crate::engine::audio::{
    AudioWorkletState,
    buffer::AUDIO_CHUNK_SIZE,
};
use crate::debug::debug_data::DebugData;
use crate::shared_types::{TuningSystem, MidiNote, increment_midi_note, decrement_midi_note};

/// Get just the note name (without octave) from a MIDI note number
fn midi_note_to_display_name(midi_note: MidiNote) -> String {
    let full_name = crate::shared_types::midi_note_to_name(midi_note);
    // Extract just the note name by removing the octave number
    let note_end = full_name.chars().position(|c| c.is_numeric() || c == '-').unwrap_or(full_name.len());
    full_name[..note_end].to_string()
}
use std::rc::Rc;
use std::cell::RefCell;

pub struct DebugPanel {
    debug_data: DebugData,
    presenter: Rc<RefCell<crate::presentation::Presenter>>,
    last_metrics_update: f64,
    
    // UI state for debug controls
    test_signal_enabled: bool,
    test_signal_volume: f32,
    test_signal_midi_note: MidiNote,
    test_signal_nudge_percent: f32,
}

impl DebugPanel {
    pub fn new(
        debug_data: DebugData,
        presenter: Rc<RefCell<crate::presentation::Presenter>>,
    ) -> Self {
        Self::with_initial_frequency(debug_data, presenter)
    }
    
    /// Create a new DebugPanel with an optional initial frequency
    pub fn with_initial_frequency(
        debug_data: DebugData,
        presenter: Rc<RefCell<crate::presentation::Presenter>>,
    ) -> Self {
        Self {
            debug_data,
            presenter,
            last_metrics_update: 0.0,
            
            // Initialize UI state
            test_signal_enabled: false,
            test_signal_volume: 15.0,
            test_signal_midi_note: crate::app_config::DEFAULT_TUNING_FORK_NOTE,
            test_signal_nudge_percent: 0.0,
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
                
                // Test Signal Controls Section (debug actions)
                self.render_test_signal_controls(ui, model_data);
                ui.separator();
                
            });
        });
    }
    
    /// Render audio devices section (debug-specific data)
    fn render_audio_devices_section(&self, ui: &mut Ui) {
        let devices = &self.debug_data.audio_devices;
        
        egui::CollapsingHeader::new("Audio Devices")
            .default_open(false)
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
            .default_open(false)
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
                
            });
    }
    
    /// Render buffer pool statistics section (debug-specific data)
    fn render_buffer_pool_stats_section(&self, ui: &mut Ui) {
        egui::CollapsingHeader::new("Buffer Pool Statistics")
            .default_open(false)
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
                let bar_width = ui.available_width() - 100.0;
                let bar_height = 20.0;
                
                // RMS Level Section
                ui.label("RMS Level");
                if let Some(volume) = self.debug_data.get_volume_level() {
                    ui.label(format!("Value: {:.3}", volume.rms_amplitude));
                } else {
                    ui.label("Value: --");
                }
                
                // RMS meter visualization
                let (rms_normalized, rms_bar_color) = if let Some(volume) = self.debug_data.get_volume_level() {
                    let rms_amplitude = volume.rms_amplitude.clamp(0.0, 1.0);
                    
                    // RMS-specific color thresholds (lower than peak)
                    let rms_color = if rms_amplitude >= 1.0 {
                        Color32::RED  // High RMS level
                    } else if rms_amplitude > 0.5 {
                        Color32::YELLOW  // Medium RMS level
                    } else {
                        Color32::GREEN  // Normal RMS level
                    };
                    
                    (rms_amplitude, rms_color)
                } else {
                    (0.0, Color32::GRAY)
                };
                
                let (rms_rect, _response) = ui.allocate_exact_size(Vec2::new(bar_width, bar_height), egui::Sense::hover());
                ui.painter().rect_filled(rms_rect, 2.0, Color32::from_gray(40));
                
                let rms_filled_width = rms_rect.width() * rms_normalized;
                let rms_filled_rect = egui::Rect::from_min_size(rms_rect.min, Vec2::new(rms_filled_width, rms_rect.height()));
                ui.painter().rect_filled(rms_filled_rect, 2.0, rms_bar_color);
                
                ui.add_space(10.0);
                
                // Peak Level Section
                ui.label("Peak Level");
                if let Some(volume) = self.debug_data.get_volume_level() {
                    ui.label(format!("Value: {:.3}", volume.peak_amplitude));
                } else {
                    ui.label("Value: --");
                }
                
                // Peak meter visualization
                let (peak_normalized, peak_bar_color) = if let Some(volume) = self.debug_data.get_volume_level() {
                    let peak_amplitude = volume.peak_amplitude.clamp(0.0, 1.0);
                    
                    // Peak-specific color thresholds
                    let peak_color = if peak_amplitude >= 1.0 {
                        Color32::RED  // Near clipping
                    } else if peak_amplitude > 0.7 {
                        Color32::YELLOW  // High level
                    } else {
                        Color32::GREEN  // Normal level
                    };
                    
                    (peak_amplitude, peak_color)
                } else {
                    (0.0, Color32::GRAY)
                };
                
                let (peak_rect, _response) = ui.allocate_exact_size(Vec2::new(bar_width, bar_height), egui::Sense::hover());
                ui.painter().rect_filled(peak_rect, 2.0, Color32::from_gray(40));
                
                let peak_filled_width = peak_rect.width() * peak_normalized;
                let peak_filled_rect = egui::Rect::from_min_size(peak_rect.min, Vec2::new(peak_filled_width, peak_rect.height()));
                ui.painter().rect_filled(peak_filled_rect, 2.0, peak_bar_color);
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
                            (self.debug_data.get_interval_semitones(), self.debug_data.get_tuning_fork_note()) {
                            let interval_name = crate::shared_types::interval_name_from_semitones(interval_semitones);
                            let (color, display_text) = if interval_semitones == 0 || interval_semitones.abs() == 12 || interval_semitones.abs() == 7 || interval_semitones.abs() == 5 {
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
                } else {
                    ui.label("Closest Note: --");
                    ui.label("Cents Offset: --");
                    ui.label("Interval: --");
                }
            });
    }
    
    /// Render test signal controls (debug actions)
    fn render_test_signal_controls(&mut self, ui: &mut Ui, model_data: &crate::shared_types::ModelUpdateResult) {
        egui::CollapsingHeader::new("Test Signal Controls")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.test_signal_enabled, "Enable Test Signal").changed() {
                        self.send_test_signal_action(model_data);
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("MIDI Note:");
                    
                    // Display current MIDI note name
                    let note_name = midi_note_to_display_name(self.test_signal_midi_note);
                    let octave = (self.test_signal_midi_note as i16 / 12) - 1;
                    ui.label(format!("{}{} ({})", note_name, octave, self.test_signal_midi_note));
                    
                    // Decrement button with bounds checking
                    let can_decrement = self.test_signal_midi_note > 0;
                    ui.add_enabled_ui(can_decrement, |ui| {
                        if ui.button("-").clicked() {
                            if let Some(new_note) = decrement_midi_note(self.test_signal_midi_note) {
                                self.test_signal_midi_note = new_note;
                                if self.test_signal_enabled {
                                    self.send_test_signal_action(model_data);
                                }
                            }
                        }
                    });
                    
                    // Increment button with bounds checking
                    let can_increment = self.test_signal_midi_note < 127;
                    ui.add_enabled_ui(can_increment, |ui| {
                        if ui.button("+").clicked() {
                            if let Some(new_note) = increment_midi_note(self.test_signal_midi_note) {
                                self.test_signal_midi_note = new_note;
                                if self.test_signal_enabled {
                                    self.send_test_signal_action(model_data);
                                }
                            }
                        }
                    });
                    
                    // Display current frequency with error handling
                    match self.calculate_midi_note_frequency_safe(
                        self.test_signal_midi_note, 
                        model_data.tuning_fork_note, 
                        model_data.tuning_system
                    ) {
                        Ok(frequency) => {
                            ui.label(format!("({:.1} Hz)", frequency));
                        }
                        Err(_) => {
                            ui.colored_label(Color32::RED, "(Error)");
                        }
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Nudge:");
                    
                    // Display current nudge percentage
                    ui.label(format!("{:+.1}%", self.test_signal_nudge_percent));
                    
                    // Decrement nudge button with bounds checking
                    let can_decrement_nudge = self.test_signal_nudge_percent > -50.0;
                    ui.add_enabled_ui(can_decrement_nudge, |ui| {
                        if ui.button("-").clicked() {
                            // Decrement by 1%, with bounds checking
                            self.test_signal_nudge_percent = (self.test_signal_nudge_percent - 1.0).max(-50.0);
                            if self.test_signal_enabled {
                                self.send_test_signal_action(model_data);
                            }
                        }
                    });
                    
                    // Increment nudge button with bounds checking
                    let can_increment_nudge = self.test_signal_nudge_percent < 50.0;
                    ui.add_enabled_ui(can_increment_nudge, |ui| {
                        if ui.button("+").clicked() {
                            // Increment by 1%, with bounds checking
                            self.test_signal_nudge_percent = (self.test_signal_nudge_percent + 1.0).min(50.0);
                            if self.test_signal_enabled {
                                self.send_test_signal_action(model_data);
                            }
                        }
                    });
                    
                    // Reset button
                    if ui.button("Reset").on_hover_text("Reset nudge to 0%").clicked() {
                        self.test_signal_nudge_percent = 0.0;
                        if self.test_signal_enabled {
                            self.send_test_signal_action(model_data);
                        }
                    }
                    
                    // Display final nudged frequency with error handling
                    match self.calculate_final_frequency_safe(
                        self.test_signal_midi_note,
                        self.test_signal_nudge_percent,
                        model_data.tuning_fork_note,
                        model_data.tuning_system
                    ) {
                        Ok((base_freq, final_freq)) => {
                            ui.label(format!("({:.1} Hz → {:.1} Hz)", base_freq, final_freq));
                        }
                        Err(_) => {
                            ui.colored_label(Color32::RED, "(Error calculating frequency)");
                        }
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Volume:");
                    
                    // Volume slider with better formatting
                    let volume_response = ui.add(
                        egui::Slider::new(&mut self.test_signal_volume, 0.0..=100.0)
                            .suffix("%")
                            .show_value(true)
                            .clamp_to_range(true)
                    );
                    
                    if volume_response.changed() && self.test_signal_enabled {
                        self.send_test_signal_action(model_data);
                    }
                    
                    // Show amplitude value as tooltip
                    let amplitude = self.test_signal_volume / 100.0;
                    volume_response.on_hover_text(format!("Amplitude: {:.3}", amplitude));
                });
                
            });
    }
    
    
    // Debug action helper methods
    
    #[cfg(debug_assertions)]
    fn send_test_signal_action(&self, model_data: &crate::shared_types::ModelUpdateResult) {
        if let Ok(mut presenter) = self.presenter.try_borrow_mut() {
            // Calculate frequency with error handling
            match self.calculate_final_frequency_safe(
                self.test_signal_midi_note,
                self.test_signal_nudge_percent,
                model_data.tuning_fork_note,
                model_data.tuning_system
            ) {
                Ok((_, final_frequency)) => {
                    // Ensure frequency is within audio range
                    let clamped_frequency = final_frequency.clamp(20.0, 20_000.0);
                    
                    presenter.on_test_signal_configured(
                        self.test_signal_enabled,
                        clamped_frequency,
                        self.test_signal_volume,
                    );
                }
                Err(e) => {
                    // Log error in debug mode
                    crate::common::warn_log!("[DEBUG_PANEL] Error calculating test signal frequency: {}", e);
                    
                    // Disable test signal on error
                    presenter.on_test_signal_configured(
                        false,
                        440.0, // Default to A4
                        self.test_signal_volume,
                    );
                }
            }
        }
    }
    
    /// Convert MIDI note to frequency considering the tuning system
    /// 
    /// # Arguments
    /// 
    /// * `midi_note` - The MIDI note number (0-127)
    /// * `tuning_fork` - The tuning fork for the tuning system
    /// * `tuning_system` - The tuning system to use
    /// 
    /// # Returns
    /// 
    /// The frequency in Hz according to the specified tuning system
    fn midi_note_to_frequency_with_tuning(
        &self,
        midi_note: MidiNote,
        tuning_fork_note: MidiNote,
        tuning_system: TuningSystem,
    ) -> f32 {
        let tuning_fork_frequency = crate::music_theory::midi_note_to_standard_frequency(tuning_fork_note);
        let interval_semitones = (midi_note as i32) - (tuning_fork_note as i32);
        crate::music_theory::interval_frequency(tuning_system, tuning_fork_frequency, interval_semitones)
    }
    
    /// Safely calculate MIDI note frequency with error handling
    fn calculate_midi_note_frequency_safe(
        &self,
        midi_note: MidiNote,
        tuning_fork_note: MidiNote,
        tuning_system: TuningSystem,
    ) -> Result<f32, &'static str> {
        // Validate MIDI notes
        if midi_note > 127 || tuning_fork_note > 127 {
            return Err("Invalid MIDI note");
        }
        
        let frequency = self.midi_note_to_frequency_with_tuning(midi_note, tuning_fork_note, tuning_system);
        
        // Validate frequency is in reasonable range
        if frequency <= 0.0 || frequency > 20_000.0 {
            return Err("Frequency out of range");
        }
        
        Ok(frequency)
    }
    
    /// Safely calculate final frequency with nudge applied
    fn calculate_final_frequency_safe(
        &self,
        midi_note: MidiNote,
        nudge_percent: f32,
        tuning_fork: MidiNote,
        tuning_system: TuningSystem,
    ) -> Result<(f32, f32), &'static str> {
        // Calculate base frequency
        let base_frequency = self.calculate_midi_note_frequency_safe(midi_note, tuning_fork, tuning_system)?;
        
        // Validate nudge percentage
        if !(-50.0..=50.0).contains(&nudge_percent) {
            return Err("Nudge percentage out of range");
        }
        
        // Calculate final frequency with nudge
        let final_frequency = base_frequency * (1.0 + nudge_percent / 100.0);
        
        // Validate final frequency
        if final_frequency <= 0.0 || final_frequency > 20_000.0 {
            return Err("Final frequency out of range");
        }
        
        Ok((base_frequency, final_frequency))
    }
}