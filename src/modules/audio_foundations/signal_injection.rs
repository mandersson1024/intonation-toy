// Signal Injection System - STORY-016
// Seamless integration of test signals into audio processing pipeline

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use super::signal_generator::{SignalGenerator, WebSignalGenerator, SignalConfig, GenerationState};
use super::test_signal_library::{TestSignalLibrary, TestSignal};
use super::audio_events::AudioEvent;
use crate::modules::application_core::event_bus::EventBus;

/// Signal injection manager for audio processing pipeline
pub struct SignalInjectionManager {
    generator: Arc<Mutex<WebSignalGenerator>>,
    test_library: Arc<Mutex<TestSignalLibrary>>,
    injection_state: Arc<Mutex<InjectionState>>,
    buffer_queue: Arc<Mutex<VecDeque<f32>>>,
    event_bus: Option<Arc<dyn EventBus>>,
}

/// Current state of signal injection
#[derive(Debug, Clone, PartialEq)]
pub struct InjectionState {
    pub is_active: bool,
    pub current_signal: Option<String>,
    pub playback_position: usize,
    pub loop_enabled: bool,
    pub mix_with_input: bool,
    pub mix_ratio: f32, // 0.0 = only input, 1.0 = only generated signal
    pub auto_stop: Option<u32>, // Stop after N milliseconds
    pub injection_mode: InjectionMode,
}

/// Signal injection modes
#[derive(Debug, Clone, PartialEq)]
pub enum InjectionMode {
    Replace,    // Replace input audio completely
    Mix,        // Mix with input audio
    Insert,     // Insert between input buffers
    Overlay,    // Overlay on top of input
}

/// Signal injection configuration
#[derive(Debug, Clone)]
pub struct InjectionConfig {
    pub signal_name: Option<String>,     // Use pre-recorded signal
    pub generator_config: Option<SignalConfig>, // Generate signal on-demand
    pub mode: InjectionMode,
    pub mix_ratio: f32,
    pub loop_enabled: bool,
    pub auto_stop_ms: Option<u32>,
    pub fade_in_ms: Option<u32>,
    pub fade_out_ms: Option<u32>,
}

impl Default for InjectionConfig {
    fn default() -> Self {
        Self {
            signal_name: None,
            generator_config: None,
            mode: InjectionMode::Replace,
            mix_ratio: 1.0,
            loop_enabled: false,
            auto_stop_ms: None,
            fade_in_ms: Some(50), // 50ms fade in
            fade_out_ms: Some(50), // 50ms fade out
        }
    }
}

/// Result of signal injection operation
#[derive(Debug, Clone)]
pub struct InjectionResult {
    pub success: bool,
    pub message: String,
    pub signal_info: Option<SignalInfo>,
}

/// Information about injected signal
#[derive(Debug, Clone)]
pub struct SignalInfo {
    pub name: String,
    pub duration_ms: u32,
    pub sample_rate: f32,
    pub expected_frequency: Option<f32>,
    pub amplitude: f32,
}

impl SignalInjectionManager {
    /// Create new signal injection manager
    pub fn new(sample_rate: f32, event_bus: Option<Arc<dyn EventBus>>) -> Self {
        Self {
            generator: Arc::new(Mutex::new(WebSignalGenerator::new(sample_rate))),
            test_library: Arc::new(Mutex::new(TestSignalLibrary::new(sample_rate))),
            injection_state: Arc::new(Mutex::new(InjectionState {
                is_active: false,
                current_signal: None,
                playback_position: 0,
                loop_enabled: false,
                mix_with_input: false,
                mix_ratio: 1.0,
                auto_stop: None,
                injection_mode: InjectionMode::Replace,
            })),
            buffer_queue: Arc::new(Mutex::new(VecDeque::new())),
            event_bus,
        }
    }
    
    /// Start signal injection
    pub fn start_injection(&self, config: InjectionConfig) -> Result<InjectionResult, String> {
        let mut state = self.injection_state.lock()
            .map_err(|_| "Failed to acquire injection state lock".to_string())?;
        
        // Prepare signal based on configuration
        let signal_info = if let Some(signal_name) = &config.signal_name {
            self.setup_prerecorded_signal(signal_name)?
        } else if let Some(gen_config) = &config.generator_config {
            self.setup_generated_signal(gen_config)?
        } else {
            return Err("No signal source specified".to_string());
        };
        
        // Update injection state
        state.is_active = true;
        state.current_signal = config.signal_name.or_else(|| {
            Some(format!("generated_{:?}", config.generator_config.as_ref().unwrap().waveform))
        });
        state.playback_position = 0;
        state.loop_enabled = config.loop_enabled;
        state.mix_with_input = matches!(config.mode, InjectionMode::Mix | InjectionMode::Overlay);
        state.mix_ratio = config.mix_ratio;
        state.auto_stop = config.auto_stop_ms;
        state.injection_mode = config.mode;
        
        // Publish event
        if let Some(event_bus) = &self.event_bus {
            let event = AudioEvent::SignalInjectionStarted {
                signal_name: state.current_signal.clone().unwrap_or_default(),
                mode: format!("{:?}", state.injection_mode),
                mix_ratio: state.mix_ratio,
                timestamp_ns: self.get_timestamp_ns(),
            };
            event_bus.publish_high(Box::new(event));
        }
        
        Ok(InjectionResult {
            success: true,
            message: format!("Signal injection started: {}", signal_info.name),
            signal_info: Some(signal_info),
        })
    }
    
    /// Stop signal injection
    pub fn stop_injection(&self) -> Result<InjectionResult, String> {
        let mut state = self.injection_state.lock()
            .map_err(|_| "Failed to acquire injection state lock".to_string())?;
        
        let signal_name = state.current_signal.clone().unwrap_or_default();
        
        state.is_active = false;
        state.current_signal = None;
        state.playback_position = 0;
        
        // Clear buffer queue
        if let Ok(mut queue) = self.buffer_queue.lock() {
            queue.clear();
        }
        
        // Stop real-time generation if active
        if let Ok(mut generator) = self.generator.lock() {
            let _ = generator.stop_real_time_generation();
        }
        
        // Publish event
        if let Some(event_bus) = &self.event_bus {
            let event = AudioEvent::SignalInjectionStopped {
                signal_name: signal_name.clone(),
                timestamp_ns: self.get_timestamp_ns(),
            };
            event_bus.publish_high(Box::new(event));
        }
        
        Ok(InjectionResult {
            success: true,
            message: format!("Signal injection stopped: {}", signal_name),
            signal_info: None,
        })
    }
    
    /// Process audio buffer with signal injection
    pub fn process_buffer(&self, input_buffer: &[f32], output_buffer: &mut [f32]) -> Result<(), String> {
        let state = self.injection_state.lock()
            .map_err(|_| "Failed to acquire injection state lock".to_string())?;
        
        if !state.is_active {
            // Pass through input when injection is not active
            output_buffer.copy_from_slice(input_buffer);
            return Ok(());
        }
        
        match state.injection_mode {
            InjectionMode::Replace => {
                self.inject_replace_mode(&state, output_buffer)?;
            },
            InjectionMode::Mix => {
                self.inject_mix_mode(&state, input_buffer, output_buffer)?;
            },
            InjectionMode::Insert => {
                self.inject_insert_mode(&state, input_buffer, output_buffer)?;
            },
            InjectionMode::Overlay => {
                self.inject_overlay_mode(&state, input_buffer, output_buffer)?;
            },
        }
        
        Ok(())
    }
    
    /// Setup prerecorded signal for injection
    fn setup_prerecorded_signal(&self, signal_name: &str) -> Result<SignalInfo, String> {
        let library = self.test_library.lock()
            .map_err(|_| "Failed to acquire test library lock".to_string())?;
        
        let signal = library.get_signal(signal_name)
            .ok_or_else(|| format!("Signal '{}' not found in test library", signal_name))?;
        
        // Load signal into buffer queue
        let mut queue = self.buffer_queue.lock()
            .map_err(|_| "Failed to acquire buffer queue lock".to_string())?;
        queue.clear();
        queue.extend(signal.samples.iter());
        
        Ok(SignalInfo {
            name: signal.name.clone(),
            duration_ms: signal.duration_ms,
            sample_rate: signal.sample_rate,
            expected_frequency: signal.expected_frequency,
            amplitude: signal.expected_amplitude,
        })
    }
    
    /// Setup generated signal for injection
    fn setup_generated_signal(&self, config: &SignalConfig) -> Result<SignalInfo, String> {
        let mut generator = self.generator.lock()
            .map_err(|_| "Failed to acquire generator lock".to_string())?;
        
        // Start real-time generation
        generator.start_real_time_generation(config.clone())
            .map_err(|e| format!("Failed to start signal generation: {}", e))?;
        
        Ok(SignalInfo {
            name: format!("{:?}_{:.0}Hz", config.waveform, config.frequency),
            duration_ms: config.duration_ms.unwrap_or(0), // 0 for continuous
            sample_rate: config.sample_rate,
            expected_frequency: Some(config.frequency as f32),
            amplitude: config.amplitude,
        })
    }
    
    /// Inject signal in replace mode
    fn inject_replace_mode(&self, state: &InjectionState, output_buffer: &mut [f32]) -> Result<(), String> {
        if let Some(_) = &state.current_signal {
            // Check if using pre-recorded signal
            if let Ok(mut queue) = self.buffer_queue.lock() {
                if !queue.is_empty() {
                    self.fill_from_queue(&mut queue, output_buffer, state.loop_enabled);
                    return Ok(());
                }
            }
            
            // Use real-time generation
            if let Ok(mut generator) = self.generator.lock() {
                if generator.get_generation_state() == GenerationState::Generating {
                    let generated_buffer = generator.generate_buffer(output_buffer.len());
                    output_buffer.copy_from_slice(&generated_buffer[..output_buffer.len().min(generated_buffer.len())]);
                    return Ok(());
                }
            }
        }
        
        // Fill with silence if no signal available
        output_buffer.fill(0.0);
        Ok(())
    }
    
    /// Inject signal in mix mode
    fn inject_mix_mode(&self, state: &InjectionState, input_buffer: &[f32], output_buffer: &mut [f32]) -> Result<(), String> {
        // Start with input
        output_buffer.copy_from_slice(input_buffer);
        
        // Get signal samples
        let mut signal_buffer = vec![0.0; output_buffer.len()];
        self.inject_replace_mode(state, &mut signal_buffer)?;
        
        // Mix with input based on mix ratio
        for (i, sample) in output_buffer.iter_mut().enumerate() {
            if i < signal_buffer.len() {
                *sample = *sample * (1.0 - state.mix_ratio) + signal_buffer[i] * state.mix_ratio;
            }
        }
        
        Ok(())
    }
    
    /// Inject signal in insert mode
    fn inject_insert_mode(&self, state: &InjectionState, input_buffer: &[f32], output_buffer: &mut [f32]) -> Result<(), String> {
        // For simplicity, insert mode acts like replace mode
        // In a real implementation, this might alternate between input and signal
        self.inject_replace_mode(state, output_buffer)
    }
    
    /// Inject signal in overlay mode
    fn inject_overlay_mode(&self, state: &InjectionState, input_buffer: &[f32], output_buffer: &mut [f32]) -> Result<(), String> {
        // Start with input
        output_buffer.copy_from_slice(input_buffer);
        
        // Get signal samples
        let mut signal_buffer = vec![0.0; output_buffer.len()];
        self.inject_replace_mode(state, &mut signal_buffer)?;
        
        // Overlay signal on top of input
        for (i, sample) in output_buffer.iter_mut().enumerate() {
            if i < signal_buffer.len() {
                *sample = (*sample + signal_buffer[i] * state.mix_ratio).clamp(-1.0, 1.0);
            }
        }
        
        Ok(())
    }
    
    /// Fill output buffer from sample queue
    fn fill_from_queue(&self, queue: &mut VecDeque<f32>, output_buffer: &mut [f32], loop_enabled: bool) {
        for sample in output_buffer.iter_mut() {
            if let Some(queued_sample) = queue.pop_front() {
                *sample = queued_sample;
                
                // If looping, add back to end of queue
                if loop_enabled {
                    queue.push_back(queued_sample);
                }
            } else {
                *sample = 0.0;
            }
        }
    }
    
    /// Get current injection state
    pub fn get_injection_state(&self) -> Result<InjectionState, String> {
        self.injection_state.lock()
            .map(|state| state.clone())
            .map_err(|_| "Failed to acquire injection state lock".to_string())
    }
    
    /// Get available test signals
    pub fn get_available_signals(&self) -> Result<Vec<String>, String> {
        self.test_library.lock()
            .map(|library| library.get_signal_names())
            .map_err(|_| "Failed to acquire test library lock".to_string())
    }
    
    /// Get timestamp in nanoseconds
    fn get_timestamp_ns(&self) -> u64 {
        // In a real implementation, this would use a high-precision timer
        (js_sys::Date::now() * 1_000_000.0) as u64
    }
}

/// Quick injection functions for common use cases
impl SignalInjectionManager {
    /// Quick inject A4 440Hz sine wave
    pub fn inject_a4_tone(&self, duration_ms: u32) -> Result<InjectionResult, String> {
        let config = InjectionConfig {
            signal_name: Some("musical_a4".to_string()),
            mode: InjectionMode::Replace,
            loop_enabled: false,
            auto_stop_ms: Some(duration_ms),
            ..Default::default()
        };
        self.start_injection(config)
    }
    
    /// Quick inject calibration tone
    pub fn inject_calibration_tone(&self, frequency: f32, duration_ms: u32) -> Result<InjectionResult, String> {
        let signal_name = format!("calibration_{}hz", frequency as u32);
        let config = InjectionConfig {
            signal_name: Some(signal_name),
            mode: InjectionMode::Replace,
            loop_enabled: false,
            auto_stop_ms: Some(duration_ms),
            ..Default::default()
        };
        self.start_injection(config)
    }
    
    /// Quick inject pink noise
    pub fn inject_pink_noise(&self, amplitude: f32, duration_ms: u32) -> Result<InjectionResult, String> {
        let config = InjectionConfig {
            signal_name: Some("pink_noise".to_string()),
            mode: InjectionMode::Mix,
            mix_ratio: amplitude,
            loop_enabled: true,
            auto_stop_ms: Some(duration_ms),
            ..Default::default()
        };
        self.start_injection(config)
    }
    
    /// Quick inject frequency sweep
    pub fn inject_frequency_sweep(&self, start_freq: f32, end_freq: f32, duration_ms: u32) -> Result<InjectionResult, String> {
        let signal_name = format!("sweep_{}hz_to_{}hz", start_freq as u32, end_freq as u32);
        let config = InjectionConfig {
            signal_name: Some(signal_name),
            mode: InjectionMode::Replace,
            loop_enabled: false,
            auto_stop_ms: Some(duration_ms),
            ..Default::default()
        };
        self.start_injection(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_injection_manager_creation() {
        let manager = SignalInjectionManager::new(44100.0, None);
        let state = manager.get_injection_state().unwrap();
        assert!(!state.is_active);
    }
    
    #[test]
    fn test_signal_injection_config() {
        let config = InjectionConfig::default();
        assert_eq!(config.mode, InjectionMode::Replace);
        assert_eq!(config.mix_ratio, 1.0);
        assert!(!config.loop_enabled);
    }
    
    #[test]
    fn test_available_signals() {
        let manager = SignalInjectionManager::new(44100.0, None);
        let signals = manager.get_available_signals().unwrap();
        assert!(!signals.is_empty());
        assert!(signals.contains(&"musical_a4".to_string()));
    }
    
    #[test]
    fn test_quick_injection_functions() {
        let manager = SignalInjectionManager::new(44100.0, None);
        
        // Test A4 tone injection
        let result = manager.inject_a4_tone(1000);
        assert!(result.is_ok());
        
        let state = manager.get_injection_state().unwrap();
        assert!(state.is_active);
        
        // Stop injection
        let stop_result = manager.stop_injection();
        assert!(stop_result.is_ok());
        
        let state = manager.get_injection_state().unwrap();
        assert!(!state.is_active);
    }
}