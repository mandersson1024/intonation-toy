//! # Theme Manager
//!
//! The Theme Manager provides comprehensive theme configuration and management
//! for the immersive audio visualization experience. It supports compile-time
//! theme definitions with runtime switching capabilities.
//!
//! ## Architecture
//!
//! The Theme Manager provides:
//! - Compile-time theme registry with static theme definitions
//! - Runtime theme switching with <100ms performance target
//! - Theme persistence using browser local storage
//! - Integration with wgpu rendering pipeline
//! - Developer-friendly theme configuration API
//!
//! ## Key Components
//!
//! - [`ThemeDefinition`]: Comprehensive theme configuration structure
//! - [`ThemeManager`]: Core theme management trait
//! - [`ThemeRegistry`]: Compile-time theme registry
//! - [`ThemeError`]: Theme-specific error types
//! - [`UserThemeChoice`]: User-facing theme selection options

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// User-facing theme selection options (simplified interface)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserThemeChoice {
    Playful,
    Scientific,
}

impl UserThemeChoice {
    /// Get all available theme choices
    pub fn all() -> Vec<Self> {
        vec![Self::Playful, Self::Scientific]
    }
    
    /// Get the display name for the theme
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Playful => "Playful",
            Self::Scientific => "Scientific",
        }
    }
    
    /// Get the description for the theme
    pub fn description(&self) -> &'static str {
        match self {
            Self::Playful => "Vibrant colors and dynamic animations for an engaging experience",
            Self::Scientific => "Clean, analytical visualizations for precise monitoring",
        }
    }
}

/// Color palette definition for themes
#[derive(Debug, Clone, PartialEq)]
pub struct ColorPalette {
    pub primary: [f32; 4],          // RGBA
    pub secondary: [f32; 4],        // RGBA
    pub accent: [f32; 4],           // RGBA
    pub background: [f32; 4],       // RGBA
    pub surface: [f32; 4],          // RGBA
    pub text: [f32; 4],             // RGBA
    pub gradient_start: [f32; 4],   // RGBA
    pub gradient_end: [f32; 4],     // RGBA
}

/// Material properties for rendering
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialProperties {
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: [f32; 3],        // RGB
    pub opacity: f32,
    pub refraction_index: f32,
}

/// Lighting configuration for the scene
#[derive(Debug, Clone, PartialEq)]
pub struct LightingRig {
    pub ambient_intensity: f32,
    pub ambient_color: [f32; 3],    // RGB
    pub directional_intensity: f32,
    pub directional_color: [f32; 3], // RGB
    pub directional_direction: [f32; 3], // XYZ
    pub point_lights: &'static [PointLight],
}

/// Point light definition
#[derive(Debug, Clone, PartialEq)]
pub struct PointLight {
    pub position: [f32; 3],         // XYZ
    pub color: [f32; 3],            // RGB
    pub intensity: f32,
    pub range: f32,
}

/// Shader set configuration for themes
#[derive(Debug, Clone, PartialEq)]
pub struct ShaderSet {
    pub vertex_shader: &'static str,
    pub fragment_shader: &'static str,
    pub compute_shaders: &'static [&'static str],
    pub shader_defines: &'static [(&'static str, &'static str)],
}

/// Particle system configuration
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleConfig {
    pub max_particles: u32,
    pub emission_rate: f32,
    pub particle_lifetime: f32,
    pub size_range: (f32, f32),
    pub velocity_range: (f32, f32),
    pub color_over_lifetime: &'static [[f32; 4]], // RGBA keyframes
}

/// Animation timing and curve configuration
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationConfig {
    pub transition_duration: f32,   // seconds
    pub easing_curve: EasingCurve,
    pub loop_behavior: LoopBehavior,
    pub playback_speed: f32,
}

/// Easing curve types for animations
#[derive(Debug, Clone, PartialEq)]
pub enum EasingCurve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
    Custom(Vec<f32>), // Bezier control points
}

/// Loop behavior for animations
#[derive(Debug, Clone, PartialEq)]
pub enum LoopBehavior {
    None,
    Repeat,
    PingPong,
    Hold,
}

/// Post-processing effect chain
#[derive(Debug, Clone, PartialEq)]
pub struct EffectChain {
    pub bloom: Option<BloomEffect>,
    pub tone_mapping: Option<ToneMappingEffect>,
    pub color_grading: Option<ColorGradingEffect>,
    pub vignette: Option<VignetteEffect>,
}

/// Bloom post-processing effect
#[derive(Debug, Clone, PartialEq)]
pub struct BloomEffect {
    pub intensity: f32,
    pub threshold: f32,
    pub radius: f32,
}

/// Tone mapping effect
#[derive(Debug, Clone, PartialEq)]
pub struct ToneMappingEffect {
    pub exposure: f32,
    pub gamma: f32,
    pub contrast: f32,
}

/// Color grading effect
#[derive(Debug, Clone, PartialEq)]
pub struct ColorGradingEffect {
    pub saturation: f32,
    pub hue_shift: f32,
    pub brightness: f32,
}

/// Vignette effect
#[derive(Debug, Clone, PartialEq)]
pub struct VignetteEffect {
    pub intensity: f32,
    pub radius: f32,
    pub softness: f32,
}

/// Comprehensive theme definition structure
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeDefinition {
    pub name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    
    // Visual Configuration
    pub color_palette: ColorPalette,
    pub material_properties: MaterialProperties,
    pub lighting_config: LightingRig,
    
    // Rendering Configuration  
    pub shader_variants: ShaderSet,
    pub particle_systems: ParticleConfig,
    pub animation_timings: AnimationConfig,
    pub post_effects: EffectChain,
}

/// Theme metadata for UI display
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeMetadata {
    pub choice: UserThemeChoice,
    pub display_name: &'static str,
    pub description: &'static str,
    pub preview_image: Option<&'static str>, // Future: base64 encoded preview
}

/// Theme preview information
#[derive(Debug, Clone, PartialEq)]
pub struct ThemePreview {
    pub metadata: ThemeMetadata,
    pub dominant_colors: Vec<[f32; 4]>, // RGBA color swatches
    pub animation_style: &'static str,
    pub complexity: ThemeComplexity,
}

/// Theme complexity level
#[derive(Debug, Clone, PartialEq)]
pub enum ThemeComplexity {
    Simple,
    Moderate,
    Complex,
}

/// Theme-specific errors
#[derive(Debug, Clone, PartialEq)]
pub enum ThemeError {
    ThemeNotFound(UserThemeChoice),
    InvalidThemeData(String),
    RenderingError(String),
    StorageError(String),
    ValidationError(String),
}

impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ThemeNotFound(choice) => write!(f, "Theme not found: {:?}", choice),
            Self::InvalidThemeData(msg) => write!(f, "Invalid theme data: {}", msg),
            Self::RenderingError(msg) => write!(f, "Theme rendering error: {}", msg),
            Self::StorageError(msg) => write!(f, "Theme storage error: {}", msg),
            Self::ValidationError(msg) => write!(f, "Theme validation error: {}", msg),
        }
    }
}

impl std::error::Error for ThemeError {}

/// Core theme management trait
pub trait ThemeManager: Send + Sync {
    /// Get all available themes with metadata
    fn get_available_themes(&self) -> Vec<ThemeMetadata>;
    
    /// Get the currently active theme
    fn get_current_theme(&self) -> &'static ThemeDefinition;
    
    /// Set the active theme
    fn set_theme(&mut self, choice: UserThemeChoice) -> Result<(), ThemeError>;
    
    /// Get theme preview information
    fn get_theme_preview(&self, choice: UserThemeChoice) -> Result<ThemePreview, ThemeError>;
    
    /// Validate theme compatibility with current graphics capabilities
    fn validate_theme_compatibility(&self, choice: UserThemeChoice) -> Result<(), ThemeError>;
    
    /// Persist theme choice to browser storage
    fn persist_theme_choice(&self, choice: UserThemeChoice) -> Result<(), ThemeError>;
    
    /// Load theme choice from browser storage
    fn load_persisted_theme_choice(&self) -> Result<UserThemeChoice, ThemeError>;
}

/// Theme registry for compile-time theme definitions
pub struct ThemeRegistry {
    themes: HashMap<UserThemeChoice, ThemeDefinition>,
    current_theme: UserThemeChoice,
}

impl ThemeRegistry {
    /// Create a new theme registry with default themes
    pub fn new() -> Self {
        Self {
            themes: HashMap::new(), // Themes are accessed via external definitions
            current_theme: UserThemeChoice::Playful, // Default theme
        }
    }
    
    /// Get theme definition by choice
    pub fn get_theme(&self, choice: UserThemeChoice) -> Result<&'static ThemeDefinition, ThemeError> {
        Self::get_theme_definition(choice).ok_or(ThemeError::ThemeNotFound(choice))
    }
    
    /// Get theme definition by choice (using external definitions)
    fn get_theme_definition(choice: UserThemeChoice) -> Option<&'static ThemeDefinition> {
        // This will be implemented once the themes module is properly integrated
        // For now, return None to avoid compilation errors
        None
    }
}

impl ThemeManager for ThemeRegistry {
    fn get_available_themes(&self) -> Vec<ThemeMetadata> {
        UserThemeChoice::all()
            .into_iter()
            .map(|choice| ThemeMetadata {
                choice,
                display_name: choice.display_name(),
                description: choice.description(),
                preview_image: None, // Future enhancement
            })
            .collect()
    }
    
    fn get_current_theme(&self) -> &'static ThemeDefinition {
        Self::get_theme_definition(self.current_theme)
            .expect("Current theme should always be valid")
    }
    
    fn set_theme(&mut self, choice: UserThemeChoice) -> Result<(), ThemeError> {
        // Validate theme exists
        if Self::get_theme_definition(choice).is_none() {
            return Err(ThemeError::ThemeNotFound(choice));
        }
        
        // Validate compatibility (placeholder implementation)
        self.validate_theme_compatibility(choice)?;
        
        // Set theme
        self.current_theme = choice;
        
        // Persist choice
        self.persist_theme_choice(choice)?;
        
        Ok(())
    }
    
    fn get_theme_preview(&self, choice: UserThemeChoice) -> Result<ThemePreview, ThemeError> {
        let theme = self.get_theme(choice)?;
        
        Ok(ThemePreview {
            metadata: ThemeMetadata {
                choice,
                display_name: choice.display_name(),
                description: choice.description(),
                preview_image: None,
            },
            dominant_colors: vec![
                theme.color_palette.primary,
                theme.color_palette.secondary,
                theme.color_palette.accent,
            ],
            animation_style: match choice {
                UserThemeChoice::Playful => "Dynamic and energetic",
                UserThemeChoice::Scientific => "Precise and controlled",
            },
            complexity: match choice {
                UserThemeChoice::Playful => ThemeComplexity::Moderate,
                UserThemeChoice::Scientific => ThemeComplexity::Simple,
            },
        })
    }
    
    fn validate_theme_compatibility(&self, _choice: UserThemeChoice) -> Result<(), ThemeError> {
        // Placeholder implementation - will integrate with graphics capabilities
        // TODO: Check graphics capabilities against theme requirements
        Ok(())
    }
    
    fn persist_theme_choice(&self, choice: UserThemeChoice) -> Result<(), ThemeError> {
        // Use browser local storage for persistence
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let choice_str = match choice {
                    UserThemeChoice::Playful => "playful",
                    UserThemeChoice::Scientific => "scientific",
                };
                
                storage.set_item("pitch_toy_theme", choice_str)
                    .map_err(|_| ThemeError::StorageError("Failed to persist theme choice".to_string()))?;
            }
        }
        
        Ok(())
    }
    
    fn load_persisted_theme_choice(&self) -> Result<UserThemeChoice, ThemeError> {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(choice_str)) = storage.get_item("pitch_toy_theme") {
                    return match choice_str.as_str() {
                        "playful" => Ok(UserThemeChoice::Playful),
                        "scientific" => Ok(UserThemeChoice::Scientific),
                        _ => Err(ThemeError::InvalidThemeData(format!("Unknown theme: {}", choice_str))),
                    };
                }
            }
        }
        
        // Default to Playful theme if no persisted choice
        Ok(UserThemeChoice::Playful)
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_theme_registry_creation() {
        let registry = ThemeRegistry::new();
        assert_eq!(registry.themes.len(), 2);
        assert!(registry.themes.contains_key(&UserThemeChoice::Playful));
        assert!(registry.themes.contains_key(&UserThemeChoice::Scientific));
    }
    
    #[test]
    fn test_theme_switching() {
        let mut registry = ThemeRegistry::new();
        
        // Initially Playful
        assert_eq!(registry.current_theme, UserThemeChoice::Playful);
        
        // Switch to Scientific
        registry.set_theme(UserThemeChoice::Scientific).unwrap();
        assert_eq!(registry.current_theme, UserThemeChoice::Scientific);
        
        // Switch back to Playful
        registry.set_theme(UserThemeChoice::Playful).unwrap();
        assert_eq!(registry.current_theme, UserThemeChoice::Playful);
    }
    
    #[test]
    fn test_theme_metadata() {
        let registry = ThemeRegistry::new();
        let themes = registry.get_available_themes();
        
        assert_eq!(themes.len(), 2);
        assert!(themes.iter().any(|t| t.choice == UserThemeChoice::Playful));
        assert!(themes.iter().any(|t| t.choice == UserThemeChoice::Scientific));
    }
    
    #[test]
    fn test_theme_preview() {
        let registry = ThemeRegistry::new();
        
        let playful_preview = registry.get_theme_preview(UserThemeChoice::Playful).unwrap();
        assert_eq!(playful_preview.metadata.choice, UserThemeChoice::Playful);
        assert_eq!(playful_preview.dominant_colors.len(), 3);
        
        let scientific_preview = registry.get_theme_preview(UserThemeChoice::Scientific).unwrap();
        assert_eq!(scientific_preview.metadata.choice, UserThemeChoice::Scientific);
        assert_eq!(scientific_preview.dominant_colors.len(), 3);
    }
    
    #[test]
    fn test_user_theme_choice_display() {
        assert_eq!(UserThemeChoice::Playful.display_name(), "Playful");
        assert_eq!(UserThemeChoice::Scientific.display_name(), "Scientific");
        
        assert!(!UserThemeChoice::Playful.description().is_empty());
        assert!(!UserThemeChoice::Scientific.description().is_empty());
    }
}