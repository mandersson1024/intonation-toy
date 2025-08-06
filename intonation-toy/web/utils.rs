/// Web utility functions for color conversion and other common operations

/// Convert RGB values to CSS rgba() string
pub fn rgba_to_css(rgb: [f32; 3], alpha: f32) -> String {
    format!("rgba({}, {}, {}, {})", 
        (rgb[0] * 255.0) as u8, 
        (rgb[1] * 255.0) as u8, 
        (rgb[2] * 255.0) as u8,
        alpha
    )
}

/// Convert RGB values to CSS rgb() string
pub fn rgb_to_css(rgb: [f32; 3]) -> String {
    format!("rgb({}, {}, {})", 
        (rgb[0] * 255.0) as u8, 
        (rgb[1] * 255.0) as u8, 
        (rgb[2] * 255.0) as u8
    )
}