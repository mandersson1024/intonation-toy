// Scientific Theme - Vertex Shader
// This shader provides precise, analytical vertex transformations for the Scientific theme

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

// Scientific theme constants
const PRECISION_SCALE: f32 = 1000.0;
const MEASUREMENT_ACCURACY: f32 = 0.001;
const GRID_ALIGNMENT: f32 = 0.1;

// Quantize value to measurement precision
fn quantize(value: f32, precision: f32) -> f32 {
    return round(value / precision) * precision;
}

// Align to grid for scientific precision
fn grid_align(position: vec3<f32>, grid_size: f32) -> vec3<f32> {
    return vec3<f32>(
        round(position.x / grid_size) * grid_size,
        round(position.y / grid_size) * grid_size,
        round(position.z / grid_size) * grid_size
    );
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Precise position with minimal animation for stability
    var analytical_position = input.position;
    
    // Controlled displacement based on audio data (scientific visualization)
    let audio_displacement = quantize(uniforms.audio_energy, MEASUREMENT_ACCURACY) * 0.1;
    
    // Only apply displacement to Y axis for clean frequency visualization
    analytical_position.y += audio_displacement;
    
    // Grid alignment for measurement precision
    analytical_position = grid_align(analytical_position, GRID_ALIGNMENT);
    
    // Transform to world space
    let world_position = uniforms.model * vec4<f32>(analytical_position, 1.0);
    output.world_position = world_position.xyz;
    
    // Transform to clip space
    output.clip_position = uniforms.view_proj * world_position;
    
    // Pass through normal (TODO: transform by normal matrix)
    output.normal = normalize(input.normal);
    
    // Pass through UV coordinates
    output.uv = input.uv;
    
    // Preserve original color with slight energy-based modulation
    let energy_modulation = 1.0 + uniforms.audio_energy * 0.1;
    output.color = vec4<f32>(input.color.rgb * energy_modulation, input.color.a);
    
    // Package audio data for precise analysis in fragment shader
    output.audio_data = vec4<f32>(
        quantize(uniforms.audio_energy, MEASUREMENT_ACCURACY),
        quantize(uniforms.audio_frequency, MEASUREMENT_ACCURACY),
        quantize(uniforms.time, MEASUREMENT_ACCURACY), // Phase information
        quantize(length(analytical_position), MEASUREMENT_ACCURACY) // Amplitude
    );
    
    return output;
}