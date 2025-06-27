// Scientific Theme - Fragment Shader  
// This shader provides precise, analytical fragment shading for the Scientific theme

struct FragmentInput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) audio_data: vec4<f32>, // x: energy, y: frequency, z: phase, w: amplitude
}

struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    time: f32,
    audio_energy: f32,
    audio_frequency: f32,
    audio_waveform: array<f32, 256>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// Scientific theme color palette
const PRIMARY_COLOR: vec3<f32> = vec3<f32>(0.2, 0.6, 0.9);     // Professional blue
const SECONDARY_COLOR: vec3<f32> = vec3<f32>(0.1, 0.8, 0.6);   // Teal
const ACCENT_COLOR: vec3<f32> = vec3<f32>(0.9, 0.6, 0.2);      // Orange highlight
const GRID_COLOR: vec3<f32> = vec3<f32>(0.3, 0.4, 0.5);        // Grid lines

// Scientific visualization constants
const GRID_LINE_WIDTH: f32 = 0.01;
const MEASUREMENT_THRESHOLD: f32 = 0.1;
const COLOR_ACCURACY: f32 = 0.02;
const FREQUENCY_BANDS: i32 = 16;

// Color mapping for frequency analysis
fn frequency_to_color(frequency: f32) -> vec3<f32> {
    // Map frequency to color spectrum scientifically
    let normalized_freq = clamp(frequency, 0.0, 1.0);
    
    if (normalized_freq < 0.25) {
        // Low frequencies: Blue to Cyan
        return mix(PRIMARY_COLOR, SECONDARY_COLOR, normalized_freq * 4.0);
    } else if (normalized_freq < 0.75) {
        // Mid frequencies: Cyan to White
        return mix(SECONDARY_COLOR, vec3<f32>(1.0), (normalized_freq - 0.25) * 2.0);
    } else {
        // High frequencies: White to Orange
        return mix(vec3<f32>(1.0), ACCENT_COLOR, (normalized_freq - 0.75) * 4.0);
    }
}

// Generate analytical grid overlay
fn generate_grid(uv: vec2<f32>) -> f32 {
    let grid_x = abs(fract(uv.x * 10.0) - 0.5);
    let grid_y = abs(fract(uv.y * 10.0) - 0.5);
    
    let line_x = smoothstep(0.0, GRID_LINE_WIDTH, grid_x);
    let line_y = smoothstep(0.0, GRID_LINE_WIDTH, grid_y);
    
    return 1.0 - max(line_x, line_y);
}

// Measurement annotation visibility
fn should_show_annotation(value: f32) -> bool {
    return value > MEASUREMENT_THRESHOLD;
}

// Precision color mapping with error bounds
fn apply_color_accuracy(color: vec3<f32>) -> vec3<f32> {
    // Quantize colors to ensure measurement accuracy
    return vec3<f32>(
        round(color.r / COLOR_ACCURACY) * COLOR_ACCURACY,
        round(color.g / COLOR_ACCURACY) * COLOR_ACCURACY,
        round(color.b / COLOR_ACCURACY) * COLOR_ACCURACY
    );
}

@fragment
fn fs_main(input: FragmentInput) -> @location(0) vec4<f32> {
    // Extract quantized audio data
    let energy = input.audio_data.x;
    let frequency = input.audio_data.y;
    let phase = input.audio_data.z;
    let amplitude = input.audio_data.w;
    
    // Base scientific color mapping
    var analytical_color = frequency_to_color(frequency);
    
    // Energy-based intensity (linear, not exponential for scientific accuracy)
    analytical_color *= (0.7 + energy * 0.3);
    
    // Grid overlay for measurement reference
    let grid_intensity = generate_grid(input.uv);
    if (grid_intensity > 0.5) {
        analytical_color = mix(analytical_color, GRID_COLOR, 0.3);
    }
    
    // Amplitude visualization through brightness
    if (should_show_annotation(amplitude)) {
        analytical_color *= (1.0 + amplitude * 0.2);
    }
    
    // Phase visualization through slight hue shift (minimal for accuracy)
    let phase_shift = sin(phase) * 0.05;
    analytical_color.r += phase_shift;
    analytical_color.b -= phase_shift;
    
    // Apply color accuracy constraints
    analytical_color = apply_color_accuracy(analytical_color);
    
    // Frequency band highlighting
    let band_index = i32(frequency * f32(FREQUENCY_BANDS));
    if (band_index % 2 == 0) {
        // Subtle alternating band visualization
        analytical_color *= 1.05;
    }
    
    // Maintain scientific precision - no overbright colors
    analytical_color = clamp(analytical_color, vec3<f32>(0.0), vec3<f32>(1.0));
    
    // Alpha based on measurement confidence
    let confidence = 1.0 - abs(energy - 0.5) * 0.4; // Higher confidence near medium energy
    let alpha = input.color.a * confidence;
    
    return vec4<f32>(analytical_color, alpha);
}