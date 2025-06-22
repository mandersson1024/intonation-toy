use web_sys::{window, Navigator, MediaDevices};
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrowserCapabilities {
    pub supports_wasm: bool,
    pub supports_wasm_streaming: bool,
    pub supports_wasm_simd: bool,
    pub supports_audio_context: bool,
    pub supports_audio_worklet: bool,
    pub supports_media_devices: bool,
    pub supports_shared_array_buffer: bool,
    pub performance_api: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrowserVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompatibilityLevel {
    FullySupported,
    MostlySupported,
    PartiallySupported,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BrowserInfo {
    pub supports_wasm: bool,
    pub supports_audio_context: bool,
    pub browser_name: String,
    pub browser_version: Option<BrowserVersion>,
    pub user_agent: String,
    pub is_supported: bool,
    pub capabilities: BrowserCapabilities,
    pub compatibility_level: CompatibilityLevel,
}

impl BrowserInfo {
    pub fn detect() -> Result<Self, JsValue> {
        let window = window().ok_or("No window object")?;
        let navigator = window.navigator();
        
        let user_agent = navigator.user_agent()?;
        let browser_name = Self::detect_browser_name(&user_agent);
        let browser_version = Self::extract_browser_version(&user_agent, &browser_name);
        
        let capabilities = Self::detect_capabilities(&window, &navigator)?;
        let compatibility_level = Self::determine_compatibility_level(&browser_name, &browser_version, &capabilities);
        
        let supports_wasm = capabilities.supports_wasm;
        let supports_audio_context = capabilities.supports_audio_context;
        let is_supported = matches!(compatibility_level, CompatibilityLevel::FullySupported | CompatibilityLevel::MostlySupported);

        Ok(BrowserInfo {
            supports_wasm,
            supports_audio_context,
            browser_name,
            browser_version,
            user_agent,
            is_supported,
            capabilities,
            compatibility_level,
        })
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

    fn extract_browser_version(user_agent: &str, browser_name: &str) -> Option<BrowserVersion> {
        let version_regex = match browser_name {
            "Chrome" => r"Chrome/(\d+)\.(\d+)\.(\d+)",
            "Firefox" => r"Firefox/(\d+)\.(\d+)(?:\.(\d+))?",
            "Safari" => r"Version/(\d+)\.(\d+)(?:\.(\d+))?",
            "Edge" => r"Edg/(\d+)\.(\d+)\.(\d+)",
            _ => return None,
        };

        // Simple regex parsing (in a real implementation, you'd use a proper regex crate)
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
            Some(BrowserVersion {
                major: version_parts[0],
                minor: version_parts[1],
                patch: version_parts.get(2).copied().unwrap_or(0),
            })
        } else {
            None
        }
    }

    fn detect_capabilities(window: &web_sys::Window, navigator: &Navigator) -> Result<BrowserCapabilities, JsValue> {
        Ok(BrowserCapabilities {
            supports_wasm: Self::check_wasm_support(),
            supports_wasm_streaming: Self::check_wasm_streaming_support(),
            supports_wasm_simd: Self::check_wasm_simd_support(),
            supports_audio_context: Self::check_audio_context_support(window),
            supports_audio_worklet: Self::check_audio_worklet_support(window),
            supports_media_devices: Self::check_media_devices_support(navigator),
            supports_shared_array_buffer: Self::check_shared_array_buffer_support(),
            performance_api: Self::check_performance_api_support(window),
        })
    }

    fn check_wasm_support() -> bool {
        js_sys::eval("typeof WebAssembly === 'object'")
            .map(|val| val.as_bool().unwrap_or(false))
            .unwrap_or(false)
    }

    fn check_wasm_streaming_support() -> bool {
        js_sys::eval("typeof WebAssembly.instantiateStreaming === 'function'")
            .map(|val| val.as_bool().unwrap_or(false))
            .unwrap_or(false)
    }

    fn check_wasm_simd_support() -> bool {
        js_sys::eval("'simd' in WebAssembly")
            .map(|val| val.as_bool().unwrap_or(false))
            .unwrap_or(false)
    }

    fn check_audio_context_support(window: &web_sys::Window) -> bool {
        js_sys::eval("typeof (window.AudioContext || window.webkitAudioContext) === 'function'")
            .map(|val| val.as_bool().unwrap_or(false))
            .unwrap_or(false)
    }

    fn check_audio_worklet_support(_window: &web_sys::Window) -> bool {
        js_sys::eval("window.AudioContext && 'audioWorklet' in new (window.AudioContext || window.webkitAudioContext)()")
            .map(|val| val.as_bool().unwrap_or(false))
            .unwrap_or(false)
    }

    fn check_media_devices_support(navigator: &Navigator) -> bool {
        js_sys::eval("navigator.mediaDevices && typeof navigator.mediaDevices.getUserMedia === 'function'")
            .map(|val| val.as_bool().unwrap_or(false))
            .unwrap_or(false)
    }

    fn check_shared_array_buffer_support() -> bool {
        js_sys::eval("typeof SharedArrayBuffer !== 'undefined'")
            .map(|val| val.as_bool().unwrap_or(false))
            .unwrap_or(false)
    }

    fn check_performance_api_support(window: &web_sys::Window) -> bool {
        js_sys::eval("typeof performance !== 'undefined' && typeof performance.now === 'function'")
            .map(|val| val.as_bool().unwrap_or(false))
            .unwrap_or(false)
    }

    fn determine_compatibility_level(
        browser_name: &str,
        version: &Option<BrowserVersion>,
        capabilities: &BrowserCapabilities,
    ) -> CompatibilityLevel {
        // Check minimum version requirements
        let meets_version_req = match (browser_name, version) {
            ("Chrome", Some(v)) => v.major >= 69,
            ("Firefox", Some(v)) => v.major >= 76,
            ("Safari", Some(v)) => v.major >= 14 && (v.major > 14 || v.minor >= 1),
            ("Edge", Some(v)) => v.major >= 79,
            _ => false,
        };

        if !meets_version_req {
            return CompatibilityLevel::Unsupported;
        }

        // Check essential capabilities
        if !capabilities.supports_wasm || !capabilities.supports_audio_context {
            return CompatibilityLevel::Unsupported;
        }

        // Determine level based on optional features
        let optional_features_count = [
            capabilities.supports_wasm_streaming,
            capabilities.supports_audio_worklet,
            capabilities.supports_media_devices,
            capabilities.performance_api,
        ].iter().filter(|&&x| x).count();

        match optional_features_count {
            4 => CompatibilityLevel::FullySupported,
            3 => CompatibilityLevel::MostlySupported,
            2 => CompatibilityLevel::PartiallySupported,
            _ => CompatibilityLevel::Unsupported,
        }
    }

    pub fn get_compatibility_message(&self) -> String {
        match self.compatibility_level {
            CompatibilityLevel::FullySupported => {
                format!("✅ Your {} browser is fully supported with all features!", self.browser_name)
            }
            CompatibilityLevel::MostlySupported => {
                format!("✅ Your {} browser is mostly supported. Some advanced features may be limited.", self.browser_name)
            }
            CompatibilityLevel::PartiallySupported => {
                format!("⚠️ Your {} browser has partial support. Some features may not work optimally.", self.browser_name)
            }
            CompatibilityLevel::Unsupported => {
                if !self.capabilities.supports_wasm {
                    "❌ WebAssembly is not supported. Please upgrade your browser.".to_string()
                } else if !self.capabilities.supports_audio_context {
                    "❌ Web Audio API is not supported. Please upgrade your browser.".to_string()
                } else {
                    format!("❌ Your browser ({}) is not supported. Please upgrade to: Chrome 69+, Firefox 76+, Safari 14.1+, or Edge 79+", self.browser_name)
                }
            }
        }
    }

    pub fn get_upgrade_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        match self.compatibility_level {
            CompatibilityLevel::Unsupported => {
                recommendations.push("Please upgrade to a supported browser version:".to_string());
                recommendations.push("• Chrome 69 or later".to_string());
                recommendations.push("• Firefox 76 or later".to_string());
                recommendations.push("• Safari 14.1 or later".to_string());
                recommendations.push("• Edge 79 or later".to_string());
            }
            CompatibilityLevel::PartiallySupported => {
                recommendations.push("For optimal experience, consider upgrading your browser.".to_string());
                if !self.capabilities.supports_audio_worklet {
                    recommendations.push("• AudioWorklet support is limited - audio latency may be higher".to_string());
                }
                if !self.capabilities.supports_wasm_streaming {
                    recommendations.push("• WebAssembly streaming not available - slower loading times".to_string());
                }
            }
            _ => {} // Fully or mostly supported
        }
        
        recommendations
    }

    pub fn get_capability_report(&self) -> String {
        format!(
            "Browser Capability Report:\n\
            WebAssembly: {}\n\
            WebAssembly Streaming: {}\n\
            WebAssembly SIMD: {}\n\
            Web Audio API: {}\n\
            AudioWorklet: {}\n\
            MediaDevices: {}\n\
            SharedArrayBuffer: {}\n\
            Performance API: {}",
            if self.capabilities.supports_wasm { "✅" } else { "❌" },
            if self.capabilities.supports_wasm_streaming { "✅" } else { "❌" },
            if self.capabilities.supports_wasm_simd { "✅" } else { "❌" },
            if self.capabilities.supports_audio_context { "✅" } else { "❌" },
            if self.capabilities.supports_audio_worklet { "✅" } else { "❌" },
            if self.capabilities.supports_media_devices { "✅" } else { "❌" },
            if self.capabilities.supports_shared_array_buffer { "✅" } else { "❌" },
            if self.capabilities.performance_api { "✅" } else { "❌" },
        )
    }
} 