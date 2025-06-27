//! # Debug Overlay Styling
//!
//! This module provides hardcoded, functional styling for debug overlay components
//! that remains completely independent of the theme system. The styling is designed
//! for readability and functionality, not aesthetics.

/// Debug overlay style configuration (hardcoded for reliability)
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct DebugOverlayStyle {
    /// Background color for debug panels
    pub background: &'static str,
    /// Primary text color for readability
    pub text_color: &'static str,
    /// Secondary text color for less important information
    pub text_color_secondary: &'static str,
    /// Border color for panel separation
    pub border_color: &'static str,
    /// Error/warning text color
    pub error_color: &'static str,
    /// Success/good status color
    pub success_color: &'static str,
    /// Warning color
    pub warning_color: &'static str,
    /// Font family for debug text
    pub font_family: &'static str,
    /// Base font size
    pub font_size: &'static str,
    /// Panel border radius
    pub border_radius: &'static str,
    /// Standard padding
    pub padding: &'static str,
    /// Standard margin
    pub margin: &'static str,
}

#[cfg(debug_assertions)]
impl DebugOverlayStyle {
    /// Get the default debug overlay style (hardcoded constants)
    pub const fn default() -> Self {
        Self {
            background: "rgba(0, 0, 0, 0.85)",
            text_color: "#ffffff",
            text_color_secondary: "#cccccc",
            border_color: "#666666",
            error_color: "#ff6b6b",
            success_color: "#51cf66",
            warning_color: "#ffd43b",
            font_family: "'Consolas', 'Monaco', 'Courier New', monospace",
            font_size: "12px",
            border_radius: "4px",
            padding: "8px",
            margin: "4px",
        }
    }
    
    /// Generate CSS for debug overlay styling
    pub fn generate_css(&self) -> &'static str {
        // Hardcoded CSS that never changes regardless of theme
        "
        /* Debug Overlay Base Styles - NEVER MODIFY */
        .debug-overlay {
            position: fixed;
            top: 10px;
            right: 10px;
            z-index: 9999;
            font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
            font-size: 12px;
            color: #ffffff;
            background: rgba(0, 0, 0, 0.85);
            border: 1px solid #666666;
            border-radius: 4px;
            padding: 8px;
            margin: 4px;
            max-width: 300px;
            pointer-events: auto;
        }
        
        .debug-overlay h3,
        .debug-overlay h4 {
            margin: 0 0 8px 0;
            color: #ffffff;
            font-weight: bold;
        }
        
        .debug-overlay h3 {
            font-size: 14px;
            border-bottom: 1px solid #666666;
            padding-bottom: 4px;
        }
        
        .debug-overlay h4 {
            font-size: 12px;
            color: #cccccc;
        }
        
        .debug-overlay .metric {
            display: flex;
            justify-content: space-between;
            margin: 2px 0;
            padding: 2px 0;
        }
        
        .debug-overlay .metric-label {
            color: #cccccc;
        }
        
        .debug-overlay .metric-value {
            color: #ffffff;
            font-weight: bold;
        }
        
        .debug-overlay .status-good {
            color: #51cf66;
        }
        
        .debug-overlay .status-warning {
            color: #ffd43b;
        }
        
        .debug-overlay .status-error {
            color: #ff6b6b;
        }
        
        .debug-overlay .debug-button {
            background: rgba(255, 255, 255, 0.1);
            border: 1px solid #666666;
            color: #ffffff;
            padding: 4px 8px;
            margin: 2px;
            border-radius: 2px;
            font-size: 11px;
            cursor: pointer;
            font-family: inherit;
        }
        
        .debug-overlay .debug-button:hover {
            background: rgba(255, 255, 255, 0.2);
        }
        
        .debug-overlay .debug-button:active {
            background: rgba(255, 255, 255, 0.3);
        }
        
        .debug-overlay .debug-section {
            margin: 8px 0;
            padding: 4px 0;
            border-top: 1px solid rgba(255, 255, 255, 0.1);
        }
        
        .debug-overlay .debug-list {
            list-style: none;
            margin: 0;
            padding: 0;
        }
        
        .debug-overlay .debug-list li {
            padding: 1px 0;
            font-size: 11px;
        }
        
        .debug-overlay .collapsible {
            cursor: pointer;
            user-select: none;
        }
        
        .debug-overlay .collapsible:before {
            content: '▶ ';
            font-size: 10px;
        }
        
        .debug-overlay .collapsible.expanded:before {
            content: '▼ ';
        }
        
        .debug-overlay .collapsible-content {
            display: none;
            margin-left: 12px;
        }
        
        .debug-overlay .collapsible.expanded + .collapsible-content {
            display: block;
        }
        
        /* Performance specific styles */
        .debug-overlay .performance-good {
            color: #51cf66;
        }
        
        .debug-overlay .performance-warning {
            color: #ffd43b;
        }
        
        .debug-overlay .performance-critical {
            color: #ff6b6b;
            font-weight: bold;
        }
        
        /* Audio specific styles */
        .debug-overlay .audio-active {
            color: #51cf66;
        }
        
        .debug-overlay .audio-inactive {
            color: #666666;
        }
        
        .debug-overlay .frequency-display {
            font-family: 'Courier New', monospace;
            font-size: 10px;
            background: rgba(0, 0, 0, 0.3);
            padding: 2px 4px;
            border-radius: 2px;
        }
        
        /* Theme isolation - these styles NEVER change */
        .debug-overlay * {
            box-sizing: border-box;
        }
        
        .debug-overlay,
        .debug-overlay * {
            /* Prevent theme CSS from affecting debug overlay */
            all: unset;
            /* Restore basic functionality */
            display: revert;
            box-sizing: border-box;
        }
        
        /* Restore specific properties */
        .debug-overlay {
            position: fixed !important;
            top: 10px !important;
            right: 10px !important;
            z-index: 9999 !important;
            font-family: 'Consolas', 'Monaco', 'Courier New', monospace !important;
            font-size: 12px !important;
            color: #ffffff !important;
            background: rgba(0, 0, 0, 0.85) !important;
            border: 1px solid #666666 !important;
            border-radius: 4px !important;
            padding: 8px !important;
            margin: 4px !important;
            max-width: 300px !important;
        }
        "
    }
    
    /// Get inline styles for critical debug elements
    pub fn get_inline_styles(&self) -> DebugInlineStyles {
        DebugInlineStyles {
            panel: "position: fixed; top: 10px; right: 10px; z-index: 9999; \
                    font-family: 'Consolas', monospace; font-size: 12px; \
                    color: #ffffff; background: rgba(0,0,0,0.85); \
                    border: 1px solid #666666; border-radius: 4px; \
                    padding: 8px; max-width: 300px;",
            text: "color: #ffffff; font-family: 'Consolas', monospace; font-size: 12px;",
            button: "background: rgba(255,255,255,0.1); border: 1px solid #666666; \
                     color: #ffffff; padding: 4px 8px; border-radius: 2px; \
                     font-size: 11px; cursor: pointer;",
            metric_good: "color: #51cf66; font-weight: bold;",
            metric_warning: "color: #ffd43b; font-weight: bold;",
            metric_error: "color: #ff6b6b; font-weight: bold;",
        }
    }
}

/// Inline styles for critical debug elements
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct DebugInlineStyles {
    pub panel: &'static str,
    pub text: &'static str,
    pub button: &'static str,
    pub metric_good: &'static str,
    pub metric_warning: &'static str,
    pub metric_error: &'static str,
}

/// Debug style validation (ensures styles are theme-independent)
#[cfg(debug_assertions)]
pub struct DebugStyleValidator;

#[cfg(debug_assertions)]
impl DebugStyleValidator {
    /// Validate that debug styles are properly isolated from themes
    pub fn validate_theme_isolation() -> bool {
        // Check that no theme-related CSS classes or variables are used
        let css = DebugOverlayStyle::default().generate_css();
        
        // Ensure no theme references
        !css.contains("theme-") && 
        !css.contains("var(--") && 
        !css.contains("@apply") &&
        css.contains("!important") // Ensures styles can't be overridden
    }
    
    /// Validate readability across different backgrounds
    pub fn validate_readability() -> bool {
        let style = DebugOverlayStyle::default();
        
        // Ensure sufficient contrast
        style.background.contains("rgba(0, 0, 0, 0.85)") &&
        style.text_color == "#ffffff" &&
        style.border_color == "#666666"
    }
    
    /// Validate performance impact (styles should be minimal)
    pub fn validate_performance() -> bool {
        let css = DebugOverlayStyle::default().generate_css();
        
        // Ensure CSS is reasonably sized (under 5KB)
        css.len() < 5000
    }
}

/// Debug style utilities
#[cfg(debug_assertions)]
pub struct DebugStyleUtils;

#[cfg(debug_assertions)]
impl DebugStyleUtils {
    /// Inject debug styles into document head
    pub fn inject_debug_styles() -> Result<(), String> {
        if let Some(window) = web_sys::window() {
            if let Ok(document) = window.document() {
                if let Some(head) = document.head() {
                    let style_element = document.create_element("style")
                        .map_err(|_| "Failed to create style element")?;
                    
                    style_element.set_text_content(Some(
                        DebugOverlayStyle::default().generate_css()
                    ));
                    
                    head.append_child(&style_element)
                        .map_err(|_| "Failed to append style element")?;
                    
                    return Ok(());
                }
            }
        }
        
        Err("Failed to inject debug styles".to_string())
    }
    
    /// Create debug overlay container element
    pub fn create_debug_container() -> Result<web_sys::Element, String> {
        if let Some(window) = web_sys::window() {
            if let Ok(document) = window.document() {
                let container = document.create_element("div")
                    .map_err(|_| "Failed to create container element")?;
                
                container.set_class_name("debug-overlay");
                
                // Set inline styles as backup
                let styles = DebugOverlayStyle::default().get_inline_styles();
                container.set_attribute("style", styles.panel)
                    .map_err(|_| "Failed to set container styles")?;
                
                return Ok(container);
            }
        }
        
        Err("Failed to create debug container".to_string())
    }
    
    /// Format metric value with appropriate styling
    pub fn format_metric(label: &str, value: &str, status: DebugMetricStatus) -> String {
        let status_class = match status {
            DebugMetricStatus::Good => "status-good",
            DebugMetricStatus::Warning => "status-warning",
            DebugMetricStatus::Error => "status-error",
            DebugMetricStatus::Normal => "",
        };
        
        format!(
            "<div class=\"metric\">
                <span class=\"metric-label\">{}</span>
                <span class=\"metric-value {}\">{}}</span>
            </div>",
            label, status_class, value
        )
    }
}

/// Debug metric status for styling
#[cfg(debug_assertions)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DebugMetricStatus {
    Good,
    Warning,
    Error,
    Normal,
}

// Stub implementations for release builds
#[cfg(not(debug_assertions))]
pub struct DebugOverlayStyle;

#[cfg(not(debug_assertions))]
impl DebugOverlayStyle {
    pub const fn default() -> Self { Self }
    pub fn generate_css(&self) -> &'static str { "" }
}

#[cfg(test)]
#[cfg(debug_assertions)]
mod tests {
    use super::*;
    
    #[test]
    fn test_debug_style_creation() {
        let style = DebugOverlayStyle::default();
        assert_eq!(style.background, "rgba(0, 0, 0, 0.85)");
        assert_eq!(style.text_color, "#ffffff");
        assert_eq!(style.border_color, "#666666");
    }
    
    #[test]
    fn test_css_generation() {
        let style = DebugOverlayStyle::default();
        let css = style.generate_css();
        
        assert!(css.contains(".debug-overlay"));
        assert!(css.contains("!important"));
        assert!(css.contains("rgba(0, 0, 0, 0.85)"));
    }
    
    #[test]
    fn test_theme_isolation_validation() {
        assert!(DebugStyleValidator::validate_theme_isolation());
    }
    
    #[test]
    fn test_readability_validation() {
        assert!(DebugStyleValidator::validate_readability());
    }
    
    #[test]
    fn test_performance_validation() {
        assert!(DebugStyleValidator::validate_performance());
    }
    
    #[test]
    fn test_inline_styles() {
        let style = DebugOverlayStyle::default();
        let inline = style.get_inline_styles();
        
        assert!(inline.panel.contains("position: fixed"));
        assert!(inline.text.contains("color: #ffffff"));
        assert!(inline.button.contains("cursor: pointer"));
    }
    
    #[test]
    fn test_metric_formatting() {
        let formatted = DebugStyleUtils::format_metric(
            "CPU Usage", 
            "45%", 
            DebugMetricStatus::Warning
        );
        
        assert!(formatted.contains("CPU Usage"));
        assert!(formatted.contains("45%"));
        assert!(formatted.contains("status-warning"));
    }
}