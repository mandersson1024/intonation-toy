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
    // Core audio processing chain
    pub microphone_source: MediaStreamAudioSourceNode,
    pub microphone_gain: GainNode,
    pub mixer_gain: GainNode,
    pub audioworklet_node: AudioWorkletNode,
    
    // Analysis nodes (parallel taps)
    pub analyser_node: AnalyserNode,
    
    // Additional audio sources
    pub test_signal_oscillator: OscillatorNode,
    pub test_signal_gain: GainNode,
    pub tuning_fork_oscillator: OscillatorNode,
    pub tuning_fork_gain: GainNode,
    
    pub output_gain: GainNode,

    // Audio context reference
    audio_context: AudioContext,
}

impl AudioSignalFlow {
    /// Creates a new AudioSignalFlow with all nodes initialized and connected
    pub fn new(
        context: AudioContext,
        input: MediaStreamAudioSourceNode,
        worklet: AudioWorkletNode,
    ) -> Self {
        // Create nodes
        let input_gain = context.create_gain().unwrap();
        let analyser = context.create_analyser().unwrap();
        let mixer_gain = context.create_gain().unwrap();
        let test_signal = context.create_oscillator().unwrap();
        let test_signal_gain = context.create_gain().unwrap();
        let tuning_fork = context.create_oscillator().unwrap();
        let tuning_fork_gain = context.create_gain().unwrap();
        let output_gain = context.create_gain().unwrap();

        // Connect all nodes
        input.connect_with_audio_node(&input_gain).unwrap();
        input_gain.connect_with_audio_node(&analyser).unwrap();
        input_gain.connect_with_audio_node(&mixer_gain).unwrap();
        test_signal.connect_with_audio_node(&test_signal_gain).unwrap();
        test_signal_gain.connect_with_audio_node(&mixer_gain).unwrap();
        mixer_gain.connect_with_audio_node(&worklet).unwrap();
        worklet.connect_with_audio_node(&output_gain).unwrap();
        output_gain.connect_with_audio_node(&context.destination()).unwrap();
        tuning_fork.connect_with_audio_node(&tuning_fork_gain).unwrap();
        tuning_fork_gain.connect_with_audio_node(&context.destination()).unwrap();
        
        Self {
            microphone_source: input,
            microphone_gain: input_gain,
            mixer_gain,
            audioworklet_node: worklet,
            analyser_node: analyser,
            test_signal_oscillator: test_signal,
            test_signal_gain,
            tuning_fork_oscillator: tuning_fork,
            tuning_fork_gain,
            output_gain,
            audio_context: context,
        }
    }
