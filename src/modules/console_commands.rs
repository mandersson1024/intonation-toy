// Console Commands Orchestrator
// Central location for registering all non-built-in console commands
// Provides a clean separation between console infrastructure and module-specific commands
//
// Commands should not be unit tested

use crate::modules::console::{ConsoleCommandRegistry, ConsoleCommand, ConsoleCommandResult, ConsoleOutput};

use crate::modules::audio::{MicrophoneManager, AudioContextManager, get_audio_context_manager};
use crate::modules::{platform::Platform, common::dev_log};

/// Creates a fully configured console command registry with all module commands
pub fn create_console_registry() -> ConsoleCommandRegistry {
    let mut registry = ConsoleCommandRegistry::new();
    
    // Register platform module commands
    registry.register(Box::new(StatusCommand));
    
    // Register audio module commands
    registry.register(Box::new(MicStatusCommand));
    registry.register(Box::new(AudioContextCommand));
    
    registry
}

// Platform Commands Implementation
// These commands require access to platform module and are therefore not built-in

// Status Command
struct StatusCommand;

impl ConsoleCommand for StatusCommand {
    fn name(&self) -> &str {
        "status"
    }
    
    fn description(&self) -> &str {
        "Show application status"
    }
    
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        let mut outputs = Vec::new();
        
        // Application status
        let build_type = if cfg!(debug_assertions) { "Development" } else { "Production" };
        outputs.push(ConsoleOutput::info(&format!("Application Status: {} Build Running", build_type)));
        
        // Platform information
        let platform_info = Platform::get_platform_info();
        outputs.push(ConsoleOutput::info(&format!("Platform: {}", platform_info)));
        
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

// Audio Commands Implementation
// These commands require access to the audio module and are therefore not built-in

// Microphone Status Command
struct MicStatusCommand;

impl ConsoleCommand for MicStatusCommand {
    fn name(&self) -> &str {
        "mic-status"
    }
    
    fn description(&self) -> &str {
        "Show microphone status and device information"
    }
    
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        let mut outputs = Vec::new();
        
        if !MicrophoneManager::is_supported() {
            return ConsoleCommandResult::MultipleOutputs(outputs);
        }

        // Create a manager to check current state
        let manager = MicrophoneManager::new();
        let state = manager.state();
        
        let status_text = format!("  Permission Status: {}", state);
        let output = match state {
            crate::modules::audio::AudioPermission::Granted => ConsoleOutput::success(&status_text),
            crate::modules::audio::AudioPermission::Denied => ConsoleOutput::error(&status_text),
            crate::modules::audio::AudioPermission::Requesting => ConsoleOutput::warning(&status_text),
            crate::modules::audio::AudioPermission::Unavailable => ConsoleOutput::error(&status_text),
            crate::modules::audio::AudioPermission::Uninitialized => ConsoleOutput::warning(&status_text),
        };
        outputs.push(output);
        
        // Show stream info if available
        if let Some(_stream) = manager.get_stream() {
            let info = manager.stream_info();
            outputs.push(ConsoleOutput::info(&format!("  Sample Rate: {:.1} kHz", info.sample_rate / 1000.0)));
            outputs.push(ConsoleOutput::info(&format!("  Buffer Size: {} samples", info.buffer_size)));
            
            if let Some(device_label) = &info.device_label {
                outputs.push(ConsoleOutput::info(&format!("  Device: {}", device_label)));
            }
        } else {
            outputs.push(ConsoleOutput::warning("  No active stream"));
        }
        
        ConsoleCommandResult::MultipleOutputs(outputs)
    }
}



// Audio Context Command
struct AudioContextCommand;

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
            let system_output = match state {
                crate::modules::audio::AudioContextState::Running => ConsoleOutput::success(&system_status_text),
                crate::modules::audio::AudioContextState::Suspended => ConsoleOutput::warning(&system_status_text),
                crate::modules::audio::AudioContextState::Closed => ConsoleOutput::error(&system_status_text),
                crate::modules::audio::AudioContextState::Uninitialized => ConsoleOutput::warning(&system_status_text),
                crate::modules::audio::AudioContextState::Initializing => ConsoleOutput::info(&system_status_text),
                crate::modules::audio::AudioContextState::Recreating => ConsoleOutput::warning(&system_status_text),
            };
            outputs.push(system_output);
        } else {
            outputs.push(ConsoleOutput::warning("  Audio Context State: Not Initialized"));
            outputs.push(ConsoleOutput::warning("  Audio system has not been initialized yet"));
        }
        
        ConsoleCommandResult::MultipleOutputs(outputs)
    }
}


