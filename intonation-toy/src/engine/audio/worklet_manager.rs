use web_sys::AudioWorkletNode;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::MessageEvent;
use crate::common::dev_log;
use super::{
    AudioError,
    message_protocol::{AudioWorkletMessageFactory, ToWorkletMessage, FromWorkletMessage, MessageEnvelope, MessageSerializer, FromJsMessage},
    data_types::{AudioWorkletStatus, VolumeAnalysis, PitchData},
    message_protocol::BufferPoolStats,
    pitch_analyzer::PitchAnalyzer,
    volume_detector::VolumeDetector,
    data_types::AudioWorkletState,
};
use crate::app_config::AUDIO_CHUNK_SIZE;
use std::rc::Rc;
use std::cell::RefCell;


/// Shared data structure for AudioWorklet message handling
struct AudioWorkletSharedData {
    volume_detector: Option<Rc<RefCell<VolumeDetector>>>,
    batches_processed: u32,
    pitch_analyzer: Option<Rc<RefCell<PitchAnalyzer>>>,
    buffer_pool_stats: Option<BufferPoolStats>,
    last_volume_analysis: Option<VolumeAnalysis>,
}

impl AudioWorkletSharedData {
    fn new() -> Self {
        Self {
            volume_detector: None,
            batches_processed: 0,
            pitch_analyzer: None,
            buffer_pool_stats: None,
            last_volume_analysis: None,
        }
    }
}

/// Focused AudioWorkletManager - handles ONLY worklet-specific operations
/// 
/// This manager is responsible exclusively for:
/// - AudioWorklet message handling and communication  
/// - Processing control (start/stop)
/// - Worklet state management
/// - Message protocol and buffer management
/// 
/// Non-worklet functionality (audio routing, generation, analysis setup) 
/// is handled by other specialized managers in the AudioPipeline.
pub struct AudioWorkletManager {
    worklet_node: AudioWorkletNode,
    state: AudioWorkletState,
    chunk_counter: u32,
    _message_closure: Option<wasm_bindgen::closure::Closure<dyn FnMut(MessageEvent)>>,
    shared_data: Option<Rc<RefCell<AudioWorkletSharedData>>>,
    message_factory: AudioWorkletMessageFactory,
    
    // Analysis components (set by AudioPipeline, used by message handling)
    pitch_analyzer: Option<Rc<RefCell<PitchAnalyzer>>>,
    volume_detector: Option<Rc<RefCell<VolumeDetector>>>,
    last_volume_analysis: Option<VolumeAnalysis>,
}

impl AudioWorkletManager {
    /// Creates a new focused AudioWorkletManager
    /// 
    /// # Parameters
    /// - `worklet_node`: The AudioWorkletNode from the signal flow
    /// 
    /// # Returns
    /// Result containing the configured AudioWorkletManager or error description
    pub fn new(worklet_node: AudioWorkletNode) -> Result<Self, String> {
        dev_log!("Creating focused AudioWorkletManager");
        
        Ok(Self {
            worklet_node,
            state: AudioWorkletState::Ready,
            chunk_counter: 0,
            _message_closure: None,
            shared_data: None,
            message_factory: AudioWorkletMessageFactory::new(),
            pitch_analyzer: None,
            volume_detector: None,
            last_volume_analysis: None,
        })
    }

    /// Get reference to the worklet node
    pub fn get_worklet_node(&self) -> &AudioWorkletNode {
        &self.worklet_node
    }

    /// Setup message handling for the AudioWorklet processor
    pub fn setup_message_handling(&mut self) -> Result<(), AudioError> {
        let worklet = &self.worklet_node;
        // Clean up existing closure and port handler
        self._message_closure = None;
        
        // Clear port message handler to disconnect previous closures
        let port = worklet.port()
            .map_err(|e| AudioError::Generic(format!("Failed to get AudioWorklet port: {:?}", e)))?;
        port.set_onmessage(None);
        
        // Create shared data for the message handler
        let shared_data = Rc::new(RefCell::new(AudioWorkletSharedData::new()));
        
        // Store the shared data in the manager for later access
        self.shared_data = Some(shared_data.clone());
        
        // Store references to components that will be used in the handler
        if let Some(volume_detector) = &self.volume_detector {
            shared_data.borrow_mut().volume_detector = Some(volume_detector.clone());
        }
        shared_data.borrow_mut().pitch_analyzer = self.pitch_analyzer.clone();
        dev_log!("✓ Pitch analyzer passed to AudioWorklet shared data");
        
        // Capture only the specific fields needed for the message handler
        let shared_data_clone = shared_data.clone();
        let worklet_node_clone = worklet.clone();
        let message_factory_clone = self.message_factory.clone();
        
        let closure = Closure::wrap(Box::new(move |event: MessageEvent| {
            Self::handle_worklet_message_static(
                event, 
                shared_data_clone.clone(), 
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

    /// Handle messages from the AudioWorklet processor (static version)
    fn handle_worklet_message_static(
        event: MessageEvent, 
        shared_data: Rc<RefCell<AudioWorkletSharedData>>,
        worklet_node: AudioWorkletNode,
        message_factory: AudioWorkletMessageFactory
    ) {
        let data = event.data();
        
        // Try to deserialize using structured message protocol
        if let Ok(obj) = data.dyn_into::<js_sys::Object>() {
            // Try typed message deserialization first
            match Self::try_deserialize_typed_message(&obj) {
                Ok(envelope) => {
                    Self::handle_typed_worklet_message_static(
                        envelope, 
                        &shared_data, 
                        &obj,
                        worklet_node,
                        message_factory
                    );
                }
                Err(e) => {
                    dev_log!("ERROR: Failed to deserialize typed message: {}", e);
                    dev_log!("ERROR: All messages must use the structured message protocol");
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
    fn handle_typed_worklet_message_static(
        envelope: MessageEnvelope<FromWorkletMessage>,
        shared_data: &Rc<RefCell<AudioWorkletSharedData>>,
        original_obj: &js_sys::Object,
        worklet_node: AudioWorkletNode,
        message_factory: AudioWorkletMessageFactory
    ) {
        match envelope.payload {
            FromWorkletMessage::ProcessorReady { batch_size: _ } => {
                dev_log!("AudioWorklet processor ready");
                dev_log!("AudioWorklet state changed to: Ready");
            }
            FromWorkletMessage::ProcessingStarted => {
                dev_log!("AudioWorklet state changed to: Processing");
            }
            FromWorkletMessage::ProcessingStopped => {
                dev_log!("✓ AudioWorklet processing stopped");
                dev_log!("AudioWorklet state changed to: Stopped");
            }
            FromWorkletMessage::AudioDataBatch { data } => {
                Self::handle_typed_audio_data_batch_static(
                    data, 
                    shared_data, 
                    original_obj,
                    &worklet_node,
                    &message_factory
                );
            }
            FromWorkletMessage::ProcessingError { error } => {
                dev_log!("🎵 AUDIO_DEBUG: ✗ AudioWorklet processing error: {}", error);
                dev_log!("AudioWorklet state changed to: Failed");
            }
            FromWorkletMessage::BatchConfigUpdated { config: _ } => {
                // Configuration confirmation received - no action needed
                dev_log!("AudioWorklet confirmed batch configuration update");
            }
        }
    }

    /// Handle typed audio data batch from the AudioWorklet processor (static version)
    fn handle_typed_audio_data_batch_static(
        data: super::message_protocol::AudioDataBatch,
        shared_data: &Rc<RefCell<AudioWorkletSharedData>>,
        original_obj: &js_sys::Object,
        worklet_node: &AudioWorkletNode,
        message_factory: &AudioWorkletMessageFactory
    ) {
        // Extract buffer pool statistics from the audio data batch
        if let Some(buffer_pool_stats) = &data.buffer_pool_stats {
            // Store in shared data for other components
            shared_data.borrow_mut().buffer_pool_stats = Some(buffer_pool_stats.clone());
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
            
            if let Ok(buffer_val) = js_sys::Reflect::get(&payload_obj, &"buffer".into()) {
                if let Ok(array_buffer) = buffer_val.dyn_into::<js_sys::ArrayBuffer>() {
                    
                    // Convert ArrayBuffer to Float32Array for processing
                    let float32_array = js_sys::Float32Array::new(&array_buffer);
                    let array_length = float32_array.length() as usize;
                    let mut audio_samples = vec![0.0f32; array_length];
                    float32_array.copy_to(&mut audio_samples);
                    
                    // Perform actual audio processing
                    Self::process_audio_samples(&audio_samples, shared_data);
                    
                    // Return buffer to worklet for recycling (ping-pong pattern is always enabled)
                    if let Some(buffer_id) = data.buffer_id {
                        if let Err(e) = Self::return_buffer_to_worklet_static(
                            array_buffer, 
                            buffer_id,
                            worklet_node,
                            message_factory
                        ) {
                            dev_log!("Warning: Failed to return buffer to worklet: {}", e);
                        }
                    } else {
                        dev_log!("Warning: No buffer_id found in AudioDataBatch - cannot return buffer");
                    }
                } else {
                    dev_log!("Warning: Buffer field is not an ArrayBuffer");
                }
            } else {
                dev_log!("Warning: No buffer field found in payload");
            }
        } else {
            dev_log!("Warning: Could not extract payload object");
        }
        
        // Update batches processed count
        {
            let mut shared_data_mut = shared_data.borrow_mut();
            shared_data_mut.batches_processed += 1;
        }
    }

    /// Process audio samples for pitch and volume analysis
    fn process_audio_samples(
        audio_samples: &[f32],
        shared_data: &Rc<RefCell<AudioWorkletSharedData>>
    ) {
        let batches_processed = shared_data.borrow().batches_processed;
        
        // Perform volume analysis
        let volume_detector = shared_data.borrow().volume_detector.clone();
        
        if let Some(volume_detector) = volume_detector {
            match volume_detector.borrow_mut().analyze() {
                Ok(volume_analysis) => {
                    // Store the volume analysis result in the shared data
                    shared_data.borrow_mut().last_volume_analysis = Some(volume_analysis);
                }
                Err(err) => {
                    dev_log!("Volume analysis failed: {:?}", err);
                }
            }
        } 
        
        // Perform pitch analysis
        let pitch_analyzer = shared_data.borrow().pitch_analyzer.clone();
        
        if let Some(pitch_analyzer) = pitch_analyzer {
            match pitch_analyzer.borrow_mut().analyze_samples(audio_samples) {
                Ok(Some(_pitch_result)) => {
                    // Pitch data is now returned through the analyze methods
                    // and collected by Engine::update()
                }
                Ok(None) => {
                    // No pitch detected, which is normal for silence or noise
                    if batches_processed <= 5 {
                        dev_log!("No pitch detected in this batch");
                    }
                }
                Err(e) => {
                    if batches_processed <= 5 {
                        dev_log!("Pitch analysis error: {}", e);
                    }
                }
            }
        }
    }

    /// Send typed control message to AudioWorklet processor
    fn send_typed_control_message(&self, message: ToWorkletMessage) -> Result<(), AudioError> {
        let worklet = &self.worklet_node;
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
        
        let port = worklet.port()
            .map_err(|e| AudioError::Generic(format!("Failed to get AudioWorklet port: {:?}", e)))?;
        port.post_message(&js_message)
            .map_err(|e| AudioError::Generic(format!("Failed to send message: {:?}", e)))?;
        
        dev_log!("Sent typed control message to AudioWorklet: {:?} (ID: {})", envelope.payload, envelope.message_id);
        Ok(())
    }

    /// Return buffer to AudioWorklet for recycling (ping-pong pattern) - static version
    fn return_buffer_to_worklet_static(
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

    /// Start audio processing
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

    /// Check if audio processing is active
    pub fn is_processing(&self) -> bool {
        matches!(self.state, AudioWorkletState::Processing)
    }

    /// Get current audio worklet status
    pub fn get_status(&self) -> AudioWorkletStatus {
        // Get batches processed from shared data (updated by message handler) 
        let batches_processed = if let Some(ref shared_data) = self.shared_data {
            let data = shared_data.borrow();
            data.batches_processed
        } else {
            // Fallback: estimate batches from chunks
            self.chunk_counter / (crate::app_config::BUFFER_SIZE as u32 / AUDIO_CHUNK_SIZE as u32)
        };
        
        AudioWorkletStatus {
            state: self.state.clone(),
            processor_loaded: true,
            chunk_size: AUDIO_CHUNK_SIZE as u32,
            batch_size: crate::app_config::BUFFER_SIZE as u32,
            batches_processed,
        }
    }

    /// Get buffer pool statistics
    pub fn get_buffer_pool_statistics(&self) -> Option<BufferPoolStats> {
        self.shared_data.as_ref()?.borrow().buffer_pool_stats.clone()
    }

    /// Get current volume data if available
    pub fn get_volume_data(&self) -> Option<super::data_types::VolumeLevelData> {
        // First check if we have volume data from the shared data (from message handler)
        if let Some(ref shared_data) = self.shared_data {
            if let Some(ref analysis) = shared_data.borrow().last_volume_analysis {
                return Some(super::data_types::VolumeLevelData {
                    rms_amplitude: analysis.rms_amplitude,
                    peak_amplitude: analysis.peak_amplitude,
                    fft_data: analysis.fft_data.clone(),
                });
            }
        }
        
        // Fall back to the instance's last_volume_analysis
        self.last_volume_analysis.as_ref().map(|analysis| {
            super::data_types::VolumeLevelData {
                rms_amplitude: analysis.rms_amplitude,
                peak_amplitude: analysis.peak_amplitude,
                fft_data: analysis.fft_data.clone(),
            }
        })
    }

    /// Get the latest pitch data from the pitch analyzer
    /// Returns None if no pitch analyzer is configured or if data is unavailable
    pub fn get_pitch_data(&self) -> Option<PitchData> {
        self.pitch_analyzer.as_ref()
            .and_then(|analyzer| analyzer.try_borrow().ok())
            .and_then(|borrowed| borrowed.get_latest_pitch_data())
    }

    /// Set volume detector for audio analysis (called by AudioPipeline)
    pub fn set_volume_detector(&mut self, detector: VolumeDetector) {
        // Wrap the detector in Rc<RefCell<>> for shared ownership
        let detector_rc = Rc::new(RefCell::new(detector));
        
        // Store the detector instance
        self.volume_detector = Some(detector_rc.clone());
        
        // Also update the shared data with the same detector instance
        if let Some(shared_data) = &self.shared_data {
            shared_data.borrow_mut().volume_detector = Some(detector_rc);
        }
    }

    /// Set pitch analyzer for direct audio processing (called by AudioPipeline)
    pub fn set_pitch_analyzer(&mut self, analyzer: Rc<RefCell<PitchAnalyzer>>) {
        self.pitch_analyzer = Some(analyzer.clone());
        dev_log!("Pitch analyzer configured for direct processing");
        
        // Update the message handler to include the pitch analyzer
        if let Some(shared_data) = &self.shared_data {
            shared_data.borrow_mut().pitch_analyzer = Some(analyzer);
        }
    }

    /// Disconnect and cleanup the worklet manager
    pub fn disconnect(&mut self) -> Result<(), AudioError> {
        let _ = self.worklet_node.disconnect();
        dev_log!("AudioWorklet disconnected");
        
        self.state = AudioWorkletState::Uninitialized;
        Ok(())
    }
}

impl Drop for AudioWorkletManager {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}