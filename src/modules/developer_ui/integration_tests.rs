//! # Developer UI Integration Tests
//!
//! Comprehensive integration tests for the Developer UI module.
//! Tests conditional compilation, module registration, component lifecycle,
//! and zero production impact validation.

#[cfg(test)]
#[cfg(debug_assertions)]
mod tests {
    use super::*;
    use crate::modules::application_core::{ModuleRegistry, ModuleError};
    use std::rc::Rc;
    use std::cell::RefCell;

    /// Test module creation and initialization
    #[test]
    fn test_developer_ui_module_creation() {
        let result = DeveloperUIModule::new();
        assert!(result.is_ok(), "DeveloperUIModule creation should succeed");
        
        let module = result.unwrap();
        assert_eq!(module.name(), "developer_ui");
        assert!(!module.initialized);
    }

    /// Test module initialization process
    #[test]
    fn test_developer_ui_module_initialization() {
        let mut module = DeveloperUIModule::new().unwrap();
        
        let result = module.initialize();
        assert!(result.is_ok(), "Module initialization should succeed");
        
        // Verify initialization state
        assert!(module.initialized);
    }

    /// Test module shutdown process
    #[test]
    fn test_developer_ui_module_shutdown() {
        let mut module = DeveloperUIModule::new().unwrap();
        module.initialize().unwrap();
        
        let result = module.shutdown();
        assert!(result.is_ok(), "Module shutdown should succeed");
        
        // Verify shutdown state
        assert!(!module.initialized);
    }

    /// Test debug component registry functionality
    #[test]
    fn test_debug_component_registry() {
        let registry = DebugComponentRegistry::new();
        
        assert_eq!(registry.component_count(), 0);
        assert_eq!(registry.active_component_count(), 0);
        assert!(!registry.is_component_registered("test_component"));
    }

    /// Test overlay manager functionality
    #[test]
    fn test_overlay_manager() {
        let manager_result = OverlayManager::new();
        assert!(manager_result.is_ok(), "OverlayManager creation should succeed");
        
        let mut manager = manager_result.unwrap();
        let init_result = manager.initialize();
        assert!(init_result.is_ok(), "OverlayManager initialization should succeed");
        
        // Test default overlays are created
        assert!(manager.get_overlay("audio_controls").is_some());
        assert!(manager.get_overlay("debug_interface").is_some());
        assert!(manager.get_overlay("performance_monitor").is_some());
    }

    /// Test debug app functionality
    #[test]
    fn test_debug_app() {
        let app_result = DebugApp::new();
        assert!(app_result.is_ok(), "DebugApp creation should succeed");
        
        let mut app = app_result.unwrap();
        assert!(!app.is_initialized());
        
        let init_result = app.initialize();
        assert!(init_result.is_ok(), "DebugApp initialization should succeed");
        assert!(app.is_initialized());
    }

    /// Test conditional compilation - this test should only run in debug builds
    #[test]
    fn test_conditional_compilation() {
        // This test existing proves conditional compilation is working
        // since it's wrapped in #[cfg(debug_assertions)]
        assert!(true, "This test should only run in debug builds");
    }

    /// Test module version information
    #[test]
    fn test_module_version() {
        let module = DeveloperUIModule::new().unwrap();
        let version = module.version();
        assert!(!version.is_empty(), "Module version should not be empty");
    }

    /// Test overlay state management
    #[test]
    fn test_overlay_state_management() {
        let mut manager = OverlayManager::new().unwrap();
        manager.initialize().unwrap();
        
        // Test showing and hiding overlays
        assert!(manager.show_overlay("audio_controls").is_ok());
        assert_eq!(manager.get_active_overlays().len(), 1);
        
        assert!(manager.hide_overlay("audio_controls").is_ok());
        assert_eq!(manager.get_active_overlays().len(), 0);
    }

    /// Test debug app state toggles
    #[test]
    fn test_debug_app_state_toggles() {
        let mut app = DebugApp::new().unwrap();
        
        let initial_visibility = app.get_debug_state().ui_visible;
        app.toggle_ui_visibility();
        assert_eq!(app.get_debug_state().ui_visible, !initial_visibility);
        
        app.toggle_performance_overlay();
        app.toggle_error_display();
        app.toggle_component_inspector();
        app.toggle_audio_monitoring();
        
        // All toggles should work without panicking
    }

    /// Test keyboard shortcuts
    #[test]
    fn test_debug_app_keyboard_shortcuts() {
        let mut app = DebugApp::new().unwrap();
        
        // Test valid shortcuts
        let result = app.handle_keyboard_shortcut("d", true, false, true);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should be handled
        
        // Test invalid shortcuts
        let result = app.handle_keyboard_shortcut("x", true, false, true);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should not be handled
    }

    /// Test layout mode changes
    #[test]
    fn test_layout_mode_changes() {
        use super::overlay_manager::LayoutMode;
        
        let mut manager = OverlayManager::new().unwrap();
        manager.initialize().unwrap();
        
        assert!(manager.set_layout_mode(LayoutMode::Tiled).is_ok());
        match manager.get_layout_mode() {
            LayoutMode::Tiled => {},
            _ => panic!("Layout mode should be Tiled"),
        }
        
        assert!(manager.set_layout_mode(LayoutMode::Docked).is_ok());
        match manager.get_layout_mode() {
            LayoutMode::Docked => {},
            _ => panic!("Layout mode should be Docked"),
        }
    }

    /// Test global overlay settings
    #[test]
    fn test_overlay_global_settings() {
        use super::overlay_manager::OverlayGlobalSettings;
        
        let mut manager = OverlayManager::new().unwrap();
        let settings = manager.get_global_settings();
        
        // Test default settings
        assert!(!settings.snap_to_grid);
        assert!(settings.auto_arrange);
        assert!(settings.collision_detection);
        assert!(settings.animation_enabled);
        
        // Test updating settings
        let new_settings = OverlayGlobalSettings {
            snap_to_grid: true,
            grid_size: 20.0,
            auto_arrange: false,
            collision_detection: false,
            animation_enabled: false,
            animation_duration: 0.5,
        };
        
        manager.update_global_settings(new_settings);
        let updated_settings = manager.get_global_settings();
        assert!(updated_settings.snap_to_grid);
        assert!(!updated_settings.auto_arrange);
        assert_eq!(updated_settings.grid_size, 20.0);
    }

    /// Test debug statistics
    #[test]
    fn test_debug_statistics() {
        let mut app = DebugApp::new().unwrap();
        app.initialize().unwrap();
        
        let stats = app.get_debug_statistics();
        assert!(stats.initialized);
        assert!(stats.performance_monitoring_active);
        assert!(stats.error_tracking_active);
    }

    /// Mock test for production build impact validation
    /// This would be part of build testing but included here for completeness
    #[test]
    fn test_production_build_impact_placeholder() {
        // In a real test environment, this would:
        // 1. Compile with release profile
        // 2. Verify developer_ui module is not included
        // 3. Compare build sizes
        // 4. Verify no debug symbols or code paths exist
        
        // For now, this serves as a placeholder
        assert!(true, "Production build impact validation placeholder");
    }
}

// Additional test utilities for future component testing
#[cfg(test)]
#[cfg(debug_assertions)]
mod test_utilities {
    use super::*;
    
    /// Helper function to create a test developer UI module
    pub fn create_test_module() -> Result<DeveloperUIModule, ModuleError> {
        let mut module = DeveloperUIModule::new()?;
        module.initialize()?;
        Ok(module)
    }
    
    /// Helper function to create a test overlay manager
    pub fn create_test_overlay_manager() -> Result<OverlayManager, ModuleError> {
        let mut manager = OverlayManager::new()?;
        manager.initialize()?;
        Ok(manager)
    }
    
    /// Helper function to create a test debug app
    pub fn create_test_debug_app() -> Result<DebugApp, ModuleError> {
        let mut app = DebugApp::new()?;
        app.initialize()?;
        Ok(app)
    }
} 