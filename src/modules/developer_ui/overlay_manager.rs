//! # Overlay Manager
//!
//! Manages debug overlay coordination and layout management for the Developer UI.
//! This component handles the positioning, sizing, and coordination of multiple
//! debug overlays within the application interface.

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::modules::application_core::EventBus;
use super::debug_component_registry::{OverlayState, OverlayPosition, OverlaySize};

/// Manager for debug overlay coordination and layout
#[cfg(debug_assertions)]
pub struct OverlayManager {
    overlays: HashMap<String, DebugOverlay>,
    active_overlays: Vec<String>,
    layout_mode: LayoutMode,
    event_bus: Option<Rc<dyn EventBus>>,
    initialized: bool,
    global_settings: OverlayGlobalSettings,
}

/// Individual debug overlay configuration
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct DebugOverlay {
    pub id: String,
    pub title: String,
    pub state: OverlayState,
    pub resizable: bool,
    pub movable: bool,
    pub collapsible: bool,
    pub z_index: i32,
    pub component_type: OverlayComponentType,
}

/// Types of debug overlay components
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub enum OverlayComponentType {
    AudioControls,
    DebugInterface,
    ErrorDisplay,
    MetricsDisplay,
    PerformanceMonitor,
    ComponentInspector,
}

/// Layout modes for overlay arrangement
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub enum LayoutMode {
    Free,           // Free positioning
    Tiled,          // Tiled layout
    Docked,         // Docked to edges
    Tabbed,         // Tabbed interface
}

/// Global overlay settings
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct OverlayGlobalSettings {
    pub snap_to_grid: bool,
    pub grid_size: f32,
    pub auto_arrange: bool,
    pub collision_detection: bool,
    pub animation_enabled: bool,
    pub animation_duration: f32,
}

#[cfg(debug_assertions)]
impl Default for OverlayGlobalSettings {
    fn default() -> Self {
        Self {
            snap_to_grid: false,
            grid_size: 10.0,
            auto_arrange: true,
            collision_detection: true,
            animation_enabled: true,
            animation_duration: 0.3,
        }
    }
}

#[cfg(debug_assertions)]
impl OverlayManager {
    /// Create a new overlay manager
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            overlays: HashMap::new(),
            active_overlays: Vec::new(),
            layout_mode: LayoutMode::Free,
            event_bus: None,
            initialized: false,
            global_settings: OverlayGlobalSettings::default(),
        })
    }

    /// Initialize the overlay manager
    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.initialized {
            return Ok(());
        }

        // Setup default overlays
        self.setup_default_overlays()?;
        
        // Initialize layout system
        self.initialize_layout_system()?;
        
        self.initialized = true;
        Ok(())
    }

    /// Shutdown the overlay manager
    pub fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.overlays.clear();
        self.active_overlays.clear();
        self.initialized = false;
        Ok(())
    }

    /// Connect to event bus
    pub fn connect_event_bus(&mut self, event_bus: Rc<dyn EventBus>) -> Result<(), Box<dyn std::error::Error>> {
        self.event_bus = Some(event_bus);
        Ok(())
    }

    /// Register a debug overlay
    pub fn register_overlay(&mut self, overlay: DebugOverlay) -> Result<(), Box<dyn std::error::Error>> {
        if self.overlays.contains_key(&overlay.id) {
            return Err((
                format!("Overlay '{}' is already registered", overlay.id)
            ));
        }

        self.overlays.insert(overlay.id.clone(), overlay);
        Ok(())
    }

    /// Unregister a debug overlay
    pub fn unregister_overlay(&mut self, overlay_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.overlays.remove(overlay_id);
        self.active_overlays.retain(|id| id != overlay_id);
        Ok(())
    }

    /// Show an overlay
    pub fn show_overlay(&mut self, overlay_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.overlays.contains_key(overlay_id) {
            return Err((
                format!("Overlay '{}' not found", overlay_id)
            ));
        }

        if let Some(overlay) = self.overlays.get_mut(overlay_id) {
            overlay.state.visible = true;
        }

        if !self.active_overlays.contains(&overlay_id.to_string()) {
            self.active_overlays.push(overlay_id.to_string());
        }

        // Apply layout if auto-arrange is enabled
        if self.global_settings.auto_arrange {
            self.arrange_overlays()?;
        }

        Ok(())
    }

    /// Hide an overlay
    pub fn hide_overlay(&mut self, overlay_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(overlay) = self.overlays.get_mut(overlay_id) {
            overlay.state.visible = false;
        }

        self.active_overlays.retain(|id| id != overlay_id);
        Ok(())
    }

    /// Move an overlay to a new position
    pub fn move_overlay(&mut self, overlay_id: &str, position: OverlayPosition) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(overlay) = self.overlays.get_mut(overlay_id) {
            if !overlay.movable {
                return Err((
                    format!("Overlay '{}' is not movable", overlay_id)
                ));
            }

            let final_position = if self.global_settings.snap_to_grid {
                self.snap_to_grid(position)
            } else {
                position
            };

            overlay.state.position = final_position;

            // Check for collisions if enabled
            if self.global_settings.collision_detection {
                self.resolve_collisions(overlay_id)?;
            }
        }

        Ok(())
    }

    /// Resize an overlay
    pub fn resize_overlay(&mut self, overlay_id: &str, size: OverlaySize) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(overlay) = self.overlays.get_mut(overlay_id) {
            if !overlay.resizable {
                return Err((
                    format!("Overlay '{}' is not resizable", overlay_id)
                ));
            }

            overlay.state.size = size;
        }

        Ok(())
    }

    /// Set layout mode
    pub fn set_layout_mode(&mut self, mode: LayoutMode) -> Result<(), Box<dyn std::error::Error>> {
        self.layout_mode = mode;
        self.arrange_overlays()?;
        Ok(())
    }

    /// Get current layout mode
    pub fn get_layout_mode(&self) -> &LayoutMode {
        &self.layout_mode
    }

    /// Get overlay by ID
    pub fn get_overlay(&self, overlay_id: &str) -> Option<&DebugOverlay> {
        self.overlays.get(overlay_id)
    }

    /// Get list of active overlays
    pub fn get_active_overlays(&self) -> Vec<String> {
        self.active_overlays.clone()
    }

    /// Get global settings
    pub fn get_global_settings(&self) -> &OverlayGlobalSettings {
        &self.global_settings
    }

    /// Update global settings
    pub fn update_global_settings(&mut self, settings: OverlayGlobalSettings) {
        self.global_settings = settings;
    }

    /// Arrange overlays according to current layout mode
    fn arrange_overlays(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.layout_mode {
            LayoutMode::Free => {
                // No automatic arrangement in free mode
                Ok(())
            },
            LayoutMode::Tiled => {
                self.arrange_tiled()
            },
            LayoutMode::Docked => {
                self.arrange_docked()
            },
            LayoutMode::Tabbed => {
                self.arrange_tabbed()
            },
        }
    }

    /// Arrange overlays in tiled layout
    fn arrange_tiled(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let overlay_count = self.active_overlays.len();
        if overlay_count == 0 {
            return Ok(());
        }

        // Calculate grid dimensions
        let cols = (overlay_count as f32).sqrt().ceil() as usize;
        let rows = (overlay_count + cols - 1) / cols;

        // Assuming viewport dimensions (would be provided by the UI system)
        let viewport_width = 1200.0;
        let viewport_height = 800.0;
        let overlay_width = viewport_width / cols as f32;
        let overlay_height = viewport_height / rows as f32;

        for (index, overlay_id) in self.active_overlays.iter().enumerate() {
            if let Some(overlay) = self.overlays.get_mut(overlay_id) {
                let col = index % cols;
                let row = index / cols;

                overlay.state.position = OverlayPosition {
                    x: col as f32 * overlay_width,
                    y: row as f32 * overlay_height,
                };
                overlay.state.size = OverlaySize {
                    width: overlay_width,
                    height: overlay_height,
                };
            }
        }

        Ok(())
    }

    /// Arrange overlays in docked layout
    fn arrange_docked(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Dock overlays to edges of the viewport
        let viewport_width = 1200.0;
        let viewport_height = 800.0;
        let dock_width = 300.0;
        let dock_height = 200.0;

        for (index, overlay_id) in self.active_overlays.iter().enumerate() {
            if let Some(overlay) = self.overlays.get_mut(overlay_id) {
                match index % 4 {
                    0 => { // Left dock
                        overlay.state.position = OverlayPosition { x: 0.0, y: 0.0 };
                        overlay.state.size = OverlaySize { width: dock_width, height: viewport_height };
                    },
                    1 => { // Right dock
                        overlay.state.position = OverlayPosition { x: viewport_width - dock_width, y: 0.0 };
                        overlay.state.size = OverlaySize { width: dock_width, height: viewport_height };
                    },
                    2 => { // Top dock
                        overlay.state.position = OverlayPosition { x: dock_width, y: 0.0 };
                        overlay.state.size = OverlaySize { width: viewport_width - 2.0 * dock_width, height: dock_height };
                    },
                    3 => { // Bottom dock
                        overlay.state.position = OverlayPosition { x: dock_width, y: viewport_height - dock_height };
                        overlay.state.size = OverlaySize { width: viewport_width - 2.0 * dock_width, height: dock_height };
                    },
                    _ => unreachable!(),
                }
            }
        }

        Ok(())
    }

    /// Arrange overlays in tabbed layout
    fn arrange_tabbed(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // All overlays occupy the same space, with tabs for switching
        let viewport_width = 1200.0;
        let viewport_height = 800.0;
        let tab_height = 30.0;

        for overlay_id in &self.active_overlays {
            if let Some(overlay) = self.overlays.get_mut(overlay_id) {
                overlay.state.position = OverlayPosition { x: 0.0, y: tab_height };
                overlay.state.size = OverlaySize { 
                    width: viewport_width, 
                    height: viewport_height - tab_height 
                };
            }
        }

        Ok(())
    }

    /// Snap position to grid
    fn snap_to_grid(&self, position: OverlayPosition) -> OverlayPosition {
        let grid_size = self.global_settings.grid_size;
        OverlayPosition {
            x: (position.x / grid_size).round() * grid_size,
            y: (position.y / grid_size).round() * grid_size,
        }
    }

    /// Resolve overlay collisions
    fn resolve_collisions(&mut self, _overlay_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Collision detection and resolution logic would be implemented here
        // This is a placeholder for the collision resolution system
        Ok(())
    }

    /// Setup default debug overlays
    fn setup_default_overlays(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let default_overlays = vec![
            DebugOverlay {
                id: "audio_controls".to_string(),
                title: "Audio Controls".to_string(),
                state: OverlayState {
                    visible: false,
                    position: OverlayPosition { x: 10.0, y: 10.0 },
                    size: OverlaySize { width: 300.0, height: 200.0 },
                    transparency: 0.9,
                },
                resizable: true,
                movable: true,
                collapsible: true,
                z_index: 100,
                component_type: OverlayComponentType::AudioControls,
            },
            DebugOverlay {
                id: "debug_interface".to_string(),
                title: "Debug Interface".to_string(),
                state: OverlayState {
                    visible: false,
                    position: OverlayPosition { x: 320.0, y: 10.0 },
                    size: OverlaySize { width: 400.0, height: 300.0 },
                    transparency: 0.9,
                },
                resizable: true,
                movable: true,
                collapsible: true,
                z_index: 101,
                component_type: OverlayComponentType::DebugInterface,
            },
            DebugOverlay {
                id: "performance_monitor".to_string(),
                title: "Performance Monitor".to_string(),
                state: OverlayState {
                    visible: false,
                    position: OverlayPosition { x: 10.0, y: 220.0 },
                    size: OverlaySize { width: 350.0, height: 180.0 },
                    transparency: 0.9,
                },
                resizable: true,
                movable: true,
                collapsible: true,
                z_index: 102,
                component_type: OverlayComponentType::PerformanceMonitor,
            },
        ];

        for overlay in default_overlays {
            self.register_overlay(overlay)?;
        }

        Ok(())
    }

    /// Initialize layout system
    fn initialize_layout_system(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize the layout management system
        // This would setup event listeners, calculate initial positions, etc.
        Ok(())
    }
}

#[cfg(test)]
#[cfg(debug_assertions)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_manager_creation() {
        let manager = OverlayManager::new();
        assert!(manager.is_ok());
        
        let manager = manager.unwrap();
        assert!(!manager.initialized);
        assert_eq!(manager.overlays.len(), 0);
        assert_eq!(manager.active_overlays.len(), 0);
    }

    #[test]
    fn test_overlay_registration() {
        let mut manager = OverlayManager::new().unwrap();
        
        let overlay = DebugOverlay {
            id: "test_overlay".to_string(),
            title: "Test Overlay".to_string(),
            state: OverlayState::default(),
            resizable: true,
            movable: true,
            collapsible: true,
            z_index: 100,
            component_type: OverlayComponentType::DebugInterface,
        };

        assert!(manager.register_overlay(overlay).is_ok());
        assert_eq!(manager.overlays.len(), 1);
        assert!(manager.get_overlay("test_overlay").is_some());
    }

    #[test]
    fn test_overlay_show_hide() {
        let mut manager = OverlayManager::new().unwrap();
        
        let overlay = DebugOverlay {
            id: "test_overlay".to_string(),
            title: "Test Overlay".to_string(),
            state: OverlayState::default(),
            resizable: true,
            movable: true,
            collapsible: true,
            z_index: 100,
            component_type: OverlayComponentType::DebugInterface,
        };

        manager.register_overlay(overlay).unwrap();
        
        assert!(manager.show_overlay("test_overlay").is_ok());
        assert_eq!(manager.active_overlays.len(), 1);
        
        assert!(manager.hide_overlay("test_overlay").is_ok());
        assert_eq!(manager.active_overlays.len(), 0);
    }

    #[test]
    fn test_layout_mode_changes() {
        let mut manager = OverlayManager::new().unwrap();
        manager.initialize().unwrap();
        
        assert!(manager.set_layout_mode(LayoutMode::Tiled).is_ok());
        match manager.get_layout_mode() {
            LayoutMode::Tiled => {},
            _ => panic!("Layout mode was not set correctly"),
        }
    }
} 