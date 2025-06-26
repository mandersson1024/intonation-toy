use super::*;
use web_sys::{window, Navigator};
use wasm_bindgen::prelude::*;
use std::time::Instant;

/// Enhanced browser compatibility detector implementation
pub struct BrowserCompatibilityImpl {
    cached_browser_info: std::sync::Mutex<Option<(BrowserInfo, Instant)>>,
    performance_profile_cache: std::sync::Mutex<Option<(BrowserPerformanceProfile, Instant)>>,
}

impl BrowserCompatibilityImpl {
    pub fn new() -> Self {
        Self {
            cached_browser_info: std::sync::Mutex::new(None),
            performance_profile_cache: std::sync::Mutex::new(None),
        }
    }
    
    /// Detect browser with performance profiling capabilities
    fn detect_browser_internal() -> Result<BrowserInfo, PlatformError> {
        #[cfg(target_arch = "wasm32")]
        {
            let window = window().ok_or(PlatformError::BrowserDetectionFailed("No window object".to_string()))?;
            let navigator = window.navigator();
            
            let user_agent = navigator.user_agent()
                .map_err(|_| PlatformError::BrowserDetectionFailed("Failed to get user agent".to_string()))?;
            
            let browser_name = Self::detect_browser_name(&user_agent);
            let version = Self::extract_browser_version(&user_agent, &browser_name);
            
            let capabilities = Self::detect_capabilities(&window, &navigator)
                .map_err(|e| PlatformError::BrowserDetectionFailed(format!("Capability detection failed: {:?}", e)))?;
            
            let compatibility_level = Self::determine_compatibility_level(&browser_name, &version, &capabilities);
            
            Ok(BrowserInfo {
                browser_name,
                version,
                user_agent,
                capabilities,
                compatibility_level,
            })
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Non-WASM fallback for testing
            Ok(BrowserInfo {
                browser_name: "Testing".to_string(),
                version: BrowserVersion { major: 1, minor: 0, patch: 0 },
                user_agent: "Test Environment".to_string(),
                capabilities: BrowserCapabilities {
                    supports_wasm: true,
                    supports_wasm_streaming: true,
                    supports_wasm_simd: false,
                    supports_audio_context: true,
                    supports_audio_worklet: true,
                    supports_media_devices: true,
                    supports_shared_array_buffer: false,
                    performance_api: true,
                },
                compatibility_level: CompatibilityLevel::FullySupported,
            })
        }
    }
    
    fn detect_browser_name(user_agent: &str) -> String {
        if user_agent.contains("Chrome") && !user_agent.contains("Edg") {
            "Chrome".to_string()
        } else if user_agent.contains("Firefox") {
            "Firefox".to_string()
        } else if user_agent.contains("Safari") && !user_agent.contains("Chrome") {
            "Safari".to_string()
        } else if user_agent.contains("Edg") {
            "Edge".to_string()
        } else {
            "Unknown".to_string()
        }
    }
    
    fn extract_browser_version(user_agent: &str, browser_name: &str) -> BrowserVersion {
        let parts: Vec<&str> = user_agent.split(&[' ', '/', '.']).collect();
        let mut version_parts = Vec::new();
        
        for (i, part) in parts.iter().enumerate() {
            if *part == browser_name || (browser_name == "Edge" && *part == "Edg") {
                if i + 1 < parts.len() {
                    if let Ok(major) = parts[i + 1].parse::<u32>() {
                        version_parts.push(major);
                        if i + 2 < parts.len() {
                            if let Ok(minor) = parts[i + 2].parse::<u32>() {
                                version_parts.push(minor);
                                if i + 3 < parts.len() {
                                    if let Ok(patch) = parts[i + 3].parse::<u32>() {
                                        version_parts.push(patch);
                                    }
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }

        if version_parts.len() >= 2 {
            BrowserVersion {
                major: version_parts[0],
                minor: version_parts[1],
                patch: version_parts.get(2).copied().unwrap_or(0),
            }
        } else {
            BrowserVersion { major: 0, minor: 0, patch: 0 }
        }
    }
    
    #[cfg(target_arch = "wasm32")]
    fn detect_capabilities(window: &web_sys::Window, navigator: &Navigator) -> Result<BrowserCapabilities, JsValue> {
        Ok(BrowserCapabilities {
            supports_wasm: Self::check_wasm_support(),
            supports_wasm_streaming: Self::check_wasm_streaming_support(),
            supports_wasm_simd: Self::check_wasm_simd_support(),
            supports_audio_context: Self::check_audio_context_support(),
            supports_audio_worklet: Self::check_audio_worklet_support(),
            supports_media_devices: Self::check_media_devices_support(),
            supports_shared_array_buffer: Self::check_shared_array_buffer_support(),
            performance_api: Self::check_performance_api_support(),
        })
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn detect_capabilities(_window: &(), _navigator: &()) -> Result<BrowserCapabilities, ()> {
        Ok(BrowserCapabilities {
            supports_wasm: true,
            supports_wasm_streaming: true,
            supports_wasm_simd: false,
            supports_audio_context: true,
            supports_audio_worklet: true,
            supports_media_devices: true,
            supports_shared_array_buffer: false,
            performance_api: true,
        })
    }
    
    fn check_wasm_support() -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::eval("typeof WebAssembly === 'object'")
                .map(|val| val.as_bool().unwrap_or(false))
                .unwrap_or(false)
        }
        #[cfg(not(target_arch = "wasm32"))]
        true
    }

    fn check_wasm_streaming_support() -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::eval("typeof WebAssembly.instantiateStreaming === 'function'")
                .map(|val| val.as_bool().unwrap_or(false))
                .unwrap_or(false)
        }
        #[cfg(not(target_arch = "wasm32"))]
        true
    }

    fn check_wasm_simd_support() -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::eval("'simd' in WebAssembly")
                .map(|val| val.as_bool().unwrap_or(false))
                .unwrap_or(false)
        }
        #[cfg(not(target_arch = "wasm32"))]
        false
    }

    fn check_audio_context_support() -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::eval("typeof (window.AudioContext || window.webkitAudioContext) === 'function'")
                .map(|val| val.as_bool().unwrap_or(false))
                .unwrap_or(false)
        }
        #[cfg(not(target_arch = "wasm32"))]
        true
    }

    fn check_audio_worklet_support() -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::eval("window.AudioContext && 'audioWorklet' in new (window.AudioContext || window.webkitAudioContext)()")
                .map(|val| val.as_bool().unwrap_or(false))
                .unwrap_or(false)
        }
        #[cfg(not(target_arch = "wasm32"))]
        true
    }

    fn check_media_devices_support() -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::eval("navigator.mediaDevices && typeof navigator.mediaDevices.getUserMedia === 'function'")
                .map(|val| val.as_bool().unwrap_or(false))
                .unwrap_or(false)
        }
        #[cfg(not(target_arch = "wasm32"))]
        true
    }

    fn check_shared_array_buffer_support() -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::eval("typeof SharedArrayBuffer !== 'undefined'")
                .map(|val| val.as_bool().unwrap_or(false))
                .unwrap_or(false)
        }
        #[cfg(not(target_arch = "wasm32"))]
        false
    }

    fn check_performance_api_support() -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::eval("typeof performance !== 'undefined' && typeof performance.now === 'function'")
                .map(|val| val.as_bool().unwrap_or(false))
                .unwrap_or(false)
        }
        #[cfg(not(target_arch = "wasm32"))]
        true
    }
    
    fn determine_compatibility_level(
        browser_name: &str,
        version: &BrowserVersion,
        capabilities: &BrowserCapabilities,
    ) -> CompatibilityLevel {
        // Enhanced version requirements based on tech stack spec
        let meets_version_req = match browser_name {
            "Chrome" => version.major >= 69,
            "Firefox" => version.major >= 76,
            "Safari" => version.major >= 14 && (version.major > 14 || version.minor >= 1),
            "Edge" => version.major >= 79,
            _ => false,
        };

        if !meets_version_req {
            return CompatibilityLevel::Unsupported;
        }

        // Essential capabilities for audio processing
        if !capabilities.supports_wasm || !capabilities.supports_audio_context {
            return CompatibilityLevel::Unsupported;
        }

        // Enhanced scoring based on performance characteristics
        let performance_score = [
            capabilities.supports_wasm_streaming,      // Faster loading
            capabilities.supports_audio_worklet,       // Low latency audio
            capabilities.supports_media_devices,       // Microphone access
            capabilities.performance_api,              // Performance monitoring
            capabilities.supports_wasm_simd,           // SIMD optimizations
        ].iter().filter(|&&x| x).count();

        match performance_score {
            5 => CompatibilityLevel::FullySupported,
            4 => CompatibilityLevel::FullySupported,
            3 => CompatibilityLevel::MostlySupported,
            2 => CompatibilityLevel::PartiallySupported,
            _ => CompatibilityLevel::Unsupported,
        }
    }
    
    /// Generate performance profile based on browser characteristics
    fn generate_performance_profile(browser_info: &BrowserInfo) -> BrowserPerformanceProfile {
        let audio_latency_profile = match (&browser_info.browser_name.as_str(), &browser_info.capabilities.supports_audio_worklet) {
            ("Chrome", true) => AudioLatencyProfile::UltraLow,
            ("Edge", true) => AudioLatencyProfile::UltraLow,
            ("Firefox", true) => AudioLatencyProfile::Low,
            ("Safari", true) => AudioLatencyProfile::Low,
            (_, true) => AudioLatencyProfile::Medium,
            _ => AudioLatencyProfile::High,
        };
        
        let memory_characteristics = MemoryCharacteristics {
            available_heap: Self::estimate_heap_size(&browser_info.browser_name),
            supports_shared_memory: browser_info.capabilities.supports_shared_array_buffer,
            garbage_collection_impact: Self::estimate_gc_impact(&browser_info.browser_name),
        };
        
        let cpu_performance_tier = match browser_info.browser_name.as_str() {
            "Chrome" => CpuPerformanceTier::High,
            "Edge" => CpuPerformanceTier::High,
            "Firefox" => CpuPerformanceTier::Medium,
            "Safari" => CpuPerformanceTier::Medium,
            _ => CpuPerformanceTier::Low,
        };
        
        BrowserPerformanceProfile {
            audio_latency_profile,
            memory_characteristics,
            cpu_performance_tier,
        }
    }
    
    fn estimate_heap_size(browser_name: &str) -> u64 {
        match browser_name {
            "Chrome" | "Edge" => 4 * 1024 * 1024 * 1024, // 4GB typical
            "Firefox" => 2 * 1024 * 1024 * 1024,         // 2GB typical
            "Safari" => 1024 * 1024 * 1024,              // 1GB typical (more conservative)
            _ => 512 * 1024 * 1024,                       // 512MB fallback
        }
    }
    
    fn estimate_gc_impact(browser_name: &str) -> GcImpact {
        match browser_name {
            "Chrome" | "Edge" => GcImpact::Minimal,
            "Firefox" => GcImpact::Moderate,
            "Safari" => GcImpact::Moderate,
            _ => GcImpact::Significant,
        }
    }
    
    /// Check if cache is valid (not older than 30 seconds for performance, 5 minutes for browser info)
    fn is_browser_cache_valid(cached_time: &Instant) -> bool {
        cached_time.elapsed().as_secs() < 300 // 5 minutes
    }
    
    fn is_performance_cache_valid(cached_time: &Instant) -> bool {
        cached_time.elapsed().as_secs() < 30 // 30 seconds
    }
}

impl Default for BrowserCompatibilityImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserCompatibility for BrowserCompatibilityImpl {
    fn detect_browser(&self) -> Result<BrowserInfo, PlatformError> {
        // Check cache first (5-minute window as per <5ms requirement after first detection)
        {
            let cache = self.cached_browser_info.lock().unwrap();
            if let Some((ref info, ref cached_time)) = *cache {
                if Self::is_browser_cache_valid(cached_time) {
                    return Ok(info.clone());
                }
            }
        }
        
        // Perform fresh detection
        let browser_info = Self::detect_browser_internal()?;
        
        // Update cache
        {
            let mut cache = self.cached_browser_info.lock().unwrap();
            *cache = Some((browser_info.clone(), Instant::now()));
        }
        
        Ok(browser_info)
    }
    
    fn get_cached_browser_info(&self) -> Option<&BrowserInfo> {
        // Note: This method signature doesn't work well with Mutex, 
        // so we'll return None and recommend using detect_browser() instead
        // which handles caching internally and meets the <5ms requirement
        None
    }
    
    fn is_feature_supported(&self, feature: &str) -> bool {
        if let Ok(browser_info) = self.detect_browser() {
            match feature {
                "wasm" => browser_info.capabilities.supports_wasm,
                "wasm_streaming" => browser_info.capabilities.supports_wasm_streaming,
                "wasm_simd" => browser_info.capabilities.supports_wasm_simd,
                "audio_context" => browser_info.capabilities.supports_audio_context,
                "audio_worklet" => browser_info.capabilities.supports_audio_worklet,
                "media_devices" => browser_info.capabilities.supports_media_devices,
                "shared_array_buffer" => browser_info.capabilities.supports_shared_array_buffer,
                "performance_api" => browser_info.capabilities.performance_api,
                _ => false,
            }
        } else {
            false
        }
    }
    
    fn get_performance_profile(&self) -> BrowserPerformanceProfile {
        // Check cache first (30-second window for performance metrics)
        {
            let cache = self.performance_profile_cache.lock().unwrap();
            if let Some((ref profile, ref cached_time)) = *cache {
                if Self::is_performance_cache_valid(cached_time) {
                    return profile.clone();
                }
            }
        }
        
        // Generate fresh performance profile
        let profile = if let Ok(browser_info) = self.detect_browser() {
            Self::generate_performance_profile(&browser_info)
        } else {
            // Fallback profile
            BrowserPerformanceProfile {
                audio_latency_profile: AudioLatencyProfile::High,
                memory_characteristics: MemoryCharacteristics {
                    available_heap: 512 * 1024 * 1024, // 512MB fallback
                    supports_shared_memory: false,
                    garbage_collection_impact: GcImpact::Significant,
                },
                cpu_performance_tier: CpuPerformanceTier::Low,
            }
        };
        
        // Update cache
        {
            let mut cache = self.performance_profile_cache.lock().unwrap();
            *cache = Some((profile.clone(), Instant::now()));
        }
        
        profile
    }
} 