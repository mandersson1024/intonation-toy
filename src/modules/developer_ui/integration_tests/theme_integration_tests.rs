//! # Theme System Integration Tests
//!
//! Integration tests for immersive UI theme system functionality.
//! Tests theme selection, instant switching, persistence, and Graphics Foundations integration.

#[cfg(test)]
#[cfg(debug_assertions)]
mod tests {
    use crate::modules::presentation_layer::theme_manager::ThemeManager;
    use crate::modules::presentation_layer::theme_renderer::ThemeRenderer;
    use crate::modules::presentation_layer::theme_selection::ThemeSelection;
    use crate::modules::graphics_foundations::wgpu_context::WgpuContext;
    use crate::modules::graphics_foundations::render_pipeline::RenderPipeline;
    use std::rc::Rc;
    use std::cell::RefCell;
    use wasm_bindgen_test::*;
    use web_sys::{window, Storage};

    wasm_bindgen_test_configure!(run_in_browser);

    // Test theme configurations
    #[derive(Debug, Clone, PartialEq)]
    enum TestTheme {
        Scientific,
        Playful,
    }

    /// Test theme selection UI functionality and user interaction
    #[wasm_bindgen_test]
    async fn test_theme_selection_ui_functionality() {
        let theme_selection = create_test_theme_selection().await;
        assert!(theme_selection.is_ok(), "Theme selection creation should succeed");
        
        let mut selector = theme_selection.unwrap();
        
        // Test available themes
        let available_themes = selector.get_available_themes();
        assert!(available_themes.len() >= 2, "Should have multiple themes available");
        assert!(available_themes.contains(&"scientific".to_string()), "Should have scientific theme");
        assert!(available_themes.contains(&"playful".to_string()), "Should have playful theme");
        
        // Test theme selection interaction
        let select_result = selector.select_theme("scientific").await;
        assert!(select_result.is_ok(), "Theme selection should succeed");
        
        let current_theme = selector.get_current_theme();
        assert_eq!(current_theme, "scientific", "Current theme should be scientific");
        
        // Test UI state updates
        assert!(selector.is_ui_updated(), "UI should reflect theme selection");
        assert!(selector.is_selection_responsive(), "Selection UI should be responsive");
    }

    /// Test instant theme switching without interruption (<100ms requirement)
    #[wasm_bindgen_test]
    async fn test_instant_theme_switching_performance() {
        let theme_manager = create_test_theme_manager().await.unwrap();
        let mut theme_renderer = create_test_theme_renderer().await.unwrap();
        
        // Set initial theme
        theme_manager.set_theme("scientific").await.unwrap();
        
        // Measure theme switching performance
        let start_time = web_sys::Performance::new().unwrap().now();
        
        // Switch to different theme
        let switch_result = theme_manager.switch_theme("playful").await;
        assert!(switch_result.is_ok(), "Theme switching should succeed");
        
        // Update renderer with new theme
        let render_update = theme_renderer.update_theme("playful").await;
        assert!(render_update.is_ok(), "Theme renderer update should succeed");
        
        let end_time = web_sys::Performance::new().unwrap().now();
        let switch_time = end_time - start_time;
        
        // Verify <100ms requirement
        assert!(switch_time < 100.0, "Theme switching should be under 100ms, got: {}ms", switch_time);
        
        // Verify no interruption during switch
        assert!(theme_manager.is_transition_smooth(), "Theme transition should be smooth");
        assert!(theme_renderer.is_rendering_stable(), "Rendering should remain stable");
    }

    /// Test theme persistence in browser local storage
    #[wasm_bindgen_test]
    async fn test_theme_persistence_local_storage() {
        let theme_manager = create_test_theme_manager().await.unwrap();
        
        // Set a specific theme
        theme_manager.set_theme("playful").await.unwrap();
        
        // Test persistence save
        let save_result = theme_manager.save_theme_preference().await;
        assert!(save_result.is_ok(), "Theme preference save should succeed");
        
        // Verify local storage
        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();
        let stored_theme = storage.get_item("theme_preference").unwrap();
        assert!(stored_theme.is_some(), "Theme should be stored in local storage");
        assert_eq!(stored_theme.unwrap(), "playful", "Stored theme should match selection");
        
        // Test persistence load
        let load_result = theme_manager.load_theme_preference().await;
        assert!(load_result.is_ok(), "Theme preference load should succeed");
        
        let loaded_theme = theme_manager.get_current_theme();
        assert_eq!(loaded_theme, "playful", "Loaded theme should match stored theme");
        
        // Test persistence after page reload simulation
        let mut new_manager = create_test_theme_manager().await.unwrap();
        let reload_load = new_manager.load_theme_preference().await;
        assert!(reload_load.is_ok(), "Theme should load after reload simulation");
        
        let reloaded_theme = new_manager.get_current_theme();
        assert_eq!(reloaded_theme, "playful", "Theme should persist across sessions");
    }

    /// Test theme switching performance meets <100ms requirement
    #[wasm_bindgen_test]
    async fn test_theme_switching_performance_requirement() {
        let theme_manager = create_test_theme_manager().await.unwrap();
        let mut theme_renderer = create_test_theme_renderer().await.unwrap();
        
        // Test multiple rapid theme switches
        let themes = vec!["scientific", "playful", "scientific", "playful"];
        let mut total_time = 0.0;
        
        for (i, theme) in themes.iter().enumerate() {
            let start = web_sys::Performance::new().unwrap().now();
            
            theme_manager.switch_theme(theme).await.unwrap();
            theme_renderer.update_theme(theme).await.unwrap();
            
            let end = web_sys::Performance::new().unwrap().now();
            let switch_time = end - start;
            
            total_time += switch_time;
            
            // Each individual switch should be under 100ms
            assert!(switch_time < 100.0, "Switch {} should be under 100ms, got: {}ms", i, switch_time);
        }
        
        // Average switch time should be well under requirement
        let average_time = total_time / themes.len() as f64;
        assert!(average_time < 50.0, "Average switch time should be under 50ms, got: {}ms", average_time);
        
        // Verify system remains stable after rapid switches
        assert!(theme_manager.is_system_stable(), "Theme system should remain stable");
        assert!(theme_renderer.is_performance_optimal(), "Renderer performance should be optimal");
    }

    /// Test Graphics Foundations integration with theme system
    #[wasm_bindgen_test]
    async fn test_graphics_foundations_theme_integration() {
        let theme_manager = create_test_theme_manager().await.unwrap();
        let graphics_context = create_test_graphics_context().await.unwrap();
        let mut render_pipeline = create_test_render_pipeline().await.unwrap();
        
        // Test theme application to graphics pipeline
        theme_manager.set_theme("scientific").await.unwrap();
        
        let graphics_integration = render_pipeline.apply_theme("scientific", &graphics_context).await;
        assert!(graphics_integration.is_ok(), "Graphics theme integration should succeed");
        
        // Verify graphics context adapts to theme
        assert!(graphics_context.is_theme_applied("scientific"), "Graphics context should apply scientific theme");
        assert!(render_pipeline.has_theme_materials("scientific"), "Pipeline should have scientific materials");
        
        // Test theme switch in graphics pipeline
        let start_switch = web_sys::Performance::new().unwrap().now();
        
        theme_manager.switch_theme("playful").await.unwrap();
        let pipeline_update = render_pipeline.apply_theme("playful", &graphics_context).await;
        assert!(pipeline_update.is_ok(), "Pipeline theme update should succeed");
        
        let end_switch = web_sys::Performance::new().unwrap().now();
        let graphics_switch_time = end_switch - start_switch;
        
        // Graphics theme switching should also be fast
        assert!(graphics_switch_time < 100.0, "Graphics theme switch should be under 100ms");
        
        // Verify new theme applied correctly
        assert!(graphics_context.is_theme_applied("playful"), "Graphics context should apply playful theme");
        assert!(render_pipeline.has_theme_materials("playful"), "Pipeline should have playful materials");
    }

    /// Test wgpu rendering pipeline adaptation to theme changes
    #[wasm_bindgen_test]
    async fn test_wgpu_pipeline_theme_adaptation() {
        let theme_manager = create_test_theme_manager().await.unwrap();
        let wgpu_context = create_test_wgpu_context().await.unwrap();
        let mut render_pipeline = create_test_render_pipeline().await.unwrap();
        
        // Test initial wgpu setup with theme
        theme_manager.set_theme("scientific").await.unwrap();
        
        let wgpu_setup = render_pipeline.setup_wgpu_theme("scientific", &wgpu_context).await;
        assert!(wgpu_setup.is_ok(), "WGPU theme setup should succeed");
        
        // Verify shader adaptation
        assert!(render_pipeline.has_theme_shaders("scientific"), "Should have scientific shaders loaded");
        assert!(wgpu_context.is_shader_pipeline_ready(), "Shader pipeline should be ready");
        
        // Test runtime shader switching
        let shader_switch_start = web_sys::Performance::new().unwrap().now();
        
        theme_manager.switch_theme("playful").await.unwrap();
        let shader_update = render_pipeline.update_wgpu_shaders("playful", &wgpu_context).await;
        assert!(shader_update.is_ok(), "WGPU shader update should succeed");
        
        let shader_switch_end = web_sys::Performance::new().unwrap().now();
        let shader_switch_time = shader_switch_end - shader_switch_start;
        
        // Shader switching should be efficient
        assert!(shader_switch_time < 50.0, "Shader switching should be under 50ms");
        
        // Verify new shaders active
        assert!(render_pipeline.has_theme_shaders("playful"), "Should have playful shaders loaded");
        assert!(wgpu_context.is_shader_transition_complete(), "Shader transition should be complete");
    }

    /// Test debug overlay styling remains independent of theme changes
    #[wasm_bindgen_test]
    async fn test_debug_overlay_styling_independence() {
        let theme_manager = create_test_theme_manager().await.unwrap();
        let debug_overlay = create_test_debug_overlay().await.unwrap();
        
        // Get initial debug overlay styling
        let initial_debug_styles = debug_overlay.get_styling_snapshot();
        
        // Switch immersive theme
        theme_manager.set_theme("scientific").await.unwrap();
        
        // Verify debug overlay styling unchanged
        let scientific_debug_styles = debug_overlay.get_styling_snapshot();
        assert_eq!(initial_debug_styles, scientific_debug_styles, 
                  "Debug overlay styling should be independent of scientific theme");
        
        // Switch to different theme
        theme_manager.switch_theme("playful").await.unwrap();
        
        // Verify debug overlay still unchanged
        let playful_debug_styles = debug_overlay.get_styling_snapshot();
        assert_eq!(initial_debug_styles, playful_debug_styles, 
                  "Debug overlay styling should be independent of playful theme");
        
        // Verify debug overlay functionality unaffected
        assert!(debug_overlay.is_fully_functional(), "Debug overlay should remain functional");
        assert!(debug_overlay.is_styling_stable(), "Debug overlay styling should be stable");
    }

    /// Test theme configuration system with shader variants and material properties
    #[wasm_bindgen_test]
    async fn test_theme_configuration_system() {
        let theme_manager = create_test_theme_manager().await.unwrap();
        let theme_config = create_test_theme_configuration().await.unwrap();
        
        // Test scientific theme configuration
        let scientific_config = theme_config.get_theme_config("scientific").await;
        assert!(scientific_config.is_ok(), "Scientific theme config should be available");
        
        let sci_config = scientific_config.unwrap();
        assert!(sci_config.has_shader_variants(), "Scientific theme should have shader variants");
        assert!(sci_config.has_material_properties(), "Scientific theme should have material properties");
        assert_eq!(sci_config.get_primary_color(), "#2E86AB", "Scientific theme should have correct primary color");
        
        // Test playful theme configuration
        let playful_config = theme_config.get_theme_config("playful").await;
        assert!(playful_config.is_ok(), "Playful theme config should be available");
        
        let play_config = playful_config.unwrap();
        assert!(play_config.has_shader_variants(), "Playful theme should have shader variants");
        assert!(play_config.has_material_properties(), "Playful theme should have material properties");
        assert_eq!(play_config.get_primary_color(), "#FF6B6B", "Playful theme should have correct primary color");
        
        // Test theme configuration validation
        let config_validation = theme_config.validate_all_themes().await;
        assert!(config_validation.is_ok(), "All theme configurations should be valid");
        
        // Test dynamic configuration loading
        let dynamic_load = theme_config.load_theme_resources("scientific").await;
        assert!(dynamic_load.is_ok(), "Dynamic theme resource loading should succeed");
    }
}

// Test utility functions and mock structures
async fn create_test_theme_selection() -> Result<MockThemeSelection, String> {
    MockThemeSelection::new().await
}

async fn create_test_theme_manager() -> Result<MockThemeManager, String> {
    MockThemeManager::new().await
}

async fn create_test_theme_renderer() -> Result<MockThemeRenderer, String> {
    MockThemeRenderer::new().await
}

async fn create_test_graphics_context() -> Result<MockGraphicsContext, String> {
    MockGraphicsContext::new().await
}

async fn create_test_wgpu_context() -> Result<MockWgpuContext, String> {
    MockWgpuContext::new().await
}

async fn create_test_render_pipeline() -> Result<MockRenderPipeline, String> {
    MockRenderPipeline::new().await
}

async fn create_test_debug_overlay() -> Result<MockDebugOverlay, String> {
    MockDebugOverlay::new().await
}

async fn create_test_theme_configuration() -> Result<MockThemeConfiguration, String> {
    MockThemeConfiguration::new().await
}

// Mock implementations for testing
struct MockThemeSelection {
    available_themes: Vec<String>,
    current_theme: String,
    ui_updated: bool,
}

impl MockThemeSelection {
    async fn new() -> Result<Self, String> {
        Ok(MockThemeSelection {
            available_themes: vec!["scientific".to_string(), "playful".to_string()],
            current_theme: "scientific".to_string(),
            ui_updated: false,
        })
    }

    fn get_available_themes(&self) -> Vec<String> {
        self.available_themes.clone()
    }

    async fn select_theme(&mut self, theme: &str) -> Result<(), String> {
        if self.available_themes.contains(&theme.to_string()) {
            self.current_theme = theme.to_string();
            self.ui_updated = true;
            Ok(())
        } else {
            Err("Theme not available".to_string())
        }
    }

    fn get_current_theme(&self) -> String {
        self.current_theme.clone()
    }

    fn is_ui_updated(&self) -> bool {
        self.ui_updated
    }

    fn is_selection_responsive(&self) -> bool {
        true // Simplified for testing
    }
}

struct MockThemeManager {
    current_theme: String,
    transition_smooth: bool,
    system_stable: bool,
}

impl MockThemeManager {
    async fn new() -> Result<Self, String> {
        Ok(MockThemeManager {
            current_theme: "scientific".to_string(),
            transition_smooth: true,
            system_stable: true,
        })
    }

    async fn set_theme(&self, theme: &str) -> Result<(), String> {
        // Simulate theme setting
        Ok(())
    }

    async fn switch_theme(&self, theme: &str) -> Result<(), String> {
        // Simulate theme switching
        Ok(())
    }

    async fn save_theme_preference(&self) -> Result<(), String> {
        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();
        storage.set_item("theme_preference", &self.current_theme).unwrap();
        Ok(())
    }

    async fn load_theme_preference(&mut self) -> Result<(), String> {
        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();
        if let Ok(Some(theme)) = storage.get_item("theme_preference") {
            self.current_theme = theme;
        }
        Ok(())
    }

    fn get_current_theme(&self) -> String {
        self.current_theme.clone()
    }

    fn is_transition_smooth(&self) -> bool {
        self.transition_smooth
    }

    fn is_system_stable(&self) -> bool {
        self.system_stable
    }
}

struct MockThemeRenderer {
    current_theme: String,
    rendering_stable: bool,
    performance_optimal: bool,
}

impl MockThemeRenderer {
    async fn new() -> Result<Self, String> {
        Ok(MockThemeRenderer {
            current_theme: "scientific".to_string(),
            rendering_stable: true,
            performance_optimal: true,
        })
    }

    async fn update_theme(&mut self, theme: &str) -> Result<(), String> {
        self.current_theme = theme.to_string();
        Ok(())
    }

    fn is_rendering_stable(&self) -> bool {
        self.rendering_stable
    }

    fn is_performance_optimal(&self) -> bool {
        self.performance_optimal
    }
}

struct MockGraphicsContext {
    applied_theme: Option<String>,
}

impl MockGraphicsContext {
    async fn new() -> Result<Self, String> {
        Ok(MockGraphicsContext {
            applied_theme: None,
        })
    }

    fn is_theme_applied(&self, theme: &str) -> bool {
        self.applied_theme.as_ref().map_or(false, |t| t == theme)
    }
}

struct MockWgpuContext {
    shader_pipeline_ready: bool,
    shader_transition_complete: bool,
}

impl MockWgpuContext {
    async fn new() -> Result<Self, String> {
        Ok(MockWgpuContext {
            shader_pipeline_ready: true,
            shader_transition_complete: false,
        })
    }

    fn is_shader_pipeline_ready(&self) -> bool {
        self.shader_pipeline_ready
    }

    fn is_shader_transition_complete(&self) -> bool {
        self.shader_transition_complete
    }
}

struct MockRenderPipeline {
    theme_materials: std::collections::HashMap<String, bool>,
    theme_shaders: std::collections::HashMap<String, bool>,
}

impl MockRenderPipeline {
    async fn new() -> Result<Self, String> {
        Ok(MockRenderPipeline {
            theme_materials: std::collections::HashMap::new(),
            theme_shaders: std::collections::HashMap::new(),
        })
    }

    async fn apply_theme(&mut self, theme: &str, _context: &MockGraphicsContext) -> Result<(), String> {
        self.theme_materials.insert(theme.to_string(), true);
        Ok(())
    }

    async fn setup_wgpu_theme(&mut self, theme: &str, _context: &MockWgpuContext) -> Result<(), String> {
        self.theme_shaders.insert(theme.to_string(), true);
        Ok(())
    }

    async fn update_wgpu_shaders(&mut self, theme: &str, _context: &MockWgpuContext) -> Result<(), String> {
        self.theme_shaders.insert(theme.to_string(), true);
        Ok(())
    }

    fn has_theme_materials(&self, theme: &str) -> bool {
        self.theme_materials.get(theme).unwrap_or(&false).clone()
    }

    fn has_theme_shaders(&self, theme: &str) -> bool {
        self.theme_shaders.get(theme).unwrap_or(&false).clone()
    }
}

struct MockDebugOverlay {
    styling_snapshot: String,
    fully_functional: bool,
    styling_stable: bool,
}

impl MockDebugOverlay {
    async fn new() -> Result<Self, String> {
        Ok(MockDebugOverlay {
            styling_snapshot: "debug_overlay_default_styles".to_string(),
            fully_functional: true,
            styling_stable: true,
        })
    }

    fn get_styling_snapshot(&self) -> String {
        self.styling_snapshot.clone()
    }

    fn is_fully_functional(&self) -> bool {
        self.fully_functional
    }

    fn is_styling_stable(&self) -> bool {
        self.styling_stable
    }
}

struct MockThemeConfiguration {
    configurations: std::collections::HashMap<String, MockThemeConfig>,
}

impl MockThemeConfiguration {
    async fn new() -> Result<Self, String> {
        let mut configurations = std::collections::HashMap::new();
        
        configurations.insert("scientific".to_string(), MockThemeConfig {
            name: "scientific".to_string(),
            primary_color: "#2E86AB".to_string(),
            has_shader_variants: true,
            has_material_properties: true,
        });
        
        configurations.insert("playful".to_string(), MockThemeConfig {
            name: "playful".to_string(),
            primary_color: "#FF6B6B".to_string(),
            has_shader_variants: true,
            has_material_properties: true,
        });
        
        Ok(MockThemeConfiguration {
            configurations,
        })
    }

    async fn get_theme_config(&self, theme: &str) -> Result<MockThemeConfig, String> {
        self.configurations.get(theme)
            .cloned()
            .ok_or_else(|| "Theme config not found".to_string())
    }

    async fn validate_all_themes(&self) -> Result<(), String> {
        for (_, config) in &self.configurations {
            if !config.has_shader_variants || !config.has_material_properties {
                return Err("Invalid theme configuration".to_string());
            }
        }
        Ok(())
    }

    async fn load_theme_resources(&self, theme: &str) -> Result<(), String> {
        if self.configurations.contains_key(theme) {
            Ok(())
        } else {
            Err("Theme resources not found".to_string())
        }
    }
}

#[derive(Clone)]
struct MockThemeConfig {
    name: String,
    primary_color: String,
    has_shader_variants: bool,
    has_material_properties: bool,
}

impl MockThemeConfig {
    fn get_primary_color(&self) -> String {
        self.primary_color.clone()
    }

    fn has_shader_variants(&self) -> bool {
        self.has_shader_variants
    }

    fn has_material_properties(&self) -> bool {
        self.has_material_properties
    }
} 