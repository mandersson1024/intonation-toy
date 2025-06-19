//! Communication bridge between audio processing and GUI threads
//! 
//! This module provides lock-free, real-time safe communication using ring buffers
//! and message passing to ensure audio processing is never blocked.

use crossbeam::channel;
use ringbuf::{HeapRb, Rb};
use std::sync::Arc;

pub mod message_bus;

pub use message_bus::MessageBus;

/// Audio analysis result sent from audio thread to GUI
#[derive(Debug, Clone, Copy)]
pub struct AudioMessage {
    /// Detected fundamental frequency in Hz
    pub frequency: f32,
    /// Confidence level of detection (0.0 to 1.0)
    pub confidence: f32,
    /// Deviation from reference pitch in cents
    pub cents_deviation: f32,
    /// Detected musical note name
    pub note_name: [char; 4], // e.g., ['C', '#', '4', '\0']
    /// Musical interval from reference
    pub interval_cents: i16,
    /// Interval name (e.g., "Major 3rd")
    pub interval_name: [char; 16],
    /// Timestamp for synchronization
    pub timestamp_us: u64,
}

impl Default for AudioMessage {
    fn default() -> Self {
        Self {
            frequency: 0.0,
            confidence: 0.0,
            cents_deviation: 0.0,
            note_name: ['\0'; 4],
            interval_cents: 0,
            interval_name: ['\0'; 16],
            timestamp_us: 0,
        }
    }
}

/// Configuration messages sent from GUI to audio thread
#[derive(Debug, Clone)]
pub enum ControlMessage {
    /// Set reference pitch frequency
    SetReference(f32),
    /// Set reference note by name
    SetReferenceNote(String),
    /// Enable/disable audio processing
    SetEnabled(bool),
    /// Set tuning system (12-TET vs Just Intonation)
    SetTuningSystem(TuningSystem),
}

#[derive(Debug, Clone, Copy)]
pub enum TuningSystem {
    EqualTemperament,
    JustIntonation,
}

/// Type aliases for the communication channels
pub type AudioReceiver = channel::Receiver<AudioMessage>;
pub type AudioSender = channel::Sender<AudioMessage>;
pub type ControlReceiver = channel::Receiver<ControlMessage>;
pub type ControlSender = channel::Sender<ControlMessage>;

/// Creates the communication channels between audio and GUI threads
pub fn create_channels() -> (
    (AudioSender, AudioReceiver),
    (ControlSender, ControlReceiver),
) {
    let audio_channels = channel::bounded(1); // Latest audio data only
    let control_channels = channel::unbounded(); // Don't drop control messages
    
    (audio_channels, control_channels)
} 