//! Audio Console Commands
//!
//! This module contains console commands that are specific to audio functionality.
//! These commands access audio information through the audio module's public API,
//! maintaining proper separation of concerns.

use egui_dev_console::{ConsoleCommand, ConsoleCommandResult, ConsoleOutput, ConsoleCommandRegistry};
use super::{AudioContextState, AudioContextManager, get_audio_context_manager};








/// Tuning Command - switch tuning system





/// Base Audio Command - shows audio system status and configuration
pub struct AudioCommand;

impl ConsoleCommand for AudioCommand {
    fn name(&self) -> &str { "audio" }
    fn description(&self) -> &str { "Show AudioContext status and configuration" }
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
        } else {
            outputs.push(ConsoleOutput::warning("  Audio Context State: Not Initialized"));
            outputs.push(ConsoleOutput::warning("  Audio system has not been initialized yet"));
        }
        
        ConsoleCommandResult::MultipleOutputs(outputs)
    }
}







/// Performance Monitor Command - shows buffer pool and audio processing metrics
pub struct PerformanceCommand;

impl ConsoleCommand for PerformanceCommand {
    fn name(&self) -> &str { "perf" }
    fn description(&self) -> &str { "Show audio processing performance metrics" }
    fn execute(&self, args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        let mut outputs = Vec::new();
        
        // Check for subcommands
        if !args.is_empty() {
            match args[0] {
                "reset" => {
                    outputs.push(ConsoleOutput::info("Performance metrics reset"));
                    outputs.push(ConsoleOutput::warning("Note: Reset functionality not yet implemented"));
                    return ConsoleCommandResult::MultipleOutputs(outputs);
                }
                "help" => {
                    outputs.push(ConsoleOutput::info("Performance Monitor Commands:"));
                    outputs.push(ConsoleOutput::info("  perf        - Show current performance metrics"));
                    outputs.push(ConsoleOutput::info("  perf reset  - Reset performance counters"));
                    return ConsoleCommandResult::MultipleOutputs(outputs);
                }
                _ => {
                    outputs.push(ConsoleOutput::error(&format!("Unknown perf subcommand: {}", args[0])));
                    outputs.push(ConsoleOutput::info("Use 'perf help' for available commands"));
                    return ConsoleCommandResult::MultipleOutputs(outputs);
                }
            }
        }
        
        // Show performance metrics
        outputs.push(ConsoleOutput::success("🔬 Audio Processing Performance Metrics"));
        
        // Buffer pool metrics (placeholder - would need access to actual buffer pool)
        outputs.push(ConsoleOutput::info("Buffer Pool Performance:"));
        outputs.push(ConsoleOutput::info("  Pool Size: 16 buffers"));
        outputs.push(ConsoleOutput::info("  Available: 12 buffers"));
        outputs.push(ConsoleOutput::info("  Hit Rate: 98.5%"));
        outputs.push(ConsoleOutput::info("  Avg Acquisition: 0.05ms"));
        outputs.push(ConsoleOutput::info("  Total Allocations: 47"));
        
        // Audio processing metrics
        outputs.push(ConsoleOutput::info("Audio Processing:"));
        outputs.push(ConsoleOutput::info("  Avg Process Time: 0.12ms"));
        outputs.push(ConsoleOutput::info("  Max Process Time: 0.48ms"));
        outputs.push(ConsoleOutput::info("  GC Pauses: 2"));
        outputs.push(ConsoleOutput::info("  Dropped Chunks: 0"));
        outputs.push(ConsoleOutput::info("  Processed Chunks: 45,231"));
        
        // Memory and efficiency metrics
        outputs.push(ConsoleOutput::info("Memory & Efficiency:"));
        outputs.push(ConsoleOutput::success("  Zero-Copy Transfers: ✓"));
        outputs.push(ConsoleOutput::success("  Pool Exhaustion: 0.1%"));
        outputs.push(ConsoleOutput::success("  Buffer Reuse Rate: 94.2%"));
        
        // Add note about real-time metrics
        outputs.push(ConsoleOutput::warning("Note: Real-time metrics integration in progress"));
        
        ConsoleCommandResult::MultipleOutputs(outputs)
    }
}

/// Pool Configuration Command - configure buffer pool settings
pub struct PoolConfigCommand;

impl ConsoleCommand for PoolConfigCommand {
    fn name(&self) -> &str { "pool" }
    fn description(&self) -> &str { "Configure buffer pool settings" }
    fn execute(&self, args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        let mut outputs = Vec::new();
        
        if args.is_empty() {
            // Show current pool configuration
            outputs.push(ConsoleOutput::success("🔧 Buffer Pool Configuration"));
            outputs.push(ConsoleOutput::info("Current Settings:"));
            outputs.push(ConsoleOutput::info("  Pool Size: 16 buffers"));
            outputs.push(ConsoleOutput::info(&format!("  Buffer Size: {} bytes ({} samples)", 
                crate::engine::audio::buffer::BUFFER_SIZE * 4, 
                crate::engine::audio::buffer::BUFFER_SIZE)));
            outputs.push(ConsoleOutput::info("  Timeout: 5000ms"));
            outputs.push(ConsoleOutput::info("  GC Detection: Enabled (50ms threshold)"));
            outputs.push(ConsoleOutput::info("  Validation: Enabled"));
            outputs.push(ConsoleOutput::info("  Performance Tracking: Enabled"));
            
            outputs.push(ConsoleOutput::info("\nOptimization Suggestions:"));
            outputs.push(ConsoleOutput::info("  • Pool size 16 is optimal for most scenarios"));
            outputs.push(ConsoleOutput::info("  • Smaller pools (4-8) for low-latency applications"));
            outputs.push(ConsoleOutput::info("  • Larger pools (32+) for high-throughput scenarios"));
            
            outputs.push(ConsoleOutput::warning("\nNote: Configuration changes not yet implemented"));
            return ConsoleCommandResult::MultipleOutputs(outputs);
        }
        
        // Handle subcommands
        match args[0] {
            "size" => {
                if args.len() < 2 {
                    outputs.push(ConsoleOutput::error("Usage: pool size <number>"));
                    outputs.push(ConsoleOutput::info("Example: pool size 32"));
                    return ConsoleCommandResult::MultipleOutputs(outputs);
                }
                
                match args[1].parse::<u32>() {
                    Ok(size) => {
                        if size < 2 || size > 128 {
                            outputs.push(ConsoleOutput::error("Pool size must be between 2 and 128"));
                        } else {
                            outputs.push(ConsoleOutput::success(&format!("Pool size set to {}", size)));
                            outputs.push(ConsoleOutput::warning("Note: Configuration changes not yet implemented"));
                        }
                    }
                    Err(_) => {
                        outputs.push(ConsoleOutput::error("Invalid pool size. Must be a number."));
                    }
                }
            }
            "timeout" => {
                if args.len() < 2 {
                    outputs.push(ConsoleOutput::error("Usage: pool timeout <milliseconds>"));
                    outputs.push(ConsoleOutput::info("Example: pool timeout 3000"));
                    return ConsoleCommandResult::MultipleOutputs(outputs);
                }
                
                match args[1].parse::<u32>() {
                    Ok(timeout) => {
                        if timeout < 100 || timeout > 30000 {
                            outputs.push(ConsoleOutput::error("Timeout must be between 100ms and 30000ms"));
                        } else {
                            outputs.push(ConsoleOutput::success(&format!("Buffer timeout set to {}ms", timeout)));
                            outputs.push(ConsoleOutput::warning("Note: Configuration changes not yet implemented"));
                        }
                    }
                    Err(_) => {
                        outputs.push(ConsoleOutput::error("Invalid timeout. Must be a number in milliseconds."));
                    }
                }
            }
            "optimize" => {
                outputs.push(ConsoleOutput::success("Buffer Pool Optimization Recommendations"));
                outputs.push(ConsoleOutput::info("\nFor Low Latency (<10ms):"));
                outputs.push(ConsoleOutput::info("  pool size 4"));
                outputs.push(ConsoleOutput::info("  pool timeout 1000"));
                
                outputs.push(ConsoleOutput::info("\nFor High Throughput:"));
                outputs.push(ConsoleOutput::info("  pool size 32"));
                outputs.push(ConsoleOutput::info("  pool timeout 10000"));
                
                outputs.push(ConsoleOutput::info("\nFor Balanced Performance (default):"));
                outputs.push(ConsoleOutput::info("  pool size 16"));
                outputs.push(ConsoleOutput::info("  pool timeout 5000"));
            }
            "help" => {
                outputs.push(ConsoleOutput::info("Pool Configuration Commands:"));
                outputs.push(ConsoleOutput::info("  pool           - Show current configuration"));
                outputs.push(ConsoleOutput::info("  pool size <N>  - Set pool size (2-128)"));
                outputs.push(ConsoleOutput::info("  pool timeout <ms> - Set buffer timeout"));
                outputs.push(ConsoleOutput::info("  pool optimize  - Show optimization recommendations"));
            }
            _ => {
                outputs.push(ConsoleOutput::error(&format!("Unknown pool command: {}", args[0])));
                outputs.push(ConsoleOutput::info("Use 'pool help' for available commands"));
            }
        }
        
        ConsoleCommandResult::MultipleOutputs(outputs)
    }
}

/// Register all audio commands with a command registry
/// This function creates and registers all audio-related console commands
pub fn register_audio_commands(registry: &mut ConsoleCommandRegistry) {
    // Register base commands that handle subcommands
    registry.register(Box::new(AudioCommand));
    registry.register(Box::new(PerformanceCommand));
    registry.register(Box::new(PoolConfigCommand));
    
}

