// Graceful Recovery Implementation - STORY-014
// Handles graceful recovery from device changes during audio recording

use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::MediaStream;
use super::device_manager::{AudioDevice, DeviceError};
use super::device_monitor::{DeviceRecoveryAction};
use super::audio_events::*;
use crate::modules::application_core::event_bus::EventBus;
use crate::modules::application_core::typed_event_bus::TypedEventBus;

/// Trait for graceful recovery management
pub trait GracefulRecoveryManager: Send + Sync {
    /// Handle device change during active recording
    fn handle_device_change_during_recording(&mut self, device_id: &str, change_type: DeviceChangeType) -> Result<RecoveryResult, RecoveryError>;
    
    /// Switch to fallback device
    fn switch_to_fallback_device(&mut self) -> Result<RecoveryResult, RecoveryError>;
    
    /// Pause recording gracefully
    fn pause_recording_gracefully(&mut self, reason: String) -> Result<(), RecoveryError>;
    
    /// Resume recording after recovery
    fn resume_recording_after_recovery(&mut self) -> Result<(), RecoveryError>;
    
    /// Get current recovery state
    fn get_recovery_state(&self) -> RecoveryState;
    
    /// Set fallback devices in priority order
    fn set_fallback_devices(&mut self, devices: Vec<String>) -> Result<(), RecoveryError>;
    
    /// Test fallback devices availability
    fn test_fallback_devices(&self) -> Result<FallbackTestResult, RecoveryError>;
}

/// Types of device changes that can occur
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceChangeType {
    Disconnected,
    Error(String),
    PermissionRevoked,
    InUseByOtherApp,
    QualityDegraded,
    LatencyIncreased,
}

/// Recovery result information
#[derive(Debug, Clone, PartialEq)]
pub struct RecoveryResult {
    pub success: bool,
    pub action_taken: RecoveryAction,
    pub new_device_id: Option<String>,
    pub downtime_ms: u32,
    pub quality_impact: QualityImpact,
    pub user_notification: Option<String>,
}

/// Recovery actions that can be taken
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryAction {
    NoActionNeeded,
    SwitchedToFallback(String),
    PausedRecording,
    StoppedRecording,
    ContinuedWithDegradedQuality,
    RequestedUserIntervention,
}

/// Quality impact of recovery action
#[derive(Debug, Clone, PartialEq)]
pub enum QualityImpact {
    None,
    Minor,
    Moderate,
    Severe,
    Unknown,
}

/// Current recovery state
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryState {
    Normal,
    RecoveryInProgress,
    UsingFallback(String),
    RecordingPaused(String),
    RecoveryFailed(String),
}

/// Fallback device test results
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTestResult {
    pub available_fallbacks: Vec<String>,
    pub unavailable_fallbacks: Vec<(String, String)>, // device_id, reason
    pub recommended_primary: Option<String>,
}

/// Recovery management errors
#[derive(Debug, Clone)]
pub enum RecoveryError {
    NoFallbackDevicesAvailable,
    FallbackDeviceInUse(String),
    RecoveryInProgress,
    InvalidState(String),
    PermissionError(String),
    InternalError(String),
}

impl fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecoveryError::NoFallbackDevicesAvailable => {
                write!(f, "No fallback devices available for recovery")
            }
            RecoveryError::FallbackDeviceInUse(id) => {
                write!(f, "Fallback device {} is in use", id)
            }
            RecoveryError::RecoveryInProgress => {
                write!(f, "Recovery operation already in progress")
            }
            RecoveryError::InvalidState(msg) => {
                write!(f, "Invalid recovery state: {}", msg)
            }
            RecoveryError::PermissionError(msg) => {
                write!(f, "Permission error during recovery: {}", msg)
            }
            RecoveryError::InternalError(msg) => {
                write!(f, "Internal recovery error: {}", msg)
            }
        }
    }
}

impl Error for RecoveryError {}

/// Web-based graceful recovery manager
pub struct WebGracefulRecoveryManager {
    current_device_id: Option<String>,
    fallback_devices: Vec<String>,
    recovery_state: RecoveryState,
    current_stream: Option<MediaStream>,
    recording_active: bool,
    event_bus: Option<Arc<TypedEventBus>>,
    recovery_settings: RecoverySettings,
}

/// Recovery configuration settings
#[derive(Debug, Clone)]
pub struct RecoverySettings {
    pub auto_switch_to_fallback: bool,
    pub pause_on_device_error: bool,
    pub max_recovery_attempts: u32,
    pub recovery_timeout_ms: u32,
    pub quality_threshold: f32,
    pub latency_threshold_ms: f32,
}

impl Default for RecoverySettings {
    fn default() -> Self {
        Self {
            auto_switch_to_fallback: true,
            pause_on_device_error: false,
            max_recovery_attempts: 3,
            recovery_timeout_ms: 5000,
            quality_threshold: 0.7,
            latency_threshold_ms: 100.0,
        }
    }
}

impl WebGracefulRecoveryManager {
    /// Create a new graceful recovery manager
    pub fn new() -> Self {
        Self {
            current_device_id: None,
            fallback_devices: Vec::new(),
            recovery_state: RecoveryState::Normal,
            current_stream: None,
            recording_active: false,
            event_bus: None,
            recovery_settings: RecoverySettings::default(),
        }
    }
    
    /// Set event bus for publishing recovery events
    pub fn set_event_bus(&mut self, event_bus: Arc<TypedEventBus>) {
        self.event_bus = Some(event_bus);
    }
    
    /// Set recovery configuration
    pub fn set_recovery_settings(&mut self, settings: RecoverySettings) {
        self.recovery_settings = settings;
    }
    
    /// Set current device and stream
    pub fn set_current_device(&mut self, device_id: String, stream: MediaStream) {
        self.current_device_id = Some(device_id);
        self.current_stream = Some(stream);
    }
    
    /// Start recording session
    pub fn start_recording(&mut self) -> Result<(), RecoveryError> {
        if self.current_device_id.is_none() {
            return Err(RecoveryError::InvalidState("No device selected".to_string()));
        }
        
        self.recording_active = true;
        self.recovery_state = RecoveryState::Normal;
        
        // Publish recording started event
        self.publish_recovery_event(
            RecoveryEventType::RecordingStarted,
            "Recording session started".to_string()
        );
        
        Ok(())
    }
    
    /// Stop recording session
    pub fn stop_recording(&mut self) -> Result<(), RecoveryError> {
        self.recording_active = false;
        
        // Clean up current stream
        if let Some(ref stream) = self.current_stream {
            let tracks = stream.get_audio_tracks();
            for i in 0..tracks.length() {
                let track = tracks.get(i);
                let track: web_sys::MediaStreamTrack = track.into();
                track.stop();
            }
        }
        
        self.current_stream = None;
        self.recovery_state = RecoveryState::Normal;
        
        // Publish recording stopped event
        self.publish_recovery_event(
            RecoveryEventType::RecordingStopped,
            "Recording session stopped".to_string()
        );
        
        Ok(())
    }
    
    /// Determine recovery strategy based on device change type
    fn determine_recovery_strategy(&self, change_type: &DeviceChangeType) -> RecoveryAction {
        match change_type {
            DeviceChangeType::Disconnected => {
                if self.recovery_settings.auto_switch_to_fallback && !self.fallback_devices.is_empty() {
                    RecoveryAction::SwitchedToFallback(self.fallback_devices[0].clone())
                } else if self.recovery_settings.pause_on_device_error {
                    RecoveryAction::PausedRecording
                } else {
                    RecoveryAction::StoppedRecording
                }
            }
            DeviceChangeType::Error(_) => {
                if self.recovery_settings.pause_on_device_error {
                    RecoveryAction::PausedRecording
                } else {
                    RecoveryAction::RequestedUserIntervention
                }
            }
            DeviceChangeType::PermissionRevoked => {
                RecoveryAction::RequestedUserIntervention
            }
            DeviceChangeType::InUseByOtherApp => {
                if !self.fallback_devices.is_empty() {
                    RecoveryAction::SwitchedToFallback(self.fallback_devices[0].clone())
                } else {
                    RecoveryAction::PausedRecording
                }
            }
            DeviceChangeType::QualityDegraded => {
                RecoveryAction::ContinuedWithDegradedQuality
            }
            DeviceChangeType::LatencyIncreased => {
                RecoveryAction::ContinuedWithDegradedQuality
            }
        }
    }
    
    /// Calculate quality impact of recovery action
    fn calculate_quality_impact(&self, action: &RecoveryAction) -> QualityImpact {
        match action {
            RecoveryAction::NoActionNeeded => QualityImpact::None,
            RecoveryAction::SwitchedToFallback(_) => QualityImpact::Minor,
            RecoveryAction::PausedRecording => QualityImpact::Moderate,
            RecoveryAction::StoppedRecording => QualityImpact::Severe,
            RecoveryAction::ContinuedWithDegradedQuality => QualityImpact::Moderate,
            RecoveryAction::RequestedUserIntervention => QualityImpact::Severe,
        }
    }
    
    /// Generate user notification message
    fn generate_user_notification(&self, action: &RecoveryAction, device_id: &str) -> Option<String> {
        match action {
            RecoveryAction::SwitchedToFallback(fallback_id) => {
                Some(format!("Audio device '{}' disconnected. Switched to fallback device '{}'.", device_id, fallback_id))
            }
            RecoveryAction::PausedRecording => {
                Some(format!("Audio device '{}' encountered an issue. Recording paused. Please check your device connection.", device_id))
            }
            RecoveryAction::StoppedRecording => {
                Some(format!("Audio device '{}' disconnected. Recording stopped.", device_id))
            }
            RecoveryAction::RequestedUserIntervention => {
                Some(format!("Audio device '{}' requires attention. Please check device settings and permissions.", device_id))
            }
            RecoveryAction::ContinuedWithDegradedQuality => {
                Some(format!("Audio device '{}' quality degraded. Recording continues with reduced quality.", device_id))
            }
            RecoveryAction::NoActionNeeded => None,
        }
    }
    
    /// Publish recovery event
    fn publish_recovery_event(&self, event_type: RecoveryEventType, message: String) {
        if let Some(ref event_bus) = self.event_bus {
            let event = RecoveryEvent {
                event_type,
                device_id: self.current_device_id.clone(),
                recovery_action: None, // Would be filled in by caller
                success: true, // Would be determined by caller
                message,
                timestamp_ns: get_timestamp_ns(),
            };
            
            if let Err(e) = event_bus.publish(event) {
                web_sys::console::warn_1(&format!("Failed to publish recovery event: {:?}", e).into());
            }
        }
    }
}

impl GracefulRecoveryManager for WebGracefulRecoveryManager {
    fn handle_device_change_during_recording(&mut self, device_id: &str, change_type: DeviceChangeType) -> Result<RecoveryResult, RecoveryError> {
        if !self.recording_active {
            return Ok(RecoveryResult {
                success: true,
                action_taken: RecoveryAction::NoActionNeeded,
                new_device_id: None,
                downtime_ms: 0,
                quality_impact: QualityImpact::None,
                user_notification: None,
            });
        }
        
        if matches!(self.recovery_state, RecoveryState::RecoveryInProgress) {
            return Err(RecoveryError::RecoveryInProgress);
        }
        
        let start_time = get_current_time_ms();
        self.recovery_state = RecoveryState::RecoveryInProgress;
        
        // Determine recovery strategy
        let recovery_action = self.determine_recovery_strategy(&change_type);
        
        // Execute recovery action
        let (success, new_device_id) = match &recovery_action {
            RecoveryAction::SwitchedToFallback(fallback_id) => {
                // Attempt to switch to fallback device
                match self.switch_to_fallback_device() {
                    Ok(result) => (result.success, result.new_device_id),
                    Err(_) => (false, None),
                }
            }
            RecoveryAction::PausedRecording => {
                // Pause recording
                match self.pause_recording_gracefully("Device change detected".to_string()) {
                    Ok(_) => (true, None),
                    Err(_) => (false, None),
                }
            }
            RecoveryAction::StoppedRecording => {
                // Stop recording
                match self.stop_recording() {
                    Ok(_) => (true, None),
                    Err(_) => (false, None),
                }
            }
            RecoveryAction::ContinuedWithDegradedQuality => {
                // Continue with current device but note quality degradation
                (true, Some(device_id.to_string()))
            }
            RecoveryAction::RequestedUserIntervention => {
                // Pause and request user action
                match self.pause_recording_gracefully("User intervention required".to_string()) {
                    Ok(_) => (true, None),
                    Err(_) => (false, None),
                }
            }
            RecoveryAction::NoActionNeeded => (true, None),
        };
        
        let end_time = get_current_time_ms();
        let downtime_ms = (end_time - start_time) as u32;
        
        // Update recovery state
        self.recovery_state = if success {
            match &recovery_action {
                RecoveryAction::SwitchedToFallback(fallback_id) => RecoveryState::UsingFallback(fallback_id.clone()),
                RecoveryAction::PausedRecording => RecoveryState::RecordingPaused("Device change".to_string()),
                _ => RecoveryState::Normal,
            }
        } else {
            RecoveryState::RecoveryFailed(format!("Failed to execute {:?}", recovery_action))
        };
        
        let quality_impact = self.calculate_quality_impact(&recovery_action);
        let user_notification = self.generate_user_notification(&recovery_action, device_id);
        
        // Publish recovery event
        self.publish_recovery_event(
            if success { RecoveryEventType::RecoverySucceeded } else { RecoveryEventType::RecoveryFailed },
            format!("Device change recovery: {:?}", recovery_action)
        );
        
        Ok(RecoveryResult {
            success,
            action_taken: recovery_action,
            new_device_id,
            downtime_ms,
            quality_impact,
            user_notification,
        })
    }
    
    fn switch_to_fallback_device(&mut self) -> Result<RecoveryResult, RecoveryError> {
        if self.fallback_devices.is_empty() {
            return Err(RecoveryError::NoFallbackDevicesAvailable);
        }
        
        let fallback_id = self.fallback_devices[0].clone();
        
        // In a real implementation, this would:
        // 1. Stop current stream
        // 2. Request new stream from fallback device
        // 3. Update current_device_id and current_stream
        
        self.current_device_id = Some(fallback_id.clone());
        self.recovery_state = RecoveryState::UsingFallback(fallback_id.clone());
        
        Ok(RecoveryResult {
            success: true,
            action_taken: RecoveryAction::SwitchedToFallback(fallback_id.clone()),
            new_device_id: Some(fallback_id),
            downtime_ms: 200, // Estimated switch time
            quality_impact: QualityImpact::Minor,
            user_notification: Some("Switched to backup microphone".to_string()),
        })
    }
    
    fn pause_recording_gracefully(&mut self, reason: String) -> Result<(), RecoveryError> {
        if !self.recording_active {
            return Ok(());
        }
        
        // Pause recording but keep stream alive
        self.recording_active = false;
        self.recovery_state = RecoveryState::RecordingPaused(reason.clone());
        
        // Publish paused event
        self.publish_recovery_event(
            RecoveryEventType::RecordingPaused,
            format!("Recording paused: {}", reason)
        );
        
        Ok(())
    }
    
    fn resume_recording_after_recovery(&mut self) -> Result<(), RecoveryError> {
        if self.recording_active {
            return Ok(());
        }
        
        if self.current_device_id.is_none() || self.current_stream.is_none() {
            return Err(RecoveryError::InvalidState("No device or stream available".to_string()));
        }
        
        self.recording_active = true;
        self.recovery_state = RecoveryState::Normal;
        
        // Publish resumed event
        self.publish_recovery_event(
            RecoveryEventType::RecordingResumed,
            "Recording resumed after recovery".to_string()
        );
        
        Ok(())
    }
    
    fn get_recovery_state(&self) -> RecoveryState {
        self.recovery_state.clone()
    }
    
    fn set_fallback_devices(&mut self, devices: Vec<String>) -> Result<(), RecoveryError> {
        self.fallback_devices = devices;
        Ok(())
    }
    
    fn test_fallback_devices(&self) -> Result<FallbackTestResult, RecoveryError> {
        // In a real implementation, this would test each fallback device
        let available_fallbacks = self.fallback_devices.clone();
        let unavailable_fallbacks = Vec::new();
        let recommended_primary = available_fallbacks.first().cloned();
        
        Ok(FallbackTestResult {
            available_fallbacks,
            unavailable_fallbacks,
            recommended_primary,
        })
    }
}

// Utility functions
fn get_current_time_ms() -> f32 {
    if let Some(performance) = web_sys::window().and_then(|w| w.performance()) {
        performance.now() as f32
    } else {
        0.0
    }
}

fn get_timestamp_ns() -> u64 {
    if let Some(performance) = web_sys::window().and_then(|w| w.performance()) {
        (performance.now() * 1_000_000.0) as u64
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_recovery_result() {
        let result = RecoveryResult {
            success: true,
            action_taken: RecoveryAction::SwitchedToFallback("fallback-device".to_string()),
            new_device_id: Some("fallback-device".to_string()),
            downtime_ms: 100,
            quality_impact: QualityImpact::Minor,
            user_notification: Some("Switched to backup device".to_string()),
        };
        
        assert!(result.success);
        assert_eq!(result.downtime_ms, 100);
        assert_eq!(result.quality_impact, QualityImpact::Minor);
    }
    
    #[test]
    fn test_recovery_settings() {
        let settings = RecoverySettings::default();
        assert!(settings.auto_switch_to_fallback);
        assert_eq!(settings.max_recovery_attempts, 3);
        assert_eq!(settings.recovery_timeout_ms, 5000);
    }
    
    #[test]
    fn test_device_change_type() {
        let change = DeviceChangeType::Error("Test error".to_string());
        match change {
            DeviceChangeType::Error(msg) => assert_eq!(msg, "Test error"),
            _ => panic!("Expected Error variant"),
        }
    }
}