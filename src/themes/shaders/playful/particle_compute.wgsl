// Playful Theme - Particle Compute Shader
// This compute shader manages dynamic particle systems for the Playful theme

struct Particle {
    position: vec3<f32>,
    velocity: vec3<f32>,
    color: vec4<f32>,
    life: f32,
    size: f32,
    _padding: vec2<f32>, // Align to 16 bytes
}

struct ParticleSystem {
    particle_count: u32,
    emission_rate: f32,
    time: f32,
    delta_time: f32,
    audio_energy: f32,
    audio_frequency: f32,
    _padding: vec2<f32>, // Align to 16 bytes
}

@group(0) @binding(0)
var<storage, read_write> particles: array<Particle>;

@group(0) @binding(1)
var<uniform> system: ParticleSystem;

// Playful theme constants
const GRAVITY: vec3<f32> = vec3<f32>(0.0, -0.5, 0.0);
const ENERGY_BOOST: f32 = 2.0;
const COLOR_CYCLING_SPEED: f32 = 3.0;
const RAINBOW_INTENSITY: f32 = 0.8;

// Random number generation (simple hash)
fn hash(seed: u32) -> f32 {
    var x = seed;
    x = ((x >> 16u) ^ x) * 0x45d9f3bu;
    x = ((x >> 16u) ^ x) * 0x45d9f3bu;
    x = (x >> 16u) ^ x;
    return f32(x) * (1.0 / 4294967296.0);
}

// Generate rainbow color based on time and energy
fn generate_rainbow_color(t: f32, energy: f32) -> vec4<f32> {
    let hue = fract(t * COLOR_CYCLING_SPEED);
    let saturation = 0.8 + energy * 0.2;
    let value = 0.7 + energy * 0.3;
    
    // Simple HSV to RGB conversion
    let c = value * saturation;
    let x = c * (1.0 - abs((hue * 6.0) % 2.0 - 1.0));
    let m = value - c;
    
    var rgb: vec3<f32>;
    if (hue < 1.0/6.0) {
        rgb = vec3<f32>(c, x, 0.0);
    } else if (hue < 2.0/6.0) {
        rgb = vec3<f32>(x, c, 0.0);
    } else if (hue < 3.0/6.0) {
        rgb = vec3<f32>(0.0, c, x);
    } else if (hue < 4.0/6.0) {
        rgb = vec3<f32>(0.0, x, c);
    } else if (hue < 5.0/6.0) {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }
    
    return vec4<f32>(rgb + vec3<f32>(m), 1.0);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= system.particle_count) {
        return;
    }
    
    var particle = particles[index];
    
    // Update particle life
    particle.life -= system.delta_time;
    
    // Reset particle if dead
    if (particle.life <= 0.0) {
        // Respawn particle with audio-driven parameters
        let seed = index + u32(system.time * 1000.0);
        particle.position = vec3<f32>(
            (hash(seed) - 0.5) * 4.0,
            hash(seed + 1u) * 2.0 - 1.0,
            (hash(seed + 2u) - 0.5) * 4.0
        );
        
        // Audio-influenced velocity
        let energy_boost = system.audio_energy * ENERGY_BOOST;
        particle.velocity = vec3<f32>(
            (hash(seed + 3u) - 0.5) * (1.0 + energy_boost),
            hash(seed + 4u) * (2.0 + energy_boost),
            (hash(seed + 5u) - 0.5) * (1.0 + energy_boost)
        );
        
        // Life influenced by audio frequency
        particle.life = 3.0 + system.audio_frequency * 2.0;
        
        // Size influenced by audio energy
        particle.size = 0.1 + system.audio_energy * 0.7;
        
        // Rainbow color based on time and audio
        particle.color = generate_rainbow_color(system.time + f32(index) * 0.1, system.audio_energy);
    }
    
    // Physics update
    particle.velocity += GRAVITY * system.delta_time;
    particle.position += particle.velocity * system.delta_time;
    
    // Audio-reactive color cycling
    let color_phase = system.time * COLOR_CYCLING_SPEED + f32(index) * 0.1;
    particle.color = mix(
        particle.color,
        generate_rainbow_color(color_phase, system.audio_energy),
        system.audio_energy * 0.3 * system.delta_time
    );
    
    // Size pulsing based on audio
    let audio_pulse = sin(system.time * 10.0) * system.audio_energy * 0.2;
    particle.size = max(0.05, particle.size * (1.0 + audio_pulse));
    
    // Fade alpha based on life
    let life_ratio = particle.life / 3.0;
    particle.color.a = life_ratio * (0.7 + system.audio_energy * 0.3);
    
    particles[index] = particle;
}