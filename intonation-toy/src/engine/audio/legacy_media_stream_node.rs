use std::fmt;
use crate::engine::AudioEngine;

#[derive(Debug, Clone)]
pub enum AudioError {
    NotSupported(String),
    Generic(String),
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioError::NotSupported(msg) => write!(f, "Not supported: {}", msg),
            AudioError::Generic(msg) => write!(f, "Audio error: {}", msg),
        }
    }
}
/// Creates a MediaStreamAudioSourceNode from a MediaStream
pub fn legacy_create_media_stream_node(
    media_stream: &web_sys::MediaStream,
    audio_context: &web_sys::AudioContext,
) -> Result<web_sys::MediaStreamAudioSourceNode, String> {
    audio_context.create_media_stream_source(media_stream)
        .map_err(|e| format!("Failed to create audio source: {:?}", e))
}

/// Connects a MediaStreamAudioSourceNode to the audio worklet
pub fn legacy_connect_media_stream_node_to_audioworklet(
    source: &web_sys::MediaStreamAudioSourceNode,
    audio_engine: &mut AudioEngine,
) -> Result<(), String> {
    let result = audio_engine.audioworklet_manager
        .connect_microphone(source.as_ref(), false)
        .map_err(|e| e.to_string());
    
    match result {
        Ok(_) => {
            if !audio_engine.audioworklet_manager.is_processing() {
                let _ = audio_engine.audioworklet_manager.start_processing();
            }
            
            Ok(())
        }
        Err(e) => {
            Err(format!("Failed to connect microphone: {:?}", e))
        }
    }
}

