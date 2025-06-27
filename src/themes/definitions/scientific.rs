//! # Scientific Theme Definition
//!
//! The Scientific theme provides a clean, analytical audio visualization experience
//! with precise colors, controlled animations, and professional aesthetics.
//! It's designed for accurate audio analysis and monitoring.

use crate::modules::presentation_layer::theme_manager::*;

/// Scientific theme definition with clean aesthetics and precise visualizations
pub const SCIENTIFIC_THEME: ThemeDefinition = ThemeDefinition {
    name: "scientific",
    display_name: "Scientific",
    description: "Clean, analytical visualizations for precise monitoring",
    
    color_palette: ColorPalette {
        primary: [0.2, 0.6, 0.9, 1.0],        // Professional blue
        secondary: [0.1, 0.8, 0.6, 1.0],      // Teal
        accent: [0.9, 0.6, 0.2, 1.0],         // Warm orange for highlights
        background: [0.05, 0.05, 0.08, 1.0],  // Very dark blue-gray
        surface: [0.1, 0.1, 0.15, 1.0],       // Dark blue-gray
        text: [0.9, 0.95, 1.0, 1.0],          // Cool white
        gradient_start: [0.2, 0.6, 0.9, 1.0], // Blue
        gradient_end: [0.1, 0.8, 0.6, 1.0],   // Teal
    },
    
    material_properties: MaterialProperties {
        metallic: 0.8,  // More metallic for precision look
        roughness: 0.2, // Smooth for clean appearance
        emissive: [0.1, 0.15, 0.2],
        opacity: 0.95,
        refraction_index: 1.5,
    },
    
    lighting_config: LightingRig {
        ambient_intensity: 0.2,  // Lower ambient for contrast
        ambient_color: [0.8, 0.9, 1.0],
        directional_intensity: 1.0,  // Strong directional light for clarity
        directional_color: [1.0, 1.0, 1.0],
        directional_direction: [-0.3, -0.8, -0.2],
        point_lights: &[
            PointLight {
                position: [0.0, 5.0, 2.0],
                color: [0.8, 0.9, 1.0],  // Cool white
                intensity: 1.0,
                range: 15.0,
            },
            PointLight {
                position: [3.0, 2.0, -1.0],
                color: [0.9, 0.95, 1.0], // Slightly warm white
                intensity: 0.6,
                range: 10.0,
            },
        ],
    },
    
    shader_variants: ShaderSet {
        vertex_shader: "shaders/scientific/vertex.wgsl",
        fragment_shader: "shaders/scientific/fragment.wgsl",
        compute_shaders: &[
            "shaders/scientific/analysis_compute.wgsl",
            "shaders/scientific/frequency_analysis.wgsl",
            "shaders/scientific/precision_render.wgsl"
        ],
        shader_defines: &[
            ("SCIENTIFIC_MODE", "1"),
            ("PRECISE_RENDERING", "1"),
            ("ANALYTICAL_VIEW", "1"),
            ("HIGH_PRECISION", "1"),
        ],
    },
    
    particle_systems: ParticleConfig {
        max_particles: 1000,  // Fewer particles for cleaner look
        emission_rate: 20.0,  // Controlled emission
        particle_lifetime: 5.0,  // Longer lifetime for stability
        size_range: (0.05, 0.3),  // Smaller, more precise particles
        velocity_range: (0.2, 1.0),  // Controlled movement
        color_over_lifetime: &[
            [0.2, 0.6, 0.9, 1.0], // Birth: professional blue
            [0.15, 0.7, 0.8, 0.9], // Early: blue-teal
            [0.1, 0.8, 0.6, 0.8], // Mid: teal
            [0.9, 0.6, 0.2, 0.6], // Late: orange highlight
            [0.8, 0.9, 1.0, 0.0], // Death: cool white, transparent
        ],
    },
    
    animation_timings: AnimationConfig {
        transition_duration: 0.3,  // Fast, precise transitions
        easing_curve: EasingCurve::Linear,  // Linear for scientific precision
        loop_behavior: LoopBehavior::None,  // No looping for stability
        playback_speed: 1.0,  // Standard speed
    },
    
    post_effects: EffectChain {
        bloom: Some(BloomEffect {
            intensity: 0.3,  // Subtle bloom
            threshold: 0.8,  // High threshold for precision
            radius: 0.8,
        }),
        tone_mapping: Some(ToneMappingEffect {
            exposure: 1.0,   // Standard exposure
            gamma: 2.2,      // Standard gamma
            contrast: 1.0,   // No contrast enhancement
        }),
        color_grading: Some(ColorGradingEffect {
            saturation: 1.0, // Natural saturation
            hue_shift: 0.0,  // No hue shift
            brightness: 1.0, // Standard brightness
        }),
        vignette: None,  // No vignette for scientific clarity
    },
};

/// Configuration constants specific to the Scientific theme
pub mod config {
    /// Grid line spacing for analytical overlays (in normalized units)
    pub const GRID_SPACING: f32 = 0.1;
    
    /// Frequency analysis update rate in Hz
    pub const ANALYSIS_UPDATE_RATE: f32 = 60.0;
    
    /// Measurement precision decimal places
    pub const MEASUREMENT_PRECISION: u32 = 3;
    
    /// Data smoothing factor for stable readings
    pub const DATA_SMOOTHING_FACTOR: f32 = 0.8;
    
    /// Threshold for showing measurement annotations
    pub const ANNOTATION_THRESHOLD: f32 = 0.1;
    
    /// Color accuracy requirement (Delta E)
    pub const COLOR_ACCURACY_REQUIREMENT: f32 = 2.0;
    
    /// Minimum contrast ratio for readability
    pub const MIN_CONTRAST_RATIO: f32 = 4.5;
}

/// Scientific visualization modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScientificMode {
    /// Standard frequency analysis
    FrequencyAnalysis,
    /// Spectral visualization with precise measurements
    SpectralAnalysis,
    /// Waveform analysis with time domain
    WaveformAnalysis,
    /// Phase correlation analysis
    PhaseAnalysis,
}

impl ScientificMode {
    /// Get all available scientific modes
    pub fn all() -> Vec<Self> {
        vec![
            Self::FrequencyAnalysis,
            Self::SpectralAnalysis,
            Self::WaveformAnalysis,
            Self::PhaseAnalysis,
        ]
    }
    
    /// Get display name for the mode
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::FrequencyAnalysis => "Frequency Analysis",
            Self::SpectralAnalysis => "Spectral Analysis",
            Self::WaveformAnalysis => "Waveform Analysis",
            Self::PhaseAnalysis => "Phase Analysis",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scientific_theme_definition() {
        assert_eq!(SCIENTIFIC_THEME.name, "scientific");
        assert_eq!(SCIENTIFIC_THEME.display_name, "Scientific");
        assert!(!SCIENTIFIC_THEME.description.is_empty());
        
        // Validate professional color palette
        assert!(SCIENTIFIC_THEME.color_palette.primary[2] > 0.8); // High blue component
        assert!(SCIENTIFIC_THEME.color_palette.background[0] < 0.1); // Dark background
        assert!(SCIENTIFIC_THEME.color_palette.background[1] < 0.1);
        assert!(SCIENTIFIC_THEME.color_palette.background[2] < 0.1);
        
        // Validate precise animation settings
        assert_eq!(SCIENTIFIC_THEME.animation_timings.playback_speed, 1.0);
        assert_eq!(SCIENTIFIC_THEME.animation_timings.loop_behavior, LoopBehavior::None);
        assert_eq!(SCIENTIFIC_THEME.animation_timings.easing_curve, EasingCurve::Linear);
        
        // Validate controlled particle system
        assert!(SCIENTIFIC_THEME.particle_systems.max_particles <= 1500);
        assert!(SCIENTIFIC_THEME.particle_systems.emission_rate <= 30.0);
        
        // Validate minimal post-effects for accuracy
        assert!(SCIENTIFIC_THEME.post_effects.bloom.is_some());
        let bloom = SCIENTIFIC_THEME.post_effects.bloom.as_ref().unwrap();
        assert!(bloom.intensity < 0.5); // Subtle bloom
        
        assert!(SCIENTIFIC_THEME.post_effects.vignette.is_none()); // No vignette
        
        let color_grading = SCIENTIFIC_THEME.post_effects.color_grading.as_ref().unwrap();
        assert_eq!(color_grading.saturation, 1.0); // Natural saturation
        assert_eq!(color_grading.hue_shift, 0.0); // No hue shift
    }
    
    #[test]
    fn test_scientific_shader_configuration() {
        assert!(SCIENTIFIC_THEME.shader_variants.shader_defines.iter().any(|&(k, _)| k == "SCIENTIFIC_MODE"));
        assert!(SCIENTIFIC_THEME.shader_variants.shader_defines.iter().any(|&(k, _)| k == "PRECISE_RENDERING"));
        assert!(SCIENTIFIC_THEME.shader_variants.shader_defines.iter().any(|&(k, _)| k == "ANALYTICAL_VIEW"));
        assert!(SCIENTIFIC_THEME.shader_variants.shader_defines.iter().any(|&(k, _)| k == "HIGH_PRECISION"));
        
        assert!(SCIENTIFIC_THEME.shader_variants.compute_shaders.len() >= 2);
    }
    
    #[test]
    fn test_scientific_lighting_setup() {
        assert!(SCIENTIFIC_THEME.lighting_config.ambient_intensity < 0.3);
        assert_eq!(SCIENTIFIC_THEME.lighting_config.directional_intensity, 1.0);
        assert!(!SCIENTIFIC_THEME.lighting_config.point_lights.is_empty());
        
        // Verify neutral colored lights
        let first_light = &SCIENTIFIC_THEME.lighting_config.point_lights[0];
        assert!(first_light.color[0] > 0.7); // High red
        assert!(first_light.color[1] > 0.8); // High green
        assert!(first_light.color[2] > 0.9); // High blue (cool white)
    }
    
    #[test]
    fn test_scientific_config_constants() {
        assert!(config::GRID_SPACING > 0.0);
        assert!(config::GRID_SPACING < 1.0);
        assert!(config::ANALYSIS_UPDATE_RATE > 0.0);
        assert!(config::MEASUREMENT_PRECISION > 0);
        assert!(config::DATA_SMOOTHING_FACTOR > 0.0);
        assert!(config::DATA_SMOOTHING_FACTOR <= 1.0);
        assert!(config::ANNOTATION_THRESHOLD > 0.0);
        assert!(config::COLOR_ACCURACY_REQUIREMENT > 0.0);
        assert!(config::MIN_CONTRAST_RATIO >= 3.0);
    }
    
    #[test]
    fn test_scientific_modes() {
        let modes = ScientificMode::all();
        assert_eq!(modes.len(), 4);
        
        for mode in modes {
            assert!(!mode.display_name().is_empty());
        }
    }
}