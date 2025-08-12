//! Web utility functions for color conversion and other common operations.

/// Convert RGB values to CSS rgba() string.
/// 
/// Takes floating-point RGB values in the range [0.0, 1.0] and an alpha value,
/// converting them to a CSS rgba() string with integer RGB values [0, 255].
/// 
/// # Arguments
/// 
/// * `rgb` - Array of three f32 values representing red, green, and blue channels [0.0-1.0]
/// * `alpha` - Alpha (opacity) value [0.0-1.0]
/// 
/// # Returns
/// 
/// A CSS rgba() string like "rgba(255, 128, 0, 0.5)"
/// 
/// # Example
/// 
/// ```rust
/// let color = rgba_to_css([1.0, 0.5, 0.0], 0.5);
/// assert_eq!(color, "rgba(255, 127, 0, 0.5)");
/// ```
pub fn rgba_to_css(rgb: [f32; 3], alpha: f32) -> String {
    format!("rgba({}, {}, {}, {})", 
        (rgb[0] * 255.0) as u8, 
        (rgb[1] * 255.0) as u8, 
        (rgb[2] * 255.0) as u8,
        alpha
    )
}

/// Convert RGB values to CSS rgb() string.
/// 
/// Takes floating-point RGB values in the range [0.0, 1.0] and converts them
/// to a CSS rgb() string with integer RGB values [0, 255].
/// 
/// # Arguments
/// 
/// * `rgb` - Array of three f32 values representing red, green, and blue channels [0.0-1.0]
/// 
/// # Returns
/// 
/// A CSS rgb() string like "rgb(255, 128, 0)"
/// 
/// # Example
/// 
/// ```rust
/// let color = rgb_to_css([1.0, 0.5, 0.0]);
/// assert_eq!(color, "rgb(255, 127, 0)");
/// ```
pub fn rgb_to_css(rgb: [f32; 3]) -> String {
    format!("rgb({}, {}, {})", 
        (rgb[0] * 255.0) as u8, 
        (rgb[1] * 255.0) as u8, 
        (rgb[2] * 255.0) as u8
    )
}