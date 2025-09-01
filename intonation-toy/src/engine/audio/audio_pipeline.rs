use web_sys::{AudioWorkletNode, AudioNode, GainNode};
use super::tuning_fork_node::TuningForkAudioNode;
use super::test_signal_node::TestSignalAudioNode;

pub struct AudioPipeline {
    pub worklet_node: AudioWorkletNode,
    pub tuning_fork_node: TuningForkAudioNode,
    pub test_signal_node: TestSignalAudioNode,
    pub legacy_mixer_gain_node: GainNode,
    pub legacy_microphone_gain_node: GainNode,
    pub legacy_microphone_source_node: Option<AudioNode>,
}