# Presenter Layer Architecture Proposal

## Overview

This document outlines the proposed architecture for adding a presenter layer to the audio application, providing clean separation between audio processing and presentation logic while maintaining real-time performance.

**Technology Stack:**
- **Language**: Rust (compiled to WebAssembly)
- **Target**: Modern browsers only
- **Graphics**: Immersive WebGL-rendered UI using three-d crate
- **Interaction**: User gestures handled through raycasting into 3D world
- **No HTML UI**: All interface elements are GPU-rendered 3D objects

## Architectural Assessment

The proposed API design aligns well with the existing event-driven architecture and follows established patterns in the codebase for performance and type safety.

## Core Architecture

### 1. Event-Driven Consistency

Following the existing domain-specific event dispatcher pattern with separate dispatchers for different priority levels:

```rust
// High-priority real-time events (processed immediately)
type RealTimeEventDispatcher = SharedEventDispatcher<RealTimeEvent>;

pub enum RealTimeEvent {
    DataUpdated(RealTimeData),
    PitchDetected(PitchData),
    VolumeChanged(VolumeData),
    FFTUpdated(FFTData),
}

// Lower-priority configuration events (can be batched/deferred)
type ConfigEventDispatcher = SharedEventDispatcher<ConfigEvent>;

pub enum ConfigEvent {
    ConfigUpdated(PresentationConfig),
    TuningSystemChanged(TuningSystem),
    RootNoteChanged(MusicalNote),
}
```

### 2. Configuration Structure

Building on existing configuration patterns:

```rust
pub struct PresentationConfig {
    // Core tuning configuration
    pub tuning_system: TuningSystem,
    pub reference_note: MusicalNote,
    pub reference_pitch: f32,
    pub root_note: MusicalNote,

    // Real-time behavior configuration
    pub update_frequency_hz: f32,  // 60fps target
    pub data_smoothing: bool,      // Apply temporal smoothing to reduce jitter in pitch/volume data
    pub confidence_threshold: f32,
}

impl PresentationConfig {
    pub fn for_development() -> Self { /* relaxed settings */ }
    pub fn for_production() -> Self { /* optimized settings */ }
}
```

### 3. Real-Time Data Structure

Enhanced data structure with confidence weighting and performance metrics:

```rust
pub struct RealTimeData {
    // Core audio data
    pub volume: VolumeData,
    pub detected_pitch: Option<PitchData>,
    pub fft: FFTData,

    // Metadata for presentation decisions
    pub timestamp: f64,
    pub processing_latency_ms: f32,
    pub confidence_score: f32,

    // Musical context
    pub current_note: Option<MusicalNote>,
    pub interval_from_root: Option<IntervalData>,
    pub tuning_deviation_cents: f32,
}

pub struct VolumeData {
    pub rms_db: f32,
    pub level: VolumeLevel,
    pub confidence: f32,
}

pub struct PitchData {
    pub frequency: f32,
    pub confidence: f32,
    pub note: MusicalNote,
    pub stability: PitchStability,
}
```

### 4. Service Interface Pattern

Following the existing ConsoleAudioService pattern with separate dispatchers:

```rust
pub trait PresentationService {
    // Configuration management
    fn update_config(&self, config: PresentationConfig) -> Result<(), ConfigError>;
    fn get_current_config(&self) -> PresentationConfig;

    // Event subscription (automatic cleanup)
    // High-priority real-time events
    fn subscribe_to_real_time_events(&self, callback: RealTimeEventCallback) -> SubscriptionHandle;
    
    // Lower-priority configuration events
    fn subscribe_to_config_events(&self, callback: ConfigEventCallback) -> SubscriptionHandle;

    // Musical context control
    fn set_tuning_system(&self, system: TuningSystem) -> Result<(), TuningError>;
    fn select_root_note(&self, note: MusicalNote, frequency: f32) -> Result<(), NoteError>;
    
    // Direct audio system control (must be called synchronously from UI gesture)
    fn request_microphone_permission(&self) -> Result<(), PermissionError>;

    // Performance monitoring
    fn get_performance_metrics(&self) -> PresentationMetrics;
}

// Service implementation with dual dispatchers
pub struct PresentationServiceImpl {
    real_time_dispatcher: RealTimeEventDispatcher,
    config_dispatcher: ConfigEventDispatcher,
    config: Arc<RwLock<PresentationConfig>>,
    // ... other fields
}
```

## Architectural Enhancements

### 1. Zero-Allocation Real-Time Processing

Following the BufferPool pattern:

```rust
pub struct PresentationDataPool {
    // Pre-allocated buffers for real-time data
    real_time_buffers: Vec<RealTimeData>,
    fft_buffers: Vec<Vec<f32>>,

    // Memory budget enforcement
    max_memory_mb: usize,
    current_allocation: usize,
}
```

### 2. Batch Processing for Performance

Leveraging existing event batching patterns:

```rust
pub struct PresentationBatch {
    pub timestamp: f64,
    pub data_updates: Vec<RealTimeData>,
    pub config_changes: Vec<ConfigChange>,
    pub performance_metrics: PresentationMetrics,
}
```

### 3. Error Handling Strategy

Following the fail-fast validation pattern:

```rust
pub enum PresentationError {
    InvalidTuningSystem(String),
    UnsupportedReferenceNote(String),
    FrequencyOutOfRange { frequency: f32, min: f32, max: f32 },
    ConfigurationConflict(String),
    PerformanceThresholdExceeded(String),
}
```

## Integration with Existing Architecture

### Event Flow Integration

```rust
// Audio events flow into presentation events
AudioEvent::PitchDetected { frequency, confidence, note }
    ↓
PresentationEvent::RealTimeDataUpdated(RealTimeData {
    detected_pitch: Some(PitchData { frequency, confidence, note, ... }),
    interval_from_root: Some(calculate_interval(note, root_note)),
    ...
})
```

### Configuration Validation

Leveraging existing validation patterns:

```rust
impl PresentationConfig {
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if self.reference_pitch < 220.0 || self.reference_pitch > 880.0 {
            errors.push(ValidationError::InvalidReferencePitch);
        }

        if self.update_frequency_hz > 60.0 {
            errors.push(ValidationError::UpdateFrequencyTooHigh);
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

## Musical Context Extensions

### 1. Interval Listening Mode

```rust
pub enum IntervalType {
    Root,
    MinorSecond,
    MajorSecond,
    MinorThird,
    MajorThird,
    PerfectFourth,
    AugmentedFourth,
    PerfectFifth,
    MinorSixth,
    MajorSixth,
    MinorSeventh,
    MajorSeventh,
}

pub struct IntervalData {
    pub interval_type: IntervalType,
    pub octave: i32, // 0, 1, -1, etc.
    pub cents_deviation: f32,
}
```

### 2. Advanced Tuning Systems

```rust
pub enum TuningSystem {
    EqualTemperament,
    JustIntonation,
}
```

## Implementation Recommendations

### Phased Implementation

1. **Phase 1**: Core config + real-time data structures
2. **Phase 2**: Event dispatcher integration
3. **Phase 3**: Service interface implementation
4. **Phase 4**: Performance optimization + memory pooling

### Testing Strategy

Following existing patterns:
- Mock implementations for unit testing
- Test signal integration for deterministic testing
- Performance benchmarking with configurable thresholds

### Performance Targets

- Data update frequency: 60fps (16ms intervals)
- Event processing: <1ms for real-time updates (best effort, not critical)
- Memory allocation: Zero allocation during real-time operation
- Configuration changes: <5ms response time

## Success Metrics

This API design would be architecturally successful if it:

1. Maintains real-time performance (≤16ms graphics latency, ≤11ms audio latency)
2. Follows zero-allocation principles during operation
3. Integrates seamlessly with existing event patterns
4. Enables clean separation between presentation and audio logic
5. Supports future extensibility without breaking changes
6. Achieves stable 60fps rendering with immersive 3D UI

## Real-Time Audio and Graphics Coexistence

### Threading Architecture

The application requires careful coordination between two demanding real-time systems:

```rust
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Audio Thread  │    │ Graphics Thread │    │   Main Thread   │
│   (High Prio)   │    │   (Med Prio)    │    │   (Low Prio)    │
│                 │    │                 │    │                 │
│ • DSP Processing│    │ • 60fps Render  │    │ • UI Logic      │
│ • Pitch Detect  │    │ • GPU Commands  │    │ • Event Handling│
│ • FFT Analysis  │    │ • Resource Mgmt │    │ • Config Updates│
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                        │                        │
         └────────── Lock-Free Data Flow ─────────────────┘
```

### Lock-Free Communication Strategy

**Audio → Graphics Data Flow:**
```rust
// Single-producer, single-consumer ring buffer
pub struct AudioDataRing<T> {
    buffer: Box<[T]>,
    write_index: AtomicUsize,
    read_index: AtomicUsize,
    capacity: usize,
}

impl<T: Copy> AudioDataRing<T> {
    pub fn push(&self, item: T) -> Result<(), T> {
        // Audio thread writes without blocking
    }
    
    pub fn pop(&self) -> Option<T> {
        // Graphics thread reads without blocking
    }
}
```

**Graphics → Audio Configuration:**
```rust
// Atomic configuration updates
pub struct AtomicAudioConfig {
    tuning_system: AtomicU8,
    root_note: AtomicU8,
    reference_pitch: AtomicU32, // IEEE 754 bits
}

impl AtomicAudioConfig {
    pub fn update_tuning_system(&self, system: TuningSystem) {
        self.tuning_system.store(system as u8, Ordering::Release);
    }
    
    pub fn load_config(&self) -> AudioConfig {
        // Audio thread reads atomically
    }
}
```

### Performance Budgets

**Audio Thread (every ~11ms at 44.1kHz/512 samples):**
- DSP Processing: 6ms
- Pitch Detection: 3ms
- Data Publishing: 1ms
- Buffer: 1ms

**Graphics Thread (every 16.67ms at 60fps):**
- Data Consumption: 1ms
- 3D Scene Update: 5ms
- WebGL Rendering: 8ms
- Present: 1ms
- Buffer: 1.67ms

### Critical Design Principles

1. **Audio Thread Priority**: Highest system priority to prevent audio dropouts
2. **Graphics Graceful Degradation**: Can drop frames if needed, audio cannot
3. **Zero Blocking**: Neither thread should ever wait for the other
4. **Buffering Strategy**: Multiple frames of audio data buffered for graphics consumption
5. **Memory Pre-allocation**: All buffers allocated at startup

### Resource Management

**CPU Affinity:**
```rust
// Pin audio thread to dedicated core
#[cfg(target_os = "linux")]
fn set_audio_thread_affinity() {
    let mut cpu_set = libc::cpu_set_t::default();
    unsafe {
        libc::CPU_ZERO(&mut cpu_set);
        libc::CPU_SET(0, &mut cpu_set); // Pin to core 0
        libc::sched_setaffinity(0, size_of::<libc::cpu_set_t>(), &cpu_set);
    }
}
```

**Memory Pools:**
```rust
pub struct RealTimeMemoryPool {
    // Pre-allocated buffers
    real_time_data_pool: Vec<RealTimeData>,
    fft_buffer_pool: Vec<Vec<f32>>,
    
    // Pool management
    available_indices: AtomicU32, // Bitset for available buffers
}
```

### Data Synchronization Pattern

```rust
pub struct PresentationServiceImpl {
    // Lock-free audio data ring buffer
    audio_data_ring: AudioDataRing<RealTimeData>,
    
    // Atomic configuration
    audio_config: AtomicAudioConfig,
    
    // Graphics thread state
    current_visual_state: RwLock<VisualState>,
}

impl PresentationServiceImpl {
    pub fn publish_audio_data(&self, data: RealTimeData) {
        // Called from audio thread - never blocks
        let _ = self.audio_data_ring.push(data);
    }
    
    pub fn consume_audio_data(&self) -> Vec<RealTimeData> {
        // Called from graphics thread - never blocks
        let mut data = Vec::new();
        while let Some(item) = self.audio_data_ring.pop() {
            data.push(item);
        }
        data
    }
}
```

## WebGL Graphics Integration

The presenter layer integrates with three-d crate for immersive WebGL rendering:

### 3D UI Architecture
- **No HTML/DOM**: All UI elements are 3D meshes rendered via WebGL
- **Raycast Interaction**: Mouse/touch events converted to 3D world rays
- **GPU-Accelerated**: All visual elements leverage GPU compute and rendering
- **Modern Browser Features**: Uses WebGL 2.0 and WebAssembly for performance

### Rendering Pipeline
- **60fps target**: Real-time data updates at 16ms intervals
- **3D Scene Management**: Musical visualization and UI in shared 3D space
- **Performance Optimization**: Frustum culling, LOD, and batched rendering
- **Responsive Design**: Viewport-aware 3D layout system

### User Interaction Model
```rust
// Example: Converting mouse position to 3D world interaction
fn handle_mouse_click(position: (f64, f64), camera: &Camera) -> Option<UiAction> {
    let ray = camera.ray_from_screen_coordinates(position);
    
    // Test ray against 3D UI elements
    if let Some(hit) = raycast_ui_elements(&ray) {
        return Some(hit.action);
    }
    
    None
}
```

### WebAssembly Considerations
- **Memory Management**: Linear memory model with efficient data structures
- **Performance**: Near-native speed for audio processing and 3D rendering
- **Browser APIs**: Direct access to WebAudio, WebGL, and input events
- **DevConsole**: HTML-based debug overlay (debug builds only)

## Next Steps

The proposed API foundation provides a solid architectural base. Key areas for further discussion and refinement include:

- Specific implementation details for each phase
- Integration points with existing audio processing pipeline
- Performance benchmarking and optimization strategies
- User interface considerations for configuration management