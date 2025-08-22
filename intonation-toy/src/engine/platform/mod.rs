// Platform detection and feature support module
// Centralizes browser API availability checks and platform-specific logic

pub mod commands;

use crate::common::dev_log;


#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

/// Platform feature validation results
#[derive(Debug, Clone, PartialEq)]
pub enum PlatformValidationResult {
    AllSupported,
    // TODO: For semantic clarity we could implement the type MissingApi, but as long
    // as the implementation is identical to CriticalApi we keep it simple and use that.
    MissingCriticalApis(Vec<CriticalApi>),
    MobileDevice,
}

/// Critical APIs that must be available for application startup
#[derive(Debug, Clone, PartialEq)]
pub enum CriticalApi {
    WebAudioApi,
    GetUserMedia,
    AudioWorklet,
    Canvas,
    WebGL2,
}

/// API support status
#[derive(Debug, Clone)]
pub struct ApiStatus {
    pub api: CriticalApi,
    pub supported: bool,
    pub details: Option<String>,
}

impl std::fmt::Display for CriticalApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CriticalApi::WebAudioApi => write!(f, "Web Audio API"),
            CriticalApi::GetUserMedia => write!(f, "getUserMedia API"),
            CriticalApi::AudioWorklet => write!(f, "AudioWorklet"),
            CriticalApi::Canvas => write!(f, "Canvas"),
            CriticalApi::WebGL2 => write!(f, "WebGL2"),
        }
    }
}

/// Platform feature detection and initialization
pub struct Platform;

impl Platform {
    /// Get list of all critical APIs that must be checked (single source of truth)
    pub fn get_critical_apis() -> Vec<CriticalApi> {
        vec![
            CriticalApi::WebAudioApi,
            CriticalApi::GetUserMedia,
            CriticalApi::AudioWorklet,
            CriticalApi::Canvas,
            CriticalApi::WebGL2,
        ]
    }

    /// Get detailed status of all critical APIs
    /// Optimized to reuse shared contexts (AudioContext, Canvas) across multiple checks
    pub fn get_api_status() -> Vec<ApiStatus> {
        #[cfg(target_arch = "wasm32")]
        {
        let mut results = Vec::new();
        
        // Get shared window/document once
        let window = match web_sys::window() {
            Some(w) => w,
            None => {
                // If window fails, all browser APIs fail
                return Self::get_critical_apis()
                    .into_iter()
                    .map(|api| ApiStatus {
                        api,
                        supported: false,
                        details: Some("Window object not available".to_string()),
                    })
                    .collect();
            }
        };
        
        let document = window.document();
        
        // Check getUserMedia (safe, no popup)
        let is_supported = super::audio::permission::PermissionManager::is_supported();
        results.push(ApiStatus {
            api: CriticalApi::GetUserMedia,
            supported: is_supported,
            details: if is_supported {
                Some("getUserMedia API available".to_string())
            } else {
                Some("getUserMedia API not available".to_string())
            },
        });
        
        // Create shared AudioContext for WebAudio + AudioWorklet checks
        let audio_context = if js_sys::Reflect::has(&window, &"AudioContext".into()).unwrap_or(false) {
            js_sys::Reflect::get(&window, &"AudioContext".into())
                .ok()
                .and_then(|constructor| constructor.dyn_into::<js_sys::Function>().ok())
                .and_then(|constructor| js_sys::Reflect::construct(&constructor, &js_sys::Array::new()).ok())
        } else {
            None
        };
        
        // WebAudio check using shared context
        results.push(match &audio_context {
            Some(_) => ApiStatus {
                api: CriticalApi::WebAudioApi,
                supported: true,
                details: Some("AudioContext creation successful".to_string()),
            },
            None => ApiStatus {
                api: CriticalApi::WebAudioApi,
                supported: false,
                details: Some("AudioContext creation failed".to_string()),
            },
        });
        
        // AudioWorklet check using same AudioContext
        results.push(match &audio_context {
            Some(ctx) => {
                let has_audioworklet = js_sys::Reflect::has(ctx, &"audioWorklet".into()).unwrap_or(false);
                ApiStatus {
                    api: CriticalApi::AudioWorklet,
                    supported: has_audioworklet,
                    details: if has_audioworklet {
                        Some("AudioWorklet API available".to_string())
                    } else {
                        Some("AudioWorklet not supported".to_string())
                    },
                }
            },
            None => ApiStatus {
                api: CriticalApi::AudioWorklet,
                supported: false,
                details: Some("AudioContext creation failed".to_string()),
            },
        });
        
        // Create shared canvas for Canvas + WebGL2 checks
        let canvas = document.and_then(|doc| {
            doc.create_element("canvas").ok().and_then(|canvas| {
                canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok()
            })
        });
        
        // Canvas check using shared canvas
        results.push(match &canvas {
            Some(_) => ApiStatus {
                api: CriticalApi::Canvas,
                supported: true,
                details: Some("Canvas element creation successful".to_string()),
            },
            None => ApiStatus {
                api: CriticalApi::Canvas,
                supported: false,
                details: Some("Canvas element creation failed".to_string()),
            },
        });
        
        // WebGL2 check using same canvas
        results.push(match &canvas {
            Some(canvas) => {
                match canvas.get_context("webgl2") {
                    Ok(Some(_)) => ApiStatus {
                        api: CriticalApi::WebGL2,
                        supported: true,
                        details: Some("WebGL2 context creation successful".to_string()),
                    },
                    Ok(None) => ApiStatus {
                        api: CriticalApi::WebGL2,
                        supported: false,
                        details: Some("WebGL2 context not available".to_string()),
                    },
                    Err(_) => ApiStatus {
                        api: CriticalApi::WebGL2,
                        supported: false,
                        details: Some("WebGL2 not supported".to_string()),
                    },
                }
            },
            None => ApiStatus {
                api: CriticalApi::WebGL2,
                supported: false,
                details: Some("Canvas element creation failed".to_string()),
            },
        });
        
        results
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::get_critical_apis()
                .into_iter()
                .map(|api| ApiStatus {
                    api,
                    supported: false,
                    details: Some("Not running in browser environment".to_string()),
                })
                .collect()
        }
    }

    /// Validate all critical platform features required for application startup
    /// Returns validation result that caller MUST handle - application should not start if APIs are missing
    pub fn check_feature_support() -> PlatformValidationResult {
        dev_log!("Validating critical platform APIs...");
        
        // Check for mobile device first
        if Self::is_mobile_device() {
            dev_log!("✗ Mobile device detected - application not supported on mobile devices");
            return PlatformValidationResult::MobileDevice;
        }
        
        let api_statuses = Self::get_api_status();
        let missing_apis: Vec<CriticalApi> = api_statuses
            .iter()
            .filter(|status| !status.supported)
            .map(|status| status.api.clone())
            .collect();
        
        if missing_apis.is_empty() {
            dev_log!("✓ All critical platform APIs are supported");
            PlatformValidationResult::AllSupported
        } else {
            dev_log!("✗ Critical platform APIs are missing: {:?}", missing_apis);
            PlatformValidationResult::MissingCriticalApis(missing_apis)
        }
    }

    /// Get platform information string for debugging
    pub fn get_platform_info() -> String {
        // In WASM environment, get actual user agent
        #[cfg(target_arch = "wasm32")]
        let user_agent = web_sys::window()
            .and_then(|w| w.navigator().user_agent().ok())
            .unwrap_or_else(|| "Unknown".to_string());
        
        // In native environment (tests), return placeholder
        #[cfg(not(target_arch = "wasm32"))]
        let user_agent = "Unknown".to_string();
        
        format!("UserAgent: {}", user_agent)
    }

    /// Check if the current platform is a mobile device
    #[cfg(target_arch = "wasm32")]
    pub fn is_mobile_device() -> bool {
        if let Some(window) = web_sys::window() {
            if let Ok(user_agent) = window.navigator().user_agent() {
                // Common mobile device patterns
                let mobile_patterns = [
                    "Android",
                    "iPhone",
                    "iPad",
                    "iPod",
                    "BlackBerry",
                    "Windows Phone",
                    "webOS",
                    "Opera Mini",
                    "IEMobile",
                    "Mobile",
                    "Tablet"
                ];
                
                let ua_lower = user_agent.to_lowercase();
                return mobile_patterns.iter().any(|pattern| ua_lower.contains(&pattern.to_lowercase()));
            }
        }
        // If we can't get user agent, assume not mobile
        false
    }
    
    /// Check if the current platform is a mobile device
    #[cfg(not(target_arch = "wasm32"))]
    pub fn is_mobile_device() -> bool {
        // In native environment (tests), always return false
        false
    }
}

