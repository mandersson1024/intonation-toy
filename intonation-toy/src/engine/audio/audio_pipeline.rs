use web_sys::{AudioContext, AudioWorkletNode, AudioWorkletNodeOptions, AudioNode, GainNode, AnalyserNode};
use super::tuning_fork_node::TuningForkAudioNode;
use super::test_signal_node::TestSignalAudioNode;
use super::signal_generator::{SignalGeneratorConfig, TuningForkConfig};
use super::AudioError;
use crate::common::dev_log;

pub struct AudioPipeline {
    pub worklet_node: AudioWorkletNode,
    pub tuning_fork_node: TuningForkAudioNode,
    pub test_signal_node: TestSignalAudioNode,
    pub mixer_gain_node: GainNode,
    pub microphone_gain_node: GainNode,
    pub microphone_source_node: AudioNode,
    pub analyser_node: AnalyserNode,
    pub output_to_speakers: bool,
}

impl AudioPipeline {
    pub fn new(audio_context: &AudioContext, media_stream: &web_sys::MediaStream) -> Result<Self, String> {
        // Create AudioWorkletNode
        let worklet_node = Self::create_worklet_node(audio_context)?;
        
        // Create microphone gain node
        let legacy_microphone_gain_node = audio_context
            .create_gain()
            .map_err(|_| "Failed to create microphone gain node".to_string())?;
        legacy_microphone_gain_node.gain().set_value(1.0);
        
        // Create mixer gain node
        let legacy_mixer_gain_node = audio_context
            .create_gain()
            .map_err(|_| "Failed to create mixer gain node".to_string())?;
        legacy_mixer_gain_node.gain().set_value(1.0);
        
        // Create tuning fork node with default config
        let default_tuning_fork_config = TuningForkConfig {
            frequency: 440.0, // A4
            volume: 0.0,      // Start muted
        };
        let tuning_fork_node = TuningForkAudioNode::new(audio_context, default_tuning_fork_config)
            .map_err(|e| format!("Failed to create tuning fork node: {:?}", e))?;
        
        // Create test signal node with default config (disabled by default)
        let test_signal_config = SignalGeneratorConfig {
            enabled: false,
            frequency: 440.0, // A4
            amplitude: 0.0,
            sample_rate: audio_context.sample_rate() as u32,
        };
        let test_signal_node = TestSignalAudioNode::new(audio_context, test_signal_config)
            .map_err(|e| format!("Failed to create test signal node: {:?}", e))?;
        
        // Create analyser node for volume analysis
        let analyser_node = audio_context
            .create_analyser()
            .map_err(|e| format!("Failed to create analyser node: {:?}", e))?;
        
        // Configure analyser node with FFT size of 128
        analyser_node.set_fft_size(128);
        // Set smoothing time constant to 0.0 for real-time analysis
        analyser_node.set_smoothing_time_constant(0.0);
        
        dev_log!("AudioPipeline analyser node created with FFT size: 128");
        
        // Create media stream source node
        let microphone_source = audio_context.create_media_stream_source(media_stream)
            .map_err(|e| format!("Failed to create media stream source: {:?}", e))?;
        dev_log!("✓ MediaStreamAudioSourceNode created");
        
        let mut pipeline = Self {
            worklet_node,
            tuning_fork_node,
            test_signal_node,
            mixer_gain_node: legacy_mixer_gain_node,
            microphone_gain_node: legacy_microphone_gain_node,
            microphone_source_node: microphone_source.clone().into(),
            analyser_node,
            output_to_speakers: false,
        };
        
        // Connect microphone automatically
        pipeline.connect_microphone(microphone_source.as_ref())
            .map_err(|e| format!("Failed to connect microphone: {:?}", e))?;
        dev_log!("✓ Microphone automatically connected to audio pipeline");
        
        Ok(pipeline)
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
    
    /// Connect microphone input and set up audio routing
    /// 
    /// Sets up the complete audio routing chain:
    /// Microphone Source -> Microphone Gain -> Mixer -> AudioWorklet -> (optionally) Speakers
    /// Test Signal -> Mixer (connected but disabled by default)
    /// 
    /// Returns the microphone gain node for external volume detector connection
    pub fn connect_microphone(&mut self, microphone_source: &AudioNode) -> Result<&GainNode, AudioError> {
        // Store microphone source
        self.microphone_source_node = microphone_source.clone();
        
        // Unified connection setup - always the same regardless of state:
        // Microphone Source -> Microphone Gain -> Mixer -> AudioWorklet
        
        // 1. Connect microphone source to microphone gain
        let mic_gain = &self.microphone_gain_node;
        microphone_source.connect_with_audio_node(mic_gain)
            .map_err(|e| AudioError::Generic(format!("Failed to connect microphone to gain node: {:?}", e)))?;
        dev_log!("Connected microphone to gain node");
        
        // 2. Connect microphone gain to mixer
        let mixer = &self.mixer_gain_node;
        mic_gain.connect_with_audio_node(mixer)
            .map_err(|e| AudioError::Generic(format!("Failed to connect microphone gain to mixer: {:?}", e)))?;
        dev_log!("Connected microphone gain to mixer");
        
        // 3. Connect test signal to mixer (it's disabled by default, so this is safe)
        self.test_signal_node.connect_to(mixer)
            .map_err(|e| AudioError::Generic(format!("Failed to connect test signal to mixer: {:?}", e)))?;
        dev_log!("Connected test signal to mixer");
        
        // 4. Connect mixer to analyser node for volume detection
        mixer.connect_with_audio_node(&self.analyser_node)
            .map_err(|e| AudioError::Generic(format!("Failed to connect mixer to analyser: {:?}", e)))?;
        dev_log!("Connected mixer to analyser node");
        
        // 5. Connect mixer to worklet
        mixer.connect_with_audio_node(&self.worklet_node)
            .map_err(|e| AudioError::Generic(format!("Failed to connect mixer to worklet: {:?}", e)))?;
        dev_log!("Connected mixer to worklet");
        
        // 6. Connect AudioWorklet to speakers if output is enabled
        if self.output_to_speakers {
            let audio_context = self.worklet_node.context();
            if let Err(e) = self.worklet_node.connect_with_audio_node(&audio_context.destination()) {
                dev_log!("Failed to connect worklet to speakers: {:?}", e);
            } else {
                dev_log!("Connected worklet to speakers");
            }
        }
        
        Ok(&self.microphone_gain_node)
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
        self.microphone_gain_node.gain().set_value(clamped_volume);
        dev_log!("Set microphone volume to {:.2} (requested: {:.2})", clamped_volume, volume);
    }
    
    /// Update tuning fork audio configuration
    /// 
    /// This method manages the dedicated TuningForkAudioNode that connects directly to speakers,
    /// independent of the main AudioWorklet processing pipeline. Tuning fork audio is always
    /// audible regardless of the output_to_speakers flag.
    pub fn update_tuning_fork_config(&mut self, config: super::signal_generator::TuningForkConfig) {
        dev_log!("Updating tuning fork audio config - frequency: {} Hz", config.frequency);
        
        // Update the tuning fork audio node
        self.tuning_fork_node.update_config(config);
    }
    
    /// Update test signal generator configuration (unified routing - no reconnection needed)
    pub fn update_test_signal_config(&mut self, config: super::signal_generator::SignalGeneratorConfig) {
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
        
        // Then manage local TestSignalAudioNode
        if config.enabled {
            // Update existing node
            self.test_signal_node.update_config(config);
            dev_log!("Updated test signal node configuration");
        } else {
            // Disable test signal but keep node for potential re-enabling
            self.test_signal_node.disable();
            dev_log!("Disabled test signal node");
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
            
            let audio_config = super::signal_generator::SignalGeneratorConfig {
                enabled: config.enabled,
                frequency: config.frequency,
                amplitude: config.volume / 100.0,
                sample_rate: crate::app_config::STANDARD_SAMPLE_RATE,
            };
            
            self.update_test_signal_config(audio_config);
            dev_log!(
                "[DEBUG] ✓ Test signal control updated - enabled: {}, freq: {}, vol: {}%", 
                config.enabled, config.frequency, config.volume
            );
        }
        Ok(())
    }
    
    /// Disconnect and cleanup all audio nodes
    pub fn disconnect(&mut self) {
        // Disconnect worklet node
        let _ = self.worklet_node.disconnect();
        dev_log!("AudioWorklet disconnected");
        
        // Clean up the test signal node
        self.test_signal_node.cleanup();
        dev_log!("Test signal node cleaned up");
        
        // Clean up the mixer node
        let _ = self.mixer_gain_node.disconnect();
        dev_log!("Mixer node disconnected and cleaned up");
        
        // Clean up the microphone gain node
        let _ = self.microphone_gain_node.disconnect();
        dev_log!("Microphone gain node disconnected and cleaned up");
        
        // Clean up the analyser node
        let _ = self.analyser_node.disconnect();
        dev_log!("Analyser node disconnected and cleaned up");
        
        // Disconnect microphone source node
        let _ = self.microphone_source_node.disconnect();
        
        // Note: tuning fork audio node cleanup is handled by its Drop trait
    }
}