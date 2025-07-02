// Platform detection and feature support module
// Centralizes browser API availability checks and platform-specific logic

use crate::modules::common::dev_log;
use crate::modules::audio::MicrophoneManager;

/// Platform feature validation results
#[derive(Debug, Clone, PartialEq)]
pub enum PlatformValidationResult {
    AllSupported,
    // TODO: For semantic clarity we could implement the type MissingApi, but as long
    // as the implementation is identical to CriticalApi we keep it simple and use that.
    MissingCriticalApis(Vec<CriticalApi>),
}

/// Critical APIs that must be available for application startup
#[derive(Debug, Clone, PartialEq)]
pub enum CriticalApi {
    WebAudioApi,
    GetUserMedia,
    AudioWorklet,
    Canvas,
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
        ]
    }

    /// Get detailed status of all critical APIs
    pub fn get_api_status() -> Vec<ApiStatus> {
        Self::get_critical_apis()
            .into_iter()
            .map(|api| Self::check_api_status(api))
            .collect()
    }

    /// Check status of a specific API with details
    pub fn check_api_status(api: CriticalApi) -> ApiStatus {
        match api {
            CriticalApi::WebAudioApi => Self::check_webaudio_status(),
            CriticalApi::GetUserMedia => Self::check_getusermedia_status(),
            CriticalApi::AudioWorklet => Self::check_audioworklet_status(),
            CriticalApi::Canvas => Self::check_canvas_status(),
        }
    }

    /// Validate all critical platform features required for application startup
    /// Returns validation result that caller MUST handle - application should not start if APIs are missing
    pub fn check_feature_support() -> PlatformValidationResult {
        dev_log!("Validating critical platform APIs...");
        
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
    
    /// Check Web Audio API support status
    fn check_webaudio_status() -> ApiStatus {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let audio_context_available = js_sys::Reflect::has(&window, &"AudioContext".into()).unwrap_or(false);
                if audio_context_available {
                    ApiStatus {
                        api: CriticalApi::WebAudioApi,
                        supported: true,
                        details: Some("Web Audio API available".to_string()),
                    }
                } else {
                    ApiStatus {
                        api: CriticalApi::WebAudioApi,
                        supported: false,
                        details: Some("AudioContext not found".to_string()),
                    }
                }
            } else {
                ApiStatus {
                    api: CriticalApi::WebAudioApi,
                    supported: false,
                    details: Some("Window object not available".to_string()),
                }
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        ApiStatus {
            api: CriticalApi::WebAudioApi,
            supported: false,
            details: Some("Not running in browser environment".to_string()),
        }
    }

    /// Check getUserMedia API support status
    fn check_getusermedia_status() -> ApiStatus {
        let is_supported = MicrophoneManager::is_supported();
        
        ApiStatus {
            api: CriticalApi::GetUserMedia,
            supported: is_supported,
            details: if is_supported {
                Some("getUserMedia API available".to_string())
            } else {
                Some("getUserMedia API not available".to_string())
            },
        }
    }

    /// Check AudioWorklet support status
    fn check_audioworklet_status() -> ApiStatus {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let navigator = window.navigator();
                if let Ok(media_devices) = navigator.media_devices() {
                    // AudioWorklet is available if we can access the AudioContext constructor
                    let has_audioworklet = js_sys::Reflect::has(&window, &"audioWorklet".into()).unwrap_or(false);
                    
                    ApiStatus {
                        api: CriticalApi::AudioWorklet,
                        supported: has_audioworklet,
                        details: if has_audioworklet {
                            Some("AudioWorklet API available".to_string())
                        } else {
                            Some("AudioWorklet not supported".to_string())
                        },
                    }
                } else {
                    ApiStatus {
                        api: CriticalApi::AudioWorklet,
                        supported: false,
                        details: Some("MediaDevices not available".to_string()),
                    }
                }
            } else {
                ApiStatus {
                    api: CriticalApi::AudioWorklet,
                    supported: false,
                    details: Some("Window object not available".to_string()),
                }
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        ApiStatus {
            api: CriticalApi::AudioWorklet,
            supported: false,
            details: Some("Not running in browser environment".to_string()),
        }
    }

    /// Check Canvas support status
    fn check_canvas_status() -> ApiStatus {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    let has_canvas = document.create_element("canvas").is_ok();
                    
                    ApiStatus {
                        api: CriticalApi::Canvas,
                        supported: has_canvas,
                        details: if has_canvas {
                            Some("HTML5 Canvas API available".to_string())
                        } else {
                            Some("Canvas element not supported".to_string())
                        },
                    }
                } else {
                    ApiStatus {
                        api: CriticalApi::Canvas,
                        supported: false,
                        details: Some("Document object not available".to_string()),
                    }
                }
            } else {
                ApiStatus {
                    api: CriticalApi::Canvas,
                    supported: false,
                    details: Some("Window object not available".to_string()),
                }
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        ApiStatus {
            api: CriticalApi::Canvas,
            supported: false,
            details: Some("Not running in browser environment".to_string()),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_info_generation() {
        // Since this involves browser APIs, we test the fallback behavior in native tests
        // The actual browser functionality will be tested in future WASM integration tests
        let info = Platform::get_platform_info();
        assert!(info.starts_with("UserAgent:"));
        // In native test environment, this should return "UserAgent: Unknown"
        assert_eq!(info, "UserAgent: Unknown");
    }

    #[test]
    fn test_platform_validation_result_types() {
        // Test that our validation result types work correctly
        let all_supported = PlatformValidationResult::AllSupported;
        assert!(matches!(all_supported, PlatformValidationResult::AllSupported));

        let missing = PlatformValidationResult::MissingCriticalApis(vec![CriticalApi::WebAudioApi]);
        match missing {
            PlatformValidationResult::MissingCriticalApis(apis) => {
                assert_eq!(apis.len(), 1);
                assert_eq!(apis[0], CriticalApi::WebAudioApi);
            }
            _ => panic!("Expected MissingCriticalApis variant"),
        }
    }

}