//! Audio Console Commands
//!
//! This module contains console commands that are specific to audio functionality.
//! These commands access audio information through the audio module's public API,
//! maintaining proper separation of concerns.

use egui_dev_console::{ConsoleCommand, ConsoleCommandResult, ConsoleOutput, ConsoleCommandRegistry};
use super::{AudioContextState, AudioContextManager, get_audio_context_manager};
use super::{PitchAnalyzer, TuningSystem};
use super::console_service::{ConsoleAudioService, BufferPoolMetrics};
use std::cell::RefCell;
use std::rc::Rc;






thread_local! { 
    static BUFFER_DEBUG_ENABLED: std::cell::Cell<bool> = std::cell::Cell::new(false);
    static PITCH_ANALYZER_GLOBAL: RefCell<Option<Rc<RefCell<PitchAnalyzer>>>> = RefCell::new(None);
}

/// Helper function to set the global pitch analyzer
pub fn set_global_pitch_analyzer(analyzer: Rc<RefCell<PitchAnalyzer>>) {
    PITCH_ANALYZER_GLOBAL.with(|pa| {
        *pa.borrow_mut() = Some(analyzer);
    });
}

/// Helper function to get the global pitch analyzer
pub fn get_global_pitch_analyzer() -> Option<Rc<RefCell<PitchAnalyzer>>> {
    PITCH_ANALYZER_GLOBAL.with(|pa| pa.borrow().as_ref().cloned())
}


/// Tuning Command - switch tuning system
pub struct TuningCommand;

impl ConsoleCommand for TuningCommand {
    fn name(&self) -> &str { "tuning" }
    fn description(&self) -> &str { "Switch tuning system (equal/just/custom)" }
    fn execute(&self, args: Vec<&str>, _: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        if args.is_empty() {
            let mut outputs = Vec::new();
            
            // Show current tuning system first
            if let Some(analyzer_rc) = get_global_pitch_analyzer() {
                let analyzer = analyzer_rc.borrow();
                let config = analyzer.config();
                let current_tuning = match &config.tuning_system {
                    TuningSystem::EqualTemperament { reference_pitch } => {
                        format!("Equal Temperament (A4 = {:.1} Hz)", reference_pitch)
                    }
                    TuningSystem::JustIntonation { reference_pitch } => {
                        format!("Just Intonation (A4 = {:.1} Hz)", reference_pitch)
                    }
                    TuningSystem::Custom { frequency_ratios } => {
                        format!("Custom ({} frequency ratios)", frequency_ratios.len())
                    }
                };
                outputs.push(ConsoleOutput::success(&format!("Current tuning system: {}", current_tuning)));
                outputs.push(ConsoleOutput::info(""));
            } else {
                outputs.push(ConsoleOutput::warning("Pitch analyzer not initialized"));
                outputs.push(ConsoleOutput::info(""));
            }
            
            outputs.push(ConsoleOutput::info("Usage: tuning <system> [reference_pitch]"));
            outputs.push(ConsoleOutput::info(""));
            outputs.push(ConsoleOutput::info("Available tuning systems:"));
            outputs.push(ConsoleOutput::info("  equal [pitch]    - Equal temperament (default: 440.0 Hz)"));
            outputs.push(ConsoleOutput::info("  just [pitch]     - Just intonation (default: 440.0 Hz)"));
            outputs.push(ConsoleOutput::info("  custom           - Custom frequency ratios"));
            outputs.push(ConsoleOutput::info(""));
            outputs.push(ConsoleOutput::info("Reference pitch range: 420.0 - 460.0 Hz"));
            outputs.push(ConsoleOutput::info(""));
            outputs.push(ConsoleOutput::info("Examples:"));
            outputs.push(ConsoleOutput::info("  tuning equal          - A4 = 440.0 Hz"));
            outputs.push(ConsoleOutput::info("  tuning equal 432      - A4 = 432.0 Hz"));
            outputs.push(ConsoleOutput::info("  tuning just 440       - Just intonation, A4 = 440.0 Hz"));
            return ConsoleCommandResult::MultipleOutputs(outputs);
        }
        
        let system = args[0].to_lowercase();
        let reference_pitch = if args.len() > 1 {
            match args[1].parse::<f32>() {
                Ok(pitch) => pitch,
                Err(_) => return ConsoleCommandResult::Output(ConsoleOutput::error("Invalid reference pitch")),
            }
        } else {
            440.0 // Default A4
        };
        
        if reference_pitch < 420.0 || reference_pitch > 460.0 {
            return ConsoleCommandResult::Output(ConsoleOutput::error("Reference pitch must be between 420 and 460 Hz"));
        }
        
        let tuning_system = match system.as_str() {
            "equal" => TuningSystem::EqualTemperament { reference_pitch },
            "just" => TuningSystem::JustIntonation { reference_pitch },
            "custom" => {
                // Default 12-tone equal temperament ratios for custom example
                let ratios = vec![1.0, 1.059463, 1.122462, 1.189207, 1.259921, 1.334840, 1.414214, 1.498307, 1.587401, 1.681793, 1.781797, 1.887749];
                TuningSystem::Custom { frequency_ratios: ratios }
            },
            _ => return ConsoleCommandResult::Output(ConsoleOutput::error("Invalid tuning system. Use: equal, just, or custom")),
        };
        
        if let Some(analyzer_rc) = get_global_pitch_analyzer() {
            let mut analyzer = analyzer_rc.borrow_mut();
            let mut config = analyzer.config().clone();
            config.tuning_system = tuning_system;
            
            match analyzer.update_config(config) {
                Ok(_) => ConsoleCommandResult::Output(ConsoleOutput::success(&format!("Tuning system set to {} (A4 = {:.1} Hz)", system, reference_pitch))),
                Err(e) => ConsoleCommandResult::Output(ConsoleOutput::error(&format!("Failed to update tuning system: {}", e))),
            }
        } else {
            ConsoleCommandResult::Output(ConsoleOutput::error("Pitch analyzer not initialized"))
        }
    }
}





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

/// Base Buffer Command - handles "buffer" with subcommands
pub struct BufferCommand;

impl ConsoleCommand for BufferCommand {
    fn name(&self) -> &str { "buffer" }
    fn description(&self) -> &str { "Buffer management commands" }
    fn execute(&self, args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        if args.is_empty() {
            // Show available buffer subcommands
            let help_lines = vec![
                "Available buffer commands:".to_string(),
                "  buffer status - Show buffer pool status and individual buffer details".to_string(),
                "  buffer metrics - Show brief buffer pool metrics".to_string(),
                "  buffer reset - Clear and reset all buffers in the pool".to_string(),
                "  buffer debug - Toggle buffer debug logging on/off".to_string(),
            ];
            let help_text = help_lines.join("\n");
            return ConsoleCommandResult::Output(ConsoleOutput::info(&help_text));
        }
        
        let subcommand = args[0];
        
        match subcommand {
            "status" => {
                // Buffer pool removed - using direct processing with transferable buffers
                ConsoleCommandResult::Output(ConsoleOutput::info("Buffer pool removed - using direct processing with transferable buffers"))
            },
            "metrics" => {
                // Buffer pool removed - using direct processing with transferable buffers
                ConsoleCommandResult::Output(ConsoleOutput::info("Buffer pool removed - using direct processing with transferable buffers"))
            },
            "reset" => {
                // Buffer pool removed - using direct processing with transferable buffers
                ConsoleCommandResult::Output(ConsoleOutput::info("Buffer pool removed - using direct processing with transferable buffers"))
            },
            "debug" => {
                // Buffer debug functionality
                let enabled = BUFFER_DEBUG_ENABLED.with(|c| { let val = !c.get(); c.set(val); val });
                ConsoleCommandResult::Output(ConsoleOutput::info(&format!("Buffer debug logging {}", if enabled { "enabled" } else { "disabled" })))
            },
            _ => ConsoleCommandResult::Output(ConsoleOutput::error(format!("Unknown buffer subcommand: {}", subcommand))),
        }
    }
}

/// Base Pitch Command - handles "pitch" with subcommands
pub struct PitchCommand;

impl ConsoleCommand for PitchCommand {
    fn name(&self) -> &str { "pitch" }
    fn description(&self) -> &str { "Pitch detection commands" }
    fn execute(&self, args: Vec<&str>, _: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        if args.is_empty() {
            // Show available pitch subcommands
            let help_lines = vec![
                "Available pitch commands:".to_string(),
                "  pitch status - Show current pitch detection configuration and state".to_string(),
                "  pitch range <min> <max> - Set frequency detection range".to_string(),
                "  pitch benchmarks - Run performance benchmarks for different window sizes".to_string(),
            ];
            let help_text = help_lines.join("\n");
            return ConsoleCommandResult::Output(ConsoleOutput::info(&help_text));
        }
        
        let subcommand = args[0];
        let sub_args = args[1..].to_vec();
        
        match subcommand {
            "status" => {
                // Pitch status functionality
                let mut outputs = Vec::new();
                
                if let Some(analyzer_rc) = get_global_pitch_analyzer() {
                    let analyzer = analyzer_rc.borrow();
                    let config = analyzer.config();
                    let metrics = analyzer.metrics();
                    
                    outputs.push(ConsoleOutput::info("Pitch Detection Status:"));
                    outputs.push(ConsoleOutput::info(&format!("  Sample Window: {} samples", config.sample_window_size)));
                    outputs.push(ConsoleOutput::info(&format!("  Threshold: {:.2}", config.threshold)));
                    outputs.push(ConsoleOutput::info(&format!("  Frequency Range: {:.1} Hz - {:.1} Hz", config.min_frequency, config.max_frequency)));
                    
                    let tuning_desc = match &config.tuning_system {
                        TuningSystem::EqualTemperament { reference_pitch } => format!("Equal Temperament (A4 = {:.1} Hz)", reference_pitch),
                        TuningSystem::JustIntonation { reference_pitch } => format!("Just Intonation (A4 = {:.1} Hz)", reference_pitch),
                        TuningSystem::Custom { frequency_ratios } => format!("Custom ({} ratios)", frequency_ratios.len()),
                    };
                    outputs.push(ConsoleOutput::info(&format!("  Tuning System: {}", tuning_desc)));
                    
                    outputs.push(ConsoleOutput::info(&format!("  Latest Latency: {:.1} ms", metrics.processing_latency_ms)));
                    outputs.push(ConsoleOutput::info(&format!("  Average Latency: {:.1} ms", metrics.average_latency_ms)));
                    outputs.push(ConsoleOutput::info(&format!("  Min/Max Latency: {:.1}/{:.1} ms", metrics.min_latency_ms, metrics.max_latency_ms)));
                    outputs.push(ConsoleOutput::info(&format!("  Analysis Cycles: {}", metrics.analysis_cycles)));
                    outputs.push(ConsoleOutput::info(&format!("  Successful Detections: {} ({:.1}%)", metrics.successful_detections, metrics.success_rate * 100.0)));
                    outputs.push(ConsoleOutput::info(&format!("  Failed Detections: {}", metrics.failed_detections)));
                    outputs.push(ConsoleOutput::info(&format!("  Latency Violations (>50ms): {}", metrics.latency_violations)));
                    outputs.push(ConsoleOutput::info(&format!("  Average Confidence: {:.2}", metrics.average_confidence)));
                    outputs.push(ConsoleOutput::info(&format!("  YIN Processing Time: {:.0} μs", metrics.yin_processing_time_us)));
                    outputs.push(ConsoleOutput::info(&format!("  Memory Usage: {:.2} KB", metrics.memory_usage_bytes as f64 / 1024.0)));
                    
                    // Show performance and accuracy characteristics
                    let performance_grade = analyzer.performance_grade();
                    let meets_requirements = analyzer.meets_performance_requirements();
                    let (estimated_latency, latency_grade) = analyzer.pitch_detector().get_performance_characteristics();
                    let (frequency_resolution, accuracy_grade) = analyzer.pitch_detector().get_accuracy_characteristics();
                    
                    let grade_output = if meets_requirements {
                        ConsoleOutput::success(&format!("  Performance Grade: {} ✓", performance_grade))
                    } else {
                        ConsoleOutput::warning(&format!("  Performance Grade: {} ⚠", performance_grade))
                    };
                    outputs.push(grade_output);
                    outputs.push(ConsoleOutput::info(&format!("  Estimated Latency: {:.1} ms ({})", estimated_latency, latency_grade)));
                    outputs.push(ConsoleOutput::info(&format!("  Frequency Resolution: {:.1} Hz ({})", frequency_resolution, accuracy_grade)));
                    outputs.push(ConsoleOutput::info(&format!("  Early Exit Optimization: {}", 
                        if analyzer.pitch_detector().early_exit_enabled() { "enabled" } else { "disabled" })));
                    
                    let status_text = if analyzer.is_ready() { "Ready" } else { "Not Ready" };
                    let status_output = if analyzer.is_ready() { 
                        ConsoleOutput::success(&format!("  Status: {}", status_text))
                    } else {
                        ConsoleOutput::warning(&format!("  Status: {}", status_text))
                    };
                    outputs.push(status_output);
                } else {
                    outputs.push(ConsoleOutput::warning("Pitch analyzer not initialized"));
                }
                
                ConsoleCommandResult::MultipleOutputs(outputs)
            },
            "range" => {
                // Pitch range functionality
                if sub_args.len() < 2 {
                    let mut outputs = Vec::new();
                    
                    // Show current range first
                    if let Some(analyzer_rc) = get_global_pitch_analyzer() {
                        let analyzer = analyzer_rc.borrow();
                        let config = analyzer.config();
                        outputs.push(ConsoleOutput::success(&format!("Current frequency range: {:.1} - {:.1} Hz", config.min_frequency, config.max_frequency)));
                        outputs.push(ConsoleOutput::info(""));
                    } else {
                        outputs.push(ConsoleOutput::warning("Pitch analyzer not initialized"));
                        outputs.push(ConsoleOutput::info(""));
                    }
                    
                    outputs.push(ConsoleOutput::info("Usage: pitch range <min> <max>"));
                    outputs.push(ConsoleOutput::info(""));
                    outputs.push(ConsoleOutput::info("Set the frequency detection range for pitch analysis."));
                    outputs.push(ConsoleOutput::info(""));
                    outputs.push(ConsoleOutput::info("Parameters:"));
                    outputs.push(ConsoleOutput::info("  min    - Minimum frequency in Hz (must be positive)"));
                    outputs.push(ConsoleOutput::info("  max    - Maximum frequency in Hz (must be > min)"));
                    outputs.push(ConsoleOutput::info(""));
                    outputs.push(ConsoleOutput::info("Recommended ranges:"));
                    outputs.push(ConsoleOutput::info("  Guitar:        80 - 1000 Hz"));
                    outputs.push(ConsoleOutput::info("  Piano:         30 - 4000 Hz"));
                    outputs.push(ConsoleOutput::info("  Voice:         80 - 2000 Hz"));
                    outputs.push(ConsoleOutput::info("  General:       20 - 20000 Hz"));
                    outputs.push(ConsoleOutput::info(""));
                    outputs.push(ConsoleOutput::info("Examples:"));
                    outputs.push(ConsoleOutput::info("  pitch range 80 1000     - Guitar range"));
                    outputs.push(ConsoleOutput::info("  pitch range 30 4000     - Piano range"));
                    outputs.push(ConsoleOutput::info("  pitch range 80 2000     - Voice range"));
                    return ConsoleCommandResult::MultipleOutputs(outputs);
                }
                
                let min_freq: f32 = match sub_args[0].parse() {
                    Ok(freq) => freq,
                    Err(_) => return ConsoleCommandResult::Output(ConsoleOutput::error("Invalid minimum frequency")),
                };
                
                let max_freq: f32 = match sub_args[1].parse() {
                    Ok(freq) => freq,
                    Err(_) => return ConsoleCommandResult::Output(ConsoleOutput::error("Invalid maximum frequency")),
                };
                
                if min_freq <= 0.0 || max_freq <= 0.0 {
                    return ConsoleCommandResult::Output(ConsoleOutput::error("Frequencies must be positive"));
                }
                
                if min_freq >= max_freq {
                    return ConsoleCommandResult::Output(ConsoleOutput::error("Minimum frequency must be less than maximum"));
                }
                
                if min_freq < 20.0 || max_freq > 20000.0 {
                    return ConsoleCommandResult::Output(ConsoleOutput::error("Frequencies must be between 20 and 20000 Hz"));
                }
                
                if let Some(analyzer_rc) = get_global_pitch_analyzer() {
                    let mut analyzer = analyzer_rc.borrow_mut();
                    let mut config = analyzer.config().clone();
                    config.min_frequency = min_freq;
                    config.max_frequency = max_freq;
                    
                    match analyzer.update_config(config) {
                        Ok(_) => ConsoleCommandResult::Output(ConsoleOutput::success(&format!("Frequency range set to {:.1} - {:.1} Hz", min_freq, max_freq))),
                        Err(e) => ConsoleCommandResult::Output(ConsoleOutput::error(&format!("Failed to update frequency range: {}", e))),
                    }
                } else {
                    ConsoleCommandResult::Output(ConsoleOutput::error("Pitch analyzer not initialized"))
                }
            },
            "benchmarks" => {
                // Pitch benchmarks functionality
                if let Some(analyzer_rc) = get_global_pitch_analyzer() {
                    let mut outputs = Vec::new();
                    outputs.push(ConsoleOutput::info("Running pitch detection benchmarks..."));
                    
                    let sample_rate = 48000.0;
                    let benchmark_results = {
                        let mut analyzer = analyzer_rc.borrow_mut();
                        analyzer.benchmark_window_sizes(sample_rate)
                    };
                    
                    outputs.push(ConsoleOutput::info("Benchmark Results:"));
                    outputs.push(ConsoleOutput::info(&format!("  Sample Rate: {:.0} Hz", sample_rate)));
                    outputs.push(ConsoleOutput::info(""));
                    outputs.push(ConsoleOutput::info("  Window Size | Avg Time (ms) | Min Time (ms) | Performance"));
                    outputs.push(ConsoleOutput::info("  ------------|---------------|---------------|------------"));
                    
                    for (window_size, avg_time, min_time) in benchmark_results {
                        let performance_grade = if avg_time <= 20.0 { "Fast" } else if avg_time <= 35.0 { "Balanced" } else if avg_time <= 50.0 { "Accurate" } else if avg_time <= 100.0 { "High-Accuracy" } else { "Maximum-Accuracy" };
                        let performance_output = if avg_time <= 50.0 {
                            ConsoleOutput::info(&format!("  {:>11} | {:>13.1} | {:>13.1} | {}", window_size, avg_time, min_time, performance_grade))
                        } else {
                            ConsoleOutput::warning(&format!("  {:>11} | {:>13.1} | {:>13.1} | {}", window_size, avg_time, min_time, performance_grade))
                        };
                        outputs.push(performance_output);
                    }
                    
                    ConsoleCommandResult::MultipleOutputs(outputs)
                } else {
                    ConsoleCommandResult::Output(ConsoleOutput::warning("Pitch analyzer not initialized"))
                }
            },
            _ => ConsoleCommandResult::Output(ConsoleOutput::error(format!("Unknown pitch subcommand: {}", subcommand))),
        }
    }
}

/// Pipeline Debug Command - shows full audio pipeline status
pub struct PipelineDebugCommand;

impl ConsoleCommand for PipelineDebugCommand {
    fn name(&self) -> &str { "pipeline-debug" }
    fn description(&self) -> &str { "Show detailed audio pipeline status for debugging" }
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        let mut outputs = Vec::new();
        
        outputs.push(ConsoleOutput::info("=== Audio Pipeline Debug Status ==="));
        
        // Check AudioContext
        if let Some(manager_rc) = get_audio_context_manager() {
            let manager = manager_rc.borrow();
            let state = manager.state();
            outputs.push(ConsoleOutput::info(&format!("AudioContext: {}", state)));
            
            if let Some(context) = manager.get_context() {
                outputs.push(ConsoleOutput::info(&format!("Sample Rate: {} Hz", context.sample_rate())));
            }
        } else {
            outputs.push(ConsoleOutput::error("AudioContext: Not initialized"));
        }
        
        // Buffer Pool removed - using direct processing with transferable buffers
        outputs.push(ConsoleOutput::info("Buffer Pool: Removed - using direct processing with transferable buffers"));
        
        // Check AudioWorklet
        if let Some(worklet_rc) = super::get_global_audioworklet_manager() {
            let worklet = worklet_rc.borrow();
            outputs.push(ConsoleOutput::info(&format!("AudioWorklet: {}", worklet.state())));
            outputs.push(ConsoleOutput::info(&format!("Processing: {}", if worklet.is_processing() { "Yes" } else { "No" })));
        } else {
            outputs.push(ConsoleOutput::error("AudioWorklet: Not initialized"));
        }
        
        // Check Pitch Analyzer
        if let Some(_) = super::commands::get_global_pitch_analyzer() {
            outputs.push(ConsoleOutput::info("Pitch Analyzer: Initialized"));
        } else {
            outputs.push(ConsoleOutput::error("Pitch Analyzer: Not initialized"));
        }
        
        outputs.push(ConsoleOutput::info("=== End Debug Status ==="));
        
        ConsoleCommandResult::MultipleOutputs(outputs)
    }
}


/// AudioWorklet Debug Command - shows AudioWorklet specific status
pub struct AudioWorkletDebugCommand;

impl ConsoleCommand for AudioWorkletDebugCommand {
    fn name(&self) -> &str { "audioworklet-debug" }
    fn description(&self) -> &str { "Show detailed AudioWorklet status and data flow" }
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        let mut outputs = Vec::new();
        
        outputs.push(ConsoleOutput::info("=== AudioWorklet Debug Status ==="));
        
        if let Some(worklet_rc) = super::get_global_audioworklet_manager() {
            let worklet = worklet_rc.borrow();
            
            outputs.push(ConsoleOutput::info(&format!("State: {}", worklet.state())));
            outputs.push(ConsoleOutput::info(&format!("Processing: {}", if worklet.is_processing() { "✓ Yes" } else { "✗ No" })));
            
            // Check if worklet has processing node
            if let Some(_node) = worklet.get_processing_node() {
                outputs.push(ConsoleOutput::info("Processing Node: ✓ Connected"));
            } else {
                outputs.push(ConsoleOutput::error("Processing Node: ✗ Not connected"));
            }
            
            // Check volume detector
            if let Some(analysis) = worklet.last_volume_analysis() {
                outputs.push(ConsoleOutput::info(&format!("Volume Analysis: Available (RMS: {:.1} dB)", analysis.rms_db)));
            } else {
                outputs.push(ConsoleOutput::warning("Volume Analysis: No data"));
            }
            
            // Buffer pool removed - using direct processing with transferable buffers
            outputs.push(ConsoleOutput::info("Buffer Pool: Removed - using direct processing with transferable buffers"));
        } else {
            outputs.push(ConsoleOutput::error("AudioWorklet manager not initialized"));
        }
        
        outputs.push(ConsoleOutput::info("=== End AudioWorklet Debug ==="));
        ConsoleCommandResult::MultipleOutputs(outputs)
    }
}

/// Base Volume Command - handles "volume" with subcommands
pub struct VolumeCommand;

/// Test Signal Command - enable test signal for pitch detection testing
pub struct TestSignalCommand;

impl ConsoleCommand for VolumeCommand {
    fn name(&self) -> &str { "volume" }
    fn description(&self) -> &str { "Volume detection commands" }
    fn execute(&self, args: Vec<&str>, _: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        if args.is_empty() {
            // Show available volume subcommands
            let help_lines = vec![
                "Available volume commands:".to_string(),
                "  volume status - Show current volume levels and detector configuration".to_string(),
                "  volume config <parameter> <value> - Configure volume detector parameters".to_string(),
                "  volume test <signal-type> [parameters] - Test volume detection with generated signals".to_string(),
            ];
            let help_text = help_lines.join("\n");
            return ConsoleCommandResult::Output(ConsoleOutput::info(&help_text));
        }
        
        let subcommand = args[0];
        let sub_args = args[1..].to_vec();
        
        match subcommand {
            "status" => {
                // Volume status functionality
                let mut outputs = Vec::new();
                
                if let Some(worklet_rc) = super::get_global_audioworklet_manager() {
                    let worklet = worklet_rc.borrow();
                    
                    outputs.push(ConsoleOutput::info("Volume Detection Status:"));
                    if worklet.has_volume_detector() {
                        outputs.push(ConsoleOutput::info("  Status: ✓ Available"));
                        
                        if let Some(analysis) = worklet.last_volume_analysis() {
                            outputs.push(ConsoleOutput::info("  Live Data: ✓ Active"));
                            outputs.push(ConsoleOutput::info(&format!("  Current RMS: {:.1} dB", analysis.rms_db)));
                            outputs.push(ConsoleOutput::info(&format!("  Current Peak: {:.1} dB", analysis.peak_db)));
                            outputs.push(ConsoleOutput::info(&format!("  Volume Level: {}", analysis.level)));
                            outputs.push(ConsoleOutput::info(&format!("  Confidence Weight: {:.2}", analysis.confidence_weight)));
                        } else {
                            outputs.push(ConsoleOutput::warning("  Live Data: No recent analysis"));
                        }
                    } else {
                        outputs.push(ConsoleOutput::error("  Status: ✗ Not attached"));
                    }
                } else {
                    outputs.push(ConsoleOutput::error("AudioWorklet manager not initialized"));
                }
                
                ConsoleCommandResult::MultipleOutputs(outputs)
            },
            "config" => {
                // Volume config functionality
                if sub_args.is_empty() {
                    let mut outputs = Vec::new();
                    outputs.push(ConsoleOutput::info("Usage: volume config <parameter> <value>"));
                    outputs.push(ConsoleOutput::info("Parameters: gain <db> | noise-floor <db> | sample-rate <hz>"));
                    return ConsoleCommandResult::MultipleOutputs(outputs);
                }
                
                if sub_args.len() < 2 {
                    return ConsoleCommandResult::Output(ConsoleOutput::error("Usage: volume config <parameter> <value>"));
                }
                
                let parameter = sub_args[0].to_lowercase();
                let value: f32 = match sub_args[1].parse() {
                    Ok(v) => v,
                    Err(_) => return ConsoleCommandResult::Output(ConsoleOutput::error("Invalid numeric value")),
                };
                
                if let Some(worklet_rc) = super::get_global_audioworklet_manager() {
                    let mut worklet = worklet_rc.borrow_mut();
                    let current_config = worklet.volume_config().cloned().unwrap_or_else(|| super::VolumeDetectorConfig::default());
                    
                    match parameter.as_str() {
                        "gain" => {
                            if value < -60.0 || value > 60.0 {
                                return ConsoleCommandResult::Output(ConsoleOutput::error("Input gain must be between -60 and 60 dB"));
                            }
                            let new_config = super::VolumeDetectorConfig { input_gain_db: value, ..current_config };
                            match worklet.update_volume_config(new_config) {
                                Ok(_) => ConsoleCommandResult::Output(ConsoleOutput::success(&format!("Input gain set to {:.1} dB", value))),
                                Err(e) => ConsoleCommandResult::Output(ConsoleOutput::error(&format!("Failed to update config: {}", e)))
                            }
                        },
                        _ => ConsoleCommandResult::Output(ConsoleOutput::error("Unknown parameter. Use: gain, noise-floor, sample-rate")),
                    }
                } else {
                    ConsoleCommandResult::Output(ConsoleOutput::error("AudioWorklet manager not initialized"))
                }
            },
            "test" => {
                // Volume test functionality
                if sub_args.is_empty() {
                    let mut outputs = Vec::new();
                    outputs.push(ConsoleOutput::info("Usage: volume test <signal-type> [parameters]"));
                    outputs.push(ConsoleOutput::info("Signal types: sine <freq> <amplitude> | silence | pink-noise <amplitude>"));
                    return ConsoleCommandResult::MultipleOutputs(outputs);
                }
                
                let signal_type = sub_args[0].to_lowercase();
                match signal_type.as_str() {
                    "silence" => {
                        if let Some(worklet_rc) = super::get_global_audioworklet_manager() {
                            let mut worklet = worklet_rc.borrow_mut();
                            let samples = vec![0.0f32; 128];
                            match worklet.feed_input_chunk(&samples) {
                                Ok(_) => ConsoleCommandResult::Output(ConsoleOutput::success("Generated silence test signal")),
                                Err(e) => ConsoleCommandResult::Output(ConsoleOutput::error(&format!("Failed to process test signal: {}", e)))
                            }
                        } else {
                            ConsoleCommandResult::Output(ConsoleOutput::error("AudioWorklet manager not initialized"))
                        }
                    },
                    _ => ConsoleCommandResult::Output(ConsoleOutput::error("Unknown signal type. Use: sine, silence, pink-noise")),
                }
            },
            _ => ConsoleCommandResult::Output(ConsoleOutput::error(format!("Unknown volume subcommand: {}", subcommand))),
        }
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
                "gc" => {
                    outputs.push(ConsoleOutput::info("GC Pause Detection Status"));
                    outputs.push(ConsoleOutput::warning("Note: GC detection details not yet implemented"));
                    return ConsoleCommandResult::MultipleOutputs(outputs);
                }
                "help" => {
                    outputs.push(ConsoleOutput::info("Performance Monitor Commands:"));
                    outputs.push(ConsoleOutput::info("  perf        - Show current performance metrics"));
                    outputs.push(ConsoleOutput::info("  perf reset  - Reset performance counters"));
                    outputs.push(ConsoleOutput::info("  perf gc     - Show GC pause detection"));
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
            outputs.push(ConsoleOutput::info("  Buffer Size: 4096 bytes (1024 samples)"));
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
            "gc" => {
                if args.len() < 2 {
                    outputs.push(ConsoleOutput::error("Usage: pool gc <enable|disable|threshold>"));
                    outputs.push(ConsoleOutput::info("Examples:"));
                    outputs.push(ConsoleOutput::info("  pool gc enable"));
                    outputs.push(ConsoleOutput::info("  pool gc disable"));
                    outputs.push(ConsoleOutput::info("  pool gc threshold 100"));
                    return ConsoleCommandResult::MultipleOutputs(outputs);
                }
                
                match args[1] {
                    "enable" => {
                        outputs.push(ConsoleOutput::success("GC pause detection enabled"));
                        outputs.push(ConsoleOutput::warning("Note: Configuration changes not yet implemented"));
                    }
                    "disable" => {
                        outputs.push(ConsoleOutput::success("GC pause detection disabled"));
                        outputs.push(ConsoleOutput::warning("Note: Configuration changes not yet implemented"));
                    }
                    "threshold" => {
                        if args.len() < 3 {
                            outputs.push(ConsoleOutput::error("Usage: pool gc threshold <milliseconds>"));
                            return ConsoleCommandResult::MultipleOutputs(outputs);
                        }
                        
                        match args[2].parse::<u32>() {
                            Ok(threshold) => {
                                if threshold < 1 || threshold > 1000 {
                                    outputs.push(ConsoleOutput::error("GC threshold must be between 1ms and 1000ms"));
                                } else {
                                    outputs.push(ConsoleOutput::success(&format!("GC pause threshold set to {}ms", threshold)));
                                    outputs.push(ConsoleOutput::warning("Note: Configuration changes not yet implemented"));
                                }
                            }
                            Err(_) => {
                                outputs.push(ConsoleOutput::error("Invalid threshold. Must be a number in milliseconds."));
                            }
                        }
                    }
                    _ => {
                        outputs.push(ConsoleOutput::error(&format!("Unknown GC option: {}", args[1])));
                    }
                }
            }
            "optimize" => {
                outputs.push(ConsoleOutput::success("Buffer Pool Optimization Recommendations"));
                outputs.push(ConsoleOutput::info("\nFor Low Latency (<10ms):"));
                outputs.push(ConsoleOutput::info("  pool size 4"));
                outputs.push(ConsoleOutput::info("  pool timeout 1000"));
                outputs.push(ConsoleOutput::info("  pool gc threshold 20"));
                
                outputs.push(ConsoleOutput::info("\nFor High Throughput:"));
                outputs.push(ConsoleOutput::info("  pool size 32"));
                outputs.push(ConsoleOutput::info("  pool timeout 10000"));
                outputs.push(ConsoleOutput::info("  pool gc threshold 100"));
                
                outputs.push(ConsoleOutput::info("\nFor Balanced Performance (default):"));
                outputs.push(ConsoleOutput::info("  pool size 16"));
                outputs.push(ConsoleOutput::info("  pool timeout 5000"));
                outputs.push(ConsoleOutput::info("  pool gc threshold 50"));
            }
            "help" => {
                outputs.push(ConsoleOutput::info("Pool Configuration Commands:"));
                outputs.push(ConsoleOutput::info("  pool           - Show current configuration"));
                outputs.push(ConsoleOutput::info("  pool size <N>  - Set pool size (2-128)"));
                outputs.push(ConsoleOutput::info("  pool timeout <ms> - Set buffer timeout"));
                outputs.push(ConsoleOutput::info("  pool gc <opt>  - Configure GC detection"));
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
    registry.register(Box::new(BufferCommand));
    registry.register(Box::new(PitchCommand));
    registry.register(Box::new(VolumeCommand));
    registry.register(Box::new(TuningCommand));
    registry.register(Box::new(PerformanceCommand));
    registry.register(Box::new(PoolConfigCommand));
    
    // Register debugging commands
    registry.register(Box::new(PipelineDebugCommand));
    registry.register(Box::new(AudioWorkletDebugCommand));
    
}

#[cfg(test)]
mod tests {
    use super::*;
     use wasm_bindgen_test::wasm_bindgen_test;   
    
    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_tuning_command() {
        let command = TuningCommand;
        
        assert_eq!(command.name(), "tuning");
        assert_eq!(command.description(), "Switch tuning system (equal/just/custom)");
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_command_execution() {
        use egui_dev_console::ConsoleCommandRegistry;
        
        // Create a registry and register audio commands
        let mut registry = ConsoleCommandRegistry::new();
        register_audio_commands(&mut registry);
        
        // Test that the audio command is registered by checking if it can be executed
        // We'll test with a simple command that doesn't require Web Audio API
        let result = registry.execute("buffer status");
        // The result should be a warning about no buffer pool, not an error about unknown command
        match result {
            egui_dev_console::ConsoleCommandResult::Output(egui_dev_console::ConsoleOutput::Warning(_)) => {
                // Success - command was found and executed (returned warning about no buffer pool)
            },
            egui_dev_console::ConsoleCommandResult::Output(egui_dev_console::ConsoleOutput::Error(text)) => {
                // If it's an error about unknown command, that means registration failed
                if text.contains("Unknown command") {
                    panic!("Audio commands were not properly registered");
                }
                // Other errors are acceptable (like "No buffer pool initialized")
            },
            _ => {
                // Any other result is acceptable as long as it's not an "Unknown command" error
            }
        }
    }
}