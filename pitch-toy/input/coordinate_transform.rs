//! Coordinate Transformation
//!
//! This module provides coordinate transformation utilities for converting
//! screen space coordinates to graphics space coordinates and vice versa.

use web_sys::HtmlCanvasElement;

/// Coordinate transformer for converting between screen and graphics space
#[derive(Debug, Clone)]
pub struct CoordinateTransformer {
    canvas_width: f32,
    canvas_height: f32,
    graphics_width: f32,
    graphics_height: f32,
}

impl CoordinateTransformer {
    /// Create a new coordinate transformer
    /// 
    /// # Arguments
    /// * `canvas` - HTML canvas element to get dimensions from
    /// * `graphics_width` - Width of the graphics coordinate system
    /// * `graphics_height` - Height of the graphics coordinate system
    pub fn new(canvas: &HtmlCanvasElement, graphics_width: f32, graphics_height: f32) -> Self {
        Self {
            canvas_width: canvas.width() as f32,
            canvas_height: canvas.height() as f32,
            graphics_width,
            graphics_height,
        }
    }
    
    /// Update canvas dimensions (call when canvas is resized)
    pub fn update_canvas_size(&mut self, canvas: &HtmlCanvasElement) {
        self.canvas_width = canvas.width() as f32;
        self.canvas_height = canvas.height() as f32;
    }
    
    /// Update graphics dimensions (call when graphics coordinate system changes)
    pub fn update_graphics_size(&mut self, width: f32, height: f32) {
        self.graphics_width = width;
        self.graphics_height = height;
    }
    
    /// Transform screen coordinates to graphics coordinates
    /// 
    /// # Arguments
    /// * `screen_x` - X coordinate in screen space (pixels from left edge)
    /// * `screen_y` - Y coordinate in screen space (pixels from top edge)
    /// 
    /// # Returns
    /// Graphics coordinates as (x, y) tuple
    pub fn screen_to_graphics(&self, screen_x: f32, screen_y: f32) -> (f32, f32) {
        // Convert screen coordinates to normalized coordinates (0.0 to 1.0)
        let norm_x = screen_x / self.canvas_width;
        let norm_y = screen_y / self.canvas_height;
        
        // Convert normalized coordinates to graphics space
        // Graphics space: (0,0) is center, X increases right, Y increases up
        let graphics_x = (norm_x - 0.5) * self.graphics_width;
        let graphics_y = (0.5 - norm_y) * self.graphics_height; // Flip Y axis
        
        (graphics_x, graphics_y)
    }
    
    /// Transform graphics coordinates to screen coordinates
    /// 
    /// # Arguments
    /// * `graphics_x` - X coordinate in graphics space
    /// * `graphics_y` - Y coordinate in graphics space
    /// 
    /// # Returns
    /// Screen coordinates as (x, y) tuple in pixels
    pub fn graphics_to_screen(&self, graphics_x: f32, graphics_y: f32) -> (f32, f32) {
        // Convert graphics coordinates to normalized coordinates
        let norm_x = (graphics_x / self.graphics_width) + 0.5;
        let norm_y = 0.5 - (graphics_y / self.graphics_height); // Flip Y axis
        
        // Convert normalized coordinates to screen pixels
        let screen_x = norm_x * self.canvas_width;
        let screen_y = norm_y * self.canvas_height;
        
        (screen_x, screen_y)
    }
    
    /// Check if screen coordinates are within canvas bounds
    pub fn is_within_canvas(&self, screen_x: f32, screen_y: f32) -> bool {
        screen_x >= 0.0 && screen_x < self.canvas_width &&
        screen_y >= 0.0 && screen_y < self.canvas_height
    }
    
    /// Get canvas dimensions
    pub fn canvas_dimensions(&self) -> (f32, f32) {
        (self.canvas_width, self.canvas_height)
    }
    
    /// Get graphics dimensions
    pub fn graphics_dimensions(&self) -> (f32, f32) {
        (self.graphics_width, self.graphics_height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_mock_transformer() -> CoordinateTransformer {
        // Mock transformer with 800x600 canvas and 2.0x1.5 graphics space
        CoordinateTransformer {
            canvas_width: 800.0,
            canvas_height: 600.0,
            graphics_width: 2.0,
            graphics_height: 1.5,
        }
    }
    
    #[test]
    fn test_screen_to_graphics_center() {
        let transformer = create_mock_transformer();
        
        // Center of canvas should map to (0, 0) in graphics space
        let (gx, gy) = transformer.screen_to_graphics(400.0, 300.0);
        assert!((gx - 0.0).abs() < 0.001);
        assert!((gy - 0.0).abs() < 0.001);
    }
    
    #[test]
    fn test_screen_to_graphics_corners() {
        let transformer = create_mock_transformer();
        
        // Top-left corner
        let (gx, gy) = transformer.screen_to_graphics(0.0, 0.0);
        assert!((gx - (-1.0)).abs() < 0.001);
        assert!((gy - 0.75).abs() < 0.001);
        
        // Bottom-right corner
        let (gx, gy) = transformer.screen_to_graphics(800.0, 600.0);
        assert!((gx - 1.0).abs() < 0.001);
        assert!((gy - (-0.75)).abs() < 0.001);
    }
    
    #[test]
    fn test_graphics_to_screen_center() {
        let transformer = create_mock_transformer();
        
        // Graphics center (0, 0) should map to canvas center
        let (sx, sy) = transformer.graphics_to_screen(0.0, 0.0);
        assert!((sx - 400.0).abs() < 0.001);
        assert!((sy - 300.0).abs() < 0.001);
    }
    
    #[test]
    fn test_graphics_to_screen_extremes() {
        let transformer = create_mock_transformer();
        
        // Graphics extremes should map to canvas edges
        let (sx, sy) = transformer.graphics_to_screen(-1.0, 0.75);
        assert!((sx - 0.0).abs() < 0.001);
        assert!((sy - 0.0).abs() < 0.001);
        
        let (sx, sy) = transformer.graphics_to_screen(1.0, -0.75);
        assert!((sx - 800.0).abs() < 0.001);
        assert!((sy - 600.0).abs() < 0.001);
    }
    
    #[test]
    fn test_round_trip_conversion() {
        let transformer = create_mock_transformer();
        
        // Test that screen -> graphics -> screen preserves coordinates
        let original_screen = (200.0, 150.0);
        let (gx, gy) = transformer.screen_to_graphics(original_screen.0, original_screen.1);
        let (sx, sy) = transformer.graphics_to_screen(gx, gy);
        
        assert!((sx - original_screen.0).abs() < 0.001);
        assert!((sy - original_screen.1).abs() < 0.001);
    }
    
    #[test]
    fn test_is_within_canvas() {
        let transformer = create_mock_transformer();
        
        // Test points within canvas
        assert!(transformer.is_within_canvas(400.0, 300.0));
        assert!(transformer.is_within_canvas(0.0, 0.0));
        assert!(transformer.is_within_canvas(799.0, 599.0));
        
        // Test points outside canvas
        assert!(!transformer.is_within_canvas(-1.0, 300.0));
        assert!(!transformer.is_within_canvas(400.0, -1.0));
        assert!(!transformer.is_within_canvas(800.0, 300.0));
        assert!(!transformer.is_within_canvas(400.0, 600.0));
    }
    
    #[test]
    fn test_update_canvas_size() {
        let mut transformer = create_mock_transformer();
        
        // Update to new canvas size
        transformer.canvas_width = 1024.0;
        transformer.canvas_height = 768.0;
        
        // Verify dimensions changed
        assert_eq!(transformer.canvas_dimensions(), (1024.0, 768.0));
        
        // Test coordinate transformation with new size
        let (gx, gy) = transformer.screen_to_graphics(512.0, 384.0); // New center
        assert!((gx - 0.0).abs() < 0.001);
        assert!((gy - 0.0).abs() < 0.001);
    }
    
    #[test]
    fn test_update_graphics_size() {
        let mut transformer = create_mock_transformer();
        
        // Update to new graphics size
        transformer.update_graphics_size(4.0, 3.0);
        
        // Verify dimensions changed
        assert_eq!(transformer.graphics_dimensions(), (4.0, 3.0));
        
        // Test coordinate transformation with new size
        let (gx, gy) = transformer.screen_to_graphics(0.0, 0.0); // Top-left
        assert!((gx - (-2.0)).abs() < 0.001);
        assert!((gy - 1.5).abs() < 0.001);
    }
}