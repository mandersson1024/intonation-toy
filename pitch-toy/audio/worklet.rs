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
    AudioNode
};
use js_sys;
use std::fmt;
use crate::common::dev_log;
use super::{AudioError, context::AudioContextManager, VolumeDetector, VolumeDetectorConfig, VolumeAnalysis};

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
    buffer_pool: Option<std::rc::Rc<std::cell::RefCell<crate::audio::buffer_pool::BufferPool<f32>>>>,
    event_dispatcher: Option<crate::events::AudioEventDispatcher>,
    volume_detector: Option<VolumeDetector>,
    last_volume_analysis: Option<VolumeAnalysis>,
    chunk_counter: u32,
}

impl AudioWorkletManager {
    /// Create new AudioWorklet manager
    pub fn new() -> Self {
        Self {
            worklet_node: None,
            state: AudioWorkletState::Uninitialized,
            config: AudioWorkletConfig::default(),
            buffer_pool: None,
            event_dispatcher: None,
            volume_detector: None,
            last_volume_analysis: None,
            chunk_counter: 0,
        }
    }
    
    /// Create new AudioWorklet manager with custom configuration
    pub fn with_config(config: AudioWorkletConfig) -> Self {
        Self {
            worklet_node: None,
            state: AudioWorkletState::Uninitialized,
            config,
            buffer_pool: None,
            event_dispatcher: None,
            volume_detector: None,
            last_volume_analysis: None,
            chunk_counter: 0,
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
        
        self.state = AudioWorkletState::Initializing;
        dev_log!("Initializing AudioWorklet processor");
        
        // Try AudioWorklet first
        if Self::is_worklet_supported(context) {
            match self.initialize_worklet(audio_context).await {
                Ok(()) => {
                    dev_log!("✓ AudioWorklet initialized successfully");
                    self.state = AudioWorkletState::Ready;
                    return Ok(());
                }
                Err(e) => {
                    dev_log!("✗ AudioWorklet initialization failed: {:?}", e);
                    self.state = AudioWorkletState::Failed;
                    return Err(e);
                }
            }
        }
        
        // AudioWorklet required
        self.state = AudioWorkletState::Failed;
        Err(AudioError::NotSupported(
            "AudioWorklet not supported".to_string()
        ))
    }
    
    /// Initialize AudioWorklet processor
    async fn initialize_worklet(&mut self, context: &AudioContext) -> Result<(), AudioError> {
        // TODO: In a real implementation, you would load the AudioWorklet processor script here
        // For now, we'll create a simple pass-through processor
        
        // Create AudioWorklet node with options
        let options = AudioWorkletNodeOptions::new();
        options.set_number_of_inputs(1);
        options.set_number_of_outputs(1);
        options.set_output_channel_count(&js_sys::Array::of1(&js_sys::Number::from(
            self.config.output_channels
        )));
        
        // Note: In production, you would call addModule first to load the processor
        // context.audio_worklet().add_module("/audio-processor.js").await?;
        
        // For now, we'll create a placeholder node structure
        // This would be replaced with actual AudioWorkletNode creation after module loading
        match self.create_worklet_node_placeholder(context) {
            Ok(node) => {
                self.worklet_node = Some(node);
                dev_log!("AudioWorklet node created with {} input channels, {} output channels", 
                        self.config.input_channels, self.config.output_channels);
                Ok(())
            }
            Err(e) => {
                Err(AudioError::StreamInitFailed(
                    format!("Failed to create AudioWorklet node: {:?}", e)
                ))
            }
        }
    }
    
    /// Create AudioWorklet node placeholder (TODO: Replace with actual implementation)
    fn create_worklet_node_placeholder(&self, _context: &AudioContext) -> Result<AudioWorkletNode, js_sys::Error> {
        // TODO: Replace this placeholder with actual AudioWorkletNode creation
        // This is a stub implementation that would be replaced when the AudioWorklet
        // processor script is implemented
        
        // For now, we'll attempt to create a basic node structure
        // In production, this would be:
        // AudioWorkletNode::new_with_options(context, "pitch-processor", &options)
        
        // Since we can't create a real AudioWorkletNode without a registered processor,
        // we'll return an error for now
        Err(js_sys::Error::new("AudioWorklet processor not yet implemented"))
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
        
        self.state = AudioWorkletState::Stopped;
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

    /// Attach a shared buffer pool for real-time audio filling
    pub fn set_buffer_pool(&mut self, pool: std::rc::Rc<std::cell::RefCell<crate::audio::buffer_pool::BufferPool<f32>>>) {
        self.buffer_pool = Some(pool);
    }

    /// Attach an event dispatcher for publishing BufferEvents
    pub fn set_event_dispatcher(&mut self, dispatcher: crate::events::AudioEventDispatcher) {
        self.event_dispatcher = Some(dispatcher);
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
            let volume_analysis = detector.process_buffer(samples, timestamp);
            
            // Check for volume change events
            if let Some(previous) = &self.last_volume_analysis {
                let rms_change = (volume_analysis.rms_db - previous.rms_db).abs();
                
                // Publish volume change event if significant change (>3dB)
                if rms_change > 3.0 {
                    if let Some(dispatcher) = &self.event_dispatcher {
                        dispatcher.borrow().publish(crate::events::audio_events::AudioEvent::VolumeChanged {
                            previous_rms_db: previous.rms_db,
                            current_rms_db: volume_analysis.rms_db,
                            change_db: volume_analysis.rms_db - previous.rms_db,
                            timestamp,
                        });
                    }
                }

                // Check for volume warnings (problematic levels)
                let current_problematic = matches!(volume_analysis.level, 
                    super::VolumeLevel::Silent | super::VolumeLevel::Clipping);
                let previous_problematic = matches!(previous.level, 
                    super::VolumeLevel::Silent | super::VolumeLevel::Clipping);
                
                if current_problematic && !previous_problematic {
                    if let Some(dispatcher) = &self.event_dispatcher {
                        let message = match volume_analysis.level {
                            super::VolumeLevel::Silent => "Input level too low".to_string(),
                            super::VolumeLevel::Clipping => "Input level clipping".to_string(),
                            _ => "Volume level warning".to_string(),
                        };
                        
                        dispatcher.borrow().publish(crate::events::audio_events::AudioEvent::VolumeWarning {
                            level: volume_analysis.level,
                            rms_db: volume_analysis.rms_db,
                            message,
                            timestamp,
                        });
                    }
                }
            }

            // Publish volume detected event every 16 chunks (~11.6ms at 48kHz)
            self.chunk_counter += 1;
            if self.chunk_counter % 16 == 0 {
                if let Some(dispatcher) = &self.event_dispatcher {
                    dispatcher.borrow().publish(crate::events::audio_events::AudioEvent::VolumeDetected {
                        rms_db: volume_analysis.rms_db,
                        peak_db: volume_analysis.peak_db,
                        peak_fast_db: volume_analysis.peak_fast_db,
                        peak_slow_db: volume_analysis.peak_slow_db,
                        level: volume_analysis.level,
                        confidence_weight: volume_analysis.confidence_weight,
                        timestamp,
                    });
                }
            }

            // Store the current analysis for next comparison
            self.last_volume_analysis = Some(volume_analysis);
        }

        // Buffer management
        let pool_rc = self.buffer_pool.as_ref().ok_or("No buffer pool attached")?.clone();
        let mut pool = pool_rc.borrow_mut();
        let buffer = pool.get_mut(0).ok_or("BufferPool is empty")?;

        buffer.write_chunk(samples);

        // Publish buffer events if dispatcher present
        if let Some(dispatcher) = &self.event_dispatcher {
            if buffer.is_full() {
                dispatcher.borrow().publish(crate::events::audio_events::AudioEvent::BufferFilled {
                    buffer_index: 0,
                    length: buffer.len(),
                });
            }

            if buffer.has_overflowed() {
                dispatcher.borrow().publish(crate::events::audio_events::AudioEvent::BufferOverflow {
                    buffer_index: 0,
                    overflow_count: buffer.overflow_count(),
                });
            }

            // Periodically publish metrics (every 256 chunks for example) – simple heuristic here
            if buffer.len() % 256 == 0 {
                dispatcher.borrow().publish(crate::events::audio_events::AudioEvent::BufferMetrics {
                    total_buffers: pool.len(),
                    total_overflows: pool.total_overflows(),
                    memory_bytes: pool.memory_usage_bytes(),
                });
            }
        }

        Ok(())
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

    #[test]
    fn test_audio_worklet_state_display() {
        assert_eq!(AudioWorkletState::Uninitialized.to_string(), "Uninitialized");
        assert_eq!(AudioWorkletState::Initializing.to_string(), "Initializing");
        assert_eq!(AudioWorkletState::Ready.to_string(), "Ready");
        assert_eq!(AudioWorkletState::Processing.to_string(), "Processing");
        assert_eq!(AudioWorkletState::Stopped.to_string(), "Stopped");
        assert_eq!(AudioWorkletState::Failed.to_string(), "Failed");
    }

    #[test]
    fn test_audio_worklet_config_default() {
        let config = AudioWorkletConfig::default();
        assert_eq!(config.chunk_size, 128);
        assert_eq!(config.input_channels, 1);
        assert_eq!(config.output_channels, 1);
    }

    #[test]
    fn test_audio_worklet_config_builders() {
        let stereo_config = AudioWorkletConfig::stereo();
        assert_eq!(stereo_config.input_channels, 2);
        assert_eq!(stereo_config.output_channels, 2);
        
        let custom_config = AudioWorkletConfig::with_channels(4, 2);
        assert_eq!(custom_config.input_channels, 4);
        assert_eq!(custom_config.output_channels, 2);
        
    }

    #[test]
    fn test_audio_worklet_manager_new() {
        let manager = AudioWorkletManager::new();
        assert_eq!(*manager.state(), AudioWorkletState::Uninitialized);
        assert!(!manager.is_processing());
        assert_eq!(manager.chunk_size(), 128);
        assert!(manager.get_processing_node().is_none());
    }

    #[test]
    fn test_audio_worklet_manager_with_config() {
        let config = AudioWorkletConfig::stereo();
        let manager = AudioWorkletManager::with_config(config.clone());
        
        assert_eq!(*manager.state(), AudioWorkletState::Uninitialized);
        assert_eq!(manager.config().input_channels, 2);
        assert_eq!(manager.config().output_channels, 2);
    }

    #[test]
    fn test_audio_worklet_manager_state_transitions() {
        let mut manager = AudioWorkletManager::new();
        
        // Cannot start processing from uninitialized state
        assert!(manager.start_processing().is_err());
        
        // Manually set state for testing (avoiding web-sys calls)
        manager.state = AudioWorkletState::Ready;
        
        // Test state transitions without web-sys dependencies
        assert!(manager.start_processing().is_ok());
        assert_eq!(*manager.state(), AudioWorkletState::Processing);
        assert!(manager.is_processing());
        
        // Can stop from processing state
        assert!(manager.stop_processing().is_ok());
        assert_eq!(*manager.state(), AudioWorkletState::Stopped);
        assert!(!manager.is_processing());
    }

    #[test]
    fn test_audio_worklet_manager_disconnect() {
        let mut manager = AudioWorkletManager::new();
        manager.state = AudioWorkletState::Ready;
        
        assert!(manager.disconnect().is_ok());
        assert_eq!(*manager.state(), AudioWorkletState::Uninitialized);
    }

    #[test]
    fn test_feed_input_chunk_and_events() {
        use crate::audio::{BufferPool, VolumeDetector};
        use crate::events::event_dispatcher::create_shared_dispatcher;
        use std::rc::Rc;
        use std::cell::RefCell;

        // Create dispatcher and track events
        let dispatcher = create_shared_dispatcher();
        let received = Rc::new(RefCell::new(Vec::new()));
        let recv_clone = received.clone();
        dispatcher.borrow_mut().subscribe("buffer_filled", move |e| { recv_clone.borrow_mut().push(e); });

        // Create pool with one buffer 256 samples capacity
        let pool = Rc::new(RefCell::new(BufferPool::<f32>::new(1, 256).unwrap()));

        // Create manager with volume detector
        let mut mgr = AudioWorkletManager::new();
        mgr.set_buffer_pool(pool.clone());
        mgr.set_event_dispatcher(dispatcher.clone());
        mgr.set_volume_detector(VolumeDetector::new_default());

        // Feed two chunks of 128 samples each (fills buffer)
        let chunk = vec![0.1_f32; 128]; // Use small signal for volume detection
        mgr.feed_input_chunk(&chunk).unwrap();
        mgr.feed_input_chunk(&chunk).unwrap();

        // Expect buffer_filled event
        assert_eq!(received.borrow().len(), 1);
        
        // Should have volume analysis available
        assert!(mgr.last_volume_analysis().is_some());
    }

    #[test]
    fn test_volume_detection_integration() {
        use crate::audio::{BufferPool, VolumeDetector, VolumeDetectorConfig};
        use crate::events::event_dispatcher::create_shared_dispatcher;
        use std::rc::Rc;
        use std::cell::RefCell;

        // Create dispatcher and track volume events
        let dispatcher = create_shared_dispatcher();
        let volume_events = Rc::new(RefCell::new(Vec::new()));
        let vol_clone = volume_events.clone();
        dispatcher.borrow_mut().subscribe("volume_detected", move |e| { vol_clone.borrow_mut().push(e); });

        // Create pool and manager with volume detector
        let pool = Rc::new(RefCell::new(BufferPool::<f32>::new(1, 256).unwrap()));
        let mut mgr = AudioWorkletManager::new();
        mgr.set_buffer_pool(pool.clone());
        mgr.set_event_dispatcher(dispatcher.clone());
        
        let config = VolumeDetectorConfig {
            sample_rate: 48000.0,
            ..VolumeDetectorConfig::default()
        };
        mgr.set_volume_detector(VolumeDetector::new(config).unwrap());

        // Feed 16 chunks to trigger volume event publication
        let chunk = vec![0.1_f32; 128];
        for _ in 0..16 {
            mgr.feed_input_chunk(&chunk).unwrap();
        }

        // Should have published volume detected event
        assert!(volume_events.borrow().len() > 0);
        
        // Should have volume analysis available
        let analysis = mgr.last_volume_analysis().unwrap();
        assert!(analysis.rms_db.is_finite());
        assert!(analysis.confidence_weight > 0.0);
    }

    #[test]
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