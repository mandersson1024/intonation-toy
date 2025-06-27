//! # Theme System
//!
//! The Theme System provides compile-time theme definitions and runtime
//! theme management for the pitch-toy application. It supports rich
//! theme configuration with shader variants, animations, and visual effects.
//!
//! ## Architecture
//!
//! The Theme System provides:
//! - Compile-time theme registry with static definitions
//! - Runtime theme switching with performance optimization
//! - Theme persistence and user preference management
//! - Integration with wgpu rendering pipeline
//! - Developer-friendly theme configuration API
//!
//! ## Theme Organization
//!
//! ```
//! src/themes/
//! ├── mod.rs                    # Theme registry and exports
//! ├── definitions/              # Individual theme definitions
//! │   ├── playful.rs           # Playful theme definition
//! │   └── scientific.rs        # Scientific theme definition
//! └── shaders/                  # Per-theme shader variants
//!     ├── playful/             # Playful theme shaders
//!     └── scientific/          # Scientific theme shaders
//! ```

pub mod definitions;

// Re-export theme definitions for easy access
pub use definitions::*;

// Import theme manager from presentation layer
pub use crate::modules::presentation_layer::theme_manager::*;

/// Global theme registry instance
pub static THEME_REGISTRY: std::sync::LazyLock<std::sync::RwLock<ThemeRegistry>> = 
    std::sync::LazyLock::new(|| {
        std::sync::RwLock::new(ThemeRegistry::new())
    });

/// Get a reference to the global theme registry
pub fn get_theme_registry() -> &'static std::sync::RwLock<ThemeRegistry> {
    &THEME_REGISTRY
}

/// Convenience function to get the current theme
pub fn get_current_theme() -> Result<ThemeDefinition, ThemeError> {
    let registry = THEME_REGISTRY.read()
        .map_err(|_| ThemeError::ValidationError("Failed to acquire theme registry lock".to_string()))?;
    Ok(registry.get_current_theme().clone())
}

/// Convenience function to set the current theme
pub fn set_current_theme(choice: UserThemeChoice) -> Result<(), ThemeError> {
    let mut registry = THEME_REGISTRY.write()
        .map_err(|_| ThemeError::ValidationError("Failed to acquire theme registry lock".to_string()))?;
    registry.set_theme(choice)
}

/// Convenience function to get all available themes
pub fn get_available_themes() -> Result<Vec<ThemeMetadata>, ThemeError> {
    let registry = THEME_REGISTRY.read()
        .map_err(|_| ThemeError::ValidationError("Failed to acquire theme registry lock".to_string()))?;
    Ok(registry.get_available_themes())
}

/// Convenience function to get theme preview
pub fn get_theme_preview(choice: UserThemeChoice) -> Result<ThemePreview, ThemeError> {
    let registry = THEME_REGISTRY.read()
        .map_err(|_| ThemeError::ValidationError("Failed to acquire theme registry lock".to_string()))?;
    registry.get_theme_preview(choice)
}

/// Initialize theme system with persisted user preference
pub fn initialize_theme_system() -> Result<UserThemeChoice, ThemeError> {
    let mut registry = THEME_REGISTRY.write()
        .map_err(|_| ThemeError::ValidationError("Failed to acquire theme registry lock".to_string()))?;
    
    // Try to load persisted theme choice
    let choice = registry.load_persisted_theme_choice()
        .unwrap_or(UserThemeChoice::Playful); // Default to Playful if no persisted choice
    
    // Set the theme
    registry.set_theme(choice)?;
    
    Ok(choice)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_global_theme_registry() {
        let registry = get_theme_registry();
        let registry_guard = registry.read().unwrap();
        
        assert_eq!(registry_guard.themes.len(), 2);
        assert!(registry_guard.themes.contains_key(&UserThemeChoice::Playful));
        assert!(registry_guard.themes.contains_key(&UserThemeChoice::Scientific));
    }
    
    #[test]
    fn test_convenience_functions() {
        // Test get_available_themes
        let themes = get_available_themes().unwrap();
        assert_eq!(themes.len(), 2);
        
        // Test theme switching
        set_current_theme(UserThemeChoice::Scientific).unwrap();
        let current = get_current_theme().unwrap();
        assert_eq!(current.name, "scientific");
        
        // Test theme preview
        let preview = get_theme_preview(UserThemeChoice::Playful).unwrap();
        assert_eq!(preview.metadata.choice, UserThemeChoice::Playful);
    }
    
    #[test]
    fn test_theme_system_initialization() {
        let choice = initialize_theme_system().unwrap();
        assert!(matches!(choice, UserThemeChoice::Playful | UserThemeChoice::Scientific));
    }
}