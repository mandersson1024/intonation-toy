// Playful Theme - Fragment Shader
// This shader provides vibrant, dynamic fragment shading for the Playful theme

struct FragmentInput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) audio_energy: f32,
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

// Playful theme color palette
const PRIMARY_COLOR: vec3<f32> = vec3<f32>(1.0, 0.3, 0.5);     // Bright pink
const SECONDARY_COLOR: vec3<f32> = vec3<f32>(0.2, 0.8, 1.0);   // Cyan
const ACCENT_COLOR: vec3<f32> = vec3<f32>(1.0, 0.8, 0.2);      // Orange

// Animation constants
const RAINBOW_SPEED: f32 = 0.5;
const ENERGY_BOOST: f32 = 1.5;
const SHIMMER_FREQUENCY: f32 = 10.0;

// HSV to RGB conversion for rainbow effects
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    let c = v * s;
    let x = c * (1.0 - abs((h * 6.0) % 2.0 - 1.0));
    let m = v - c;
    
    var rgb: vec3<f32>;
    if (h < 1.0/6.0) {
        rgb = vec3<f32>(c, x, 0.0);
    } else if (h < 2.0/6.0) {
        rgb = vec3<f32>(x, c, 0.0);
    } else if (h < 3.0/6.0) {
        rgb = vec3<f32>(0.0, c, x);
    } else if (h < 4.0/6.0) {
        rgb = vec3<f32>(0.0, x, c);
    } else if (h < 5.0/6.0) {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }
    
    return rgb + vec3<f32>(m);
}

// Generate rainbow color based on position and time
fn generate_rainbow_color(position: vec2<f32>, time: f32) -> vec3<f32> {
    let hue = fract(time * RAINBOW_SPEED + position.x * 0.5 + position.y * 0.3);
    return hsv_to_rgb(hue, 0.8, 1.0);
}

// Shimmer effect for high-energy moments
fn apply_shimmer(color: vec3<f32>, position: vec2<f32>, time: f32, energy: f32) -> vec3<f32> {
    let shimmer_phase = sin(time * SHIMMER_FREQUENCY + position.x * 20.0 + position.y * 15.0);
    let shimmer_intensity = energy * 0.3 * (shimmer_phase * 0.5 + 0.5);
    return color + vec3<f32>(shimmer_intensity);
}

@fragment
fn fs_main(input: FragmentInput) -> @location(0) vec4<f32> {
    // Base color from vertex shader (already animated)
    var final_color = input.color.rgb;
    
    // Add rainbow effect based on audio energy
    if (input.audio_energy > 0.3) {
        let rainbow_color = generate_rainbow_color(input.uv, uniforms.time);
        let rainbow_mix = (input.audio_energy - 0.3) / 0.7; // Normalize to 0-1
        final_color = mix(final_color, rainbow_color, rainbow_mix * 0.6);
    }
    
    // Apply energy-based color boosting
    final_color *= (1.0 + input.audio_energy * ENERGY_BOOST);
    
    // Add shimmer effect for high energy
    if (input.audio_energy > 0.6) {
        final_color = apply_shimmer(final_color, input.uv, uniforms.time, input.audio_energy);
    }
    
    // Gradient overlay based on position
    let gradient_factor = input.uv.y;
    let gradient_color = mix(PRIMARY_COLOR, SECONDARY_COLOR, gradient_factor);
    final_color = mix(final_color, gradient_color, 0.2);
    
    // Pulsing brightness based on audio frequency
    let frequency_pulse = sin(uniforms.time * 5.0) * uniforms.audio_frequency * 0.1;
    final_color += vec3<f32>(frequency_pulse);
    
    // Ensure colors stay in valid range but allow overbright for bloom
    final_color = max(final_color, vec3<f32>(0.0));
    
    // Alpha blending for particle effects
    let alpha = input.color.a * (0.7 + input.audio_energy * 0.3);
    
    return vec4<f32>(final_color, alpha);
}