#![cfg(target_arch = "wasm32")]

use web_sys::{AudioWorkletNode, MessageEvent};
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsCast;
use crate::common::dev_log;
use super::VolumeDetector;
use super::message_protocol::{AudioWorkletMessageFactory, FromWorkletMessage, MessageEnvelope, FromJsMessage};

// Internal state that needs to be shared between the manager and message handler
pub(super) struct MessageHandlerState {
    pub(super) batches_processed: u32,
    pub(super) buffer_pool_stats: Option<super::message_protocol::BufferPoolStats>,
    pub(super) last_volume_analysis: Option<super::VolumeAnalysis>,
    pub(super) latest_pitch_data: Option<super::pitch_detector::PitchResult>,
}

/// Handle messages from the AudioWorklet processor (static version)
pub(super) fn handle_worklet_message(
    event: MessageEvent, 
    handler_state: Rc<RefCell<MessageHandlerState>>,
    volume_detector: Rc<RefCell<VolumeDetector>>,
    pitch_analyzer: Rc<RefCell<super::pitch_analyzer::PitchAnalyzer>>,
    worklet_node: AudioWorkletNode,
    message_factory: AudioWorkletMessageFactory
) {
    let data = event.data();
    
    // Try to deserialize using structured message protocol
    if let Ok(obj) = data.dyn_into::<js_sys::Object>() {
        // Try typed message deserialization first
        match try_deserialize_typed_message(&obj) {
            Ok(envelope) => {
                handle_typed_worklet_message(
                    envelope, 
                    &handler_state,
                    &volume_detector,
                    &pitch_analyzer,
                    &obj,
                    worklet_node,
                    message_factory
                );
            }
            Err(e) => {
                dev_log!("ERROR: Failed to deserialize typed message: {}", e);
            }
        }
    } else {
        dev_log!("Warning: Received non-object message from AudioWorklet");
    }
}

/// Try to deserialize a JavaScript object as a typed message envelope
fn try_deserialize_typed_message(obj: &js_sys::Object) -> Result<MessageEnvelope<FromWorkletMessage>, String> {
    
    // Check if this looks like a structured message (has message_id and payload fields)
    let has_message_id = js_sys::Reflect::has(obj, &"messageId".into()).unwrap_or(false);
    let has_payload = js_sys::Reflect::has(obj, &"payload".into()).unwrap_or(false);
    
    if !has_message_id || !has_payload {
        return Err("Not a structured message envelope".to_string());
    }
    
    // Extract the envelope fields
    let message_id = js_sys::Reflect::get(obj, &"messageId".into())
        .map_err(|e| format!("Failed to get messageId: {:?}", e))?
        .as_f64()
        .ok_or("messageId must be number")?
        as u32;
        
    let payload_obj = js_sys::Reflect::get(obj, &"payload".into())
        .map_err(|e| format!("Failed to get payload: {:?}", e))?
        .dyn_into::<js_sys::Object>()
        .map_err(|_| "payload must be object")?;
    
    // Deserialize the payload
    let payload = FromWorkletMessage::from_js_object(&payload_obj)
        .map_err(|e| format!("Failed to deserialize payload: {:?}", e))?;
    
    Ok(MessageEnvelope {
        message_id,
        payload,
    })
}

/// Handle typed messages from the AudioWorklet processor (static version)
fn handle_typed_worklet_message(
    envelope: MessageEnvelope<FromWorkletMessage>,
    handler_state: &Rc<RefCell<MessageHandlerState>>,
    volume_detector: &Rc<RefCell<VolumeDetector>>,
    pitch_analyzer: &Rc<RefCell<super::pitch_analyzer::PitchAnalyzer>>,
    original_obj: &js_sys::Object,
    worklet_node: AudioWorkletNode,
    message_factory: AudioWorkletMessageFactory
) {
    match envelope.payload {
        FromWorkletMessage::AudioDataBatch { data } => {
            handle_typed_audio_data_batch(
                data, 
                handler_state,
                volume_detector,
                pitch_analyzer,
                original_obj,
                &worklet_node,
                &message_factory
            );
        }
        FromWorkletMessage::ProcessingError { error } => {
            dev_log!("✗ AudioWorklet processing error: {}", error);
        }
    }
}

/// Handle typed audio data batch from the AudioWorklet processor (static version)
fn handle_typed_audio_data_batch(
    data: super::message_protocol::AudioDataBatch,
    handler_state: &Rc<RefCell<MessageHandlerState>>,
    volume_detector: &Rc<RefCell<VolumeDetector>>,
    pitch_analyzer: &Rc<RefCell<super::pitch_analyzer::PitchAnalyzer>>,
    original_obj: &js_sys::Object,
    worklet_node: &AudioWorkletNode,
    message_factory: &AudioWorkletMessageFactory
) {
    // Extract buffer pool statistics from the audio data batch
    if let Some(buffer_pool_stats) = &data.buffer_pool_stats {
        // Store in handler state for other components
        handler_state.borrow_mut().buffer_pool_stats = Some(buffer_pool_stats.clone());
    }
    
    // Validate the batch metadata
    if data.sample_count == 0 {
        dev_log!("Warning: Received audio data batch with zero samples");
        return;
    }
    
    if data.buffer_length == 0 {
        dev_log!("Warning: Received audio data batch with zero buffer length");
        return;
    }
    
    // Audio data batch received and processing
    
    // Extract the ArrayBuffer from the payload
    if let Ok(payload_obj) = js_sys::Reflect::get(original_obj, &"payload".into())
        .and_then(|p| p.dyn_into::<js_sys::Object>()) {
        
        let Ok(buffer_val) = js_sys::Reflect::get(&payload_obj, &"buffer".into()) else {
            dev_log!("Warning: No buffer field found in payload");
            return;
        };
        
        let Ok(array_buffer) = buffer_val.dyn_into::<js_sys::ArrayBuffer>() else {
            dev_log!("Warning: Buffer field is not an ArrayBuffer");
            return;
        };
        
        // Convert ArrayBuffer to Float32Array for processing
        let float32_array = js_sys::Float32Array::new(&array_buffer);
        let array_length = float32_array.length() as usize;
        let mut audio_samples = vec![0.0f32; array_length];
        float32_array.copy_to(&mut audio_samples);
        
        // Perform actual audio processing
        process_audio_samples(&audio_samples, handler_state, volume_detector, pitch_analyzer);
        
        // Return buffer to worklet for recycling (ping-pong pattern is always enabled)
        let Some(buffer_id) = data.buffer_id else {
            dev_log!("Warning: No buffer_id found in AudioDataBatch - cannot return buffer");
            return;
        };
        
        if let Err(e) = return_buffer_to_worklet(
            array_buffer.into(), 
            buffer_id,
            worklet_node,
            message_factory
        ) {
            dev_log!("Warning: Failed to return buffer to worklet: {}", e);
        }
    } else {
        dev_log!("Warning: Could not extract payload object");
    }
    
    // Update batches processed count
    {
        let mut handler_state_mut = handler_state.borrow_mut();
        handler_state_mut.batches_processed += 1;
    }
    
    // Note: Status updates are handled elsewhere, no need to call publish_status_update here
}

/// Process audio samples for pitch and volume analysis
fn process_audio_samples(
    audio_samples: &[f32],
    handler_state: &Rc<RefCell<MessageHandlerState>>,
    volume_detector: &Rc<RefCell<VolumeDetector>>,
    pitch_analyzer: &Rc<RefCell<super::pitch_analyzer::PitchAnalyzer>>
) {
    // Perform volume analysis
    let volume_analysis = volume_detector.borrow_mut().analyze();
    handler_state.borrow_mut().last_volume_analysis = Some(volume_analysis); 
    
    // Perform pitch analysis and store results in handler state
    let pitch_data = pitch_analyzer.borrow_mut().analyze_samples(audio_samples);
    handler_state.borrow_mut().latest_pitch_data = pitch_data;
}

/// Return buffer to AudioWorklet for recycling (ping-pong pattern) - static version
fn return_buffer_to_worklet(
    buffer: js_sys::ArrayBuffer, 
    buffer_id: u32,
    worklet_node: &AudioWorkletNode,
    message_factory: &AudioWorkletMessageFactory
) -> Result<(), super::AudioError> {
    // Create ReturnBuffer message
    let return_message = match message_factory.return_buffer(buffer_id) {
        Ok(msg) => msg,
        Err(e) => {
            return Err(super::AudioError::Generic(format!("Failed to create return buffer message: {:?}", e)));
        }
    };
    
    // Serialize the message
    let serializer = super::message_protocol::MessageSerializer::new();
    let js_message = match serializer.serialize_envelope(&return_message) {
        Ok(msg) => msg,
        Err(e) => {
            return Err(super::AudioError::Generic(format!("Failed to serialize return buffer message: {:?}", e)));
        }
    };
    
    // Add buffer to the message for transfer
    if let Err(e) = js_sys::Reflect::set(&js_message, &"buffer".into(), &buffer) {
        return Err(super::AudioError::Generic(format!("Failed to add buffer to message: {:?}", e)));
    }
    
    // Send message with buffer as transferable
    let port = worklet_node.port()
        .map_err(|e| super::AudioError::Generic(format!("Failed to get worklet port: {:?}", e)))?;
    
    let transferables = js_sys::Array::new();
    transferables.push(&buffer);
    
    port.post_message_with_transferable(&js_message, &transferables)
        .map_err(|e| super::AudioError::Generic(format!("Failed to send return buffer message: {:?}", e)))?;
    
    Ok(())
}