use super::{AudioMessage, ControlMessage, AudioReceiver, AudioSender, ControlReceiver, ControlSender};
use std::time::{Duration, Instant};
use anyhow::Result;

/// Central message bus for coordinating audio-GUI communication
pub struct MessageBus {
    /// Sends audio analysis results to GUI
    audio_sender: AudioSender,
    /// Receives audio analysis results in GUI
    audio_receiver: AudioReceiver,
    /// Sends control commands from GUI to audio
    control_sender: ControlSender,
    /// Receives control commands in audio thread
    control_receiver: ControlReceiver,
}

impl MessageBus {
    /// Create a new message bus with communication channels
    pub fn new() -> Self {
        let ((audio_sender, audio_receiver), (control_sender, control_receiver)) = 
            super::create_channels();
        
        Self {
            audio_sender,
            audio_receiver,
            control_sender,
            control_receiver,
        }
    }

    /// Split the message bus for use in different threads
    /// Returns (audio_handle, gui_handle)
    pub fn split(self) -> (AudioHandle, GuiHandle) {
        let audio_handle = AudioHandle {
            audio_sender: self.audio_sender,
            control_receiver: self.control_receiver,
        };
        
        let gui_handle = GuiHandle {
            audio_receiver: self.audio_receiver,
            control_sender: self.control_sender,
        };
        
        (audio_handle, gui_handle)
    }
}

/// Handle for audio thread to send results and receive control messages
pub struct AudioHandle {
    audio_sender: AudioSender,
    control_receiver: ControlReceiver,
}

impl AudioHandle {
    /// Send audio analysis result to GUI (non-blocking)
    pub fn send_audio_result(&self, message: AudioMessage) -> Result<()> {
        // Use try_send to avoid blocking audio thread
        self.audio_sender.try_send(message)
            .map_err(|e| anyhow::anyhow!("Failed to send audio message: {}", e))
    }

    /// Check for control messages from GUI (non-blocking)
    pub fn receive_control_message(&self) -> Option<ControlMessage> {
        self.control_receiver.try_recv().ok()
    }

    /// Process all pending control messages
    pub fn process_control_messages<F>(&self, mut handler: F) 
    where
        F: FnMut(ControlMessage),
    {
        while let Some(message) = self.receive_control_message() {
            handler(message);
        }
    }
}

/// Handle for GUI thread to receive audio results and send control messages
pub struct GuiHandle {
    audio_receiver: AudioReceiver,
    control_sender: ControlSender,
}

impl GuiHandle {
    /// Get latest audio analysis result (non-blocking)
    pub fn get_latest_audio_result(&self) -> Option<AudioMessage> {
        // Try to get the most recent message, discarding older ones
        let mut latest = None;
        while let Ok(message) = self.audio_receiver.try_recv() {
            latest = Some(message);
        }
        latest
    }

    /// Send control message to audio thread
    pub fn send_control_message(&self, message: ControlMessage) -> Result<()> {
        self.control_sender.send(message)
            .map_err(|e| anyhow::anyhow!("Failed to send control message: {}", e))
    }

    /// Wait for audio result with timeout
    pub fn wait_for_audio_result(&self, timeout: Duration) -> Result<AudioMessage> {
        self.audio_receiver.recv_timeout(timeout)
            .map_err(|e| anyhow::anyhow!("Timeout waiting for audio result: {}", e))
    }
}

// Convenience methods for common operations
impl GuiHandle {
    /// Set reference frequency
    pub fn set_reference_frequency(&self, freq: f32) -> Result<()> {
        self.send_control_message(ControlMessage::SetReference(freq))
    }

    /// Set reference by note name
    pub fn set_reference_note(&self, note: &str) -> Result<()> {
        self.send_control_message(ControlMessage::SetReferenceNote(note.to_string()))
    }

    /// Enable or disable audio processing
    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        self.send_control_message(ControlMessage::SetEnabled(enabled))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_message_bus_communication() {
        let bus = MessageBus::new();
        let (audio_handle, gui_handle) = bus.split();

        // Test audio -> GUI communication
        let test_message = AudioMessage {
            frequency: 440.0,
            confidence: 0.9,
            ..Default::default()
        };

        audio_handle.send_audio_result(test_message).unwrap();
        let received = gui_handle.get_latest_audio_result().unwrap();
        
        assert_eq!(received.frequency, 440.0);
        assert_eq!(received.confidence, 0.9);
    }

    #[test]
    fn test_control_message_sending() {
        let bus = MessageBus::new();
        let (audio_handle, gui_handle) = bus.split();

        // Test GUI -> audio communication
        gui_handle.set_reference_frequency(442.0).unwrap();
        
        let received = audio_handle.receive_control_message().unwrap();
        match received {
            ControlMessage::SetReference(freq) => assert_eq!(freq, 442.0),
            _ => panic!("Wrong message type received"),
        }
    }
} 