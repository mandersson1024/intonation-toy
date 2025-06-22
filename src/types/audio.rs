use serde::{Deserialize, Serialize};

/// Audio processing state for the application
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AudioProcessingState {
    /// No audio processing active
    Inactive,
    /// Initializing audio context and components
    Initializing,
    /// Ready to process audio but not currently processing
    Ready,
    /// Actively processing audio with real-time feedback
    Processing,
    /// Processing suspended (e.g., page not visible)
    Suspended,
    /// Error state with description
    Error(String),
}

impl Default for AudioProcessingState {
    fn default() -> Self {
        Self::Inactive
    }
}

/// Audio device information
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    pub device_id: String,
    pub label: String,
    pub kind: AudioDeviceKind,
    pub sample_rate: f32,
    pub channel_count: u32,
    pub is_default: bool,
}

/// Audio device types
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AudioDeviceKind {
    AudioInput,
    AudioOutput,
}

/// Real-time audio data with pitch detection results
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RealtimeAudioData {
    /// Detected pitch frequency in Hz (-1.0 if no pitch detected)
    pub pitch_frequency: f32,
    
    /// Confidence level of pitch detection (0.0 to 1.0)
    pub confidence: f32,
    
    /// Audio level (RMS) for visualization (0.0 to 1.0)
    pub audio_level: f32,
    
    /// Processing time for this buffer in milliseconds
    pub processing_time_ms: f32,
    
    /// Timestamp when this data was generated
    pub timestamp: f64,
    
    /// Whether pitch detection was successful
    pub pitch_detected: bool,
    
    /// Musical note information (if pitch detected)
    pub musical_note: Option<MusicalNote>,
    
    /// Audio quality indicators
    pub quality: AudioQuality,
}

impl Default for RealtimeAudioData {
    fn default() -> Self {
        Self {
            pitch_frequency: -1.0,
            confidence: 0.0,
            audio_level: 0.0,
            processing_time_ms: 0.0,
            timestamp: 0.0,
            pitch_detected: false,
            musical_note: None,
            quality: AudioQuality::default(),
        }
    }
}

/// Musical note information derived from pitch
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MusicalNote {
    /// Note name (e.g., "A", "C#", "Bb")
    pub note_name: String,
    
    /// Octave number
    pub octave: i32,
    
    /// Cents deviation from the nearest semitone (-50 to +50)
    pub cents_offset: f32,
    
    /// Whether the note is in tune (within tolerance)
    pub is_in_tune: bool,
    
    /// Target frequency for this note
    pub target_frequency: f32,
}

/// Audio quality indicators
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioQuality {
    /// Signal-to-noise ratio estimate
    pub signal_to_noise_ratio: f32,
    
    /// Whether the signal is clipping
    pub is_clipping: bool,
    
    /// Whether the signal level is too low
    pub is_too_quiet: bool,
    
    /// Overall quality score (0.0 to 1.0)
    pub quality_score: f32,
    
    /// Quality issues detected
    pub issues: Vec<AudioQualityIssue>,
}

impl Default for AudioQuality {
    fn default() -> Self {
        Self {
            signal_to_noise_ratio: 0.0,
            is_clipping: false,
            is_too_quiet: false,
            quality_score: 1.0,
            issues: Vec::new(),
        }
    }
}

/// Audio quality issues that can be detected
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AudioQualityIssue {
    /// Audio signal is clipping/distorting
    Clipping,
    /// Audio signal is too quiet
    TooQuiet,
    /// High noise level detected
    HighNoise,
    /// Unstable pitch detection
    UnstablePitch,
    /// Processing latency too high
    HighLatency,
    /// Microphone connection issues
    ConnectionIssues,
}

/// Performance metrics for audio processing
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioPerformanceMetrics {
    /// Average processing time per buffer in milliseconds
    pub avg_processing_time_ms: f32,
    
    /// Maximum processing time observed
    pub max_processing_time_ms: f32,
    
    /// Minimum processing time observed  
    pub min_processing_time_ms: f32,
    
    /// Total audio latency (input to output) in milliseconds
    pub total_latency_ms: f32,
    
    /// AudioContext base latency
    pub audio_context_latency_ms: f32,
    
    /// Output latency (speakers/headphones)
    pub output_latency_ms: f32,
    
    /// Processing latency (our algorithms)
    pub processing_latency_ms: f32,
    
    /// CPU utilization percentage for audio processing
    pub cpu_utilization: f32,
    
    /// Memory usage for audio buffers in bytes
    pub memory_usage_bytes: u64,
    
    /// Number of audio dropouts/glitches detected
    pub dropout_count: u32,
    
    /// Whether performance meets target requirements
    pub meets_target_latency: bool,
    
    /// Performance grade (A, B, C, D, F)
    pub performance_grade: PerformanceGrade,
    
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

impl Default for AudioPerformanceMetrics {
    fn default() -> Self {
        Self {
            avg_processing_time_ms: 0.0,
            max_processing_time_ms: 0.0,
            min_processing_time_ms: 0.0,
            total_latency_ms: 0.0,
            audio_context_latency_ms: 0.0,
            output_latency_ms: 0.0,
            processing_latency_ms: 0.0,
            cpu_utilization: 0.0,
            memory_usage_bytes: 0,
            dropout_count: 0,
            meets_target_latency: true,
            performance_grade: PerformanceGrade::A,
            recommendations: Vec::new(),
        }
    }
}

/// Performance grade for audio processing
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PerformanceGrade {
    /// Excellent performance (< 5ms latency)
    A,
    /// Good performance (5-10ms latency)
    B,
    /// Acceptable performance (10-20ms latency)
    C,
    /// Poor performance (20-50ms latency)
    D,
    /// Unacceptable performance (> 50ms latency)
    F,
}

/// Audio buffer configuration for optimization
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioBufferConfig {
    /// Buffer size in samples
    pub buffer_size: usize,
    
    /// Sample rate in Hz
    pub sample_rate: f32,
    
    /// Number of channels
    pub channel_count: u32,
    
    /// Target latency in milliseconds
    pub target_latency_ms: f32,
    
    /// Whether to use AudioWorklet (vs ScriptProcessor fallback)
    pub use_audio_worklet: bool,
    
    /// WASM optimization settings
    pub wasm_config: WasmOptimizationConfig,
}

impl Default for AudioBufferConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1024,
            sample_rate: 44100.0,
            channel_count: 1,
            target_latency_ms: 10.0,
            use_audio_worklet: true,
            wasm_config: WasmOptimizationConfig::default(),
        }
    }
}

/// WASM optimization configuration
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WasmOptimizationConfig {
    /// Enable SIMD optimizations if available
    pub enable_simd: bool,
    
    /// Use memory-efficient algorithms
    pub memory_efficient: bool,
    
    /// Maximum memory allocation for audio processing
    pub max_memory_mb: u32,
    
    /// Enable multi-threading if available
    pub enable_multithreading: bool,
    
    /// Optimization level (0-3, higher = more optimized)
    pub optimization_level: u8,
}

impl Default for WasmOptimizationConfig {
    fn default() -> Self {
        Self {
            enable_simd: true,
            memory_efficient: true,
            max_memory_mb: 64,
            enable_multithreading: false, // Not widely supported yet
            optimization_level: 2,
        }
    }
}

/// Audio stream connection state
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AudioStreamState {
    /// No stream connected
    Disconnected,
    /// Connecting to audio stream
    Connecting,
    /// Stream connected and active
    Connected,
    /// Stream connected but suspended
    Suspended,
    /// Stream connection failed
    Failed(String),
    /// Stream disconnecting
    Disconnecting,
}

impl Default for AudioStreamState {
    fn default() -> Self {
        Self::Disconnected
    }
}

/// Audio processing event for component communication
#[derive(Clone, Debug, PartialEq)]
pub enum AudioProcessingEvent {
    /// Audio engine state changed
    StateChanged(AudioProcessingState),
    
    /// New audio data available
    AudioDataUpdate(RealtimeAudioData),
    
    /// Performance metrics updated
    PerformanceUpdate(AudioPerformanceMetrics),
    
    /// Audio quality issue detected
    QualityIssue(AudioQualityIssue),
    
    /// Stream connection state changed
    StreamStateChanged(AudioStreamState),
    
    /// Device list updated
    DevicesUpdated(Vec<AudioDeviceInfo>),
    
    /// Error occurred
    Error(String),
} 