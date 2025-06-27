// Playful Theme - Vertex Shader
// This shader provides vibrant, dynamic vertex transformations for the Playful theme

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
}

struct VertexOutput {
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

// Playful theme constants
const ANIMATION_SPEED: f32 = 1.2;
const WAVE_AMPLITUDE: f32 = 0.3;
const COLOR_CYCLE_SPEED: f32 = 2.0;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Dynamic position based on audio energy and time
    var animated_position = input.position;
    
    // Wave effect based on audio energy
    let wave_offset = sin(uniforms.time * ANIMATION_SPEED + input.position.x * 5.0) 
                     * uniforms.audio_energy * WAVE_AMPLITUDE;
    animated_position.y += wave_offset;
    
    // Pulsing effect based on audio frequency
    let pulse_scale = 1.0 + uniforms.audio_energy * 0.2;
    animated_position *= pulse_scale;
    
    // Transform to world space
    let world_position = uniforms.model * vec4<f32>(animated_position, 1.0);
    output.world_position = world_position.xyz;
    
    // Transform to clip space
    output.clip_position = uniforms.view_proj * world_position;
    
    // Pass through normal (TODO: transform by normal matrix)
    output.normal = input.normal;
    
    // Pass through UV coordinates
    output.uv = input.uv;
    
    // Dynamic color cycling for playful effect
    let time_factor = uniforms.time * COLOR_CYCLE_SPEED;
    let color_shift = vec3<f32>(
        sin(time_factor) * 0.5 + 0.5,
        sin(time_factor + 2.094) * 0.5 + 0.5,  // 2π/3 offset
        sin(time_factor + 4.188) * 0.5 + 0.5   // 4π/3 offset
    );
    
    // Blend original color with dynamic color cycling
    output.color = vec4<f32>(
        mix(input.color.rgb, color_shift, uniforms.audio_energy * 0.7),
        input.color.a
    );
    
    // Pass audio energy for fragment shader
    output.audio_energy = uniforms.audio_energy;
    
    return output;
}