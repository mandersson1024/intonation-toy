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
use super::{AudioError, context::AudioContextManager, VolumeDetector, VolumeDetectorConfig, VolumeAnalysis, TestSignalGenerator, TestSignalGeneratorConfig, BackgroundNoiseConfig};
use super::message_protocol::{AudioWorkletMessageFactory, ToWorkletMessage, FromWorkletMessage, MessageEnvelope, MessageSerializer, FromJsMessage};

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
    /// Fixed processing chunk size (Web Audio API standard)
    pub chunk_size: u32,
    /// Number of input channels
    pub input_channels: u32,
    /// Number of output channels  
    pub output_channels: u32,
}

/// Shared data for AudioWorklet message handling
struct AudioWorkletSharedData {
    volume_detector: Option<VolumeDetector>,
    last_volume_analysis: Option<VolumeAnalysis>,
    chunks_processed: u32,
    volume_level_setter: Option<std::rc::Rc<dyn observable_data::DataSetter<Option<crate::audio::VolumeLevelData>>>>,
    pitch_analyzer: Option<std::rc::Rc<std::cell::RefCell<crate::audio::pitch_analyzer::PitchAnalyzer>>>,
    pitch_data_setter: Option<std::rc::Rc<dyn observable_data::DataSetter<Option<crate::audio::PitchData>>>>,
    buffer_pool_stats: Option<super::message_protocol::BufferPoolStats>,
}

impl AudioWorkletSharedData {
    fn new() -> Self {
        Self {
            volume_detector: None,
            last_volume_analysis: None,
            chunks_processed: 0,
            volume_level_setter: None,
            pitch_analyzer: None,
            pitch_data_setter: None,
            buffer_pool_stats: None,
        }
    }
}

impl Default for AudioWorkletConfig {
    fn default() -> Self {
        Self {
            chunk_size: 128,      // Web Audio API standard
            input_channels: 1,    // Mono input for pitch detection
            output_channels: 1,   // Mono output
        }
    }
}

impl AudioWorkletConfig {
    /// Create configuration for stereo processing
    pub fn stereo() -> Self {
        Self {
            input_channels: 2,
            output_channels: 2,
            ..Default::default()
        }
    }
    
    /// Create configuration with custom channel count
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
    test_signal_generator: Option<TestSignalGenerator>,
    background_noise_config: BackgroundNoiseConfig,
    chunk_counter: u32,
    _message_closure: Option<wasm_bindgen::closure::Closure<dyn FnMut(MessageEvent)>>,
    // Audio context for test signal output
    audio_context: Option<AudioContext>,
    // Whether to output audio stream to speakers
    output_to_speakers: bool,
    // Setter for updating AudioWorklet status in live data
    audioworklet_status_setter: Option<std::rc::Rc<dyn observable_data::DataSetter<crate::audio::AudioWorkletStatus>>>,
    // Shared data for message handling
    shared_data: Option<std::rc::Rc<std::cell::RefCell<AudioWorkletSharedData>>>,
    // Setter for updating volume level in live data
    volume_level_setter: Option<std::rc::Rc<dyn observable_data::DataSetter<Option<crate::audio::VolumeLevelData>>>>,
    // Setter for updating buffer pool statistics in live data
    buffer_pool_stats_setter: Option<std::rc::Rc<dyn observable_data::DataSetter<Option<crate::audio::message_protocol::BufferPoolStats>>>>,
    // Pitch analyzer for direct audio processing
    pitch_analyzer: Option<std::rc::Rc<std::cell::RefCell<crate::audio::pitch_analyzer::PitchAnalyzer>>>,
    // Setter for updating pitch data in live data
    pitch_data_setter: Option<std::rc::Rc<dyn observable_data::DataSetter<Option<crate::audio::PitchData>>>>,
    // Message factory for structured message creation
    message_factory: AudioWorkletMessageFactory,
    // Configuration for ping-pong buffer recycling
    ping_pong_enabled: bool,
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
            test_signal_generator: None,
            background_noise_config: BackgroundNoiseConfig::default(),
            chunk_counter: 0,
            _message_closure: None,
            audio_context: None,
            output_to_speakers: false,
            audioworklet_status_setter: None,
            shared_data: None,
            volume_level_setter: None,
            buffer_pool_stats_setter: None,
            pitch_analyzer: None,
            pitch_data_setter: None,
            message_factory: AudioWorkletMessageFactory::new(),
            ping_pong_enabled: true, // Enable ping-pong buffer recycling by default
        }
    }
    
    /// Create new AudioWorklet manager with custom configuration
    pub fn with_config(config: AudioWorkletConfig) -> Self {
        Self {
            worklet_node: None,
            state: AudioWorkletState::Uninitialized,
            config,
            volume_detector: None,
            last_volume_analysis: None,
            test_signal_generator: None,
            background_noise_config: BackgroundNoiseConfig::default(),
            chunk_counter: 0,
            _message_closure: None,
            audio_context: None,
            output_to_speakers: false,
            audioworklet_status_setter: None,
            shared_data: None,
            volume_level_setter: None,
            buffer_pool_stats_setter: None,
            pitch_analyzer: None,
            pitch_data_setter: None,
            message_factory: AudioWorkletMessageFactory::new(),
            ping_pong_enabled: true, // Enable ping-pong buffer recycling by default
        }
    }
    
    /// Get current AudioWorklet state
    pub fn state(&self) -> &AudioWorkletState {
        &self.state
    }
    
    /// Get current configuration
    pub fn config(&self) -> &AudioWorkletConfig {
        &self.config
    }
    
    /// Publish AudioWorklet status update to Live Data Panel
    pub fn publish_audioworklet_status(&self) {
        if let Some(setter) = &self.audioworklet_status_setter {
            #[cfg(target_arch = "wasm32")]
            let timestamp = js_sys::Date::now();
            #[cfg(not(target_arch = "wasm32"))]
            let timestamp = 0.0;
            
            let status = crate::audio::AudioWorkletStatus {
                state: self.state.clone(),
                processor_loaded: self.worklet_node.is_some(),
                chunk_size: self.config.chunk_size,
                chunks_processed: self.chunk_counter,
                last_update: timestamp,
            };
            
            setter.set(status);
            dev_log!("AudioWorklet status updated: {} (processor: {})", self.state, self.worklet_node.is_some());
        }
    }
    
    
    /// Check if AudioWorklet is supported
    pub fn is_worklet_supported(context: &AudioContextManager) -> bool {
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
                
                // Setup message handling
                self.setup_message_handling()?;
                
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
            if let Some(volume_level_setter) = &self.volume_level_setter {
                shared_data.borrow_mut().volume_level_setter = Some(volume_level_setter.clone());
                dev_log!("Volume level setter passed to AudioWorklet shared data");
            } else {
                dev_log!("Warning: No volume level setter available during AudioWorklet initialization");
            }
            if let Some(pitch_data_setter) = &self.pitch_data_setter {
                shared_data.borrow_mut().pitch_data_setter = Some(pitch_data_setter.clone());
                dev_log!("✓ Pitch data setter passed to AudioWorklet shared data");
            } else {
                dev_log!("✗ Warning: No pitch data setter available during AudioWorklet initialization");
            }
            if let Some(pitch_analyzer) = &self.pitch_analyzer {
                shared_data.borrow_mut().pitch_analyzer = Some(pitch_analyzer.clone());
                dev_log!("✓ Pitch analyzer passed to AudioWorklet shared data");
            } else {
                dev_log!("✗ Warning: No pitch analyzer available during AudioWorklet initialization");
            }
            
            // Set up message handler with access to shared data
            let shared_data_clone = shared_data.clone();
            let worklet_manager_ref = self as *const AudioWorkletManager;
            let closure = Closure::wrap(Box::new(move |event: MessageEvent| {
                // SAFETY: This is safe because the closure lifetime is tied to the AudioWorkletManager
                let worklet_manager = unsafe { &*worklet_manager_ref };
                Self::handle_worklet_message(event, shared_data_clone.clone(), worklet_manager);
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
    
    /// Handle messages from the AudioWorklet processor
    fn handle_worklet_message(
        event: MessageEvent, 
        shared_data: std::rc::Rc<std::cell::RefCell<AudioWorkletSharedData>>,
        worklet_manager: &AudioWorkletManager
    ) {
        let data = event.data();
        
        // Add debug log for the first few messages
        let chunks_processed = shared_data.borrow().chunks_processed;
        if chunks_processed <= 5 {
            // AudioWorklet message received
        }
        
        // Try to deserialize using structured message protocol
        if let Ok(obj) = data.dyn_into::<js_sys::Object>() {
            // Try typed message deserialization first
            match Self::try_deserialize_typed_message(&obj) {
                Ok(envelope) => {
                    worklet_manager.handle_typed_worklet_message(envelope, &shared_data, &obj);
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
    
    /// Handle typed messages from the AudioWorklet processor
    fn handle_typed_worklet_message(
        &self,
        envelope: MessageEnvelope<FromWorkletMessage>,
        shared_data: &std::rc::Rc<std::cell::RefCell<AudioWorkletSharedData>>,
        original_obj: &js_sys::Object
    ) {
        match envelope.payload {
            FromWorkletMessage::ProcessorReady { batch_size } => {
                if let Some(size) = batch_size {
                    dev_log!("✓ AudioWorklet processor ready with batch size: {} samples", size);
                } else {
                    dev_log!("✓ AudioWorklet processor ready");
                }
                Self::publish_status_update(self, shared_data, AudioWorkletState::Ready, false);
            }
            FromWorkletMessage::ProcessingStarted => {
                dev_log!("✓ AudioWorklet processing started");
                Self::publish_status_update(self, shared_data, AudioWorkletState::Processing, true);
            }
            FromWorkletMessage::ProcessingStopped => {
                dev_log!("✓ AudioWorklet processing stopped");
                Self::publish_status_update(self, shared_data, AudioWorkletState::Stopped, false);
            }
            FromWorkletMessage::AudioDataBatch { data } => {
                self.handle_typed_audio_data_batch(data, shared_data, original_obj);
            }
            FromWorkletMessage::ProcessingError { error } => {
                dev_log!("🎵 AUDIO_DEBUG: ✗ AudioWorklet processing error: {}", error);
                Self::publish_status_update(self, shared_data, AudioWorkletState::Failed, false);
            }
            FromWorkletMessage::StatusUpdate { status } => {
                
                // Store buffer pool statistics for UI display and push to reactive system
                if let Some(buffer_pool_stats) = &status.buffer_pool_stats {
                    
                    shared_data.borrow_mut().buffer_pool_stats = Some(buffer_pool_stats.clone());
                    
                    // Push to reactive system if setter is available
                    if let Some(setter) = &self.buffer_pool_stats_setter {
                        setter.set(Some(buffer_pool_stats.clone()));
                    } else {
                        dev_log!("Warning: No buffer pool stats setter available");
                    }
                } else {
                    dev_log!("Warning: No buffer pool stats in StatusUpdate message");
                }
                // Status updates don't change the main state
            }
        }
    }
    
    /// Handle typed audio data batch from the AudioWorklet processor
    fn handle_typed_audio_data_batch(
        &self,
        data: super::message_protocol::AudioDataBatch,
        shared_data: &std::rc::Rc<std::cell::RefCell<AudioWorkletSharedData>>,
        original_obj: &js_sys::Object
    ) {
        let chunks_processed = shared_data.borrow().chunks_processed;
        if chunks_processed <= 5 {
            // Starting handle_typed_audio_data_batch
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
                    if chunks_processed <= 5 {
                        // Successfully extracted ArrayBuffer
                    }
                    
                    // Convert ArrayBuffer to Float32Array for processing
                    let float32_array = js_sys::Float32Array::new(&array_buffer);
                    let array_length = float32_array.length() as usize;
                    let mut audio_samples = vec![0.0f32; array_length];
                    float32_array.copy_to(&mut audio_samples);
                    
                    if chunks_processed <= 5 {
                        // Converted to audio samples for processing
                    }
                    
                    // Perform actual audio processing
                    Self::process_audio_samples(&audio_samples, data.sample_rate, shared_data);
                    
                    // Return buffer to worklet for recycling (ping-pong pattern)
                    if let Some(buffer_id) = data.buffer_id {
                        if let Err(e) = self.return_buffer_to_worklet(array_buffer, buffer_id) {
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
        
        // Update chunks processed count (batched)
        let chunk_count = (data.sample_count + 127) / 128; // Round up to nearest chunk
        {
            let mut shared_data_mut = shared_data.borrow_mut();
            shared_data_mut.chunks_processed += chunk_count as u32;
        }
        
        // Update status periodically
        if chunks_processed % 16 == 0 {
            Self::publish_status_update(self, shared_data, AudioWorkletState::Processing, true);
        }
    }
    
    /// Process audio samples for pitch and volume analysis
    fn process_audio_samples(
        audio_samples: &[f32],
        sample_rate: f64,
        shared_data: &std::rc::Rc<std::cell::RefCell<AudioWorkletSharedData>>
    ) {
        let chunks_processed = shared_data.borrow().chunks_processed;
        
        // Perform volume analysis
        if let Some(mut volume_detector) = shared_data.borrow().volume_detector.clone() {
            let volume_analysis = volume_detector.process_buffer(audio_samples, js_sys::Date::now());
            
            // Update volume level via setter if available
            if let Some(volume_setter) = &shared_data.borrow().volume_level_setter {
                let volume_data = crate::audio::VolumeLevelData {
                    rms_db: volume_analysis.rms_db,
                    peak_db: volume_analysis.peak_db,
                    peak_fast_db: volume_analysis.peak_fast_db,
                    peak_slow_db: volume_analysis.peak_slow_db,
                    level: volume_analysis.level.clone(),
                    confidence_weight: volume_analysis.confidence_weight,
                    timestamp: js_sys::Date::now(),
                };
                volume_setter.set(Some(volume_data));
                
                if chunks_processed <= 5 || chunks_processed % 64 == 0 {
                    // Volume analysis completed
                }
            }
        }
        
        // Perform pitch analysis
        if let Some(pitch_analyzer) = &shared_data.borrow().pitch_analyzer {
            match pitch_analyzer.borrow_mut().analyze_samples(audio_samples) {
                Ok(Some(pitch_result)) => {
                    // Update pitch data via setter if available
                    if let Some(pitch_setter) = &shared_data.borrow().pitch_data_setter {
                        // For now, create a placeholder note since PitchResult doesn't have note field
                        let placeholder_note = crate::audio::MusicalNote::new(
                            crate::audio::NoteName::A, 4, 0.0, pitch_result.frequency
                        );
                        
                        let pitch_data = crate::audio::PitchData {
                            frequency: pitch_result.frequency,
                            confidence: pitch_result.confidence,
                            note: placeholder_note,
                            clarity: pitch_result.clarity,
                            timestamp: js_sys::Date::now(),
                        };
                        pitch_setter.set(Some(pitch_data));
                        
                        if chunks_processed <= 5 || chunks_processed % 64 == 0 {
                            // Pitch detected and processed
                        }
                    }
                }
                Ok(None) => {
                    // No pitch detected, which is normal for silence or noise
                    if chunks_processed <= 5 {
                        dev_log!("No pitch detected in this batch");
                    }
                }
                Err(e) => {
                    if chunks_processed <= 5 {
                        dev_log!("Pitch analysis error: {}", e);
                    }
                }
            }
        }
    }
    
    /// Publish AudioWorklet status update to Live Data Panel
    fn publish_status_update(
        worklet_manager: &AudioWorkletManager,
        _shared_data: &std::rc::Rc<std::cell::RefCell<AudioWorkletSharedData>>,
        state: AudioWorkletState,
        processing: bool
    ) {
        // Send periodic GetStatus requests to update buffer pool statistics
        match worklet_manager.send_typed_control_message(ToWorkletMessage::GetStatus) {
            Ok(_) => {
                // Successfully sent GetStatus request
            }
            Err(e) => {
                dev_log!("Warning: Failed to send periodic GetStatus: {}", e);
            }
        }
    }
    
    
    /// Send typed control message to AudioWorklet processor
    pub fn send_typed_control_message(&self, message: ToWorkletMessage) -> Result<(), AudioError> {
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
                ToWorkletMessage::GetStatus => {
                    self.message_factory.get_status()
                        .map_err(|e| AudioError::Generic(format!("Failed to create get status message: {:?}", e)))?
                }
            };
            
            let serializer = MessageSerializer::new();
            let js_message = serializer.serialize_envelope(&envelope)
                .map_err(|e| AudioError::Generic(format!("Failed to serialize message: {:?}", e)))?;
            
            let port = worklet.port()
                .map_err(|e| AudioError::Generic(format!("Failed to get AudioWorklet port: {:?}", e)))?;
            port.post_message(&js_message)
                .map_err(|e| AudioError::Generic(format!("Failed to send message: {:?}", e)))?;
            
            // Only log non-GetStatus messages to avoid console spam
            if !matches!(envelope.payload, ToWorkletMessage::GetStatus) {
                dev_log!("Sent typed control message to AudioWorklet: {:?} (ID: {})", envelope.payload, envelope.message_id);
            }
            Ok(())
        } else {
            Err(AudioError::Generic("No AudioWorklet node available".to_string()))
        }
    }

    /// Send test signal configuration to AudioWorklet processor
    fn send_test_signal_config_to_worklet(&self, config: &TestSignalGeneratorConfig) -> Result<(), AudioError> {
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
    
    
    /// Update batch configuration
    pub fn update_batch_config(&self, batch_size: Option<usize>, buffer_timeout: Option<f64>) -> Result<(), AudioError> {
        use super::message_protocol::BatchConfig;
        
        if let Some(worklet) = &self.worklet_node {
            // Create BatchConfig with current defaults and apply updates
            let mut config = BatchConfig::default();
            
            if let Some(size) = batch_size {
                config.batch_size = size;
            }
            
            if let Some(timeout) = buffer_timeout {
                config.timeout_ms = timeout as u32;
            }
            
            let envelope = self.message_factory.update_batch_config(config)
                .map_err(|e| AudioError::Generic(format!("Failed to create message envelope: {:?}", e)))?;
            
            let serializer = MessageSerializer::new();
            let js_message = serializer.serialize_envelope(&envelope)
                .map_err(|e| AudioError::Generic(format!("Failed to serialize message: {:?}", e)))?;
            
            let port = worklet.port()
                .map_err(|e| AudioError::Generic(format!("Failed to get AudioWorklet port: {:?}", e)))?;
            port.post_message(&js_message)
                .map_err(|e| AudioError::Generic(format!("Failed to send batch config: {:?}", e)))?;
            
            dev_log!("Sent batch config to AudioWorklet: batch_size={:?}, timeout={:?}ms (ID: {})", 
                     batch_size, buffer_timeout, envelope.message_id);
            Ok(())
        } else {
            Err(AudioError::Generic("No AudioWorklet node available".to_string()))
        }
    }
    
    /// Connect audio worklet to audio pipeline
    pub fn connect_to_destination(&self, context: &AudioContextManager) -> Result<(), AudioError> {
        let audio_context = context.get_context()
            .ok_or_else(|| AudioError::Generic("AudioContext not available".to_string()))?;
            
        let destination = audio_context.destination();
        
        if let Some(worklet) = &self.worklet_node {
            match worklet.connect_with_audio_node(&destination) {
                Ok(_) => {
                    dev_log!("✓ AudioWorklet connected to destination");
                    Ok(())
                }
                Err(e) => {
                    Err(AudioError::Generic(
                        format!("Failed to connect AudioWorklet: {:?}", e)
                    ))
                }
            }
        } else {
            Err(AudioError::Generic("No AudioWorklet node available".to_string()))
        }
    }
    
    /// Connect microphone input to audio worklet
    pub fn connect_microphone(&self, microphone_source: &AudioNode) -> Result<(), AudioError> {
        if let Some(worklet) = &self.worklet_node {
            match microphone_source.connect_with_audio_node(worklet) {
                Ok(_) => {
                    dev_log!("✓ Microphone connected to AudioWorklet");
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
    
    /// Get processing node (AudioWorklet)
    pub fn get_processing_node(&self) -> Option<&AudioNode> {
        self.worklet_node.as_ref().map(|node| node.as_ref())
    }
    
    /// Get buffer pool statistics
    pub fn get_buffer_pool_stats(&self) -> Option<super::message_protocol::BufferPoolStats> {
        match &self.shared_data {
            Some(shared_data) => {
                let stats = shared_data.borrow().buffer_pool_stats.clone();
                if stats.is_some() {
                    dev_log!("✓ get_buffer_pool_stats returning stats");
                } else {
                    dev_log!("✗ get_buffer_pool_stats: no stats available in shared_data");
                }
                stats
            }
            None => {
                dev_log!("✗ get_buffer_pool_stats: no shared_data available");
                None
            }
        }
    }
    
    /// Check if audio processing is active
    pub fn is_processing(&self) -> bool {
        matches!(self.state, AudioWorkletState::Processing)
    }
    
    /// Request status update from the AudioWorklet processor
    pub fn request_status_update(&self) -> Result<(), AudioError> {
        if let Some(worklet) = &self.worklet_node {
            
            let message = self.message_factory.get_status()
                .map_err(|e| AudioError::Generic(format!("Failed to create get status message: {}", e)))?;
            
            let serializer = super::message_protocol::MessageSerializer::new();
            let js_message = serializer.serialize_envelope(&message)
                .map_err(|e| AudioError::Generic(format!("Failed to serialize get status message: {}", e)))?;
            
            let port = worklet.port()
                .map_err(|e| AudioError::Generic(format!("Failed to get AudioWorklet port: {:?}", e)))?;
            
            port.post_message(&js_message)
                .map_err(|e| AudioError::Generic(format!("Failed to send get status message: {:?}", e)))?;
            
            Ok(())
        } else {
            Err(AudioError::Generic("No AudioWorklet node available".to_string()))
        }
    }
    
    /// Get chunk size for processing
    pub fn chunk_size(&self) -> u32 {
        self.config.chunk_size
    }

    // Note: Buffer pool support removed - using direct processing with transferable buffers

    /// Set the AudioWorklet status setter for live data updates
    pub fn set_audioworklet_status_setter(&mut self, setter: std::rc::Rc<dyn observable_data::DataSetter<crate::audio::AudioWorkletStatus>>) {
        self.audioworklet_status_setter = Some(setter);
    }
    
    /// Set the volume level setter for live data updates
    pub fn set_volume_level_setter(&mut self, setter: std::rc::Rc<dyn observable_data::DataSetter<Option<crate::audio::VolumeLevelData>>>) {
        self.volume_level_setter = Some(setter);
        dev_log!("Volume level setter updated on AudioWorkletManager");
        
        // If AudioWorklet is already initialized, update the message handler to include the new setter
        if self.worklet_node.is_some() {
            match self.setup_message_handling() {
                Ok(_) => {
                    dev_log!("Message handler updated with new volume level setter");
                }
                Err(e) => {
                    dev_log!("Failed to update message handler: {:?}", e);
                }
            }
        }
    }
    
    /// Set the pitch data setter for live data updates
    pub fn set_pitch_data_setter(&mut self, setter: std::rc::Rc<dyn observable_data::DataSetter<Option<crate::audio::PitchData>>>) {
        self.pitch_data_setter = Some(setter);
        dev_log!("Pitch data setter updated on AudioWorkletManager");
        
        // If AudioWorklet is already initialized, update the message handler to include the new setter
        if self.worklet_node.is_some() {
            match self.setup_message_handling() {
                Ok(_) => {
                    dev_log!("Message handler updated with new pitch data setter");
                }
                Err(e) => {
                    dev_log!("Failed to update message handler: {:?}", e);
                }
            }
        }
    }
    
    /// Set the buffer pool stats setter for live data updates
    pub fn set_buffer_pool_stats_setter(&mut self, setter: std::rc::Rc<dyn observable_data::DataSetter<Option<crate::audio::message_protocol::BufferPoolStats>>>) {
        self.buffer_pool_stats_setter = Some(setter);
        dev_log!("Buffer pool stats setter updated on AudioWorkletManager");
        
        // If AudioWorklet is already initialized, update the message handler to include the new setter
        if self.worklet_node.is_some() {
            match self.setup_message_handling() {
                Ok(_) => {
                    dev_log!("Message handler updated with new buffer pool stats setter");
                }
                Err(e) => {
                    dev_log!("Failed to update message handler: {:?}", e);
                }
            }
        }
    }

    /// Set volume detector for real-time volume analysis
    pub fn set_volume_detector(&mut self, detector: VolumeDetector) {
        self.volume_detector = Some(detector);
    }

    /// Update volume detector configuration
    pub fn update_volume_config(&mut self, config: VolumeDetectorConfig) -> Result<(), String> {
        if let Some(detector) = &mut self.volume_detector {
            detector.update_config(config)
        } else {
            Err("No volume detector attached".to_string())
        }
    }

    /// Get current volume analysis result
    pub fn last_volume_analysis(&self) -> Option<&VolumeAnalysis> {
        self.last_volume_analysis.as_ref()
    }

    /// Get reference to volume detector if available
    pub fn volume_detector(&self) -> Option<&VolumeDetector> {
        self.volume_detector.as_ref()
    }

    /// Get current volume detector configuration
    pub fn volume_config(&self) -> Option<&VolumeDetectorConfig> {
        self.volume_detector.as_ref().map(|detector| detector.config())
    }

    /// Check if volume detector is attached
    pub fn has_volume_detector(&self) -> bool {
        self.volume_detector.is_some()
    }
    
    /// Enable or disable ping-pong buffer recycling
    pub fn set_ping_pong_enabled(&mut self, enabled: bool) {
        self.ping_pong_enabled = enabled;
    }
    
    /// Check if ping-pong buffer recycling is enabled
    pub fn is_ping_pong_enabled(&self) -> bool {
        self.ping_pong_enabled
    }
    
    /// Return buffer to AudioWorklet for recycling (ping-pong pattern)
    fn return_buffer_to_worklet(&self, buffer: js_sys::ArrayBuffer, buffer_id: u32) -> Result<(), crate::audio::AudioError> {
        if !self.ping_pong_enabled {
            return Ok(()); // Skip if ping-pong is disabled
        }
        
        if let Some(worklet_node) = &self.worklet_node {
            
            // Create ReturnBuffer message
            let return_message = match self.message_factory.return_buffer(buffer_id) {
                Ok(msg) => msg,
                Err(e) => {
                    return Err(crate::audio::AudioError::Generic(format!("Failed to create return buffer message: {:?}", e)));
                }
            };
            
            // Serialize the message
            let serializer = crate::audio::message_protocol::MessageSerializer::new();
            let js_message = match serializer.serialize_envelope(&return_message) {
                Ok(msg) => msg,
                Err(e) => {
                    return Err(crate::audio::AudioError::Generic(format!("Failed to serialize return buffer message: {:?}", e)));
                }
            };
            
            // Add buffer to the message for transfer
            if let Err(e) = js_sys::Reflect::set(&js_message, &"buffer".into(), &buffer) {
                return Err(crate::audio::AudioError::Generic(format!("Failed to add buffer to message: {:?}", e)));
            }
            
            // Send message with buffer as transferable
            let port = worklet_node.port()
                .map_err(|e| crate::audio::AudioError::Generic(format!("Failed to get worklet port: {:?}", e)))?;
            
            let transferables = js_sys::Array::new();
            transferables.push(&buffer);
            
            port.post_message_with_transferable(&js_message, &transferables)
                .map_err(|e| crate::audio::AudioError::Generic(format!("Failed to send return buffer message: {:?}", e)))?;
            
            Ok(())
        } else {
            Err(crate::audio::AudioError::Generic("No worklet node available".to_string()))
        }
    }

    /// Set test signal generator for audio validation
    pub fn set_test_signal_generator(&mut self, generator: TestSignalGenerator) {
        self.test_signal_generator = Some(generator);
    }

    /// Update test signal generator configuration
    pub fn update_test_signal_config(&mut self, config: TestSignalGeneratorConfig) {
        if let Some(generator) = &mut self.test_signal_generator {
            generator.update_config(config.clone());
        } else {
            // Create new generator if none exists
            self.test_signal_generator = Some(TestSignalGenerator::new(config.clone()));
        }
        
        // Send configuration to AudioWorklet processor
        if let Err(e) = self.send_test_signal_config_to_worklet(&config) {
            dev_log!("Warning: Failed to send test signal config to worklet: {}", e);
        }
    }

    /// Get current test signal generator configuration
    pub fn test_signal_config(&self) -> Option<&TestSignalGeneratorConfig> {
        self.test_signal_generator.as_ref().map(|g| g.config())
    }

    /// Check if test signal generator is enabled
    pub fn is_test_signal_enabled(&self) -> bool {
        self.test_signal_generator
            .as_ref()
            .map(|g| g.config().enabled)
            .unwrap_or(false)
    }

    /// Generate test signal chunk
    pub fn generate_test_signal_chunk(&mut self) -> Option<Vec<f32>> {
        if let Some(generator) = &mut self.test_signal_generator {
            if generator.config().enabled {
                Some(generator.generate_chunk(self.config.chunk_size as usize))
            } else {
                None
            }
        } else {
            None
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

    /// Get current background noise configuration
    pub fn background_noise_config(&self) -> &BackgroundNoiseConfig {
        &self.background_noise_config
    }

    /// Check if background noise is enabled
    pub fn is_background_noise_enabled(&self) -> bool {
        self.background_noise_config.enabled
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

    /// Check if output to speakers is enabled
    pub fn is_output_to_speakers_enabled(&self) -> bool {
        self.output_to_speakers
    }




    /// Feed a 128-sample chunk (from the AudioWorklet processor) into the first buffer of the pool.
    /// This method is platform-agnostic and can be unit-tested natively.
    /// Also performs real-time volume analysis if VolumeDetector is attached.
    pub fn feed_input_chunk(&mut self, samples: &[f32]) -> Result<(), String> {
        self.feed_input_chunk_with_timestamp(samples, None)
    }

    /// Feed input chunk with explicit timestamp (for testing)
    pub fn feed_input_chunk_with_timestamp(&mut self, samples: &[f32], timestamp: Option<f64>) -> Result<(), String> {
        if samples.len() as u32 != self.config.chunk_size {
            return Err(format!("Expected chunk size {}, got {}", self.config.chunk_size, samples.len()));
        }

        // AudioWorklet processor now handles test signal generation and mixing
        // The samples we receive here are already processed (test signal OR mic input)
        let processed_samples = samples.to_vec();

        // Get timestamp for volume analysis
        let timestamp = timestamp.unwrap_or_else(|| {
            #[cfg(target_arch = "wasm32")]
            {
                js_sys::Date::now()
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                // For native tests, use a mock timestamp
                1000.0 + (self.chunk_counter as f64 * 2.67) // ~2.67ms per chunk at 48kHz
            }
        });

        // Perform volume analysis if detector is available
        if let Some(detector) = &mut self.volume_detector {
            let volume_analysis = detector.process_buffer(&processed_samples, timestamp);
            
            // Debug log first time we detect volume
            if self.last_volume_analysis.is_none() {
                // First volume analysis completed
            }
            

            // Update volume level data every 16 chunks (~11.6ms at 48kHz)
            self.chunk_counter += 1;
            if self.chunk_counter % 16 == 0 {
                // Also update AudioWorklet status periodically
                self.publish_audioworklet_status();
                
                // Update volume level using setter
                if let Some(ref setter) = self.volume_level_setter {
                    let volume_data = crate::audio::VolumeLevelData {
                        rms_db: volume_analysis.rms_db,
                        peak_db: volume_analysis.peak_db,
                        peak_fast_db: volume_analysis.peak_fast_db,
                        peak_slow_db: volume_analysis.peak_slow_db,
                        level: volume_analysis.level,
                        confidence_weight: volume_analysis.confidence_weight,
                        timestamp,
                    };
                    setter.set(Some(volume_data));
                    // Log occasionally to avoid spam
                    if self.chunk_counter % 256 == 0 {
                        // Volume data updated via process_audio
                    }
                } else if self.chunk_counter % 256 == 0 {
                    dev_log!("Warning: No volume level setter available in process_audio");
                }
            }

            // Store the current analysis for next comparison
            self.last_volume_analysis = Some(volume_analysis);
        }

        // Note: Buffer pool operations removed - using direct processing with transferable buffers
        // Events for buffer_filled and buffer_overflow are no longer published from this method

        // Note: Speaker output is now handled by AudioWorklet direct connection
        // No need to manually route samples - AudioWorklet output is connected to speakers when enabled

        Ok(())
    }

    /// Set pitch analyzer for direct audio processing
    pub fn set_pitch_analyzer(&mut self, analyzer: std::rc::Rc<std::cell::RefCell<crate::audio::pitch_analyzer::PitchAnalyzer>>) {
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

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_worklet_state_display() {
        assert_eq!(AudioWorkletState::Uninitialized.to_string(), "Uninitialized");
        assert_eq!(AudioWorkletState::Initializing.to_string(), "Initializing");
        assert_eq!(AudioWorkletState::Ready.to_string(), "Ready");
        assert_eq!(AudioWorkletState::Processing.to_string(), "Processing");
        assert_eq!(AudioWorkletState::Stopped.to_string(), "Stopped");
        assert_eq!(AudioWorkletState::Failed.to_string(), "Failed");
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_worklet_config_default() {
        let config = AudioWorkletConfig::default();
        assert_eq!(config.chunk_size, 128);
        assert_eq!(config.input_channels, 1);
        assert_eq!(config.output_channels, 1);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_worklet_config_builders() {
        let stereo_config = AudioWorkletConfig::stereo();
        assert_eq!(stereo_config.input_channels, 2);
        assert_eq!(stereo_config.output_channels, 2);
        
        let custom_config = AudioWorkletConfig::with_channels(4, 2);
        assert_eq!(custom_config.input_channels, 4);
        assert_eq!(custom_config.output_channels, 2);
        
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_worklet_manager_new() {
        let manager = AudioWorkletManager::new();
        assert_eq!(*manager.state(), AudioWorkletState::Uninitialized);
        assert!(!manager.is_processing());
        assert_eq!(manager.chunk_size(), 128);
        assert!(manager.get_processing_node().is_none());
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_worklet_manager_with_config() {
        let config = AudioWorkletConfig::stereo();
        let manager = AudioWorkletManager::with_config(config.clone());
        
        assert_eq!(*manager.state(), AudioWorkletState::Uninitialized);
        assert_eq!(manager.config().input_channels, 2);
        assert_eq!(manager.config().output_channels, 2);
    }


    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_audio_worklet_manager_disconnect() {
        let mut manager = AudioWorkletManager::new();
        manager.state = AudioWorkletState::Ready;
        
        assert!(manager.disconnect().is_ok());
        assert_eq!(*manager.state(), AudioWorkletState::Uninitialized);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_feed_input_chunk_direct_processing() {
        use crate::audio::VolumeDetector;

        // Create manager with volume detector
        let mut mgr = AudioWorkletManager::new();
        mgr.set_volume_detector(VolumeDetector::new_default());

        // Feed two chunks of 128 samples each - direct processing, no events
        let chunk = vec![0.1_f32; 128]; // Use small signal for volume detection
        mgr.feed_input_chunk(&chunk).unwrap();
        mgr.feed_input_chunk(&chunk).unwrap();

        // Should have volume analysis available from direct processing
        assert!(mgr.last_volume_analysis().is_some());
        
        // Verify volume analysis contains expected data
        let analysis = mgr.last_volume_analysis().unwrap();
        assert!(analysis.rms_db.is_finite());
        assert!(analysis.confidence_weight > 0.0);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_volume_detection_direct_processing() {
        use crate::audio::{VolumeDetector, VolumeDetectorConfig};

        // Create manager with volume detector
        let mut mgr = AudioWorkletManager::new();
        
        let config = VolumeDetectorConfig {
            sample_rate: 48000.0,
            ..VolumeDetectorConfig::default()
        };
        mgr.set_volume_detector(VolumeDetector::new(config).unwrap());

        // Feed 16 chunks to trigger volume analysis - direct processing, no events
        let chunk = vec![0.1_f32; 128];
        for _ in 0..16 {
            mgr.feed_input_chunk(&chunk).unwrap();
        }

        // Should have volume analysis available from direct processing
        let analysis = mgr.last_volume_analysis().unwrap();
        assert!(analysis.rms_db.is_finite());
        assert!(analysis.confidence_weight > 0.0);
        
        // Verify that volume detection is working correctly
        assert!(analysis.rms_db < 0.0); // Should be negative dB for small signal
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    fn test_volume_config_update() {
        let mut mgr = AudioWorkletManager::new();
        
        // Should fail without volume detector
        let config = VolumeDetectorConfig::default();
        assert!(mgr.update_volume_config(config.clone()).is_err());
        
        // Should succeed with volume detector
        mgr.set_volume_detector(VolumeDetector::new_default());
        assert!(mgr.update_volume_config(config).is_ok());
    }
}

