//! # Web Audio API Buffer Compatibility
//!
//! This module provides compatibility utilities for converting between the buffer reference
//! system and Web Audio API buffer formats. It enables seamless integration with browser
//! audio processing while maintaining zero-copy performance characteristics.
//!
//! ## Key Features
//!
//! - Conversion between BufferRef and AudioBuffer formats
//! - Zero-copy operations where possible
//! - Type-safe channel and format handling
//! - Browser compatibility layer for different audio formats
//!
//! ## Usage Example
//!
//! ```rust
//! use crate::modules::application_core::web_audio_compat::*;
//! use crate::modules::application_core::buffer_ref::*;
//!
//! // Convert Web Audio API data to BufferRef
//! let audio_data: Vec<f32> = get_audio_buffer_data();
//! let metadata = BufferMetadata::new(44100, 2, 512);
//! let buffer_ref = BufferRef::new(audio_data, metadata);
//!
//! // Create Web Audio compatible format
//! let web_audio_info = WebAudioBufferInfo::from_buffer_ref(&buffer_ref);
//! ```

use super::buffer_ref::{BufferRef, BufferMetadata};

/// Web Audio API buffer format information.
///
/// Contains all the information needed to create or interpret Web Audio API
/// AudioBuffer objects, with compatibility for different browser implementations.
#[derive(Debug, Clone)]
pub struct WebAudioBufferInfo {
    /// Number of audio channels
    pub number_of_channels: u32,
    /// Sample rate in Hz
    pub sample_rate: f32,
    /// Length in sample frames
    pub length: u32,
    /// Audio data organized by channel
    pub channel_data: Vec<Vec<f32>>,
    /// Buffer duration in seconds
    pub duration: f64,
}

impl WebAudioBufferInfo {
    /// Creates Web Audio buffer info from a BufferRef.
    ///
    /// This performs the necessary format conversion and channel separation
    /// required by the Web Audio API.
    ///
    /// # Arguments
    /// * `buffer_ref` - Buffer reference to convert
    ///
    /// # Returns
    /// Web Audio compatible buffer information
    ///
    /// # Performance
    /// - Interleaved to planar conversion may require data copying
    /// - Single-channel buffers are zero-copy
    /// - Multi-channel buffers require deinterleaving
    pub fn from_buffer_ref(buffer_ref: &BufferRef<f32>) -> Self {
        let metadata = buffer_ref.metadata();
        let data = buffer_ref.data();
        
        let channel_data = if metadata.channels == 1 {
            // Single channel - can use data directly
            vec![data.to_vec()]
        } else {
            // Multiple channels - need to deinterleave
            deinterleave_audio_data(data, metadata.channels as usize)
        };
        
        Self {
            number_of_channels: metadata.channels as u32,
            sample_rate: metadata.sample_rate as f32,
            length: metadata.frame_count as u32,
            channel_data,
            duration: metadata.duration_seconds(),
        }
    }
    
    /// Creates a BufferRef from Web Audio buffer information.
    ///
    /// Converts planar channel data back to interleaved format suitable
    /// for BufferRef storage and processing.
    ///
    /// # Returns
    /// * `Ok(buffer_ref)` - Conversion successful
    /// * `Err(error)` - Conversion failed (mismatched channel data, etc.)
    pub fn to_buffer_ref(&self) -> Result<BufferRef<f32>, WebAudioError> {
        // Validate channel data consistency
        if self.channel_data.is_empty() {
            return Err(WebAudioError::InvalidChannelData("No channel data provided".to_string()));
        }
        
        let frame_count = self.channel_data[0].len();
        for (i, channel) in self.channel_data.iter().enumerate() {
            if channel.len() != frame_count {
                return Err(WebAudioError::InvalidChannelData(
                    format!("Channel {} length mismatch: expected {}, got {}", i, frame_count, channel.len())
                ));
            }
        }
        
        // Create interleaved data
        let interleaved_data = if self.channel_data.len() == 1 {
            // Single channel - can use data directly
            self.channel_data[0].clone()
        } else {
            // Multiple channels - need to interleave
            interleave_audio_data(&self.channel_data)
        };
        
        // Create metadata
        let metadata = BufferMetadata::new(
            self.sample_rate as u32,
            self.channel_data.len() as u8,
            frame_count,
        );
        
        Ok(BufferRef::new(interleaved_data, metadata))
    }
    
    /// Validates the Web Audio buffer information for consistency.
    pub fn validate(&self) -> Result<(), WebAudioError> {
        if self.number_of_channels == 0 {
            return Err(WebAudioError::InvalidFormat("Number of channels cannot be zero".to_string()));
        }
        
        if self.sample_rate <= 0.0 {
            return Err(WebAudioError::InvalidFormat("Sample rate must be positive".to_string()));
        }
        
        if self.length == 0 {
            return Err(WebAudioError::InvalidFormat("Buffer length cannot be zero".to_string()));
        }
        
        if self.channel_data.len() != self.number_of_channels as usize {
            return Err(WebAudioError::InvalidChannelData(
                format!("Channel data count {} doesn't match number_of_channels {}", 
                       self.channel_data.len(), self.number_of_channels)
            ));
        }
        
        // Check each channel has correct length
        for (i, channel) in self.channel_data.iter().enumerate() {
            if channel.len() != self.length as usize {
                return Err(WebAudioError::InvalidChannelData(
                    format!("Channel {} length {} doesn't match buffer length {}", 
                           i, channel.len(), self.length)
                ));
            }
        }
        
        Ok(())
    }
}

/// Audio processing utilities for Web Audio API compatibility.
pub struct WebAudioProcessor;

impl WebAudioProcessor {
    /// Processes audio data using a Web Audio API compatible callback.
    ///
    /// This enables integration with existing Web Audio processing code
    /// while maintaining the buffer reference system's performance benefits.
    ///
    /// # Arguments
    /// * `buffer_ref` - Input buffer reference
    /// * `processor` - Processing function with Web Audio API signature
    ///
    /// # Returns
    /// Processed buffer reference
    pub fn process_with_callback<F>(
        buffer_ref: &BufferRef<f32>,
        mut processor: F,
    ) -> Result<BufferRef<f32>, WebAudioError>
    where
        F: FnMut(&mut [f32], u32, f32) -> Result<(), String>,
    {
        let metadata = buffer_ref.metadata();
        let mut processed_data = buffer_ref.data().to_vec();
        
        // Process with Web Audio API style callback
        processor(
            &mut processed_data,
            metadata.channels as u32,
            metadata.sample_rate as f32,
        ).map_err(|e| WebAudioError::ProcessingFailed(e))?;
        
        Ok(BufferRef::new(processed_data, metadata.clone()))
    }
    
    /// Converts buffer reference to AudioWorklet processor compatible format.
    ///
    /// Creates channel arrays suitable for AudioWorklet processor input/output.
    pub fn to_worklet_format(buffer_ref: &BufferRef<f32>) -> Vec<Vec<f32>> {
        let metadata = buffer_ref.metadata();
        let data = buffer_ref.data();
        
        if metadata.channels == 1 {
            vec![data.to_vec()]
        } else {
            deinterleave_audio_data(data, metadata.channels as usize)
        }
    }
    
    /// Creates buffer reference from AudioWorklet processor output format.
    pub fn from_worklet_format(
        channel_arrays: Vec<Vec<f32>>,
        sample_rate: u32,
    ) -> Result<BufferRef<f32>, WebAudioError> {
        if channel_arrays.is_empty() {
            return Err(WebAudioError::InvalidChannelData("No channel data provided".to_string()));
        }
        
        let frame_count = channel_arrays[0].len();
        let channels = channel_arrays.len() as u8;
        
        let data = if channels == 1 {
            channel_arrays.into_iter().next().unwrap()
        } else {
            interleave_audio_data(&channel_arrays)
        };
        
        let metadata = BufferMetadata::new(sample_rate, channels, frame_count);
        Ok(BufferRef::new(data, metadata))
    }
}

/// Browser-specific audio format information.
///
/// Different browsers may have slightly different audio format requirements
/// or capabilities. This struct captures browser-specific information for
/// optimal compatibility.
#[derive(Debug, Clone)]
pub struct BrowserAudioInfo {
    /// Browser user agent string
    pub user_agent: String,
    /// Supported sample rates
    pub supported_sample_rates: Vec<u32>,
    /// Maximum channel count
    pub max_channels: u8,
    /// Preferred buffer size
    pub preferred_buffer_size: usize,
    /// AudioContext capabilities
    pub context_capabilities: AudioContextCapabilities,
}

/// AudioContext capability information
#[derive(Debug, Clone)]
pub struct AudioContextCapabilities {
    /// Base latency in seconds
    pub base_latency: f64,
    /// Output latency in seconds  
    pub output_latency: f64,
    /// Current sample rate
    pub sample_rate: f32,
    /// AudioWorklet support
    pub supports_audio_worklet: bool,
    /// Maximum number of channels for output
    pub max_channel_count: u32,
}

/// Utility functions for audio data format conversion
pub mod format_utils {
    /// Converts interleaved audio data to planar (channel-separated) format.
    ///
    /// # Arguments
    /// * `interleaved` - Interleaved audio data
    /// * `channel_count` - Number of audio channels
    ///
    /// # Returns
    /// Vector of channel data arrays
    pub fn deinterleave_audio_data(interleaved: &[f32], channel_count: usize) -> Vec<Vec<f32>> {
        if channel_count == 0 {
            return Vec::new();
        }
        
        let frame_count = interleaved.len() / channel_count;
        let mut channels = vec![Vec::with_capacity(frame_count); channel_count];
        
        for frame in 0..frame_count {
            for channel in 0..channel_count {
                let sample_index = frame * channel_count + channel;
                channels[channel].push(interleaved[sample_index]);
            }
        }
        
        channels
    }
    
    /// Converts planar audio data to interleaved format.
    ///
    /// # Arguments
    /// * `channels` - Channel data arrays
    ///
    /// # Returns
    /// Interleaved audio data
    pub fn interleave_audio_data(channels: &[Vec<f32>]) -> Vec<f32> {
        if channels.is_empty() {
            return Vec::new();
        }
        
        let frame_count = channels[0].len();
        let channel_count = channels.len();
        let mut interleaved = Vec::with_capacity(frame_count * channel_count);
        
        for frame in 0..frame_count {
            for channel in channels {
                interleaved.push(channel[frame]);
            }
        }
        
        interleaved
    }
}

// Re-export format utilities at module level for convenience
pub use format_utils::{deinterleave_audio_data, interleave_audio_data};

/// Errors that can occur during Web Audio API compatibility operations
#[derive(Debug, Clone)]
pub enum WebAudioError {
    /// Invalid audio format or parameters
    InvalidFormat(String),
    /// Invalid or inconsistent channel data
    InvalidChannelData(String),
    /// Audio processing failed
    ProcessingFailed(String),
    /// Browser compatibility issue
    BrowserCompatibility(String),
    /// Internal error with context
    Internal(String),
}

impl std::fmt::Display for WebAudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebAudioError::InvalidFormat(msg) => write!(f, "Invalid audio format: {}", msg),
            WebAudioError::InvalidChannelData(msg) => write!(f, "Invalid channel data: {}", msg),
            WebAudioError::ProcessingFailed(msg) => write!(f, "Audio processing failed: {}", msg),
            WebAudioError::BrowserCompatibility(msg) => write!(f, "Browser compatibility issue: {}", msg),
            WebAudioError::Internal(msg) => write!(f, "Internal Web Audio error: {}", msg),
        }
    }
}

impl std::error::Error for WebAudioError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::application_core::buffer_ref::{BufferRef, BufferMetadata};

    #[test]
    fn test_web_audio_buffer_info_from_mono_buffer() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let metadata = BufferMetadata::new(44100, 1, 4);
        let buffer_ref = BufferRef::new(data.clone(), metadata);
        
        let web_audio_info = WebAudioBufferInfo::from_buffer_ref(&buffer_ref);
        
        assert_eq!(web_audio_info.number_of_channels, 1);
        assert_eq!(web_audio_info.sample_rate, 44100.0);
        assert_eq!(web_audio_info.length, 4);
        assert_eq!(web_audio_info.channel_data.len(), 1);
        assert_eq!(web_audio_info.channel_data[0], data);
    }

    #[test]
    fn test_web_audio_buffer_info_from_stereo_buffer() {
        // Interleaved stereo data: L1, R1, L2, R2, L3, R3
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let metadata = BufferMetadata::new(44100, 2, 3);
        let buffer_ref = BufferRef::new(data, metadata);
        
        let web_audio_info = WebAudioBufferInfo::from_buffer_ref(&buffer_ref);
        
        assert_eq!(web_audio_info.number_of_channels, 2);
        assert_eq!(web_audio_info.sample_rate, 44100.0);
        assert_eq!(web_audio_info.length, 3);
        assert_eq!(web_audio_info.channel_data.len(), 2);
        
        // Check channel separation
        assert_eq!(web_audio_info.channel_data[0], vec![1.0, 3.0, 5.0]); // Left channel
        assert_eq!(web_audio_info.channel_data[1], vec![2.0, 4.0, 6.0]); // Right channel
    }

    #[test]
    fn test_web_audio_buffer_info_to_buffer_ref() {
        let channel_data = vec![
            vec![1.0, 3.0, 5.0], // Left channel
            vec![2.0, 4.0, 6.0], // Right channel
        ];
        
        let web_audio_info = WebAudioBufferInfo {
            number_of_channels: 2,
            sample_rate: 44100.0,
            length: 3,
            channel_data,
            duration: 3.0 / 44100.0,
        };
        
        let buffer_ref = web_audio_info.to_buffer_ref().unwrap();
        
        assert_eq!(buffer_ref.metadata().channels, 2);
        assert_eq!(buffer_ref.metadata().sample_rate, 44100);
        assert_eq!(buffer_ref.metadata().frame_count, 3);
        
        // Check interleaved data
        let expected_data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        assert_eq!(buffer_ref.data(), &expected_data);
    }

    #[test]
    fn test_deinterleave_audio_data() {
        let interleaved = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]; // L1,R1,L2,R2,L3,R3
        let channels = deinterleave_audio_data(&interleaved, 2);
        
        assert_eq!(channels.len(), 2);
        assert_eq!(channels[0], vec![1.0, 3.0, 5.0]); // Left
        assert_eq!(channels[1], vec![2.0, 4.0, 6.0]); // Right
    }

    #[test]
    fn test_interleave_audio_data() {
        let channels = vec![
            vec![1.0, 3.0, 5.0], // Left
            vec![2.0, 4.0, 6.0], // Right
        ];
        let interleaved = interleave_audio_data(&channels);
        
        let expected = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        assert_eq!(interleaved, expected);
    }

    #[test]
    fn test_web_audio_processor_to_worklet_format() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let metadata = BufferMetadata::new(44100, 2, 3);
        let buffer_ref = BufferRef::new(data, metadata);
        
        let worklet_format = WebAudioProcessor::to_worklet_format(&buffer_ref);
        
        assert_eq!(worklet_format.len(), 2);
        assert_eq!(worklet_format[0], vec![1.0, 3.0, 5.0]); // Left
        assert_eq!(worklet_format[1], vec![2.0, 4.0, 6.0]); // Right
    }

    #[test]
    fn test_web_audio_processor_from_worklet_format() {
        let channel_arrays = vec![
            vec![1.0, 3.0, 5.0], // Left
            vec![2.0, 4.0, 6.0], // Right
        ];
        
        let buffer_ref = WebAudioProcessor::from_worklet_format(channel_arrays, 44100).unwrap();
        
        assert_eq!(buffer_ref.metadata().channels, 2);
        assert_eq!(buffer_ref.metadata().sample_rate, 44100);
        assert_eq!(buffer_ref.metadata().frame_count, 3);
        
        let expected_data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        assert_eq!(buffer_ref.data(), &expected_data);
    }

    #[test]
    fn test_web_audio_info_validation() {
        let mut info = WebAudioBufferInfo {
            number_of_channels: 2,
            sample_rate: 44100.0,
            length: 3,
            channel_data: vec![
                vec![1.0, 2.0, 3.0],
                vec![4.0, 5.0, 6.0],
            ],
            duration: 3.0 / 44100.0,
        };
        
        // Valid case
        assert!(info.validate().is_ok());
        
        // Invalid channel count
        info.number_of_channels = 0;
        assert!(info.validate().is_err());
        
        // Invalid sample rate
        info.number_of_channels = 2;
        info.sample_rate = 0.0;
        assert!(info.validate().is_err());
        
        // Mismatched channel data
        info.sample_rate = 44100.0;
        info.channel_data = vec![vec![1.0, 2.0, 3.0]]; // Only 1 channel instead of 2
        assert!(info.validate().is_err());
    }
}