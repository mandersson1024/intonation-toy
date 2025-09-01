use web_sys::{AudioContext, AudioWorkletNode, AudioWorkletNodeOptions, AudioNode, GainNode};
use super::tuning_fork_node::TuningForkAudioNode;
use super::test_signal_node::TestSignalAudioNode;
use super::signal_generator::{SignalGeneratorConfig, TuningForkConfig};
use crate::common::dev_log;

pub struct AudioPipeline {
    pub worklet_node: AudioWorkletNode,
    pub tuning_fork_node: TuningForkAudioNode,
    pub test_signal_node: TestSignalAudioNode,
    pub legacy_mixer_gain_node: GainNode,
    pub legacy_microphone_gain_node: GainNode,
    pub legacy_microphone_source_node: Option<AudioNode>,
}

impl AudioPipeline {
    pub fn new(audio_context: &AudioContext) -> Result<Self, String> {
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
        
        Ok(Self {
            worklet_node,
            tuning_fork_node,
            test_signal_node,
            legacy_mixer_gain_node,
            legacy_microphone_gain_node,
            legacy_microphone_source_node: None,
        })
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