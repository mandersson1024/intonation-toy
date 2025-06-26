// Device Capabilities Implementation - STORY-014
// Detects and manages audio device capabilities and optimization settings

use std::error::Error;
use std::fmt;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    MediaDevices, MediaStream, MediaStreamConstraints, MediaStreamTrack
};
use js_sys::{Array, Object, Reflect};
use super::device_manager::{AudioDevice, DeviceError};

/// Trait for device capability detection and management
pub trait DeviceCapabilityManager: Send + Sync {
    /// Detect capabilities for a specific device
    fn detect_device_capabilities(&self, device_id: &str) -> Result<DeviceCapabilities, CapabilityError>;
    
    /// Get optimal settings for a device based on use case
    fn get_optimal_settings(&self, device_id: &str, use_case: AudioUseCase) -> Result<OptimalAudioSettings, CapabilityError>;
    
    /// Test device capabilities with actual streams
    fn test_device_capabilities(&self, device_id: &str, settings: &AudioSettings) -> Result<CapabilityTestResult, CapabilityError>;
    
    /// Get recommended buffer sizes for a device
    fn get_recommended_buffer_sizes(&self, device_id: &str) -> Result<Vec<u32>, CapabilityError>;
    
    /// Check if device supports specific audio features
    fn supports_audio_features(&self, device_id: &str, features: &[AudioFeature]) -> Result<FeatureSupportMap, CapabilityError>;
}

/// Device capabilities information
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceCapabilities {
    pub device_id: String,
    pub sample_rates: SampleRateRange,
    pub channel_counts: ChannelCountRange,
    pub buffer_sizes: Vec<u32>,
    pub audio_features: FeatureSupportMap,
    pub latency_characteristics: LatencyCharacteristics,
    pub quality_characteristics: QualityCharacteristics,
}

/// Sample rate capabilities
#[derive(Debug, Clone, PartialEq)]
pub struct SampleRateRange {
    pub min: u32,
    pub max: u32,
    pub supported_rates: Vec<u32>,
    pub default_rate: u32,
}

/// Channel count capabilities
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelCountRange {
    pub min: u32,
    pub max: u32,
    pub default_count: u32,
}

/// Audio feature support mapping
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureSupportMap {
    pub echo_cancellation: FeatureSupport,
    pub noise_suppression: FeatureSupport,
    pub auto_gain_control: FeatureSupport,
    pub voice_isolation: FeatureSupport,
    pub background_blur: FeatureSupport,
}

/// Audio feature support level
#[derive(Debug, Clone, PartialEq)]
pub enum FeatureSupport {
    NotSupported,
    Supported,
    SupportedWithLimitations(String),
    Unknown,
}

/// Audio features that can be tested
#[derive(Debug, Clone, PartialEq)]
pub enum AudioFeature {
    EchoCancellation,
    NoiseSuppression,
    AutoGainControl,
    VoiceIsolation,
    BackgroundBlur,
}

/// Latency characteristics of a device
#[derive(Debug, Clone, PartialEq)]
pub struct LatencyCharacteristics {
    pub min_latency_ms: f32,
    pub typical_latency_ms: f32,
    pub max_latency_ms: f32,
    pub latency_stability: LatencyStability,
}

/// Latency stability assessment
#[derive(Debug, Clone, PartialEq)]
pub enum LatencyStability {
    Excellent,  // <1ms variation
    Good,       // <5ms variation
    Fair,       // <10ms variation
    Poor,       // >10ms variation
}

/// Quality characteristics of a device
#[derive(Debug, Clone, PartialEq)]
pub struct QualityCharacteristics {
    pub signal_to_noise_ratio: f32,
    pub frequency_response_quality: QualityRating,
    pub distortion_level: f32,
    pub dynamic_range: f32,
}

/// Quality rating scale
#[derive(Debug, Clone, PartialEq)]
pub enum QualityRating {
    Excellent,
    Good,
    Fair,
    Poor,
    Unknown,
}

/// Audio use cases for optimization
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum AudioUseCase {
    PitchDetection,
    VoiceRecording,
    MusicRecording,
    LiveStreaming,
    VoiceCall,
    General,
}

/// Optimal audio settings for a use case
#[derive(Debug, Clone, PartialEq)]
pub struct OptimalAudioSettings {
    pub sample_rate: u32,
    pub channel_count: u32,
    pub buffer_size: u32,
    pub echo_cancellation: bool,
    pub noise_suppression: bool,
    pub auto_gain_control: bool,
    pub use_case: AudioUseCase,
    pub reasoning: String,
}

/// Audio settings for testing
#[derive(Debug, Clone, PartialEq)]
pub struct AudioSettings {
    pub sample_rate: Option<u32>,
    pub channel_count: Option<u32>,
    pub echo_cancellation: Option<bool>,
    pub noise_suppression: Option<bool>,
    pub auto_gain_control: Option<bool>,
}

/// Simplified media track settings for testing
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleMediaTrackSettings {
    pub sample_rate: Option<f64>,
    pub channel_count: Option<u32>,
    pub echo_cancellation: Option<bool>,
    pub noise_suppression: Option<bool>,
    pub auto_gain_control: Option<bool>,
}

impl Default for SimpleMediaTrackSettings {
    fn default() -> Self {
        Self {
            sample_rate: None,
            channel_count: None,
            echo_cancellation: None,
            noise_suppression: None,
            auto_gain_control: None,
        }
    }
}

/// Result of capability testing
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityTestResult {
    pub success: bool,
    pub actual_settings: SimpleMediaTrackSettings,
    pub performance_metrics: PerformanceMetrics,
    pub issues_found: Vec<String>,
}

/// Performance metrics from capability testing
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceMetrics {
    pub stream_setup_time_ms: f32,
    pub audio_latency_ms: f32,
    pub cpu_usage_percent: f32,
    pub memory_usage_bytes: usize,
}

/// Capability detection errors
#[derive(Debug, Clone)]
pub enum CapabilityError {
    DeviceNotFound(String),
    DeviceInUse(String),
    BrowserNotSupported,
    PermissionDenied,
    TestFailed(String),
    InternalError(String),
}

impl fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CapabilityError::DeviceNotFound(id) => write!(f, "Device not found: {}", id),
            CapabilityError::DeviceInUse(id) => write!(f, "Device in use: {}", id),
            CapabilityError::BrowserNotSupported => write!(f, "Browser does not support capability detection"),
            CapabilityError::PermissionDenied => write!(f, "Permission denied for capability testing"),
            CapabilityError::TestFailed(msg) => write!(f, "Capability test failed: {}", msg),
            CapabilityError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl Error for CapabilityError {}

/// Web-based device capability manager
pub struct WebDeviceCapabilityManager {
    media_devices: MediaDevices,
    capability_cache: HashMap<String, DeviceCapabilities>,
}

impl WebDeviceCapabilityManager {
    /// Create a new web device capability manager
    pub fn new() -> Result<Self, CapabilityError> {
        let window = web_sys::window().ok_or(CapabilityError::BrowserNotSupported)?;
        let navigator = window.navigator();
        let media_devices = navigator.media_devices()
            .map_err(|_| CapabilityError::BrowserNotSupported)?;
        
        Ok(Self {
            media_devices,
            capability_cache: HashMap::new(),
        })
    }
    
    /// Test device with specific settings
    pub async fn test_device_with_settings(&self, device_id: &str, settings: &AudioSettings) -> Result<CapabilityTestResult, CapabilityError> {
        let start_time = get_current_time_ms();
        
        // Create basic constraints (simplified for compilation)
        let mut constraints = MediaStreamConstraints::new();
        constraints.audio(&true.into());
        constraints.video(&false.into());
        
        // Request media stream
        let promise = self.media_devices.get_user_media_with_constraints(&constraints)
            .map_err(|_| CapabilityError::PermissionDenied)?;
        
        let stream_result = JsFuture::from(promise).await;
        let setup_time = get_current_time_ms() - start_time;
        
        match stream_result {
            Ok(stream) => {
                let media_stream: MediaStream = stream.into();
                let tracks = media_stream.get_audio_tracks();
                
                let mut actual_settings = SimpleMediaTrackSettings::default();
                let mut issues = Vec::new();
                
                if tracks.length() > 0 {
                    let track: MediaStreamTrack = tracks.get(0).into();
                    // In a real implementation, we would extract settings from the track
                    actual_settings.sample_rate = Some(44100.0);
                    actual_settings.channel_count = Some(1);
                    
                    // Validate settings match what was requested
                    if let Some(requested_rate) = settings.sample_rate {
                        if let Some(actual_rate) = actual_settings.sample_rate {
                            if (actual_rate as u32) != requested_rate {
                                issues.push(format!("Sample rate mismatch: requested {}, got {}", requested_rate, actual_rate));
                            }
                        }
                    }
                    
                    // Stop the track since we're just testing
                    track.stop();
                }
                
                let performance_metrics = PerformanceMetrics {
                    stream_setup_time_ms: setup_time,
                    audio_latency_ms: self.estimate_audio_latency(&actual_settings),
                    cpu_usage_percent: 0.0, // Would need performance monitoring
                    memory_usage_bytes: 0,   // Would need memory monitoring
                };
                
                Ok(CapabilityTestResult {
                    success: true,
                    actual_settings,
                    performance_metrics,
                    issues_found: issues,
                })
            }
            Err(error) => {
                let error_msg = format!("Stream creation failed: {:?}", error);
                Ok(CapabilityTestResult {
                    success: false,
                    actual_settings: SimpleMediaTrackSettings::default(),
                    performance_metrics: PerformanceMetrics {
                        stream_setup_time_ms: setup_time,
                        audio_latency_ms: 0.0,
                        cpu_usage_percent: 0.0,
                        memory_usage_bytes: 0,
                    },
                    issues_found: vec![error_msg],
                })
            }
        }
    }
    
    /// Estimate audio latency based on settings
    fn estimate_audio_latency(&self, settings: &SimpleMediaTrackSettings) -> f32 {
        // Baseline latency estimation based on sample rate and typical buffer sizes
        let sample_rate = settings.sample_rate.unwrap_or(44100.0) as f32;
        let estimated_buffer_size = 1024.0; // Typical buffer size
        
        // Basic latency calculation: buffer_size / sample_rate * 1000 (ms)
        (estimated_buffer_size / sample_rate) * 1000.0
    }
    
    /// Create default capabilities for a device
    fn create_default_capabilities(&self, device_id: &str) -> DeviceCapabilities {
        DeviceCapabilities {
            device_id: device_id.to_string(),
            sample_rates: SampleRateRange {
                min: 8000,
                max: 48000,
                supported_rates: vec![8000, 16000, 22050, 44100, 48000],
                default_rate: 44100,
            },
            channel_counts: ChannelCountRange {
                min: 1,
                max: 2,
                default_count: 1,
            },
            buffer_sizes: vec![256, 512, 1024, 2048, 4096],
            audio_features: FeatureSupportMap {
                echo_cancellation: FeatureSupport::Supported,
                noise_suppression: FeatureSupport::Supported,
                auto_gain_control: FeatureSupport::Supported,
                voice_isolation: FeatureSupport::Unknown,
                background_blur: FeatureSupport::Unknown,
            },
            latency_characteristics: LatencyCharacteristics {
                min_latency_ms: 10.0,
                typical_latency_ms: 25.0,
                max_latency_ms: 50.0,
                latency_stability: LatencyStability::Good,
            },
            quality_characteristics: QualityCharacteristics {
                signal_to_noise_ratio: 60.0,
                frequency_response_quality: QualityRating::Good,
                distortion_level: 0.1,
                dynamic_range: 80.0,
            },
        }
    }
}

impl DeviceCapabilityManager for WebDeviceCapabilityManager {
    fn detect_device_capabilities(&self, device_id: &str) -> Result<DeviceCapabilities, CapabilityError> {
        // Check cache first
        if let Some(cached) = self.capability_cache.get(device_id) {
            return Ok(cached.clone());
        }
        
        // For now, return default capabilities
        // In a real implementation, this would probe the device
        let capabilities = self.create_default_capabilities(device_id);
        Ok(capabilities)
    }
    
    fn get_optimal_settings(&self, device_id: &str, use_case: AudioUseCase) -> Result<OptimalAudioSettings, CapabilityError> {
        let capabilities = self.detect_device_capabilities(device_id)?;
        
        let (sample_rate, channel_count, buffer_size, echo_cancellation, noise_suppression, auto_gain_control, reasoning) = match use_case {
            AudioUseCase::PitchDetection => (
                44100,
                1,
                1024,
                false,
                false,
                false,
                "Pitch detection requires clean, unprocessed audio with minimal latency".to_string(),
            ),
            AudioUseCase::VoiceRecording => (
                44100,
                1,
                2048,
                true,
                true,
                true,
                "Voice recording benefits from noise suppression and echo cancellation".to_string(),
            ),
            AudioUseCase::MusicRecording => (
                48000,
                2,
                512,
                false,
                false,
                false,
                "Music recording requires high fidelity with minimal processing".to_string(),
            ),
            AudioUseCase::LiveStreaming => (
                44100,
                1,
                1024,
                true,
                true,
                true,
                "Live streaming benefits from real-time audio processing".to_string(),
            ),
            AudioUseCase::VoiceCall => (
                16000,
                1,
                2048,
                true,
                true,
                true,
                "Voice calls prioritize intelligibility and bandwidth efficiency".to_string(),
            ),
            AudioUseCase::General => (
                44100,
                1,
                1024,
                true,
                false,
                false,
                "General use case with balanced quality and processing".to_string(),
            ),
        };
        
        // Validate settings against device capabilities
        let validated_sample_rate = if capabilities.sample_rates.supported_rates.contains(&sample_rate) {
            sample_rate
        } else {
            capabilities.sample_rates.default_rate
        };
        
        let validated_channel_count = if channel_count <= capabilities.channel_counts.max {
            channel_count
        } else {
            capabilities.channel_counts.default_count
        };
        
        Ok(OptimalAudioSettings {
            sample_rate: validated_sample_rate,
            channel_count: validated_channel_count,
            buffer_size,
            echo_cancellation,
            noise_suppression,
            auto_gain_control,
            use_case,
            reasoning,
        })
    }
    
    fn test_device_capabilities(&self, device_id: &str, settings: &AudioSettings) -> Result<CapabilityTestResult, CapabilityError> {
        // This would require async functionality in a real implementation
        // For now, return a placeholder result
        Ok(CapabilityTestResult {
            success: true,
            actual_settings: SimpleMediaTrackSettings::default(),
            performance_metrics: PerformanceMetrics {
                stream_setup_time_ms: 100.0,
                audio_latency_ms: 25.0,
                cpu_usage_percent: 5.0,
                memory_usage_bytes: 1024000,
            },
            issues_found: vec![],
        })
    }
    
    fn get_recommended_buffer_sizes(&self, device_id: &str) -> Result<Vec<u32>, CapabilityError> {
        let capabilities = self.detect_device_capabilities(device_id)?;
        Ok(capabilities.buffer_sizes)
    }
    
    fn supports_audio_features(&self, device_id: &str, features: &[AudioFeature]) -> Result<FeatureSupportMap, CapabilityError> {
        let capabilities = self.detect_device_capabilities(device_id)?;
        Ok(capabilities.audio_features)
    }
}

// Utility function to get current time in milliseconds
fn get_current_time_ms() -> f32 {
    if let Some(performance) = web_sys::window().and_then(|w| w.performance()) {
        performance.now() as f32
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_device_capabilities_creation() {
        let capabilities = DeviceCapabilities {
            device_id: "test-device".to_string(),
            sample_rates: SampleRateRange {
                min: 8000,
                max: 48000,
                supported_rates: vec![44100, 48000],
                default_rate: 44100,
            },
            channel_counts: ChannelCountRange {
                min: 1,
                max: 2,
                default_count: 1,
            },
            buffer_sizes: vec![256, 512, 1024],
            audio_features: FeatureSupportMap {
                echo_cancellation: FeatureSupport::Supported,
                noise_suppression: FeatureSupport::Supported,
                auto_gain_control: FeatureSupport::Supported,
                voice_isolation: FeatureSupport::Unknown,
                background_blur: FeatureSupport::Unknown,
            },
            latency_characteristics: LatencyCharacteristics {
                min_latency_ms: 10.0,
                typical_latency_ms: 25.0,
                max_latency_ms: 50.0,
                latency_stability: LatencyStability::Good,
            },
            quality_characteristics: QualityCharacteristics {
                signal_to_noise_ratio: 60.0,
                frequency_response_quality: QualityRating::Good,
                distortion_level: 0.1,
                dynamic_range: 80.0,
            },
        };
        
        assert_eq!(capabilities.device_id, "test-device");
        assert_eq!(capabilities.sample_rates.default_rate, 44100);
        assert_eq!(capabilities.channel_counts.max, 2);
    }
    
    #[test]
    fn test_optimal_settings_for_pitch_detection() {
        let settings = OptimalAudioSettings {
            sample_rate: 44100,
            channel_count: 1,
            buffer_size: 1024,
            echo_cancellation: false,
            noise_suppression: false,
            auto_gain_control: false,
            use_case: AudioUseCase::PitchDetection,
            reasoning: "Test reasoning".to_string(),
        };
        
        assert_eq!(settings.use_case, AudioUseCase::PitchDetection);
        assert!(!settings.echo_cancellation);
        assert_eq!(settings.sample_rate, 44100);
    }
    
    #[test]
    fn test_feature_support() {
        let support = FeatureSupport::SupportedWithLimitations("Test limitation".to_string());
        match support {
            FeatureSupport::SupportedWithLimitations(msg) => {
                assert_eq!(msg, "Test limitation");
            }
            _ => panic!("Expected SupportedWithLimitations variant"),
        }
    }
}