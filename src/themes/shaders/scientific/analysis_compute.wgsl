// Scientific Theme - Analysis Compute Shader
// This compute shader performs precise audio analysis for the Scientific theme

struct AudioSample {
    amplitude: f32,
    phase: f32,
    frequency: f32,
    _padding: f32, // Align to 16 bytes
}

struct AnalysisData {
    sample_count: u32,
    sample_rate: f32,
    time: f32,
    analysis_window: f32,
    frequency_resolution: f32,
    amplitude_threshold: f32,
    _padding: vec2<f32>, // Align to 16 bytes
}

struct FrequencyBin {
    frequency: f32,
    magnitude: f32,
    phase: f32,
    confidence: f32,
}

@group(0) @binding(0)
var<storage, read> input_samples: array<AudioSample>;

@group(0) @binding(1)
var<storage, read_write> frequency_bins: array<FrequencyBin>;

@group(0) @binding(2)
var<uniform> analysis: AnalysisData;

// Scientific analysis constants
const PI: f32 = 3.14159265359;
const TWO_PI: f32 = 6.28318530718;
const PRECISION_THRESHOLD: f32 = 0.001;
const CONFIDENCE_THRESHOLD: f32 = 0.1;
const MAX_FREQUENCY_BINS: u32 = 512u;

// Hann window function for spectral analysis
fn hann_window(n: u32, N: u32) -> f32 {
    return 0.5 * (1.0 - cos(TWO_PI * f32(n) / f32(N - 1u)));
}

// Complex number structure for FFT
struct Complex {
    real: f32,
    imag: f32,
}

// Complex multiplication
fn complex_mul(a: Complex, b: Complex) -> Complex {
    return Complex(
        a.real * b.real - a.imag * b.imag,
        a.real * b.imag + a.imag * b.real
    );
}

// Complex addition
fn complex_add(a: Complex, b: Complex) -> Complex {
    return Complex(a.real + b.real, a.imag + b.imag);
}

// Calculate magnitude of complex number
fn complex_magnitude(c: Complex) -> f32 {
    return sqrt(c.real * c.real + c.imag * c.imag);
}

// Calculate phase of complex number
fn complex_phase(c: Complex) -> f32 {
    return atan2(c.imag, c.real);
}

// Discrete Fourier Transform (simplified for demonstration)
fn dft_bin(k: u32) -> Complex {
    var result = Complex(0.0, 0.0);
    let N = min(analysis.sample_count, 1024u); // Limit for performance
    
    for (var n = 0u; n < N; n = n + 1u) {
        if (n >= analysis.sample_count) {
            break;
        }
        
        let sample = input_samples[n];
        let angle = -TWO_PI * f32(k) * f32(n) / f32(N);
        let window = hann_window(n, N);
        
        let weighted_amplitude = sample.amplitude * window;
        let twiddle = Complex(cos(angle), sin(angle));
        let weighted_sample = Complex(weighted_amplitude, 0.0);
        
        result = complex_add(result, complex_mul(weighted_sample, twiddle));
    }
    
    return result;
}

// Calculate statistical confidence for frequency bin
fn calculate_confidence(magnitude: f32, noise_floor: f32) -> f32 {
    if (magnitude < noise_floor) {
        return 0.0;
    }
    
    let snr = magnitude / noise_floor;
    return min(1.0, snr / 10.0); // Normalize to 0-1 range
}

// Estimate noise floor from surrounding bins
fn estimate_noise_floor(bin_index: u32) -> f32 {
    var sum = 0.0;
    var count = 0u;
    let window_size = 5u;
    
    for (var i = 0u; i < window_size; i = i + 1u) {
        if (bin_index + i < MAX_FREQUENCY_BINS && bin_index + i < arrayLength(&frequency_bins)) {
            sum += frequency_bins[bin_index + i].magnitude;
            count = count + 1u;
        }
        if (bin_index >= i && bin_index - i < arrayLength(&frequency_bins)) {
            sum += frequency_bins[bin_index - i].magnitude;
            count = count + 1u;
        }
    }
    
    return if (count > 0u) { sum / f32(count) * 0.1 } else { 0.01 };
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let bin_index = global_id.x;
    if (bin_index >= MAX_FREQUENCY_BINS || bin_index >= arrayLength(&frequency_bins)) {
        return;
    }
    
    // Calculate frequency for this bin
    let frequency = f32(bin_index) * analysis.frequency_resolution;
    
    // Perform DFT for this frequency bin
    let dft_result = dft_bin(bin_index);
    let magnitude = complex_magnitude(dft_result);
    let phase = complex_phase(dft_result);
    
    // Apply precision threshold for scientific accuracy
    let quantized_magnitude = if (magnitude < PRECISION_THRESHOLD) { 0.0 } else { magnitude };
    let quantized_phase = round(phase * 1000.0) / 1000.0; // 3 decimal places
    
    // Calculate measurement confidence
    let noise_floor = estimate_noise_floor(bin_index);
    let confidence = calculate_confidence(quantized_magnitude, noise_floor);
    
    // Only store results above confidence threshold
    if (confidence >= CONFIDENCE_THRESHOLD) {
        frequency_bins[bin_index] = FrequencyBin(
            frequency,
            quantized_magnitude,
            quantized_phase,
            confidence
        );
    } else {
        // Mark as invalid measurement
        frequency_bins[bin_index] = FrequencyBin(
            frequency,
            0.0,
            0.0,
            0.0
        );
    }
}