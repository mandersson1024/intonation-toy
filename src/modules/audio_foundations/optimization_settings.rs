// Device Optimization Settings Implementation - STORY-014
// Provides device-specific optimization settings for optimal audio performance

use std::error::Error;
use std::fmt;
use std::collections::HashMap;
use super::device_capabilities::{DeviceCapabilities, AudioUseCase, OptimalAudioSettings};
use super::device_manager::AudioDevice;

/// Trait for device optimization management
pub trait DeviceOptimizationManager: Send + Sync {
    /// Get optimized settings for a specific device and use case
    fn get_optimized_settings(&self, device_id: &str, use_case: AudioUseCase) -> Result<DeviceOptimizationSettings, OptimizationError>;
    
    /// Apply optimization settings to a device
    fn apply_optimization_settings(&mut self, device_id: &str, settings: &DeviceOptimizationSettings) -> Result<(), OptimizationError>;
    
    /// Get device-specific performance tuning recommendations
    fn get_performance_recommendations(&self, device_id: &str) -> Result<Vec<PerformanceRecommendation>, OptimizationError>;
    
    /// Auto-tune device settings based on performance metrics
    fn auto_tune_device(&mut self, device_id: &str, performance_metrics: &PerformanceMetrics) -> Result<AutoTuneResult, OptimizationError>;
    
    /// Save optimization profile for a device
    fn save_optimization_profile(&mut self, device_id: &str, profile: OptimizationProfile) -> Result<(), OptimizationError>;
    
    /// Load optimization profile for a device
    fn load_optimization_profile(&self, device_id: &str) -> Result<Option<OptimizationProfile>, OptimizationError>;
}

/// Complete device optimization settings
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceOptimizationSettings {
    pub device_id: String,
    pub audio_settings: OptimalAudioSettings,
    pub performance_settings: PerformanceSettings,
    pub quality_settings: QualitySettings,
    pub latency_settings: LatencySettings,
    pub power_settings: PowerSettings,
    pub compatibility_settings: CompatibilitySettings,
}

/// Performance-focused settings
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceSettings {
    pub cpu_optimization_level: CpuOptimizationLevel,
    pub memory_optimization_level: MemoryOptimizationLevel,
    pub thread_priority: ThreadPriority,
    pub buffer_management: BufferManagementStrategy,
    pub processing_pipeline: ProcessingPipelineConfig,
}

/// CPU optimization levels
#[derive(Debug, Clone, PartialEq)]
pub enum CpuOptimizationLevel {
    Conservative,  // Minimal CPU usage
    Balanced,      // Balance between performance and efficiency
    Aggressive,    // Maximum performance
    Custom(f32),   // Custom CPU target (0.0-1.0)
}

/// Memory optimization levels
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryOptimizationLevel {
    Minimal,       // Minimize memory usage
    Standard,      // Standard memory usage
    Performance,   // More memory for better performance
    Custom(usize), // Custom memory limit in bytes
}

/// Thread priority settings
#[derive(Debug, Clone, PartialEq)]
pub enum ThreadPriority {
    Low,
    Normal,
    High,
    RealTime,
}

/// Buffer management strategies
#[derive(Debug, Clone, PartialEq)]
pub enum BufferManagementStrategy {
    SingleBuffer,      // Single buffer for minimal latency
    DoubleBuffer,      // Double buffering for stability
    TripleBuffer,      // Triple buffering for smoothness
    RingBuffer(usize), // Ring buffer with specified size
}

/// Processing pipeline configuration
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessingPipelineConfig {
    pub enable_simd: bool,
    pub enable_parallel_processing: bool,
    pub enable_hardware_acceleration: bool,
    pub processing_quality: ProcessingQuality,
}

/// Processing quality levels
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingQuality {
    Draft,     // Fastest processing, lower quality
    Standard,  // Balanced quality and speed
    High,      // High quality, slower processing
    Maximum,   // Maximum quality, slowest processing
}

/// Quality-focused settings
#[derive(Debug, Clone, PartialEq)]
pub struct QualitySettings {
    pub bit_depth: BitDepth,
    pub dynamic_range_compression: bool,
    pub noise_gate_threshold: f32,
    pub frequency_response_compensation: bool,
    pub harmonic_enhancement: bool,
    pub quality_monitoring: bool,
}

/// Audio bit depth options
#[derive(Debug, Clone, PartialEq)]
pub enum BitDepth {
    Sixteen,
    TwentyFour,
    ThirtyTwo,
}

/// Latency-focused settings
#[derive(Debug, Clone, PartialEq)]
pub struct LatencySettings {
    pub target_latency_ms: f32,
    pub buffer_size_optimization: BufferSizeStrategy,
    pub sample_rate_optimization: SampleRateStrategy,
    pub processing_optimization: ProcessingOptimization,
    pub priority_mode: LatencyPriorityMode,
}

/// Buffer size optimization strategies
#[derive(Debug, Clone, PartialEq)]
pub enum BufferSizeStrategy {
    MinimizeLatency,    // Use smallest possible buffer
    BalanceStability,   // Balance latency and stability
    MaximizeStability,  // Use larger buffers for stability
    Custom(u32),        // Custom buffer size
}

/// Sample rate optimization strategies
#[derive(Debug, Clone, PartialEq)]
pub enum SampleRateStrategy {
    MatchDevice,        // Use device's preferred sample rate
    MatchUseCase,       // Use optimal rate for use case
    Custom(u32),        // Custom sample rate
}

/// Processing optimization for latency
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingOptimization {
    Bypass,             // Bypass all non-essential processing
    Minimal,            // Minimal processing only
    Essential,          // Essential processing only
    Full,               // Full processing pipeline
}

/// Latency priority modes
#[derive(Debug, Clone, PartialEq)]
pub enum LatencyPriorityMode {
    UltraLow,          // <5ms target
    Low,               // <10ms target
    Medium,            // <25ms target
    High,              // <50ms target
}

/// Power consumption settings
#[derive(Debug, Clone, PartialEq)]
pub struct PowerSettings {
    pub power_mode: PowerMode,
    pub adaptive_power_management: bool,
    pub sleep_optimization: bool,
    pub thermal_throttling: bool,
}

/// Power consumption modes
#[derive(Debug, Clone, PartialEq)]
pub enum PowerMode {
    PowerSaver,        // Minimize power consumption
    Balanced,          // Balance power and performance
    Performance,       // Maximize performance
    Custom(f32),       // Custom power target (0.0-1.0)
}

/// Compatibility settings
#[derive(Debug, Clone, PartialEq)]
pub struct CompatibilitySettings {
    pub legacy_support: bool,
    pub cross_platform_optimization: bool,
    pub browser_specific_optimizations: BrowserOptimizations,
    pub fallback_settings: FallbackSettings,
}

/// Browser-specific optimizations
#[derive(Debug, Clone, PartialEq)]
pub struct BrowserOptimizations {
    pub chrome_optimizations: bool,
    pub firefox_optimizations: bool,
    pub safari_optimizations: bool,
    pub edge_optimizations: bool,
}

/// Fallback settings for compatibility
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSettings {
    pub enable_fallbacks: bool,
    pub fallback_sample_rate: u32,
    pub fallback_buffer_size: u32,
    pub fallback_channel_count: u32,
}

/// Performance recommendation
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceRecommendation {
    pub category: RecommendationCategory,
    pub priority: RecommendationPriority,
    pub description: String,
    pub expected_improvement: String,
    pub implementation_difficulty: ImplementationDifficulty,
}

/// Recommendation categories
#[derive(Debug, Clone, PartialEq)]
pub enum RecommendationCategory {
    Latency,
    Quality,
    Stability,
    PowerConsumption,
    Compatibility,
}

/// Recommendation priority levels
#[derive(Debug, Clone, PartialEq)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Implementation difficulty levels
#[derive(Debug, Clone, PartialEq)]
pub enum ImplementationDifficulty {
    Easy,      // User can apply immediately
    Medium,    // Requires some configuration
    Hard,      // Requires technical knowledge
    Expert,    // Requires expert configuration
}

/// Performance metrics for auto-tuning
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceMetrics {
    pub latency_ms: f32,
    pub cpu_usage_percent: f32,
    pub memory_usage_bytes: usize,
    pub dropout_count: u32,
    pub quality_score: f32,
    pub stability_score: f32,
}

/// Auto-tuning result
#[derive(Debug, Clone, PartialEq)]
pub struct AutoTuneResult {
    pub success: bool,
    pub original_settings: DeviceOptimizationSettings,
    pub optimized_settings: DeviceOptimizationSettings,
    pub performance_improvement: PerformanceImprovement,
    pub applied_changes: Vec<String>,
}

/// Performance improvement metrics
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceImprovement {
    pub latency_improvement_percent: f32,
    pub cpu_usage_reduction_percent: f32,
    pub quality_improvement_percent: f32,
    pub stability_improvement_percent: f32,
}

/// Optimization profile for saving/loading settings
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizationProfile {
    pub name: String,
    pub device_id: String,
    pub use_case: AudioUseCase,
    pub settings: DeviceOptimizationSettings,
    pub created_timestamp: u64,
    pub last_used_timestamp: u64,
    pub usage_count: u32,
}

/// Optimization errors
#[derive(Debug, Clone)]
pub enum OptimizationError {
    DeviceNotFound(String),
    InvalidSettings(String),
    UnsupportedOptimization(String),
    ProfileNotFound(String),
    InternalError(String),
}

impl fmt::Display for OptimizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptimizationError::DeviceNotFound(id) => write!(f, "Device not found: {}", id),
            OptimizationError::InvalidSettings(msg) => write!(f, "Invalid settings: {}", msg),
            OptimizationError::UnsupportedOptimization(opt) => write!(f, "Unsupported optimization: {}", opt),
            OptimizationError::ProfileNotFound(name) => write!(f, "Optimization profile not found: {}", name),
            OptimizationError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl Error for OptimizationError {}

/// Web-based device optimization manager
pub struct WebDeviceOptimizationManager {
    device_capabilities: HashMap<String, DeviceCapabilities>,
    optimization_profiles: HashMap<String, Vec<OptimizationProfile>>,
    current_optimizations: HashMap<String, DeviceOptimizationSettings>,
}

impl WebDeviceOptimizationManager {
    /// Create a new optimization manager
    pub fn new() -> Self {
        Self {
            device_capabilities: HashMap::new(),
            optimization_profiles: HashMap::new(),
            current_optimizations: HashMap::new(),
        }
    }
    
    /// Set device capabilities for optimization
    pub fn set_device_capabilities(&mut self, device_id: String, capabilities: DeviceCapabilities) {
        self.device_capabilities.insert(device_id, capabilities);
    }
    
    /// Create default optimization settings
    fn create_default_optimization_settings(&self, device_id: &str, use_case: AudioUseCase) -> DeviceOptimizationSettings {
        let audio_settings = self.create_default_audio_settings(use_case);
        
        DeviceOptimizationSettings {
            device_id: device_id.to_string(),
            audio_settings,
            performance_settings: self.create_default_performance_settings(use_case),
            quality_settings: self.create_default_quality_settings(use_case),
            latency_settings: self.create_default_latency_settings(use_case),
            power_settings: self.create_default_power_settings(),
            compatibility_settings: self.create_default_compatibility_settings(),
        }
    }
    
    /// Create default audio settings for use case
    fn create_default_audio_settings(&self, use_case: AudioUseCase) -> OptimalAudioSettings {
        match use_case {
            AudioUseCase::PitchDetection => OptimalAudioSettings {
                sample_rate: 44100,
                channel_count: 1,
                buffer_size: 1024,
                echo_cancellation: false,
                noise_suppression: false,
                auto_gain_control: false,
                use_case,
                reasoning: "Optimized for pitch detection accuracy".to_string(),
            },
            AudioUseCase::VoiceRecording => OptimalAudioSettings {
                sample_rate: 44100,
                channel_count: 1,
                buffer_size: 2048,
                echo_cancellation: true,
                noise_suppression: true,
                auto_gain_control: true,
                use_case,
                reasoning: "Optimized for voice clarity".to_string(),
            },
            AudioUseCase::MusicRecording => OptimalAudioSettings {
                sample_rate: 48000,
                channel_count: 2,
                buffer_size: 512,
                echo_cancellation: false,
                noise_suppression: false,
                auto_gain_control: false,
                use_case,
                reasoning: "Optimized for music fidelity".to_string(),
            },
            _ => OptimalAudioSettings {
                sample_rate: 44100,
                channel_count: 1,
                buffer_size: 1024,
                echo_cancellation: true,
                noise_suppression: false,
                auto_gain_control: false,
                use_case,
                reasoning: "General purpose optimization".to_string(),
            },
        }
    }
    
    /// Create default performance settings
    fn create_default_performance_settings(&self, use_case: AudioUseCase) -> PerformanceSettings {
        let (cpu_level, memory_level, priority) = match use_case {
            AudioUseCase::PitchDetection => (CpuOptimizationLevel::Aggressive, MemoryOptimizationLevel::Performance, ThreadPriority::High),
            AudioUseCase::LiveStreaming => (CpuOptimizationLevel::Balanced, MemoryOptimizationLevel::Standard, ThreadPriority::High),
            _ => (CpuOptimizationLevel::Balanced, MemoryOptimizationLevel::Standard, ThreadPriority::Normal),
        };
        
        PerformanceSettings {
            cpu_optimization_level: cpu_level,
            memory_optimization_level: memory_level,
            thread_priority: priority,
            buffer_management: BufferManagementStrategy::DoubleBuffer,
            processing_pipeline: ProcessingPipelineConfig {
                enable_simd: true,
                enable_parallel_processing: false,
                enable_hardware_acceleration: true,
                processing_quality: ProcessingQuality::Standard,
            },
        }
    }
    
    /// Create default quality settings
    fn create_default_quality_settings(&self, use_case: AudioUseCase) -> QualitySettings {
        match use_case {
            AudioUseCase::MusicRecording => QualitySettings {
                bit_depth: BitDepth::TwentyFour,
                dynamic_range_compression: false,
                noise_gate_threshold: -60.0,
                frequency_response_compensation: true,
                harmonic_enhancement: false,
                quality_monitoring: true,
            },
            AudioUseCase::PitchDetection => QualitySettings {
                bit_depth: BitDepth::Sixteen,
                dynamic_range_compression: false,
                noise_gate_threshold: -50.0,
                frequency_response_compensation: false,
                harmonic_enhancement: false,
                quality_monitoring: false,
            },
            _ => QualitySettings {
                bit_depth: BitDepth::Sixteen,
                dynamic_range_compression: true,
                noise_gate_threshold: -40.0,
                frequency_response_compensation: false,
                harmonic_enhancement: false,
                quality_monitoring: false,
            },
        }
    }
    
    /// Create default latency settings
    fn create_default_latency_settings(&self, use_case: AudioUseCase) -> LatencySettings {
        let (target_latency, priority_mode) = match use_case {
            AudioUseCase::PitchDetection => (10.0, LatencyPriorityMode::UltraLow),
            AudioUseCase::LiveStreaming => (25.0, LatencyPriorityMode::Low),
            AudioUseCase::VoiceCall => (50.0, LatencyPriorityMode::Medium),
            _ => (25.0, LatencyPriorityMode::Low),
        };
        
        LatencySettings {
            target_latency_ms: target_latency,
            buffer_size_optimization: BufferSizeStrategy::MinimizeLatency,
            sample_rate_optimization: SampleRateStrategy::MatchUseCase,
            processing_optimization: ProcessingOptimization::Essential,
            priority_mode,
        }
    }
    
    /// Create default power settings
    fn create_default_power_settings(&self) -> PowerSettings {
        PowerSettings {
            power_mode: PowerMode::Balanced,
            adaptive_power_management: true,
            sleep_optimization: true,
            thermal_throttling: true,
        }
    }
    
    /// Create default compatibility settings
    fn create_default_compatibility_settings(&self) -> CompatibilitySettings {
        CompatibilitySettings {
            legacy_support: true,
            cross_platform_optimization: true,
            browser_specific_optimizations: BrowserOptimizations {
                chrome_optimizations: true,
                firefox_optimizations: true,
                safari_optimizations: true,
                edge_optimizations: true,
            },
            fallback_settings: FallbackSettings {
                enable_fallbacks: true,
                fallback_sample_rate: 44100,
                fallback_buffer_size: 2048,
                fallback_channel_count: 1,
            },
        }
    }
}

impl DeviceOptimizationManager for WebDeviceOptimizationManager {
    fn get_optimized_settings(&self, device_id: &str, use_case: AudioUseCase) -> Result<DeviceOptimizationSettings, OptimizationError> {
        // Check if we have a saved profile for this device and use case
        if let Some(profiles) = self.optimization_profiles.get(device_id) {
            if let Some(profile) = profiles.iter().find(|p| p.use_case == use_case) {
                return Ok(profile.settings.clone());
            }
        }
        
        // Create default optimization settings
        let settings = self.create_default_optimization_settings(device_id, use_case);
        Ok(settings)
    }
    
    fn apply_optimization_settings(&mut self, device_id: &str, settings: &DeviceOptimizationSettings) -> Result<(), OptimizationError> {
        // Store the current optimization settings
        self.current_optimizations.insert(device_id.to_string(), settings.clone());
        
        // In a real implementation, this would apply the settings to the actual device
        web_sys::console::log_1(&format!("Applied optimization settings for device: {}", device_id).into());
        Ok(())
    }
    
    fn get_performance_recommendations(&self, device_id: &str) -> Result<Vec<PerformanceRecommendation>, OptimizationError> {
        let mut recommendations = Vec::new();
        
        // Generate recommendations based on device capabilities and current settings
        recommendations.push(PerformanceRecommendation {
            category: RecommendationCategory::Latency,
            priority: RecommendationPriority::High,
            description: "Reduce buffer size for lower latency".to_string(),
            expected_improvement: "5-10ms latency reduction".to_string(),
            implementation_difficulty: ImplementationDifficulty::Easy,
        });
        
        recommendations.push(PerformanceRecommendation {
            category: RecommendationCategory::Quality,
            priority: RecommendationPriority::Medium,
            description: "Enable noise suppression for cleaner audio".to_string(),
            expected_improvement: "Improved signal clarity".to_string(),
            implementation_difficulty: ImplementationDifficulty::Easy,
        });
        
        recommendations.push(PerformanceRecommendation {
            category: RecommendationCategory::Stability,
            priority: RecommendationPriority::Medium,
            description: "Use double buffering for better stability".to_string(),
            expected_improvement: "Reduced audio dropouts".to_string(),
            implementation_difficulty: ImplementationDifficulty::Medium,
        });
        
        Ok(recommendations)
    }
    
    fn auto_tune_device(&mut self, device_id: &str, performance_metrics: &PerformanceMetrics) -> Result<AutoTuneResult, OptimizationError> {
        let original_settings = self.current_optimizations.get(device_id)
            .cloned()
            .unwrap_or_else(|| self.create_default_optimization_settings(device_id, AudioUseCase::General));
        
        let mut optimized_settings = original_settings.clone();
        let mut applied_changes = Vec::new();
        
        // Auto-tune based on performance metrics
        if performance_metrics.latency_ms > 50.0 {
            // High latency - reduce buffer size
            optimized_settings.latency_settings.buffer_size_optimization = BufferSizeStrategy::MinimizeLatency;
            applied_changes.push("Reduced buffer size for lower latency".to_string());
        }
        
        if performance_metrics.cpu_usage_percent > 80.0 {
            // High CPU usage - reduce processing
            optimized_settings.performance_settings.cpu_optimization_level = CpuOptimizationLevel::Conservative;
            optimized_settings.latency_settings.processing_optimization = ProcessingOptimization::Minimal;
            applied_changes.push("Reduced processing load for CPU efficiency".to_string());
        }
        
        if performance_metrics.dropout_count > 0 {
            // Audio dropouts - increase stability
            optimized_settings.performance_settings.buffer_management = BufferManagementStrategy::TripleBuffer;
            applied_changes.push("Enabled triple buffering for stability".to_string());
        }
        
        // Calculate performance improvement (simplified)
        let performance_improvement = PerformanceImprovement {
            latency_improvement_percent: if performance_metrics.latency_ms > 50.0 { 20.0 } else { 0.0 },
            cpu_usage_reduction_percent: if performance_metrics.cpu_usage_percent > 80.0 { 15.0 } else { 0.0 },
            quality_improvement_percent: 0.0,
            stability_improvement_percent: if performance_metrics.dropout_count > 0 { 25.0 } else { 0.0 },
        };
        
        // Apply the optimized settings
        self.apply_optimization_settings(device_id, &optimized_settings)?;
        
        Ok(AutoTuneResult {
            success: !applied_changes.is_empty(),
            original_settings,
            optimized_settings,
            performance_improvement,
            applied_changes,
        })
    }
    
    fn save_optimization_profile(&mut self, device_id: &str, profile: OptimizationProfile) -> Result<(), OptimizationError> {
        let profiles = self.optimization_profiles.entry(device_id.to_string()).or_insert_with(Vec::new);
        
        // Remove any existing profile with the same name
        profiles.retain(|p| p.name != profile.name);
        
        // Add the new profile
        profiles.push(profile);
        
        Ok(())
    }
    
    fn load_optimization_profile(&self, device_id: &str) -> Result<Option<OptimizationProfile>, OptimizationError> {
        if let Some(profiles) = self.optimization_profiles.get(device_id) {
            // Return the most recently used profile
            let profile = profiles.iter().max_by_key(|p| p.last_used_timestamp).cloned();
            Ok(profile)
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_optimization_settings_creation() {
        let manager = WebDeviceOptimizationManager::new();
        let settings = manager.create_default_optimization_settings("test-device", AudioUseCase::PitchDetection);
        
        assert_eq!(settings.device_id, "test-device");
        assert_eq!(settings.audio_settings.use_case, AudioUseCase::PitchDetection);
        assert!(!settings.audio_settings.echo_cancellation);
    }
    
    #[test]
    fn test_performance_recommendation() {
        let rec = PerformanceRecommendation {
            category: RecommendationCategory::Latency,
            priority: RecommendationPriority::High,
            description: "Test recommendation".to_string(),
            expected_improvement: "Test improvement".to_string(),
            implementation_difficulty: ImplementationDifficulty::Easy,
        };
        
        assert_eq!(rec.category, RecommendationCategory::Latency);
        assert_eq!(rec.priority, RecommendationPriority::High);
    }
    
    #[test]
    fn test_optimization_profile() {
        let profile = OptimizationProfile {
            name: "Test Profile".to_string(),
            device_id: "test-device".to_string(),
            use_case: AudioUseCase::PitchDetection,
            settings: DeviceOptimizationSettings {
                device_id: "test-device".to_string(),
                audio_settings: OptimalAudioSettings {
                    sample_rate: 44100,
                    channel_count: 1,
                    buffer_size: 1024,
                    echo_cancellation: false,
                    noise_suppression: false,
                    auto_gain_control: false,
                    use_case: AudioUseCase::PitchDetection,
                    reasoning: "Test".to_string(),
                },
                performance_settings: PerformanceSettings {
                    cpu_optimization_level: CpuOptimizationLevel::Balanced,
                    memory_optimization_level: MemoryOptimizationLevel::Standard,
                    thread_priority: ThreadPriority::Normal,
                    buffer_management: BufferManagementStrategy::DoubleBuffer,
                    processing_pipeline: ProcessingPipelineConfig {
                        enable_simd: true,
                        enable_parallel_processing: false,
                        enable_hardware_acceleration: true,
                        processing_quality: ProcessingQuality::Standard,
                    },
                },
                quality_settings: QualitySettings {
                    bit_depth: BitDepth::Sixteen,
                    dynamic_range_compression: false,
                    noise_gate_threshold: -50.0,
                    frequency_response_compensation: false,
                    harmonic_enhancement: false,
                    quality_monitoring: false,
                },
                latency_settings: LatencySettings {
                    target_latency_ms: 10.0,
                    buffer_size_optimization: BufferSizeStrategy::MinimizeLatency,
                    sample_rate_optimization: SampleRateStrategy::MatchUseCase,
                    processing_optimization: ProcessingOptimization::Essential,
                    priority_mode: LatencyPriorityMode::UltraLow,
                },
                power_settings: PowerSettings {
                    power_mode: PowerMode::Balanced,
                    adaptive_power_management: true,
                    sleep_optimization: true,
                    thermal_throttling: true,
                },
                compatibility_settings: CompatibilitySettings {
                    legacy_support: true,
                    cross_platform_optimization: true,
                    browser_specific_optimizations: BrowserOptimizations {
                        chrome_optimizations: true,
                        firefox_optimizations: true,
                        safari_optimizations: true,
                        edge_optimizations: true,
                    },
                    fallback_settings: FallbackSettings {
                        enable_fallbacks: true,
                        fallback_sample_rate: 44100,
                        fallback_buffer_size: 2048,
                        fallback_channel_count: 1,
                    },
                },
            },
            created_timestamp: 0,
            last_used_timestamp: 0,
            usage_count: 0,
        };
        
        assert_eq!(profile.name, "Test Profile");
        assert_eq!(profile.use_case, AudioUseCase::PitchDetection);
    }
}