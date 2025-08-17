use crate::engine::audio::{
    AudioPermission,
    AudioDevices,
};
use crate::debug::debug_panel::data_types::{PerformanceMetrics, VolumeLevelData, PitchData, AudioWorkletStatus};
use crate::shared_types::{EngineUpdateResult, PermissionState, ModelUpdateResult, IntonationData};

/// HybridLiveData structure that holds actual data instead of observers
/// Updated for Task 8a to work with the new update return struct pattern
#[derive(Clone)]
pub struct DebugData {
    // Debug-specific data (direct access)
    pub audio_devices: AudioDevices,
    pub performance_metrics: PerformanceMetrics,
    pub audioworklet_status: AudioWorkletStatus,
    pub buffer_pool_stats: Option<crate::engine::audio::message_protocol::BufferPoolStats>,
    
    // Core data from engine and model layers
    pub volume_level: Option<VolumeLevelData>,
    pub pitch_data: Option<PitchData>,
    pub intonation_data: Option<IntonationData>,
    pub microphone_permission: AudioPermission,
    pub audio_errors: Vec<crate::shared_types::Error>,
    pub interval_semitones: Option<i32>,
    pub tuning_fork_note: Option<crate::shared_types::MidiNote>,
}

impl Default for DebugData {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugData {
    /// Create new HybridLiveData with default values
    pub fn new() -> Self {
        Self {
            // Debug-specific data
            audio_devices: AudioDevices {
                input_devices: Vec::new(),
                output_devices: Vec::new(),
            },
            performance_metrics: PerformanceMetrics::default(),
            audioworklet_status: AudioWorkletStatus::default(),
            buffer_pool_stats: None,
            
            // Core data
            volume_level: None,
            pitch_data: None,
            intonation_data: None,
            microphone_permission: AudioPermission::Uninitialized,
            audio_errors: Vec::new(),
            interval_semitones: None,
            tuning_fork_note: None,
        }
    }

    /// Update core data from engine and model results
    /// This method receives data from the main update loop
    pub fn update_from_layers(
        &mut self,
        engine_result: &EngineUpdateResult,
        model_result: Option<&ModelUpdateResult>,
    ) {
        // Update permission state from engine
        self.microphone_permission = match engine_result.permission_state {
            PermissionState::NotRequested => AudioPermission::Uninitialized,
            PermissionState::Requested => AudioPermission::Requesting,
            PermissionState::Granted => AudioPermission::Granted,
            PermissionState::Denied => AudioPermission::Denied,
        };
        
        // Update audio errors from engine
        self.audio_errors = engine_result.audio_errors.clone();
        
        // Update volume and pitch data from engine analysis
        if let Some(analysis) = &engine_result.audio_analysis {
            // Convert Volume to VolumeLevelData
            // Note: both peak and rms are amplitude values (0.0-1.0)
            self.volume_level = Some(VolumeLevelData {
                peak_amplitude: analysis.volume_level.peak_amplitude,
                rms_amplitude: analysis.volume_level.rms_amplitude,
            });
            
            // Convert Pitch to PitchData
            self.pitch_data = match &analysis.pitch {
                crate::shared_types::Pitch::Detected(frequency, clarity) => {
                    Some(PitchData {
                        frequency: *frequency,
                        clarity: *clarity,
                        timestamp: analysis.timestamp,
                    })
                },
                crate::shared_types::Pitch::NotDetected => None,
            };
        } else {
            self.volume_level = None;
            self.pitch_data = None;
        }
        
        // Update accuracy data from model result if available
        if let Some(model) = model_result {
            self.intonation_data = Some(model.accuracy.clone());
            self.interval_semitones = Some(model.interval_semitones);
            self.tuning_fork_note = Some(model.tuning_fork_note);
        }
    }
    
    /// Update debug-specific data (called separately by debug systems)
    pub fn update_debug_data(
        &mut self,
        audio_devices: Option<AudioDevices>,
        performance_metrics: Option<PerformanceMetrics>,
        audioworklet_status: Option<AudioWorkletStatus>,
        buffer_pool_stats: Option<crate::engine::audio::message_protocol::BufferPoolStats>,
    ) {
        if let Some(devices) = audio_devices {
            self.audio_devices = devices;
        }
        if let Some(metrics) = performance_metrics {
            self.performance_metrics = metrics;
        }
        if let Some(status) = audioworklet_status {
            self.audioworklet_status = status;
        }
        if let Some(stats) = buffer_pool_stats {
            self.buffer_pool_stats = Some(stats);
        }
    }

    /// Get volume level data
    pub fn get_volume_level(&self) -> Option<VolumeLevelData> {
        self.volume_level.clone()
    }

    /// Get pitch data
    pub fn get_pitch_data(&self) -> Option<PitchData> {
        self.pitch_data.clone()
    }

    /// Get accuracy data
    pub fn get_intonation_data(&self) -> Option<IntonationData> {
        self.intonation_data.clone()
    }
    
    /// Get interval semitones
    pub fn get_interval_semitones(&self) -> Option<i32> {
        self.interval_semitones
    }
    
    /// Get tuning fork
    pub fn get_tuning_fork_note(&self) -> Option<crate::shared_types::MidiNote> {
        self.tuning_fork_note
    }
}