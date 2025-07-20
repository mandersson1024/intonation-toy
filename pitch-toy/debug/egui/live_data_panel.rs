// EGUI Live Data Panel
// Real-time data visualization and monitoring for egui interface

use three_d::egui::{self, Color32, Vec2, Ui};
use std::rc::Rc;

use crate::engine::audio::{
    MusicalNote,
    AudioWorkletState, ConsoleAudioServiceImpl,
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

// Conversion functions between debug types and audio types
impl From<crate::engine::audio::VolumeLevelData> for VolumeLevelData {
    fn from(audio_data: crate::engine::audio::VolumeLevelData) -> Self {
        Self {
            rms_db: audio_data.rms_db,
            peak_db: audio_data.peak_db,
        }
    }
}

impl From<crate::engine::audio::PitchData> for PitchData {
    fn from(audio_data: crate::engine::audio::PitchData) -> Self {
        Self {
            frequency: audio_data.frequency,
            confidence: audio_data.confidence,
            note: audio_data.note,
            clarity: audio_data.clarity,
            timestamp: audio_data.timestamp,
        }
    }
}

impl From<crate::engine::audio::AudioWorkletStatus> for AudioWorkletStatus {
    fn from(audio_data: crate::engine::audio::AudioWorkletStatus) -> Self {
        Self {
            state: audio_data.state,
            processor_loaded: audio_data.processor_loaded,
            chunk_size: audio_data.chunk_size,
            chunks_processed: audio_data.chunks_processed,
            last_update: audio_data.last_update,
        }
    }
}


/// EGUI Live Data Panel - Real-time audio monitoring and control interface
pub struct EguiLiveDataPanel {
    live_data: LiveData,
    last_metrics_update: f64,
}

impl EguiLiveDataPanel {
    /// Create new EGUI Live Data Panel
    pub fn new(
        live_data: LiveData,
    ) -> Self {
            
        Self {
            live_data,
            last_metrics_update: 0.0,
        }
    }
    
    
    /// Apply output to speakers setting to audio system
    
    /// Check for configuration changes and apply them
    fn check_and_apply_changes(&mut self) {
        // Check output to speakers changes
    }
    
    /// Render the live data panel
    pub fn render(&mut self, gui_context: &egui::Context) {
        let screen_rect = gui_context.screen_rect();
        egui::Window::new("Live Data Panel")
            .default_pos([0.0, 0.0])
            .default_size(Vec2::new(400.0, screen_rect.height()))
            .resizable(true)
            .show(gui_context, |ui| {
                self.render_content(ui);
                
                // Check for changes and apply them after rendering
                self.check_and_apply_changes();
            });
    }
    
    /// Render panel content
    fn render_content(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
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
                
                // Buffer Pool Statistics Section
                self.render_buffer_pool_stats_section(ui);
                ui.separator();
                
                // Volume Level Section
                self.render_volume_level_section(ui);
                ui.separator();
                
                // Pitch Detection Section
                self.render_pitch_detection_section(ui);
                ui.separator();
                
                // Test Signal Controls Section
                ui.separator();
                
                // Global Audio Controls Section
                    });
        });
    }
    
    /// Render audio devices section
    fn render_audio_devices_section(&self, ui: &mut Ui) {
        let devices = self.live_data.audio_devices.get();
        
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
    
    /// Render AudioWorklet status section
    fn render_audioworklet_status_section(&self, ui: &mut Ui) {
        egui::CollapsingHeader::new("AudioWorklet Status")
            .default_open(true)
            .show(ui, |ui| {
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
            });
    }
    
    /// Render performance metrics section
    fn render_performance_metrics_section(&mut self, ui: &mut Ui) {
        egui::CollapsingHeader::new("Performance Metrics")
            .default_open(true)
            .show(ui, |ui| {
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
            });
    }
    
    /// Render buffer pool statistics section
    fn render_buffer_pool_stats_section(&self, ui: &mut Ui) {
        egui::CollapsingHeader::new("Buffer Pool Statistics")
            .default_open(true)
            .show(ui, |ui| {
        
        // Buffer pool statistics are now updated reactively via periodic status updates
        // No manual request needed - just consume the reactive data
        let stats = self.live_data.buffer_pool_stats.get();
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
                
                // Pool efficiency metrics
                ui.horizontal(|ui| {
                    ui.label("Hit Rate:");
                    let hit_rate_color = if stats.pool_hit_rate > 90.0 {
                        Color32::GREEN
                    } else if stats.pool_hit_rate > 75.0 {
                        Color32::YELLOW
                    } else {
                        Color32::RED
                    };
                    ui.colored_label(hit_rate_color, format!("{:.1}%", stats.pool_hit_rate));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Efficiency:");
                    let efficiency_color = if stats.pool_efficiency > 90.0 {
                        Color32::GREEN
                    } else if stats.pool_efficiency > 75.0 {
                        Color32::YELLOW
                    } else {
                        Color32::RED
                    };
                    ui.colored_label(efficiency_color, format!("{:.1}%", stats.pool_efficiency));
                });
                
                // Usage statistics
                ui.horizontal(|ui| {
                    ui.label("Transfers:");
                    ui.label(format!("{}", stats.transfer_count));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Exhausted:");
                    let exhausted_color = if stats.pool_exhausted_count == 0 {
                        Color32::GREEN
                    } else if stats.pool_exhausted_count < 10 {
                        Color32::YELLOW
                    } else {
                        Color32::RED
                    };
                    ui.colored_label(exhausted_color, format!("{}", stats.pool_exhausted_count));
                });
                
                // Performance metrics
                ui.horizontal(|ui| {
                    ui.label("Avg Acquisition:");
                    ui.label(format!("{:.2}ms", stats.avg_acquisition_time_ms));
                });
                
                
                // Data transfer stats
                ui.horizontal(|ui| {
                    ui.label("Data Transferred:");
                    ui.label(format!("{:.2} MB", stats.total_megabytes_transferred));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Utilization:");
                    ui.label(format!("{:.1}%", stats.buffer_utilization_percent));
                });
                
        } else {
            // Pool status
            ui.horizontal(|ui| {
                ui.label("Pool Status:");
                ui.colored_label(Color32::GRAY, "");
            });
            
            // Pool efficiency metrics
            ui.horizontal(|ui| {
                ui.label("Hit Rate:");
                ui.colored_label(Color32::GRAY, "");
            });
            
            ui.horizontal(|ui| {
                ui.label("Efficiency:");
                ui.colored_label(Color32::GRAY, "");
            });
            
            // Usage statistics
            ui.horizontal(|ui| {
                ui.label("Transfers:");
                ui.label("");
            });
            
            ui.horizontal(|ui| {
                ui.label("Exhausted:");
                ui.colored_label(Color32::GRAY, "");
            });
            
            // Performance metrics
            ui.horizontal(|ui| {
                ui.label("Avg Acquisition:");
                ui.label("ms");
            });
            
            // Data transfer stats
            ui.horizontal(|ui| {
                ui.label("Data Transferred:");
                ui.label("MB");
            });
            
            ui.horizontal(|ui| {
                ui.label("Utilization:");
                ui.label("%");
            });
        }
            });
    }
    
    /// Render volume level section
    fn render_volume_level_section(&self, ui: &mut Ui) {
        egui::CollapsingHeader::new("Volume Level")
            .default_open(true)
            .show(ui, |ui| {
                if let Some(volume) = self.live_data.volume_level.get() {
                    
                    ui.label(format!("RMS: {:.1} dB", volume.rms_db));
                    ui.label(format!("Peak: {:.1} dB", volume.peak_db));
                    
                    // Volume bar visualization
                    let bar_width = ui.available_width() - 100.0;
                    let bar_height = 20.0;
                    
                    // Convert dB to amplitude (amplitude = 10^(dB/20))
                    // For -60 dB: amplitude = 10^(-60/20) = 10^(-3) = 0.001
                    // For 0 dB: amplitude = 10^(0/20) = 1.0
                    let amplitude = 10.0_f32.powf(volume.peak_db / 20.0);
                    let normalized = amplitude.clamp(0.0, 1.0);
                    let bar_color = Color32::GREEN;
                    
                    let (rect, _response) = ui.allocate_exact_size(Vec2::new(bar_width, bar_height), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, Color32::from_gray(40));
                    
                    let filled_width = rect.width() * normalized;
                    let filled_rect = egui::Rect::from_min_size(rect.min, Vec2::new(filled_width, rect.height()));
                    ui.painter().rect_filled(filled_rect, 2.0, bar_color);
                    
                } else {
                    ui.label("RMS:  dB");
                    ui.label("Peak:  dB");
                    
                    // Volume bar visualization (empty)
                    let bar_width = ui.available_width() - 100.0;
                    let bar_height = 20.0;
                    
                    let (rect, _response) = ui.allocate_exact_size(Vec2::new(bar_width, bar_height), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, Color32::from_gray(40));
                }
            });
    }
    
    /// Render pitch detection section
    fn render_pitch_detection_section(&self, ui: &mut Ui) {
        egui::CollapsingHeader::new("Pitch Detection")
            .default_open(true)
            .show(ui, |ui| {
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
                    ui.label("Frequency:  Hz");
                    ui.label("Note:  ()");
                    ui.label("Cents: ");
                    ui.label("Confidence: ");
                    ui.label("Clarity: ");
                    ui.label("Age: s");
                }
            });
    }
    
    
}

// Adapter setters that convert between debug types and audio types
pub struct VolumeDataAdapter {
    inner: std::rc::Rc<dyn observable_data::DataSetter<Option<VolumeLevelData>>>,
}

impl VolumeDataAdapter {
    pub fn new(inner: std::rc::Rc<dyn observable_data::DataSetter<Option<VolumeLevelData>>>) -> Self {
        Self { inner }
    }
}

unsafe impl Send for VolumeDataAdapter {}
unsafe impl Sync for VolumeDataAdapter {}

impl observable_data::DataSetter<Option<crate::engine::audio::VolumeLevelData>> for VolumeDataAdapter {
    fn set(&self, data: Option<crate::engine::audio::VolumeLevelData>) {
        let converted = data.map(|d| d.into());
        self.inner.set(converted);
    }
}

pub struct PitchDataAdapter {
    inner: std::rc::Rc<dyn observable_data::DataSetter<Option<PitchData>>>,
}

impl PitchDataAdapter {
    pub fn new(inner: std::rc::Rc<dyn observable_data::DataSetter<Option<PitchData>>>) -> Self {
        Self { inner }
    }
}

unsafe impl Send for PitchDataAdapter {}
unsafe impl Sync for PitchDataAdapter {}

impl observable_data::DataSetter<Option<crate::engine::audio::PitchData>> for PitchDataAdapter {
    fn set(&self, data: Option<crate::engine::audio::PitchData>) {
        let converted = data.map(|d| d.into());
        self.inner.set(converted);
    }
}

pub struct AudioWorkletStatusAdapter {
    inner: std::rc::Rc<dyn observable_data::DataSetter<AudioWorkletStatus>>,
}

impl AudioWorkletStatusAdapter {
    pub fn new(inner: std::rc::Rc<dyn observable_data::DataSetter<AudioWorkletStatus>>) -> Self {
        Self { inner }
    }
}

unsafe impl Send for AudioWorkletStatusAdapter {}
unsafe impl Sync for AudioWorkletStatusAdapter {}

impl observable_data::DataSetter<crate::engine::audio::AudioWorkletStatus> for AudioWorkletStatusAdapter {
    fn set(&self, data: crate::engine::audio::AudioWorkletStatus) {
        let converted = data.into();
        self.inner.set(converted);
    }
}