// Signal Generator Implementation - STORY-016
// Test signal generation for pitch detection testing and calibration

use std::f64::consts::PI;
use std::sync::{Arc, Mutex};
use std::error::Error;
use std::fmt;
use wasm_bindgen::prelude::*;
use js_sys::Math;
use std::time::{Duration, Instant};
use crate::modules::audio_foundations::audio_events::*;

/// Signal generator trait for creating test audio signals
pub trait SignalGenerator: Send + Sync {
    /// Generate sine wave signal
    fn generate_sine(&self, freq: f64, amplitude: f32, duration_ms: u32) -> Result<Vec<f32>, SignalError>;
    
    /// Generate sawtooth wave signal
    fn generate_sawtooth(&self, freq: f64, amplitude: f32, duration_ms: u32) -> Result<Vec<f32>, SignalError>;
    
    /// Generate square wave signal
    fn generate_square(&self, freq: f64, amplitude: f32, duration_ms: u32) -> Result<Vec<f32>, SignalError>;
    
    /// Generate triangle wave signal
    fn generate_triangle(&self, freq: f64, amplitude: f32, duration_ms: u32) -> Result<Vec<f32>, SignalError>;
    
    /// Generate frequency sweep signal
    fn generate_sweep(&self, start_freq: f64, end_freq: f64, amplitude: f32, duration_ms: u32) -> Result<Vec<f32>, SignalError>;
    
    /// Generate pink noise
    fn generate_pink_noise(&self, amplitude: f32, duration_ms: u32) -> Result<Vec<f32>, SignalError>;
    
    /// Generate white noise
    fn generate_white_noise(&self, amplitude: f32, duration_ms: u32) -> Result<Vec<f32>, SignalError>;
    
    /// Start real-time signal generation
    fn start_real_time_generation(&mut self, config: SignalConfig) -> Result<(), SignalError>;
    
    /// Stop real-time signal generation
    fn stop_real_time_generation(&mut self) -> Result<(), SignalError>;
    
    /// Check if real-time generation is active
    fn is_real_time_active(&self) -> bool;
    
    /// Get next real-time buffer
    fn get_next_buffer(&mut self, buffer_size: usize) -> Result<Vec<f32>, SignalError>;
}

/// Waveform types supported by the signal generator
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WaveformType {
    Sine,
    Sawtooth,
    Square,
    Triangle,
    Sweep,
    PinkNoise,
    WhiteNoise,
}

/// Signal generation configuration
#[derive(Debug, Clone)]
pub struct SignalConfig {
    pub waveform: WaveformType,
    pub frequency: f64,
    pub amplitude: f32,
    pub duration_ms: Option<u32>, // None for continuous generation
    pub sweep_end_freq: Option<f64>, // For sweep waveforms
    pub sample_rate: u32,
    pub phase_offset: f64,
}

impl Default for SignalConfig {
    fn default() -> Self {
        Self {
            waveform: WaveformType::Sine,
            frequency: 440.0, // A4 note
            amplitude: 0.5,
            duration_ms: Some(1000), // 1 second
            sweep_end_freq: None,
            sample_rate: 44100,
            phase_offset: 0.0,
        }
    }
}

/// Real-time generation state
#[derive(Debug, Clone, PartialEq)]
pub enum GenerationState {
    Stopped,
    Generating,
    Error(String),
}

/// Signal generation errors
#[derive(Debug, Clone)]
pub enum SignalError {
    InvalidFrequency(String),
    InvalidAmplitude(String),
    InvalidDuration(String),
    InvalidSampleRate(String),
    GenerationFailed(String),
    NotSupported(String),
    RealTimeError(String),
}

impl fmt::Display for SignalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignalError::InvalidFrequency(msg) => write!(f, "Invalid frequency: {}", msg),
            SignalError::InvalidAmplitude(msg) => write!(f, "Invalid amplitude: {}", msg),
            SignalError::InvalidDuration(msg) => write!(f, "Invalid duration: {}", msg),
            SignalError::InvalidSampleRate(msg) => write!(f, "Invalid sample rate: {}", msg),
            SignalError::GenerationFailed(msg) => write!(f, "Generation failed: {}", msg),
            SignalError::NotSupported(msg) => write!(f, "Not supported: {}", msg),
            SignalError::RealTimeError(msg) => write!(f, "Real-time generation error: {}", msg),
        }
    }
}

impl Error for SignalError {}

/// Web-based signal generator implementation
pub struct WebSignalGenerator {
    sample_rate: u32,
    real_time_config: Option<SignalConfig>,
    phase_accumulator: f64,
    pink_noise_state: PinkNoiseState,
    random_seed: u32,
    is_active: bool,
}

/// Pink noise generator state for proper pink noise generation
struct PinkNoiseState {
    b0: f32,
    b1: f32,
    b2: f32,
    b3: f32,
    b4: f32,
    b5: f32,
    b6: f32,
}

impl Default for PinkNoiseState {
    fn default() -> Self {
        Self {
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            b4: 0.0,
            b5: 0.0,
            b6: 0.0,
        }
    }
}

impl WebSignalGenerator {
    /// Create new signal generator with specified sample rate
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            real_time_config: None,
            phase_accumulator: 0.0,
            pink_noise_state: PinkNoiseState::default(),
            random_seed: 12345,
            is_active: false,
        }
    }
    
    /// Validate signal parameters
    fn validate_params(&self, freq: f64, amplitude: f32, duration_ms: u32) -> Result<(), SignalError> {
        if freq <= 0.0 || freq > (self.sample_rate as f64 / 2.0) {
            return Err(SignalError::InvalidFrequency(
                format!("Frequency {} must be between 0 and {} Hz", freq, self.sample_rate / 2.0)
            ));
        }
        
        if amplitude < 0.0 || amplitude > 1.0 {
            return Err(SignalError::InvalidAmplitude(
                format!("Amplitude {} must be between 0.0 and 1.0", amplitude)
            ));
        }
        
        if duration_ms == 0 || duration_ms > 300_000 { // Max 5 minutes
            return Err(SignalError::InvalidDuration(
                format!("Duration {} ms must be between 1 and 300000 ms", duration_ms)
            ));
        }
        
        Ok(())
    }
    
    /// Calculate buffer size for duration
    fn calculate_buffer_size(&self, duration_ms: u32) -> usize {
        (self.sample_rate as f64 * duration_ms as f64 / 1000.0) as usize
    }
    
    /// Generate white noise sample
    fn white_noise_sample(&mut self) -> f32 {
        // Linear congruential generator
        self.random_seed = self.random_seed.wrapping_mul(1103515245).wrapping_add(12345);
        let normalized = (self.random_seed as f32) / (u32::MAX as f32);
        (normalized - 0.5) * 2.0 // Range: -1.0 to 1.0
    }
    
    /// Generate pink noise sample using Paul Kellett's algorithm
    fn pink_noise_sample(&mut self) -> f32 {
        let white = self.white_noise_sample();
        
        self.pink_noise_state.b0 = 0.99886 * self.pink_noise_state.b0 + white * 0.0555179;
        self.pink_noise_state.b1 = 0.99332 * self.pink_noise_state.b1 + white * 0.0750759;
        self.pink_noise_state.b2 = 0.96900 * self.pink_noise_state.b2 + white * 0.1538520;
        self.pink_noise_state.b3 = 0.86650 * self.pink_noise_state.b3 + white * 0.3104856;
        self.pink_noise_state.b4 = 0.55000 * self.pink_noise_state.b4 + white * 0.5329522;
        self.pink_noise_state.b5 = -0.7616 * self.pink_noise_state.b5 - white * 0.0168980;
        
        let pink = self.pink_noise_state.b0 + self.pink_noise_state.b1 + self.pink_noise_state.b2 + 
                   self.pink_noise_state.b3 + self.pink_noise_state.b4 + self.pink_noise_state.b5 + 
                   self.pink_noise_state.b6 + white * 0.5362;
        
        self.pink_noise_state.b6 = white * 0.115926;
        
        pink * 0.11 // Scale to reasonable amplitude
    }
}

impl SignalGenerator for WebSignalGenerator {
    fn generate_sine(&self, freq: f64, amplitude: f32, duration_ms: u32) -> Result<Vec<f32>, SignalError> {
        self.validate_params(freq, amplitude, duration_ms)?;
        
        let buffer_size = self.calculate_buffer_size(duration_ms);
        let mut buffer = Vec::with_capacity(buffer_size);
        
        let angular_freq = 2.0 * PI * freq / self.sample_rate as f64;
        
        for i in 0..buffer_size {
            let sample = amplitude * (angular_freq * i as f64).sin() as f32;
            buffer.push(sample);
        }
        
        Ok(buffer)
    }
    
    fn generate_sawtooth(&self, freq: f64, amplitude: f32, duration_ms: u32) -> Result<Vec<f32>, SignalError> {
        self.validate_params(freq, amplitude, duration_ms)?;
        
        let buffer_size = self.calculate_buffer_size(duration_ms);
        let mut buffer = Vec::with_capacity(buffer_size);
        
        let period = self.sample_rate as f64 / freq;
        
        for i in 0..buffer_size {
            let phase = (i as f64 % period) / period;
            let sample = amplitude * (2.0 * phase - 1.0) as f32;
            buffer.push(sample);
        }
        
        Ok(buffer)
    }
    
    fn generate_square(&self, freq: f64, amplitude: f32, duration_ms: u32) -> Result<Vec<f32>, SignalError> {
        self.validate_params(freq, amplitude, duration_ms)?;
        
        let buffer_size = self.calculate_buffer_size(duration_ms);
        let mut buffer = Vec::with_capacity(buffer_size);
        
        let period = self.sample_rate as f64 / freq;
        
        for i in 0..buffer_size {
            let phase = (i as f64 % period) / period;
            let sample = if phase < 0.5 { amplitude } else { -amplitude };
            buffer.push(sample);
        }
        
        Ok(buffer)
    }
    
    fn generate_triangle(&self, freq: f64, amplitude: f32, duration_ms: u32) -> Result<Vec<f32>, SignalError> {
        self.validate_params(freq, amplitude, duration_ms)?;
        
        let buffer_size = self.calculate_buffer_size(duration_ms);
        let mut buffer = Vec::with_capacity(buffer_size);
        
        let period = self.sample_rate as f64 / freq;
        
        for i in 0..buffer_size {
            let phase = (i as f64 % period) / period;
            let sample = if phase < 0.5 {
                amplitude * (4.0 * phase - 1.0) as f32
            } else {
                amplitude * (3.0 - 4.0 * phase) as f32
            };
            buffer.push(sample);
        }
        
        Ok(buffer)
    }
    
    fn generate_sweep(&self, start_freq: f64, end_freq: f64, amplitude: f32, duration_ms: u32) -> Result<Vec<f32>, SignalError> {
        self.validate_params(start_freq, amplitude, duration_ms)?;
        self.validate_params(end_freq, amplitude, duration_ms)?;
        
        let buffer_size = self.calculate_buffer_size(duration_ms);
        let mut buffer = Vec::with_capacity(buffer_size);
        
        let total_samples = buffer_size as f64;
        let freq_delta = end_freq - start_freq;
        
        let mut phase = 0.0;
        
        for i in 0..buffer_size {
            let progress = i as f64 / total_samples;
            let current_freq = start_freq + freq_delta * progress;
            let angular_freq = 2.0 * PI * current_freq / self.sample_rate as f64;
            
            phase += angular_freq;
            let sample = amplitude * phase.sin() as f32;
            buffer.push(sample);
        }
        
        Ok(buffer)
    }
    
    fn generate_pink_noise(&self, amplitude: f32, duration_ms: u32) -> Result<Vec<f32>, SignalError> {
        if amplitude < 0.0 || amplitude > 1.0 {
            return Err(SignalError::InvalidAmplitude(
                format!("Amplitude {} must be between 0.0 and 1.0", amplitude)
            ));
        }
        
        let buffer_size = self.calculate_buffer_size(duration_ms);
        let mut buffer = Vec::with_capacity(buffer_size);
        let mut generator = Self::new(self.sample_rate);
        
        for _ in 0..buffer_size {
            let sample = amplitude * generator.pink_noise_sample();
            buffer.push(sample);
        }
        
        Ok(buffer)
    }
    
    fn generate_white_noise(&self, amplitude: f32, duration_ms: u32) -> Result<Vec<f32>, SignalError> {
        if amplitude < 0.0 || amplitude > 1.0 {
            return Err(SignalError::InvalidAmplitude(
                format!("Amplitude {} must be between 0.0 and 1.0", amplitude)
            ));
        }
        
        let buffer_size = self.calculate_buffer_size(duration_ms);
        let mut buffer = Vec::with_capacity(buffer_size);
        let mut generator = Self::new(self.sample_rate);
        
        for _ in 0..buffer_size {
            let sample = amplitude * generator.white_noise_sample();
            buffer.push(sample);
        }
        
        Ok(buffer)
    }
    
    fn start_real_time_generation(&mut self, config: SignalConfig) -> Result<(), SignalError> {
        // Validate configuration
        if let Some(duration) = config.duration_ms {
            self.validate_params(config.frequency, config.amplitude, duration)?;
        } else {
            // For continuous generation, just validate frequency and amplitude
            if config.frequency <= 0.0 || config.frequency > (self.sample_rate as f64 / 2.0) {
                return Err(SignalError::InvalidFrequency(
                    format!("Frequency {} Hz must be between 0 and {} Hz", config.frequency, self.sample_rate / 2)
                ));
            }
            if config.amplitude < 0.0 || config.amplitude > 1.0 {
                return Err(SignalError::InvalidAmplitude(
                    format!("Amplitude {} must be between 0.0 and 1.0", config.amplitude)
                ));
            }
        }
        
        self.real_time_config = Some(config);
        self.phase_accumulator = 0.0;
        self.is_active = true;
        
        Ok(())
    }
    
    fn stop_real_time_generation(&mut self) -> Result<(), SignalError> {
        self.real_time_config = None;
        self.is_active = false;
        self.phase_accumulator = 0.0;
        Ok(())
    }
    
    fn is_real_time_active(&self) -> bool {
        self.is_active && self.real_time_config.is_some()
    }
    
    fn get_next_buffer(&mut self, buffer_size: usize) -> Result<Vec<f32>, SignalError> {
        if !self.is_real_time_active() {
            return Err(SignalError::RealTimeError("Real-time generation not active".to_string()));
        }
        
        let config = self.real_time_config.clone().unwrap();
        let mut buffer = Vec::with_capacity(buffer_size);
        
        match config.waveform {
            WaveformType::Sine => {
                let angular_freq = 2.0 * PI * config.frequency / self.sample_rate as f64;
                
                for _ in 0..buffer_size {
                    let sample = config.amplitude * (self.phase_accumulator + config.phase_offset).sin() as f32;
                    buffer.push(sample);
                    self.phase_accumulator += angular_freq;
                    
                    // Prevent phase accumulator overflow
                    if self.phase_accumulator >= 2.0 * PI {
                        self.phase_accumulator -= 2.0 * PI;
                    }
                }
            }
            
            WaveformType::Sawtooth => {
                let period = self.sample_rate as f64 / config.frequency;
                
                for _ in 0..buffer_size {
                    let phase = (self.phase_accumulator % period) / period;
                    let sample = config.amplitude * (2.0 * phase - 1.0) as f32;
                    buffer.push(sample);
                    self.phase_accumulator += 1.0;
                }
            }
            
            WaveformType::Square => {
                let period = self.sample_rate as f64 / config.frequency;
                
                for _ in 0..buffer_size {
                    let phase = (self.phase_accumulator % period) / period;
                    let sample = if phase < 0.5 { config.amplitude } else { -config.amplitude };
                    buffer.push(sample);
                    self.phase_accumulator += 1.0;
                }
            }
            
            WaveformType::Triangle => {
                let period = self.sample_rate as f64 / config.frequency;
                
                for _ in 0..buffer_size {
                    let phase = (self.phase_accumulator % period) / period;
                    let sample = if phase < 0.5 {
                        config.amplitude * (4.0 * phase - 1.0) as f32
                    } else {
                        config.amplitude * (3.0 - 4.0 * phase) as f32
                    };
                    buffer.push(sample);
                    self.phase_accumulator += 1.0;
                }
            }
            
            WaveformType::PinkNoise => {
                for _ in 0..buffer_size {
                    let sample = config.amplitude * self.pink_noise_sample();
                    buffer.push(sample);
                }
            }
            
            WaveformType::WhiteNoise => {
                for _ in 0..buffer_size {
                    let sample = config.amplitude * self.white_noise_sample();
                    buffer.push(sample);
                }
            }
            
            WaveformType::Sweep => {
                // For real-time generation, sweep is not supported yet
                // Fall back to sine wave
                let angular_freq = 2.0 * PI * config.frequency / self.sample_rate as f64;
                
                for _ in 0..buffer_size {
                    let sample = config.amplitude * (self.phase_accumulator + config.phase_offset).sin() as f32;
                    buffer.push(sample);
                    self.phase_accumulator += angular_freq;
                    
                    if self.phase_accumulator >= 2.0 * PI {
                        self.phase_accumulator -= 2.0 * PI;
                    }
                }
            }
        }
        
        Ok(buffer)
    }
}

/// Pre-recorded test signal library
pub struct TestSignalLibrary {
    sample_rate: u32,
    generator: WebSignalGenerator,
}

impl TestSignalLibrary {
    /// Create new test signal library
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            generator: WebSignalGenerator::new(sample_rate),
        }
    }
    
    /// Generate musical notes (A0 to C8)
    pub fn generate_musical_note(&self, note: &str, octave: u8, duration_ms: u32) -> Result<Vec<f32>, SignalError> {
        let frequency = self.note_to_frequency(note, octave)?;
        self.generator.generate_sine(frequency, 0.5, duration_ms)
    }
    
    /// Generate chord (multiple frequencies combined)
    pub fn generate_chord(&self, frequencies: &[f64], duration_ms: u32) -> Result<Vec<f32>, SignalError> {
        if frequencies.is_empty() {
            return Err(SignalError::InvalidFrequency("No frequencies provided".to_string()));
        }
        
        let buffer_size = self.generator.calculate_buffer_size(duration_ms);
        let mut buffer = vec![0.0; buffer_size];
        let amplitude_per_freq = 0.5 / frequencies.len() as f32;
        
        for &freq in frequencies {
            let component = self.generator.generate_sine(freq, amplitude_per_freq, duration_ms)?;
            for (i, sample) in component.iter().enumerate() {
                if i < buffer.len() {
                    buffer[i] += sample;
                }
            }
        }
        
        Ok(buffer)
    }
    
    /// Generate tuning reference signals (A440, etc.)
    pub fn generate_tuning_reference(&self, reference: TuningReference, duration_ms: u32) -> Result<Vec<f32>, SignalError> {
        let frequency = match reference {
            TuningReference::A440 => 440.0,
            TuningReference::A442 => 442.0,
            TuningReference::A432 => 432.0,
            TuningReference::MiddleC => 261.626, // C4
        };
        
        self.generator.generate_sine(frequency, 0.7, duration_ms)
    }
    
    /// Convert note name and octave to frequency
    pub fn note_to_frequency(&self, note: &str, octave: u8) -> Result<f64, SignalError> {
        let semitone_offset = match note.to_uppercase().as_str() {
            "C" => 0,
            "C#" | "DB" => 1,
            "D" => 2,
            "D#" | "EB" => 3,
            "E" => 4,
            "F" => 5,
            "F#" | "GB" => 6,
            "G" => 7,
            "G#" | "AB" => 8,
            "A" => 9,
            "A#" | "BB" => 10,
            "B" => 11,
            _ => return Err(SignalError::InvalidFrequency(format!("Unknown note: {}", note))),
        };
        
        if octave > 8 {
            return Err(SignalError::InvalidFrequency(format!("Octave {} too high (max 8)", octave)));
        }
        
        // A4 = 440 Hz, use equal temperament
        let midi_note = (octave as i32 + 1) * 12 + semitone_offset;
        let a4_midi = 69; // A4 MIDI note number
        let frequency = 440.0 * 2_f64.powf((midi_note - a4_midi) as f64 / 12.0);
        
        Ok(frequency)
    }
}

/// Tuning reference frequencies
#[derive(Debug, Clone, Copy)]
pub enum TuningReference {
    A440, // Standard tuning
    A442, // Some orchestras
    A432, // Alternative tuning
    MiddleC, // C4
}

/// Signal injection interface for audio processing pipeline
pub struct SignalInjector {
    generator: Arc<Mutex<WebSignalGenerator>>,
    injection_active: bool,
    injection_config: Option<SignalConfig>,
}

impl SignalInjector {
    /// Create new signal injector
    pub fn new(sample_rate: u32) -> Self {
        Self {
            generator: Arc::new(Mutex::new(WebSignalGenerator::new(sample_rate))),
            injection_active: false,
            injection_config: None,
        }
    }
    
    /// Start signal injection
    pub fn start_injection(&mut self, config: SignalConfig) -> Result<(), SignalError> {
        let mut generator = self.generator.lock().map_err(|_| 
            SignalError::RealTimeError("Failed to lock generator".to_string()))?;
            
        generator.start_real_time_generation(config.clone())?;
        self.injection_config = Some(config);
        self.injection_active = true;
        
        Ok(())
    }
    
    /// Stop signal injection
    pub fn stop_injection(&mut self) -> Result<(), SignalError> {
        let mut generator = self.generator.lock().map_err(|_| 
            SignalError::RealTimeError("Failed to lock generator".to_string()))?;
            
        generator.stop_real_time_generation()?;
        self.injection_active = false;
        self.injection_config = None;
        
        Ok(())
    }
    
    /// Inject signal into audio buffer (replaces or mixes)
    pub fn inject_into_buffer(&mut self, buffer: &mut [f32], mix_ratio: f32) -> Result<(), SignalError> {
        if !self.injection_active {
            return Ok(()); // No injection, buffer unchanged
        }
        
        let mut generator = self.generator.lock().map_err(|_| 
            SignalError::RealTimeError("Failed to lock generator".to_string()))?;
            
        let signal_buffer = generator.get_next_buffer(buffer.len())?;
        
        // Mix or replace based on mix_ratio (0.0 = replace, 1.0 = original only)
        for (i, sample) in buffer.iter_mut().enumerate() {
            if i < signal_buffer.len() {
                *sample = *sample * mix_ratio + signal_buffer[i] * (1.0 - mix_ratio);
            }
        }
        
        Ok(())
    }
    
    /// Check if injection is active
    pub fn is_injection_active(&self) -> bool {
        self.injection_active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sine_wave_generation() {
        let generator = WebSignalGenerator::new(44100);
        let signal = generator.generate_sine(440.0, 0.5, 1000);
        
        assert_eq!(signal.len(), 44100); // 1 second at 44.1kHz
        assert!(signal.iter().all(|&x| x.abs() <= 0.5)); // Within amplitude bounds
    }
    
    #[test]
    fn test_square_wave_generation() {
        let generator = WebSignalGenerator::new(44100);
        let signal = generator.generate_square(440.0, 0.5, 100);
        
        assert!(!signal.is_empty());
        assert!(signal.iter().all(|&x| x == 0.5 || x == -0.5)); // Only two levels
    }
    
    #[test]
    fn test_real_time_generation() {
        let mut generator = WebSignalGenerator::new(44100);
        let config = SignalConfig {
            waveform: WaveformType::Sine,
            frequency: 440.0,
            amplitude: 0.5,
            duration_ms: None,
            sweep_end_freq: None,
            sample_rate: 44100,
            phase_offset: 0.0,
        };
        
        assert!(generator.start_real_time_generation(config).is_ok());
        assert_eq!(generator.get_generation_state(), GenerationState::Generating);
        
        let buffer = generator.generate_buffer(1024);
        assert_eq!(buffer.len(), 1024);
        assert!(buffer.iter().any(|&x| x != 0.0)); // Should contain signal
        
        assert!(generator.stop_real_time_generation().is_ok());
        assert_eq!(generator.get_generation_state(), GenerationState::Stopped);
    }
    
    #[test]
    fn test_parameter_validation() {
        let generator = WebSignalGenerator::new(44100);
        
        // Invalid frequency (too high)
        let signal = generator.generate_sine(50000.0, 0.5, 100);
        assert!(signal.is_empty());
        
        // Invalid amplitude
        let signal = generator.generate_sine(440.0, 2.0, 100);
        assert!(signal.is_empty());
    }
    
    #[test]
    fn test_sweep_generation() {
        let generator = WebSignalGenerator::new(44100);
        let signal = generator.generate_sweep(100.0, 1000.0, 0.5, 1000);
        
        assert_eq!(signal.len(), 44100);
        assert!(signal.iter().all(|&x| x.abs() <= 0.5));
    }
}