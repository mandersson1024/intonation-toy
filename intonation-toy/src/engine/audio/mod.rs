pub mod microphone;
pub mod context;
pub mod worklet;
pub mod permission;
pub mod pitch_detector;
pub mod pitch_analyzer;
pub mod volume_detector;
pub mod signal_generator;
pub mod message_protocol;
pub mod data_types;
pub mod tuning_fork_node;
pub mod test_signal_node;
pub mod signal_flow;

use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    static AUDIO_CONTEXT_MANAGER: RefCell<Option<Rc<RefCell<context::AudioContextManager>>>> = const { RefCell::new(None) };
}

pub fn set_global_audio_context_manager(manager: Rc<RefCell<context::AudioContextManager>>) {
    AUDIO_CONTEXT_MANAGER.with(|global_manager| {
        *global_manager.borrow_mut() = Some(manager);
    });
}

pub use context::{AudioSystemContext, AudioDevices};
pub use worklet::AudioWorkletState;
pub use signal_generator::{SignalGeneratorConfig, TuningForkConfig};
pub use data_types::{VolumeLevelData, PitchData, AudioWorkletStatus, VolumeAnalysis};
pub use permission::AudioPermission;
pub use tuning_fork_node::TuningForkAudioNode;
pub use test_signal_node::TestSignalAudioNode;
pub use signal_flow::AudioSignalFlow;

use microphone::AudioError;
pub use volume_detector::VolumeDetector;
