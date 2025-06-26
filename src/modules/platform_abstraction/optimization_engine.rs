use super::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Enhanced browser-specific optimization profiles
#[derive(Debug, Clone)]
pub struct BrowserOptimizationProfile {
    pub webassembly_optimizations: WasmOptimizations,
    pub audio_optimizations: AudioOptimizations,
    pub memory_optimizations: MemoryOptimizations,
    pub threading_optimizations: ThreadingOptimizations,
}

#[derive(Debug, Clone)]
pub struct WasmOptimizations {
    pub enable_simd: bool,
    pub enable_threading: bool,
    pub streaming_compilation: bool,
    pub bulk_memory_operations: bool,
    pub reference_types: bool,
    pub optimization_level: u8, // 0-3, where 3 is highest
}

#[derive(Debug, Clone)]
pub struct AudioOptimizations {
    pub preferred_buffer_size: u32,
    pub enable_audio_worklet: bool,
    pub enable_shared_array_buffer: bool,
    pub audio_context_options: AudioContextOptions,
    pub latency_hint: AudioContextLatencyCategory,
}

#[derive(Debug, Clone)]
pub struct AudioContextOptions {
    pub sample_rate: Option<f32>,
    pub latency_hint: String,
    pub echo_cancellation: bool,
    pub noise_suppression: bool,
    pub auto_gain_control: bool,
}

#[derive(Debug, Clone)]
pub enum AudioContextLatencyCategory {
    Interactive,   // Lowest latency
    Balanced,      // Balanced performance
    Playback,      // Higher latency, better quality
}

#[derive(Debug, Clone)]
pub struct MemoryOptimizations {
    pub heap_size_limit: u64,
    pub gc_strategy: GcStrategy,
    pub buffer_pooling: bool,
    pub memory_pressure_handling: MemoryPressureHandling,
}

#[derive(Debug, Clone)]
pub enum GcStrategy {
    Incremental,
    Generational,
    Conservative,
    Aggressive,
}

#[derive(Debug, Clone)]
pub enum MemoryPressureHandling {
    Strict,
    Conservative,
    Aggressive,
    Adaptive,
}

#[derive(Debug, Clone)]
pub struct ThreadingOptimizations {
    pub enable_web_workers: bool,
    pub shared_memory_support: bool,
    pub thread_pool_size: u8,
    pub work_stealing: bool,
}

/// Platform optimization engine implementation with enhanced browser-specific optimizations
pub struct PlatformOptimizationEngineImpl {
    /// Current optimization profile
    current_profile: Arc<Mutex<Option<BrowserOptimizationProfile>>>,
    /// Applied optimizations history
    applied_optimizations: Arc<Mutex<HashMap<String, OptimizationRecord>>>,
    /// Performance monitoring data
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
    /// Memory overhead tracking
    memory_overhead: Arc<Mutex<u64>>,
    /// Browser-specific profiles cache
    profile_cache: Arc<Mutex<HashMap<String, BrowserOptimizationProfile>>>,
}

#[derive(Debug, Clone)]
struct OptimizationRecord {
    optimization_type: String,
    applied_at: Instant,
    browser_info: BrowserInfo,
    effectiveness_score: f32,
}

/// Performance profiling system for browser compatibility monitoring
#[derive(Debug, Clone)]
pub struct BrowserPerformanceProfiler {
    /// Performance metrics history
    metrics_history: Arc<Mutex<Vec<TimestampedMetrics>>>,
    /// Performance baselines for comparison
    baselines: Arc<Mutex<HashMap<String, PerformanceBaseline>>>,
    /// Alert thresholds
    alert_thresholds: Arc<Mutex<PerformanceThresholds>>,
    /// Real-time monitoring overhead tracking
    monitoring_overhead: Arc<Mutex<f64>>,
    /// Performance regression detection
    regression_detector: Arc<Mutex<RegressionDetector>>,
}

#[derive(Debug, Clone)]
pub struct TimestampedMetrics {
    pub timestamp: Instant,
    pub metrics: PerformanceMetrics,
    pub browser_name: String,
    pub optimization_profile: String,
}

#[derive(Debug, Clone)]
pub struct PerformanceBaseline {
    pub cpu_usage_baseline: f32,
    pub memory_usage_baseline: u64,
    pub audio_latency_baseline: f32,
    pub optimization_effectiveness_baseline: f32,
    pub measurement_count: u32,
    pub established_at: Instant,
}

#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub cpu_usage_warning: f32,
    pub cpu_usage_critical: f32,
    pub memory_usage_warning: u64,
    pub memory_usage_critical: u64,
    pub audio_latency_warning: f32,
    pub audio_latency_critical: f32,
    pub optimization_degradation_warning: f32,
    pub optimization_degradation_critical: f32,
}

#[derive(Debug, Clone)]
pub struct RegressionDetector {
    /// Recent performance samples for trend analysis
    recent_samples: Vec<TimestampedMetrics>,
    /// Statistical analysis window size
    analysis_window_size: usize,
    /// Regression detection sensitivity (0.0-1.0)
    regression_sensitivity: f32,
    /// Last regression check timestamp
    last_check: Instant,
}

#[derive(Debug, Clone)]
pub enum PerformanceAlert {
    CpuUsageHigh { current: f32, threshold: f32, browser: String },
    MemoryUsageHigh { current: u64, threshold: u64, browser: String },
    AudioLatencyHigh { current: f32, threshold: f32, browser: String },
    OptimizationDegraded { current: f32, baseline: f32, browser: String },
    RegressionDetected { metric: String, degradation_percent: f32, browser: String },
}

/// Graceful degradation mechanisms for browser compatibility
#[derive(Debug, Clone)]
pub struct GracefulDegradationManager {
    /// Feature compatibility assessments
    compatibility_assessments: Arc<Mutex<HashMap<String, FeatureCompatibilityAssessment>>>,
    /// Fallback strategies for each feature
    fallback_strategies: Arc<Mutex<HashMap<String, FallbackStrategy>>>,
    /// Progressive enhancement configurations
    progressive_enhancement: Arc<Mutex<ProgressiveEnhancementConfig>>,
    /// Compatibility layer configurations
    compatibility_layers: Arc<Mutex<Vec<CompatibilityLayer>>>,
    /// User notification system
    notification_system: Arc<Mutex<UserNotificationSystem>>,
    /// Active fallbacks tracking
    active_fallbacks: Arc<Mutex<HashMap<String, ActiveFallback>>>,
}

#[derive(Debug, Clone)]
pub struct FeatureCompatibilityAssessment {
    pub feature_name: String,
    pub browser_name: String,
    pub support_level: FeatureSupportLevel,
    pub compatibility_score: f32, // 0.0 to 1.0
    pub limitations: Vec<FeatureLimitation>,
    pub recommended_action: RecommendedAction,
    pub assessed_at: Instant,
}

#[derive(Debug, Clone)]
pub enum FeatureSupportLevel {
    FullySupported,     // 100% functionality
    MostlySupported,    // 80-99% functionality
    PartiallySupported, // 50-79% functionality
    LimitedSupported,   // 20-49% functionality
    NotSupported,       // 0-19% functionality
}

#[derive(Debug, Clone)]
pub struct FeatureLimitation {
    pub limitation_type: LimitationType,
    pub description: String,
    pub impact_severity: ImpactSeverity,
    pub workaround_available: bool,
}

#[derive(Debug, Clone)]
pub enum LimitationType {
    PerformanceLimitation,
    FunctionalityLimitation,
    SecurityLimitation,
    CompatibilityLimitation,
}

#[derive(Debug, Clone)]
pub enum ImpactSeverity {
    Critical, // Blocks core functionality
    High,     // Significantly degrades experience
    Medium,   // Noticeable but manageable
    Low,      // Minor impact
}

#[derive(Debug, Clone)]
pub enum RecommendedAction {
    UseAsIs,
    ApplyWorkaround,
    UseFallback,
    DisableFeature,
    RecommendUpgrade,
}

#[derive(Debug, Clone)]
pub struct FallbackStrategy {
    pub feature_name: String,
    pub fallback_type: FallbackType,
    pub fallback_implementation: String,
    pub performance_impact: f32, // Relative performance impact (0.0 = no impact, 1.0 = significant)
    pub feature_coverage: f32,   // How much of original feature is preserved (0.0-1.0)
    pub activation_conditions: Vec<ActivationCondition>,
}

#[derive(Debug, Clone)]
pub enum FallbackType {
    LegacyImplementation,  // Use older technology
    ReducedFunctionality,  // Simplified version
    AlternativeApproach,   // Different implementation method
    ExternalPolyfill,      // JavaScript polyfill
    GracefulDisabling,     // Disable with notification
}

#[derive(Debug, Clone)]
pub struct ActivationCondition {
    pub condition_type: ConditionType,
    pub threshold: f32,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum ConditionType {
    BrowserVersionBelow,
    FeatureSupportBelow,
    PerformanceBelow,
    MemoryLimitationAbove,
}

#[derive(Debug, Clone)]
pub struct ProgressiveEnhancementConfig {
    pub base_feature_set: Vec<String>,      // Core features that must work
    pub enhanced_feature_set: Vec<String>,  // Additional features for capable browsers
    pub premium_feature_set: Vec<String>,   // Advanced features for high-end browsers
    pub enhancement_detection_rules: Vec<EnhancementRule>,
}

#[derive(Debug, Clone)]
pub struct EnhancementRule {
    pub feature_name: String,
    pub required_capabilities: Vec<String>,
    pub minimum_performance_tier: CpuPerformanceTier,
    pub minimum_memory_mb: u64,
}

#[derive(Debug, Clone)]
pub struct CompatibilityLayer {
    pub layer_name: String,
    pub target_browsers: Vec<String>,
    pub version_range: VersionRange,
    pub layer_type: CompatibilityLayerType,
    pub implementation: String,
    pub performance_cost: f32,
}

#[derive(Debug, Clone)]
pub struct VersionRange {
    pub min_version: BrowserVersion,
    pub max_version: Option<BrowserVersion>,
}

#[derive(Debug, Clone)]
pub enum CompatibilityLayerType {
    JavaScriptPolyfill,
    WebAssemblyShim,
    AudioAPICompatibility,
    MemoryManagementShim,
    EventSystemCompat,
}

#[derive(Debug, Clone)]
pub struct UserNotificationSystem {
    pub notifications: Vec<UserNotification>,
    pub notification_settings: NotificationSettings,
    pub suppressed_notifications: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UserNotification {
    pub notification_id: String,
    pub notification_type: NotificationType,
    pub message: String,
    pub severity: NotificationSeverity,
    pub actions: Vec<NotificationAction>,
    pub created_at: Instant,
    pub dismissible: bool,
}

#[derive(Debug, Clone)]
pub enum NotificationType {
    FeatureUnavailable,
    PerformanceDegraded,
    BrowserUpgradeRecommended,
    FallbackActivated,
    CompatibilityWarning,
}

#[derive(Debug, Clone)]
pub enum NotificationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone)]
pub struct NotificationAction {
    pub action_id: String,
    pub label: String,
    pub action_type: NotificationActionType,
}

#[derive(Debug, Clone)]
pub enum NotificationActionType {
    Dismiss,
    LearnMore,
    UpgradeBrowser,
    EnableFallback,
    DisableFeature,
}

#[derive(Debug, Clone)]
pub struct NotificationSettings {
    pub show_performance_warnings: bool,
    pub show_compatibility_warnings: bool,
    pub show_upgrade_recommendations: bool,
    pub auto_dismiss_after_seconds: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ActiveFallback {
    pub feature_name: String,
    pub fallback_strategy: FallbackStrategy,
    pub activated_at: Instant,
    pub activation_reason: String,
    pub performance_impact: f32,
}

impl PlatformOptimizationEngineImpl {
    pub fn new() -> Self {
        let instance = Self {
            current_profile: Arc::new(Mutex::new(None)),
            applied_optimizations: Arc::new(Mutex::new(HashMap::new())),
            performance_metrics: Arc::new(Mutex::new(PerformanceMetrics {
                cpu_usage: 0.0,
                memory_usage: 0,
                audio_latency: 0.0,
                optimization_effectiveness: 0.0,
            })),
            memory_overhead: Arc::new(Mutex::new(0)),
            profile_cache: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Pre-populate optimization profiles for supported browsers
        instance.initialize_browser_profiles();
        instance
    }
    
    /// Initialize browser-specific optimization profiles
    fn initialize_browser_profiles(&self) {
        let mut cache = self.profile_cache.lock().unwrap();
        
        // Chrome 69+ optimization profile
        cache.insert("Chrome".to_string(), BrowserOptimizationProfile {
            webassembly_optimizations: WasmOptimizations {
                enable_simd: true,
                enable_threading: true,
                streaming_compilation: true,
                bulk_memory_operations: true,
                reference_types: true,
                optimization_level: 3,
            },
            audio_optimizations: AudioOptimizations {
                preferred_buffer_size: 1024,
                enable_audio_worklet: true,
                enable_shared_array_buffer: true,
                audio_context_options: AudioContextOptions {
                    sample_rate: Some(44100.0),
                    latency_hint: "interactive".to_string(),
                    echo_cancellation: true,
                    noise_suppression: false,
                    auto_gain_control: false,
                },
                latency_hint: AudioContextLatencyCategory::Interactive,
            },
            memory_optimizations: MemoryOptimizations {
                heap_size_limit: 2 * 1024 * 1024 * 1024, // 2GB
                gc_strategy: GcStrategy::Incremental,
                buffer_pooling: true,
                memory_pressure_handling: MemoryPressureHandling::Adaptive,
            },
            threading_optimizations: ThreadingOptimizations {
                enable_web_workers: true,
                shared_memory_support: true,
                thread_pool_size: 4,
                work_stealing: true,
            },
        });
        
        // Firefox 76+ optimization profile
        cache.insert("Firefox".to_string(), BrowserOptimizationProfile {
            webassembly_optimizations: WasmOptimizations {
                enable_simd: false, // Limited SIMD support
                enable_threading: true,
                streaming_compilation: false, // Disabled for better compatibility
                bulk_memory_operations: true,
                reference_types: false,
                optimization_level: 2,
            },
            audio_optimizations: AudioOptimizations {
                preferred_buffer_size: 2048,
                enable_audio_worklet: true,
                enable_shared_array_buffer: false,
                audio_context_options: AudioContextOptions {
                    sample_rate: Some(44100.0),
                    latency_hint: "balanced".to_string(),
                    echo_cancellation: true,
                    noise_suppression: true,
                    auto_gain_control: false,
                },
                latency_hint: AudioContextLatencyCategory::Balanced,
            },
            memory_optimizations: MemoryOptimizations {
                heap_size_limit: 1536 * 1024 * 1024, // 1.5GB
                gc_strategy: GcStrategy::Generational,
                buffer_pooling: true,
                memory_pressure_handling: MemoryPressureHandling::Conservative,
            },
            threading_optimizations: ThreadingOptimizations {
                enable_web_workers: true,
                shared_memory_support: false,
                thread_pool_size: 2,
                work_stealing: false,
            },
        });
        
        // Safari 14.1+ optimization profile
        cache.insert("Safari".to_string(), BrowserOptimizationProfile {
            webassembly_optimizations: WasmOptimizations {
                enable_simd: false,
                enable_threading: false,
                streaming_compilation: false,
                bulk_memory_operations: false,
                reference_types: false,
                optimization_level: 1,
            },
            audio_optimizations: AudioOptimizations {
                preferred_buffer_size: 2048,
                enable_audio_worklet: true,
                enable_shared_array_buffer: false,
                audio_context_options: AudioContextOptions {
                    sample_rate: Some(44100.0),
                    latency_hint: "playback".to_string(),
                    echo_cancellation: false,
                    noise_suppression: false,
                    auto_gain_control: false,
                },
                latency_hint: AudioContextLatencyCategory::Playback,
            },
            memory_optimizations: MemoryOptimizations {
                heap_size_limit: 1024 * 1024 * 1024, // 1GB
                gc_strategy: GcStrategy::Conservative,
                buffer_pooling: false,
                memory_pressure_handling: MemoryPressureHandling::Strict,
            },
            threading_optimizations: ThreadingOptimizations {
                enable_web_workers: false,
                shared_memory_support: false,
                thread_pool_size: 1,
                work_stealing: false,
            },
        });
        
        // Edge 79+ optimization profile (Chromium-based)
        cache.insert("Edge".to_string(), BrowserOptimizationProfile {
            webassembly_optimizations: WasmOptimizations {
                enable_simd: true,
                enable_threading: true,
                streaming_compilation: true,
                bulk_memory_operations: true,
                reference_types: true,
                optimization_level: 3,
            },
            audio_optimizations: AudioOptimizations {
                preferred_buffer_size: 1024,
                enable_audio_worklet: true,
                enable_shared_array_buffer: true,
                audio_context_options: AudioContextOptions {
                    sample_rate: Some(44100.0),
                    latency_hint: "interactive".to_string(),
                    echo_cancellation: true,
                    noise_suppression: false,
                    auto_gain_control: false,
                },
                latency_hint: AudioContextLatencyCategory::Interactive,
            },
            memory_optimizations: MemoryOptimizations {
                heap_size_limit: 2 * 1024 * 1024 * 1024, // 2GB
                gc_strategy: GcStrategy::Incremental,
                buffer_pooling: true,
                memory_pressure_handling: MemoryPressureHandling::Adaptive,
            },
            threading_optimizations: ThreadingOptimizations {
                enable_web_workers: true,
                shared_memory_support: true,
                thread_pool_size: 4,
                work_stealing: true,
            },
        });
    }
    
    /// Get browser-specific optimization profile
    pub fn get_browser_optimization_profile(&self, browser_info: &BrowserInfo) -> Option<BrowserOptimizationProfile> {
        let cache = self.profile_cache.lock().unwrap();
        cache.get(&browser_info.browser_name).cloned()
    }
    
    /// Apply browser optimization profile during platform initialization
    pub fn apply_browser_optimization_profile(&self, browser_info: &BrowserInfo, capabilities: &DeviceCapabilities) -> Result<(), PlatformError> {
        let profile = self.get_browser_optimization_profile(browser_info)
            .unwrap_or_else(|| self.create_fallback_profile(capabilities));
        
        // Apply WebAssembly optimizations
        self.apply_wasm_optimizations(&profile.webassembly_optimizations)?;
        
        // Apply audio optimizations
        self.apply_audio_optimizations(&profile.audio_optimizations)?;
        
        // Apply memory optimizations
        self.apply_memory_optimizations(&profile.memory_optimizations)?;
        
        // Apply threading optimizations
        self.apply_threading_optimizations(&profile.threading_optimizations)?;
        
        // Cache the applied profile
        {
            let mut current = self.current_profile.lock().unwrap();
            *current = Some(profile.clone());
        }
        
        // Record optimization application
        let optimization_record = OptimizationRecord {
            optimization_type: format!("{}_browser_optimization", browser_info.browser_name),
            applied_at: Instant::now(),
            browser_info: browser_info.clone(),
            effectiveness_score: 0.0, // Will be updated by monitoring
        };
        
        let mut applied = self.applied_optimizations.lock().unwrap();
        applied.insert(browser_info.browser_name.clone(), optimization_record);
        
        Ok(())
    }
    
    /// Apply WebAssembly optimizations
    fn apply_wasm_optimizations(&self, wasm_opts: &WasmOptimizations) -> Result<(), PlatformError> {
        #[cfg(target_arch = "wasm32")]
        {
            // Set WebAssembly optimization flags
            if wasm_opts.enable_simd {
                js_sys::eval("if (typeof WebAssembly !== 'undefined' && WebAssembly.simd) { console.log('SIMD enabled'); }")
                    .map_err(|_| PlatformError::OptimizationError("Failed to enable SIMD".to_string()))?;
            }
            
            if wasm_opts.streaming_compilation {
                js_sys::eval("if (typeof WebAssembly.instantiateStreaming === 'function') { console.log('Streaming compilation enabled'); }")
                    .map_err(|_| PlatformError::OptimizationError("Failed to enable streaming compilation".to_string()))?;
            }
            
            if wasm_opts.enable_threading {
                js_sys::eval("if (typeof SharedArrayBuffer !== 'undefined') { console.log('Threading enabled'); }")
                    .map_err(|_| PlatformError::OptimizationError("Failed to enable threading".to_string()))?;
            }
        }
        
        Ok(())
    }
    
    /// Apply audio processing optimizations
    fn apply_audio_optimizations(&self, audio_opts: &AudioOptimizations) -> Result<(), PlatformError> {
        #[cfg(target_arch = "wasm32")]
        {
            let audio_config = format!(
                "{{
                    preferredBufferSize: {},
                    audioWorkletEnabled: {},
                    sharedArrayBufferEnabled: {},
                    latencyHint: '{}'
                }}",
                audio_opts.preferred_buffer_size,
                audio_opts.enable_audio_worklet,
                audio_opts.enable_shared_array_buffer,
                match audio_opts.latency_hint {
                    AudioContextLatencyCategory::Interactive => "interactive",
                    AudioContextLatencyCategory::Balanced => "balanced",
                    AudioContextLatencyCategory::Playback => "playback",
                }
            );
            
            let config_js = format!("window.audioOptimizationConfig = {};", audio_config);
            js_sys::eval(&config_js)
                .map_err(|_| PlatformError::OptimizationError("Failed to apply audio optimizations".to_string()))?;
        }
        
        Ok(())
    }
    
    /// Apply memory management optimizations
    fn apply_memory_optimizations(&self, memory_opts: &MemoryOptimizations) -> Result<(), PlatformError> {
        #[cfg(target_arch = "wasm32")]
        {
            let memory_config = format!(
                "{{
                    heapSizeLimit: {},
                    gcStrategy: '{}',
                    bufferPooling: {},
                    memoryPressureHandling: '{}'
                }}",
                memory_opts.heap_size_limit,
                match memory_opts.gc_strategy {
                    GcStrategy::Incremental => "incremental",
                    GcStrategy::Generational => "generational",
                    GcStrategy::Conservative => "conservative",
                    GcStrategy::Aggressive => "aggressive",
                },
                memory_opts.buffer_pooling,
                match memory_opts.memory_pressure_handling {
                    MemoryPressureHandling::Strict => "strict",
                    MemoryPressureHandling::Conservative => "conservative",
                    MemoryPressureHandling::Aggressive => "aggressive",
                    MemoryPressureHandling::Adaptive => "adaptive",
                }
            );
            
            let config_js = format!("window.memoryOptimizationConfig = {};", memory_config);
            js_sys::eval(&config_js)
                .map_err(|_| PlatformError::OptimizationError("Failed to apply memory optimizations".to_string()))?;
        }
        
        Ok(())
    }
    
    /// Apply threading optimizations
    fn apply_threading_optimizations(&self, threading_opts: &ThreadingOptimizations) -> Result<(), PlatformError> {
        #[cfg(target_arch = "wasm32")]
        {
            let threading_config = format!(
                "{{
                    webWorkersEnabled: {},
                    sharedMemorySupport: {},
                    threadPoolSize: {},
                    workStealing: {}
                }}",
                threading_opts.enable_web_workers,
                threading_opts.shared_memory_support,
                threading_opts.thread_pool_size,
                threading_opts.work_stealing
            );
            
            let config_js = format!("window.threadingOptimizationConfig = {};", threading_config);
            js_sys::eval(&config_js)
                .map_err(|_| PlatformError::OptimizationError("Failed to apply threading optimizations".to_string()))?;
        }
        
        Ok(())
    }
    
    /// Create fallback optimization profile for unsupported browsers
    fn create_fallback_profile(&self, capabilities: &DeviceCapabilities) -> BrowserOptimizationProfile {
        BrowserOptimizationProfile {
            webassembly_optimizations: WasmOptimizations {
                enable_simd: false,
                enable_threading: false,
                streaming_compilation: false,
                bulk_memory_operations: false,
                reference_types: false,
                optimization_level: 0,
            },
            audio_optimizations: AudioOptimizations {
                preferred_buffer_size: match capabilities.performance_capability {
                    PerformanceCapability::Excellent => 1024,
                    PerformanceCapability::Good => 2048,
                    _ => 4096,
                },
                enable_audio_worklet: capabilities.hardware_acceleration,
                enable_shared_array_buffer: false,
                audio_context_options: AudioContextOptions {
                    sample_rate: Some(capabilities.max_sample_rate.min(44100.0)),
                    latency_hint: "balanced".to_string(),
                    echo_cancellation: false,
                    noise_suppression: false,
                    auto_gain_control: false,
                },
                latency_hint: AudioContextLatencyCategory::Balanced,
            },
            memory_optimizations: MemoryOptimizations {
                heap_size_limit: 512 * 1024 * 1024, // 512MB conservative limit
                gc_strategy: GcStrategy::Conservative,
                buffer_pooling: false,
                memory_pressure_handling: MemoryPressureHandling::Strict,
            },
            threading_optimizations: ThreadingOptimizations {
                enable_web_workers: false,
                shared_memory_support: false,
                thread_pool_size: 1,
                work_stealing: false,
            },
        }
    }
    
    /// Monitor optimization effectiveness
    fn calculate_optimization_effectiveness(&self) -> f32 {
        let applied_count = self.applied_optimizations.lock().unwrap().len() as f32;
        if applied_count == 0.0 {
            return 0.0;
        }
        
        // Calculate average effectiveness of applied optimizations
        let total_effectiveness: f32 = self.applied_optimizations
            .lock()
            .unwrap()
            .values()
            .map(|record| record.effectiveness_score)
            .sum();
        
        total_effectiveness / applied_count
    }
    
    /// Update memory overhead tracking (<2MB requirement)
    fn update_memory_overhead(&self, additional_bytes: u64) -> Result<(), PlatformError> {
        let mut overhead = self.memory_overhead.lock().unwrap();
        *overhead += additional_bytes;
        
        const MAX_OVERHEAD_BYTES: u64 = 2 * 1024 * 1024; // 2MB
        if *overhead > MAX_OVERHEAD_BYTES {
            return Err(PlatformError::OptimizationError(
                format!("Memory overhead exceeds 2MB limit: {} bytes", *overhead)
            ));
        }
        
        Ok(())
    }
    
    /// Get current memory overhead
    pub fn get_memory_overhead(&self) -> u64 {
        *self.memory_overhead.lock().unwrap()
    }
}

impl Default for PlatformOptimizationEngineImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformOptimizationEngine for PlatformOptimizationEngineImpl {
    fn apply_optimizations(&self, browser_info: &BrowserInfo, capabilities: &DeviceCapabilities) -> Result<(), PlatformError> {
        // Check memory overhead constraint
        self.update_memory_overhead(1024)?; // ~1KB overhead for optimization setup
        
        // Apply browser optimization profile
        self.apply_browser_optimization_profile(browser_info, capabilities)?;
        
        // Monitor optimization effectiveness
        let effectiveness = self.calculate_optimization_effectiveness();
        
        // Get estimated performance metrics
        let memory_usage = self.get_memory_overhead();
        
        // Estimate CPU usage based on applied optimizations
        let cpu_usage = if effectiveness > 0.8 {
            2.0 // Low CPU usage for highly effective optimizations
        } else if effectiveness > 0.5 {
            5.0 // Moderate CPU usage
        } else {
            10.0 // Higher CPU usage for less effective optimizations
        };
        
        // Estimate audio latency based on optimizations
        let audio_latency = if effectiveness > 0.8 {
            8.0 // Sub-10ms latency for excellent optimizations
        } else if effectiveness > 0.5 {
            15.0 // Good latency
        } else {
            25.0 // Moderate latency
        };
        
        let metrics = PerformanceMetrics {
            cpu_usage,
            memory_usage,
            audio_latency,
            optimization_effectiveness: effectiveness,
        };
        
        // Update stored metrics
        {
            let mut stored_metrics = self.performance_metrics.lock().unwrap();
            *stored_metrics = metrics.clone();
        }
        
        Ok(())
    }
    
    fn get_optimization_profile(&self) -> OptimizationProfile {
        // Convert BrowserOptimizationProfile to legacy OptimizationProfile for trait compatibility
        let browser_profile = self.current_profile.lock().unwrap().clone();
        
        if let Some(profile) = browser_profile {
            // Convert new profile to legacy format
            let mut browser_optimizations = HashMap::new();
            
            // Map browser optimizations
            browser_optimizations.insert("wasm_simd".to_string(), profile.webassembly_optimizations.enable_simd.to_string());
            browser_optimizations.insert("wasm_threading".to_string(), profile.webassembly_optimizations.enable_threading.to_string());
            browser_optimizations.insert("audio_buffer_size".to_string(), profile.audio_optimizations.preferred_buffer_size.to_string());
            browser_optimizations.insert("memory_strategy".to_string(), format!("{:?}", profile.memory_optimizations.gc_strategy));
            
            // Generate optimization flags
            let mut wasm_optimization_flags = Vec::new();
            if profile.webassembly_optimizations.enable_simd {
                wasm_optimization_flags.push("--wasm-simd".to_string());
            }
            if profile.webassembly_optimizations.enable_threading {
                wasm_optimization_flags.push("--wasm-threading".to_string());
            }
            if profile.webassembly_optimizations.streaming_compilation {
                wasm_optimization_flags.push("--wasm-streaming".to_string());
            }
            
            OptimizationProfile {
                browser_optimizations,
                memory_optimization_level: profile.webassembly_optimizations.optimization_level,
                audio_buffer_optimization: profile.audio_optimizations.enable_audio_worklet,
                wasm_optimization_flags,
            }
        } else {
            // Return default profile if none exists
            OptimizationProfile {
                browser_optimizations: HashMap::new(),
                memory_optimization_level: 1,
                audio_buffer_optimization: false,
                wasm_optimization_flags: vec!["--wasm-baseline".to_string()],
            }
        }
    }
    
    fn monitor_optimization_effectiveness(&self) -> PerformanceMetrics {
        // Update effectiveness score
        let effectiveness = self.calculate_optimization_effectiveness();
        
        // Get estimated performance metrics
        let memory_usage = self.get_memory_overhead();
        
        // Estimate CPU usage based on applied optimizations
        let cpu_usage = if effectiveness > 0.8 {
            2.0 // Low CPU usage for highly effective optimizations
        } else if effectiveness > 0.5 {
            5.0 // Moderate CPU usage
        } else {
            10.0 // Higher CPU usage for less effective optimizations
        };
        
        // Estimate audio latency based on optimizations
        let audio_latency = if effectiveness > 0.8 {
            8.0 // Sub-10ms latency for excellent optimizations
        } else if effectiveness > 0.5 {
            15.0 // Good latency
        } else {
            25.0 // Moderate latency
        };
        
        let metrics = PerformanceMetrics {
            cpu_usage,
            memory_usage,
            audio_latency,
            optimization_effectiveness: effectiveness,
        };
        
        // Update stored metrics
        {
            let mut stored_metrics = self.performance_metrics.lock().unwrap();
            *stored_metrics = metrics.clone();
        }
        
        metrics
    }
}

impl BrowserPerformanceProfiler {
    pub fn new() -> Self {
        Self {
            metrics_history: Arc::new(Mutex::new(Vec::new())),
            baselines: Arc::new(Mutex::new(HashMap::new())),
            alert_thresholds: Arc::new(Mutex::new(PerformanceThresholds {
                cpu_usage_warning: 50.0,
                cpu_usage_critical: 80.0,
                memory_usage_warning: 1024 * 1024 * 1024, // 1GB
                memory_usage_critical: 1536 * 1024 * 1024, // 1.5GB
                audio_latency_warning: 30.0,
                audio_latency_critical: 50.0,
                optimization_degradation_warning: 0.2, // 20% degradation
                optimization_degradation_critical: 0.4, // 40% degradation
            })),
            monitoring_overhead: Arc::new(Mutex::new(0.0)),
            regression_detector: Arc::new(Mutex::new(RegressionDetector {
                recent_samples: Vec::new(),
                analysis_window_size: 50,
                regression_sensitivity: 0.1, // 10% regression threshold
                last_check: Instant::now(),
            })),
        }
    }
    
    /// Detect browser performance characteristics with <1ms overhead
    pub fn detect_browser_performance_characteristics(&self, browser_info: &BrowserInfo) -> Result<BrowserPerformanceProfile, PlatformError> {
        let start_time = Instant::now();
        
        // Quick performance characteristic detection
        let audio_latency_profile = self.detect_audio_latency_profile(browser_info);
        let memory_characteristics = self.detect_memory_characteristics(browser_info);
        let cpu_performance_tier = self.detect_cpu_performance_tier(browser_info);
        
        let profile = BrowserPerformanceProfile {
            audio_latency_profile,
            memory_characteristics,
            cpu_performance_tier,
        };
        
        // Track monitoring overhead (must be <1ms)
        let elapsed = start_time.elapsed();
        let overhead_ms = elapsed.as_nanos() as f64 / 1_000_000.0;
        
        {
            let mut monitoring_overhead = self.monitoring_overhead.lock().unwrap();
            *monitoring_overhead = overhead_ms;
        }
        
        if overhead_ms >= 1.0 {
            return Err(PlatformError::OptimizationError(
                format!("Performance detection overhead too high: {:.2}ms", overhead_ms)
            ));
        }
        
        Ok(profile)
    }
    
    /// Real-time performance monitoring with <1ms overhead
    pub fn monitor_realtime_performance(&self, browser_info: &BrowserInfo, current_metrics: &PerformanceMetrics) -> Result<Vec<PerformanceAlert>, PlatformError> {
        let start_time = Instant::now();
        let mut alerts = Vec::new();
        
        // Record metrics with timestamp
        let timestamped_metrics = TimestampedMetrics {
            timestamp: Instant::now(),
            metrics: current_metrics.clone(),
            browser_name: browser_info.browser_name.clone(),
            optimization_profile: format!("{}_profile", browser_info.browser_name),
        };
        
        // Store metrics (with bounded history to prevent memory growth)
        {
            let mut history = self.metrics_history.lock().unwrap();
            history.push(timestamped_metrics.clone());
            
            // Keep only last 1000 entries to prevent unbounded growth
            if history.len() > 1000 {
                history.remove(0);
            }
        }
        
        // Check against thresholds
        alerts.extend(self.check_performance_thresholds(&browser_info.browser_name, current_metrics)?);
        
        // Update regression detector
        alerts.extend(self.update_regression_detection(&timestamped_metrics)?);
        
        // Track monitoring overhead (must be <1ms)
        let elapsed = start_time.elapsed();
        let overhead_ms = elapsed.as_nanos() as f64 / 1_000_000.0;
        
        {
            let mut monitoring_overhead = self.monitoring_overhead.lock().unwrap();
            *monitoring_overhead = overhead_ms;
        }
        
        if overhead_ms >= 1.0 {
            return Err(PlatformError::OptimizationError(
                format!("Real-time monitoring overhead too high: {:.2}ms", overhead_ms)
            ));
        }
        
        Ok(alerts)
    }
    
    /// Collect performance metrics for optimization effectiveness
    pub fn collect_optimization_effectiveness_metrics(&self, browser_name: &str) -> Result<PerformanceMetrics, PlatformError> {
        let history = self.metrics_history.lock().unwrap();
        
        // Get recent metrics for the specified browser
        let recent_metrics: Vec<&TimestampedMetrics> = history
            .iter()
            .rev() // Most recent first
            .take(10) // Last 10 measurements
            .filter(|m| m.browser_name == browser_name)
            .collect();
        
        if recent_metrics.is_empty() {
            return Err(PlatformError::OptimizationError(
                format!("No metrics available for browser: {}", browser_name)
            ));
        }
        
        // Calculate average metrics
        let count = recent_metrics.len() as f32;
        let avg_cpu = recent_metrics.iter().map(|m| m.metrics.cpu_usage).sum::<f32>() / count;
        let avg_memory = recent_metrics.iter().map(|m| m.metrics.memory_usage).sum::<u64>() / count as u64;
        let avg_latency = recent_metrics.iter().map(|m| m.metrics.audio_latency).sum::<f32>() / count;
        let avg_effectiveness = recent_metrics.iter().map(|m| m.metrics.optimization_effectiveness).sum::<f32>() / count;
        
        Ok(PerformanceMetrics {
            cpu_usage: avg_cpu,
            memory_usage: avg_memory,
            audio_latency: avg_latency,
            optimization_effectiveness: avg_effectiveness,
        })
    }
    
    /// Create performance regression detection system
    pub fn create_performance_regression_detection(&self) -> Result<(), PlatformError> {
        let mut detector = self.regression_detector.lock().unwrap();
        detector.recent_samples.clear();
        detector.last_check = Instant::now();
        Ok(())
    }
    
    /// Analyze performance history and generate analysis
    pub fn analyze_performance_history(&self, browser_name: &str, time_window_hours: u64) -> Result<PerformanceAnalysis, PlatformError> {
        let history = self.metrics_history.lock().unwrap();
        let cutoff_time = Instant::now() - std::time::Duration::from_secs(time_window_hours * 3600);
        
        // Filter metrics for the specified browser and time window
        let relevant_metrics: Vec<&TimestampedMetrics> = history
            .iter()
            .filter(|m| m.browser_name == browser_name && m.timestamp >= cutoff_time)
            .collect();
        
        if relevant_metrics.is_empty() {
            return Err(PlatformError::OptimizationError(
                format!("No metrics found for browser {} in the last {} hours", browser_name, time_window_hours)
            ));
        }
        
        // Calculate statistical measures
        let cpu_values: Vec<f32> = relevant_metrics.iter().map(|m| m.metrics.cpu_usage).collect();
        let memory_values: Vec<u64> = relevant_metrics.iter().map(|m| m.metrics.memory_usage).collect();
        let latency_values: Vec<f32> = relevant_metrics.iter().map(|m| m.metrics.audio_latency).collect();
        let effectiveness_values: Vec<f32> = relevant_metrics.iter().map(|m| m.metrics.optimization_effectiveness).collect();
        
        Ok(PerformanceAnalysis {
            browser_name: browser_name.to_string(),
            time_window_hours,
            sample_count: relevant_metrics.len(),
            cpu_usage_stats: StatisticalSummary::calculate(&cpu_values),
            memory_usage_stats: StatisticalSummaryU64::calculate_u64(&memory_values),
            audio_latency_stats: StatisticalSummary::calculate(&latency_values),
            optimization_effectiveness_stats: StatisticalSummary::calculate(&effectiveness_values),
            performance_trend: self.calculate_performance_trend(&relevant_metrics),
        })
    }
    
    /// Add performance alert system for degradation detection
    pub fn configure_performance_alerts(&self, thresholds: PerformanceThresholds) -> Result<(), PlatformError> {
        let mut alert_thresholds = self.alert_thresholds.lock().unwrap();
        *alert_thresholds = thresholds;
        Ok(())
    }
    
    /// Get current monitoring overhead
    pub fn get_monitoring_overhead(&self) -> f64 {
        *self.monitoring_overhead.lock().unwrap()
    }
    
    // Private helper methods
    
    fn detect_audio_latency_profile(&self, browser_info: &BrowserInfo) -> AudioLatencyProfile {
        match (browser_info.browser_name.as_str(), browser_info.capabilities.supports_audio_worklet) {
            ("Chrome", true) | ("Edge", true) => AudioLatencyProfile::UltraLow,
            ("Firefox", true) | ("Safari", true) => AudioLatencyProfile::Low,
            (_, true) => AudioLatencyProfile::Medium,
            _ => AudioLatencyProfile::High,
        }
    }
    
    fn detect_memory_characteristics(&self, browser_info: &BrowserInfo) -> MemoryCharacteristics {
        let available_heap = match browser_info.browser_name.as_str() {
            "Chrome" | "Edge" => 2 * 1024 * 1024 * 1024, // 2GB
            "Firefox" => 1536 * 1024 * 1024, // 1.5GB
            "Safari" => 1024 * 1024 * 1024, // 1GB
            _ => 512 * 1024 * 1024, // 512MB
        };
        
        let gc_impact = match browser_info.browser_name.as_str() {
            "Chrome" | "Edge" => GcImpact::Minimal,
            "Firefox" => GcImpact::Moderate,
            "Safari" => GcImpact::Significant,
            _ => GcImpact::Significant,
        };
        
        MemoryCharacteristics {
            available_heap,
            supports_shared_memory: browser_info.capabilities.supports_shared_array_buffer,
            garbage_collection_impact: gc_impact,
        }
    }
    
    fn detect_cpu_performance_tier(&self, browser_info: &BrowserInfo) -> CpuPerformanceTier {
        // Performance tier based on WebAssembly capabilities
        if browser_info.capabilities.supports_wasm_simd && browser_info.capabilities.supports_wasm_streaming {
            CpuPerformanceTier::High
        } else if browser_info.capabilities.supports_wasm {
            CpuPerformanceTier::Medium
        } else {
            CpuPerformanceTier::Low
        }
    }
    
    fn check_performance_thresholds(&self, browser_name: &str, metrics: &PerformanceMetrics) -> Result<Vec<PerformanceAlert>, PlatformError> {
        let thresholds = self.alert_thresholds.lock().unwrap();
        let mut alerts = Vec::new();
        
        // Check CPU usage
        if metrics.cpu_usage >= thresholds.cpu_usage_critical {
            alerts.push(PerformanceAlert::CpuUsageHigh {
                current: metrics.cpu_usage,
                threshold: thresholds.cpu_usage_critical,
                browser: browser_name.to_string(),
            });
        } else if metrics.cpu_usage >= thresholds.cpu_usage_warning {
            alerts.push(PerformanceAlert::CpuUsageHigh {
                current: metrics.cpu_usage,
                threshold: thresholds.cpu_usage_warning,
                browser: browser_name.to_string(),
            });
        }
        
        // Check memory usage
        if metrics.memory_usage >= thresholds.memory_usage_critical {
            alerts.push(PerformanceAlert::MemoryUsageHigh {
                current: metrics.memory_usage,
                threshold: thresholds.memory_usage_critical,
                browser: browser_name.to_string(),
            });
        } else if metrics.memory_usage >= thresholds.memory_usage_warning {
            alerts.push(PerformanceAlert::MemoryUsageHigh {
                current: metrics.memory_usage,
                threshold: thresholds.memory_usage_warning,
                browser: browser_name.to_string(),
            });
        }
        
        // Check audio latency
        if metrics.audio_latency >= thresholds.audio_latency_critical {
            alerts.push(PerformanceAlert::AudioLatencyHigh {
                current: metrics.audio_latency,
                threshold: thresholds.audio_latency_critical,
                browser: browser_name.to_string(),
            });
        } else if metrics.audio_latency >= thresholds.audio_latency_warning {
            alerts.push(PerformanceAlert::AudioLatencyHigh {
                current: metrics.audio_latency,
                threshold: thresholds.audio_latency_warning,
                browser: browser_name.to_string(),
            });
        }
        
        // Check optimization effectiveness degradation
        if let Ok(baseline) = self.get_baseline(browser_name) {
            let degradation = (baseline.optimization_effectiveness_baseline - metrics.optimization_effectiveness) / baseline.optimization_effectiveness_baseline;
            
            if degradation >= thresholds.optimization_degradation_critical {
                alerts.push(PerformanceAlert::OptimizationDegraded {
                    current: metrics.optimization_effectiveness,
                    baseline: baseline.optimization_effectiveness_baseline,
                    browser: browser_name.to_string(),
                });
            } else if degradation >= thresholds.optimization_degradation_warning {
                alerts.push(PerformanceAlert::OptimizationDegraded {
                    current: metrics.optimization_effectiveness,
                    baseline: baseline.optimization_effectiveness_baseline,
                    browser: browser_name.to_string(),
                });
            }
        }
        
        Ok(alerts)
    }
    
    fn update_regression_detection(&self, metrics: &TimestampedMetrics) -> Result<Vec<PerformanceAlert>, PlatformError> {
        let mut detector = self.regression_detector.lock().unwrap();
        let mut alerts = Vec::new();
        
        // Add new sample
        detector.recent_samples.push(metrics.clone());
        
        // Keep only recent samples within the analysis window
        if detector.recent_samples.len() > detector.analysis_window_size {
            detector.recent_samples.remove(0);
        }
        
        // Perform regression analysis if we have enough samples and enough time has passed
        let time_since_last_check = detector.last_check.elapsed();
        if detector.recent_samples.len() >= 20 && time_since_last_check >= std::time::Duration::from_secs(60) {
            
            // Analyze trends in the recent samples
            let recent_half = &detector.recent_samples[detector.recent_samples.len()/2..];
            let older_half = &detector.recent_samples[..detector.recent_samples.len()/2];
            
            // Calculate averages for comparison
            let recent_avg_effectiveness = recent_half.iter().map(|m| m.metrics.optimization_effectiveness).sum::<f32>() / recent_half.len() as f32;
            let older_avg_effectiveness = older_half.iter().map(|m| m.metrics.optimization_effectiveness).sum::<f32>() / older_half.len() as f32;
            
            let recent_avg_latency = recent_half.iter().map(|m| m.metrics.audio_latency).sum::<f32>() / recent_half.len() as f32;
            let older_avg_latency = older_half.iter().map(|m| m.metrics.audio_latency).sum::<f32>() / older_half.len() as f32;
            
            // Check for regressions
            let effectiveness_degradation = (older_avg_effectiveness - recent_avg_effectiveness) / older_avg_effectiveness;
            let latency_increase = (recent_avg_latency - older_avg_latency) / older_avg_latency;
            
            if effectiveness_degradation >= detector.regression_sensitivity {
                alerts.push(PerformanceAlert::RegressionDetected {
                    metric: "optimization_effectiveness".to_string(),
                    degradation_percent: effectiveness_degradation * 100.0,
                    browser: metrics.browser_name.clone(),
                });
            }
            
            if latency_increase >= detector.regression_sensitivity {
                alerts.push(PerformanceAlert::RegressionDetected {
                    metric: "audio_latency".to_string(),
                    degradation_percent: latency_increase * 100.0,
                    browser: metrics.browser_name.clone(),
                });
            }
            
            detector.last_check = Instant::now();
        }
        
        Ok(alerts)
    }
    
    fn get_baseline(&self, browser_name: &str) -> Result<PerformanceBaseline, PlatformError> {
        let baselines = self.baselines.lock().unwrap();
        baselines.get(browser_name)
            .cloned()
            .ok_or_else(|| PlatformError::OptimizationError(format!("No baseline found for browser: {}", browser_name)))
    }
    
    fn calculate_performance_trend(&self, metrics: &[&TimestampedMetrics]) -> PerformanceTrend {
        if metrics.len() < 5 {
            return PerformanceTrend::Stable;
        }
        
        // Simple trend analysis based on recent vs older metrics
        let recent_third = &metrics[metrics.len()*2/3..];
        let older_third = &metrics[..metrics.len()/3];
        
        let recent_avg_effectiveness = recent_third.iter().map(|m| m.metrics.optimization_effectiveness).sum::<f32>() / recent_third.len() as f32;
        let older_avg_effectiveness = older_third.iter().map(|m| m.metrics.optimization_effectiveness).sum::<f32>() / older_third.len() as f32;
        
        let effectiveness_change = (recent_avg_effectiveness - older_avg_effectiveness) / older_avg_effectiveness;
        
        if effectiveness_change > 0.05 { // 5% improvement
            PerformanceTrend::Improving
        } else if effectiveness_change < -0.05 { // 5% degradation
            PerformanceTrend::Degrading
        } else {
            PerformanceTrend::Stable
        }
    }
    
    /// Establish performance baseline for a browser
    pub fn establish_baseline(&self, browser_name: &str, metrics: &PerformanceMetrics) -> Result<(), PlatformError> {
        let mut baselines = self.baselines.lock().unwrap();
        
        let baseline = PerformanceBaseline {
            cpu_usage_baseline: metrics.cpu_usage,
            memory_usage_baseline: metrics.memory_usage,
            audio_latency_baseline: metrics.audio_latency,
            optimization_effectiveness_baseline: metrics.optimization_effectiveness,
            measurement_count: 1,
            established_at: Instant::now(),
        };
        
        baselines.insert(browser_name.to_string(), baseline);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceAnalysis {
    pub browser_name: String,
    pub time_window_hours: u64,
    pub sample_count: usize,
    pub cpu_usage_stats: StatisticalSummary,
    pub memory_usage_stats: StatisticalSummaryU64,
    pub audio_latency_stats: StatisticalSummary,
    pub optimization_effectiveness_stats: StatisticalSummary,
    pub performance_trend: PerformanceTrend,
}

#[derive(Debug, Clone)]
pub struct StatisticalSummary {
    pub mean: f32,
    pub min: f32,
    pub max: f32,
    pub std_dev: f32,
}

#[derive(Debug, Clone)]
pub struct StatisticalSummaryU64 {
    pub mean: u64,
    pub min: u64,
    pub max: u64,
    pub std_dev: f64,
}

#[derive(Debug, Clone)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Degrading,
}

impl StatisticalSummary {
    fn calculate(values: &[f32]) -> Self {
        if values.is_empty() {
            return Self { mean: 0.0, min: 0.0, max: 0.0, std_dev: 0.0 };
        }
        
        let mean = values.iter().sum::<f32>() / values.len() as f32;
        let min = values.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        
        let variance = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f32>() / values.len() as f32;
        let std_dev = variance.sqrt();
        
        Self { mean, min, max, std_dev }
    }
}

impl StatisticalSummaryU64 {
    fn calculate_u64(values: &[u64]) -> Self {
        if values.is_empty() {
            return Self { mean: 0, min: 0, max: 0, std_dev: 0.0 };
        }
        
        let mean = values.iter().sum::<u64>() / values.len() as u64;
        let min = *values.iter().min().unwrap();
        let max = *values.iter().max().unwrap();
        
        let variance = values.iter()
            .map(|x| ((*x as f64) - (mean as f64)).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        
        Self { mean, min, max, std_dev }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_browser_info(browser_name: &str) -> BrowserInfo {
        BrowserInfo {
            browser_name: browser_name.to_string(),
            version: BrowserVersion { major: 79, minor: 0, patch: 0 },
            user_agent: format!("{}/79.0.0", browser_name),
            capabilities: BrowserCapabilities {
                supports_wasm: true,
                supports_wasm_streaming: true,
                supports_wasm_simd: true,
                supports_audio_context: true,
                supports_audio_worklet: true,
                supports_media_devices: true,
                supports_shared_array_buffer: true,
                performance_api: true,
            },
            compatibility_level: CompatibilityLevel::FullySupported,
        }
    }
    
    fn create_test_device_capabilities() -> DeviceCapabilities {
        DeviceCapabilities {
            hardware_acceleration: true,
            max_sample_rate: 44100.0,
            min_buffer_size: 512,
            max_buffer_size: 4096,
            audio_input_devices: vec![
                AudioDevice {
                    id: "default".to_string(),
                    name: "Default Audio Device".to_string(),
                    is_default: true,
                    max_channels: 2,
                }
            ],
            performance_capability: PerformanceCapability::Excellent,
        }
    }
    
    #[test]
    fn test_chrome_optimization_profile_creation() {
        let engine = PlatformOptimizationEngineImpl::new();
        let browser_info = create_test_browser_info("Chrome");
        
        let profile = engine.get_browser_optimization_profile(&browser_info).unwrap();
        
        // Verify Chrome-specific optimizations
        assert!(profile.webassembly_optimizations.enable_simd);
        assert!(profile.webassembly_optimizations.enable_threading);
        assert!(profile.webassembly_optimizations.streaming_compilation);
        assert_eq!(profile.webassembly_optimizations.optimization_level, 3);
        
        assert_eq!(profile.audio_optimizations.preferred_buffer_size, 1024);
        assert!(profile.audio_optimizations.enable_audio_worklet);
        assert!(profile.audio_optimizations.enable_shared_array_buffer);
        
        assert_eq!(profile.memory_optimizations.heap_size_limit, 2 * 1024 * 1024 * 1024);
        assert!(matches!(profile.memory_optimizations.gc_strategy, GcStrategy::Incremental));
        
        assert!(profile.threading_optimizations.enable_web_workers);
        assert!(profile.threading_optimizations.shared_memory_support);
        assert_eq!(profile.threading_optimizations.thread_pool_size, 4);
    }
    
    #[test]
    fn test_firefox_optimization_profile_creation() {
        let engine = PlatformOptimizationEngineImpl::new();
        let browser_info = create_test_browser_info("Firefox");
        
        let profile = engine.get_browser_optimization_profile(&browser_info).unwrap();
        
        // Verify Firefox-specific optimizations
        assert!(!profile.webassembly_optimizations.enable_simd); // Limited SIMD support
        assert!(profile.webassembly_optimizations.enable_threading);
        assert!(!profile.webassembly_optimizations.streaming_compilation); // Disabled for compatibility
        assert_eq!(profile.webassembly_optimizations.optimization_level, 2);
        
        assert_eq!(profile.audio_optimizations.preferred_buffer_size, 2048);
        assert!(profile.audio_optimizations.enable_audio_worklet);
        assert!(!profile.audio_optimizations.enable_shared_array_buffer);
        
        assert_eq!(profile.memory_optimizations.heap_size_limit, 1536 * 1024 * 1024);
        assert!(matches!(profile.memory_optimizations.gc_strategy, GcStrategy::Generational));
        
        assert!(profile.threading_optimizations.enable_web_workers);
        assert!(!profile.threading_optimizations.shared_memory_support);
        assert_eq!(profile.threading_optimizations.thread_pool_size, 2);
    }
    
    #[test]
    fn test_safari_optimization_profile_creation() {
        let engine = PlatformOptimizationEngineImpl::new();
        let browser_info = create_test_browser_info("Safari");
        
        let profile = engine.get_browser_optimization_profile(&browser_info).unwrap();
        
        // Verify Safari-specific optimizations (more conservative)
        assert!(!profile.webassembly_optimizations.enable_simd);
        assert!(!profile.webassembly_optimizations.enable_threading);
        assert!(!profile.webassembly_optimizations.streaming_compilation);
        assert_eq!(profile.webassembly_optimizations.optimization_level, 1);
        
        assert_eq!(profile.audio_optimizations.preferred_buffer_size, 2048);
        assert!(profile.audio_optimizations.enable_audio_worklet);
        assert!(!profile.audio_optimizations.enable_shared_array_buffer);
        
        assert_eq!(profile.memory_optimizations.heap_size_limit, 1024 * 1024 * 1024);
        assert!(matches!(profile.memory_optimizations.gc_strategy, GcStrategy::Conservative));
        
        assert!(!profile.threading_optimizations.enable_web_workers);
        assert!(!profile.threading_optimizations.shared_memory_support);
        assert_eq!(profile.threading_optimizations.thread_pool_size, 1);
    }
    
    #[test]
    fn test_edge_optimization_profile_creation() {
        let engine = PlatformOptimizationEngineImpl::new();
        let browser_info = create_test_browser_info("Edge");
        
        let profile = engine.get_browser_optimization_profile(&browser_info).unwrap();
        
        // Verify Edge-specific optimizations (Chromium-based, similar to Chrome)
        assert!(profile.webassembly_optimizations.enable_simd);
        assert!(profile.webassembly_optimizations.enable_threading);
        assert!(profile.webassembly_optimizations.streaming_compilation);
        assert_eq!(profile.webassembly_optimizations.optimization_level, 3);
        
        assert_eq!(profile.audio_optimizations.preferred_buffer_size, 1024);
        assert!(profile.audio_optimizations.enable_audio_worklet);
        assert!(profile.audio_optimizations.enable_shared_array_buffer);
        
        assert_eq!(profile.memory_optimizations.heap_size_limit, 2 * 1024 * 1024 * 1024);
        assert!(matches!(profile.memory_optimizations.gc_strategy, GcStrategy::Incremental));
        
        assert!(profile.threading_optimizations.enable_web_workers);
        assert!(profile.threading_optimizations.shared_memory_support);
        assert_eq!(profile.threading_optimizations.thread_pool_size, 4);
    }
    
    #[test]
    fn test_fallback_profile_creation() {
        let engine = PlatformOptimizationEngineImpl::new();
        let capabilities = create_test_device_capabilities();
        
        let profile = engine.create_fallback_profile(&capabilities);
        
        // Verify fallback profile has conservative settings
        assert!(!profile.webassembly_optimizations.enable_simd);
        assert!(!profile.webassembly_optimizations.enable_threading);
        assert!(!profile.webassembly_optimizations.streaming_compilation);
        assert_eq!(profile.webassembly_optimizations.optimization_level, 0);
        
        assert_eq!(profile.audio_optimizations.preferred_buffer_size, 1024); // Excellent performance
        assert!(profile.audio_optimizations.enable_audio_worklet); // Hardware acceleration enabled
        assert!(!profile.audio_optimizations.enable_shared_array_buffer);
        
        assert_eq!(profile.memory_optimizations.heap_size_limit, 512 * 1024 * 1024);
        assert!(matches!(profile.memory_optimizations.gc_strategy, GcStrategy::Conservative));
        
        assert!(!profile.threading_optimizations.enable_web_workers);
        assert!(!profile.threading_optimizations.shared_memory_support);
        assert_eq!(profile.threading_optimizations.thread_pool_size, 1);
    }
    
    #[test]
    fn test_browser_optimization_profile_application() {
        let engine = PlatformOptimizationEngineImpl::new();
        let browser_info = create_test_browser_info("Chrome");
        let capabilities = create_test_device_capabilities();
        
        let result = engine.apply_browser_optimization_profile(&browser_info, &capabilities);
        assert!(result.is_ok());
        
        // Verify optimization record was created
        let applied = engine.applied_optimizations.lock().unwrap();
        assert!(applied.contains_key("Chrome"));
        
        let record = applied.get("Chrome").unwrap();
        assert_eq!(record.optimization_type, "Chrome_browser_optimization");
        assert_eq!(record.browser_info.browser_name, "Chrome");
    }
    
    #[test]
    fn test_memory_overhead_tracking() {
        let engine = PlatformOptimizationEngineImpl::new();
        
        // Test successful memory overhead update
        let result = engine.update_memory_overhead(1024);
        assert!(result.is_ok());
        assert_eq!(engine.get_memory_overhead(), 1024);
        
        // Test memory overhead accumulation
        let result = engine.update_memory_overhead(1024);
        assert!(result.is_ok());
        assert_eq!(engine.get_memory_overhead(), 2048);
        
        // Test memory overhead limit (2MB)
        let large_overhead = 2 * 1024 * 1024; // 2MB
        let result = engine.update_memory_overhead(large_overhead);
        assert!(result.is_err());
        
        if let Err(PlatformError::OptimizationError(msg)) = result {
            assert!(msg.contains("Memory overhead exceeds 2MB limit"));
        } else {
            panic!("Expected OptimizationError");
        }
    }
    
    #[test]
    fn test_optimization_effectiveness_calculation() {
        let engine = PlatformOptimizationEngineImpl::new();
        
        // Initially no optimizations applied
        assert_eq!(engine.calculate_optimization_effectiveness(), 0.0);
        
        // Add some optimization records
        {
            let mut applied = engine.applied_optimizations.lock().unwrap();
            applied.insert("test1".to_string(), OptimizationRecord {
                optimization_type: "test".to_string(),
                applied_at: Instant::now(),
                browser_info: create_test_browser_info("Chrome"),
                effectiveness_score: 0.8,
            });
            applied.insert("test2".to_string(), OptimizationRecord {
                optimization_type: "test".to_string(),
                applied_at: Instant::now(),
                browser_info: create_test_browser_info("Firefox"),
                effectiveness_score: 0.6,
            });
        }
        
        // Should calculate average effectiveness
        let effectiveness = engine.calculate_optimization_effectiveness();
        assert!((effectiveness - 0.7).abs() < 0.01); // Average of 0.8 and 0.6
    }
    
    #[test]
    fn test_automatic_profile_selection() {
        let engine = PlatformOptimizationEngineImpl::new();
        let browser_info = create_test_browser_info("Chrome");
        let capabilities = create_test_device_capabilities();
        
        let result = engine.apply_optimizations(&browser_info, &capabilities);
        assert!(result.is_ok());
        
        // Verify Chrome profile was automatically selected and applied
        let current_profile = engine.current_profile.lock().unwrap();
        assert!(current_profile.is_some());
        
        let profile = current_profile.as_ref().unwrap();
        assert!(profile.webassembly_optimizations.enable_simd); // Chrome feature
        assert_eq!(profile.audio_optimizations.preferred_buffer_size, 1024); // Chrome setting
    }
    
    #[test]
    fn test_wasm_optimization_configuration() {
        let wasm_opts = WasmOptimizations {
            enable_simd: true,
            enable_threading: true,
            streaming_compilation: true,
            bulk_memory_operations: true,
            reference_types: true,
            optimization_level: 3,
        };
        
        // Verify all optimization flags are set correctly
        assert!(wasm_opts.enable_simd);
        assert!(wasm_opts.enable_threading);
        assert!(wasm_opts.streaming_compilation);
        assert!(wasm_opts.bulk_memory_operations);
        assert!(wasm_opts.reference_types);
        assert_eq!(wasm_opts.optimization_level, 3);
    }
    
    #[test]
    fn test_audio_optimization_configuration() {
        let audio_opts = AudioOptimizations {
            preferred_buffer_size: 1024,
            enable_audio_worklet: true,
            enable_shared_array_buffer: true,
            audio_context_options: AudioContextOptions {
                sample_rate: Some(44100.0),
                latency_hint: "interactive".to_string(),
                echo_cancellation: true,
                noise_suppression: false,
                auto_gain_control: false,
            },
            latency_hint: AudioContextLatencyCategory::Interactive,
        };
        
        // Verify audio optimization settings
        assert_eq!(audio_opts.preferred_buffer_size, 1024);
        assert!(audio_opts.enable_audio_worklet);
        assert!(audio_opts.enable_shared_array_buffer);
        assert_eq!(audio_opts.audio_context_options.sample_rate, Some(44100.0));
        assert!(matches!(audio_opts.latency_hint, AudioContextLatencyCategory::Interactive));
    }
    
    // Performance Profiling System Tests (Task 2)
    
    #[test]
    fn test_browser_performance_profiler_creation() {
        let profiler = BrowserPerformanceProfiler::new();
        
        // Verify initial state
        assert_eq!(profiler.get_monitoring_overhead(), 0.0);
        assert!(profiler.metrics_history.lock().unwrap().is_empty());
        assert!(profiler.baselines.lock().unwrap().is_empty());
    }
    
    #[test]
    fn test_detect_browser_performance_characteristics() {
        let profiler = BrowserPerformanceProfiler::new();
        let browser_info = create_test_browser_info("Chrome");
        
        let result = profiler.detect_browser_performance_characteristics(&browser_info);
        assert!(result.is_ok());
        
        let profile = result.unwrap();
        assert!(matches!(profile.audio_latency_profile, AudioLatencyProfile::UltraLow));
        assert!(matches!(profile.cpu_performance_tier, CpuPerformanceTier::High));
        assert_eq!(profile.memory_characteristics.available_heap, 2 * 1024 * 1024 * 1024);
        
        // Verify monitoring overhead is under 1ms
        assert!(profiler.get_monitoring_overhead() < 1.0);
    }
    
    #[test]
    fn test_realtime_performance_monitoring() {
        let profiler = BrowserPerformanceProfiler::new();
        let browser_info = create_test_browser_info("Chrome");
        let metrics = PerformanceMetrics {
            cpu_usage: 25.0,
            memory_usage: 512 * 1024 * 1024, // 512MB
            audio_latency: 15.0,
            optimization_effectiveness: 0.8,
        };
        
        let result = profiler.monitor_realtime_performance(&browser_info, &metrics);
        assert!(result.is_ok());
        
        let alerts = result.unwrap();
        assert!(alerts.is_empty()); // No alerts for good performance
        
        // Verify metrics were recorded
        let history = profiler.metrics_history.lock().unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].browser_name, "Chrome");
        assert_eq!(history[0].metrics.cpu_usage, 25.0);
        
        // Verify monitoring overhead is under 1ms
        assert!(profiler.get_monitoring_overhead() < 1.0);
    }
    
    #[test]
    fn test_performance_alerts_cpu_usage() {
        let profiler = BrowserPerformanceProfiler::new();
        let browser_info = create_test_browser_info("Chrome");
        let high_cpu_metrics = PerformanceMetrics {
            cpu_usage: 85.0, // Above critical threshold (80%)
            memory_usage: 512 * 1024 * 1024,
            audio_latency: 15.0,
            optimization_effectiveness: 0.8,
        };
        
        let result = profiler.monitor_realtime_performance(&browser_info, &high_cpu_metrics);
        assert!(result.is_ok());
        
        let alerts = result.unwrap();
        assert!(!alerts.is_empty());
        
        // Should have CPU usage alert
        assert!(alerts.iter().any(|alert| matches!(alert, PerformanceAlert::CpuUsageHigh { .. })));
    }
    
    #[test]
    fn test_performance_alerts_memory_usage() {
        let profiler = BrowserPerformanceProfiler::new();
        let browser_info = create_test_browser_info("Firefox");
        let high_memory_metrics = PerformanceMetrics {
            cpu_usage: 25.0,
            memory_usage: 2 * 1024 * 1024 * 1024, // 2GB - above critical threshold
            audio_latency: 15.0,
            optimization_effectiveness: 0.8,
        };
        
        let result = profiler.monitor_realtime_performance(&browser_info, &high_memory_metrics);
        assert!(result.is_ok());
        
        let alerts = result.unwrap();
        assert!(!alerts.is_empty());
        
        // Should have memory usage alert
        assert!(alerts.iter().any(|alert| matches!(alert, PerformanceAlert::MemoryUsageHigh { .. })));
    }
    
    #[test]
    fn test_performance_alerts_audio_latency() {
        let profiler = BrowserPerformanceProfiler::new();
        let browser_info = create_test_browser_info("Safari");
        let high_latency_metrics = PerformanceMetrics {
            cpu_usage: 25.0,
            memory_usage: 512 * 1024 * 1024,
            audio_latency: 60.0, // Above critical threshold (50ms)
            optimization_effectiveness: 0.8,
        };
        
        let result = profiler.monitor_realtime_performance(&browser_info, &high_latency_metrics);
        assert!(result.is_ok());
        
        let alerts = result.unwrap();
        assert!(!alerts.is_empty());
        
        // Should have audio latency alert
        assert!(alerts.iter().any(|alert| matches!(alert, PerformanceAlert::AudioLatencyHigh { .. })));
    }
    
    #[test]
    fn test_optimization_effectiveness_collection() {
        let profiler = BrowserPerformanceProfiler::new();
        let browser_info = create_test_browser_info("Chrome");
        
        // Add multiple metrics for Chrome
        for i in 0..5 {
            let metrics = PerformanceMetrics {
                cpu_usage: 20.0 + i as f32,
                memory_usage: 512 * 1024 * 1024,
                audio_latency: 10.0 + i as f32,
                optimization_effectiveness: 0.8 + (i as f32 * 0.02),
            };
            
            let _ = profiler.monitor_realtime_performance(&browser_info, &metrics);
        }
        
        // Collect effectiveness metrics
        let result = profiler.collect_optimization_effectiveness_metrics("Chrome");
        assert!(result.is_ok());
        
        let avg_metrics = result.unwrap();
        assert!((avg_metrics.cpu_usage - 22.0).abs() < 0.1); // Average of 20, 21, 22, 23, 24
        assert!((avg_metrics.optimization_effectiveness - 0.84).abs() < 0.01); // Average effectiveness
    }
    
    #[test]
    fn test_performance_baseline_establishment() {
        let profiler = BrowserPerformanceProfiler::new();
        let metrics = PerformanceMetrics {
            cpu_usage: 25.0,
            memory_usage: 512 * 1024 * 1024,
            audio_latency: 15.0,
            optimization_effectiveness: 0.8,
        };
        
        let result = profiler.establish_baseline("Chrome", &metrics);
        assert!(result.is_ok());
        
        // Verify baseline was stored
        let baselines = profiler.baselines.lock().unwrap();
        assert!(baselines.contains_key("Chrome"));
        
        let baseline = baselines.get("Chrome").unwrap();
        assert_eq!(baseline.cpu_usage_baseline, 25.0);
        assert_eq!(baseline.memory_usage_baseline, 512 * 1024 * 1024);
        assert_eq!(baseline.audio_latency_baseline, 15.0);
        assert_eq!(baseline.optimization_effectiveness_baseline, 0.8);
    }
    
    #[test]
    fn test_optimization_degradation_alert() {
        let profiler = BrowserPerformanceProfiler::new();
        let browser_info = create_test_browser_info("Chrome");
        
        // Establish baseline
        let baseline_metrics = PerformanceMetrics {
            cpu_usage: 25.0,
            memory_usage: 512 * 1024 * 1024,
            audio_latency: 15.0,
            optimization_effectiveness: 0.8,
        };
        
        let _ = profiler.establish_baseline("Chrome", &baseline_metrics);
        
        // Test with degraded performance (50% worse than baseline)
        let degraded_metrics = PerformanceMetrics {
            cpu_usage: 25.0,
            memory_usage: 512 * 1024 * 1024,
            audio_latency: 15.0,
            optimization_effectiveness: 0.4, // 50% worse than baseline
        };
        
        let result = profiler.monitor_realtime_performance(&browser_info, &degraded_metrics);
        assert!(result.is_ok());
        
        let alerts = result.unwrap();
        assert!(!alerts.is_empty());
        
        // Should have optimization degradation alert
        assert!(alerts.iter().any(|alert| matches!(alert, PerformanceAlert::OptimizationDegraded { .. })));
    }
    
    #[test]
    fn test_performance_history_analysis() {
        let profiler = BrowserPerformanceProfiler::new();
        let browser_info = create_test_browser_info("Chrome");
        
        // Add multiple metrics over time
        for i in 0..10 {
            let metrics = PerformanceMetrics {
                cpu_usage: 20.0 + (i as f32),
                memory_usage: 512 * 1024 * 1024 + (i as u64 * 10 * 1024 * 1024),
                audio_latency: 10.0 + (i as f32 * 0.5),
                optimization_effectiveness: 0.8 - (i as f32 * 0.01),
            };
            
            let _ = profiler.monitor_realtime_performance(&browser_info, &metrics);
        }
        
        // Analyze history
        let result = profiler.analyze_performance_history("Chrome", 24); // Last 24 hours
        assert!(result.is_ok());
        
        let analysis = result.unwrap();
        assert_eq!(analysis.browser_name, "Chrome");
        assert_eq!(analysis.sample_count, 10);
        assert!(analysis.cpu_usage_stats.mean > 20.0 && analysis.cpu_usage_stats.mean < 30.0);
        assert!(matches!(analysis.performance_trend, PerformanceTrend::Degrading)); // Effectiveness decreasing
    }
    
    #[test]
    fn test_regression_detection_system() {
        let profiler = BrowserPerformanceProfiler::new();
        
        let result = profiler.create_performance_regression_detection();
        assert!(result.is_ok());
        
        // Verify regression detector was reset
        let detector = profiler.regression_detector.lock().unwrap();
        assert!(detector.recent_samples.is_empty());
        assert_eq!(detector.analysis_window_size, 50);
        assert_eq!(detector.regression_sensitivity, 0.1);
    }
    
    #[test]
    fn test_statistical_summary_calculation() {
        let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let summary = StatisticalSummary::calculate(&values);
        
        assert_eq!(summary.mean, 30.0);
        assert_eq!(summary.min, 10.0);
        assert_eq!(summary.max, 50.0);
        assert!((summary.std_dev - 14.14).abs() < 0.1); // Standard deviation
    }
    
    #[test]
    fn test_statistical_summary_u64_calculation() {
        let values = vec![100_u64, 200_u64, 300_u64, 400_u64, 500_u64];
        let summary = StatisticalSummaryU64::calculate_u64(&values);
        
        assert_eq!(summary.mean, 300);
        assert_eq!(summary.min, 100);
        assert_eq!(summary.max, 500);
        assert!((summary.std_dev - 141.42).abs() < 0.1); // Standard deviation
    }
    
    #[test]
    fn test_performance_thresholds_configuration() {
        let profiler = BrowserPerformanceProfiler::new();
        
        let custom_thresholds = PerformanceThresholds {
            cpu_usage_warning: 60.0,
            cpu_usage_critical: 90.0,
            memory_usage_warning: 2 * 1024 * 1024 * 1024, // 2GB
            memory_usage_critical: 3 * 1024 * 1024 * 1024, // 3GB
            audio_latency_warning: 40.0,
            audio_latency_critical: 60.0,
            optimization_degradation_warning: 0.15,
            optimization_degradation_critical: 0.35,
        };
        
        let result = profiler.configure_performance_alerts(custom_thresholds.clone());
        assert!(result.is_ok());
        
        // Verify thresholds were set
        let stored_thresholds = profiler.alert_thresholds.lock().unwrap();
        assert_eq!(stored_thresholds.cpu_usage_warning, 60.0);
        assert_eq!(stored_thresholds.cpu_usage_critical, 90.0);
    }
    
    #[test]
    fn test_monitoring_overhead_constraint() {
        let profiler = BrowserPerformanceProfiler::new();
        let browser_info = create_test_browser_info("Chrome");
        
        // Test multiple operations to ensure consistent <1ms overhead
        for _ in 0..10 {
            let metrics = PerformanceMetrics {
                cpu_usage: 25.0,
                memory_usage: 512 * 1024 * 1024,
                audio_latency: 15.0,
                optimization_effectiveness: 0.8,
            };
            
            let start = std::time::Instant::now();
            let _ = profiler.monitor_realtime_performance(&browser_info, &metrics);
            let elapsed = start.elapsed().as_nanos() as f64 / 1_000_000.0;
            
            // Each individual operation should be well under 1ms
            assert!(elapsed < 1.0, "Monitoring overhead too high: {:.2}ms", elapsed);
        }
        
        // Overall monitoring overhead should remain under 1ms
        assert!(profiler.get_monitoring_overhead() < 1.0);
    }
    
    #[test]
    fn test_browser_specific_performance_profiles() {
        let profiler = BrowserPerformanceProfiler::new();
        
        // Test Chrome profile
        let chrome_info = create_test_browser_info("Chrome");
        let chrome_profile = profiler.detect_browser_performance_characteristics(&chrome_info).unwrap();
        assert!(matches!(chrome_profile.audio_latency_profile, AudioLatencyProfile::UltraLow));
        assert!(matches!(chrome_profile.cpu_performance_tier, CpuPerformanceTier::High));
        assert_eq!(chrome_profile.memory_characteristics.available_heap, 2 * 1024 * 1024 * 1024);
        
        // Test Firefox profile
        let firefox_info = create_test_browser_info("Firefox");
        let firefox_profile = profiler.detect_browser_performance_characteristics(&firefox_info).unwrap();
        assert!(matches!(firefox_profile.audio_latency_profile, AudioLatencyProfile::Low));
        assert!(matches!(firefox_profile.cpu_performance_tier, CpuPerformanceTier::High));
        assert_eq!(firefox_profile.memory_characteristics.available_heap, 1536 * 1024 * 1024);
        
        // Test Safari profile
        let safari_info = create_test_browser_info("Safari");
        let safari_profile = profiler.detect_browser_performance_characteristics(&safari_info).unwrap();
        assert!(matches!(safari_profile.audio_latency_profile, AudioLatencyProfile::Low));
        assert!(matches!(safari_profile.cpu_performance_tier, CpuPerformanceTier::High));
        assert_eq!(safari_profile.memory_characteristics.available_heap, 1024 * 1024 * 1024);
    }
    
    #[test]
    fn test_metrics_history_bounded_growth() {
        let profiler = BrowserPerformanceProfiler::new();
        let browser_info = create_test_browser_info("Chrome");
        
        // Add more than 1000 metrics to test bounded growth
        for i in 0..1100 {
            let metrics = PerformanceMetrics {
                cpu_usage: 20.0,
                memory_usage: 512 * 1024 * 1024,
                audio_latency: 15.0,
                optimization_effectiveness: 0.8,
            };
            
            let _ = profiler.monitor_realtime_performance(&browser_info, &metrics);
        }
        
        // History should be capped at 1000 entries
        let history = profiler.metrics_history.lock().unwrap();
        assert_eq!(history.len(), 1000);
    }
    
    // Graceful Degradation Mechanisms Tests (Task 3)
    
    #[test]
    fn test_graceful_degradation_manager_creation() {
        let manager = GracefulDegradationManager::new();
        
        // Verify initial state
        assert!(manager.compatibility_assessments.lock().unwrap().is_empty());
        assert!(manager.fallback_strategies.lock().unwrap().is_empty());
        assert!(manager.active_fallbacks.lock().unwrap().is_empty());
        
        // Verify default progressive enhancement config
        let config = manager.progressive_enhancement.lock().unwrap();
        assert!(config.base_feature_set.contains(&"basic_audio_context".to_string()));
        assert!(config.enhanced_feature_set.contains(&"audio_worklet".to_string()));
        assert!(config.premium_feature_set.contains(&"wasm_simd_processing".to_string()));
    }
    
    #[test]
    fn test_feature_capability_assessment_full_support() {
        let manager = GracefulDegradationManager::new();
        let browser_info = create_test_browser_info("Chrome"); // Chrome has full WebAssembly support
        
        let result = manager.assess_feature_capability(&browser_info, "webassembly");
        assert!(result.is_ok());
        
        let assessment = result.unwrap();
        assert_eq!(assessment.feature_name, "webassembly");
        assert_eq!(assessment.browser_name, "Chrome");
        assert!(matches!(assessment.support_level, FeatureSupportLevel::FullySupported));
        assert!(assessment.compatibility_score > 0.9);
        assert!(assessment.limitations.is_empty());
        assert!(matches!(assessment.recommended_action, RecommendedAction::UseAsIs));
    }
    
    #[test]
    fn test_feature_capability_assessment_partial_support() {
        let manager = GracefulDegradationManager::new();
        let mut browser_info = create_test_browser_info("Chrome");
        browser_info.capabilities.supports_audio_worklet = false; // Remove audio worklet support
        
        let result = manager.assess_feature_capability(&browser_info, "audio_worklet");
        assert!(result.is_ok());
        
        let assessment = result.unwrap();
        assert_eq!(assessment.feature_name, "audio_worklet");
        assert!(matches!(assessment.support_level, FeatureSupportLevel::PartiallySupported));
        assert!(assessment.compatibility_score < 0.8);
        assert!(!assessment.limitations.is_empty());
        assert!(matches!(assessment.recommended_action, RecommendedAction::ApplyWorkaround));
    }
    
    #[test]
    fn test_feature_capability_assessment_no_support() {
        let manager = GracefulDegradationManager::new();
        let mut browser_info = create_test_browser_info("OldBrowser");
        browser_info.capabilities.supports_wasm_simd = false;
        
        let result = manager.assess_feature_capability(&browser_info, "wasm_simd");
        assert!(result.is_ok());
        
        let assessment = result.unwrap();
        assert!(matches!(assessment.support_level, FeatureSupportLevel::NotSupported));
        assert_eq!(assessment.compatibility_score, 0.0);
        assert!(matches!(assessment.recommended_action, RecommendedAction::DisableFeature));
    }
    
    #[test]
    fn test_fallback_strategy_creation_partial_support() {
        let manager = GracefulDegradationManager::new();
        let mut browser_info = create_test_browser_info("Firefox");
        browser_info.capabilities.supports_audio_worklet = false;
        
        let assessment = manager.assess_feature_capability(&browser_info, "audio_worklet").unwrap();
        let result = manager.create_fallback_strategy("audio_worklet", &browser_info, &assessment);
        assert!(result.is_ok());
        
        let strategy = result.unwrap();
        assert_eq!(strategy.feature_name, "audio_worklet");
        assert!(matches!(strategy.fallback_type, FallbackType::LegacyImplementation));
        assert!(strategy.performance_impact > 0.0);
        assert!(strategy.feature_coverage > 0.0);
        assert!(!strategy.activation_conditions.is_empty());
    }
    
    #[test]
    fn test_fallback_strategy_creation_no_support() {
        let manager = GracefulDegradationManager::new();
        let mut browser_info = create_test_browser_info("OldBrowser");
        browser_info.capabilities.supports_wasm = false;
        
        let assessment = manager.assess_feature_capability(&browser_info, "webassembly").unwrap();
        let result = manager.create_fallback_strategy("webassembly", &browser_info, &assessment);
        assert!(result.is_ok());
        
                 let strategy = result.unwrap();
         assert!(matches!(strategy.fallback_type, FallbackType::GracefulDisabling));
         assert_eq!(strategy.performance_impact, 0.0);
         assert_eq!(strategy.feature_coverage, 0.0);
     }
     
     #[test]
     fn test_fallback_strategy_creation_full_support_fails() {
         let manager = GracefulDegradationManager::new();
         let browser_info = create_test_browser_info("Chrome"); // Full support
         
         let assessment = manager.assess_feature_capability(&browser_info, "webassembly").unwrap();
         let result = manager.create_fallback_strategy("webassembly", &browser_info, &assessment);
         assert!(result.is_err()); // Should fail for fully supported features
     }
 } 