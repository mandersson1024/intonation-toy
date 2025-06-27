//! # Theme Selection UI
//!
//! The Theme Selection UI provides a simple interface for users to select
//! between available themes. It includes theme previews, instant switching,
//! and persistence across browser sessions.

use crate::modules::presentation_layer::theme_manager::*;
use std::rc::Rc;
use std::cell::RefCell;

/// Theme selection state
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeSelectionState {
    pub available_themes: Vec<ThemeMetadata>,
    pub current_theme: UserThemeChoice,
    pub preview_theme: Option<UserThemeChoice>,
    pub theme_switching: bool,
    pub last_switch_time: f64,
}

impl Default for ThemeSelectionState {
    fn default() -> Self {
        Self {
            available_themes: UserThemeChoice::all()
                .into_iter()
                .map(|choice| ThemeMetadata {
                    choice,
                    display_name: choice.display_name(),
                    description: choice.description(),
                    preview_image: None,
                })
                .collect(),
            current_theme: UserThemeChoice::Playful,
            preview_theme: None,
            theme_switching: false,
            last_switch_time: 0.0,
        }
    }
}

/// Theme selection events
#[derive(Debug, Clone, PartialEq)]
pub enum ThemeSelectionEvent {
    /// User selected a theme for preview
    PreviewTheme(UserThemeChoice),
    /// User confirmed theme selection
    SelectTheme(UserThemeChoice),
    /// Clear theme preview
    ClearPreview,
    /// Theme switching started
    SwitchingStarted,
    /// Theme switching completed
    SwitchingCompleted(UserThemeChoice, f64), // theme, duration_ms
    /// Theme switch failed
    SwitchingFailed(ThemeError),
}

/// Theme selection UI controller
pub struct ThemeSelection {
    state: ThemeSelectionState,
    theme_registry: Rc<RefCell<ThemeRegistry>>,
    switch_start_time: Option<f64>,
}

impl ThemeSelection {
    /// Create a new theme selection controller
    pub fn new() -> Self {
        Self {
            state: ThemeSelectionState::default(),
            theme_registry: Rc::new(RefCell::new(ThemeRegistry::new())),
            switch_start_time: None,
        }
    }
    
    /// Get current state
    pub fn get_state(&self) -> &ThemeSelectionState {
        &self.state
    }
    
    /// Initialize theme selection with persisted choice
    pub fn initialize(&mut self) -> Result<(), ThemeError> {
        // Load persisted theme choice
        let registry = self.theme_registry.borrow();
        let persisted_theme = registry.load_persisted_theme_choice()
            .unwrap_or(UserThemeChoice::Playful);
        
        self.state.current_theme = persisted_theme;
        Ok(())
    }
    
    /// Handle theme selection events
    pub fn handle_event(&mut self, event: ThemeSelectionEvent) -> Result<(), ThemeError> {
        match event {
            ThemeSelectionEvent::PreviewTheme(theme) => {
                self.state.preview_theme = Some(theme);
            }
            
            ThemeSelectionEvent::SelectTheme(theme) => {
                self.start_theme_switch(theme)?;
            }
            
            ThemeSelectionEvent::ClearPreview => {
                self.state.preview_theme = None;
            }
            
            ThemeSelectionEvent::SwitchingStarted => {
                self.state.theme_switching = true;
                self.switch_start_time = web_sys::window()
                    .and_then(|w| w.performance())
                    .map(|p| p.now());
            }
            
            ThemeSelectionEvent::SwitchingCompleted(theme, duration) => {
                self.state.theme_switching = false;
                self.state.current_theme = theme;
                self.state.last_switch_time = duration;
                self.switch_start_time = None;
            }
            
            ThemeSelectionEvent::SwitchingFailed(error) => {
                self.state.theme_switching = false;
                self.switch_start_time = None;
                return Err(error);
            }
        }
        
        Ok(())
    }
    
    /// Start theme switching process
    fn start_theme_switch(&mut self, theme: UserThemeChoice) -> Result<(), ThemeError> {
        if self.state.theme_switching {
            return Ok(()); // Already switching
        }
        
        // Start switch timing
        self.handle_event(ThemeSelectionEvent::SwitchingStarted)?;
        
        // Apply theme change
        let switch_result = {
            let mut registry = self.theme_registry.borrow_mut();
            registry.set_theme(theme)
        };
        
        match switch_result {
            Ok(()) => {
                let duration = self.calculate_switch_duration();
                self.handle_event(ThemeSelectionEvent::SwitchingCompleted(theme, duration))?;
            }
            Err(error) => {
                self.handle_event(ThemeSelectionEvent::SwitchingFailed(error.clone()))?;
                return Err(error);
            }
        }
        
        Ok(())
    }
    
    /// Calculate theme switch duration
    fn calculate_switch_duration(&self) -> f64 {
        if let Some(start_time) = self.switch_start_time {
            if let Some(current_time) = web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now()) {
                return current_time - start_time;
            }
        }
        0.0
    }
    
    /// Get theme preview information
    pub fn get_theme_preview(&self, choice: UserThemeChoice) -> Result<ThemePreview, ThemeError> {
        let registry = self.theme_registry.borrow();
        registry.get_theme_preview(choice)
    }
    
    /// Check if theme switching is within performance requirements (<100ms)
    pub fn is_switch_performance_acceptable(&self) -> bool {
        self.state.last_switch_time < 100.0 || self.state.last_switch_time == 0.0
    }
    
    /// Get performance statistics
    pub fn get_performance_stats(&self) -> ThemeSelectionStats {
        ThemeSelectionStats {
            last_switch_duration_ms: self.state.last_switch_time,
            total_themes_available: self.state.available_themes.len(),
            current_theme: self.state.current_theme,
            performance_acceptable: self.is_switch_performance_acceptable(),
        }
    }
}

/// Theme selection performance statistics
#[derive(Debug, Clone)]
pub struct ThemeSelectionStats {
    pub last_switch_duration_ms: f64,
    pub total_themes_available: usize,
    pub current_theme: UserThemeChoice,
    pub performance_acceptable: bool,
}

/// HTML generation for theme selection UI (simplified without full web framework)
impl ThemeSelection {
    /// Generate theme selection HTML structure
    pub fn generate_html(&self) -> String {
        let mut html = String::new();
        
        html.push_str("<div class=\"theme-selection-container\">");
        html.push_str("<h3>Choose Your Theme</h3>");
        
        // Theme options
        html.push_str("<div class=\"theme-options\">");
        for theme_meta in &self.state.available_themes {
            let is_current = theme_meta.choice == self.state.current_theme;
            let is_preview = Some(theme_meta.choice) == self.state.preview_theme;
            
            let mut classes = vec!["theme-option"];
            if is_current { classes.push("current"); }
            if is_preview { classes.push("preview"); }
            if self.state.theme_switching { classes.push("switching"); }
            
            html.push_str(&format!(
                "<div class=\"{}\" data-theme=\"{:?}\">",
                classes.join(" "),
                theme_meta.choice
            ));
            
            // Theme preview
            html.push_str("<div class=\"theme-preview\">");
            self.generate_theme_preview_html(&mut html, theme_meta.choice);
            html.push_str("</div>");
            
            // Theme info
            html.push_str(&format!(
                "<div class=\"theme-info\">
                    <h4>{}</h4>
                    <p>{}</p>
                </div>",
                theme_meta.display_name,
                theme_meta.description
            ));
            
            html.push_str("</div>");
        }
        html.push_str("</div>");
        
        // Performance indicator
        if self.state.last_switch_time > 0.0 {
            let status = if self.is_switch_performance_acceptable() {
                "good"
            } else {
                "warning"
            };
            
            html.push_str(&format!(
                "<div class=\"performance-indicator {}\">
                    Last switch: {:.1}ms
                </div>",
                status,
                self.state.last_switch_time
            ));
        }
        
        html.push_str("</div>");
        html
    }
    
    /// Generate theme preview HTML
    fn generate_theme_preview_html(&self, html: &mut String, choice: UserThemeChoice) {
        if let Ok(preview) = self.get_theme_preview(choice) {
            html.push_str("<div class=\"color-swatches\">");
            for (i, color) in preview.dominant_colors.iter().enumerate() {
                html.push_str(&format!(
                    "<div class=\"color-swatch\" style=\"background-color: rgba({}, {}, {}, {})\"></div>",
                    (color[0] * 255.0) as u8,
                    (color[1] * 255.0) as u8,
                    (color[2] * 255.0) as u8,
                    color[3]
                ));
            }
            html.push_str("</div>");
            
            html.push_str(&format!(
                "<div class=\"animation-style\">{}</div>",
                preview.animation_style
            ));
        }
    }
    
    /// Generate CSS for theme selection UI
    pub fn generate_css() -> &'static str {
        "
        .theme-selection-container {
            padding: 20px;
            background: rgba(0, 0, 0, 0.8);
            border-radius: 8px;
            color: white;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }
        
        .theme-selection-container h3 {
            margin: 0 0 16px 0;
            font-weight: 600;
            font-size: 18px;
        }
        
        .theme-options {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 16px;
        }
        
        .theme-option {
            padding: 16px;
            border: 2px solid transparent;
            border-radius: 8px;
            background: rgba(255, 255, 255, 0.1);
            cursor: pointer;
            transition: all 0.2s ease;
        }
        
        .theme-option:hover {
            background: rgba(255, 255, 255, 0.15);
            border-color: rgba(255, 255, 255, 0.3);
        }
        
        .theme-option.current {
            border-color: #4CAF50;
            background: rgba(76, 175, 80, 0.2);
        }
        
        .theme-option.preview {
            border-color: #2196F3;
            background: rgba(33, 150, 243, 0.2);
        }
        
        .theme-option.switching {
            opacity: 0.6;
            pointer-events: none;
        }
        
        .theme-preview {
            margin-bottom: 12px;
        }
        
        .color-swatches {
            display: flex;
            gap: 4px;
            margin-bottom: 8px;
        }
        
        .color-swatch {
            width: 24px;
            height: 24px;
            border-radius: 4px;
            border: 1px solid rgba(255, 255, 255, 0.2);
        }
        
        .animation-style {
            font-size: 12px;
            color: rgba(255, 255, 255, 0.7);
            font-style: italic;
        }
        
        .theme-info h4 {
            margin: 0 0 4px 0;
            font-size: 16px;
            font-weight: 600;
        }
        
        .theme-info p {
            margin: 0;
            font-size: 14px;
            color: rgba(255, 255, 255, 0.8);
            line-height: 1.4;
        }
        
        .performance-indicator {
            margin-top: 16px;
            padding: 8px 12px;
            border-radius: 4px;
            font-size: 12px;
            text-align: center;
        }
        
        .performance-indicator.good {
            background: rgba(76, 175, 80, 0.2);
            color: #4CAF50;
        }
        
        .performance-indicator.warning {
            background: rgba(255, 152, 0, 0.2);
            color: #FF9800;
        }
        "
    }
}

impl Default for ThemeSelection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_theme_selection_creation() {
        let theme_selection = ThemeSelection::new();
        assert_eq!(theme_selection.state.available_themes.len(), 2);
        assert_eq!(theme_selection.state.current_theme, UserThemeChoice::Playful);
        assert!(!theme_selection.state.theme_switching);
    }
    
    #[test]
    fn test_theme_preview_event() {
        let mut theme_selection = ThemeSelection::new();
        
        theme_selection.handle_event(
            ThemeSelectionEvent::PreviewTheme(UserThemeChoice::Scientific)
        ).unwrap();
        
        assert_eq!(theme_selection.state.preview_theme, Some(UserThemeChoice::Scientific));
    }
    
    #[test]
    fn test_theme_selection_event() {
        let mut theme_selection = ThemeSelection::new();
        
        theme_selection.handle_event(
            ThemeSelectionEvent::SelectTheme(UserThemeChoice::Scientific)
        ).unwrap();
        
        assert_eq!(theme_selection.state.current_theme, UserThemeChoice::Scientific);
        assert!(!theme_selection.state.theme_switching);
    }
    
    #[test]
    fn test_performance_tracking() {
        let mut theme_selection = ThemeSelection::new();
        
        // Simulate fast switch
        theme_selection.state.last_switch_time = 50.0;
        assert!(theme_selection.is_switch_performance_acceptable());
        
        // Simulate slow switch
        theme_selection.state.last_switch_time = 150.0;
        assert!(!theme_selection.is_switch_performance_acceptable());
    }
    
    #[test]
    fn test_html_generation() {
        let theme_selection = ThemeSelection::new();
        let html = theme_selection.generate_html();
        
        assert!(html.contains("theme-selection-container"));
        assert!(html.contains("Choose Your Theme"));
        assert!(html.contains("Playful"));
        assert!(html.contains("Scientific"));
    }
    
    #[test]
    fn test_css_generation() {
        let css = ThemeSelection::generate_css();
        assert!(css.contains(".theme-selection-container"));
        assert!(css.contains(".theme-option"));
        assert!(css.contains(".performance-indicator"));
    }
}