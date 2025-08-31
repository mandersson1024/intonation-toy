use web_sys::GainNode;
use crate::common::dev_log;
use super::AudioError;

/// AudioRouter manages all audio routing and mixing operations
/// 
/// This component is responsible for:
/// - Microphone input gain control
/// - Speaker output routing and volume control
/// - Audio mixing and routing between different sources
/// - Using gain nodes from the AudioSignalFlow
/// 
/// The router uses the gain nodes from the signal flow and provides
/// clean interfaces for controlling audio routing and mixing.
pub struct AudioRouter {
    // Input routing nodes from signal flow
    input_gain: GainNode,
    input_routing_state: InputRoutingState,
    
    // Output routing nodes from signal flow  
    output_routing_state: OutputRoutingState,
}

/// State tracking for input routing and processing
#[derive(Debug, Clone)]
struct InputRoutingState {
    microphone_volume: f32,
    microphone_enabled: bool,
}

impl Default for InputRoutingState {
    fn default() -> Self {
        Self {
            microphone_volume: 1.0,
            microphone_enabled: true,
        }
    }
}

/// State tracking for output routing
#[derive(Debug, Clone)]
struct OutputRoutingState {
    speaker_output_enabled: bool,
    master_volume: f32,
}

impl Default for OutputRoutingState {
    fn default() -> Self {
        Self {
            speaker_output_enabled: true,
            master_volume: 1.0,
        }
    }
}

impl AudioRouter {
    /// Creates a new AudioRouter
    /// 
    /// # Parameters
    /// - `input_gain`: Input gain node from AudioSignalFlow for microphone control
    /// 
    /// # Returns
    /// Result containing the configured AudioRouter or error description
    pub fn new(input_gain: GainNode) -> Result<Self, String> {
        dev_log!("Creating AudioRouter with signal flow gain nodes");
        
        Ok(Self {
            input_gain,
            input_routing_state: InputRoutingState::default(),
            output_routing_state: OutputRoutingState::default(),
        })
    }

    /// Set microphone input volume
    /// 
    /// Controls the gain level of the microphone input using the input gain node
    /// from the AudioSignalFlow. Volume is clamped to the range [0.0, 1.0].
    /// 
    /// # Parameters
    /// - `volume`: Volume level from 0.0 (muted) to 1.0 (full volume)
    /// 
    /// # Returns
    /// Result indicating success or failure of the volume adjustment
    pub fn set_microphone_volume(&mut self, volume: f32) -> Result<(), AudioError> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        
        self.input_gain.gain().set_value(clamped_volume);
        self.input_routing_state.microphone_volume = clamped_volume;
        
        dev_log!("✓ Microphone volume set to {:.2} via AudioRouter", clamped_volume);
        Ok(())
    }

    /// Enable or disable microphone input
    /// 
    /// Controls whether microphone input is processed by setting the input gain.
    /// When disabled, the microphone input is effectively muted.
    /// 
    /// # Parameters
    /// - `enabled`: True to enable microphone input, false to disable
    pub fn set_microphone_enabled(&mut self, enabled: bool) -> Result<(), AudioError> {
        if enabled {
            // Restore the previous volume level
            self.input_gain.gain().set_value(self.input_routing_state.microphone_volume);
            dev_log!("✓ Microphone input enabled (volume: {:.2})", self.input_routing_state.microphone_volume);
        } else {
            // Mute the input but preserve the volume setting
            self.input_gain.gain().set_value(0.0);
            dev_log!("✓ Microphone input disabled");
        }
        
        self.input_routing_state.microphone_enabled = enabled;
        Ok(())
    }

    /// Set whether to output audio stream to speakers
    /// 
    /// Controls whether the processed audio is routed to the speakers/output.
    /// Note: The current signal flow already connects to the destination,
    /// so this method manages the routing state for future implementation.
    /// 
    /// # Parameters
    /// - `enabled`: True to enable speaker output, false to disable
    pub fn set_output_to_speakers(&mut self, enabled: bool) {
        self.output_routing_state.speaker_output_enabled = enabled;
        
        if enabled {
            // The signal flow already connects worklet to destination
            // This could control a master output gain in the future
            dev_log!("✓ Speaker output enabled via AudioRouter");
        } else {
            // Could disconnect from destination or use a master gain in the future
            dev_log!("✓ Speaker output disabled (routing state updated)");
        }
    }

    /// Set master output volume
    /// 
    /// Controls the overall output volume for all audio going to speakers.
    /// Note: This is currently a state-only operation as the signal flow
    /// doesn't have a master output gain node yet.
    /// 
    /// # Parameters
    /// - `volume`: Master volume level from 0.0 (muted) to 1.0 (full volume)
    pub fn set_master_volume(&mut self, volume: f32) {
        let clamped_volume = volume.clamp(0.0, 1.0);
        self.output_routing_state.master_volume = clamped_volume;
        
        dev_log!("✓ Master volume set to {:.2} (state updated)", clamped_volume);
        // TODO: Apply to master output gain node when available in signal flow
    }

    /// Get current microphone volume level
    /// 
    /// # Returns
    /// Current microphone volume level (0.0 to 1.0)
    pub fn get_microphone_volume(&self) -> f32 {
        self.input_routing_state.microphone_volume
    }

    /// Check if microphone input is enabled
    /// 
    /// # Returns
    /// True if microphone input is enabled and processing audio
    pub fn is_microphone_enabled(&self) -> bool {
        self.input_routing_state.microphone_enabled
    }

    /// Check if speaker output is enabled
    /// 
    /// # Returns
    /// True if audio is being routed to speakers
    pub fn is_speaker_output_enabled(&self) -> bool {
        self.output_routing_state.speaker_output_enabled
    }

    /// Get current master volume level
    /// 
    /// # Returns
    /// Current master volume level (0.0 to 1.0)
    pub fn get_master_volume(&self) -> f32 {
        self.output_routing_state.master_volume
    }

    /// Get current input routing state
    /// 
    /// Returns information about the current microphone input configuration.
    /// 
    /// # Returns
    /// Current input routing state information
    pub fn get_input_state(&self) -> &InputRoutingState {
        &self.input_routing_state
    }

    /// Get current output routing state
    /// 
    /// Returns information about the current speaker output configuration.
    /// 
    /// # Returns
    /// Current output routing state information
    pub fn get_output_state(&self) -> &OutputRoutingState {
        &self.output_routing_state
    }

    /// Mute all audio routing
    /// 
    /// This is a convenience method to quickly mute both input and output
    /// without changing the underlying volume settings.
    pub fn mute_all(&mut self) -> Result<(), AudioError> {
        // Mute microphone input
        self.input_gain.gain().set_value(0.0);
        self.input_routing_state.microphone_enabled = false;
        
        // Disable speaker output (state-only for now)
        self.output_routing_state.speaker_output_enabled = false;
        
        dev_log!("✓ All audio routing muted");
        Ok(())
    }

    /// Unmute all audio routing
    /// 
    /// This is a convenience method to restore audio routing to previous settings.
    pub fn unmute_all(&mut self) -> Result<(), AudioError> {
        // Restore microphone input to previous volume
        self.input_gain.gain().set_value(self.input_routing_state.microphone_volume);
        self.input_routing_state.microphone_enabled = true;
        
        // Enable speaker output
        self.output_routing_state.speaker_output_enabled = true;
        
        dev_log!("✓ All audio routing unmuted (volume: {:.2})", 
                self.input_routing_state.microphone_volume);
        Ok(())
    }

    /// Disconnect and cleanup the audio router
    /// 
    /// This method cleans up the router state and prepares it for disposal.
    /// 
    /// # Returns
    /// Result indicating success or failure of the cleanup
    pub fn disconnect(&mut self) -> Result<(), AudioError> {
        // Mute all audio
        self.input_gain.gain().set_value(0.0);
        
        // Reset state
        self.input_routing_state.microphone_enabled = false;
        self.input_routing_state.microphone_volume = 0.0;
        self.output_routing_state.speaker_output_enabled = false;
        self.output_routing_state.master_volume = 0.0;
        
        dev_log!("✓ AudioRouter disconnected and cleaned up");
        Ok(())
    }

    /// Get reference to input gain node
    /// 
    /// Provides access to the underlying Web Audio API GainNode for advanced use cases.
    /// 
    /// # Returns
    /// Reference to the input GainNode from signal flow
    pub fn get_input_gain(&self) -> &GainNode {
        &self.input_gain
    }
}

impl Drop for AudioRouter {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}