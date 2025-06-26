// Audio Events for Event Bus Integration - STORY-013

use std::any::Any;
use std::fmt;
use crate::modules::application_core::event_bus::{Event, EventPriority};
use crate::legacy::hooks::use_microphone_permission::PermissionState;

/// Get current timestamp in nanoseconds
pub fn get_timestamp_ns() -> u64 {
    // Use performance.now() if available, otherwise use current time
    #[cfg(target_arch = "wasm32")]
    {
        (web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now() * 1_000_000.0) // Convert ms to ns
            .unwrap_or(0.0)) as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}

/// Pitch detection result event - Enhanced for STORY-015
#[derive(Debug, Clone)]
pub struct PitchDetectionEvent {
    pub frequency: f32,
    pub confidence: f32,
    pub clarity: f32,
    pub harmonic_content: f32,
    pub algorithm_used: crate::modules::audio_foundations::multi_algorithm_pitch_detector::PitchAlgorithm,
    pub processing_time_ns: u64,
    pub timestamp_ns: u64,
    pub source_buffer_ref: String,
    pub snr_estimate: f32,
    pub is_valid: bool,
}

impl Event for PitchDetectionEvent {
    fn event_type(&self) -> &'static str {
        "PitchDetectionEvent"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp_ns
    }
    
    fn priority(&self) -> EventPriority {
        EventPriority::Critical // High priority for real-time audio
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Audio processing state change event
#[derive(Debug, Clone)]
pub struct AudioProcessingStateEvent {
    pub old_state: crate::modules::audio_foundations::AudioEngineState,
    pub new_state: crate::modules::audio_foundations::AudioEngineState,
    pub timestamp_ns: u64,
    pub context: String,
}

impl Event for AudioProcessingStateEvent {
    fn event_type(&self) -> &'static str {
        "AudioProcessingStateEvent"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp_ns
    }
    
    fn priority(&self) -> EventPriority {
        EventPriority::High
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}


/// Audio performance metrics event
#[derive(Debug, Clone)]
pub struct AudioPerformanceMetricsEvent {
    pub end_to_end_latency_ms: f32,
    pub processing_latency_ms: f32,
    pub cpu_usage_percent: f32,
    pub memory_usage_bytes: usize,
    pub dropout_count: u32,
    pub buffer_underruns: u32,
    pub timestamp_ns: u64,
}

impl Event for AudioPerformanceMetricsEvent {
    fn event_type(&self) -> &'static str {
        "AudioPerformanceMetricsEvent"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp_ns
    }
    
    fn priority(&self) -> EventPriority {
        EventPriority::Normal // Non-critical for monitoring
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Audio error event
#[derive(Debug, Clone)]
pub struct AudioErrorEvent {
    pub error_type: AudioErrorType,
    pub message: String,
    pub context: String,
    pub recovery_suggestion: Option<String>,
    pub timestamp_ns: u64,
}

impl Event for AudioErrorEvent {
    fn event_type(&self) -> &'static str {
        "AudioErrorEvent"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp_ns
    }
    
    fn priority(&self) -> EventPriority {
        match self.error_type {
            AudioErrorType::Critical => EventPriority::Critical,
            AudioErrorType::Warning => EventPriority::High,
            AudioErrorType::Info => EventPriority::Normal,
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Supporting data structures

/// Signal analysis information
#[derive(Debug, Clone)]
pub struct SignalInfo {
    pub amplitude: f32,
    pub clarity: f32,
    pub harmonic_content: f32,
    pub noise_floor: f32,
}

/// Device connection state
#[derive(Debug, Clone)]
pub enum DeviceState {
    Connected,
    Disconnected,
    Error(String),
}

/// Audio device information
#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub sample_rate: f32,
    pub buffer_size: usize,
    pub channels: u32,
    pub latency_ms: f64,
}

/// Permission status for microphone access
#[derive(Debug, Clone)]
pub enum PermissionStatus {
    Granted,
    Denied,
    Prompt,
    Unknown,
}

/// Audio error types
#[derive(Debug, Clone)]
pub enum AudioErrorType {
    Critical,
    Warning,
    Info,
}

// Device Manager Events - STORY-014

/// Device list updated event
#[derive(Debug, Clone)]
pub struct DeviceListUpdatedEvent {
    pub devices: Vec<crate::modules::audio_foundations::device_manager::AudioDevice>,
    pub timestamp_ns: u64,
}

impl Event for DeviceListUpdatedEvent {
    fn event_type(&self) -> &'static str {
        "DeviceListUpdatedEvent"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp_ns
    }
    
    fn priority(&self) -> EventPriority {
        EventPriority::Normal
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Microphone permission event
#[derive(Debug, Clone)]
pub struct MicrophonePermissionEvent {
    pub event_type: PermissionEventType,
    pub permission_status: PermissionStatus,
    pub user_action_required: bool,
    pub recovery_instructions: Option<String>,
    pub timestamp_ns: u64,
}

impl Event for MicrophonePermissionEvent {
    fn event_type(&self) -> &'static str {
        "MicrophonePermissionEvent"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp_ns
    }
    
    fn priority(&self) -> EventPriority {
        match self.event_type {
            PermissionEventType::Denied => EventPriority::High,
            PermissionEventType::Granted => EventPriority::Normal,
            _ => EventPriority::Normal,
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Permission event types
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionEventType {
    RequestStarted,
    Granted,
    Denied,
    Revoked,
    Changed,
}

/// Device monitoring event
#[derive(Debug, Clone)]
pub struct DeviceMonitoringEvent {
    pub event_type: DeviceMonitoringEventType,
    pub message: String,
    pub timestamp_ns: u64,
}

impl Event for DeviceMonitoringEvent {
    fn event_type(&self) -> &'static str {
        "DeviceMonitoringEvent"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp_ns
    }
    
    fn priority(&self) -> EventPriority {
        match self.event_type {
            DeviceMonitoringEventType::DeviceError => EventPriority::High,
            DeviceMonitoringEventType::RecoveryActionRequired => EventPriority::High,
            _ => EventPriority::Normal,
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Device monitoring event types
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceMonitoringEventType {
    MonitoringStarted,
    MonitoringStopped,
    DeviceConnected,
    DeviceDisconnected,
    DeviceError,
    DeviceListChanged,
    RecoveryActionRequired,
}

/// Recovery event for graceful handling
#[derive(Debug, Clone)]
pub struct RecoveryEvent {
    pub event_type: RecoveryEventType,
    pub device_id: Option<String>,
    pub recovery_action: Option<String>,
    pub success: bool,
    pub message: String,
    pub timestamp_ns: u64,
}

impl Event for RecoveryEvent {
    fn event_type(&self) -> &'static str {
        "RecoveryEvent"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp_ns
    }
    
    fn priority(&self) -> EventPriority {
        match self.event_type {
            RecoveryEventType::RecoveryFailed => EventPriority::High,
            RecoveryEventType::RecordingPaused => EventPriority::High,
            _ => EventPriority::Normal,
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Recovery event types
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryEventType {
    RecordingStarted,
    RecordingStopped,
    RecordingPaused,
    RecordingResumed,
    RecoveryStarted,
    RecoverySucceeded,
    RecoveryFailed,
}

// STORY-015: Enhanced Audio Events for Multi-Algorithm Pitch Detection

/// Signal analysis event for comprehensive audio signal information
#[derive(Debug, Clone)]
pub struct SignalAnalysisEvent {
    pub snr_estimate: f32,
    pub signal_complexity: f32,
    pub buffer_size: usize,
    pub rms_energy: f32,
    pub peak_amplitude: f32,
    pub timestamp_ns: u64,
}

impl Event for SignalAnalysisEvent {
    fn event_type(&self) -> &'static str {
        "SignalAnalysisEvent"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp_ns
    }
    
    fn priority(&self) -> EventPriority {
        EventPriority::Medium
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Algorithm performance comparison event
#[derive(Debug, Clone)]
pub struct AlgorithmPerformanceEvent {
    pub yin_processing_time_ns: u64,
    pub mcleod_processing_time_ns: u64,
    pub yin_accuracy_score: f32,
    pub mcleod_accuracy_score: f32,
    pub recommended_algorithm: crate::modules::audio_foundations::multi_algorithm_pitch_detector::PitchAlgorithm,
    pub recommendation_confidence: f32,
    pub timestamp_ns: u64,
}

impl Event for AlgorithmPerformanceEvent {
    fn event_type(&self) -> &'static str {
        "AlgorithmPerformanceEvent"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp_ns
    }
    
    fn priority(&self) -> EventPriority {
        EventPriority::Low
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Algorithm switch event (for runtime switching)
#[derive(Debug, Clone)]
pub struct AlgorithmSwitchEvent {
    pub old_algorithm: crate::modules::audio_foundations::multi_algorithm_pitch_detector::PitchAlgorithm,
    pub new_algorithm: crate::modules::audio_foundations::multi_algorithm_pitch_detector::PitchAlgorithm,
    pub reason: AlgorithmSwitchReason,
    pub timestamp_ns: u64,
}

impl Event for AlgorithmSwitchEvent {
    fn event_type(&self) -> &'static str {
        "AlgorithmSwitchEvent"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp_ns
    }
    
    fn priority(&self) -> EventPriority {
        EventPriority::Normal
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Reasons for algorithm switching
#[derive(Debug, Clone, PartialEq)]
pub enum AlgorithmSwitchReason {
    UserSelection,
    AutomaticOptimization,
    SignalCharacteristicsChange,
    PerformanceOptimization,
    ErrorRecovery,
}

/// Pitch detection configuration change event
#[derive(Debug, Clone)]
pub struct PitchConfigurationEvent {
    pub config_change: PitchConfigurationChange,
    pub old_value: String,
    pub new_value: String,
    pub timestamp_ns: u64,
}

impl Event for PitchConfigurationEvent {
    fn event_type(&self) -> &'static str {
        "PitchConfigurationEvent"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp_ns
    }
    
    fn priority(&self) -> EventPriority {
        EventPriority::Normal
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Types of pitch detection configuration changes
#[derive(Debug, Clone, PartialEq)]
pub enum PitchConfigurationChange {
    AlgorithmChange,
    FrequencyRangeChange,
    ThresholdChange,
    ConfidenceScoringToggle,
    HarmonicAnalysisToggle,
    SampleRateChange,
}

/// Device state event (updated with more comprehensive information)
#[derive(Debug, Clone)]
pub struct MicrophoneStateEvent {
    pub state: DeviceState,
    pub device_info: Option<crate::modules::audio_foundations::device_manager::AudioDevice>,
    pub permissions: PermissionStatus,
    pub timestamp_ns: u64,
}

impl Event for MicrophoneStateEvent {
    fn event_type(&self) -> &'static str {
        "MicrophoneStateEvent"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp_ns
    }
    
    fn priority(&self) -> EventPriority {
        match self.state {
            DeviceState::Error(_) => EventPriority::High,
            DeviceState::Disconnected => EventPriority::High,
            DeviceState::Connected => EventPriority::Normal,
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// STORY-017: Performance Monitoring Events

/// Audio performance event for real-time metrics
#[derive(Debug, Clone)]
pub struct AudioPerformanceEvent {
    pub metric_type: String,
    pub value: f32,
    pub unit: String,
    pub timestamp: u64,
    pub operation_context: Option<String>,
}

impl Event for AudioPerformanceEvent {
    fn event_type(&self) -> &'static str {
        "AudioPerformanceEvent"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
    
    fn priority(&self) -> EventPriority {
        EventPriority::Medium
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Performance alert event for threshold violations
#[derive(Debug, Clone)]
pub struct PerformanceAlertEvent {
    pub alert_type: String,
    pub severity: String,
    pub threshold_value: f32,
    pub actual_value: f32,
    pub message: String,
    pub timestamp: u64,
    pub requires_attention: bool,
}

impl Event for PerformanceAlertEvent {
    fn event_type(&self) -> &'static str {
        "PerformanceAlertEvent"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
    
    fn priority(&self) -> EventPriority {
        if self.requires_attention {
            EventPriority::High
        } else {
            EventPriority::Normal
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Performance regression detection event
#[derive(Debug, Clone)]
pub struct PerformanceRegressionEvent {
    pub metric_name: String,
    pub baseline_value: f32,
    pub current_value: f32,
    pub regression_percent: f32,
    pub confidence_level: f32,
    pub impact_assessment: String,
    pub timestamp: u64,
}

impl Event for PerformanceRegressionEvent {
    fn event_type(&self) -> &'static str {
        "PerformanceRegressionEvent"
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
    
    fn priority(&self) -> EventPriority {
        EventPriority::High
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}