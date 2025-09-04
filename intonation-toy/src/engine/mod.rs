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
    audio_context: AudioContext,
    audio_pipeline: audio::audio_pipeline::AudioPipeline,
    audioworklet_manager: AudioWorkletManager,
    output_to_speakers: bool,
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

        // Extract worklet_node for AudioWorkletManager
        let worklet_node = audio_pipeline.worklet_node.clone();
        
        // Create PitchAnalyzer with default config and audio context sample rate
        let config = audio::pitch_detector::PitchDetectorConfig::default();
        let sample_rate = audio_context.sample_rate() as u32;
        let pitch_analyzer = audio::pitch_analyzer::PitchAnalyzer::new(config, sample_rate)
            .map_err(|e| {
                let error_msg = format!("Failed to create PitchAnalyzer: {}", e);
                crate::common::dev_log!("✗ {}", error_msg);
                error_msg
            })?;
        
        let mut worklet_manager = audio::worklet::AudioWorkletManager::new(audio_context.clone(), worklet_node)
            .map_err(|e| {
                let error_msg = format!("Failed to create AudioWorkletManager: {}", e);
                crate::common::dev_log!("✗ {}", error_msg);
                error_msg
            })?;

        worklet_manager.setup_message_handling(pitch_analyzer)
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
            audio_pipeline,
            output_to_speakers: false,
        };

        // Connect media stream to audioworklet (preserving existing media stream handling)
        let node = engine.create_media_stream_node(&media_stream)
            .map_err(|e| format!("MediaStream connection failed: {}", e))?;
        
        engine.connect_media_stream_to_audioworklet(&node)
            .map_err(|e| format!("MediaStream connection failed: {}", e))?;
        
        // Configure default tuning fork
        engine.update_tuning_fork_config(audio::TuningForkConfig::default());

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
        
        
        if let Some(config) = model_actions.tuning_fork_configuration {
            // Convert model action to audio system config
            let audio_config = crate::engine::audio::signal_generator::TuningForkConfig {
                frequency: config.frequency,
                volume: config.volume,
            };
            
            // Use the separate tuning fork audio node architecture
            self.update_tuning_fork_config(audio_config);
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
            
            self.update_test_signal_config(audio_config);
            crate::common::dev_log!(
                "[DEBUG] ✓ Test signal control updated - enabled: {}, freq: {}, vol: {}%", 
                config.enabled, config.frequency, config.volume
            );
        }
        Ok(())
    }

    /// Set microphone volume
    fn set_microphone_volume(&mut self, volume: f32) {
        // Clamp volume to 0.0 - 1.0 range
        let clamped_volume = volume.clamp(0.0, 1.0);
        
        // Set the gain value on the microphone gain node
        self.audio_pipeline.microphone_gain_node.gain().set_value(clamped_volume);
        
        crate::common::dev_log!("Set microphone volume to {:.2} (requested: {:.2})", clamped_volume, volume);
    }

    /// Set whether to output audio stream to speakers
    fn set_output_to_speakers(&mut self, enabled: bool) {
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
        match self.audio_pipeline.worklet_node.connect_with_audio_node(&destination) {
            Ok(_) => {
                crate::common::dev_log!("🔊 AudioWorklet connected to speakers");
            }
            Err(e) => {
                crate::common::dev_log!("🔇 Failed to connect AudioWorklet to speakers: {:?}", e);
            }
        }
    }
    
    /// Disconnect AudioWorklet output from speakers  
    fn disconnect_worklet_from_speakers(&self) {
        let destination = self.audio_context.destination();
        // Disconnect only the connection to destination (speakers)
        match self.audio_pipeline.worklet_node.disconnect_with_audio_node(&destination) {
            Ok(_) => {
                crate::common::dev_log!("🔇 AudioWorklet disconnected from speakers");
            }
            Err(e) => {
                crate::common::dev_log!("⚠️ Could not disconnect from speakers (may not be connected): {:?}", e);
            }
        }
    }

    /// Update tuning fork audio configuration
    /// 
    /// This method manages the dedicated TuningForkAudioNode that connects directly to speakers,
    /// independent of the main AudioWorklet processing pipeline. Tuning fork audio is always
    /// audible regardless of the output_to_speakers flag.
    fn update_tuning_fork_config(&mut self, config: audio::TuningForkConfig) {
        crate::common::dev_log!("[AudioEngine] Updating tuning fork audio config - frequency: {} Hz", 
                config.frequency);
        
        // Update the tuning fork audio node
        self.audio_pipeline.tuning_fork_node.update_config(config);
    }

    /// Update test signal generator configuration (unified routing - no reconnection needed)
    fn update_test_signal_config(&mut self, config: audio::SignalGeneratorConfig) {
        // Handle microphone muting for test signals to prevent feedback
        if config.enabled {
            // Mute microphone when test signal is active
            
            // Mute microphone to prevent feedback (no reconnection needed - just volume control)
            self.set_microphone_volume(0.0);
            
            // Enable speaker output for test signal
            if !self.output_to_speakers {
                self.set_output_to_speakers(true);
                crate::common::dev_log!("Automatically enabled speaker output for test signal");
            }
        }
        
        // Then manage local TestSignalAudioNode
        if config.enabled {
            // Update existing node
            self.audio_pipeline.test_signal_node.update_config(config);
            crate::common::dev_log!("Updated test signal node configuration");
        } else {
            // Disable test signal but keep node for potential re-enabling
            self.audio_pipeline.test_signal_node.disable();
            crate::common::dev_log!("Disabled test signal node");
            self.set_microphone_volume(1.0);
            self.set_output_to_speakers(false);
        }
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
        self.update_tuning_fork_config(config);
    }

    /// Creates a MediaStreamAudioSourceNode from a MediaStream
    fn create_media_stream_node(
        &self,
        media_stream: &web_sys::MediaStream,
    ) -> Result<web_sys::MediaStreamAudioSourceNode, String> {
        self.audio_context.create_media_stream_source(media_stream)
            .map_err(|e| format!("Failed to create audio source: {:?}", e))
    }

    /// Connect microphone input to audio worklet
    fn connect_microphone(&mut self, microphone_source: &web_sys::AudioNode) -> Result<(), audio::audio_error::AudioError> {
        // Get the output_to_speakers value first to avoid borrowing conflicts
        let output_to_speakers = self.output_to_speakers;
        
        // Set up audio routing through the pipeline
        self.audio_pipeline.connect_microphone(microphone_source, output_to_speakers)?;
        
        // Connect microphone gain to volume detector (parallel tap for analysis)
        let mic_gain = &self.audio_pipeline.microphone_gain_node;
        if let Err(e) = self.audioworklet_manager.volume_detector.borrow().connect_source(mic_gain) {
            crate::common::dev_log!("Failed to connect microphone gain to VolumeDetector: {:?}", e);
        } else {
            crate::common::dev_log!("Connected microphone gain to VolumeDetector");
        }
        
        Ok(())
    }

    /// Connect a MediaStreamAudioSourceNode to the audio worklet
    pub fn connect_media_stream_to_audioworklet(
        &mut self,
        source: &web_sys::MediaStreamAudioSourceNode,
    ) -> Result<(), String> {
        let result = self.connect_microphone(source.as_ref())
            .map_err(|e| e.to_string());
        
        match result {
            Ok(_) => {
                if !self.audioworklet_manager.is_processing() {
                    let _ = self.audioworklet_manager.start_processing();
                }
                
                Ok(())
            }
            Err(e) => {
                Err(format!("Failed to connect microphone: {:?}", e))
            }
        }
    }
    
}