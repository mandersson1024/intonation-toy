
use web_sys::{
    AudioContext, AudioNode, MessageEvent, AudioWorkletNode
};
use js_sys;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use crate::common::dev_log;
use super::{AudioError, volume_detector::VolumeDetector};
use super::audio_pipeline::AudioPipeline;
use super::message_protocol::{AudioWorkletMessageFactory, ToWorkletMessage, FromWorkletMessage, MessageEnvelope, MessageSerializer, FromJsMessage};
use crate::app_config::AUDIO_CHUNK_SIZE;


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


struct AudioWorkletSharedData {
    volume_detector: Rc<RefCell<VolumeDetector>>,
    batches_processed: u32,
    pitch_analyzer: Option<Rc<RefCell<super::pitch_analyzer::PitchAnalyzer>>>,
    buffer_pool_stats: Option<super::message_protocol::BufferPoolStats>,
    last_volume_analysis: Option<super::VolumeAnalysis>,
}

impl AudioWorkletSharedData {
    fn new(audio_context: &AudioContext) -> Result<Self, String> {
        let volume_detector = VolumeDetector::new(audio_context)
            .map_err(|e| format!("Failed to create VolumeDetector: {:?}", e))?;
        Ok(Self {
            volume_detector: Rc::new(RefCell::new(volume_detector)),
            batches_processed: 0,
            pitch_analyzer: None,
            buffer_pool_stats: None,
            last_volume_analysis: None,
        })
    }
}


pub struct AudioWorkletManager {
    state: AudioWorkletState,
    volume_detector: Rc<RefCell<VolumeDetector>>,
    _message_closure: Option<wasm_bindgen::closure::Closure<dyn FnMut(MessageEvent)>>,
    pub output_to_speakers: bool,
    shared_data: Rc<RefCell<AudioWorkletSharedData>>,
    pitch_analyzer: Rc<RefCell<super::pitch_analyzer::PitchAnalyzer>>,
    message_factory: AudioWorkletMessageFactory,
    pub audio_pipeline: AudioPipeline,
    worklet_node: web_sys::AudioWorkletNode,
}


impl AudioWorkletManager {
    pub fn new(audio_context: AudioContext, audio_pipeline: AudioPipeline, worklet_node: web_sys::AudioWorkletNode) -> Result<Self, String> {
        let volume_detector = VolumeDetector::new(&audio_context)
            .map_err(|e| format!("Failed to create VolumeDetector: {:?}", e))?;
        
        let shared_data = Rc::new(RefCell::new(
            AudioWorkletSharedData::new(&audio_context)
                .map_err(|e| format!("Failed to create shared data: {}", e))?
        ));
        
        // Create PitchAnalyzer with default config and audio context sample rate
        let config = super::pitch_detector::PitchDetectorConfig::default();
        let sample_rate = audio_context.sample_rate() as u32;
        let pitch_analyzer = super::pitch_analyzer::PitchAnalyzer::new(config, sample_rate)
            .map_err(|e| format!("Failed to initialize PitchAnalyzer: {}", e))?;
        let pitch_analyzer = Rc::new(RefCell::new(pitch_analyzer));
        
        Ok(Self {
            state: AudioWorkletState::Ready,
            volume_detector: Rc::new(RefCell::new(volume_detector)),
            _message_closure: None,
            output_to_speakers: false,
            shared_data,
            pitch_analyzer,
            message_factory: AudioWorkletMessageFactory::new(),
            audio_pipeline,
            worklet_node,
        })
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
        
        // Store references to components that will be used in the handler
        // Copy volume detector reference to shared data
        self.shared_data.borrow_mut().volume_detector = self.volume_detector.clone();
        self.shared_data.borrow_mut().pitch_analyzer = Some(self.pitch_analyzer.clone());
        dev_log!("✓ Pitch analyzer passed to AudioWorklet shared data");
        
        // Capture only the specific fields needed for the message handler
        let shared_data_clone = self.shared_data.clone();
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
        
        // Note: Status updates are handled elsewhere, no need to call publish_status_update here
    }
    
    /// Process audio samples for pitch and volume analysis
    fn process_audio_samples(
        audio_samples: &[f32],
        shared_data: &Rc<RefCell<AudioWorkletSharedData>>
    ) {
        // Perform volume analysis
        let volume_detector = shared_data.borrow().volume_detector.clone();
        
        match volume_detector.borrow_mut().analyze() {
                Ok(volume_analysis) => {
                    // Store the volume analysis result in shared data
                    shared_data.borrow_mut().last_volume_analysis = Some(volume_analysis);
                }
            Err(err) => {
                dev_log!("Volume analysis failed: {:?}", err);
            }
        } 
        
        // Perform pitch analysis
        let pitch_analyzer = shared_data.borrow().pitch_analyzer.clone();
        
        if let Some(pitch_analyzer) = pitch_analyzer {
            // Perform analysis - results are collected by Engine::update()
            let _ = pitch_analyzer.borrow_mut().analyze_samples(audio_samples);
        }
    }
    
    
    /// Send typed control message to AudioWorklet processor
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

    
    /// Connect microphone input to audio worklet
    pub fn connect_microphone(&mut self, microphone_source: &AudioNode) -> Result<(), AudioError> {
        // Set up audio routing through the pipeline
        let mic_gain = self.audio_pipeline.connect_microphone(microphone_source, self.output_to_speakers)?;
        
        // Connect microphone gain to volume detector (parallel tap for analysis)
        if let Err(e) = self.volume_detector.borrow().connect_source(mic_gain) {
            dev_log!("Failed to connect microphone gain to VolumeDetector: {:?}", e);
        } else {
            dev_log!("Connected microphone gain to VolumeDetector");
        }
        
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
    
    pub fn get_buffer_pool_statistics(&self) -> Option<super::message_protocol::BufferPoolStats> {
        self.shared_data.borrow().buffer_pool_stats.clone()
    }
    
    pub fn is_processing(&self) -> bool {
        matches!(self.state, AudioWorkletState::Processing)
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
    
    
    /// Ensure microphone gain node exists
    
    




    /// Get current AudioWorklet status
    pub fn get_status(&self) -> super::AudioWorkletStatus {
        super::AudioWorkletStatus {
            state: self.state.clone(),
            processor_loaded: true,
            chunk_size: AUDIO_CHUNK_SIZE as u32,
            batch_size: crate::app_config::BUFFER_SIZE as u32,
            batches_processed: self.shared_data.borrow().batches_processed,
        }
    }
    
    /// Get current volume analysis if available
    pub fn get_volume_data(&self) -> Option<super::VolumeLevelData> {
        // Check if we have volume data from the shared data (from message handler)
        if let Some(ref analysis) = self.shared_data.borrow().last_volume_analysis {
            Some(super::VolumeLevelData {
                rms_amplitude: analysis.rms_amplitude,
                peak_amplitude: analysis.peak_amplitude,
                fft_data: analysis.fft_data.clone(),
            })
        } else {
            None
        }
    }

    /// Get the latest pitch data from the pitch analyzer
    /// Returns None if data is unavailable
    pub fn get_pitch_data(&self) -> Option<super::PitchData> {
        self.pitch_analyzer.try_borrow().ok()
            .and_then(|borrowed| borrowed.get_latest_pitch_data())
    }
}

