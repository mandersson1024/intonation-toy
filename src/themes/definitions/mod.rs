//! # Theme Definitions
//!
//! This module contains the individual theme definitions for the pitch-toy
//! application. Each theme provides a complete configuration for visual
//! appearance, rendering parameters, and user experience.
//!
//! ## Available Themes
//!
//! - [`playful`]: Vibrant, dynamic theme with energetic animations
//! - [`scientific`]: Clean, analytical theme with precise visualizations

pub mod playful;
pub mod scientific;

// Re-export theme definitions
pub use playful::PLAYFUL_THEME;
pub use scientific::SCIENTIFIC_THEME;

/// Get all theme definitions as a static array
pub const ALL_THEMES: &[&crate::modules::presentation_layer::theme_manager::ThemeDefinition] = &[
    &PLAYFUL_THEME,
    &SCIENTIFIC_THEME,
];

/// Get theme definition by choice
pub fn get_theme_definition(
    choice: crate::modules::presentation_layer::theme_manager::UserThemeChoice
) -> Option<&'static crate::modules::presentation_layer::theme_manager::ThemeDefinition> {
    match choice {
        crate::modules::presentation_layer::theme_manager::UserThemeChoice::Playful => Some(&PLAYFUL_THEME),
        crate::modules::presentation_layer::theme_manager::UserThemeChoice::Scientific => Some(&SCIENTIFIC_THEME),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::presentation_layer::theme_manager::UserThemeChoice;
    
    #[test]
    fn test_all_themes_available() {
        assert_eq!(ALL_THEMES.len(), 2);
        
        let playful = get_theme_definition(UserThemeChoice::Playful);
        assert!(playful.is_some());
        assert_eq!(playful.unwrap().name, "playful");
        
        let scientific = get_theme_definition(UserThemeChoice::Scientific);
        assert!(scientific.is_some());
        assert_eq!(scientific.unwrap().name, "scientific");
    }
    
    #[test]
    fn test_theme_definitions_valid() {
        for theme in ALL_THEMES {
            assert!(!theme.name.is_empty());
            assert!(!theme.display_name.is_empty());
            assert!(!theme.description.is_empty());
            
            // Validate color palette
            assert!(theme.color_palette.primary[3] > 0.0); // Alpha > 0
            assert!(theme.color_palette.background[3] > 0.0); // Alpha > 0
            
            // Validate animation config
            assert!(theme.animation_timings.transition_duration > 0.0);
            assert!(theme.animation_timings.playback_speed > 0.0);
        }
    }
}