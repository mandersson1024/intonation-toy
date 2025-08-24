use web_sys::{AudioContext, GainNode, AudioWorkletNode, MediaStreamAudioSourceNode, OscillatorNode, AnalyserNode};

/// Represents the complete audio signal flow with all Web Audio API nodes
/// 
/// This struct manages the creation and storage of all audio nodes in the signal processing chain.
/// The nodes are created but not initialized - initialization happens externally after creation.
/// 
/// Signal Flow:
/// ```
/// Microphone Input → Microphone Gain → Mixer Gain → AudioWorklet → [Optional] Speakers
///                           ↓ (parallel)
///                       AnalyserNode
///                           
/// Test Signal → Mixer Gain ↗
/// 
/// Tuning Fork → Speakers (direct, independent path)
/// ```
pub struct AudioSignalFlow {
    pub input: MediaStreamAudioSourceNode,
    pub input_gain: GainNode,
    pub mixer_gain: GainNode,
    pub worklet: AudioWorkletNode,
    pub analyser: AnalyserNode,
    pub test_signal_osc: OscillatorNode,
    pub test_signal_gain: GainNode,
    pub tuning_fork_osc: OscillatorNode,
    pub tuning_fork_gain: GainNode,
    pub output_gain: GainNode,
}

impl AudioSignalFlow {
    /// Creates a new AudioSignalFlow with all nodes initialized and connected
    pub fn new(
        context: AudioContext,
        input: MediaStreamAudioSourceNode,
        worklet: AudioWorkletNode,
    ) -> Self {
        // Create
        let input_gain = context.create_gain().unwrap();
        let mixer_gain = context.create_gain().unwrap();
        let test_signal_osc = context.create_oscillator().unwrap();
        let test_signal_gain = context.create_gain().unwrap();
        let analyser = context.create_analyser().unwrap();
        let tuning_fork_osc = context.create_oscillator().unwrap();
        let tuning_fork_gain = context.create_gain().unwrap();
        let output_gain = context.create_gain().unwrap();

        // Connect
        input.connect_with_audio_node(&input_gain).unwrap();
        input_gain.connect_with_audio_node(&analyser).unwrap();
        input_gain.connect_with_audio_node(&mixer_gain).unwrap();
        test_signal_osc.connect_with_audio_node(&test_signal_gain).unwrap();
        test_signal_gain.connect_with_audio_node(&mixer_gain).unwrap();
        mixer_gain.connect_with_audio_node(&worklet).unwrap();
        worklet.connect_with_audio_node(&output_gain).unwrap();
        output_gain.connect_with_audio_node(&context.destination()).unwrap();
        tuning_fork_osc.connect_with_audio_node(&tuning_fork_gain).unwrap();
        tuning_fork_gain.connect_with_audio_node(&context.destination()).unwrap();
        
        Self {
            input,
            input_gain,
            test_signal_osc,
            test_signal_gain,
            worklet,
            mixer_gain,
            analyser,
            tuning_fork_osc,
            tuning_fork_gain,
            output_gain,
        }
    }
}
