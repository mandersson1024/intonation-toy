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

pub mod audio;
pub(crate) mod platform;

use crate::common::shared_types::EngineUpdateResult;
use crate::model::ModelLayerActions;
use crate::app_config::STANDARD_SAMPLE_RATE;

// Debug-only imports for conditional compilation
#[cfg(debug_assertions)]
use crate::presentation::{DebugLayerActions, ConfigureTestSignal};
#[cfg(debug_assertions)]
use self::audio::{AudioWorkletStatus, message_protocol::BufferPoolStats};


/// AudioEngine - The engine layer of the three-layer architecture
/// 
/// This struct represents the raw audio processing and hardware interface layer
/// of the application. It handles low-level audio operations, browser API
/// interactions, and microphone/speaker communication, with no musical interpretation.
/// 
/// The engine provides raw audio data (frequencies in Hz, volume amplitudes) to
/// the model layer, which handles all musical logic including tuning systems,
/// root notes, and pitch relationships.
pub struct AudioEngine {
    /// Audio system context for managing audio processing
    audio_context: Option<std::rc::Rc<std::cell::RefCell<audio::AudioSystemContext>>>,
}

impl AudioEngine {
    /// Create a new AudioEngine for raw audio processing
    /// 
    /// This constructor accepts audio components from `load_worklet_early()`
    /// and sets up all audio processing components (pitch analyzer, volume detector, message handling).
    /// 
    /// # Arguments
    /// 
    /// * `media_stream` - The MediaStream to connect to the audio worklet
    /// * `audio_context` - AudioContext from early worklet loading
    /// * `audio_worklet_node` - AudioWorkletNode from early worklet loading
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(AudioEngine)` on successful initialization, or `Err(String)`
    /// if audio system initialization fails.
    pub async fn new(
        media_stream: web_sys::MediaStream,
        audio_context: web_sys::AudioContext,
        audio_worklet_node: web_sys::AudioWorkletNode
    ) -> Result<Self, String> {
        crate::common::dev_log!("Creating AudioEngine with worklet components");
        
        // Create audio context using provided components
        let audio_context_obj = match audio::AudioSystemContext::create(audio_context, audio_worklet_node).await {
            Ok(context) => context,
            Err(e) => {
                crate::common::error_log!("✗ AudioEngine creation failed: {}", e);
                return Err(e);
            }
        };
        
        crate::common::dev_log!("✓ AudioEngine created successfully");
        
        let audio_context_rc = std::rc::Rc::new(std::cell::RefCell::new(audio_context_obj));
            
        // Connect the media stream to the audio worklet
        if let Err(e) = crate::engine::audio::microphone::connect_mediastream_to_audioworklet(
            media_stream,
            &audio_context_rc
        ) {
            return Err(format!("MediaStream connection failed: {}", e));
        }
        
        if let Ok(mut borrowed_context) = audio_context_rc.try_borrow_mut() {
            borrowed_context.configure_tuning_fork(crate::engine::audio::TuningForkConfig {
                frequency: crate::common::music_theory::midi_note_to_standard_frequency(crate::app_config::DEFAULT_TUNING_FORK_NOTE),
                volume: 0.0,
            });
        }
        
        Ok(Self {
            audio_context: Some(audio_context_rc),
        })
    }
    

    /// Update the engine layer
    /// 
    /// This method is called by the main render loop to update the engine's state.
    /// It processes raw audio data and returns uninterpreted audio analysis for
    /// the model layer to handle musical interpretation.
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
    pub fn update(&mut self) -> EngineUpdateResult {
        let Some(ref context) = self.audio_context else {
            return EngineUpdateResult {
                audio_analysis: None,
                audio_errors: vec![crate::common::shared_types::Error::ProcessingError("Audio system not initialized".to_string())],
                permission_state: crate::common::shared_types::PermissionState::NotRequested,
            };
        };
        
        let borrowed_context = context.borrow();
        EngineUpdateResult {
            audio_analysis: borrowed_context.collect_audio_analysis(),
            audio_errors: borrowed_context.collect_audio_errors(),
            permission_state: borrowed_context.collect_permission_state(),
        }
    }
    

    #[cfg(debug_assertions)]
    pub fn get_debug_audioworklet_status(&self) -> Option<AudioWorkletStatus> {
        self.audio_context.as_ref()?
            .try_borrow().ok()?
            .get_audioworklet_status()
    }

    #[cfg(debug_assertions)]
    pub fn get_debug_buffer_pool_stats(&self) -> Option<BufferPoolStats> {
        self.audio_context.as_ref()?
            .try_borrow().ok()?
            .get_buffer_pool_stats()
    }
    
    
    /// Get the audio context for async operations
    /// 
    /// Returns a clone of the Rc<RefCell<AudioSystemContext>> if available.
    /// This is used for async operations that need access to the raw audio
    /// processing context outside of the main engine instance.
    pub fn get_audio_context(&self) -> Option<std::rc::Rc<std::cell::RefCell<audio::AudioSystemContext>>> {
        self.audio_context.clone()
    }
    
    /// Execute model layer actions
    /// 
    /// Processes tuning fork audio configuration from the model layer.
    /// The engine handles raw audio while the model handles musical interpretation.
    pub fn execute_actions(&mut self, model_actions: ModelLayerActions) {
        if !model_actions.has_actions() {
            return;
        }
        
        let Some(ref audio_context) = self.audio_context else {
            debug_assert!(false, "Audio context not available for tuning fork audio execution");
            return;
        };
        
        let mut borrowed_context = audio_context.borrow_mut();
        let Some(worklet_manager) = borrowed_context.get_audioworklet_manager_mut() else {
            debug_assert!(false, "AudioWorkletManager not available for tuning fork audio control");
            return;
        };
        
        if let Some(config) = model_actions.tuning_fork_configuration {
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
        };
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
    pub fn execute_debug_actions_sync(&mut self, debug_actions: DebugLayerActions) -> Result<(), String> {
        crate::common::dev_log!("[DEBUG] Engine layer executing debug actions");
        
        // Execute test signal configurations with privileged access
        self.execute_test_signal_configurations(&debug_actions.test_signal_configurations)?;
        
        Ok(())
    }
    
    /// Execute test signal configurations with privileged engine access
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
        test_signal_configs: &[ConfigureTestSignal]
    ) -> Result<(), String> {
        for config in test_signal_configs {
            crate::common::dev_log!(
                "[DEBUG] Executing privileged test signal configuration - enabled: {}, freq: {} Hz, vol: {}%",
                config.enabled, config.frequency, config.volume
            );
            
            let Some(ref audio_context) = self.audio_context else {
                return Err("[DEBUG] Audio context not available for test signal execution".to_string());
            };
            
            let mut borrowed_context = audio_context.borrow_mut();
            if let Some(worklet_manager) = borrowed_context.get_audioworklet_manager_mut() {
                let audio_config = crate::engine::audio::SignalGeneratorConfig {
                    enabled: config.enabled,
                    frequency: config.frequency,
                    amplitude: config.volume / 100.0,
                    sample_rate: STANDARD_SAMPLE_RATE,
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
        }
        Ok(())
    }
    
}