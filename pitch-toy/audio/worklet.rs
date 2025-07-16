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
//!     if let Ok(()) = worklet_manager.initialize(&context_manager).await {
//!         println!("AudioWorklet ready for real-time processing");
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
use super::{AudioError, context::AudioContextManager, VolumeDetector, VolumeDetectorConfig, VolumeAnalysis, TestSignalGenerator, TestSignalGeneratorConfig, TestWaveform, BackgroundNoiseConfig};

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
    event_dispatcher: Option<crate::events::AudioEventDispatcher>,
    volume_detector: Option<VolumeDetector>,
    last_volume_analysis: Option<VolumeAnalysis>,
    chunks_processed: u32,
    volume_level_setter: Option<std::rc::Rc<dyn observable_data::DataSetter<Option<crate::debug::egui::live_data_panel::VolumeLevelData>>>>,
    pitch_analyzer: Option<std::rc::Rc<std::cell::RefCell<crate::audio::pitch_analyzer::PitchAnalyzer>>>,
    pitch_data_setter: Option<std::rc::Rc<dyn observable_data::DataSetter<Option<crate::debug::egui::live_data_panel::PitchData>>>>,
}

impl AudioWorkletSharedData {
    fn new() -> Self {
        Self {
            event_dispatcher: None,
            volume_detector: None,
            last_volume_analysis: None,
            chunks_processed: 0,
            volume_level_setter: None,
            pitch_analyzer: None,
            pitch_data_setter: None,
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
    event_dispatcher: Option<crate::events::AudioEventDispatcher>,
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
    audioworklet_status_setter: Option<std::rc::Rc<dyn observable_data::DataSetter<crate::debug::egui::live_data_panel::AudioWorkletStatus>>>,
    // Setter for updating volume level in live data
    volume_level_setter: Option<std::rc::Rc<dyn observable_data::DataSetter<Option<crate::debug::egui::live_data_panel::VolumeLevelData>>>>,
    // Pitch analyzer for direct audio processing
    pitch_analyzer: Option<std::rc::Rc<std::cell::RefCell<crate::audio::pitch_analyzer::PitchAnalyzer>>>,
    // Setter for updating pitch data in live data
    pitch_data_setter: Option<std::rc::Rc<dyn observable_data::DataSetter<Option<crate::debug::egui::live_data_panel::PitchData>>>>,
}

impl AudioWorkletManager {
    /// Create new AudioWorklet manager
    pub fn new() -> Self {
        Self {
            worklet_node: None,
            state: AudioWorkletState::Uninitialized,
            config: AudioWorkletConfig::default(),
            event_dispatcher: None,
            volume_detector: None,
            last_volume_analysis: None,
            test_signal_generator: None,
            background_noise_config: BackgroundNoiseConfig::default(),
            chunk_counter: 0,
            _message_closure: None,
            audio_context: None,
            output_to_speakers: false,
            audioworklet_status_setter: None,
            volume_level_setter: None,
            pitch_analyzer: None,
            pitch_data_setter: None,
        }
    }
    
    /// Create new AudioWorklet manager with custom configuration
    pub fn with_config(config: AudioWorkletConfig) -> Self {
        Self {
            worklet_node: None,
            state: AudioWorkletState::Uninitialized,
            config,
            event_dispatcher: None,
            volume_detector: None,
            last_volume_analysis: None,
            test_signal_generator: None,
            background_noise_config: BackgroundNoiseConfig::default(),
            chunk_counter: 0,
            _message_closure: None,
            audio_context: None,
            output_to_speakers: false,
            audioworklet_status_setter: None,
            volume_level_setter: None,
            pitch_analyzer: None,
            pitch_data_setter: None,
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
            
            let status = crate::debug::egui::live_data_panel::AudioWorkletStatus {
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
        module_future.await.map_err(|e| AudioError::StreamInitFailed(
            format!("AudioWorklet module loading failed: {:?}", e)
        ))?;
        
        dev_log!("✓ AudioWorklet processor module loaded successfully");
        
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
        let node = AudioWorkletNode::new_with_options(context, "pitch-processor", options)?;
        
        dev_log!("✓ AudioWorklet node created successfully");
        Ok(node)
    }
    
    /// Setup message handling for the AudioWorklet processor
    pub fn setup_message_handling(&mut self) -> Result<(), AudioError> {
        if let Some(worklet) = &self.worklet_node {
            // Create shared data for the message handler
            let shared_data = std::rc::Rc::new(std::cell::RefCell::new(AudioWorkletSharedData::new()));
            
            // Store references to components that will be used in the handler
            if let Some(dispatcher) = &self.event_dispatcher {
                shared_data.borrow_mut().event_dispatcher = Some(dispatcher.clone());
            }
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
            let closure = Closure::wrap(Box::new(move |event: MessageEvent| {
                Self::handle_worklet_message(event, shared_data_clone.clone());
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
        shared_data: std::rc::Rc<std::cell::RefCell<AudioWorkletSharedData>>
    ) {
        let data = event.data();
        
        // Add debug log for the first few messages
        let chunks_processed = shared_data.borrow().chunks_processed;
        if chunks_processed <= 5 {
            dev_log!("DEBUG: Received AudioWorklet message #{}", chunks_processed);
        }
        
        // Removed noisy debug logging
        
        // Parse message type from JavaScript
        if let Ok(obj) = data.dyn_into::<js_sys::Object>() {
            if let Ok(type_val) = js_sys::Reflect::get(&obj, &"type".into()) {
                if let Some(msg_type) = type_val.as_string() {
                    match msg_type.as_str() {
                        "processorReady" => {
                            // Extract batch configuration if available
                            let batch_size = js_sys::Reflect::get(&obj, &"batchSize".into())
                                .ok()
                                .and_then(|v| v.as_f64())
                                .map(|v| v as usize);
                            
                            let sample_rate = js_sys::Reflect::get(&obj, &"sampleRate".into())
                                .ok()
                                .and_then(|v| v.as_f64());
                            
                            if let Some(size) = batch_size {
                                dev_log!("✓ AudioWorklet processor ready with batch size: {} samples", size);
                            } else {
                                dev_log!("✓ AudioWorklet processor ready");
                            }
                            
                            if let Some(rate) = sample_rate {
                                dev_log!("  Sample rate: {} Hz", rate);
                            }
                            
                            Self::publish_status_update(&shared_data, AudioWorkletState::Ready, false);
                        }
                        "processingStarted" => {
                            dev_log!("✓ AudioWorklet processing started");
                            Self::publish_status_update(&shared_data, AudioWorkletState::Processing, true);
                        }
                        "processingStopped" => {
                            dev_log!("✓ AudioWorklet processing stopped");
                            Self::publish_status_update(&shared_data, AudioWorkletState::Stopped, false);
                        }
                        "audioData" => {
                            // Process real audio data (legacy single chunk)
                            Self::handle_audio_data(&obj, &shared_data);
                        }
                        "audioDataBatch" => {
                            // Process batched audio data with transferable buffer
                            Self::handle_audio_data_batch(&obj, &shared_data);
                        }
                        "processingError" => {
                            if let Ok(error_val) = js_sys::Reflect::get(&obj, &"error".into()) {
                                if let Some(error_msg) = error_val.as_string() {
                                    dev_log!("🎵 AUDIO_DEBUG: ✗ AudioWorklet processing error: {}", error_msg);
                                    Self::publish_status_update(&shared_data, AudioWorkletState::Failed, false);
                                }
                            }
                        }
                        _ => {
                            dev_log!("Unknown AudioWorklet message type: {}", msg_type);
                        }
                    }
                }
            }
        }
    }
    
    /// Handle audio data from the AudioWorklet processor
    fn handle_audio_data(
        obj: &js_sys::Object, 
        shared_data: &std::rc::Rc<std::cell::RefCell<AudioWorkletSharedData>>
    ) {
        // Extract audio samples from the message
        if let Ok(samples_val) = js_sys::Reflect::get(obj, &"samples".into()) {
            if let Ok(samples_array) = samples_val.dyn_into::<js_sys::Float32Array>() {
                // Convert JS Float32Array to Rust Vec<f32>
                let samples: Vec<f32> = samples_array.to_vec();
                
                // Get timestamp if available
                let timestamp = if let Ok(timestamp_val) = js_sys::Reflect::get(obj, &"timestamp".into()) {
                    timestamp_val.as_f64()
                } else {
                    None
                };
                
                // Update chunk counter (buffer pool operations removed)
                {
                    let mut data = shared_data.borrow_mut();
                    data.chunks_processed += 1;
                    
                    // Publish status update every 16 chunks (~11.6ms at 48kHz)
                    if data.chunks_processed % 16 == 0 {
                        drop(data); // Release borrow before calling publish
                        Self::publish_status_update(shared_data, AudioWorkletState::Processing, true);
                    }
                }
                
                // Note: Buffer pool operations removed - using direct processing with transferable buffers
                
                // Perform volume detection if available
                let volume_detector = shared_data.borrow().volume_detector.clone();
                if let Some(mut detector) = volume_detector {
                    let volume_analysis = detector.process_buffer(&samples, timestamp.unwrap_or(0.0));
                    // Volume detected
                    
                    
                    // Update volume level data every 16 chunks (~34ms at 48kHz)
                    let chunks_processed = shared_data.borrow().chunks_processed;
                    if chunks_processed % 16 == 0 {  // Update every 16 chunks (~34ms at 48kHz)
                        let volume_level_setter = shared_data.borrow().volume_level_setter.clone();
                        if let Some(setter) = volume_level_setter {
                            let volume_data = crate::debug::egui::live_data_panel::VolumeLevelData {
                                rms_db: volume_analysis.rms_db,
                                peak_db: volume_analysis.peak_db,
                                peak_fast_db: volume_analysis.peak_fast_db,
                                peak_slow_db: volume_analysis.peak_slow_db,
                                level: volume_analysis.level,
                                confidence_weight: volume_analysis.confidence_weight,
                                timestamp: timestamp.unwrap_or(0.0),
                            };
                            setter.set(Some(volume_data));
                            // Volume data updated successfully
                        }
                    }
                    
                    // Update stored analysis (store the detector back and update analysis)
                    {
                        let mut data = shared_data.borrow_mut();
                        data.volume_detector = Some(detector);
                        data.last_volume_analysis = Some(volume_analysis);
                    }
                }
            }
        }
    }
    
    /// Handle batched audio data from the AudioWorklet processor
    fn handle_audio_data_batch(
        obj: &js_sys::Object, 
        shared_data: &std::rc::Rc<std::cell::RefCell<AudioWorkletSharedData>>
    ) {
        let chunks_processed = shared_data.borrow().chunks_processed;
        if chunks_processed <= 5 {
            dev_log!("DEBUG: Starting handle_audio_data_batch for chunk #{}", chunks_processed);
        }
        
        // Extract the transferred ArrayBuffer
        if let Ok(buffer_val) = js_sys::Reflect::get(obj, &"buffer".into()) {
            if chunks_processed <= 5 {
                dev_log!("DEBUG: Buffer value extracted for chunk #{}", chunks_processed);
            }
            if let Ok(array_buffer) = buffer_val.dyn_into::<js_sys::ArrayBuffer>() {
                if chunks_processed <= 5 {
                    dev_log!("DEBUG: ArrayBuffer cast successful for chunk #{}", chunks_processed);
                }
                // Extract metadata
                let sample_count = js_sys::Reflect::get(obj, &"sampleCount".into())
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as usize;
                
                let timestamp = js_sys::Reflect::get(obj, &"timestamp".into())
                    .ok()
                    .and_then(|v| v.as_f64());
                
                // Create Float32Array view of the buffer
                let samples_array = js_sys::Float32Array::new(&array_buffer);
                
                // Only process the actual samples written (not the full buffer size)
                let valid_samples = if sample_count > 0 && sample_count <= samples_array.length() as usize {
                    samples_array.slice(0, sample_count as u32)
                } else {
                    samples_array
                };
                
                // Convert to Vec<f32> for processing
                let samples: Vec<f32> = valid_samples.to_vec();
                
                if chunks_processed <= 5 {
                    dev_log!("DEBUG: Converted to {} samples for processing", samples.len());
                }
                
                // Note: The ArrayBuffer is already detached after transfer, so no explicit
                // recycling is needed on the main thread. The AudioWorklet manages its own
                // buffer pool and creates new buffers as needed.
                
                // Update chunks processed count (batched)
                let chunk_count = (samples.len() + 127) / 128; // Round up to nearest chunk
                {
                    let mut data = shared_data.borrow_mut();
                    data.chunks_processed += chunk_count as u32;
                }
                
                if chunks_processed <= 5 {
                    dev_log!("DEBUG: Updated chunks_processed to {}", chunk_count);
                }
                
                // Process volume detection on the batch
                let volume_detector = shared_data.borrow().volume_detector.clone();
                if let Some(mut detector) = volume_detector {
                    let volume_analysis = detector.process_buffer(&samples, timestamp.unwrap_or(0.0));
                    
                    // Update volume level data on every batch
                    {
                        let volume_level_setter = shared_data.borrow().volume_level_setter.clone();
                        if let Some(setter) = volume_level_setter {
                            let volume_data = crate::debug::egui::live_data_panel::VolumeLevelData {
                                rms_db: volume_analysis.rms_db,
                                peak_db: volume_analysis.peak_db,
                                peak_fast_db: volume_analysis.peak_fast_db,
                                peak_slow_db: volume_analysis.peak_slow_db,
                                level: volume_analysis.level,
                                confidence_weight: volume_analysis.confidence_weight,
                                timestamp: timestamp.unwrap_or(0.0),
                            };
                            setter.set(Some(volume_data));
                        }
                    }
                    
                    // Store back the detector and analysis
                    let mut data = shared_data.borrow_mut();
                    data.volume_detector = Some(detector);
                    data.last_volume_analysis = Some(volume_analysis);
                }
                
                // Direct pitch analysis on batched data (Task 4)
                let pitch_analyzer = shared_data.borrow().pitch_analyzer.clone();
                
                if let Some(analyzer) = pitch_analyzer {
                    if let Ok(mut analyzer_mut) = analyzer.try_borrow_mut() {
                        match analyzer_mut.analyze_batch_direct(&samples) {
                            Ok(pitch_results) => {
                                
                                // Log pitch detection results (always log when pitch is found)
                                if !pitch_results.is_empty() {
                                    dev_log!("✓ PITCH DETECTED: {} results", pitch_results.len());
                                    for (i, result) in pitch_results.iter().enumerate() {
                                        dev_log!("  Pitch {}: freq={:.1}Hz, confidence={:.2}", 
                                            i, result.frequency, result.confidence);
                                    }
                                }
                                
                                // Update pitch data in UI if setter is available
                                let pitch_data_setter = shared_data.borrow().pitch_data_setter.clone();
                                if let Some(setter) = pitch_data_setter {
                                    if let Some(best_result) = pitch_results.first() {
                                        // Convert frequency to musical note using the analyzer's note mapper
                                        let note = analyzer_mut.frequency_to_note(best_result.frequency);
                                        let pitch_data = crate::debug::egui::live_data_panel::PitchData {
                                            frequency: best_result.frequency,
                                            confidence: best_result.confidence,
                                            note: note.clone(),
                                            clarity: best_result.clarity,
                                            timestamp: best_result.timestamp,
                                        };
                                        setter.set(Some(pitch_data));
                                        dev_log!("✓ Pitch data sent to UI: {:.1}Hz {}", best_result.frequency, note);
                                    } else {
                                        // No pitch detected, clear the data
                                        setter.set(None);
                                    }
                                }
                                
                                // Update volume confidence weighting if available
                                if let Some(volume_analysis) = shared_data.borrow().last_volume_analysis.as_ref() {
                                    analyzer_mut.update_volume_analysis(volume_analysis.clone());
                                }
                            }
                            Err(e) => {
                                // Log errors occasionally to avoid spam
                                let current_chunks_processed = shared_data.borrow().chunks_processed;
                                if current_chunks_processed <= 24 {
                                    dev_log!("Pitch detection error #{}: {}", current_chunks_processed, e);
                                }
                            }
                        }
                    }
                } else {
                    // Log warning occasionally if pitch analyzer is not available
                    let current_chunks_processed = shared_data.borrow().chunks_processed;
                    if current_chunks_processed % 256 == 0 {
                        dev_log!("Warning: No pitch analyzer available for batched processing");
                    }
                }
                
                // Removed noisy batch processing debug logging
                // Keep only essential debug logging for pitch detection
                let final_chunks_processed = shared_data.borrow().chunks_processed;
                if final_chunks_processed <= 40 {  // Increased limit since chunks_processed is higher now
                    dev_log!("DEBUG: Audio batch #{}: {} samples", final_chunks_processed, samples.len());
                }
            } else {
                if chunks_processed <= 5 {
                    dev_log!("DEBUG: ArrayBuffer cast failed for chunk #{}", chunks_processed);
                }
            }
        } else {
            if chunks_processed <= 5 {
                dev_log!("DEBUG: Buffer field extraction failed for chunk #{}", chunks_processed);
            }
        }
    }
    
    /// Publish AudioWorklet status update to Live Data Panel
    fn publish_status_update(
        shared_data: &std::rc::Rc<std::cell::RefCell<AudioWorkletSharedData>>,
        state: AudioWorkletState,
        _processing: bool
    ) {
        if let Some(_dispatcher) = &shared_data.borrow().event_dispatcher {
            let chunks_processed = shared_data.borrow().chunks_processed;
            
            // Create status update for Live Data Panel
            #[cfg(target_arch = "wasm32")]
            let timestamp = js_sys::Date::now();
            #[cfg(not(target_arch = "wasm32"))]
            let timestamp = 0.0;
            
            let _status = crate::debug::egui::live_data_panel::AudioWorkletStatus {
                state,
                processor_loaded: true, // If we're getting messages, processor is loaded
                chunk_size: 128, // Web Audio API standard
                chunks_processed,
                last_update: timestamp,
            };
            
            // TODO: Update AudioWorklet status via setter instead of events
        }
    }
    
    /// Send control message to AudioWorklet processor
    pub fn send_control_message(&self, message_type: &str) -> Result<(), AudioError> {
        if let Some(worklet) = &self.worklet_node {
            let message = js_sys::Object::new();
            js_sys::Reflect::set(&message, &"type".into(), &message_type.into())
                .map_err(|e| AudioError::Generic(format!("Failed to create message: {:?}", e)))?;
            
            let port = worklet.port()
                .map_err(|e| AudioError::Generic(format!("Failed to get AudioWorklet port: {:?}", e)))?;
            port.post_message(&message)
                .map_err(|e| AudioError::Generic(format!("Failed to send message: {:?}", e)))?;
            
            dev_log!("Sent control message to AudioWorklet: {}", message_type);
            Ok(())
        } else {
            Err(AudioError::Generic("No AudioWorklet node available".to_string()))
        }
    }

    /// Send test signal configuration to AudioWorklet processor
    fn send_test_signal_config_to_worklet(&self, config: &TestSignalGeneratorConfig) -> Result<(), AudioError> {
        if let Some(worklet) = &self.worklet_node {
            // Create the main message object
            let message = js_sys::Object::new();
            js_sys::Reflect::set(&message, &"type".into(), &"updateTestSignalConfig".into())
                .map_err(|e| AudioError::Generic(format!("Failed to set message type: {:?}", e)))?;
            
            // Create the config object
            let config_obj = js_sys::Object::new();
            js_sys::Reflect::set(&config_obj, &"enabled".into(), &config.enabled.into())
                .map_err(|e| AudioError::Generic(format!("Failed to set enabled: {:?}", e)))?;
            js_sys::Reflect::set(&config_obj, &"frequency".into(), &config.frequency.into())
                .map_err(|e| AudioError::Generic(format!("Failed to set frequency: {:?}", e)))?;
            js_sys::Reflect::set(&config_obj, &"amplitude".into(), &config.amplitude.into())
                .map_err(|e| AudioError::Generic(format!("Failed to set amplitude: {:?}", e)))?;
            js_sys::Reflect::set(&config_obj, &"sample_rate".into(), &config.sample_rate.into())
                .map_err(|e| AudioError::Generic(format!("Failed to set sample_rate: {:?}", e)))?;
            
            // Convert waveform enum to string
            let waveform_str = match config.waveform {
                TestWaveform::Sine => "sine",
                TestWaveform::Square => "square",
                TestWaveform::Sawtooth => "sawtooth",
                TestWaveform::Triangle => "triangle",
                TestWaveform::WhiteNoise => "white_noise",
                TestWaveform::PinkNoise => "pink_noise",
            };
            js_sys::Reflect::set(&config_obj, &"waveform".into(), &waveform_str.into())
                .map_err(|e| AudioError::Generic(format!("Failed to set waveform: {:?}", e)))?;
            
            // Attach config to message
            js_sys::Reflect::set(&message, &"config".into(), &config_obj)
                .map_err(|e| AudioError::Generic(format!("Failed to set config: {:?}", e)))?;
            
            // Send message
            let port = worklet.port()
                .map_err(|e| AudioError::Generic(format!("Failed to get AudioWorklet port: {:?}", e)))?;
            port.post_message(&message)
                .map_err(|e| AudioError::Generic(format!("Failed to send test signal config: {:?}", e)))?;
            
            dev_log!("Sent test signal config to AudioWorklet: enabled={}, freq={:.1}Hz, amp={:.2}", 
                     config.enabled, config.frequency, config.amplitude);
            Ok(())
        } else {
            Err(AudioError::Generic("No AudioWorklet node available".to_string()))
        }
    }
    
    
    /// Update batch configuration
    pub fn update_batch_config(&self, batch_size: Option<usize>, buffer_timeout: Option<f64>) -> Result<(), AudioError> {
        if let Some(worklet) = &self.worklet_node {
            let message = js_sys::Object::new();
            js_sys::Reflect::set(&message, &"type".into(), &"updateBatchConfig".into())
                .map_err(|e| AudioError::Generic(format!("Failed to create message: {:?}", e)))?;
            
            // Create config object
            let config = js_sys::Object::new();
            
            if let Some(size) = batch_size {
                js_sys::Reflect::set(&config, &"batchSize".into(), &JsValue::from(size as f64))
                    .map_err(|e| AudioError::Generic(format!("Failed to set batchSize: {:?}", e)))?;
            }
            
            if let Some(timeout) = buffer_timeout {
                js_sys::Reflect::set(&config, &"bufferTimeout".into(), &JsValue::from(timeout))
                    .map_err(|e| AudioError::Generic(format!("Failed to set bufferTimeout: {:?}", e)))?;
            }
            
            // Attach config to message
            js_sys::Reflect::set(&message, &"config".into(), &config)
                .map_err(|e| AudioError::Generic(format!("Failed to set config: {:?}", e)))?;
            
            // Send message
            let port = worklet.port()
                .map_err(|e| AudioError::Generic(format!("Failed to get AudioWorklet port: {:?}", e)))?;
            port.post_message(&message)
                .map_err(|e| AudioError::Generic(format!("Failed to send batch config: {:?}", e)))?;
            
            dev_log!("Sent batch config to AudioWorklet: batch_size={:?}, timeout={:?}ms", 
                     batch_size, buffer_timeout);
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
        self.send_control_message("startProcessing")?;
        
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
        self.send_control_message("stopProcessing")?;
        
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
    
    /// Check if audio processing is active
    pub fn is_processing(&self) -> bool {
        matches!(self.state, AudioWorkletState::Processing)
    }
    
    /// Get chunk size for processing
    pub fn chunk_size(&self) -> u32 {
        self.config.chunk_size
    }

    // Note: Buffer pool support removed - using direct processing with transferable buffers

    /// Attach an event dispatcher for publishing BufferEvents
    pub fn set_event_dispatcher(&mut self, dispatcher: crate::events::AudioEventDispatcher) {
        self.event_dispatcher = Some(dispatcher);
    }
    
    /// Set the AudioWorklet status setter for live data updates
    pub fn set_audioworklet_status_setter(&mut self, setter: std::rc::Rc<dyn observable_data::DataSetter<crate::debug::egui::live_data_panel::AudioWorkletStatus>>) {
        self.audioworklet_status_setter = Some(setter);
    }
    
    /// Set the volume level setter for live data updates
    pub fn set_volume_level_setter(&mut self, setter: std::rc::Rc<dyn observable_data::DataSetter<Option<crate::debug::egui::live_data_panel::VolumeLevelData>>>) {
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
    pub fn set_pitch_data_setter(&mut self, setter: std::rc::Rc<dyn observable_data::DataSetter<Option<crate::debug::egui::live_data_panel::PitchData>>>) {
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
            // Create the main message object
            let message = js_sys::Object::new();
            js_sys::Reflect::set(&message, &"type".into(), &"updateBackgroundNoiseConfig".into())
                .map_err(|e| AudioError::Generic(format!("Failed to set message type: {:?}", e)))?;
            
            // Create the config object
            let config_obj = js_sys::Object::new();
            js_sys::Reflect::set(&config_obj, &"enabled".into(), &config.enabled.into())
                .map_err(|e| AudioError::Generic(format!("Failed to set enabled: {:?}", e)))?;
            js_sys::Reflect::set(&config_obj, &"level".into(), &config.level.into())
                .map_err(|e| AudioError::Generic(format!("Failed to set level: {:?}", e)))?;
            
            // Convert noise type enum to string
            let noise_type_str = match config.noise_type {
                TestWaveform::WhiteNoise => "white_noise",
                TestWaveform::PinkNoise => "pink_noise",
                _ => "white_noise", // Default to white noise for non-noise waveforms
            };
            js_sys::Reflect::set(&config_obj, &"type".into(), &noise_type_str.into())
                .map_err(|e| AudioError::Generic(format!("Failed to set noise type: {:?}", e)))?;
            
            // Attach config to message
            js_sys::Reflect::set(&message, &"config".into(), &config_obj)
                .map_err(|e| AudioError::Generic(format!("Failed to set config: {:?}", e)))?;
            
            // Send message to AudioWorklet
            let port = worklet.port()
                .map_err(|e| AudioError::Generic(format!("Failed to get worklet port: {:?}", e)))?;
            port.post_message(&message)
                .map_err(|e| AudioError::Generic(format!("Failed to send background noise config: {:?}", e)))?;
            
            dev_log!("Background noise configuration sent to AudioWorklet: enabled={}, level={}, type={:?}", 
                    config.enabled, config.level, config.noise_type);
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
                dev_log!("First volume analysis: {:.1}dB", volume_analysis.peak_db);
            }
            

            // Update volume level data every 16 chunks (~11.6ms at 48kHz)
            self.chunk_counter += 1;
            if self.chunk_counter % 16 == 0 {
                // Also update AudioWorklet status periodically
                self.publish_audioworklet_status();
                
                // Update volume level using setter
                if let Some(ref setter) = self.volume_level_setter {
                    let volume_data = crate::debug::egui::live_data_panel::VolumeLevelData {
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
                        dev_log!("Volume data updated via process_audio: {:.1}dB", volume_analysis.peak_db);
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
        use event_dispatcher::create_shared_dispatcher;

        // Create manager with volume detector
        let mut mgr = AudioWorkletManager::new();
        mgr.set_event_dispatcher(create_shared_dispatcher());
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
        use event_dispatcher::create_shared_dispatcher;

        // Create manager with volume detector
        let mut mgr = AudioWorkletManager::new();
        mgr.set_event_dispatcher(create_shared_dispatcher());
        
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

/// Initialize AudioWorklet manager with buffer pool and event dispatcher integration
pub async fn initialize_audioworklet_manager() -> Result<(), String> {
    use crate::common::dev_log;
    
    dev_log!("Initializing AudioWorklet manager");
    
    // Get audio context manager
    let audio_context_manager = super::get_audio_context_manager()
        .ok_or_else(|| "AudioContext manager not initialized".to_string())?;
    
    // Create AudioWorklet manager
    let mut worklet_manager = AudioWorkletManager::new();
    
    // Get event dispatcher
    let event_dispatcher = crate::events::get_global_event_dispatcher();
    
    // Configure AudioWorklet manager
    worklet_manager.set_event_dispatcher(event_dispatcher.clone());
    
    // Add volume detector for real-time volume analysis
    let volume_detector = super::VolumeDetector::new_default();
    worklet_manager.set_volume_detector(volume_detector);
    
    // Note: Initial status will be published automatically by the manager
    
    // Store globally before initialization for setter access
    let worklet_manager_rc = std::rc::Rc::new(std::cell::RefCell::new(worklet_manager));
    super::set_global_audioworklet_manager(worklet_manager_rc.clone());
    
    // Setup pitch analyzer for direct processing if available
    if let Some(pitch_analyzer) = super::get_global_pitch_analyzer() {
        worklet_manager_rc.borrow_mut().set_pitch_analyzer(pitch_analyzer);
        dev_log!("✓ Pitch analyzer connected to AudioWorklet manager");
    } else {
        dev_log!("Note: No pitch analyzer available during AudioWorklet initialization");
    }
    
    // Initialize AudioWorklet
    let audio_context_ref = audio_context_manager.borrow();
    let init_result = worklet_manager_rc.borrow_mut().initialize(&*audio_context_ref).await;
    
    match init_result {
        Ok(_) => {
            dev_log!("✓ AudioWorklet processor loaded and ready");
            
            
            // Note: Status is published automatically by the manager
            
            // Note: We don't connect AudioWorklet to destination to avoid audio feedback
            // The AudioWorklet will still process audio when microphone is connected to it
            
            // Start audio processing automatically
            match worklet_manager_rc.borrow_mut().start_processing() {
                Ok(_) => {
                    dev_log!("✓ Audio processing started automatically");
                    
                    // Note: Status is published automatically by the manager
                }
                Err(e) => {
                    dev_log!("✗ Failed to start audio processing: {:?}", e);
                    
                    // Manager is already stored globally
                }
            }
            
            // Manager already stored globally
            
            Ok(())
        }
        Err(e) => {
            dev_log!("✗ AudioWorklet initialization failed: {:?}", e);
            
            // Note: Failed status is published automatically by the manager
            
            Err(format!("Failed to initialize AudioWorklet: {:?}", e))
        }
    }
}