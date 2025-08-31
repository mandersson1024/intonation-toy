
use web_sys::{
    AudioContext, AudioWorkletNode, AudioWorkletNodeOptions,
    AudioNode, MessageEvent, GainNode
};
use js_sys;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use crate::common::dev_log;
use super::{AudioError, data_types::VolumeAnalysis, SignalGeneratorConfig, volume_detector::VolumeDetector};
use super::signal_generator::TuningForkConfig;
use super::tuning_fork_node::TuningForkAudioNode;
use super::test_signal_node::TestSignalAudioNode;
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
    worklet_node: AudioWorkletNode,
    state: AudioWorkletState,
    volume_detector: Rc<RefCell<VolumeDetector>>,
    chunk_counter: u32,
    _message_closure: Option<wasm_bindgen::closure::Closure<dyn FnMut(MessageEvent)>>,
    audio_context: AudioContext,
    output_to_speakers: bool,
    shared_data: Option<Rc<RefCell<AudioWorkletSharedData>>>,
    pitch_analyzer: Option<Rc<RefCell<super::pitch_analyzer::PitchAnalyzer>>>,
    message_factory: AudioWorkletMessageFactory,
    tuning_fork_node: Option<TuningForkAudioNode>,
    test_signal_node: Option<TestSignalAudioNode>,
    legacy_mixer_gain_node: Option<GainNode>,
    legacy_microphone_gain_node: Option<GainNode>,
    legacy_microphone_source_node: Option<AudioNode>,
    prev_microphone_volume: Option<f32>,
    prev_output_to_speakers: Option<bool>,
}


impl AudioWorkletManager {
    /// Creates a new AudioWorkletManager using the return-based pattern.
    /// 
    /// This constructor is specifically designed for the return-based data flow pattern
    /// where components are created without setters and data is collected through getter
    /// methods rather than pushed to setters. The manager is initialized with default
    /// configuration and settings optimized for the return-based architecture.
    /// 
    /// # Parameters
    /// - `audio_context`: The AudioContext to use for audio operations (required)
    /// 
    /// # Default Settings
    /// - Enables ping-pong buffer recycling by default
    /// - Initializes without external dependencies or setters
    /// - Creates the AudioWorkletNode immediately
    /// 
    /// # Example
    /// ```
    /// let worklet_manager = AudioWorkletManager::new(audio_context)?;
    /// // Data is collected through getter methods rather than pushed via setters
    /// ```
    pub fn new(audio_context: AudioContext) -> Result<Self, String> {
        let worklet_node = Self::create_worklet_node(&audio_context)?;
        let volume_detector = VolumeDetector::new(&audio_context)
            .map_err(|e| format!("Failed to create VolumeDetector: {:?}", e))?;
        
        Ok(Self {
            worklet_node,
            state: AudioWorkletState::Ready,
            volume_detector: Rc::new(RefCell::new(volume_detector)),
            chunk_counter: 0,
            _message_closure: None,
            audio_context,
            output_to_speakers: false,
            shared_data: None,
            pitch_analyzer: None,
            message_factory: AudioWorkletMessageFactory::new(),
            tuning_fork_node: None,
            test_signal_node: None,
            legacy_mixer_gain_node: None,
            legacy_microphone_gain_node: None,
            legacy_microphone_source_node: None,
            prev_microphone_volume: None,
            prev_output_to_speakers: None,
        })
    }
    
    /// Create AudioWorkletNode with standard configuration
    /// 
    /// This method creates an AudioWorkletNode using standard configuration options.
    /// The worklet module must already be loaded in the AudioContext before calling this method.
    /// 
    /// # Parameters
    /// - `audio_context`: Reference to the AudioContext with worklet module loaded
    /// 
    /// # Returns
    /// Returns `Result<AudioWorkletNode, String>` where:
    /// - On success: AudioWorkletNode ready for use
    /// - On error: String describing what went wrong
    fn create_worklet_node(audio_context: &AudioContext) -> Result<AudioWorkletNode, String> {
        dev_log!("Creating AudioWorkletNode with standard configuration");
        
        // Create AudioWorkletNode with default options
        let options = AudioWorkletNodeOptions::new();
        options.set_number_of_inputs(1);
        options.set_number_of_outputs(1);
        
        // Set channel configuration
        let output_channels = js_sys::Array::of1(&js_sys::Number::from(1u32));
        options.set_channel_count(1);
        options.set_channel_count_mode(web_sys::ChannelCountMode::Explicit);
        options.set_channel_interpretation(web_sys::ChannelInterpretation::Speakers);
        options.set_output_channel_count(&output_channels);
        
        // Create the AudioWorkletNode with the registered processor
        let worklet_node = AudioWorkletNode::new_with_options(audio_context, "pitch-processor", &options)
            .map_err(|e| format!("Failed to create AudioWorkletNode 'pitch-processor': {:?}", e))?;
        
        dev_log!("✓ AudioWorkletNode created successfully");
        Ok(worklet_node)
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
        let shared_data = Rc::new(RefCell::new(
            AudioWorkletSharedData::new(&self.audio_context)
                .map_err(|e| AudioError::Generic(e))?
        ));
        
        // Store the shared data in the manager for later access
        self.shared_data = Some(shared_data.clone());
        
        // Store references to components that will be used in the handler
        // Copy volume detector reference to shared data
        shared_data.borrow_mut().volume_detector = self.volume_detector.clone();
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
        
        // Note: Status updates are handled elsewhere, no need to call publish_status_update here
    }
    
    /// Process audio samples for pitch and volume analysis
    fn process_audio_samples(
        audio_samples: &[f32],
        shared_data: &Rc<RefCell<AudioWorkletSharedData>>
    ) {
        let batches_processed = shared_data.borrow().batches_processed;
        
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

    
    /// Connect microphone input to audio worklet
    pub fn connect_microphone(&mut self, microphone_source: &AudioNode, _route_through_analyser: bool) -> Result<(), AudioError> {
        // Store microphone source
        self.legacy_microphone_source_node = Some(microphone_source.clone());
        
        // Ensure both microphone gain and mixer nodes exist
        self.ensure_microphone_gain_node()?;
        self.ensure_mixer_node()?;
        
        // Unified connection setup - always the same regardless of state:
        // Microphone Source -> Microphone Gain -> Mixer -> AudioWorklet
        
        if let Some(ref mic_gain) = self.legacy_microphone_gain_node {
            // 1. Connect microphone source to microphone gain
            microphone_source.connect_with_audio_node(mic_gain)
                .map_err(|e| AudioError::Generic(format!("Failed to connect microphone to gain node: {:?}", e)))?;
            dev_log!("Connected microphone to gain node");
            
            // 2. Connect microphone gain to volume detector (parallel tap for analysis)
            if let Err(e) = self.volume_detector.borrow().connect_source(mic_gain) {
                dev_log!("Failed to connect microphone gain to VolumeDetector: {:?}", e);
            } else {
                dev_log!("Connected microphone gain to VolumeDetector");
            }
            
            // 3. Connect microphone gain to mixer
            if let Some(ref mixer) = self.legacy_mixer_gain_node {
                mic_gain.connect_with_audio_node(mixer)
                    .map_err(|e| AudioError::Generic(format!("Failed to connect microphone gain to mixer: {:?}", e)))?;
                dev_log!("Connected microphone gain to mixer");
                
                // 4. Connect mixer to worklet
                mixer.connect_with_audio_node(&self.worklet_node)
                    .map_err(|e| AudioError::Generic(format!("Failed to connect mixer to worklet: {:?}", e)))?;
                dev_log!("Connected mixer to worklet");
            } else {
                return Err(AudioError::Generic("Mixer node not available".to_string()));
            }
        } else {
            return Err(AudioError::Generic("Microphone gain node not available".to_string()));
        }
        
        // Connect AudioWorklet to speakers if output is enabled
        if self.output_to_speakers {
            let audio_context = self.worklet_node.context();
            if let Err(e) = self.worklet_node.connect_with_audio_node(&audio_context.destination()) {
                dev_log!("Failed to connect worklet to speakers: {:?}", e);
            } else {
                dev_log!("Connected worklet to speakers");
            }
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
    
    /// Disconnect and cleanup audio worklet
    pub fn disconnect(&mut self) -> Result<(), AudioError> {
        let _ = self.worklet_node.disconnect();
        dev_log!("AudioWorklet disconnected");
        
        // Clean up the test signal node
        self.cleanup_test_signal();
        
        // Clean up the mixer node
        if let Some(mixer) = self.legacy_mixer_gain_node.take() {
            let _ = mixer.disconnect();
            dev_log!("Mixer node disconnected and cleaned up");
        }
        
        // Clean up the microphone gain node
        if let Some(mic_gain) = self.legacy_microphone_gain_node.take() {
            let _ = mic_gain.disconnect();
            dev_log!("Microphone gain node disconnected and cleaned up");
        }
        
        // Disconnect and cleanup volume detector
        if let Err(e) = self.volume_detector.borrow().disconnect() {
            dev_log!("Failed to disconnect VolumeDetector: {:?}", e);
        } else {
            dev_log!("VolumeDetector disconnected");
        }
        
        // Clear stored microphone source
        self.legacy_microphone_source_node = None;
        
        // Clear stored previous states
        self.prev_microphone_volume = None;
        self.prev_output_to_speakers = None;
        
        // Clean up the tuning fork audio node
        if self.tuning_fork_node.is_some() {
            dev_log!("[AudioWorkletManager] Cleaning up tuning fork audio node");
            self.tuning_fork_node = None; // Drop triggers cleanup
        }
        
        // worklet_node is now owned, will be dropped with self
        self.state = AudioWorkletState::Uninitialized;
        
        Ok(())
    }
    
    pub fn get_buffer_pool_statistics(&self) -> Option<super::message_protocol::BufferPoolStats> {
        self.shared_data.as_ref()?.borrow().buffer_pool_stats.clone()
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
    
    /// Ensure mixer node exists and is connected
    fn ensure_mixer_node(&mut self) -> Result<&GainNode, AudioError> {
        if self.legacy_mixer_gain_node.is_none() {
            let mixer = self.audio_context
                .create_gain()
                .map_err(|_| AudioError::Generic("Failed to create mixer gain node".to_string()))?;
            
            // Set mixer gain to unity
            mixer.gain().set_value(1.0);
            
            // Connect mixer to worklet
            mixer
                .connect_with_audio_node(&self.worklet_node)
                .map_err(|_| AudioError::Generic("Failed to connect mixer to worklet".to_string()))?;
            dev_log!("Created and connected mixer node to worklet");
            
            self.legacy_mixer_gain_node = Some(mixer);
        }
        
        Ok(self.legacy_mixer_gain_node.as_ref().unwrap())
    }
    
    /// Ensure microphone gain node exists
    fn ensure_microphone_gain_node(&mut self) -> Result<&GainNode, AudioError> {
        if self.legacy_microphone_gain_node.is_none() {
            let mic_gain = self.audio_context
                .create_gain()
                .map_err(|_| AudioError::Generic("Failed to create microphone gain node".to_string()))?;
            
            // Set initial gain to unity
            mic_gain.gain().set_value(1.0);
            
            dev_log!("Created microphone gain node with unity gain");
            self.legacy_microphone_gain_node = Some(mic_gain);
        }
        
        Ok(self.legacy_microphone_gain_node.as_ref().unwrap())
    }
    
    /// Set microphone volume
    pub fn set_microphone_volume(&mut self, volume: f32) -> Result<(), AudioError> {
        // Clamp volume to 0.0 - 1.0 range
        let clamped_volume = volume.clamp(0.0, 1.0);
        
        // Ensure microphone gain node exists
        let mic_gain = self.ensure_microphone_gain_node()?;
        
        // Set the gain value
        mic_gain.gain().set_value(clamped_volume);
        
        dev_log!("Set microphone volume to {:.2} (requested: {:.2})", clamped_volume, volume);
        
        Ok(())
    }
    
    /// Cleanup test signal node and routing
    fn cleanup_test_signal(&mut self) {
        if let Some(mut test_signal) = self.test_signal_node.take() {
            test_signal.cleanup();
            dev_log!("Cleaned up test signal node");
        }
    }
    
    /// Update test signal generator configuration (unified routing - no reconnection needed)
    pub fn update_test_signal_config(&mut self, config: SignalGeneratorConfig) {
        // Handle microphone muting for test signals to prevent feedback
        if config.enabled {
            // Store previous microphone volume before muting
            if self.prev_microphone_volume.is_none() {
                if let Some(ref mic_gain) = self.legacy_microphone_gain_node {
                    let current_volume = mic_gain.gain().value();
                    self.prev_microphone_volume = Some(current_volume);
                    dev_log!("Stored previous microphone volume: {}", current_volume);
                }
            }
            
            // Store previous speaker output state
            if self.prev_output_to_speakers.is_none() {
                self.prev_output_to_speakers = Some(self.output_to_speakers);
                dev_log!("Stored previous speaker output state: {}", self.output_to_speakers);
            }
            
            // Mute microphone to prevent feedback (no reconnection needed - just volume control)
            if let Err(e) = self.set_microphone_volume(0.0) {
                dev_log!("Failed to mute microphone for test signal: {:?}", e);
            }
            
            // Enable speaker output for test signal
            if !self.output_to_speakers {
                self.set_output_to_speakers(true);
                dev_log!("Automatically enabled speaker output for test signal");
            }
        }
        
        // Then manage local TestSignalAudioNode
        {
            if config.enabled {
                if let Some(ref mut test_signal) = self.test_signal_node {
                    // Update existing node
                    test_signal.update_config(config);
                    dev_log!("Updated existing test signal node configuration");
                } else {
                    // Create new test signal node and connect to mixer (mixer always exists now)
                    match TestSignalAudioNode::new(&self.audio_context, config, false) {
                        Ok(mut node) => {
                            // Connect test signal directly to mixer (no routing setup needed)
                            if let Some(ref mixer) = self.legacy_mixer_gain_node {
                                if let Err(e) = node.connect_to(mixer) {
                                    dev_log!("Failed to connect test signal to mixer: {:?}", e);
                                } else {
                                    dev_log!("Created and connected new test signal node to mixer");
                                }
                            } else {
                                dev_log!("Mixer not available for test signal connection");
                            }
                            self.test_signal_node = Some(node);
                        }
                        Err(e) => {
                            dev_log!("Failed to create test signal node: {:?}", e);
                        }
                    }
                }
            } else {
                // Disable test signal but keep node for potential re-enabling
                if let Some(ref mut test_signal) = self.test_signal_node {
                    test_signal.disable();
                    dev_log!("Disabled test signal node");
                }
                
                // Restore previous audio routing states
                if let Some(prev_volume) = self.prev_microphone_volume.take() {
                    if let Err(e) = self.set_microphone_volume(prev_volume) {
                        dev_log!("Failed to restore microphone volume: {:?}", e);
                    } else {
                        dev_log!("Restored microphone volume to: {}", prev_volume);
                    }
                }
                
                if let Some(prev_speakers) = self.prev_output_to_speakers.take() {
                    if self.output_to_speakers != prev_speakers {
                        self.set_output_to_speakers(prev_speakers);
                        dev_log!("Restored speaker output state to: {}", prev_speakers);
                    }
                }
            }
        }
    }

    /// Update tuning fork audio configuration
    /// 
    /// This method manages the dedicated TuningForkAudioNode that connects directly to speakers,
    /// independent of the main AudioWorklet processing pipeline. Tuning fork audio is always
    /// audible regardless of the output_to_speakers flag.
    pub fn update_tuning_fork_config(&mut self, config: TuningForkConfig) {
        dev_log!("[AudioWorkletManager] Updating tuning fork audio config - frequency: {} Hz", 
                config.frequency);
        
        // Always create or update the tuning fork audio node
        if let Some(ref mut node) = self.tuning_fork_node {
            // Update existing node
            node.update_config(config.clone());
        } else {
            // Create new node
            match TuningForkAudioNode::new(&self.audio_context, config.clone()) {
                Ok(node) => {
                    dev_log!("[AudioWorkletManager] Created new tuning fork audio node");
                    self.tuning_fork_node = Some(node);
                }
                Err(e) => {
                    dev_log!("[AudioWorkletManager] Failed to create tuning fork audio node: {:?}", e);
                }
            }
        }
        
    }


    /// Set whether to output audio stream to speakers
    pub fn set_output_to_speakers(&mut self, enabled: bool) {
        if self.output_to_speakers != enabled {
            self.output_to_speakers = enabled;
            if enabled {
                self.connect_worklet_to_speakers();
            } else {
                self.disconnect_worklet_from_speakers();
            }
        }
    }
    
    /// Connect AudioWorklet output to speakers
    fn connect_worklet_to_speakers(&self) {
        let destination = self.audio_context.destination();
        match self.worklet_node.connect_with_audio_node(&destination) {
            Ok(_) => {
                dev_log!("🔊 AudioWorklet connected to speakers");
            }
            Err(e) => {
                dev_log!("🔇 Failed to connect AudioWorklet to speakers: {:?}", e);
            }
        }
    }
    
    /// Disconnect AudioWorklet output from speakers  
    fn disconnect_worklet_from_speakers(&self) {
        let destination = self.audio_context.destination();
        // Disconnect only the connection to destination (speakers)
        match self.worklet_node.disconnect_with_audio_node(&destination) {
            Ok(_) => {
                dev_log!("🔇 AudioWorklet disconnected from speakers");
            }
            Err(e) => {
                dev_log!("⚠️ Could not disconnect from speakers (may not be connected): {:?}", e);
            }
        }
    }

    /// Get current AudioWorklet status
    pub fn get_status(&self) -> super::AudioWorkletStatus {
        // Get batches processed from shared data (updated by message handler) 
        let batches_processed = if let Some(ref shared_data) = self.shared_data {
            let data = shared_data.borrow();
            data.batches_processed
        } else {
            // Fallback: estimate batches from chunks
            self.chunk_counter / (crate::app_config::BUFFER_SIZE as u32 / AUDIO_CHUNK_SIZE as u32)
        };
        
        super::AudioWorkletStatus {
            state: self.state.clone(),
            processor_loaded: true,
            chunk_size: AUDIO_CHUNK_SIZE as u32,
            batch_size: crate::app_config::BUFFER_SIZE as u32,
            batches_processed,
        }
    }
    
    /// Get current volume analysis if available
    pub fn get_volume_data(&self) -> Option<super::VolumeLevelData> {
        // Check if we have volume data from the shared data (from message handler)
        if let Some(ref shared_data) = self.shared_data {
            if let Some(ref analysis) = shared_data.borrow().last_volume_analysis {
                return Some(super::VolumeLevelData {
                    rms_amplitude: analysis.rms_amplitude,
                    peak_amplitude: analysis.peak_amplitude,
                    fft_data: analysis.fft_data.clone(),
                });
            }
        }
        
        None
    }

    /// Get the latest pitch data from the pitch analyzer
    /// Returns None if no pitch analyzer is configured or if data is unavailable
    pub fn get_pitch_data(&self) -> Option<super::PitchData> {
        self.pitch_analyzer.as_ref()
            .and_then(|analyzer| analyzer.try_borrow().ok())
            .and_then(|borrowed| borrowed.get_latest_pitch_data())
    }

    /// Set pitch analyzer for direct audio processing
    pub fn set_pitch_analyzer(&mut self, analyzer: Rc<RefCell<super::pitch_analyzer::PitchAnalyzer>>) {
        self.pitch_analyzer = Some(analyzer);
        dev_log!("Pitch analyzer configured for direct processing");
        
        // worklet_node is always available now, update the message handler to include the pitch analyzer
        match self.setup_message_handling() {
            Ok(_) => {
                dev_log!("Message handler updated with new pitch analyzer");
            }
            Err(e) => {
                dev_log!("Failed to update message handler: {:?}", e);
            }
        }
    }
    
}

impl Drop for AudioWorkletManager {
    fn drop(&mut self) {
        // Cleanup on drop
        let _ = self.disconnect();
    }
}

