//! Audio Console Commands
//!
//! This module contains console commands that are specific to audio functionality.
//! These commands access audio information through the audio module's public API,
//! maintaining proper separation of concerns.

use crate::console::{ConsoleCommand, ConsoleCommandResult, ConsoleOutput, ConsoleCommandRegistry};
use super::{AudioContextState, AudioContextManager, get_audio_context_manager};
use wasm_bindgen_futures;

/// Audio Context Command - shows audio system status and configuration
pub struct AudioContextCommand;

impl ConsoleCommand for AudioContextCommand {
    fn name(&self) -> &str {
        "audio-context"
    }
    
    fn description(&self) -> &str {
        "Show AudioContext status and configuration"
    }
    
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        let mut outputs = Vec::new();
        
        // Check API support
        if !AudioContextManager::is_supported() {
            outputs.push(ConsoleOutput::error("  Web Audio API not supported"));
            return ConsoleCommandResult::MultipleOutputs(outputs);
        }
        
        // Get the global audio context manager
        if let Some(manager_rc) = get_audio_context_manager() {
            let manager = manager_rc.borrow();
            let state = manager.state();
            
            // Show configuration
            let config = manager.config();
            outputs.push(ConsoleOutput::info(&format!("  Buffer Size: {} samples", config.buffer_size)));
            
            // Show context details if available
            if let Some(context) = manager.get_context() {
                outputs.push(ConsoleOutput::info(&format!("  Sample Rate: {} Hz", context.sample_rate())));
            } else {
                outputs.push(ConsoleOutput::warning("  No active context"));
            }
            
            // Show detailed system status based on context state
            let system_status_text = format!("  Audio System: {}", state);
            let system_output = match *state {
                AudioContextState::Running => ConsoleOutput::success(&system_status_text),
                AudioContextState::Suspended => ConsoleOutput::warning(&system_status_text),
                AudioContextState::Closed => ConsoleOutput::error(&system_status_text),
                AudioContextState::Uninitialized => ConsoleOutput::warning(&system_status_text),
                AudioContextState::Initializing => ConsoleOutput::info(&system_status_text),
                AudioContextState::Recreating => ConsoleOutput::warning(&system_status_text),
            };
            outputs.push(system_output);
            
            // Show device information
            let devices = manager.get_cached_devices();
            let input_count = devices.input_devices.len();
            let output_count = devices.output_devices.len();
            outputs.push(ConsoleOutput::info(&format!("  Audio Devices: {} input, {} output", input_count, output_count)));
            
            // List device details
            if !devices.input_devices.is_empty() {
                outputs.push(ConsoleOutput::info("  Input Devices:"));
                for (device_id, label) in &devices.input_devices {
                    outputs.push(ConsoleOutput::info(&format!("    • {} ({})", label, device_id)));
                }
            }
            
            if !devices.output_devices.is_empty() {
                outputs.push(ConsoleOutput::info("  Output Devices:"));
                for (device_id, label) in &devices.output_devices {
                    outputs.push(ConsoleOutput::info(&format!("    • {} ({})", label, device_id)));
                }
            }
        } else {
            outputs.push(ConsoleOutput::warning("  Audio Context State: Not Initialized"));
            outputs.push(ConsoleOutput::warning("  Audio system has not been initialized yet"));
        }
        
        ConsoleCommandResult::MultipleOutputs(outputs)
    }
}

/// Audio Device List Command - shows available audio devices
pub struct AudioDeviceListCommand;

impl ConsoleCommand for AudioDeviceListCommand {
    fn name(&self) -> &str {
        "audio-devices"
    }
    
    fn description(&self) -> &str {
        "List available audio input and output devices"
    }
    
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        let mut outputs = Vec::new();
        
        outputs.push(ConsoleOutput::info("Audio Devices:"));
        
        // Get the global audio context manager
        if let Some(manager_rc) = get_audio_context_manager() {
            let manager = manager_rc.borrow();
            let devices = manager.get_cached_devices();
            
            // Show input devices
            if devices.input_devices.is_empty() {
                outputs.push(ConsoleOutput::warning("No input devices found"));
            } else {
                outputs.push(ConsoleOutput::success(&format!("Input Devices ({}):", devices.input_devices.len())));
                for (device_id, label) in &devices.input_devices {
                    outputs.push(ConsoleOutput::info(&format!("  • {}", label)));
                    outputs.push(ConsoleOutput::info(&format!("    ID: {}", device_id)));
                }
            }
            
            // Show output devices
            if devices.output_devices.is_empty() {
                outputs.push(ConsoleOutput::warning("No output devices found"));
            } else {
                outputs.push(ConsoleOutput::success(&format!("Output Devices ({}):", devices.output_devices.len())));
                for (device_id, label) in &devices.output_devices {
                    outputs.push(ConsoleOutput::info(&format!("  • {}", label)));
                    outputs.push(ConsoleOutput::info(&format!("    ID: {}", device_id)));
                }
            }
        } else {
            outputs.push(ConsoleOutput::warning("Audio system not initialized"));
            outputs.push(ConsoleOutput::info("Audio system must be initialized to list devices"));
        }
        
        ConsoleCommandResult::MultipleOutputs(outputs)
    }
}

/// Audio Refresh Command - refreshes audio device list
pub struct AudioRefreshCommand;

impl ConsoleCommand for AudioRefreshCommand {
    fn name(&self) -> &str {
        "audio-refresh"
    }
    
    fn description(&self) -> &str {
        "Refresh the audio device list"
    }
    
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        // Refresh devices through the audio context manager
        if let Some(manager_rc) = get_audio_context_manager() {
            // Use spawn_local to handle the async refresh
            wasm_bindgen_futures::spawn_local(async move {
                match manager_rc.try_borrow_mut() {
                    Ok(mut manager) => {
                        if let Err(e) = manager.refresh_audio_devices().await {
                            web_sys::console::error_1(&format!("Device refresh failed: {:?}", e).into());
                        } else {
                            web_sys::console::log_1(&"Device refresh completed successfully".into());
                        }
                    }
                    Err(_) => {
                        web_sys::console::warn_1(&"AudioContextManager busy, skipping device refresh".into());
                    }
                }
            });
            
            ConsoleCommandResult::Output(ConsoleOutput::success("Audio device refresh initiated"))
        } else {
            ConsoleCommandResult::Output(ConsoleOutput::error("Audio system not initialized"))
        }
    }
}

/// Register all audio commands with a command registry
/// This function creates and registers all audio-related console commands
pub fn register_audio_commands(registry: &mut ConsoleCommandRegistry) {
    registry.register(Box::new(AudioContextCommand));
    registry.register(Box::new(AudioDeviceListCommand));
    registry.register(Box::new(AudioRefreshCommand));
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audio_context_command() {
        let command = AudioContextCommand;
        
        assert_eq!(command.name(), "audio-context");
        assert_eq!(command.description(), "Show AudioContext status and configuration");
    }
    
    #[test]
    fn test_audio_device_list_command() {
        let command = AudioDeviceListCommand;
        
        assert_eq!(command.name(), "audio-devices");
        assert_eq!(command.description(), "List available audio input and output devices");
    }
    
    #[test]
    fn test_audio_refresh_command() {
        let command = AudioRefreshCommand;
        
        assert_eq!(command.name(), "audio-refresh");
        assert_eq!(command.description(), "Refresh the audio device list");
    }
}