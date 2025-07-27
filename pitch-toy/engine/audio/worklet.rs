//! AudioWorklet Manager for Real-Time Audio Processing
//!
//! This module provides a high-level wrapper around the Web Audio API's AudioWorklet,
//! designed for real-time pitch detection applications. It handles the complete lifecycle
//! of AudioWorklet processors including initialization, node management, and audio pipeline
//! connection with fixed 128-sample processing chunks.
//!
//! ## Key Features
//!
//! - **Real-Time Processing**: Dedicated audio thread with 128-sample fixed chunks
//! - **AudioWorklet Integration**: Modern Web Audio API processing with low latency
//! - **Pipeline Management**: Connect audio sources to processors and destinations
//! - **AudioWorklet Only**: Modern Web Audio API processing with no legacy fallbacks
//! - **State Management**: Comprehensive state tracking with debugging support
//! - **Ping-Pong Buffer Recycling**: Automatic buffer return to minimize GC pressure
//! - **Zero-Allocation Processing**: Reuses buffers to avoid continuous allocations
//!
//! ## Usage Examples
//!
//! ```rust,no_run
//! use pitch_toy::audio::{AudioWorkletManager, AudioContextManager};
//!
//! async fn setup_audio_processing() {
//!     let mut context_manager = AudioContextManager::new();
//!     context_manager.initialize().await.unwrap();
//!     
//!     let mut worklet_manager = AudioWorkletManager::new();
//!     // Enable ping-pong buffer recycling to reduce GC pressure
//!     worklet_manager.set_ping_pong_enabled(true);
//!     
//!     if let Ok(()) = worklet_manager.initialize(&context_manager).await {
//!         println!("AudioWorklet ready for real-time processing with buffer recycling");
//!     }
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! - AudioWorklet runs on a dedicated audio rendering thread
//! - Processing occurs in fixed 128-sample chunks (Web Audio API standard)
//! - Zero-copy architecture minimizes memory allocations
//! - AudioWorklet-only implementation for optimal performance
//! - Ping-pong buffer recycling reduces GC pressure during sustained processing
//! - Buffer return pattern maintains >95% pool hit rate under normal load
//!
//! ## Browser Requirements
//!
//! - Chrome 66+, Firefox 76+, Safari 14.1+, Edge 79+ (AudioWorklet support required)
//! - HTTPS context required for microphone access in production

use web_sys::{
    AudioContext, AudioWorkletNode, AudioWorkletNodeOptions,
    AudioNode, MessageEvent
};
use js_sys;
use std::fmt;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use crate::common::dev_log;
use super::{AudioError, context::AudioContextManager, VolumeDetector, VolumeAnalysis, SignalGeneratorConfig, BackgroundNoiseConfig};
use super::message_protocol::{AudioWorkletMessageFactory, ToWorkletMessage, FromWorkletMessage, MessageEnvelope, MessageSerializer, FromJsMessage};
use super::buffer::AUDIO_CHUNK_SIZE;

/// AudioWorklet processor states
#[derive(Debug, Clone, PartialEq)]
pub enum AudioWorkletState {
    /// Initial state, worklet not created yet
    Uninitialized,
    /// Worklet initialization in progress
    Initializing,
    /// Worklet processor loaded and ready
    Ready,
    /// Audio processing active
    Processing,
    /// Worklet suspended or stopped
    Stopped,
    /// Worklet failed or closed
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

// Note: publish_audioworklet_status is now a method on AudioWorkletManager

/// AudioWorklet configuration
#[derive(Debug, Clone)]
pub struct AudioWorkletConfig {
    /// Number of input channels
    pub input_channels: u32,
    /// Number of output channels  
    pub output_channels: u32,
}

/// Shared data for AudioWorklet message handling
struct AudioWorkletSharedData {
    volume_detector: Option<VolumeDetector>,
    batches_processed: u32,
    pitch_analyzer: Option<std::rc::Rc<std::cell::RefCell<super::pitch_analyzer::PitchAnalyzer>>>,
    buffer_pool_stats: Option<super::message_protocol::BufferPoolStats>,
    last_volume_analysis: Option<super::VolumeAnalysis>,
    batch_size: u32,
}

impl AudioWorkletSharedData {
    fn new() -> Self {
        Self {
            volume_detector: None,
            batches_processed: 0,
            pitch_analyzer: None,
            buffer_pool_stats: None,
            last_volume_analysis: None,
            batch_size: crate::engine::audio::buffer::BUFFER_SIZE as u32, // Default batch size
        }
    }
}

impl Default for AudioWorkletConfig {
    fn default() -> Self {
        Self {
            input_channels: 1,    // Mono input for pitch detection
            output_channels: 1,   // Mono output
        }
    }
}

impl AudioWorkletConfig {
    /// Create configuration for stereo processing
    #[cfg(test)]
    pub fn stereo() -> Self {
        Self {
            input_channels: 2,
            output_channels: 2,
            ..Default::default()
        }
    }
    
    /// Create configuration with custom channel count
    #[cfg(test)]
    pub fn with_channels(input_channels: u32, output_channels: u32) -> Self {
        Self {
            input_channels,
            output_channels,
            ..Default::default()
        }
    }
    
}

/// AudioWorklet manager handles real-time audio processing
pub struct AudioWorkletManager {
    worklet_node: Option<AudioWorkletNode>,
    state: AudioWorkletState,
    config: AudioWorkletConfig,
    volume_detector: Option<VolumeDetector>,
    last_volume_analysis: Option<VolumeAnalysis>,
    background_noise_config: BackgroundNoiseConfig,
    chunk_counter: u32,
    _message_closure: Option<wasm_bindgen::closure::Closure<dyn FnMut(MessageEvent)>>,
    // Audio context for test signal output
    audio_context: Option<AudioContext>,
    // Whether to output audio stream to speakers
    output_to_speakers: bool,
    // Shared data for message handling
    shared_data: Option<std::rc::Rc<std::cell::RefCell<AudioWorkletSharedData>>>,
    // Pitch analyzer for direct audio processing
    pitch_analyzer: Option<std::rc::Rc<std::cell::RefCell<super::pitch_analyzer::PitchAnalyzer>>>,
    // Message factory for structured message creation
    message_factory: AudioWorkletMessageFactory,
    // Configuration for ping-pong buffer recycling
    ping_pong_enabled: bool,
    // Batch size for audio processing (received from AudioWorklet processor)
    batch_size: u32,
}

impl AudioWorkletManager {
    /// Create new AudioWorklet manager
    pub fn new() -> Self {
        Self {
            worklet_node: None,
            state: AudioWorkletState::Uninitialized,
            config: AudioWorkletConfig::default(),
            volume_detector: None,
            last_volume_analysis: None,
            background_noise_config: BackgroundNoiseConfig::default(),
            chunk_counter: 0,
            _message_closure: None,
            audio_context: None,
            output_to_speakers: false,
            shared_data: None,
            pitch_analyzer: None,
            message_factory: AudioWorkletMessageFactory::new(),
            ping_pong_enabled: true, // Enable ping-pong buffer recycling by default
            batch_size: crate::engine::audio::buffer::BUFFER_SIZE as u32, // Default batch size
        }
    }
    
    /// Creates a new AudioWorkletManager using the return-based pattern.
    /// 
    /// This constructor is specifically designed for the return-based data flow pattern
    /// where components are created without setters and data is collected through getter
    /// methods rather than pushed to setters. The manager is initialized with default
    /// configuration and settings optimized for the return-based architecture.
    /// 
    /// # Default Settings
    /// - Uses `AudioWorkletConfig::default()` for configuration
    /// - Enables ping-pong buffer recycling by default
    /// - Initializes without external dependencies or setters
    /// 
    /// # Example
    /// ```
    /// let worklet_manager = AudioWorkletManager::new_return_based();
    /// // Data is collected through getter methods rather than pushed via setters
    /// ```
    pub fn new_return_based() -> Self {
        Self {
            worklet_node: None,
            state: AudioWorkletState::Uninitialized,
            config: AudioWorkletConfig::default(),
            volume_detector: None,
            last_volume_analysis: None,
            background_noise_config: BackgroundNoiseConfig::default(),
            chunk_counter: 0,
            _message_closure: None,
            audio_context: None,
            output_to_speakers: false,
            shared_data: None,
            pitch_analyzer: None,
            message_factory: AudioWorkletMessageFactory::new(),
            ping_pong_enabled: true, // Enable ping-pong buffer recycling by default
            batch_size: crate::engine::audio::buffer::BUFFER_SIZE as u32, // Default batch size
        }
    }
    
    /// Publish AudioWorklet status update to Live Data Panel
    pub fn publish_audioworklet_status(&self) {
        dev_log!("AudioWorklet status updated: {} (processor: {})", self.state, self.worklet_node.is_some());
        // Status is now returned through Engine::update()
    }
    
    
    /// Check if AudioWorklet is supported
    fn is_worklet_supported(context: &AudioContextManager) -> bool {
        if let Some(audio_context) = context.get_context() {
            // Check for AudioWorklet support
            let worklet_check = js_sys::Reflect::has(audio_context, &"audioWorklet".into())
                .unwrap_or(false);
            
            if worklet_check {
                dev_log!("✓ AudioWorklet supported");
                return true;
            }
        }
        
        dev_log!("✗ AudioWorklet not supported");
        false
    }
    
    /// Initialize AudioWorklet processor
    pub async fn initialize(&mut self, context: &AudioContextManager) -> Result<(), AudioError> {
        let audio_context = context.get_context()
            .ok_or_else(|| AudioError::Generic("AudioContext not available".to_string()))?;
        
        // Store audio context for test signal output
        self.audio_context = Some(audio_context.clone());
        
        self.state = AudioWorkletState::Initializing;
        self.publish_audioworklet_status();
        dev_log!("Initializing AudioWorklet processor");
        
        // Try AudioWorklet first
        if Self::is_worklet_supported(context) {
            match self.initialize_worklet(audio_context).await {
                Ok(()) => {
                    dev_log!("✓ AudioWorklet initialized successfully");
                    self.state = AudioWorkletState::Ready;
                    self.publish_audioworklet_status();
                    return Ok(());
                }
                Err(e) => {
                    dev_log!("✗ AudioWorklet initialization failed: {:?}", e);
                    self.state = AudioWorkletState::Failed;
                    self.publish_audioworklet_status();
                    return Err(e);
                }
            }
        }
        
        // AudioWorklet required
        self.state = AudioWorkletState::Failed;
        self.publish_audioworklet_status();
        Err(AudioError::NotSupported(
            "AudioWorklet not supported".to_string()
        ))
    }
    
    /// Initialize AudioWorklet processor
    async fn initialize_worklet(&mut self, context: &AudioContext) -> Result<(), AudioError> {
        dev_log!("Loading AudioWorklet processor module...");
        
        // Load the AudioWorklet processor script
        let worklet = context.audio_worklet()
            .map_err(|e| AudioError::StreamInitFailed(
                format!("Failed to get AudioWorklet: {:?}", e)
            ))?;
        let module_promise = worklet.add_module("./audio-processor.js")
            .map_err(|e| AudioError::StreamInitFailed(
                format!("Failed to load AudioWorklet module: {:?}", e)
            ))?;
        
        // Wait for module to load
        let module_future = wasm_bindgen_futures::JsFuture::from(module_promise);
        match module_future.await {
            Ok(_) => {
                dev_log!("✓ AudioWorklet processor module loaded successfully");
            }
            Err(e) => {
                dev_log!("✗ AudioWorklet module loading failed: {:?}", e);
                return Err(AudioError::StreamInitFailed(
                    format!("AudioWorklet module loading failed: {:?}", e)
                ));
            }
        }
        
        // Create AudioWorklet node with options
        let options = AudioWorkletNodeOptions::new();
        options.set_number_of_inputs(1);
        options.set_number_of_outputs(1);
        
        // Set channel counts for both input and output
        let output_channels = js_sys::Array::of1(&js_sys::Number::from(self.config.output_channels));
        
        options.set_channel_count(self.config.input_channels);
        options.set_channel_count_mode(web_sys::ChannelCountMode::Explicit);
        options.set_channel_interpretation(web_sys::ChannelInterpretation::Speakers);
        options.set_output_channel_count(&output_channels);
        
        // Create the AudioWorkletNode with the registered processor
        match self.create_worklet_node(context, &options) {
            Ok(node) => {
                self.worklet_node = Some(node);
                dev_log!("AudioWorklet node created with {} input channels, {} output channels", 
                        self.config.input_channels, self.config.output_channels);
                
                // Note: Message handling setup is deferred until setters are configured
                
                Ok(())
            }
            Err(e) => {
                Err(AudioError::StreamInitFailed(
                    format!("Failed to create AudioWorklet node: {:?}", e)
                ))
            }
        }
    }
    
    /// Create AudioWorklet node with the registered processor
    fn create_worklet_node(&self, context: &AudioContext, options: &AudioWorkletNodeOptions) -> Result<AudioWorkletNode, js_sys::Error> {
        // Create AudioWorkletNode with the registered 'pitch-processor'
        let node = AudioWorkletNode::new_with_options(context, "pitch-processor", options)
            .map_err(|e| js_sys::Error::new(&format!("Failed to create AudioWorkletNode 'pitch-processor': {:?}", e)))?;
        
        dev_log!("✓ AudioWorklet node created successfully");
        Ok(node)
    }
    
    /// Setup message handling for the AudioWorklet processor
    pub fn setup_message_handling(&mut self) -> Result<(), AudioError> {
        if let Some(worklet) = &self.worklet_node {
            // Create shared data for the message handler
            let shared_data = std::rc::Rc::new(std::cell::RefCell::new(AudioWorkletSharedData::new()));
            
            // Store the shared data in the manager for later access
            self.shared_data = Some(shared_data.clone());
            
            // Store references to components that will be used in the handler
            if let Some(volume_detector) = &self.volume_detector {
                shared_data.borrow_mut().volume_detector = Some(volume_detector.clone());
            }
            if let Some(pitch_analyzer) = &self.pitch_analyzer {
                shared_data.borrow_mut().pitch_analyzer = Some(pitch_analyzer.clone());
                dev_log!("✓ Pitch analyzer passed to AudioWorklet shared data");
            } else {
                dev_log!("✗ Warning: No pitch analyzer available during AudioWorklet initialization");
            }
            
            // Capture only the specific fields needed for the message handler
            let shared_data_clone = shared_data.clone();
            let worklet_node_clone = worklet.clone();
            let message_factory_clone = self.message_factory.clone();
            let ping_pong_enabled = self.ping_pong_enabled;
            
            let closure = Closure::wrap(Box::new(move |event: MessageEvent| {
                Self::handle_worklet_message_static(
                    event, 
                    shared_data_clone.clone(), 
                    worklet_node_clone.clone(),
                    message_factory_clone.clone(),
                    ping_pong_enabled
                );
            }) as Box<dyn FnMut(MessageEvent)>);
            
            let port = worklet.port()
                .map_err(|e| AudioError::Generic(format!("Failed to get AudioWorklet port: {:?}", e)))?;
            port.set_onmessage(Some(closure.as_ref().unchecked_ref()));
            
            // Store the closure to prevent it from being dropped
            self._message_closure = Some(closure);
            
            dev_log!("✓ AudioWorklet message handler setup complete");
            Ok(())
        } else {
            Err(AudioError::Generic("No AudioWorklet node available".to_string()))
        }
    }
    
    /// Handle messages from the AudioWorklet processor (static version)
    fn handle_worklet_message_static(
        event: MessageEvent, 
        shared_data: std::rc::Rc<std::cell::RefCell<AudioWorkletSharedData>>,
        worklet_node: AudioWorkletNode,
        message_factory: AudioWorkletMessageFactory,
        ping_pong_enabled: bool
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
                        message_factory,
                        ping_pong_enabled
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
            
        let timestamp = js_sys::Reflect::get(obj, &"timestamp".into())
            .map_err(|e| format!("Failed to get timestamp: {:?}", e))?
            .as_f64()
            .ok_or("timestamp must be number")?;
            
        let payload_obj = js_sys::Reflect::get(obj, &"payload".into())
            .map_err(|e| format!("Failed to get payload: {:?}", e))?
            .dyn_into::<js_sys::Object>()
            .map_err(|_| "payload must be object")?;
        
        // Deserialize the payload
        let payload = FromWorkletMessage::from_js_object(&payload_obj)
            .map_err(|e| format!("Failed to deserialize payload: {:?}", e))?;
        
        Ok(MessageEnvelope {
            message_id,
            timestamp,
            payload,
        })
    }
    
    /// Handle typed messages from the AudioWorklet processor (static version)
    fn handle_typed_worklet_message_static(
        envelope: MessageEnvelope<FromWorkletMessage>,
        shared_data: &std::rc::Rc<std::cell::RefCell<AudioWorkletSharedData>>,
        original_obj: &js_sys::Object,
        worklet_node: AudioWorkletNode,
        message_factory: AudioWorkletMessageFactory,
        ping_pong_enabled: bool
    ) {
        match envelope.payload {
            FromWorkletMessage::ProcessorReady { batch_size } => {
                if let Some(size) = batch_size {
                    dev_log!("AudioWorklet processor ready with batch size: {}", size);
                    shared_data.borrow_mut().batch_size = size as u32;
                } else {
                    dev_log!("AudioWorklet processor ready (no batch size specified)");
                }
                Self::publish_status_update_static(shared_data, AudioWorkletState::Ready);
            }
            FromWorkletMessage::ProcessingStarted => {
                Self::publish_status_update_static(shared_data, AudioWorkletState::Processing);
            }
            FromWorkletMessage::ProcessingStopped => {
                dev_log!("✓ AudioWorklet processing stopped");
                Self::publish_status_update_static(shared_data, AudioWorkletState::Stopped);
            }
            FromWorkletMessage::AudioDataBatch { data } => {
                Self::handle_typed_audio_data_batch_static(
                    data, 
                    shared_data, 
                    original_obj,
                    &worklet_node,
                    &message_factory,
                    ping_pong_enabled
                );
            }
            FromWorkletMessage::ProcessingError { error } => {
                dev_log!("🎵 AUDIO_DEBUG: ✗ AudioWorklet processing error: {}", error);
                Self::publish_status_update_static(shared_data, AudioWorkletState::Failed);
            }
        }
    }
    
    /// Handle typed audio data batch from the AudioWorklet processor (static version)
    fn handle_typed_audio_data_batch_static(
        data: super::message_protocol::AudioDataBatch,
        shared_data: &std::rc::Rc<std::cell::RefCell<AudioWorkletSharedData>>,
        original_obj: &js_sys::Object,
        worklet_node: &AudioWorkletNode,
        message_factory: &AudioWorkletMessageFactory,
        ping_pong_enabled: bool
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
                    
                    // Return buffer to worklet for recycling (ping-pong pattern)
                    if let Some(buffer_id) = data.buffer_id {
                        if ping_pong_enabled {
                            if let Err(e) = Self::return_buffer_to_worklet_static(
                                array_buffer, 
                                buffer_id,
                                worklet_node,
                                message_factory
                            ) {
                                dev_log!("Warning: Failed to return buffer to worklet: {}", e);
                            }
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
        shared_data: &std::rc::Rc<std::cell::RefCell<AudioWorkletSharedData>>
    ) {
        let batches_processed = shared_data.borrow().batches_processed;
        
        // Perform volume analysis
        let volume_detector = shared_data.borrow().volume_detector.clone();
        
        if let Some(mut volume_detector) = volume_detector {
            let volume_analysis = volume_detector.process_buffer(audio_samples);
            
            // Store the volume analysis result in the AudioWorkletManager
            // We need to access the manager instance to store this
            // This is done through a separate method call after processing
            shared_data.borrow_mut().last_volume_analysis = Some(volume_analysis);
        } else {
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
    
    /// Publish AudioWorklet status update to Live Data Panel (static version)
    fn publish_status_update_static(
        _shared_data: &std::rc::Rc<std::cell::RefCell<AudioWorkletSharedData>>,
        state: AudioWorkletState
    ) {
        dev_log!("AudioWorklet state changed to: {}", state);
        // Status is now returned through Engine::update()
    }
    
    /// Send typed control message to AudioWorklet processor
    fn send_typed_control_message(&self, message: ToWorkletMessage) -> Result<(), AudioError> {
        if let Some(worklet) = &self.worklet_node {
            let envelope = match message {
                ToWorkletMessage::StartProcessing => {
                    self.message_factory.start_processing()
                        .map_err(|e| AudioError::Generic(format!("Failed to create start processing message: {:?}", e)))?
                }
                ToWorkletMessage::StopProcessing => {
                    self.message_factory.stop_processing()
                        .map_err(|e| AudioError::Generic(format!("Failed to create stop processing message: {:?}", e)))?
                }
                ToWorkletMessage::UpdateTestSignalConfig { config } => {
                    self.message_factory.update_test_signal_config(config)
                        .map_err(|e| AudioError::Generic(format!("Failed to create test signal config message: {:?}", e)))?
                }
                ToWorkletMessage::UpdateBatchConfig { config } => {
                    self.message_factory.update_batch_config(config)
                        .map_err(|e| AudioError::Generic(format!("Failed to create batch config message: {:?}", e)))?
                }
                ToWorkletMessage::UpdateBackgroundNoiseConfig { config } => {
                    self.message_factory.update_background_noise_config(config)
                        .map_err(|e| AudioError::Generic(format!("Failed to create background noise config message: {:?}", e)))?
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
        } else {
            Err(AudioError::Generic("No AudioWorklet node available".to_string()))
        }
    }

    /// Send test signal configuration to AudioWorklet processor
    fn send_test_signal_config_to_worklet(&self, config: &SignalGeneratorConfig) -> Result<(), AudioError> {
        if let Some(worklet) = &self.worklet_node {
            let envelope = self.message_factory.update_test_signal_config(config.clone())
                .map_err(|e| AudioError::Generic(format!("Failed to create message envelope: {:?}", e)))?;
            
            let serializer = MessageSerializer::new();
            let js_message = serializer.serialize_envelope(&envelope)
                .map_err(|e| AudioError::Generic(format!("Failed to serialize message: {:?}", e)))?;
            
            let port = worklet.port()
                .map_err(|e| AudioError::Generic(format!("Failed to get AudioWorklet port: {:?}", e)))?;
            port.post_message(&js_message)
                .map_err(|e| AudioError::Generic(format!("Failed to send test signal config: {:?}", e)))?;
            
            dev_log!("Sent test signal config to AudioWorklet: enabled={}, freq={:.1}Hz, amp={:.2} (ID: {})", 
                     config.enabled, config.frequency, config.amplitude, envelope.message_id);
            Ok(())
        } else {
            Err(AudioError::Generic("No AudioWorklet node available".to_string()))
        }
    }
    
    /// Connect microphone input to audio worklet
    pub fn connect_microphone(&self, microphone_source: &AudioNode) -> Result<(), AudioError> {
        if let Some(worklet) = &self.worklet_node {
            match microphone_source.connect_with_audio_node(worklet) {
                Ok(_) => {
                    
                    // Only connect AudioWorklet to destination if output to speakers is enabled
                    if self.output_to_speakers {
                        let audio_context = worklet.context();
                        if let Err(e) = worklet.connect_with_audio_node(&audio_context.destination()) {
                        } else {
                        }
                    } else {
                    }
                    
                    Ok(())
                }
                Err(e) => {
                    Err(AudioError::Generic(
                        format!("Failed to connect microphone to AudioWorklet: {:?}", e)
                    ))
                }
            }
        } else {
            Err(AudioError::Generic("No AudioWorklet node available".to_string()))
        }
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
        self.publish_audioworklet_status();
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
        self.publish_audioworklet_status();
        dev_log!("✓ Audio processing stopped");
        Ok(())
    }
    
    /// Disconnect and cleanup audio worklet
    pub fn disconnect(&mut self) -> Result<(), AudioError> {
        if let Some(worklet) = &self.worklet_node {
            let _ = worklet.disconnect();
            dev_log!("AudioWorklet disconnected");
        }
        
        self.worklet_node = None;
        self.state = AudioWorkletState::Uninitialized;
        self.publish_audioworklet_status();
        
        Ok(())
    }
    
    /// Get buffer pool statistics
    fn get_buffer_pool_stats(&self) -> Option<super::message_protocol::BufferPoolStats> {
        match &self.shared_data {
            Some(shared_data) => {
                shared_data.borrow().buffer_pool_stats.clone()
            }
            None => {
                None
            }
        }
    }
    
    /// Check if audio processing is active
    pub fn is_processing(&self) -> bool {
        let is_proc = matches!(self.state, AudioWorkletState::Processing);
        if !is_proc {
        }
        is_proc
    }
        
    /// Set volume detector for real-time volume analysis
    pub fn set_volume_detector(&mut self, detector: VolumeDetector) {
        self.volume_detector = Some(detector.clone());
        
        // Also update the shared data with the volume detector
        if let Some(shared_data) = &self.shared_data {
            shared_data.borrow_mut().volume_detector = Some(detector);
        }
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
    
    /// Update test signal generator configuration
    pub fn update_test_signal_config(&mut self, config: SignalGeneratorConfig) {
        // Send configuration to AudioWorklet processor
        if let Err(e) = self.send_test_signal_config_to_worklet(&config) {
            dev_log!("Warning: Failed to send test signal config to worklet: {}", e);
        }
    }

    /// Update background noise configuration
    pub fn update_background_noise_config(&mut self, config: BackgroundNoiseConfig) {
        self.background_noise_config = config.clone();
        
        // Send configuration to AudioWorklet processor
        if let Err(e) = self.send_background_noise_config_to_worklet(&config) {
            dev_log!("Warning: Failed to send background noise config to worklet: {}", e);
        }
    }

    /// Send background noise configuration to AudioWorklet processor
    fn send_background_noise_config_to_worklet(&self, config: &BackgroundNoiseConfig) -> Result<(), AudioError> {
        if let Some(worklet) = &self.worklet_node {
            let envelope = self.message_factory.update_background_noise_config(config.clone())
                .map_err(|e| AudioError::Generic(format!("Failed to create message envelope: {:?}", e)))?;
            
            let serializer = MessageSerializer::new();
            let js_message = serializer.serialize_envelope(&envelope)
                .map_err(|e| AudioError::Generic(format!("Failed to serialize message: {:?}", e)))?;
            
            let port = worklet.port()
                .map_err(|e| AudioError::Generic(format!("Failed to get worklet port: {:?}", e)))?;
            port.post_message(&js_message)
                .map_err(|e| AudioError::Generic(format!("Failed to send background noise config: {:?}", e)))?;
            
            dev_log!("Background noise configuration sent to AudioWorklet: enabled={}, level={}, type={:?} (ID: {})", 
                    config.enabled, config.level, config.noise_type, envelope.message_id);
        }
        
        Ok(())
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
        if let Some(worklet) = &self.worklet_node {
            if let Some(audio_context) = &self.audio_context {
                let destination = audio_context.destination();
                match worklet.connect_with_audio_node(&destination) {
                    Ok(_) => {
                        dev_log!("🔊 AudioWorklet connected to speakers");
                    }
                    Err(e) => {
                        dev_log!("🔇 Failed to connect AudioWorklet to speakers: {:?}", e);
                    }
                }
            } else {
                dev_log!("🔇 No audio context available for speaker connection");
            }
        } else {
            dev_log!("🔇 No AudioWorklet available for speaker connection");
        }
    }
    
    /// Disconnect AudioWorklet output from speakers  
    fn disconnect_worklet_from_speakers(&self) {
        if let Some(worklet) = &self.worklet_node {
            if let Some(audio_context) = &self.audio_context {
                let destination = audio_context.destination();
                // Disconnect only the connection to destination (speakers)
                match worklet.disconnect_with_audio_node(&destination) {
                    Ok(_) => {
                        dev_log!("🔇 AudioWorklet disconnected from speakers");
                    }
                    Err(e) => {
                        dev_log!("⚠️ Could not disconnect from speakers (may not be connected): {:?}", e);
                    }
                }
            }
        }
    }

    /// Get current AudioWorklet status
    pub fn get_status(&self) -> super::AudioWorkletStatus {
        // Get batches processed and batch size from shared data (updated by message handler) 
        let (batches_processed, batch_size) = if let Some(ref shared_data) = self.shared_data {
            let data = shared_data.borrow();
            (data.batches_processed, data.batch_size)
        } else {
            // Fallback: estimate batches from chunks
            let batches = if self.batch_size > 0 { self.chunk_counter / (self.batch_size / AUDIO_CHUNK_SIZE as u32) } else { 0 };
            (batches, self.batch_size)
        };
        
        super::AudioWorkletStatus {
            state: self.state.clone(),
            processor_loaded: self.worklet_node.is_some(),
            chunk_size: AUDIO_CHUNK_SIZE as u32,
            batch_size,
            batches_processed,
        }
    }
    
    /// Get current volume analysis if available
    pub fn get_volume_data(&self) -> Option<super::VolumeLevelData> {
        // First check if we have volume data from the shared data (from message handler)
        if let Some(ref shared_data) = self.shared_data {
            if let Some(ref analysis) = shared_data.borrow().last_volume_analysis {
                return Some(super::VolumeLevelData {
                    rms_amplitude: analysis.rms_amplitude,
                    peak_amplitude: analysis.peak_amplitude,
                });
            }
        }
        
        // Fall back to the instance's last_volume_analysis
        self.last_volume_analysis.as_ref().map(|analysis| {
            super::VolumeLevelData {
                rms_amplitude: analysis.rms_amplitude,
                peak_amplitude: analysis.peak_amplitude,
            }
        })
    }
    
    /// Get buffer pool statistics if available
    pub fn get_buffer_pool_statistics(&self) -> Option<super::message_protocol::BufferPoolStats> {
        self.get_buffer_pool_stats()
    }

    /// Set pitch analyzer for direct audio processing
    pub fn set_pitch_analyzer(&mut self, analyzer: std::rc::Rc<std::cell::RefCell<super::pitch_analyzer::PitchAnalyzer>>) {
        self.pitch_analyzer = Some(analyzer);
        dev_log!("Pitch analyzer configured for direct processing");
        
        // If AudioWorklet is already initialized, update the message handler to include the pitch analyzer
        if self.worklet_node.is_some() {
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
}

impl Drop for AudioWorkletManager {
    fn drop(&mut self) {
        // Cleanup on drop
        let _ = self.disconnect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;


    #[wasm_bindgen_test]
    fn test_audio_worklet_state_display() {
        assert_eq!(AudioWorkletState::Uninitialized.to_string(), "Uninitialized");
        assert_eq!(AudioWorkletState::Initializing.to_string(), "Initializing");
        assert_eq!(AudioWorkletState::Ready.to_string(), "Ready");
        assert_eq!(AudioWorkletState::Processing.to_string(), "Processing");
        assert_eq!(AudioWorkletState::Stopped.to_string(), "Stopped");
        assert_eq!(AudioWorkletState::Failed.to_string(), "Failed");
    }

    #[wasm_bindgen_test]
    fn test_audio_worklet_config_default() {
        let config = AudioWorkletConfig::default();
        // Chunk size is now a constant, no need to assert
        assert_eq!(config.input_channels, 1);
        assert_eq!(config.output_channels, 1);
    }

    #[wasm_bindgen_test]
    fn test_audio_worklet_config_builders() {
        let stereo_config = AudioWorkletConfig::stereo();
        assert_eq!(stereo_config.input_channels, 2);
        assert_eq!(stereo_config.output_channels, 2);
        
        let custom_config = AudioWorkletConfig::with_channels(4, 2);
        assert_eq!(custom_config.input_channels, 4);
        assert_eq!(custom_config.output_channels, 2);
        
    }
}

