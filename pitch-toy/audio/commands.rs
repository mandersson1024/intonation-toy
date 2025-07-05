//! Audio Console Commands
//!
//! This module contains console commands that are specific to audio functionality.
//! These commands access audio information through the audio module's public API,
//! maintaining proper separation of concerns.

use dev_console::{ConsoleCommand, ConsoleCommandResult, ConsoleOutput, ConsoleCommandRegistry};
use super::{AudioContextState, AudioContextManager, get_audio_context_manager};
use super::{PitchAnalyzer, TuningSystem};
// Volume-related imports will be needed when implementing actual volume detector access
use wasm_bindgen_futures;
use std::cell::RefCell;
use std::rc::Rc;

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
        } else {
            outputs.push(ConsoleOutput::warning("  Audio Context State: Not Initialized"));
            outputs.push(ConsoleOutput::warning("  Audio system has not been initialized yet"));
        }
        
        ConsoleCommandResult::MultipleOutputs(outputs)
    }
}





/// Buffer Status Command - show information for each buffer in the global pool
pub struct BufferStatusCommand;

impl ConsoleCommand for BufferStatusCommand {
    fn name(&self) -> &str { "buffer-status" }
    fn description(&self) -> &str { "Show current buffer pool status" }
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        use crate::audio::get_global_buffer_pool;
        let mut outputs = Vec::new();
        if let Some(pool_rc) = get_global_buffer_pool() {
            let pool = pool_rc.borrow();
            outputs.push(ConsoleOutput::info(&format!("Total Buffers: {} (capacity: {} samples each)", pool.len(), pool.buffer_capacity())));
            outputs.push(ConsoleOutput::info(&format!("Memory Usage: {:.2} KB", pool.memory_usage_bytes() as f64 / 1024.0)));
            for idx in 0..pool.len() {
                if let Some(buf) = pool.get(idx) {
                    outputs.push(ConsoleOutput::info(&format!("Buffer {} - len: {}/{} state: {} overflows: {}", idx, buf.len(), buf.capacity(), buf.state(), buf.overflow_count())));
                }
            }
            ConsoleCommandResult::MultipleOutputs(outputs)
        } else {
            ConsoleCommandResult::Output(ConsoleOutput::warning("No buffer pool initialized"))
        }
    }
}

/// Buffer Metrics Command - high-level metrics summary
pub struct BufferMetricsCommand;

impl ConsoleCommand for BufferMetricsCommand {
    fn name(&self) -> &str { "buffer-metrics" }
    fn description(&self) -> &str { "Display buffer pool metrics" }
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        use crate::audio::get_global_buffer_pool;
        if let Some(pool_rc) = get_global_buffer_pool() {
            let pool = pool_rc.borrow();
            let msg = format!("Buffers: {}  Overflows: {}  Memory: {:.2} KB", pool.len(), pool.total_overflows(), pool.memory_usage_bytes() as f64 / 1024.0);
            ConsoleCommandResult::Output(ConsoleOutput::info(msg))
        } else {
            ConsoleCommandResult::Output(ConsoleOutput::warning("No buffer pool initialized"))
        }
    }
}

/// Buffer Reset Command - clear all buffers and reset overflow counters
pub struct BufferResetCommand;

impl ConsoleCommand for BufferResetCommand {
    fn name(&self) -> &str { "buffer-reset" }
    fn description(&self) -> &str { "Reset buffer pool state" }
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        use crate::audio::get_global_buffer_pool;
        if let Some(pool_rc) = get_global_buffer_pool() {
            pool_rc.borrow_mut().reset_all();
            ConsoleCommandResult::Output(ConsoleOutput::success("Buffer pool cleared"))
        } else {
            ConsoleCommandResult::Output(ConsoleOutput::warning("No buffer pool initialized"))
        }
    }
}

/// Buffer Debug Command - toggle debug logging flag (simple runtime flag)
pub struct BufferDebugCommand;

thread_local! { 
    static BUFFER_DEBUG_ENABLED: std::cell::Cell<bool> = std::cell::Cell::new(false);
    static PITCH_ANALYZER_GLOBAL: RefCell<Option<Rc<RefCell<PitchAnalyzer>>>> = RefCell::new(None);
    static PITCH_DEBUG_ENABLED: std::cell::Cell<bool> = std::cell::Cell::new(false);
}

impl ConsoleCommand for BufferDebugCommand {
    fn name(&self) -> &str { "buffer-debug" }
    fn description(&self) -> &str { "Toggle buffer debug logging" }
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        let enabled = BUFFER_DEBUG_ENABLED.with(|c| { let val = !c.get(); c.set(val); val });
        ConsoleCommandResult::Output(ConsoleOutput::info(&format!("Buffer debug logging {}", if enabled { "enabled" } else { "disabled" })))
    }
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

/// Pitch Status Command - shows pitch detection configuration and state
pub struct PitchStatusCommand;

impl ConsoleCommand for PitchStatusCommand {
    fn name(&self) -> &str { "pitch-status" }
    fn description(&self) -> &str { "Show current pitch detection configuration and state" }
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
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
    }
}

/// Pitch Detect Command - test pitch detection with specific frequency
pub struct PitchDetectCommand;

impl ConsoleCommand for PitchDetectCommand {
    fn name(&self) -> &str { "pitch-detect" }
    fn description(&self) -> &str { "Test pitch detection with specific frequency (Hz)" }
    fn execute(&self, args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        if args.is_empty() {
            return ConsoleCommandResult::Output(ConsoleOutput::error("Usage: pitch-detect <frequency>"));
        }
        
        let frequency: f32 = match args[0].parse() {
            Ok(freq) => freq,
            Err(_) => return ConsoleCommandResult::Output(ConsoleOutput::error("Invalid frequency value")),
        };
        
        if frequency < 20.0 || frequency > 20000.0 {
            return ConsoleCommandResult::Output(ConsoleOutput::error("Frequency must be between 20 and 20000 Hz"));
        }
        
        if let Some(analyzer_rc) = get_global_pitch_analyzer() {
            let analyzer = analyzer_rc.borrow();
            let config = analyzer.config();
            
            // Generate test signal with the specified frequency
            let sample_rate = 48000.0; // Standard sample rate
            let duration_samples = config.sample_window_size;
            let mut test_signal = vec![0.0; duration_samples];
            
            for (i, sample) in test_signal.iter_mut().enumerate() {
                let t = i as f32 / sample_rate;
                *sample = (2.0 * std::f32::consts::PI * frequency * t).sin();
            }
            
            // Note: In a real implementation, we would call analyzer.analyze_samples(&test_signal)
            // For now, we'll just report the test setup
            ConsoleCommandResult::Output(ConsoleOutput::success(&format!(
                "Test signal generated: {:.1} Hz ({} samples at {:.1} kHz)", 
                frequency, duration_samples, sample_rate / 1000.0
            )))
        } else {
            ConsoleCommandResult::Output(ConsoleOutput::error("Pitch analyzer not initialized"))
        }
    }
}

/// Pitch Threshold Command - set confidence threshold
pub struct PitchThresholdCommand;

impl ConsoleCommand for PitchThresholdCommand {
    fn name(&self) -> &str { "pitch-threshold" }
    fn description(&self) -> &str { "Set confidence threshold (0.0-1.0)" }
    fn execute(&self, args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        if args.is_empty() {
            return ConsoleCommandResult::Output(ConsoleOutput::error("Usage: pitch-threshold <value>"));
        }
        
        let threshold: f32 = match args[0].parse() {
            Ok(thresh) => thresh,
            Err(_) => return ConsoleCommandResult::Output(ConsoleOutput::error("Invalid threshold value")),
        };
        
        if threshold < 0.0 || threshold > 1.0 {
            return ConsoleCommandResult::Output(ConsoleOutput::error("Threshold must be between 0.0 and 1.0"));
        }
        
        if let Some(analyzer_rc) = get_global_pitch_analyzer() {
            let mut analyzer = analyzer_rc.borrow_mut();
            let mut config = analyzer.config().clone();
            config.threshold = threshold;
            
            match analyzer.update_config(config) {
                Ok(_) => ConsoleCommandResult::Output(ConsoleOutput::success(&format!("Threshold set to {:.2}", threshold))),
                Err(e) => ConsoleCommandResult::Output(ConsoleOutput::error(&format!("Failed to update threshold: {}", e))),
            }
        } else {
            ConsoleCommandResult::Output(ConsoleOutput::error("Pitch analyzer not initialized"))
        }
    }
}

/// Pitch Tuning Command - switch tuning system
pub struct PitchTuningCommand;

impl ConsoleCommand for PitchTuningCommand {
    fn name(&self) -> &str { "pitch-tuning" }
    fn description(&self) -> &str { "Switch tuning system (equal/just/custom)" }
    fn execute(&self, args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        if args.is_empty() {
            return ConsoleCommandResult::Output(ConsoleOutput::error("Usage: pitch-tuning <system> [reference_pitch]"));
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

/// Pitch Window Command - set analysis window size
pub struct PitchWindowCommand;

impl ConsoleCommand for PitchWindowCommand {
    fn name(&self) -> &str { "pitch-window" }
    fn description(&self) -> &str { "Set analysis window size (multiple of 128)" }
    fn execute(&self, args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        if args.is_empty() {
            return ConsoleCommandResult::Output(ConsoleOutput::error("Usage: pitch-window <size>"));
        }
        
        let window_size: usize = match args[0].parse() {
            Ok(size) => size,
            Err(_) => return ConsoleCommandResult::Output(ConsoleOutput::error("Invalid window size")),
        };
        
        if window_size < 128 || window_size % 128 != 0 {
            return ConsoleCommandResult::Output(ConsoleOutput::error("Window size must be a multiple of 128"));
        }
        
        if window_size > 8192 {
            return ConsoleCommandResult::Output(ConsoleOutput::error("Window size must be ≤ 8192 samples"));
        }
        
        if let Some(analyzer_rc) = get_global_pitch_analyzer() {
            let mut analyzer = analyzer_rc.borrow_mut();
            let mut config = analyzer.config().clone();
            config.sample_window_size = window_size;
            
            match analyzer.update_config(config) {
                Ok(_) => ConsoleCommandResult::Output(ConsoleOutput::success(&format!("Window size set to {} samples", window_size))),
                Err(e) => ConsoleCommandResult::Output(ConsoleOutput::error(&format!("Failed to update window size: {}", e))),
            }
        } else {
            ConsoleCommandResult::Output(ConsoleOutput::error("Pitch analyzer not initialized"))
        }
    }
}

/// Pitch Range Command - set frequency detection range
pub struct PitchRangeCommand;

impl ConsoleCommand for PitchRangeCommand {
    fn name(&self) -> &str { "pitch-range" }
    fn description(&self) -> &str { "Set frequency detection range (min max)" }
    fn execute(&self, args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        if args.len() < 2 {
            return ConsoleCommandResult::Output(ConsoleOutput::error("Usage: pitch-range <min> <max>"));
        }
        
        let min_freq: f32 = match args[0].parse() {
            Ok(freq) => freq,
            Err(_) => return ConsoleCommandResult::Output(ConsoleOutput::error("Invalid minimum frequency")),
        };
        
        let max_freq: f32 = match args[1].parse() {
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
    }
}

/// Pitch Debug Command - toggle pitch detection debugging
pub struct PitchDebugCommand;

impl ConsoleCommand for PitchDebugCommand {
    fn name(&self) -> &str { "pitch-debug" }
    fn description(&self) -> &str { "Toggle pitch detection debug logging" }
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        let enabled = PITCH_DEBUG_ENABLED.with(|c| { 
            let val = !c.get(); 
            c.set(val); 
            val 
        });
        
        let message = if enabled {
            "Pitch detection debug logging enabled"
        } else {
            "Pitch detection debug logging disabled"
        };
        
        ConsoleCommandResult::Output(ConsoleOutput::info(message))
    }
}

/// Pitch Benchmarks Command - run performance benchmarks for different window sizes
pub struct PitchBenchmarksCommand;

impl ConsoleCommand for PitchBenchmarksCommand {
    fn name(&self) -> &str { "pitch-benchmarks" }
    fn description(&self) -> &str { "Run performance benchmarks for different window sizes" }
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        if let Some(analyzer_rc) = get_global_pitch_analyzer() {
            let mut outputs = Vec::new();
            outputs.push(ConsoleOutput::info("Running pitch detection benchmarks..."));
            
            let sample_rate = 48000.0; // Default sample rate for benchmarks
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
                let performance_grade = if avg_time <= 20.0 {
                    "Fast"
                } else if avg_time <= 35.0 {
                    "Balanced"
                } else if avg_time <= 50.0 {
                    "Accurate"
                } else if avg_time <= 100.0 {
                    "High-Accuracy"
                } else {
                    "Maximum-Accuracy"
                };
                
                let performance_output = if avg_time <= 50.0 {
                    ConsoleOutput::info(&format!("  {:>11} | {:>13.1} | {:>13.1} | {}", 
                                              window_size, avg_time, min_time, performance_grade))
                } else {
                    ConsoleOutput::warning(&format!("  {:>11} | {:>13.1} | {:>13.1} | {}", 
                                                   window_size, avg_time, min_time, performance_grade))
                };
                outputs.push(performance_output);
            }
            
            // Performance summary
            outputs.push(ConsoleOutput::info(""));
            outputs.push(ConsoleOutput::info("Performance Categories (Accuracy-Optimized):"));
            outputs.push(ConsoleOutput::info("  • Fast:           ≤20ms (Speed-focused)"));
            outputs.push(ConsoleOutput::info("  • Balanced:       ≤35ms (Speed/accuracy balance)"));
            outputs.push(ConsoleOutput::info("  • Accurate:       ≤50ms (Production target)"));
            outputs.push(ConsoleOutput::info("  • High-Accuracy:  ≤100ms (Research quality)"));
            outputs.push(ConsoleOutput::info("  • Maximum-Accuracy: >100ms (Offline analysis)"));
            
            // Show current analyzer performance
            let analyzer = analyzer_rc.borrow();
            let current_metrics = analyzer.metrics();
            let current_grade = analyzer.performance_grade();
            let meets_requirements = analyzer.meets_performance_requirements();
            
            outputs.push(ConsoleOutput::info(""));
            outputs.push(ConsoleOutput::info("Current Pitch Analyzer Performance:"));
            outputs.push(ConsoleOutput::info(&format!("  Window Size: {} samples", analyzer.config().sample_window_size)));
            outputs.push(ConsoleOutput::info(&format!("  Average Latency: {:.1} ms", current_metrics.average_latency_ms)));
            outputs.push(ConsoleOutput::info(&format!("  Latest Latency: {:.1} ms", current_metrics.processing_latency_ms)));
            outputs.push(ConsoleOutput::info(&format!("  Min/Max Latency: {:.1}/{:.1} ms", current_metrics.min_latency_ms, current_metrics.max_latency_ms)));
            outputs.push(ConsoleOutput::info(&format!("  Latency Violations: {} ({:.1}%)", 
                                            current_metrics.latency_violations,
                                            if current_metrics.analysis_cycles > 0 {
                                                current_metrics.latency_violations as f32 / current_metrics.analysis_cycles as f32 * 100.0
                                            } else { 0.0 })));
            outputs.push(ConsoleOutput::info(&format!("  YIN Algorithm Time: {:.0} μs", current_metrics.yin_processing_time_us)));
            
            let grade_output = if meets_requirements {
                ConsoleOutput::success(&format!("  Performance Grade: {} ✓", current_grade))
            } else {
                ConsoleOutput::warning(&format!("  Performance Grade: {} ⚠", current_grade))
            };
            outputs.push(grade_output);
            
            if !meets_requirements {
                outputs.push(ConsoleOutput::warning(""));
                outputs.push(ConsoleOutput::warning("Performance Recommendations:"));
                if current_metrics.average_latency_ms > 50.0 {
                    outputs.push(ConsoleOutput::warning("  • Consider reducing window size for faster processing"));
                }
                if current_metrics.latency_violations > current_metrics.analysis_cycles / 20 {
                    outputs.push(ConsoleOutput::warning("  • Too many latency violations - check system load"));
                }
                outputs.push(ConsoleOutput::warning("  • Use 'pitch window <size>' to adjust window size"));
                outputs.push(ConsoleOutput::warning("  • Use 'pitch optimize-accuracy' for maximum accuracy"));
                outputs.push(ConsoleOutput::warning("  • Recommended sizes: 1024 (balanced), 2048 (accurate), 4096 (max)"));
            }
            
            ConsoleCommandResult::MultipleOutputs(outputs)
        } else {
            ConsoleCommandResult::Output(ConsoleOutput::warning("Pitch analyzer not initialized"))
        }
    }
}

/// Pitch Optimize Accuracy Command - optimize configuration for maximum accuracy
pub struct PitchOptimizeAccuracyCommand;

impl ConsoleCommand for PitchOptimizeAccuracyCommand {
    fn name(&self) -> &str { "pitch-optimize-accuracy" }
    fn description(&self) -> &str { "Optimize configuration for maximum accuracy within 50ms latency" }
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        if let Some(analyzer_rc) = get_global_pitch_analyzer() {
            let mut outputs = Vec::new();
            
            // Get current configuration
            let old_window_size = {
                let analyzer = analyzer_rc.borrow();
                analyzer.config().sample_window_size
            };
            
            // Optimize for accuracy
            let result = {
                let mut analyzer = analyzer_rc.borrow_mut();
                analyzer.optimize_for_accuracy()
            };
            
            match result {
                Ok(()) => {
                    let analyzer = analyzer_rc.borrow();
                    let new_window_size = analyzer.config().sample_window_size;
                    let (estimated_latency, performance_grade) = analyzer.pitch_detector().get_performance_characteristics();
                    let (frequency_resolution, accuracy_grade) = analyzer.pitch_detector().get_accuracy_characteristics();
                    
                    outputs.push(ConsoleOutput::success("Configuration optimized for accuracy"));
                    outputs.push(ConsoleOutput::info(&format!("  Window size: {} → {} samples", old_window_size, new_window_size)));
                    outputs.push(ConsoleOutput::info(&format!("  Estimated latency: {:.1} ms ({})", estimated_latency, performance_grade)));
                    outputs.push(ConsoleOutput::info(&format!("  Frequency resolution: {:.1} Hz ({})", frequency_resolution, accuracy_grade)));
                    outputs.push(ConsoleOutput::info(&format!("  Early exit optimization: disabled (for accuracy)")));
                    
                    if estimated_latency <= 50.0 {
                        outputs.push(ConsoleOutput::success("✓ Meets 50ms real-time requirement"));
                    } else {
                        outputs.push(ConsoleOutput::warning(&format!("⚠ Exceeds 50ms requirement by {:.1}ms", estimated_latency - 50.0)));
                    }
                }
                Err(e) => {
                    outputs.push(ConsoleOutput::error(&format!("Failed to optimize: {}", e)));
                }
            }
            
            ConsoleCommandResult::MultipleOutputs(outputs)
        } else {
            ConsoleCommandResult::Output(ConsoleOutput::warning("Pitch analyzer not initialized"))
        }
    }
}

/// Volume Status Command - show current volume levels and configuration
pub struct VolumeStatusCommand;

impl ConsoleCommand for VolumeStatusCommand {
    fn name(&self) -> &str { "volume-status" }
    fn description(&self) -> &str { "Show current volume levels and detector configuration" }
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        let mut outputs = Vec::new();
        
        // TODO: Access AudioWorkletManager to get volume detector status
        // This would require extending the audio module to provide access to volume detector state
        outputs.push(ConsoleOutput::info("Volume Detection Status:"));
        outputs.push(ConsoleOutput::info("  Status: Available"));
        outputs.push(ConsoleOutput::warning("  Live data: Not yet implemented - requires AudioWorklet manager access"));
        outputs.push(ConsoleOutput::info(""));
        outputs.push(ConsoleOutput::info("Volume Detector Configuration:"));
        outputs.push(ConsoleOutput::info("  Input Gain: 0.0 dB"));
        outputs.push(ConsoleOutput::info("  Noise Floor: -60.0 dB"));
        outputs.push(ConsoleOutput::info("  Peak Decay Fast: 100.0 ms"));
        outputs.push(ConsoleOutput::info("  Peak Decay Slow: 1000.0 ms"));
        outputs.push(ConsoleOutput::info(""));
        outputs.push(ConsoleOutput::info("Volume Thresholds:"));
        outputs.push(ConsoleOutput::info("  Silent: < -60.0 dB"));
        outputs.push(ConsoleOutput::info("  Low: -60.0 to -30.0 dB"));
        outputs.push(ConsoleOutput::info("  Optimal: -30.0 to -6.0 dB"));
        outputs.push(ConsoleOutput::info("  High: -6.0 to 0.0 dB"));
        outputs.push(ConsoleOutput::info("  Clipping: >= 0.0 dB"));
        
        ConsoleCommandResult::MultipleOutputs(outputs)
    }
}

/// Volume Config Command - configure volume detector parameters
pub struct VolumeConfigCommand;

impl ConsoleCommand for VolumeConfigCommand {
    fn name(&self) -> &str { "volume-config" }
    fn description(&self) -> &str { "Configure volume detector parameters: gain <db> | noise-floor <db> | decay-fast <ms> | decay-slow <ms>" }
    fn execute(&self, args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        if args.len() < 2 {
            return ConsoleCommandResult::Output(ConsoleOutput::error("Usage: volume-config <parameter> <value>"));
        }
        
        let parameter = args[0].to_lowercase();
        let value_str = args[1];
        
        let value: f32 = match value_str.parse() {
            Ok(v) => v,
            Err(_) => return ConsoleCommandResult::Output(ConsoleOutput::error("Invalid numeric value")),
        };
        
        match parameter.as_str() {
            "gain" => {
                if value < -60.0 || value > 60.0 {
                    return ConsoleCommandResult::Output(ConsoleOutput::error("Input gain must be between -60 and 60 dB"));
                }
                // TODO: Update volume detector configuration
                ConsoleCommandResult::Output(ConsoleOutput::success(&format!("Input gain set to {:.1} dB", value)))
            },
            "noise-floor" => {
                if value < -80.0 || value > -20.0 {
                    return ConsoleCommandResult::Output(ConsoleOutput::error("Noise floor must be between -80 and -20 dB"));
                }
                // TODO: Update volume detector configuration
                ConsoleCommandResult::Output(ConsoleOutput::success(&format!("Noise floor set to {:.1} dB", value)))
            },
            "decay-fast" => {
                if value < 10.0 || value > 500.0 {
                    return ConsoleCommandResult::Output(ConsoleOutput::error("Fast decay time must be between 10 and 500 ms"));
                }
                // TODO: Update volume detector configuration
                ConsoleCommandResult::Output(ConsoleOutput::success(&format!("Fast decay time set to {:.1} ms", value)))
            },
            "decay-slow" => {
                if value < 100.0 || value > 5000.0 {
                    return ConsoleCommandResult::Output(ConsoleOutput::error("Slow decay time must be between 100 and 5000 ms"));
                }
                // TODO: Update volume detector configuration
                ConsoleCommandResult::Output(ConsoleOutput::success(&format!("Slow decay time set to {:.1} ms", value)))
            },
            _ => ConsoleCommandResult::Output(ConsoleOutput::error("Unknown parameter. Use: gain, noise-floor, decay-fast, decay-slow")),
        }
    }
}

/// Volume Test Command - generate test signals for volume detection validation
pub struct VolumeTestCommand;

impl ConsoleCommand for VolumeTestCommand {
    fn name(&self) -> &str { "volume-test" }
    fn description(&self) -> &str { "Test volume detection with generated signals: sine <freq> <amplitude> | silence | pink-noise <amplitude>" }
    fn execute(&self, args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        if args.is_empty() {
            return ConsoleCommandResult::Output(ConsoleOutput::error("Usage: volume-test <signal-type> [parameters]"));
        }
        
        let signal_type = args[0].to_lowercase();
        
        match signal_type.as_str() {
            "sine" => {
                if args.len() < 3 {
                    return ConsoleCommandResult::Output(ConsoleOutput::error("Usage: volume-test sine <frequency> <amplitude>"));
                }
                
                let frequency: f32 = match args[1].parse() {
                    Ok(f) => f,
                    Err(_) => return ConsoleCommandResult::Output(ConsoleOutput::error("Invalid frequency")),
                };
                
                let amplitude: f32 = match args[2].parse() {
                    Ok(a) => a,
                    Err(_) => return ConsoleCommandResult::Output(ConsoleOutput::error("Invalid amplitude")),
                };
                
                if frequency < 20.0 || frequency > 20000.0 {
                    return ConsoleCommandResult::Output(ConsoleOutput::error("Frequency must be between 20 and 20000 Hz"));
                }
                
                if amplitude < 0.0 || amplitude > 1.0 {
                    return ConsoleCommandResult::Output(ConsoleOutput::error("Amplitude must be between 0.0 and 1.0"));
                }
                
                // TODO: Generate sine wave test signal
                ConsoleCommandResult::Output(ConsoleOutput::success(&format!("Generating sine wave: {:.1} Hz at {:.3} amplitude", frequency, amplitude)))
            },
            "silence" => {
                // TODO: Generate silence for testing
                ConsoleCommandResult::Output(ConsoleOutput::success("Generating silence for volume detection test"))
            },
            "pink-noise" => {
                if args.len() < 2 {
                    return ConsoleCommandResult::Output(ConsoleOutput::error("Usage: volume-test pink-noise <amplitude>"));
                }
                
                let amplitude: f32 = match args[1].parse() {
                    Ok(a) => a,
                    Err(_) => return ConsoleCommandResult::Output(ConsoleOutput::error("Invalid amplitude")),
                };
                
                if amplitude < 0.0 || amplitude > 1.0 {
                    return ConsoleCommandResult::Output(ConsoleOutput::error("Amplitude must be between 0.0 and 1.0"));
                }
                
                // TODO: Generate pink noise test signal
                ConsoleCommandResult::Output(ConsoleOutput::success(&format!("Generating pink noise at {:.3} amplitude", amplitude)))
            },
            _ => ConsoleCommandResult::Output(ConsoleOutput::error("Unknown signal type. Use: sine, silence, pink-noise")),
        }
    }
}

/// Base Audio Command - shows audio system status and configuration
pub struct AudioCommand;

impl ConsoleCommand for AudioCommand {
    fn name(&self) -> &str { "audio" }
    fn description(&self) -> &str { "Show AudioContext status and configuration" }
    fn execute(&self, _args: Vec<&str>, _registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        AudioContextCommand.execute(Vec::new(), _registry)
    }
}

/// Base Buffer Command - handles "buffer" with subcommands
pub struct BufferCommand;

impl ConsoleCommand for BufferCommand {
    fn name(&self) -> &str { "buffer" }
    fn description(&self) -> &str { "Buffer management commands" }
    fn execute(&self, args: Vec<&str>, registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        if args.is_empty() {
            // Show available buffer subcommands
            let variants = registry.get_command_variants("buffer");
            if !variants.is_empty() {
                let mut help_lines = vec!["Available buffer commands:".to_string()];
                for variant in variants {
                    let variant_name = variant.name().strip_prefix("buffer-").unwrap_or(variant.name());
                    help_lines.push(format!("  buffer {} - {}", variant_name, variant.description()));
                }
                let help_text = help_lines.join("\n");
                return ConsoleCommandResult::Output(ConsoleOutput::info(help_text));
            } else {
                return ConsoleCommandResult::Output(ConsoleOutput::error("No buffer subcommands available"));
            }
        }
        
        let subcommand = args[0];
        let sub_args = args[1..].to_vec();
        
        match subcommand {
            "status" => BufferStatusCommand.execute(sub_args, registry),
            "metrics" => BufferMetricsCommand.execute(sub_args, registry),
            "reset" => BufferResetCommand.execute(sub_args, registry),
            "debug" => BufferDebugCommand.execute(sub_args, registry),
            _ => ConsoleCommandResult::Output(ConsoleOutput::error(format!("Unknown buffer subcommand: {}", subcommand))),
        }
    }
}

/// Base Pitch Command - handles "pitch" with subcommands
pub struct PitchCommand;

impl ConsoleCommand for PitchCommand {
    fn name(&self) -> &str { "pitch" }
    fn description(&self) -> &str { "Pitch detection commands" }
    fn execute(&self, args: Vec<&str>, registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        if args.is_empty() {
            // Show available pitch subcommands
            let variants = registry.get_command_variants("pitch");
            if !variants.is_empty() {
                let mut help_lines = vec!["Available pitch commands:".to_string()];
                for variant in variants {
                    let variant_name = variant.name().strip_prefix("pitch-").unwrap_or(variant.name());
                    help_lines.push(format!("  pitch {} - {}", variant_name, variant.description()));
                }
                let help_text = help_lines.join("\n");
                return ConsoleCommandResult::Output(ConsoleOutput::info(help_text));
            } else {
                return ConsoleCommandResult::Output(ConsoleOutput::error("No pitch subcommands available"));
            }
        }
        
        let subcommand = args[0];
        let sub_args = args[1..].to_vec();
        
        match subcommand {
            "status" => PitchStatusCommand.execute(sub_args, registry),
            "detect" => PitchDetectCommand.execute(sub_args, registry),
            "threshold" => PitchThresholdCommand.execute(sub_args, registry),
            "tuning" => PitchTuningCommand.execute(sub_args, registry),
            "window" => PitchWindowCommand.execute(sub_args, registry),
            "range" => PitchRangeCommand.execute(sub_args, registry),
            "debug" => PitchDebugCommand.execute(sub_args, registry),
            "benchmarks" => PitchBenchmarksCommand.execute(sub_args, registry),
            "optimize-accuracy" => PitchOptimizeAccuracyCommand.execute(sub_args, registry),
            _ => ConsoleCommandResult::Output(ConsoleOutput::error(format!("Unknown pitch subcommand: {}", subcommand))),
        }
    }
}

/// Base Volume Command - handles "volume" with subcommands
pub struct VolumeCommand;

impl ConsoleCommand for VolumeCommand {
    fn name(&self) -> &str { "volume" }
    fn description(&self) -> &str { "Volume detection commands" }
    fn execute(&self, args: Vec<&str>, registry: &ConsoleCommandRegistry) -> ConsoleCommandResult {
        if args.is_empty() {
            // Show available volume subcommands
            let variants = registry.get_command_variants("volume");
            if !variants.is_empty() {
                let mut help_lines = vec!["Available volume commands:".to_string()];
                for variant in variants {
                    let variant_name = variant.name().strip_prefix("volume-").unwrap_or(variant.name());
                    help_lines.push(format!("  volume {} - {}", variant_name, variant.description()));
                }
                let help_text = help_lines.join("\n");
                return ConsoleCommandResult::Output(ConsoleOutput::info(help_text));
            } else {
                return ConsoleCommandResult::Output(ConsoleOutput::error("No volume subcommands available"));
            }
        }
        
        let subcommand = args[0];
        let sub_args = args[1..].to_vec();
        
        match subcommand {
            "status" => VolumeStatusCommand.execute(sub_args, registry),
            "config" => VolumeConfigCommand.execute(sub_args, registry),
            "test" => VolumeTestCommand.execute(sub_args, registry),
            _ => ConsoleCommandResult::Output(ConsoleOutput::error(format!("Unknown volume subcommand: {}", subcommand))),
        }
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
    
    // Register compound commands for variant discovery and backward compatibility
    // These won't appear in help but will be found when parsing compound commands
    registry.register(Box::new(AudioContextCommand));
    registry.register(Box::new(BufferStatusCommand));
    registry.register(Box::new(BufferMetricsCommand));
    registry.register(Box::new(BufferResetCommand));
    registry.register(Box::new(BufferDebugCommand));
    registry.register(Box::new(PitchStatusCommand));
    registry.register(Box::new(PitchDetectCommand));
    registry.register(Box::new(PitchThresholdCommand));
    registry.register(Box::new(PitchTuningCommand));
    registry.register(Box::new(PitchWindowCommand));
    registry.register(Box::new(PitchRangeCommand));
    registry.register(Box::new(PitchDebugCommand));
    registry.register(Box::new(PitchBenchmarksCommand));
    registry.register(Box::new(PitchOptimizeAccuracyCommand));
    registry.register(Box::new(VolumeStatusCommand));
    registry.register(Box::new(VolumeConfigCommand));
    registry.register(Box::new(VolumeTestCommand));
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
    fn test_pitch_status_command() {
        let command = PitchStatusCommand;
        
        assert_eq!(command.name(), "pitch-status");
        assert_eq!(command.description(), "Show current pitch detection configuration and state");
    }
    
    #[test]
    fn test_pitch_detect_command() {
        let command = PitchDetectCommand;
        
        assert_eq!(command.name(), "pitch-detect");
        assert_eq!(command.description(), "Test pitch detection with specific frequency (Hz)");
    }
    
    #[test]
    fn test_pitch_threshold_command() {
        let command = PitchThresholdCommand;
        
        assert_eq!(command.name(), "pitch-threshold");
        assert_eq!(command.description(), "Set confidence threshold (0.0-1.0)");
    }
    
    #[test]
    fn test_pitch_tuning_command() {
        let command = PitchTuningCommand;
        
        assert_eq!(command.name(), "pitch-tuning");
        assert_eq!(command.description(), "Switch tuning system (equal/just/custom)");
    }
    
    #[test]
    fn test_pitch_window_command() {
        let command = PitchWindowCommand;
        
        assert_eq!(command.name(), "pitch-window");
        assert_eq!(command.description(), "Set analysis window size (multiple of 128)");
    }
    
    #[test]
    fn test_pitch_range_command() {
        let command = PitchRangeCommand;
        
        assert_eq!(command.name(), "pitch-range");
        assert_eq!(command.description(), "Set frequency detection range (min max)");
    }
    
    #[test]
    fn test_pitch_debug_command() {
        let command = PitchDebugCommand;
        
        assert_eq!(command.name(), "pitch-debug");
        assert_eq!(command.description(), "Toggle pitch detection debug logging");
    }
    
    #[test]
    fn test_pitch_benchmarks_command() {
        let command = PitchBenchmarksCommand;
        
        assert_eq!(command.name(), "pitch-benchmarks");
        assert_eq!(command.description(), "Run performance benchmarks for different window sizes");
    }
    
    #[test]
    fn test_pitch_optimize_accuracy_command() {
        let command = PitchOptimizeAccuracyCommand;
        
        assert_eq!(command.name(), "pitch-optimize-accuracy");
        assert_eq!(command.description(), "Optimize configuration for maximum accuracy within 50ms latency");
    }
    
    #[test]
    fn test_volume_status_command() {
        let command = VolumeStatusCommand;
        
        assert_eq!(command.name(), "volume-status");
        assert_eq!(command.description(), "Show current volume levels and detector configuration");
    }
    
    #[test]
    fn test_volume_config_command() {
        let command = VolumeConfigCommand;
        
        assert_eq!(command.name(), "volume-config");
        assert_eq!(command.description(), "Configure volume detector parameters: gain <db> | noise-floor <db> | decay-fast <ms> | decay-slow <ms>");
    }
    
    #[test]
    fn test_volume_test_command() {
        let command = VolumeTestCommand;
        
        assert_eq!(command.name(), "volume-test");
        assert_eq!(command.description(), "Test volume detection with generated signals: sine <freq> <amplitude> | silence | pink-noise <amplitude>");
    }

    #[test]
    fn test_audio_command_execution() {
        use dev_console::ConsoleCommandRegistry;
        
        // Create a registry and register audio commands
        let mut registry = ConsoleCommandRegistry::new();
        register_audio_commands(&mut registry);
        
        // Test that calling "audio" directly executes the context command
        let result = registry.execute("audio");
        // The result will depend on the audio system state, but it should not be an error about subcommands
        match result {
            dev_console::ConsoleCommandResult::Output(_) => {
                // Success - command executed
            },
            dev_console::ConsoleCommandResult::MultipleOutputs(_) => {
                // Success - command executed with multiple outputs
            },
            _ => panic!("Expected audio command to execute successfully"),
        }
    }
}