use web_sys::{AnalyserNode, AudioContext, AudioWorkletNode, AudioWorkletNodeOptions, GainNode, MediaStreamAudioSourceNode, OscillatorNode, OscillatorType};
use super::audio_pipeline_configs::{SignalGeneratorConfig, TuningForkConfig};
use crate::{common::dev_log, engine::audio::AudioSignalPath};



pub enum SignalPathMode { 
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

        let worklet_node = AudioPipeline::create_worklet_node(audio_context)?;

        let signal_path = AudioSignalPath::new(audio_context.clone(), input_node, worklet_node);

        let pipeline = Self {
            signal_path,
        };

        Ok(pipeline)
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
            SignalPathMode::TuningForkMode => {
                self.signal_path.input_gain.gain().set_value(1.0);
                self.signal_path.test_signal_gain.gain().set_value(0.0);
                dev_log!("Signal path mode set to TuningForkMode - input enabled, test signal disabled");
            }
            SignalPathMode::TestSignalMode => {
                self.signal_path.input_gain.gain().set_value(0.0);
                self.signal_path.test_signal_gain.gain().set_value(1.0);
                dev_log!("Signal path mode set to TestSignalMode - input disabled, test signal enabled");
            }
        }
    }
}

pub struct AudioPipeline {
    pub input_node: MediaStreamAudioSourceNode,
    pub input_gain_node: GainNode,
    pub worklet_node: AudioWorkletNode,
    pub analyser_node: AnalyserNode,
    pub test_signal_oscillator_node: OscillatorNode,
    pub test_signal_gain_node: GainNode,
    pub tuning_fork_oscillator_node: OscillatorNode,
    pub tuning_fork_gain_node: GainNode,

    // Tuning fork nodes (integrated directly)
    tuning_fork_config: TuningForkConfig,
    // Test signal nodes (integrated directly)
    test_signal_config: SignalGeneratorConfig,
    // Other nodes
    pub mixer_gain_node: GainNode,
    pub output_to_speakers: bool,
}

impl AudioPipeline {
    pub fn new(audio_context: &AudioContext, media_stream: &web_sys::MediaStream) -> Result<Self, String> {
        // Create all nodes first
        let (input_node, input_gain_node, worklet_node, analyser_node, 
             test_signal_oscillator_node, test_signal_gain_node,
             tuning_fork_oscillator_node, tuning_fork_gain_node,
             mixer_gain_node) = Self::create_nodes(audio_context, media_stream)?;
        
        let mut pipeline = Self {
            input_node,
            input_gain_node,
            worklet_node,
            analyser_node,
            test_signal_oscillator_node,
            test_signal_gain_node,
            tuning_fork_oscillator_node,
            tuning_fork_gain_node,
            mixer_gain_node,
            tuning_fork_config: TuningForkConfig { frequency: 440.0, volume: 0.0 },
            test_signal_config: SignalGeneratorConfig { enabled: false, frequency: 440.0, amplitude: 0.0 },
            output_to_speakers: false,
        };
        
        // Connect all nodes
        pipeline.connect_all_nodes(audio_context)?;
        
        Ok(pipeline)
    }
    
    /// Create all audio nodes without connecting them
    /// Returns a tuple of all created nodes
    fn create_nodes(audio_context: &AudioContext, media_stream: &web_sys::MediaStream) 
        -> Result<(MediaStreamAudioSourceNode, GainNode, AudioWorkletNode, AnalyserNode, 
                   OscillatorNode, GainNode, OscillatorNode, GainNode, GainNode), String> {
        dev_log!("Creating audio nodes");
        
        // Create microphone source node
        let input_node = audio_context.create_media_stream_source(media_stream)
            .map_err(|e| format!("Failed to create media stream source: {:?}", e))?;
        
        // Create input gain node
        let input_gain_node = audio_context
            .create_gain()
            .map_err(|_| "Failed to create microphone gain node".to_string())?;
        input_gain_node.gain().set_value(1.0);
        
        // Create worklet node
        let worklet_node = Self::create_worklet_node(audio_context)?;
        
        // Create mixer gain node
        let mixer_gain_node = audio_context
            .create_gain()
            .map_err(|_| "Failed to create mixer gain node".to_string())?;
        mixer_gain_node.gain().set_value(1.0);
        
        // Create tuning fork oscillator with custom waveform
        let tuning_fork_oscillator_node = audio_context.create_oscillator()
            .map_err(|_| "Failed to create tuning fork oscillator".to_string())?;
        
        // Create custom waveform with harmonic series
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
        
        tuning_fork_oscillator_node.set_periodic_wave(&periodic_wave);
        tuning_fork_oscillator_node.frequency().set_value(440.0); // A4 default
        tuning_fork_oscillator_node.start()
            .map_err(|_| "Failed to start tuning fork oscillator".to_string())?;
        
        // Create tuning fork gain node
        let tuning_fork_gain_node = audio_context.create_gain()
            .map_err(|_| "Failed to create tuning fork gain node".to_string())?;
        tuning_fork_gain_node.gain().set_value(0.0); // Start muted
        
        dev_log!("✓ Tuning fork oscillator and gain created");
        
        // Create test signal oscillator
        let test_signal_oscillator_node = audio_context.create_oscillator()
            .map_err(|_| "Failed to create test signal oscillator".to_string())?;
        
        test_signal_oscillator_node.set_type(OscillatorType::Sine);
        test_signal_oscillator_node.frequency().set_value(440.0); // A4 default
        test_signal_oscillator_node.start()
            .map_err(|_| "Failed to start test signal oscillator".to_string())?;
        
        // Create test signal gain node
        let test_signal_gain_node = audio_context.create_gain()
            .map_err(|_| "Failed to create test signal gain node".to_string())?;
        test_signal_gain_node.gain().set_value(0.0); // Start disabled
        
        dev_log!("✓ Test signal oscillator and gain created");
        
        // Create analyser node for volume analysis
        let analyser_node = audio_context
            .create_analyser()
            .map_err(|e| format!("Failed to create analyser node: {:?}", e))?;
        
        // Configure analyser node with FFT size of 128
        analyser_node.set_fft_size(128);
        // Set smoothing time constant to 0.0 for real-time analysis
        analyser_node.set_smoothing_time_constant(0.0);
        
        dev_log!("✓ AudioPipeline analyser node created with FFT size: 128");
        
        Ok((input_node, input_gain_node, worklet_node, analyser_node,
            test_signal_oscillator_node, test_signal_gain_node,
            tuning_fork_oscillator_node, tuning_fork_gain_node,
            mixer_gain_node))
    }
    
    /// Connect all audio nodes to create the audio graph
    /// 
    /// Sets up the complete audio routing:
    /// - Microphone Source -> Microphone Gain -> Mixer -> AudioWorklet/Analyser -> (optionally) Speakers
    /// - Test Signal -> Test Signal Gain -> Mixer
    /// - Tuning Fork -> Tuning Fork Gain -> Speakers
    fn connect_all_nodes(&mut self, audio_context: &AudioContext) -> Result<(), String> {
        self.input_node.connect_with_audio_node(&self.input_gain_node)
            .map_err(|_| "Failed to connect microphone source to gain".to_string())?;

        self.input_gain_node.connect_with_audio_node(&self.mixer_gain_node)
            .map_err(|_| "Failed to connect microphone gain to mixer".to_string())?;

        /////
        
        self.tuning_fork_oscillator_node.connect_with_audio_node(&self.tuning_fork_gain_node)
            .map_err(|_| "Failed to connect tuning fork oscillator to gain".to_string())?;

        self.tuning_fork_gain_node.connect_with_audio_node(&audio_context.destination())
            .map_err(|_| "Failed to connect tuning fork gain to destination".to_string())?;
        
        /////
        
        self.test_signal_oscillator_node.connect_with_audio_node(&self.test_signal_gain_node)
            .map_err(|_| "Failed to connect test signal oscillator to gain".to_string())?;

        self.test_signal_gain_node.connect_with_audio_node(&self.mixer_gain_node)
            .map_err(|_| "Failed to connect test signal gain to mixer".to_string())?;
        
        self.mixer_gain_node.connect_with_audio_node(&self.worklet_node)
            .map_err(|_| "Failed to connect mixer to worklet".to_string())?;
        
        self.mixer_gain_node.connect_with_audio_node(&self.analyser_node)
            .map_err(|_| "Failed to connect mixer to analyser".to_string())?;

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
    
    
    /// Set whether audio should be output to speakers
    pub fn set_output_to_speakers(&mut self, enabled: bool) {
        if self.output_to_speakers != enabled {
            self.output_to_speakers = enabled;
            
            // Update speaker connection based on new setting
            if enabled {
                let audio_context = self.worklet_node.context();
                if let Err(e) = self.worklet_node.connect_with_audio_node(&audio_context.destination()) {
                    dev_log!("Failed to connect worklet to speakers: {:?}", e);
                } else {
                    dev_log!("Connected worklet to speakers");
                }
            } else {
                // Disconnect only the speaker connection, not all connections
                let audio_context = self.worklet_node.context();
                if let Err(e) = self.worklet_node.disconnect_with_audio_node(&audio_context.destination()) {
                    dev_log!("Failed to disconnect worklet from speakers: {:?}", e);
                } else {
                    dev_log!("Disconnected worklet from speakers");
                }
            }
        }
    }
    
    /// Set microphone volume
    pub fn set_microphone_volume(&mut self, volume: f32) {
        let clamped_volume = volume.clamp(0.0, 1.0);
        self.input_gain_node.gain().set_value(clamped_volume);
        dev_log!("Set microphone volume to {:.2} (requested: {:.2})", clamped_volume, volume);
    }
    
    /// Update tuning fork audio configuration
    /// 
    /// This method manages the integrated tuning fork oscillator that connects directly to speakers,
    /// independent of the main AudioWorklet processing pipeline. Tuning fork audio is always
    /// audible regardless of the output_to_speakers flag.
    pub fn update_tuning_fork_config(&mut self, config: super::audio_pipeline_configs::TuningForkConfig) {
        dev_log!("Updating tuning fork audio config - frequency: {} Hz", config.frequency);
        
        // Update frequency if changed
        if (self.tuning_fork_config.frequency - config.frequency).abs() > f32::EPSILON {
            self.tuning_fork_oscillator_node.frequency().set_value(config.frequency);
            self.tuning_fork_config.frequency = config.frequency;
        }
        
        // Update volume if changed (with smooth ramping)
        if (self.tuning_fork_config.volume - config.volume).abs() > f32::EPSILON {
            self.tuning_fork_config.volume = config.volume;
            self.ramp_tuning_fork_gain(config.volume);
        }
    }
    
    /// Smoothly ramp the tuning fork gain to avoid audio pops
    fn ramp_tuning_fork_gain(&self, target: f32) {
        let audio_context = &self.worklet_node.context();
        if self.tuning_fork_gain_node.gain().set_target_at_time(target, audio_context.current_time(), 0.05).is_err() {
            self.tuning_fork_gain_node.gain().set_value(target);
        }
    }
    
    /// Update test signal generator configuration (unified routing - no reconnection needed)
    pub fn update_test_signal_config(&mut self, config: super::audio_pipeline_configs::SignalGeneratorConfig) {
        // Handle microphone muting for test signals to prevent feedback
        if config.enabled {
            // Mute microphone when test signal is active
            
            // Mute microphone to prevent feedback (no reconnection needed - just volume control)
            self.set_microphone_volume(0.0);
            
            // Enable speaker output for test signal
            if !self.output_to_speakers {
                self.set_output_to_speakers(true);
                dev_log!("Automatically enabled speaker output for test signal");
            }
        }
        
        // Then manage integrated test signal nodes
        if config.enabled {
            // Update frequency and amplitude
            self.test_signal_oscillator_node.frequency().set_value(config.frequency);
            self.test_signal_gain_node.gain().set_value(config.amplitude);
            dev_log!("Updated test signal configuration - freq: {} Hz, amp: {}", config.frequency, config.amplitude);
            self.test_signal_config = config;
        } else {
            // Disable test signal by setting gain to 0
            self.test_signal_gain_node.gain().set_value(0.0);
            self.test_signal_config.enabled = false;
            dev_log!("Disabled test signal");
            self.set_microphone_volume(1.0);
            self.set_output_to_speakers(false);
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
    pub fn execute_test_signal_configurations(
        &mut self,
        test_signal_configs: &[crate::presentation::ConfigureTestSignal]
    ) -> Result<(), String> {
        for config in test_signal_configs {
            dev_log!(
                "[DEBUG] Executing privileged test signal configuration - enabled: {}, freq: {} Hz, vol: {}%",
                config.enabled, config.frequency, config.volume
            );
            
            let audio_config = super::audio_pipeline_configs::SignalGeneratorConfig {
                enabled: config.enabled,
                frequency: config.frequency,
                amplitude: config.volume / 100.0,
            };
            
            self.update_test_signal_config(audio_config);
            dev_log!(
                "[DEBUG] ✓ Test signal control updated - enabled: {}, freq: {}, vol: {}%", 
                config.enabled, config.frequency, config.volume
            );
        }
        Ok(())
    }
}