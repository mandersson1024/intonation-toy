use web_sys::{AnalyserNode, AudioContext, AudioWorkletNode, AudioWorkletNodeOptions, GainNode, MediaStreamAudioSourceNode, OscillatorNode, OscillatorType};
use super::audio_pipeline_configs::{SignalGeneratorConfig, TuningForkConfig};
use crate::{common::dev_log, engine::audio::AudioSignalPath};



pub enum SignalPathMode {
    Off,
    TuningForkMode, 
    TestSignalMode,
}

/// Audio pipeline with simplified signal path architecture
/// 
/// This struct manages audio processing using the new AudioSignalPath approach,
/// providing a cleaner separation between signal routing and configuration management.
/// The pipeline creates and manages all Web Audio API nodes through the signal path.
pub struct NewAudioPipeline {
    pub signal_path: AudioSignalPath,
    audio_context: AudioContext,
}

impl NewAudioPipeline {
    /// Create a new audio pipeline with simplified signal path
    /// 
    /// Creates all necessary audio nodes and connects them through the AudioSignalPath.
    /// The worklet module must already be loaded in the AudioContext before calling this method.
    /// 
    /// # Parameters
    /// 
    /// * `audio_context` - Reference to the AudioContext with worklet module loaded
    /// * `media_stream` - MediaStream from microphone input
    /// 
    /// # Returns
    /// 
    /// Returns `Result<Self, String>` where:
    /// - On success: NewAudioPipeline ready for audio processing
    /// - On error: String describing what went wrong
    pub fn new(audio_context: &AudioContext, media_stream: &web_sys::MediaStream) -> Result<Self, String> {
        let input_node = audio_context.create_media_stream_source(media_stream)
            .map_err(|e| format!("Failed to create media stream source: {:?}", e))?;

        let worklet_node = NewAudioPipeline::create_worklet_node(audio_context)?;

        let signal_path = AudioSignalPath::new(audio_context.clone(), input_node, worklet_node);

        // Configure analyser with FFT size of 128
        signal_path.analyser.set_fft_size(128);
        signal_path.analyser.set_smoothing_time_constant(0.0);
        
        {
            // Configure tuning fork oscillator with custom waveform
            let n = 16;
            let mut real = vec![0.0f32; n];
            let mut imag = vec![0.0f32; n];
            
            let amps: [f32; 9] = [
                0.0,   // DC offset
                1.0,   // fundamental
                0.85,  // 2nd
                0.55,  // 3rd
                0.40,  // 4th
                0.25,  // 5th
                0.18,  // 6th
                0.12,  // 7th
                0.08   // 8th
            ];

            for (i, &amp) in amps.iter().enumerate() {
                real[i] = amp;
            }

            let periodic_wave = audio_context.create_periodic_wave(&mut real, &mut imag)
                .map_err(|_| "Failed to create periodic wave".to_string())?;
            
            signal_path.tuning_fork_osc.set_periodic_wave(&periodic_wave);
            signal_path.tuning_fork_osc.frequency().set_value(440.0); // A4 default
            signal_path.tuning_fork_gain.gain().set_value(0.0); // Start muted
        }
        
        // Configure test signal oscillator
        signal_path.test_signal_osc.set_type(OscillatorType::Sine);
        signal_path.test_signal_osc.frequency().set_value(440.0); // A4 default
        
        dev_log!("✓ NewAudioPipeline nodes configured");

        let mut pipeline = Self {
            signal_path,
            audio_context: audio_context.clone(),
        };

        pipeline.set_signal_path_mode(SignalPathMode::Off);
        

        Ok(pipeline)
    }

    /// Start the audio pipeline
    /// 
    /// Starts the oscillators and sets the initial signal path mode.
    /// This method should be called after the pipeline is created to begin audio processing.
    pub fn run(&mut self) -> Result<(), String> {
        // Start the oscillators
        self.signal_path.tuning_fork_osc.start()
            .map_err(|_| "Failed to start tuning fork oscillator".to_string())?;
        self.signal_path.test_signal_osc.start()
            .map_err(|_| "Failed to start test signal oscillator".to_string())?;
        
        // Set initial mode to tuning fork mode
        self.set_signal_path_mode(SignalPathMode::TuningForkMode);
        
        Ok(())
    }

    /// Set the signal path mode for audio routing
    /// 
    /// Configures the audio pipeline to route signals according to the specified mode.
    /// This affects which audio sources are active and how they're routed through the pipeline.
    /// 
    /// # Parameters
    /// 
    /// * `mode` - The signal path mode to configure
    pub fn set_signal_path_mode(&mut self, mode: SignalPathMode) {
        match mode {
            SignalPathMode::Off => {
                self.signal_path.user_input_mute.gain().set_value(0.0);
                self.signal_path.test_signal_mute.gain().set_value(0.0);
            }
            SignalPathMode::TuningForkMode => {
                self.signal_path.user_input_mute.gain().set_value(1.0);
                self.signal_path.test_signal_mute.gain().set_value(0.0);
            }
            SignalPathMode::TestSignalMode => {
                self.signal_path.user_input_mute.gain().set_value(0.0);
                self.signal_path.test_signal_mute.gain().set_value(1.0);
            }
        }
    }

    pub fn update_tuning_fork_config(&mut self, config: super::audio_pipeline_configs::TuningForkConfig) {
        self.signal_path.tuning_fork_osc.frequency().set_value(config.frequency);
        self.ramp_tuning_fork_gain(config.volume);
    }

    fn ramp_tuning_fork_gain(&self, target: f32) {
        if self.signal_path.tuning_fork_gain.gain().set_target_at_time(target, self.audio_context.current_time(), 0.05).is_err() {
            self.signal_path.tuning_fork_gain.gain().set_value(target);
        }
    }

    /// Execute test signal configurations with privileged access
    /// 
    /// This method provides direct control over test signal generation,
    /// bypassing normal validation checks.
    /// 
    /// # Arguments
    /// 
    /// * `test_signal_configs` - Test signal configurations to execute
    /// 
    /// # Returns
    /// 
    /// Returns `Result<(), String>` indicating success or failure
    #[cfg(debug_assertions)]
    pub fn execute_test_signal_configuration(
        &mut self,
        config: &crate::presentation::ConfigureTestSignal
    ) -> Result<(), String> {
        if config.enabled {
            self.signal_path.test_signal_osc.frequency().set_value(config.frequency);
            self.signal_path.test_signal_gain.gain().set_value(config.volume / 100.0);
            self.set_signal_path_mode(SignalPathMode::TestSignalMode);
        } else {
            self.set_signal_path_mode(SignalPathMode::TuningForkMode);
        }
        Ok(())
    }

    /// Create AudioWorkletNode with standard configuration
    /// 
    /// This method creates an AudioWorkletNode using standard configuration options.
    /// The worklet module must already be loaded in the AudioContext before calling this method.
    /// 
    /// # Parameters
    /// - `audio_context`: Reference to the AudioContext with worklet module loaded
    /// 
    /// # Returns
    /// Returns `Result<AudioWorkletNode, String>` where:
    /// - On success: AudioWorkletNode ready for use
    /// - On error: String describing what went wrong
    fn create_worklet_node(audio_context: &AudioContext) -> Result<AudioWorkletNode, String> {
        dev_log!("Creating AudioWorkletNode with standard configuration");
        
        // Create AudioWorkletNode with default options
        let options = AudioWorkletNodeOptions::new();
        options.set_number_of_inputs(1);
        options.set_number_of_outputs(1);
        
        // Set channel configuration
        let output_channels = js_sys::Array::of1(&js_sys::Number::from(1u32));
        options.set_channel_count(1);
        options.set_channel_count_mode(web_sys::ChannelCountMode::Explicit);
        options.set_channel_interpretation(web_sys::ChannelInterpretation::Speakers);
        options.set_output_channel_count(&output_channels);
        
        // Create the AudioWorkletNode with the registered processor
        let worklet_node = AudioWorkletNode::new_with_options(audio_context, "pitch-processor", &options)
            .map_err(|e| format!("Failed to create AudioWorkletNode 'pitch-processor': {:?}", e))?;
        
        dev_log!("✓ AudioWorkletNode created successfully");
        Ok(worklet_node)
    }
}
