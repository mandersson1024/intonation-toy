//! Stream management for audio input lifecycle and device reconnection
//! 
//! This module provides `StreamReconnectionHandler` for managing MediaStream lifecycle,
//! detecting device disconnections, and implementing automatic reconnection logic.
//!
//! # Features
//! 
//! - **Stream Lifecycle Management**: Track connection states (Disconnected, Connecting, Connected, Reconnecting, Failed)
//! - **Device Disconnection Detection**: Monitor MediaStreamTrack state for device removal
//! - **Automatic Reconnection**: Configurable retry logic with callback-based stream recreation
//! - **Stream Health Monitoring**: Real-time health tracking with activity timeouts
//! - **Error Recovery**: Comprehensive error handling and recovery strategies
//!
//! # Example Usage
//!
//! ```rust
//! # use pitch_toy::audio::stream::{StreamReconnectionHandler, StreamConfig, StreamState};
//! # #[cfg(target_arch = "wasm32")]
//! # use web_sys::MediaStream;
//! # #[cfg(target_arch = "wasm32")]
//! # use wasm_bindgen::JsValue;
//! 
//! // Create handler with custom configuration
//! let config = StreamConfig {
//!     max_reconnect_attempts: 5,
//!     reconnect_delay_ms: 2000,
//!     health_check_interval_ms: 3000,
//!     activity_timeout_ms: 15000,
//! };
//! let mut handler = StreamReconnectionHandler::new(config);
//! 
//! // Check initial state
//! let health = handler.get_health();
//! assert_eq!(health.state, StreamState::Disconnected);
//! assert!(!handler.is_connected());
//! 
//! # #[cfg(target_arch = "wasm32")]
//! # {
//! // Set up reconnection callback (WASM only)
//! handler.set_reconnect_callback(|| {
//!     // Your MediaStream creation logic here
//!     // navigator.mediaDevices.getUserMedia({ audio: true })
//!     Err(JsValue::from_str("Example error"))
//! });
//! 
//! // Manual reconnection attempt would fail in this example
//! let result = handler.reconnect();
//! assert!(result.is_err());
//! # }
//! ```
//!
//! # Stream States
//!
//! - `Disconnected`: No active stream
//! - `Connecting`: Stream connection in progress  
//! - `Connected`: Stream active and healthy
//! - `Reconnecting`: Attempting to restore connection
//! - `Failed`: Connection failed, no more retry attempts
//!
//! # Error Handling
//!
//! The handler automatically attempts reconnection for recoverable errors:
//! - `DeviceDisconnected`: Device unplugged or removed
//! - `StreamEnded`: Stream terminated unexpectedly
//! - `PermissionRevoked`: User revoked microphone permission
//!
//! Manual reconnection can be triggered with `handler.reconnect()`.
//!
//! # Browser Compatibility
//!
//! Requires Web Audio API support:
//! - Chrome 66+, Firefox 76+, Safari 14.1+, Edge 79+
//! - HTTPS context required for getUserMedia API

use wasm_bindgen::prelude::*;
use web_sys::{MediaStream, MediaStreamTrack, MediaStreamTrackState};
use std::rc::Rc;
use std::cell::RefCell;
use crate::common::dev_log;

/// Stream connection states for tracking MediaStream lifecycle
/// 
/// Represents the current state of an audio stream connection, allowing
/// applications to respond appropriately to connection changes.
#[derive(Debug, Clone, PartialEq)]
pub enum StreamState {
    /// No active stream connection
    Disconnected,
    /// Stream connection attempt in progress  
    Connecting,
    /// Stream is active and healthy
    Connected,
    /// Attempting to restore a failed connection
    Reconnecting,
    /// Connection failed permanently (max retries exceeded)
    Failed,
}

/// Stream health information containing current status and diagnostics
/// 
/// Provides detailed information about the current state of a MediaStream,
/// including connection status, error conditions, and reconnection attempts.
#[derive(Debug, Clone)]
pub struct StreamHealth {
    /// Current connection state
    pub state: StreamState,
    /// Device identifier if available
    pub device_id: Option<String>,
    /// Timestamp of last stream activity (milliseconds since epoch)
    pub last_activity: f64,
    /// Number of reconnection attempts made
    pub reconnect_attempts: u32,
    /// Most recent error message if any
    pub error_message: Option<String>,
}

/// Configuration for stream reconnection behavior and health monitoring
/// 
/// Controls how the StreamReconnectionHandler behaves when detecting
/// connection failures and attempting recovery.
/// 
/// # Example
/// 
/// ```rust
/// # use pitch_toy::audio::stream::StreamConfig;
/// let config = StreamConfig {
///     max_reconnect_attempts: 5,      // Try up to 5 times
///     reconnect_delay_ms: 2000,       // Wait 2 seconds between attempts  
///     health_check_interval_ms: 3000, // Check health every 3 seconds
///     activity_timeout_ms: 15000,     // Timeout after 15 seconds of inactivity
/// };
/// ```
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Maximum number of automatic reconnection attempts
    pub max_reconnect_attempts: u32,
    /// Delay between reconnection attempts (milliseconds)
    pub reconnect_delay_ms: u32,
    /// Interval for periodic health checks (milliseconds)
    pub health_check_interval_ms: u32,
    /// Timeout for stream inactivity detection (milliseconds)
    pub activity_timeout_ms: u32,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            max_reconnect_attempts: 3,
            reconnect_delay_ms: 1000,
            health_check_interval_ms: 5000,
            activity_timeout_ms: 10000,
        }
    }
}

/// Error types that can occur during stream management
/// 
/// Categorizes different failure modes to enable appropriate error handling
/// and recovery strategies.
#[derive(Debug, Clone)]
pub enum StreamError {
    /// Audio input device was disconnected or removed
    DeviceDisconnected,
    /// User revoked microphone permission
    PermissionRevoked,
    /// Unable to identify or access audio device
    UnknownDevice,
    /// Automatic or manual reconnection attempt failed
    ReconnectionFailed,
    /// MediaStream ended unexpectedly
    StreamEnded,
    /// Invalid configuration or setup error
    ConfigurationError(String),
}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamError::DeviceDisconnected => write!(f, "Audio device disconnected"),
            StreamError::PermissionRevoked => write!(f, "Microphone permission revoked"),
            StreamError::UnknownDevice => write!(f, "Unknown audio device"),
            StreamError::ReconnectionFailed => write!(f, "Failed to reconnect audio stream"),
            StreamError::StreamEnded => write!(f, "Audio stream ended unexpectedly"),
            StreamError::ConfigurationError(msg) => write!(f, "Stream configuration error: {}", msg),
        }
    }
}

/// Handles MediaStream lifecycle management with automatic reconnection
/// 
/// Primary interface for managing audio stream connections, monitoring health,
/// and implementing robust reconnection logic when devices are disconnected.
/// 
/// # Features
/// 
/// - **Automatic Health Monitoring**: Continuously monitors stream and track state
/// - **Device Disconnection Detection**: Detects when audio devices are removed
/// - **Configurable Reconnection**: Automatic retry with customizable parameters  
/// - **Manual Recovery**: Supports triggered reconnection attempts
/// - **State Tracking**: Detailed status and diagnostic information
/// 
/// # Usage Pattern
/// 
/// 1. Create handler with desired configuration
/// 2. Set reconnection callback for stream recreation  
/// 3. Attach MediaStream for monitoring
/// 4. Periodically check health status
/// 5. Handle errors and reconnection events
pub struct StreamReconnectionHandler {
    stream_health: Rc<RefCell<StreamHealth>>,
    config: StreamConfig,
    current_stream: Option<MediaStream>,
    reconnect_callback: Option<Box<dyn Fn() -> Result<MediaStream, JsValue>>>,
}

impl StreamReconnectionHandler {
    /// Create new stream reconnection handler
    pub fn new(config: StreamConfig) -> Self {
        let last_activity = if cfg!(target_arch = "wasm32") {
            js_sys::Date::now()
        } else {
            0.0 // Default value for native testing
        };
        
        let stream_health = StreamHealth {
            state: StreamState::Disconnected,
            device_id: None,
            last_activity,
            reconnect_attempts: 0,
            error_message: None,
        };

        Self {
            stream_health: Rc::new(RefCell::new(stream_health)),
            config,
            current_stream: None,
            reconnect_callback: None,
        }
    }

    /// Set stream and begin monitoring
    pub fn set_stream(&mut self, stream: MediaStream) -> Result<(), StreamError> {
        dev_log!("Setting up stream monitoring for device");
        
        // Update stream health
        {
            let mut health = self.stream_health.borrow_mut();
            health.state = StreamState::Connected;
            health.last_activity = if cfg!(target_arch = "wasm32") {
                js_sys::Date::now()
            } else {
                0.0
            };
            health.reconnect_attempts = 0;
            health.error_message = None;
        }

        // Setup stream event listeners
        self.setup_stream_listeners(&stream)?;
        self.current_stream = Some(stream);

        dev_log!("✓ Stream monitoring active");
        Ok(())
    }

    /// Set reconnection callback for automatic stream recreation
    pub fn set_reconnect_callback<F>(&mut self, callback: F) 
    where
        F: Fn() -> Result<MediaStream, JsValue> + 'static,
    {
        self.reconnect_callback = Some(Box::new(callback));
    }

    /// Get current stream health status
    pub fn get_health(&self) -> StreamHealth {
        self.stream_health.borrow().clone()
    }

    /// Check if stream is currently connected
    pub fn is_connected(&self) -> bool {
        matches!(self.stream_health.borrow().state, StreamState::Connected)
    }

    /// Manually trigger reconnection attempt
    pub fn reconnect(&mut self) -> Result<(), StreamError> {
        dev_log!("Manual reconnection requested");
        
        if let Some(callback) = &self.reconnect_callback {
            self.update_state(StreamState::Reconnecting);
            
            match callback() {
                Ok(new_stream) => {
                    self.set_stream(new_stream)?;
                    dev_log!("✓ Manual reconnection successful");
                    Ok(())
                },
                Err(e) => {
                    let error_msg = format!("Reconnection failed: {:?}", e);
                    self.handle_stream_error(StreamError::ReconnectionFailed, Some(error_msg));
                    Err(StreamError::ReconnectionFailed)
                }
            }
        } else {
            Err(StreamError::ConfigurationError("No reconnect callback set".to_string()))
        }
    }

    /// Perform stream health check
    pub fn check_stream_health(&mut self) -> Result<(), StreamError> {
        let current_time = if cfg!(target_arch = "wasm32") {
            js_sys::Date::now()
        } else {
            0.0
        };
        let health = self.stream_health.borrow();
        
        // Check for activity timeout
        if current_time - health.last_activity > self.config.activity_timeout_ms as f64 {
            drop(health);
            dev_log!("Stream activity timeout detected");
            self.handle_stream_error(StreamError::StreamEnded, Some("Activity timeout".to_string()));
            return Err(StreamError::StreamEnded);
        }

        // Check stream tracks if available
        if let Some(ref stream) = self.current_stream {
            let tracks = stream.get_audio_tracks();
            
            if tracks.length() == 0 {
                drop(health);
                dev_log!("No audio tracks available in stream");
                self.handle_stream_error(StreamError::DeviceDisconnected, Some("No audio tracks".to_string()));
                return Err(StreamError::DeviceDisconnected);
            }

            // Check first track state
            if let Ok(track) = tracks.get(0).dyn_into::<MediaStreamTrack>() {
                if track.ready_state() == MediaStreamTrackState::Ended {
                    drop(health);
                    dev_log!("Audio track ended - device likely disconnected");
                    self.handle_stream_error(StreamError::DeviceDisconnected, Some("Track ended".to_string()));
                    return Err(StreamError::DeviceDisconnected);
                }
            }
        }

        // Update last activity if healthy
        drop(health);
        self.update_activity();
        Ok(())
    }

    /// Stop stream monitoring and cleanup
    pub fn stop(&mut self) {
        dev_log!("Stopping stream monitoring");
        
        if let Some(stream) = &self.current_stream {
            let tracks = stream.get_audio_tracks();
            for i in 0..tracks.length() {
                if let Ok(track) = tracks.get(i).dyn_into::<MediaStreamTrack>() {
                    track.stop();
                }
            }
        }

        self.current_stream = None;
        self.update_state(StreamState::Disconnected);
        dev_log!("✓ Stream monitoring stopped");
    }

    // Private helper methods

    fn setup_stream_listeners(&self, stream: &MediaStream) -> Result<(), StreamError> {
        let tracks = stream.get_audio_tracks();
        
        if tracks.length() == 0 {
            return Err(StreamError::ConfigurationError("No audio tracks in stream".to_string()));
        }

        // Setup track event listeners for the first audio track
        if let Ok(track) = tracks.get(0).dyn_into::<MediaStreamTrack>() {
            let health_ref = self.stream_health.clone();
            
            // Track ended event
            let ended_closure = Closure::wrap(Box::new(move || {
                let mut health = health_ref.borrow_mut();
                health.state = StreamState::Failed;
                health.error_message = Some("Track ended".to_string());
                dev_log!("Audio track ended event fired");
            }) as Box<dyn FnMut()>);

            track.set_onended(Some(ended_closure.as_ref().unchecked_ref()));
            ended_closure.forget(); // Keep closure alive
        }

        Ok(())
    }

    fn handle_stream_error(&mut self, error: StreamError, message: Option<String>) {
        dev_log!("Stream error occurred: {:?}", error);
        
        let mut health = self.stream_health.borrow_mut();
        health.state = StreamState::Failed;
        health.error_message = message.or_else(|| Some(error.to_string()));

        // Attempt automatic reconnection if configured and callback available
        if health.reconnect_attempts < self.config.max_reconnect_attempts {
            health.reconnect_attempts += 1;
            health.state = StreamState::Reconnecting;
            
            drop(health);
            dev_log!("Attempting automatic reconnection (attempt {})", self.stream_health.borrow().reconnect_attempts);
            
            // Attempt reconnection if callback is available
            if let Some(ref callback) = self.reconnect_callback {
                match callback() {
                    Ok(new_stream) => {
                        if let Err(_e) = self.set_stream(new_stream) {
                            dev_log!("Automatic reconnection failed: {:?}", _e);
                        } else {
                            dev_log!("✓ Automatic reconnection successful");
                        }
                    },
                    Err(_e) => {
                        dev_log!("Automatic reconnection callback failed: {:?}", _e);
                        let mut health = self.stream_health.borrow_mut();
                        health.state = StreamState::Failed;
                    }
                }
            }
        }
    }

    fn update_state(&self, new_state: StreamState) {
        let mut health = self.stream_health.borrow_mut();
        health.state = new_state;
    }

    fn update_activity(&self) {
        let mut health = self.stream_health.borrow_mut();
        health.last_activity = if cfg!(target_arch = "wasm32") {
            js_sys::Date::now()
        } else {
            0.0
        };
    }
}

impl Default for StreamReconnectionHandler {
    fn default() -> Self {
        Self::new(StreamConfig::default())
    }
}

