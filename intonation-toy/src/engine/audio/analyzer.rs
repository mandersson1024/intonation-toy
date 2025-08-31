use web_sys::AnalyserNode;
use crate::common::dev_log;
use super::{
    AudioError,
    volume_detector::VolumeDetector,
    pitch_analyzer::PitchAnalyzer,
    data_types::{VolumeAnalysis, VolumeLevelData, PitchData},
};
use std::rc::Rc;
use std::cell::RefCell;

/// AudioAnalyzer manages all audio analysis operations
/// 
/// This component is responsible for:
/// - Volume detection and analysis
/// - Pitch detection and analysis  
/// - Using the analyser node from the AudioSignalFlow
/// - Providing clean analysis results to the pipeline
/// 
/// The analyzer receives audio data and provides processed analysis results,
/// keeping all analysis logic centralized and focused.
pub struct AudioAnalyzer {
    analyser_node: AnalyserNode,
    volume_detector: Option<Rc<RefCell<VolumeDetector>>>,
    pitch_analyzer: Option<Rc<RefCell<PitchAnalyzer>>>,
    last_volume_analysis: Option<VolumeAnalysis>,
}

impl AudioAnalyzer {
    /// Creates a new AudioAnalyzer
    /// 
    /// # Parameters
    /// - `analyser_node`: The AnalyserNode from the AudioSignalFlow
    /// 
    /// # Returns
    /// Result containing the configured AudioAnalyzer or error description
    pub fn new(analyser_node: AnalyserNode) -> Result<Self, String> {
        dev_log!("Creating AudioAnalyzer with signal flow analyser node");
        
        Ok(Self {
            analyser_node,
            volume_detector: None,
            pitch_analyzer: None,
            last_volume_analysis: None,
        })
    }

    /// Set volume detector for audio analysis
    /// 
    /// The volume detector will be connected to the analyser node from the signal flow
    /// and used to perform volume analysis on the audio stream.
    /// 
    /// # Parameters
    /// - `detector`: VolumeDetector instance to use for analysis
    pub fn set_volume_detector(&mut self, detector: VolumeDetector) -> Result<(), AudioError> {
        // Wrap the detector in Rc<RefCell<>> for shared ownership
        let detector_rc = Rc::new(RefCell::new(detector));
        
        // Connect the volume detector to the analyser node from signal flow
        if let Err(e) = detector_rc.borrow().connect_source(&self.analyser_node) {
            return Err(AudioError::Generic(format!("Failed to connect volume detector to analyser node: {:?}", e)));
        }
        
        self.volume_detector = Some(detector_rc);
        dev_log!("✓ Volume detector connected to analyser node from signal flow");
        
        Ok(())
    }

    /// Set pitch analyzer for audio analysis
    /// 
    /// The pitch analyzer will process audio samples to detect pitch information.
    /// 
    /// # Parameters  
    /// - `analyzer`: Shared reference to PitchAnalyzer instance
    pub fn set_pitch_analyzer(&mut self, analyzer: Rc<RefCell<PitchAnalyzer>>) {
        self.pitch_analyzer = Some(analyzer);
        dev_log!("✓ Pitch analyzer configured for audio analysis");
    }

    /// Perform volume analysis
    /// 
    /// This method triggers volume analysis using the connected volume detector
    /// and stores the results for later retrieval.
    /// 
    /// # Returns
    /// Result indicating success or failure of the analysis
    pub fn analyze_volume(&mut self) -> Result<(), AudioError> {
        if let Some(ref volume_detector) = self.volume_detector {
            match volume_detector.borrow_mut().analyze() {
                Ok(volume_analysis) => {
                    self.last_volume_analysis = Some(volume_analysis);
                    Ok(())
                }
                Err(err) => {
                    Err(AudioError::Generic(format!("Volume analysis failed: {:?}", err)))
                }
            }
        } else {
            Err(AudioError::Generic("No volume detector configured".to_string()))
        }
    }

    /// Perform pitch analysis on audio samples
    /// 
    /// This method processes the provided audio samples through the pitch analyzer
    /// to detect pitch information.
    /// 
    /// # Parameters
    /// - `audio_samples`: Array of audio samples to analyze
    /// 
    /// # Returns
    /// Result containing optional pitch detection result
    pub fn analyze_pitch(&mut self, audio_samples: &[f32]) -> Result<Option<()>, AudioError> {
        if let Some(ref pitch_analyzer) = self.pitch_analyzer {
            match pitch_analyzer.borrow_mut().analyze_samples(audio_samples) {
                Ok(Some(_pitch_result)) => {
                    // Pitch data is stored in the analyzer and can be retrieved via get_pitch_data
                    Ok(Some(()))
                }
                Ok(None) => {
                    // No pitch detected, which is normal for silence or noise
                    Ok(None)
                }
                Err(e) => {
                    Err(AudioError::Generic(format!("Pitch analysis error: {}", e)))
                }
            }
        } else {
            Err(AudioError::Generic("No pitch analyzer configured".to_string()))
        }
    }

    /// Perform complete audio analysis
    /// 
    /// This is a convenience method that performs both volume and pitch analysis
    /// in a single call. Audio samples are required for pitch analysis.
    /// 
    /// # Parameters
    /// - `audio_samples`: Array of audio samples for pitch analysis
    /// 
    /// # Returns
    /// Result indicating success or failure of the combined analysis
    pub fn analyze_all(&mut self, audio_samples: &[f32]) -> Result<(), AudioError> {
        // Perform volume analysis (uses the analyser node connection)
        if let Err(e) = self.analyze_volume() {
            dev_log!("Volume analysis failed: {:?}", e);
        }

        // Perform pitch analysis (uses the provided samples)
        if let Err(e) = self.analyze_pitch(audio_samples) {
            dev_log!("Pitch analysis failed: {:?}", e);
        }

        Ok(())
    }

    /// Get current volume data if available
    /// 
    /// Returns the results of the most recent volume analysis, if any.
    /// 
    /// # Returns
    /// Optional VolumeLevelData containing analysis results
    pub fn get_volume_data(&self) -> Option<VolumeLevelData> {
        self.last_volume_analysis.as_ref().map(|analysis| {
            VolumeLevelData {
                rms_amplitude: analysis.rms_amplitude,
                peak_amplitude: analysis.peak_amplitude,
                fft_data: analysis.fft_data.clone(),
            }
        })
    }

    /// Get current pitch data if available
    /// 
    /// Returns the results of the most recent pitch analysis from the pitch analyzer.
    /// 
    /// # Returns  
    /// Optional PitchData containing pitch detection results
    pub fn get_pitch_data(&self) -> Option<PitchData> {
        self.pitch_analyzer.as_ref()
            .and_then(|analyzer| analyzer.try_borrow().ok())
            .and_then(|borrowed| borrowed.get_latest_pitch_data())
    }

    /// Get reference to the volume detector for external access
    /// 
    /// This allows other components to access the volume detector if needed,
    /// while keeping the analyzer as the primary interface for analysis operations.
    /// 
    /// # Returns
    /// Optional shared reference to the VolumeDetector
    pub fn get_volume_detector(&self) -> Option<Rc<RefCell<VolumeDetector>>> {
        self.volume_detector.clone()
    }

    /// Get reference to the pitch analyzer for external access
    /// 
    /// This allows other components to access the pitch analyzer if needed,
    /// while keeping the analyzer as the primary interface for analysis operations.
    /// 
    /// # Returns
    /// Optional shared reference to the PitchAnalyzer
    pub fn get_pitch_analyzer(&self) -> Option<Rc<RefCell<PitchAnalyzer>>> {
        self.pitch_analyzer.clone()
    }

    /// Get reference to the analyser node
    /// 
    /// Provides access to the Web Audio API AnalyserNode for advanced use cases.
    /// 
    /// # Returns
    /// Reference to the AnalyserNode from the signal flow
    pub fn get_analyser_node(&self) -> &AnalyserNode {
        &self.analyser_node
    }

    /// Disconnect and cleanup the audio analyzer
    /// 
    /// This method cleans up connections and prepares the analyzer for disposal.
    /// 
    /// # Returns
    /// Result indicating success or failure of the cleanup
    pub fn disconnect(&mut self) -> Result<(), AudioError> {
        // Disconnect volume detector if present
        if let Some(ref volume_detector) = self.volume_detector {
            if let Err(e) = volume_detector.borrow().disconnect() {
                dev_log!("Failed to disconnect VolumeDetector: {:?}", e);
            } else {
                dev_log!("✓ VolumeDetector disconnected");
            }
        }
        
        // Clear stored data
        self.last_volume_analysis = None;
        
        dev_log!("✓ AudioAnalyzer disconnected and cleaned up");
        Ok(())
    }
}

impl Drop for AudioAnalyzer {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}