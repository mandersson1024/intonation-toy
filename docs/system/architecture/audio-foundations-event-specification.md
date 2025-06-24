# Audio Foundations Module - Event Bus Communication Specification

## Overview

The **Audio Foundations Module** serves as the core audio processing system, managing microphone access, real-time pitch detection, signal generation, and performance monitoring. This document defines all events the module publishes and consumes through the centralized event bus.

## Event Categories

### 1. Critical Priority Events (Real-time Audio Processing)
- **Audio Buffer Processing** - Zero-copy audio data flow
- **Pitch Detection Results** - Real-time frequency analysis
- **Performance Threshold Violations** - Latency/processing alerts

### 2. High Priority Events (Device Management)
- **Microphone Connection State** - Device availability changes
- **Audio Context State** - Web Audio API status
- **Device Configuration Changes** - Sample rate/buffer size updates

### 3. Normal Priority Events (Configuration & Status)
- **Audio Settings Updates** - User preference changes
- **Calibration Results** - Algorithm tuning outcomes
- **Processing State Changes** - Engine lifecycle events

### 4. Low Priority Events (Metrics & Debug)
- **Performance Metrics** - Detailed statistics
- **Debug Information** - Development diagnostics
- **Educational Validation** - Learning feedback

---

## Event Schemas

### Critical Priority Events

#### AudioBufferEvent
```rust
/// Real-time audio buffer processing - zero-copy data flow
#[derive(Debug, Clone)]
pub struct AudioBufferEvent {
    /// Buffer metadata (never the actual audio data)
    pub buffer_info: AudioBufferInfo,
    /// Reference ID for shared buffer access
    pub buffer_ref_id: BufferRefId,
    /// Processing timestamp for latency tracking
    pub timestamp_ns: u64,
    /// Processing context for coordination
    pub context: ProcessingContext,
}

#[derive(Debug, Clone)]
pub struct AudioBufferInfo {
    pub sample_rate: f32,
    pub channel_count: u32,
    pub frame_count: usize,
    pub format: AudioFormat,
    pub latency_hint_ms: f32,
}

#[derive(Debug, Clone)]
pub struct ProcessingContext {
    pub source: AudioSource,
    pub processing_chain_id: ProcessingChainId,
    pub expected_consumers: Vec<ModuleId>,
}

#[derive(Debug, Clone)]
pub enum AudioSource {
    Microphone(DeviceId),
    TestSignal(TestSignalConfig),
    FilePlayback(FileId),
}
```

#### PitchDetectionEvent
```rust
/// Real-time pitch detection results
#[derive(Debug, Clone)]
pub struct PitchDetectionEvent {
    /// Detected frequency in Hz (-1.0 if no pitch detected)
    pub frequency: f32,
    /// Detection confidence (0.0-1.0)
    pub confidence: f32,
    /// Detection algorithm used
    pub algorithm: PitchAlgorithm,
    /// Audio signal characteristics
    pub signal_info: SignalInfo,
    /// Processing performance
    pub processing_time_ns: u64,
    /// Timestamp for synchronization
    pub timestamp_ns: u64,
    /// Source buffer reference
    pub source_buffer_ref: BufferRefId,
}

#[derive(Debug, Clone)]
pub struct SignalInfo {
    pub amplitude: f32,
    pub clarity: f32,
    pub harmonic_content: f32,
    pub noise_floor: f32,
    pub is_stable: bool,
}
```

#### PerformanceViolationEvent
```rust
/// Critical performance threshold violations
#[derive(Debug, Clone)]
pub struct PerformanceViolationEvent {
    pub violation_type: PerformanceViolationType,
    pub threshold: f32,
    pub actual_value: f32,
    pub severity: ViolationSeverity,
    pub module: ModuleId,
    pub timestamp_ns: u64,
    pub recovery_suggestion: Option<RecoveryAction>,
}

#[derive(Debug, Clone)]
pub enum PerformanceViolationType {
    LatencyExceeded,
    DropoutDetected,
    BufferUnderrun,
    ProcessingOverload,
    MemoryExhaustion,
}

#[derive(Debug, Clone)]
pub enum ViolationSeverity {
    Warning,
    Critical,
    SystemFailure,
}
```

### High Priority Events

#### MicrophoneStateEvent
```rust
/// Microphone connection and permission state changes
#[derive(Debug, Clone)]
pub struct MicrophoneStateEvent {
    pub state: MicrophoneState,
    pub device_info: Option<AudioDeviceInfo>,
    pub error: Option<MicrophoneError>,
    pub permissions: PermissionState,
    pub timestamp_ns: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MicrophoneState {
    Disconnected,
    Requesting,
    Connected,
    Suspended,
    Error,
}

#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    pub device_id: DeviceId,
    pub device_name: String,
    pub sample_rate: f32,
    pub buffer_size: usize,
    pub channels: u32,
    pub latency_ms: f64,
    pub supported_formats: Vec<AudioFormat>,
}
```

#### AudioContextEvent
```rust
/// Web Audio API context state changes
#[derive(Debug, Clone)]
pub struct AudioContextEvent {
    pub state: AudioContextState,
    pub sample_rate: f32,
    pub base_latency: f64,
    pub output_latency: f64,
    pub state_change_reason: Option<StateChangeReason>,
    pub timestamp_ns: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioContextState {
    Suspended,
    Running,
    Closed,
    Interrupted,
}

#[derive(Debug, Clone)]
pub enum StateChangeReason {
    UserInteraction,
    SystemSuspend,
    BrowserPolicy,
    ResourceContention,
    Error(String),
}
```

#### DeviceConfigurationEvent
```rust
/// Audio device configuration changes
#[derive(Debug, Clone)]
pub struct DeviceConfigurationEvent {
    pub device_id: DeviceId,
    pub configuration: DeviceConfiguration,
    pub change_type: ConfigurationChangeType,
    pub requires_restart: bool,
    pub timestamp_ns: u64,
}

#[derive(Debug, Clone)]
pub struct DeviceConfiguration {
    pub sample_rate: f32,
    pub buffer_size: usize,
    pub channel_layout: ChannelLayout,
    pub bit_depth: u32,
    pub latency_mode: LatencyMode,
}

#[derive(Debug, Clone)]
pub enum ConfigurationChangeType {
    UserRequested,
    SystemOptimization,
    DeviceSwitch,
    PerformanceAdaptation,
}
```

### Normal Priority Events

#### AudioSettingsEvent
```rust
/// User audio preference changes
#[derive(Debug, Clone)]
pub struct AudioSettingsEvent {
    pub setting_category: AudioSettingCategory,
    pub setting_name: String,
    pub old_value: SettingValue,
    pub new_value: SettingValue,
    pub requires_recalibration: bool,
    pub timestamp_ns: u64,
}

#[derive(Debug, Clone)]
pub enum AudioSettingCategory {
    PitchDetection,
    SignalProcessing,
    Performance,
    Calibration,
    UserInterface,
}

#[derive(Debug, Clone)]
pub enum SettingValue {
    Float(f32),
    Integer(i32),
    Boolean(bool),
    String(String),
    Enum(String),
}
```

#### CalibrationEvent
```rust
/// Algorithm calibration and tuning results
#[derive(Debug, Clone)]
pub struct CalibrationEvent {
    pub calibration_type: CalibrationType,
    pub target_frequency: f32,
    pub measured_accuracy: f32,
    pub recommendations: Vec<CalibrationRecommendation>,
    pub timestamp_ns: u64,
}

#[derive(Debug, Clone)]
pub enum CalibrationType {
    PitchDetectionAccuracy,
    LatencyOptimization,
    NoiseFloorBaseline,
    FrequencyResponse,
}

#[derive(Debug, Clone)]
pub struct CalibrationRecommendation {
    pub parameter: String,
    pub suggested_value: SettingValue,
    pub expected_improvement: f32,
    pub confidence: f32,
}
```

#### ProcessingStateEvent
```rust
/// Audio engine lifecycle state changes
#[derive(Debug, Clone)]
pub struct ProcessingStateEvent {
    pub state: ProcessingState,
    pub previous_state: ProcessingState,
    pub transition_reason: StateTransitionReason,
    pub processing_chain_id: ProcessingChainId,
    pub timestamp_ns: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingState {
    Uninitialized,
    Initializing,
    Ready,
    Processing,
    Suspended,
    Stopping,
    Error,
}

#[derive(Debug, Clone)]
pub enum StateTransitionReason {
    UserCommand,
    SystemEvent,
    ErrorRecovery,
    ResourceOptimization,
    DeviceChange,
}
```

### Low Priority Events

#### PerformanceMetricsEvent
```rust
/// Detailed performance statistics
#[derive(Debug, Clone)]
pub struct PerformanceMetricsEvent {
    pub metrics: PerformanceMetrics,
    pub measurement_period_ms: u64,
    pub timestamp_ns: u64,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Processing performance
    pub avg_processing_time_ms: f32,
    pub max_processing_time_ms: f32,
    pub processing_cpu_usage: f32,
    
    /// Latency measurements
    pub end_to_end_latency_ms: f32,
    pub audio_context_latency_ms: f32,
    pub processing_latency_ms: f32,
    
    /// Quality metrics
    pub pitch_detection_accuracy: f32,
    pub signal_quality_score: f32,
    pub dropout_count: u32,
    
    /// Resource usage
    pub memory_usage_bytes: usize,
    pub buffer_utilization: f32,
    pub thread_count: u32,
}
```

#### AudioDebugEvent
```rust
#[cfg(feature = "debug")]
/// Development and debugging information
#[derive(Debug, Clone)]
pub struct AudioDebugEvent {
    pub debug_type: AudioDebugType,
    pub data: DebugData,
    pub source_module: ModuleId,
    pub timestamp_ns: u64,
}

#[cfg(feature = "debug")]
#[derive(Debug, Clone)]
pub enum AudioDebugType {
    BufferAnalysis,
    AlgorithmState,
    PerformanceTrace,
    CalibrationStep,
    ErrorDiagnostic,
}

#[cfg(feature = "debug")]
#[derive(Debug, Clone)]
pub enum DebugData {
    BufferSnapshot(BufferSnapshot),
    AlgorithmParameters(HashMap<String, SettingValue>),
    TimingTrace(Vec<TimingEvent>),
    FrequencySpectrum(Vec<f32>),
    ErrorContext(ErrorDiagnosticInfo),
}
```

#### EducationalEvent
```rust
/// Educational feedback and musical learning data
#[derive(Debug, Clone)]
pub struct EducationalEvent {
    pub event_type: EducationalEventType,
    pub musical_context: MusicalContext,
    pub feedback: EducationalFeedback,
    pub timestamp_ns: u64,
}

#[derive(Debug, Clone)]
pub enum EducationalEventType {
    PitchAccuracyFeedback,
    IntervalRecognition,
    TuningAssistance,
    ProgressTracking,
}

#[derive(Debug, Clone)]
pub struct MusicalContext {
    pub target_note: Option<MusicalNote>,
    pub detected_note: Option<MusicalNote>,
    pub interval: Option<MusicalInterval>,
    pub scale_context: Option<MusicalScale>,
}

#[derive(Debug, Clone)]
pub struct EducationalFeedback {
    pub accuracy_cents: f32,
    pub improvement_suggestion: String,
    pub confidence_level: f32,
    pub progress_score: f32,
}
```

---

## Event Bus Integration

### Module Event Registration
```rust
impl Module for AudioFoundationsModule {
    fn register_events(&self, event_bus: &mut EventBus) -> Result<(), ModuleError> {
        // Register as publisher for all audio events
        event_bus.register_publisher::<AudioBufferEvent>(self.module_id())?;
        event_bus.register_publisher::<PitchDetectionEvent>(self.module_id())?;
        event_bus.register_publisher::<PerformanceViolationEvent>(self.module_id())?;
        event_bus.register_publisher::<MicrophoneStateEvent>(self.module_id())?;
        event_bus.register_publisher::<AudioContextEvent>(self.module_id())?;
        event_bus.register_publisher::<ProcessingStateEvent>(self.module_id())?;
        event_bus.register_publisher::<PerformanceMetricsEvent>(self.module_id())?;
        
        #[cfg(feature = "debug")]
        event_bus.register_publisher::<AudioDebugEvent>(self.module_id())?;
        
        // Register as consumer for configuration events
        event_bus.register_consumer::<AudioSettingsEvent>(
            self.module_id(),
            EventPriority::Normal,
            Box::new(|event| self.handle_audio_settings_change(event))
        )?;
        
        event_bus.register_consumer::<DeviceConfigurationEvent>(
            self.module_id(),
            EventPriority::High,
            Box::new(|event| self.handle_device_configuration_change(event))
        )?;
        
        Ok(())
    }
}
```

### Zero-Copy Audio Buffer Management
```rust
/// Audio buffer producer interface for zero-copy data flow
pub trait AudioBufferProducer {
    /// Get exclusive write access to audio buffer
    fn get_write_buffer(&mut self, frames: usize) -> Result<&mut [f32], AudioError>;
    
    /// Commit written buffer and publish event
    fn commit_buffer(&mut self, buffer_ref_id: BufferRefId, info: AudioBufferInfo) -> Result<(), AudioError>;
    
    /// Get buffer reference for sharing (read-only)
    fn get_buffer_ref(&self, buffer_ref_id: BufferRefId) -> Option<&[f32]>;
}

/// Audio buffer consumer interface for zero-copy data flow
pub trait AudioBufferConsumer {
    /// Process audio buffer by reference
    fn process_buffer(&mut self, buffer_ref: &[f32], info: &AudioBufferInfo) -> Result<(), AudioError>;
    
    /// Signal processing completion
    fn processing_complete(&mut self, buffer_ref_id: BufferRefId);
}
```

### Event Publishing Examples
```rust
impl AudioFoundationsModule {
    /// Publish real-time pitch detection result
    pub fn publish_pitch_detection(&self, frequency: f32, confidence: f32, signal_info: SignalInfo) {
        let event = PitchDetectionEvent {
            frequency,
            confidence,
            algorithm: self.current_algorithm,
            signal_info,
            processing_time_ns: self.last_processing_time,
            timestamp_ns: get_high_precision_timestamp(),
            source_buffer_ref: self.current_buffer_ref,
        };
        
        self.event_bus.publish_critical(event);
    }
    
    /// Publish microphone state change
    pub fn publish_microphone_state(&self, state: MicrophoneState, device_info: Option<AudioDeviceInfo>) {
        let event = MicrophoneStateEvent {
            state,
            device_info,
            error: self.last_microphone_error.clone(),
            permissions: self.current_permissions,
            timestamp_ns: get_high_precision_timestamp(),
        };
        
        self.event_bus.publish_high(event);
    }
    
    /// Publish performance metrics periodically
    pub fn publish_performance_metrics(&self) {
        let event = PerformanceMetricsEvent {
            metrics: self.performance_monitor.get_current_metrics(),
            measurement_period_ms: 1000, // 1 second reporting interval
            timestamp_ns: get_high_precision_timestamp(),
        };
        
        self.event_bus.publish_low(event);
    }
}
```

---

## Performance Considerations

### Event Batching
- **Critical events**: Individual dispatch for minimal latency
- **Normal/Low events**: Batched every 16ms (60 FPS aligned)
- **Debug events**: Batched every 100ms in debug builds

### Memory Management
- **Zero-copy audio data**: Only metadata in events, actual audio buffers shared by reference
- **Event pooling**: Reuse event objects to minimize allocations
- **Automatic cleanup**: Events garbage collected after all consumers process them

### Latency Guarantees
- **Critical events**: < 1ms dispatch latency
- **High priority events**: < 5ms dispatch latency
- **Normal events**: < 16ms dispatch latency
- **Low priority events**: < 100ms dispatch latency

---

## Integration with Current Codebase

### Migration Strategy
1. **Phase 1**: Wrap existing `AudioEngineService` callbacks with event publishers
2. **Phase 2**: Migrate `AudioData` and `PerformanceMetrics` to event system
3. **Phase 3**: Replace direct function calls with event-driven communication
4. **Phase 4**: Implement zero-copy buffer sharing for audio data

### Backward Compatibility
- Keep existing callback interfaces during transition
- Gradually migrate components to consume events instead of callbacks
- Maintain performance characteristics throughout migration

---

**Next Steps**: 
1. Implement the core event bus infrastructure
2. Create the AudioFoundationsModule trait implementation
3. Begin Phase 1 migration of existing audio service callbacks 