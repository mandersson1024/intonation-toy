
/// Volume level data for external consumption
/// 
/// Contains RMS and peak amplitude measurements, and optionally FFT frequency data.
/// The FFT data is normalized to 0.0-1.0 range and contains frequency bin magnitudes.
/// FFT data is None for traditional volume detection and Some(Vec<f32>) for analyser-based detection.
#[derive(Debug, Clone, PartialEq)]
pub struct VolumeLevelData {
    pub rms_amplitude: f32,
    pub peak_amplitude: f32,
    pub fft_data: Option<Vec<f32>>,
}

/// Internal volume analysis result from volume detection
/// 
/// Contains raw amplitude measurements from volume detection.
/// This is used internally by volume detectors before converting to VolumeLevelData for external consumption.
#[derive(Debug, Clone)]
pub struct VolumeAnalysis {
    pub peak_amplitude: f32,
    pub rms_amplitude: f32,
}

