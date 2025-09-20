#![cfg(target_arch = "wasm32")]

use web_sys::{AudioContext, GainNode, AudioWorkletNode, MediaStreamAudioSourceNode, OscillatorNode, AnalyserNode};

/// Represents the complete audio signal flow with all Web Audio API nodes
/// 
/// This struct manages the creation and storage of all audio nodes in the signal processing chain.
/// The nodes are created but not initialized - initialization happens externally after creation.
pub struct AudioSignalPath {
    pub user_input: MediaStreamAudioSourceNode,
    pub user_input_mute: GainNode,
    pub worklet: AudioWorkletNode,
    pub analyser: AnalyserNode,
    pub test_signal_osc: OscillatorNode,
    pub test_signal_gain: GainNode,
    pub test_signal_mute: GainNode,
    pub tonal_center_osc: OscillatorNode,
    pub tonal_center_gain: GainNode,
}

impl AudioSignalPath {
    pub fn new(
        context: AudioContext,
        user_input: MediaStreamAudioSourceNode,
        worklet: AudioWorkletNode,
    ) -> Self {

        // Create
        let user_input_mute = context.create_gain().unwrap();
        let test_signal_osc = context.create_oscillator().unwrap();
        let test_signal_gain = context.create_gain().unwrap();
        let test_signal_mute = context.create_gain().unwrap();
        let analyser = context.create_analyser().unwrap();
        let tonal_center_osc = context.create_oscillator().unwrap();
        let tonal_center_gain = context.create_gain().unwrap();

        // Connect
        user_input.connect_with_audio_node(&user_input_mute).unwrap();
        user_input_mute.connect_with_audio_node(&analyser).unwrap();
        test_signal_osc.connect_with_audio_node(&test_signal_gain).unwrap();
        test_signal_gain.connect_with_audio_node(&test_signal_mute).unwrap();
        test_signal_mute.connect_with_audio_node(&context.destination()).unwrap();
        test_signal_mute.connect_with_audio_node(&analyser).unwrap();
        analyser.connect_with_audio_node(&worklet).unwrap();
        tonal_center_osc.connect_with_audio_node(&tonal_center_gain).unwrap();
        tonal_center_gain.connect_with_audio_node(&context.destination()).unwrap();

        // user_input -> user_intput_mute -> analyser -> worklet
        // test_signal_osc -> test_signal_gain -> test_signal_mute -> [analyser -> worklet] // [destination]
        // tonal_center_osc -> tonal_center_gain -> destination

        Self {
            user_input,
            user_input_mute,
            test_signal_osc,
            test_signal_gain,
            test_signal_mute,
            worklet,
            analyser,
            tonal_center_osc,
            tonal_center_gain,
        }
    }
}
