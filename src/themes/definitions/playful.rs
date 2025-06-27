//! # Playful Theme Definition
//!
//! The Playful theme provides a vibrant, dynamic audio visualization experience
//! with energetic colors, dynamic animations, and engaging particle effects.
//! It's designed to make audio visualization fun and engaging.

use crate::modules::presentation_layer::theme_manager::*;

/// Playful theme definition with vibrant colors and dynamic animations
pub const PLAYFUL_THEME: ThemeDefinition = ThemeDefinition {
    name: "playful",
    display_name: "Playful",
    description: "Vibrant colors and dynamic animations for an engaging experience",
    
    color_palette: ColorPalette {
        primary: [1.0, 0.3, 0.5, 1.0],        // Bright pink
        secondary: [0.2, 0.8, 1.0, 1.0],      // Cyan
        accent: [1.0, 0.8, 0.2, 1.0],         // Orange
        background: [0.1, 0.1, 0.2, 1.0],     // Dark blue
        surface: [0.15, 0.15, 0.25, 1.0],     // Lighter dark blue
        text: [1.0, 1.0, 1.0, 1.0],           // White
        gradient_start: [1.0, 0.3, 0.5, 1.0], // Pink
        gradient_end: [0.2, 0.8, 1.0, 1.0],   // Cyan
    },
    
    material_properties: MaterialProperties {
        metallic: 0.1,
        roughness: 0.3,
        emissive: [0.2, 0.1, 0.3],
        opacity: 0.9,
        refraction_index: 1.4,
    },
    
    lighting_config: LightingRig {
        ambient_intensity: 0.4,
        ambient_color: [0.8, 0.9, 1.0],
        directional_intensity: 0.8,
        directional_color: [1.0, 0.9, 0.8],
        directional_direction: [-0.5, -0.7, -0.3],
        point_lights: &[
            PointLight {
                position: [2.0, 3.0, 1.0],
                color: [1.0, 0.5, 0.8],
                intensity: 1.5,
                range: 10.0,
            },
            PointLight {
                position: [-1.0, 2.0, 2.0],
                color: [0.2, 0.8, 1.0],
                intensity: 1.2,
                range: 8.0,
            },
        ],
    },
    
    shader_variants: ShaderSet {
        vertex_shader: "shaders/playful/vertex.wgsl",
        fragment_shader: "shaders/playful/fragment.wgsl",
        compute_shaders: &["shaders/playful/particle_compute.wgsl", "shaders/playful/rainbow_effect.wgsl"],
        shader_defines: &[
            ("PLAYFUL_MODE", "1"),
            ("DYNAMIC_COLORS", "1"),
            ("RAINBOW_EFFECTS", "1"),
            ("HIGH_ENERGY_MODE", "1"),
        ],
    },
    
    particle_systems: ParticleConfig {
        max_particles: 2000,
        emission_rate: 50.0,
        particle_lifetime: 3.0,
        size_range: (0.1, 0.8),
        velocity_range: (0.5, 2.0),
        color_over_lifetime: &[
            [1.0, 0.3, 0.5, 1.0], // Birth: bright pink
            [0.8, 0.6, 0.9, 0.9], // Early: purple
            [0.2, 0.8, 1.0, 0.8], // Mid: cyan
            [1.0, 0.8, 0.2, 0.6], // Late: orange
            [1.0, 0.5, 0.8, 0.0], // Death: pink, transparent
        ],
    },
    
    animation_timings: AnimationConfig {
        transition_duration: 0.8,
        easing_curve: EasingCurve::EaseInOut,
        loop_behavior: LoopBehavior::Repeat,
        playback_speed: 1.2,
    },
    
    post_effects: EffectChain {
        bloom: Some(BloomEffect {
            intensity: 0.8,
            threshold: 0.6,
            radius: 1.2,
        }),
        tone_mapping: Some(ToneMappingEffect {
            exposure: 1.1,
            gamma: 2.2,
            contrast: 1.1,
        }),
        color_grading: Some(ColorGradingEffect {
            saturation: 1.3,
            hue_shift: 0.1, // Slight hue shift for more vibrant colors
            brightness: 1.05,
        }),
        vignette: Some(VignetteEffect {
            intensity: 0.3,
            radius: 0.8,
            softness: 0.5,
        }),
    },
};

/// Configuration constants specific to the Playful theme
pub mod config {
    /// Animation speed multiplier for playful theme
    pub const ANIMATION_SPEED_MULTIPLIER: f32 = 1.2;
    
    /// Color cycling rate in Hz
    pub const COLOR_CYCLING_RATE: f32 = 0.5;
    
    /// Particle burst intensity multiplier
    pub const PARTICLE_BURST_MULTIPLIER: f32 = 1.5;
    
    /// Rainbow effect cycle duration in seconds
    pub const RAINBOW_CYCLE_DURATION: f32 = 4.0;
    
    /// Energy threshold for high-energy visual effects
    pub const HIGH_ENERGY_THRESHOLD: f32 = 0.7;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_playful_theme_definition() {
        assert_eq!(PLAYFUL_THEME.name, "playful");
        assert_eq!(PLAYFUL_THEME.display_name, "Playful");
        assert!(!PLAYFUL_THEME.description.is_empty());
        
        // Validate vibrant color palette
        assert!(PLAYFUL_THEME.color_palette.primary[0] > 0.5); // High red component
        assert!(PLAYFUL_THEME.color_palette.secondary[2] > 0.5); // High blue component
        assert!(PLAYFUL_THEME.color_palette.accent[1] > 0.5); // High green component
        
        // Validate dynamic animation settings
        assert!(PLAYFUL_THEME.animation_timings.playback_speed > 1.0);
        assert_eq!(PLAYFUL_THEME.animation_timings.loop_behavior, LoopBehavior::Repeat);
        
        // Validate particle system for high energy
        assert!(PLAYFUL_THEME.particle_systems.max_particles >= 1000);
        assert!(PLAYFUL_THEME.particle_systems.emission_rate >= 30.0);
        
        // Validate post-effects for vibrant look
        assert!(PLAYFUL_THEME.post_effects.bloom.is_some());
        assert!(PLAYFUL_THEME.post_effects.color_grading.is_some());
        
        let color_grading = PLAYFUL_THEME.post_effects.color_grading.as_ref().unwrap();
        assert!(color_grading.saturation > 1.0); // Enhanced saturation
    }
    
    #[test]
    fn test_playful_shader_configuration() {
        assert!(PLAYFUL_THEME.shader_variants.shader_defines.iter().any(|&(k, _)| k == "PLAYFUL_MODE"));
        assert!(PLAYFUL_THEME.shader_variants.shader_defines.iter().any(|&(k, _)| k == "DYNAMIC_COLORS"));
        assert!(PLAYFUL_THEME.shader_variants.shader_defines.iter().any(|&(k, _)| k == "RAINBOW_EFFECTS"));
        
        assert!(!PLAYFUL_THEME.shader_variants.compute_shaders.is_empty());
    }
    
    #[test]
    fn test_playful_lighting_setup() {
        assert!(PLAYFUL_THEME.lighting_config.ambient_intensity > 0.3);
        assert!(!PLAYFUL_THEME.lighting_config.point_lights.is_empty());
        
        // Verify colorful point lights
        let first_light = &PLAYFUL_THEME.lighting_config.point_lights[0];
        assert!(first_light.intensity > 1.0);
    }
    
    #[test]
    fn test_playful_config_constants() {
        assert!(config::ANIMATION_SPEED_MULTIPLIER > 1.0);
        assert!(config::COLOR_CYCLING_RATE > 0.0);
        assert!(config::PARTICLE_BURST_MULTIPLIER > 1.0);
        assert!(config::RAINBOW_CYCLE_DURATION > 0.0);
        assert!(config::HIGH_ENERGY_THRESHOLD > 0.0);
        assert!(config::HIGH_ENERGY_THRESHOLD < 1.0);
    }
}