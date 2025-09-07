#![cfg(target_arch = "wasm32")]
#![cfg(debug_assertions)]

// Platform Console Commands
// Commands for platform information and API status

use egui_dev_console::{ConsoleCommandRegistry, ConsoleCommand, ConsoleCommandResult, ConsoleOutput};
use crate::{common::{dev_log, shared_types::Theme}, dev_log_bold, engine::{platform::Platform, audio::audio_error::AudioError}};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsCast;

/// Register all platform commands into the console registry
pub fn register_platform_commands(registry: &mut ConsoleCommandRegistry) {
    registry.register(Box::new(ApiStatusCommand));
    registry.register(Box::new(ThemeCommand));
    registry.register(Box::new(ErrorCommand));
    registry.register(Box::new(AudioDevicesCommand));
}

// API Status Command
struct ApiStatusCommand;

impl ConsoleCommand for ApiStatusCommand {
    fn name(&self) -> &str {
        "api-status"
    }
    
    fn description(&self) -> &str {
        "Show application and API status"
    }
    
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        let mut outputs = Vec::new();
        
        // Application status
        let build_type = if cfg!(debug_assertions) { "Development" } else { "Production" };
        outputs.push(ConsoleOutput::info(format!("Application Status: {} Build Running", build_type)));
        
        // Platform information
        let platform_info = Platform::get_platform_info();
        outputs.push(ConsoleOutput::info(format!("Platform: {}", platform_info)));
        
        // Critical API status
        outputs.push(ConsoleOutput::info("Critical API Status:"));
        
        let api_statuses = Platform::get_api_status();
        for status in api_statuses {
            let status_symbol = if status.supported { "✓" } else { "✗" };
            let details = status.details.as_deref().unwrap_or("");
            
            let formatted_string = format!(
                "  {} {:<20}: {}",
                status_symbol,
                format!("{}", status.api),
                details
            );
            
            // Log to web console for debugging
            dev_log!("{}", &formatted_string);
            
            let output = if status.supported {
                ConsoleOutput::success(&formatted_string)
            } else {
                ConsoleOutput::error(&formatted_string)
            };
            outputs.push(output);
        }
        
        ConsoleCommandResult::MultipleOutputs(outputs)
    }
}

// Theme Command
struct ThemeCommand;

impl ConsoleCommand for ThemeCommand {
    fn name(&self) -> &str {
        "theme"
    }
    
    fn description(&self) -> &str {
        "Switch UI color theme (light|dark|autumn|sunset)"
    }
    
    fn execute(&self, args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        if args.is_empty() {
            // Show current theme and available options
            let current = crate::common::theme::get_current_theme().name();
            let current_colors = crate::common::theme::get_current_color_scheme();
            
            let outputs = vec![
                ConsoleOutput::info(format!("Current theme: {}", current)),
                ConsoleOutput::info(format!("Current color scheme: background={:?}, surface={:?}, text={:?}", 
                    current_colors.background, current_colors.surface, current_colors.text)),
                ConsoleOutput::info("Available themes: light, dark, autumn, sunset"),
                ConsoleOutput::info("Usage: theme <theme_name>"),
            ];
            
            return ConsoleCommandResult::MultipleOutputs(outputs);
        }
        
        let theme_name = args[0].to_lowercase();
        let new_theme = match theme_name.as_str() {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            "autumn" => Theme::Autumn,
            "sunset" => Theme::Sunset,
            _ => {
                return ConsoleCommandResult::MultipleOutputs(vec![
                    ConsoleOutput::error(format!("Unknown theme '{}'. Available themes: light, dark, autumn, sunset", theme_name))
                ]);
            }
        };
        
        // Set the new theme
        crate::common::theme::set_current_theme(new_theme);
        
        // Reapply styles - updates CSS custom properties
        crate::web::styling::update_css_variables();
        crate::common::dev_log!("CSS custom properties updated for theme: {}", theme_name);
        
        ConsoleCommandResult::MultipleOutputs(vec![
            ConsoleOutput::success(format!("Theme set to {} (CSS custom properties and WebGL components updated)", theme_name))
        ])
    }
}

// Error Command
struct ErrorCommand;

impl ConsoleCommand for ErrorCommand {
    fn name(&self) -> &str {
        "error"
    }
    
    fn description(&self) -> &str {
        "Display actual error messages used by the application (browser-unsupported|mobile-unsupported|mic-unavailable|mic-permission|browser-error)"
    }
    
    fn execute(&self, args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        if args.is_empty() {
            // Show help with available error scenarios
            let outputs = vec![
                ConsoleOutput::info("Available error scenarios:"),
                ConsoleOutput::info("  browser-unsupported  - Show browser compatibility error"),
                ConsoleOutput::info("  mobile-unsupported   - Show mobile device not supported error"),
                ConsoleOutput::info("  mic-unavailable      - Show microphone not available error"),
                ConsoleOutput::info("  mic-permission       - Show microphone permission error"),
                ConsoleOutput::info("  browser-error        - Show general browser error"),
                ConsoleOutput::info("Usage: error <scenario>"),
            ];
            
            return ConsoleCommandResult::MultipleOutputs(outputs);
        }
        
        let scenario = args[0].to_lowercase();
        match scenario.as_str() {
            "browser-unsupported" => {
                crate::web::error_message_box::show_error_with_params(&crate::common::shared_types::Error::BrowserApiNotSupported, &["required features"]);
                ConsoleCommandResult::MultipleOutputs(vec![
                    ConsoleOutput::success("Displayed browser unsupported error")
                ])
            }
            "mobile-unsupported" => {
                crate::web::error_message_box::show_error(&crate::common::shared_types::Error::MobileDeviceNotSupported);
                ConsoleCommandResult::MultipleOutputs(vec![
                    ConsoleOutput::success("Displayed mobile unsupported error")
                ])
            }
            "mic-unavailable" => {
                crate::web::error_message_box::show_error(&crate::common::shared_types::Error::MicrophoneNotAvailable);
                ConsoleCommandResult::MultipleOutputs(vec![
                    ConsoleOutput::success("Displayed microphone unavailable error")
                ])
            }
            "mic-permission" => {
                crate::web::error_message_box::show_error(&crate::common::shared_types::Error::MicrophonePermissionDenied);
                ConsoleCommandResult::MultipleOutputs(vec![
                    ConsoleOutput::success("Displayed microphone permission error")
                ])
            }
            "browser-error" => {
                crate::web::error_message_box::show_error(&crate::common::shared_types::Error::BrowserError);
                ConsoleCommandResult::MultipleOutputs(vec![
                    ConsoleOutput::success("Displayed browser error")
                ])
            }
            _ => {
                ConsoleCommandResult::MultipleOutputs(vec![
                    ConsoleOutput::error(format!("Unknown error scenario '{}'. Available scenarios: browser-unsupported, mobile-unsupported, mic-unavailable, mic-permission, browser-error", scenario))
                ])
            }
        }
    }
}

// Audio Devices Command
struct AudioDevicesCommand;

impl ConsoleCommand for AudioDevicesCommand {
    fn name(&self) -> &str {
        "audio"
    }
    
    fn description(&self) -> &str {
        "List all available audio input and output devices (enumerates devices directly, not from cache)"
    }
    
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        wasm_bindgen_futures::spawn_local(async move {    
            let result: Result<(), AudioError> = async {
                let window = web_sys::window()
                    .ok_or(AudioError::Generic("No window object".to_string()))?;
                
                let media_devices = window.navigator().media_devices()
                    .map_err(|_| AudioError::NotSupported("MediaDevices not available".to_string()))?;

                let promise = media_devices.enumerate_devices()
                    .map_err(|e| AudioError::Generic(format!("Failed to enumerate devices: {:?}", e)))?;

                let devices_js = JsFuture::from(promise).await
                    .map_err(|e| AudioError::Generic(format!("Device enumeration failed: {:?}", e)))?;
                    
                let devices = js_sys::Array::from(&devices_js);

                // Check if we have permission by looking for any device with a non-empty label
                let has_permission = (0..devices.length()).any(|i| {
                    devices.get(i)
                        .dyn_ref::<web_sys::MediaDeviceInfo>()
                        .is_some_and(|d| !d.label().is_empty())
                });

                if !has_permission && devices.length() > 0 {
                    dev_log_bold!("Found {} audio devices, but device details are unavailable", devices.length());
                    dev_log_bold!("Grant microphone access to see device names and types");
                    return Ok(());
                }

                let mut input_count = 0;
                let mut output_count = 0;

                // First pass: count devices and log input devices
                dev_log_bold!("Audio Input Devices:");
                for i in 0..devices.length() {
                    if let Some(device_info) = devices.get(i).dyn_ref::<web_sys::MediaDeviceInfo>() {
                        if device_info.kind() == web_sys::MediaDeviceKind::Audioinput {
                            input_count += 1;
                            dev_log_bold!("    {}", device_info.label());
                        }
                    }
                }

                // Second pass: log output devices
                dev_log_bold!("Audio Output Devices:");
                for i in 0..devices.length() {
                    if let Some(device_info) = devices.get(i).dyn_ref::<web_sys::MediaDeviceInfo>() {
                        if device_info.kind() == web_sys::MediaDeviceKind::Audiooutput {
                            output_count += 1;
                            dev_log_bold!("    {}", device_info.label());
                        }
                    }
                }

                dev_log_bold!("Total: {} input devices, {} output devices", input_count, output_count);
                Ok(())
            }.await;

            match result {
                Ok(()) => {},
                Err(e) => {
                    dev_log_bold!("Failed to enumerate audio devices: {:?}", e);
                }
            }
        });
        
        ConsoleCommandResult::MultipleOutputs(vec![
            ConsoleOutput::info("Enumerating audio devices..."),
            ConsoleOutput::info("Results will appear in browser console (check dev tools)"),
        ])
    }
}
