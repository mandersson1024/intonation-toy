//! Engine Layer - Raw audio processing and hardware interface
//!
//! The Engine layer is the lowest level of the three-layer architecture,
//! responsible exclusively for raw audio processing and hardware communication.
//! It handles no musical interpretation - all tuning systems, root notes, and
//! pitch analysis are handled by the Model layer.
//!
//! ## Role in Three-Layer Architecture
//!
//! The engine layer:
//! - **Audio Hardware Interface**: Manages microphone/speaker access and permissions
//! - **Raw Audio Processing**: Performs low-level pitch detection and volume analysis
//! - **Browser API Integration**: Handles Web Audio API and MediaStream operations
//! - **Data Provider**: Returns raw audio data via EngineUpdateResult for Model layer processing
//! - **No Musical Logic**: Contains no tuning systems, scales, or musical interpretation
//!
//! ## Data Flow in Engine Layer
//!
//! The engine layer:
//! - Processes raw audio data from microphone and browser APIs
//! - Performs frequency analysis and pitch detection (Hz values only)
//! - Returns structured raw data via EngineUpdateResult from update() calls
//! - Provides audio analysis, error information, and permission state
//!
//! ```rust
//! use intonation_toy::engine::AudioEngine;
//!
//! // Create engine without dependencies
//! let mut engine = AudioEngine::create().await?;
//!
//! // Engine returns data directly from update calls
//! let result = engine.update(timestamp);
//! // result contains: audio_analysis, audio_errors, permission_state
//! 
//! // Access the aggregated data
//! if let Some(analysis) = result.audio_analysis {
//!     println!("Volume: {} dB", analysis.volume_level.peak);
//!     match analysis.pitch {
//!         Pitch::Detected(freq, clarity) => {
//!             println!("Pitch: {} Hz (clarity: {})", freq, clarity);
//!         }
//!         Pitch::NotDetected => println!("No pitch detected"),
//!     }
//! }
//! 
//! // Check for errors
//! for error in &result.audio_errors {
//!     eprintln!("Audio error: {:?}", error);
//! }
//! 
//! // Check permission state
//! match result.permission_state {
//!     PermissionState::Granted => println!("Microphone access granted"),
//!     PermissionState::Denied => println!("Microphone access denied"),
//!     _ => {}
//! }
//! ```

pub mod audio;
pub(crate) mod platform;

use crate::shared_types::EngineUpdateResult;
use crate::model::ModelLayerActions;
use crate::app_config::STANDARD_SAMPLE_RATE;

// Debug-only imports for conditional compilation
#[cfg(debug_assertions)]
use crate::presentation::{DebugLayerActions, ConfigureTestSignal};
#[cfg(debug_assertions)]
use self::audio::{AudioDevices, AudioWorkletStatus, message_protocol::BufferPoolStats};

// Debug execution action structs (only available in debug builds)
#[cfg(debug_assertions)]
#[derive(Debug, Clone, PartialEq)]
pub struct ExecuteTestSignalConfiguration {
    pub enabled: bool,
    pub frequency: f32,
    pub volume: f32,
}

/// Container for all executed debug layer actions (debug builds only)
/// 
/// This struct contains vectors of privileged debug execution actions that have been
/// processed by the engine layer. These actions represent direct operations on the
/// audio system that bypass normal validation and safety checks.
/// 
/// Debug actions provide privileged access to engine internals for testing purposes:
/// - Direct test signal generation control
/// - Direct speaker output manipulation
/// 
/// These actions should only be used for debugging and testing purposes.
#[cfg(debug_assertions)]
#[derive(Debug, Clone, PartialEq)]
pub struct DebugEngineActions {
    /// Executed test signal configurations
    pub test_signal_executions: Vec<ExecuteTestSignalConfiguration>,
}

#[cfg(debug_assertions)]
impl Default for DebugEngineActions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(debug_assertions)]
impl DebugEngineActions {
    /// Create a new instance with empty debug action collections
    /// 
    /// Returns a new `DebugEngineActions` struct with all action vectors initialized
    /// as empty. This is used as the starting point for collecting executed debug actions.
    pub fn new() -> Self {
        Self {
            test_signal_executions: Vec::new(),
        }
    }
}

/// AudioEngine - The engine layer of the three-layer architecture
/// 
/// This struct represents the raw audio processing and hardware interface layer
/// of the application. It handles low-level audio operations, browser API
/// interactions, and microphone/speaker communication, with no musical interpretation.
/// 
/// The engine provides raw audio data (frequencies in Hz, volume amplitudes) to
/// the model layer, which handles all musical logic including tuning systems,
/// root notes, and pitch relationships.
/// 
/// # Example
/// 
/// ```no_run
/// use intonation_toy::engine::AudioEngine;
/// 
/// let engine = AudioEngine::create()
///     .await.expect("AudioEngine creation should succeed");
/// ```
pub struct AudioEngine {
    /// Audio system context for managing audio processing
    audio_context: Option<std::rc::Rc<std::cell::RefCell<audio::AudioSystemContext>>>,
}

impl AudioEngine {
    /// Create a new AudioEngine for raw audio processing
    /// 
    /// This constructor initializes the audio processing system for raw audio
    /// analysis. The engine provides frequency and amplitude data to the model
    /// layer for musical interpretation.
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(AudioEngine)` on successful initialization, or `Err(String)`
    /// if audio system initialization fails.
    pub async fn create() -> Result<Self, String> {
        crate::common::dev_log!("Creating AudioEngine with return-based pattern");
        
        // Create audio context using the new return-based constructor
        let mut audio_context = audio::AudioSystemContext::new_return_based();
        
        // Initialize the audio system
        match audio_context.initialize().await {
            Ok(()) => {
                crate::common::dev_log!("✓ AudioEngine created and initialized successfully");
                
                let audio_context_rc = std::rc::Rc::new(std::cell::RefCell::new(audio_context));
                
                // Initialize default tuning fork audio with zero volume
                crate::common::dev_log!("Initializing default tuning fork audio with zero volume");
                let default_tuning_fork_note = crate::app_config::DEFAULT_TUNING_FORK_NOTE;
                let default_frequency = crate::music_theory::midi_note_to_standard_frequency(default_tuning_fork_note);
                
                if let Ok(mut borrowed_context) = audio_context_rc.try_borrow_mut() {
                    let tuning_fork_config = crate::engine::audio::TuningForkConfig {
                        frequency: default_frequency,
                        volume: 0.0,
                    };
                    borrowed_context.configure_tuning_fork(tuning_fork_config);
                }
                
                Ok(Self {
                    audio_context: Some(audio_context_rc),
                })
            }
            Err(e) => {
                crate::common::dev_log!("⚠ AudioEngine created but audio system initialization failed: {}", e);
                // Still create the engine but without audio context for now
                // This allows the application to continue running
                Ok(Self {
                    audio_context: None,
                })
            }
        }
    }

    /// Update the engine layer with a new timestamp
    /// 
    /// This method is called by the main render loop to update the engine's state.
    /// It processes raw audio data and returns uninterpreted audio analysis for
    /// the model layer to handle musical interpretation.
    /// 
    /// # Arguments
    /// 
    /// * `timestamp` - The current timestamp in seconds since application start
    /// 
    /// # Returns
    /// 
    /// Returns `EngineUpdateResult` containing:
    /// - Raw audio analysis (frequency in Hz, volume amplitude)
    /// - Audio system errors and status
    /// - Microphone permission state
    /// 
    /// Note: All musical interpretation (tuning systems, intervals, pitch relationships)
    /// is handled by the model layer that processes this raw data.
    pub fn update(&mut self, timestamp: f64) -> EngineUpdateResult {
        if let Some(ref context) = self.audio_context {
            // Borrow once and collect all data to avoid multiple borrows
            let borrowed_context = context.borrow();
            let audio_analysis = borrowed_context.collect_audio_analysis(timestamp);
            let audio_errors = borrowed_context.collect_audio_errors();
            let permission_state = borrowed_context.collect_permission_state();
            
            
            EngineUpdateResult {
                audio_analysis,
                audio_errors,
                permission_state,
            }
        } else {
            // No audio context available
            EngineUpdateResult {
                audio_analysis: None,
                audio_errors: vec![crate::shared_types::Error::ProcessingError("Audio system not initialized".to_string())],
                permission_state: crate::shared_types::PermissionState::NotRequested,
            }
        }
    }
    
    #[cfg(debug_assertions)]
    pub fn get_debug_audio_devices(&self) -> Option<AudioDevices> {
        self.audio_context.as_ref().map(|ctx| {
            match ctx.try_borrow() {
                Ok(borrowed) => borrowed.get_audio_devices(),
                Err(_) => AudioDevices { input_devices: Vec::new(), output_devices: Vec::new() }
            }
        })
    }

    #[cfg(debug_assertions)]
    pub fn get_debug_audioworklet_status(&self) -> Option<AudioWorkletStatus> {
        self.audio_context.as_ref().and_then(|ctx| {
            match ctx.try_borrow() {
                Ok(borrowed) => borrowed.get_audioworklet_status(),
                Err(_) => None
            }
        })
    }

    #[cfg(debug_assertions)]
    pub fn get_debug_buffer_pool_stats(&self) -> Option<BufferPoolStats> {
        self.audio_context.as_ref().and_then(|ctx| {
            match ctx.try_borrow() {
                Ok(borrowed) => borrowed.get_buffer_pool_stats(),
                Err(_) => None
            }
        })
    }
    
    
    /// Get the audio context for async operations
    /// 
    /// Returns a clone of the Rc<RefCell<AudioSystemContext>> if available.
    /// This is used for async operations that need access to the raw audio
    /// processing context outside of the main engine instance.
    pub fn get_audio_context(&self) -> Option<std::rc::Rc<std::cell::RefCell<audio::AudioSystemContext>>> {
        self.audio_context.clone()
    }
    
    /// Connect an existing MediaStream to the raw audio processing pipeline
    /// 
    /// This method accepts a MediaStream that was obtained through user gesture
    /// and connects it directly to the AudioWorklet for raw audio analysis.
    /// The processed audio data is provided to the model layer for musical interpretation.
    /// 
    /// # Arguments
    /// 
    /// * `media_stream` - The MediaStream to connect (should contain audio tracks)
    /// 
    /// # Returns
    /// 
    /// Returns `Result<(), String>` indicating success or failure of the connection.
    pub async fn connect_mediastream(&self, media_stream: web_sys::MediaStream) -> Result<(), String> {
        if let Some(ref audio_context) = self.audio_context {
            audio::microphone::connect_existing_mediastream_to_audioworklet(media_stream, audio_context).await
        } else {
            Err("Audio system not initialized".to_string())
        }
    }
    
    /// Execute model layer actions (currently reserved for future engine-specific actions)
    /// 
    /// This method is reserved for future engine-specific actions. The engine layer
    /// focuses solely on raw audio processing and hardware interface operations.
    /// All musical interpretation (tuning systems, root notes, pitch analysis) is
    /// handled exclusively by the model layer.
    /// 
    /// # Arguments
    /// 
    /// * `model_actions` - Model layer actions (currently unused as no engine actions exist)
    /// 
    /// # Returns
    /// 
    /// Returns `Result<(), String>` indicating success (currently always succeeds)
    /// 
    /// # Engine Layer Scope
    /// 
    /// The engine layer handles only:
    /// - Raw audio hardware interface operations
    /// - Low-level audio processing (frequency detection, volume analysis)
    /// - Browser API interactions (MediaStream, AudioWorklet)
    /// - Audio device management and permissions
    /// 
    /// Musical interpretation is handled by the model layer:
    /// - Tuning system calculations
    /// - Relationship to tuning fork note
    /// - Pitch analysis and interval calculations
    pub fn execute_actions(&mut self, model_actions: ModelLayerActions) -> Result<(), String> {
        // The engine layer handles only raw audio processing and hardware interface.
        // All musical interpretation including tuning systems, tuning forks, and
        // pitch analysis is handled exclusively by the model layer.
        
        // Process tuning fork audio configurations
        for config in &model_actions.tuning_fork_configurations {
            crate::common::dev_log!(
                "Engine layer: Executing tuning fork audio configuration - frequency: {} Hz",
                config.frequency
            );
            
            // Execute the tuning fork audio configuration using the audio system
            if let Some(ref audio_context) = self.audio_context {
                let mut borrowed_context = audio_context.borrow_mut();
                if let Some(worklet_manager) = borrowed_context.get_audioworklet_manager_mut() {
                    // Convert model action to audio system config
                    let audio_config = crate::engine::audio::signal_generator::TuningForkConfig {
                        frequency: config.frequency,
                        volume: config.volume,
                    };
                    
                    // Use the separate tuning fork audio node architecture
                    worklet_manager.update_tuning_fork_config(audio_config);
                    crate::common::dev_log!(
                        "Engine layer: ✓ Tuning fork audio control updated - frequency: {} Hz", 
                        config.frequency
                    );
                } else {
                    crate::common::dev_log!(
                        "Engine layer: ⚠ AudioWorkletManager not available for tuning fork audio control"
                    );
                }
            } else {
                return Err("Audio context not available for tuning fork audio execution".to_string());
            }
        }
        
        let total_tuning_fork_audio = model_actions.tuning_fork_configurations.len();
        if total_tuning_fork_audio > 0 {
            crate::common::dev_log!("Engine layer: ✓ Executed {} tuning fork audio configurations", total_tuning_fork_audio);
        }
        
        crate::common::dev_log!("Engine layer: Action execution completed");
        
        Ok(())
    }
    
    
    /// Execute debug actions with privileged engine access (debug builds only)
    /// 
    /// This method processes debug actions from the presentation layer that provide
    /// direct, privileged access to engine operations. These actions bypass normal
    /// validation and safety checks and should only be used for testing and debugging.
    /// 
    /// # Arguments
    /// 
    /// * `debug_actions` - Debug actions from the presentation layer to execute
    /// 
    /// # Returns
    /// 
    /// Returns `Result<DebugEngineActions, String>` containing either the successfully
    /// executed debug actions or an error message if execution failed.
    /// 
    /// # Safety
    /// 
    /// Debug actions provide direct access to engine internals and bypass normal
    /// safety checks. They should only be used in debug builds for testing purposes.
    /// 
    /// # Privileged Operations
    /// 
    /// - Test signal generation: Direct control over audio worklet test signals
    /// - Speaker output: Direct manipulation of speaker output routing
    #[cfg(debug_assertions)]
    pub fn execute_debug_actions_sync(&mut self, debug_actions: DebugLayerActions) -> Result<DebugEngineActions, String> {
        crate::common::dev_log!("[DEBUG] Engine layer executing debug actions");
        
        let mut debug_engine_actions = DebugEngineActions::new();
        
        // Execute test signal configurations with privileged access
        self.execute_test_signal_configurations(
            &debug_actions.test_signal_configurations,
            &mut debug_engine_actions
        )?;
        
        let total_executed = debug_engine_actions.test_signal_executions.len();
        
        crate::common::dev_log!("[DEBUG] ✓ Engine layer successfully executed {} debug actions", total_executed);
        
        Ok(debug_engine_actions)
    }
    
    /// Execute debug actions asynchronously (async wrapper for compatibility)
    ///
    /// This method provides backward compatibility for async callers while internally
    /// using the synchronous implementation since debug actions don't require async operations.
    ///
    /// # Arguments
    ///
    /// * `debug_actions` - Debug actions to execute
    ///
    /// # Returns
    ///
    /// Returns `Result<DebugEngineActions, String>` containing the executed debug actions
    /// or an error message if execution failed.
    #[cfg(debug_assertions)]
    pub async fn execute_debug_actions(&mut self, debug_actions: DebugLayerActions) -> Result<DebugEngineActions, String> {
        self.execute_debug_actions_sync(debug_actions)
    }
    
    /// Execute test signal configurations with privileged engine access (debug builds only)
    /// 
    /// This method provides direct control over test signal generation in the audio
    /// worklet, bypassing normal validation checks.
    /// 
    /// # Arguments
    /// 
    /// * `test_signal_configs` - Test signal configurations to execute
    /// * `debug_engine_actions` - Container to store executed actions
    /// 
    /// # Returns
    /// 
    /// Returns `Result<(), String>` indicating success or failure
    #[cfg(debug_assertions)]
    fn execute_test_signal_configurations(
        &self,
        test_signal_configs: &[ConfigureTestSignal],
        debug_engine_actions: &mut DebugEngineActions
    ) -> Result<(), String> {
        for config in test_signal_configs {
            crate::common::dev_log!(
                "[DEBUG] Executing privileged test signal configuration - enabled: {}, freq: {} Hz, vol: {}%",
                config.enabled, config.frequency, config.volume
            );
            
            // Direct privileged access to test signal generation
            if let Some(ref audio_context) = self.audio_context {
                let mut borrowed_context = audio_context.borrow_mut();
                if let Some(worklet_manager) = borrowed_context.get_audioworklet_manager_mut() {
                    // Convert debug action to audio system config
                    let audio_config = crate::engine::audio::SignalGeneratorConfig {
                        enabled: config.enabled,
                        frequency: config.frequency,
                        amplitude: config.volume / 100.0, // Convert percentage to 0-1 range
                        sample_rate: STANDARD_SAMPLE_RATE, // Use standard consumer audio sample rate
                    };
                    
                    worklet_manager.update_test_signal_config(audio_config);
                    crate::common::dev_log!(
                        "[DEBUG] ✓ Test signal control updated - enabled: {}, freq: {}, vol: {}%", 
                        config.enabled, config.frequency, config.volume
                    );
                } else {
                    crate::common::dev_log!(
                        "[DEBUG] ⚠ AudioWorkletManager not available for test signal control"
                    );
                }
                
                // Record the executed action
                debug_engine_actions.test_signal_executions.push(ExecuteTestSignalConfiguration {
                    enabled: config.enabled,
                    frequency: config.frequency,
                    volume: config.volume,
                });
            } else {
                return Err("[DEBUG] Audio context not available for test signal execution".to_string());
            }
        }
        
        crate::common::dev_log!(
            "[DEBUG] ✓ Executed {} test signal configurations with privileged access",
            test_signal_configs.len()
        );
        Ok(())
    }
    
}