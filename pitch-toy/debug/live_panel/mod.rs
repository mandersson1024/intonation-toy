// Live Panel Module - Real-time data visualization and monitoring
//
// Provides real-time monitoring and visualization with:
// - Audio device enumeration display
// - Real-time permission status display
// - Performance metrics (framerate, latency)
// - Audio volume and pitch detection display
// - System health monitoring

mod component;
mod metrics_display;
mod test_signal_controls;

pub use component::LivePanel;
pub use component::LivePanelProps;
pub use component::LivePanelMsg;
pub use component::AudioWorkletStatus;
pub use test_signal_controls::{TestSignalControls, TestSignalControlsProps, TestSignalConfig, TestWaveform};