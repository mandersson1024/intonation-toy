
use web_sys::{ AudioContext, MessageEvent };

use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use crate::common::dev_log;
use crate::app_config::AUDIO_CHUNK_SIZE;
use super::{AudioError, volume_detector::VolumeDetector};
use super::message_protocol::{AudioWorkletMessageFactory, ToWorkletMessage, MessageSerializer};
use super::worklet_message_handling::{MessageHandlerState, handle_worklet_message};



#[derive(Debug, Clone, PartialEq)]
pub enum AudioWorkletState {
    Uninitialized,
    Initializing,
    Ready,
    Processing,
    Stopped,
    Failed,
}

impl fmt::Display for AudioWorkletState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioWorkletState::Uninitialized => write!(f, "Uninitialized"),
            AudioWorkletState::Initializing => write!(f, "Initializing"),
            AudioWorkletState::Ready => write!(f, "Ready"),
            AudioWorkletState::Processing => write!(f, "Processing"),
            AudioWorkletState::Stopped => write!(f, "Stopped"),
            AudioWorkletState::Failed => write!(f, "Failed"),
        }
    }
}

pub struct AudioWorkletManager {
    worklet_node: web_sys::AudioWorkletNode,
    state: AudioWorkletState,
    message_factory: AudioWorkletMessageFactory,
    _message_closure: Option<wasm_bindgen::closure::Closure<dyn FnMut(MessageEvent)>>,

    // Shared state for message handler
    handler_state: Rc<RefCell<MessageHandlerState>>,

    pub volume_detector: Rc<RefCell<VolumeDetector>>,
}

impl AudioWorkletManager {
    pub fn new(audio_context: AudioContext, worklet_node: web_sys::AudioWorkletNode) -> Result<Self, String> {
        let volume_detector = VolumeDetector::new(&audio_context)
            .map_err(|e| format!("Failed to create VolumeDetector: {:?}", e))?;
        
        
        Ok(Self {
            state: AudioWorkletState::Ready,
            volume_detector: Rc::new(RefCell::new(volume_detector)),
            _message_closure: None,
            handler_state: Rc::new(RefCell::new(MessageHandlerState {
                batches_processed: 0,
                buffer_pool_stats: None,
                last_volume_analysis: None,
                latest_pitch_data: None,
            })),
            message_factory: AudioWorkletMessageFactory::new(),
            worklet_node,
        })
    }
    
    pub fn setup_message_handling(&mut self, pitch_analyzer: super::pitch_analyzer::PitchAnalyzer) -> Result<(), AudioError> {
        let worklet = &self.worklet_node;
        // Clean up existing closure and port handler
        self._message_closure = None;
        
        // Clear port message handler to disconnect previous closures
        let port = worklet.port()
            .map_err(|e| AudioError::Generic(format!("Failed to get AudioWorklet port: {:?}", e)))?;
        port.set_onmessage(None);
        
        // Capture fields needed for the message handler
        let handler_state_clone = self.handler_state.clone();
        let volume_detector_clone = self.volume_detector.clone();
        let pitch_analyzer_clone = Rc::new(RefCell::new(pitch_analyzer));
        let worklet_node_clone = worklet.clone();
        let message_factory_clone = self.message_factory.clone();
        
        let closure = Closure::wrap(Box::new(move |event: MessageEvent| {
            handle_worklet_message(
                event, 
                handler_state_clone.clone(),
                volume_detector_clone.clone(),
                pitch_analyzer_clone.clone(),
                worklet_node_clone.clone(),
                message_factory_clone.clone()
            );
        }) as Box<dyn FnMut(MessageEvent)>);
        
        let port = worklet.port()
            .map_err(|e| AudioError::Generic(format!("Failed to get AudioWorklet port: {:?}", e)))?;
        port.set_onmessage(Some(closure.as_ref().unchecked_ref()));
        
        // Store the closure to prevent it from being dropped
        self._message_closure = Some(closure);
        
        dev_log!("✓ AudioWorklet message handler setup complete");
        Ok(())
    }
    
    fn send_typed_control_message(&self, message: ToWorkletMessage) -> Result<(), AudioError> {
        let envelope = match message {
            ToWorkletMessage::StartProcessing => {
                self.message_factory.start_processing()
                    .map_err(|e| AudioError::Generic(format!("Failed to create start processing message: {:?}", e)))?
            }
            ToWorkletMessage::StopProcessing => {
                self.message_factory.stop_processing()
                    .map_err(|e| AudioError::Generic(format!("Failed to create stop processing message: {:?}", e)))?
            }
            ToWorkletMessage::UpdateBatchConfig { config } => {
                self.message_factory.update_batch_config(config)
                    .map_err(|e| AudioError::Generic(format!("Failed to create batch config message: {:?}", e)))?
            }
            ToWorkletMessage::ReturnBuffer { buffer_id } => {
                self.message_factory.return_buffer(buffer_id)
                    .map_err(|e| AudioError::Generic(format!("Failed to create return buffer message: {:?}", e)))?
            }
        };
        
        let serializer = MessageSerializer::new();
        let js_message = serializer.serialize_envelope(&envelope)
            .map_err(|e| AudioError::Generic(format!("Failed to serialize message: {:?}", e)))?;
        let port = &self.worklet_node.port()
            .map_err(|e| AudioError::Generic(format!("Failed to get AudioWorklet port: {:?}", e)))?;
        port.post_message(&js_message)
            .map_err(|e| AudioError::Generic(format!("Failed to send message: {:?}", e)))?;
        
        dev_log!("Sent typed control message to AudioWorklet: {:?} (ID: {})", envelope.payload, envelope.message_id);
        Ok(())
    }

    pub fn start_processing(&mut self) -> Result<(), AudioError> {
        if self.state != AudioWorkletState::Ready {
            return Err(AudioError::Generic(
                format!("Cannot start processing in state: {}", self.state)
            ));
        }
        
        // Send start message to AudioWorklet processor
        self.send_typed_control_message(ToWorkletMessage::StartProcessing)?;
        
        self.state = AudioWorkletState::Processing;
        dev_log!("✓ Audio processing started using AudioWorklet");
        Ok(())
    }
    
    /// Stop audio processing
    pub fn stop_processing(&mut self) -> Result<(), AudioError> {
        if self.state != AudioWorkletState::Processing {
            return Err(AudioError::Generic(
                format!("Cannot stop processing in state: {}", self.state)
            ));
        }
        
        // Send stop message to AudioWorklet processor
        self.send_typed_control_message(ToWorkletMessage::StopProcessing)?;
        
        self.state = AudioWorkletState::Stopped;
        dev_log!("✓ Audio processing stopped");
        Ok(())
    }
    
    pub fn get_buffer_pool_statistics(&self) -> Option<super::message_protocol::BufferPoolStats> {
        self.handler_state.borrow().buffer_pool_stats.clone()
    }
    
    pub fn is_processing(&self) -> bool {
        matches!(self.state, AudioWorkletState::Processing)
    }

    pub fn get_status(&self) -> super::AudioWorkletStatus {
        super::AudioWorkletStatus {
            state: self.state.clone(),
            processor_loaded: true,
            chunk_size: AUDIO_CHUNK_SIZE as u32,
            batch_size: crate::app_config::BUFFER_SIZE as u32,
            batches_processed: self.handler_state.borrow().batches_processed,
        }
    }
    
    pub fn get_volume_data(&self) -> Option<super::VolumeLevelData> {
        // Check if we have volume data from the handler state (from message handler)
        if let Some(ref analysis) = self.handler_state.borrow().last_volume_analysis {
            Some(super::VolumeLevelData {
                rms_amplitude: analysis.rms_amplitude,
                peak_amplitude: analysis.peak_amplitude,
                fft_data: None,  // No FFT data available from VolumeAnalysis
            })
        } else {
            None
        }
    }

    pub fn get_pitch_data(&self) -> Option<super::pitch_detector::PitchResult> {
        self.handler_state.borrow().latest_pitch_data.clone()
    }
}

