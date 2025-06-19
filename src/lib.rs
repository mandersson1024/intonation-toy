//! Real-time Pitch Visualizer
//! 
//! A musical education tool that provides live pitch detection with visual feedback
//! and interval analysis. Designed for musicians and music students.

pub mod audio;
pub mod gui;
pub mod bridge;

pub use audio::{AudioEngine, PitchResult};
pub use gui::PitchVisualizerApp;
pub use bridge::{AudioMessage, MessageBus}; 