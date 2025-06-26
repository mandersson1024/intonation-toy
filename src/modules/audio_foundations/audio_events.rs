// Audio Events for Event Bus Integration - STORY-013

use std::any::Any;
use std::fmt;
use crate::modules::application_core::event_bus::{Event, EventPriority};

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

/// Pitch detection result event
#[derive(Debug, Clone)]
pub struct PitchDetectionEvent {
    pub frequency: f32,
    pub confidence: f32,
    pub signal_info: SignalInfo,
    pub processing_time_ns: u64,
    pub timestamp_ns: u64,
    pub source_buffer_ref: Option<u32>,
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

/// Microphone device state change event  
#[derive(Debug, Clone)]
pub struct MicrophoneStateEvent {
    pub state: DeviceState,
    pub device_info: Option<AudioDeviceInfo>,
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