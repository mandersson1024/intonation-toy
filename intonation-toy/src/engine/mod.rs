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
use web_sys::AudioContext;
use crate::engine::audio::data_types::AudioWorkletStatus;
use crate::engine::audio::message_protocol::BufferPoolStats;
use crate::engine::audio::worklet::AudioWorkletManager;

// Debug-only imports for conditional compilation
#[cfg(debug_assertions)]
use crate::presentation::{DebugLayerActions, ConfigureTestSignal};


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
    /// Direct reference to the Web Audio API context
    audio_context: AudioContext,
    /// Manager for audio worklet operations
    audioworklet_manager: AudioWorkletManager,
}

impl AudioEngine {
    /// Create a new AudioEngine for raw audio processing
    /// 
    /// This constructor accepts an AudioContext from `create_audio_context_and_load_worklet()`
    /// and sets up all audio processing components (pitch analyzer, volume detector, message handling).
    /// The AudioWorkletNode is created internally by the engine layer.
    /// 
    /// # Arguments
    /// 
    /// * `media_stream` - The MediaStream to connect to the audio worklet
    /// * `audio_context` - AudioContext from early worklet loading
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(AudioEngine)` on successful initialization, or `Err(String)`
    /// if audio system initialization fails.
    pub fn new(
        media_stream: web_sys::MediaStream,
        audio_context: web_sys::AudioContext
    ) -> Result<Self, String> {
        crate::common::dev_log!("✓ AudioContext attached");

        // Create audio pipeline with all audio nodes
        let audio_pipeline = audio::audio_pipeline::AudioPipeline::new(&audio_context)
            .map_err(|e| {
                let error_msg = format!("Failed to create AudioPipeline: {}", e);
                crate::common::dev_log!("✗ {}", error_msg);
                error_msg
            })?;
        crate::common::dev_log!("✓ AudioPipeline created with audio nodes");

        let mut worklet_manager = audio::worklet::AudioWorkletManager::new(audio_context.clone(), audio_pipeline)
            .map_err(|e| {
                let error_msg = format!("Failed to create AudioWorkletManager: {}", e);
                crate::common::dev_log!("✗ {}", error_msg);
                error_msg
            })?;
        crate::common::dev_log!("✓ AudioWorkletManager created with provided AudioPipeline");
        
        // PitchAnalyzer is now created internally in AudioWorkletManager::new()
        crate::common::dev_log!("✓ PitchAnalyzer initialized internally in AudioWorkletManager");

        // VolumeDetector is now created internally in AudioWorkletManager::new()
        worklet_manager.setup_message_handling()
            .map_err(|e| {
                let error_msg = format!("Failed to setup message handling: {:?}", e);
                crate::common::dev_log!("✗ {}", error_msg);
                error_msg
            })?;
        
        crate::common::dev_log!("✓ VolumeDetector initialized and configured");

        // Create the engine struct with all initialized components
        let mut engine = Self {
            audio_context,
            audioworklet_manager: worklet_manager,
        };

        // Connect media stream to audioworklet (preserving existing media stream handling)
        let node = crate::engine::audio::legacy_media_stream_node::legacy_create_media_stream_node(&media_stream, &engine.audio_context)
            .map_err(|e| format!("MediaStream connection failed: {}", e))?;
        
        crate::engine::audio::legacy_media_stream_node::legacy_connect_media_stream_node_to_audioworklet(&node, &mut engine)
            .map_err(|e| format!("MediaStream connection failed: {}", e))?;
        
        // Configure default tuning fork
        engine.audioworklet_manager.update_tuning_fork_config(audio::TuningForkConfig::default());

        crate::common::dev_log!("✓ AudioEngine fully initialized");
        Ok(engine)
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
    /// 
    /// Note: All musical interpretation (tuning systems, intervals, pitch relationships)
    /// is handled by the model layer that processes this raw data.
    pub fn update(&mut self) -> EngineUpdateResult {
        EngineUpdateResult {
            audio_analysis: self.collect_audio_analysis(),
            audio_errors: self.collect_audio_errors(),
        }
    }
    

    #[cfg(debug_assertions)]
    pub fn get_debug_audioworklet_status(&self) -> Option<AudioWorkletStatus> {
        Some(self.audioworklet_manager.get_status())
    }

    #[cfg(debug_assertions)]
    pub fn get_debug_buffer_pool_stats(&self) -> Option<BufferPoolStats> {
        self.audioworklet_manager.get_buffer_pool_statistics()
    }
    
    
    /// Execute model layer actions
    /// 
    /// Processes tuning fork audio configuration from the model layer.
    /// The engine handles raw audio while the model handles musical interpretation.
    pub fn execute_actions(&mut self, model_actions: ModelLayerActions) {
        if !model_actions.has_actions() {
            return;
        }
        
        let worklet_manager = &mut self.audioworklet_manager;
        
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
        &mut self,
        test_signal_configs: &[ConfigureTestSignal]
    ) -> Result<(), String> {
        for config in test_signal_configs {
            crate::common::dev_log!(
                "[DEBUG] Executing privileged test signal configuration - enabled: {}, freq: {} Hz, vol: {}%",
                config.enabled, config.frequency, config.volume
            );
            
            let audio_config = crate::engine::audio::SignalGeneratorConfig {
                enabled: config.enabled,
                frequency: config.frequency,
                amplitude: config.volume / 100.0,
                sample_rate: STANDARD_SAMPLE_RATE,
            };
            
            self.audioworklet_manager.update_test_signal_config(audio_config);
            crate::common::dev_log!(
                "[DEBUG] ✓ Test signal control updated - enabled: {}, freq: {}, vol: {}%", 
                config.enabled, config.frequency, config.volume
            );
        }
        Ok(())
    }
    
    /// Collect audio analysis data from the engine components
    fn collect_audio_analysis(&self) -> Option<crate::common::shared_types::AudioAnalysis> {
        use crate::common::shared_types::{Volume, Pitch, AudioAnalysis};
        
        let volume_data = self.audioworklet_manager.get_volume_data();
        let volume = volume_data.as_ref().map(|data| Volume {
            peak_amplitude: data.peak_amplitude,
            rms_amplitude: data.rms_amplitude,
        });
        
        // Extract FFT data from volume data when available
        let fft_data = volume_data.and_then(|data| data.fft_data.clone());
        
        let pitch_data = self.audioworklet_manager.get_pitch_data();
        let pitch = pitch_data.map(|data| {
            if data.frequency > 0.0 {
                Pitch::Detected(data.frequency, data.clarity)
            } else {
                Pitch::NotDetected
            }
        });
        
        (volume.is_some() || pitch.is_some()).then(|| AudioAnalysis {
            volume_level: volume.unwrap_or(Volume { peak_amplitude: 0.0, rms_amplitude: 0.0 }),
            pitch: pitch.unwrap_or(Pitch::NotDetected),
            fft_data,
        })
    }
    
    /// Collect audio errors from the engine components
    fn collect_audio_errors(&self) -> Vec<crate::common::shared_types::Error> {
        use web_sys::AudioContextState;
        let mut errors = Vec::new();
        
        if self.audio_context.state() != AudioContextState::Running {
            let error_msg = match self.audio_context.state() {
                AudioContextState::Closed => Some("AudioContext is closed"),
                // Suspended is a normal state before user interaction, not an error
                AudioContextState::Suspended => None,
                _ => None,
            };
            if let Some(msg) = error_msg {
                errors.push(crate::common::shared_types::Error::ProcessingError(msg.to_string()));
            }
        }
        
        errors
    }
    

    /// Get reference to the audio context
    pub fn get_audio_context(&self) -> &AudioContext {
        &self.audio_context
    }
    
    /// Get the current audioworklet status
    pub fn get_audioworklet_status(&self) -> Option<AudioWorkletStatus> {
        Some(self.audioworklet_manager.get_status())
    }
    
    /// Get buffer pool statistics from the audioworklet
    pub fn get_buffer_pool_stats(&self) -> Option<BufferPoolStats> {
        self.audioworklet_manager.get_buffer_pool_statistics()
    }
    
    /// Get mutable reference to the audioworklet manager
    pub fn get_audioworklet_manager_mut(&mut self) -> &mut AudioWorkletManager {
        &mut self.audioworklet_manager
    }

    /// Configure tuning fork audio settings
    pub fn configure_tuning_fork(&mut self, config: audio::TuningForkConfig) {
        self.audioworklet_manager.update_tuning_fork_config(config);
    }
    
}