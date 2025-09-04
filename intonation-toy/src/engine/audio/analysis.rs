use super::data_types::VolumeAnalysis;

pub fn analyze(time_domain_data: &[f32]) -> VolumeAnalysis {
    VolumeAnalysis {
        peak_amplitude: get_peak_amplitude_from_time_domain(time_domain_data),
        rms_amplitude: calculate_rms_amplitude_from_time_domain(time_domain_data),
    }
}

/// Calculates peak amplitude directly from time domain data
/// 
/// Uses getFloatTimeDomainData() which provides direct amplitude values
/// in the range -1.0 to 1.0. Returns the absolute maximum value.
fn get_peak_amplitude_from_time_domain(time_domain_data: &[f32]) -> f32 {
    // Find the maximum absolute value in time domain data
    time_domain_data
        .iter()
        .map(|&sample| sample.abs())
        .fold(0.0f32, f32::max)
}

/// Calculates RMS amplitude from time domain data
/// 
/// Root Mean Square provides the effective amplitude over the time window.
fn calculate_rms_amplitude_from_time_domain(time_domain_data: &[f32]) -> f32 {
    // Calculate sum of squares
    let sum_of_squares: f32 = time_domain_data
        .iter()
        .map(|&sample| sample * sample)
        .sum();
    
    // Calculate RMS: square root of mean of squares
    (sum_of_squares / time_domain_data.len() as f32).sqrt()
}