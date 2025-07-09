//! Core rendering engine and context management
//!
//! This module provides the main rendering engine for the sprite renderer,
//! including WebGL context management and rendering pipeline.

pub mod context;
pub mod batch;
pub mod culling;

use crate::{Sprite, RendererError, Vec2, Mat4};
use context::RenderContext;
use batch::BatchRenderer;

/// Camera for 2D viewport management and coordinate transformations
#[derive(Debug, Clone)]
pub struct Camera {
    /// Viewport width in pixels
    pub viewport_width: u32,
    /// Viewport height in pixels
    pub viewport_height: u32,
    /// Projection matrix for screen-to-world coordinate transformation
    pub projection_matrix: Mat4,
    /// View matrix for camera position and orientation
    pub view_matrix: Mat4,
    /// Camera position in world space
    pub position: Vec2,
    /// Camera zoom factor (1.0 = normal, >1.0 = zoomed in, <1.0 = zoomed out)
    pub zoom: f32,
    /// Near clipping plane distance
    pub near: f32,
    /// Far clipping plane distance
    pub far: f32,
}

impl Camera {
    /// Create a new camera with specified viewport dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let mut camera = Self {
            viewport_width: width,
            viewport_height: height,
            projection_matrix: Mat4::identity(),
            view_matrix: Mat4::identity(),
            position: Vec2::ZERO,
            zoom: 1.0,
            near: -1.0,
            far: 1.0,
        };
        camera.update_projection_matrix();
        camera
    }

    /// Create a default 2D camera with orthographic projection
    pub fn default_2d(width: u32, height: u32) -> Self {
        Self::new(width, height)
    }

    /// Update the viewport dimensions and recalculate projection matrix
    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.viewport_width = width;
        self.viewport_height = height;
        self.update_projection_matrix();
    }

    /// Set camera position in world space
    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
        self.update_view_matrix();
    }

    /// Move camera by offset
    pub fn translate(&mut self, offset: Vec2) {
        self.position.x += offset.x;
        self.position.y += offset.y;
        self.update_view_matrix();
    }

    /// Set camera zoom level
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.max(0.01); // Prevent negative or zero zoom
        self.update_projection_matrix();
    }

    /// Zoom in/out by a factor
    pub fn zoom_by(&mut self, factor: f32) {
        self.set_zoom(self.zoom * factor);
    }

    /// Get the combined view-projection matrix
    pub fn view_projection_matrix(&self) -> Mat4 {
        // For 2D rendering, we typically apply projection first, then view
        // Note: This is a simplified matrix multiplication for our 2D case
        self.combine_matrices(&self.projection_matrix, &self.view_matrix)
    }

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        // Convert screen coordinates (0,0 at top-left) to normalized device coordinates (-1,1)
        let normalized_x = (2.0 * screen_pos.x) / (self.viewport_width as f32) - 1.0;
        let normalized_y = 1.0 - (2.0 * screen_pos.y) / (self.viewport_height as f32); // Flip Y

        // Apply inverse projection to get world coordinates
        let world_x = (normalized_x / self.zoom) + self.position.x;
        let world_y = (normalized_y / self.zoom) + self.position.y;

        Vec2::new(world_x, world_y)
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        // Apply camera transform
        let view_x = (world_pos.x - self.position.x) * self.zoom;
        let view_y = (world_pos.y - self.position.y) * self.zoom;

        // Convert to screen coordinates
        let screen_x = (view_x + 1.0) * (self.viewport_width as f32) * 0.5;
        let screen_y = (1.0 - view_y) * (self.viewport_height as f32) * 0.5; // Flip Y

        Vec2::new(screen_x, screen_y)
    }

    /// Get the visible world bounds (what the camera can see)
    pub fn visible_bounds(&self) -> (Vec2, Vec2) {
        let half_width = (self.viewport_width as f32) / (2.0 * self.zoom);
        let half_height = (self.viewport_height as f32) / (2.0 * self.zoom);

        let min = Vec2::new(self.position.x - half_width, self.position.y - half_height);
        let max = Vec2::new(self.position.x + half_width, self.position.y + half_height);

        (min, max)
    }

    /// Update the projection matrix based on current viewport and zoom
    fn update_projection_matrix(&mut self) {
        let half_width = (self.viewport_width as f32) / (2.0 * self.zoom);
        let half_height = (self.viewport_height as f32) / (2.0 * self.zoom);

        self.projection_matrix = Mat4::orthographic(
            -half_width,  // left
            half_width,   // right
            -half_height, // bottom
            half_height,  // top
            self.near,    // near
            self.far,     // far
        );
    }

    /// Update the view matrix based on camera position
    fn update_view_matrix(&mut self) {
        self.view_matrix = Mat4::translation(-self.position.x, -self.position.y, 0.0);
    }

    /// Simple matrix multiplication for combining view and projection matrices
    fn combine_matrices(&self, a: &Mat4, b: &Mat4) -> Mat4 {
        let mut result = Mat4::identity();
        
        for row in 0..4 {
            for col in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += a.get(row, k) * b.get(k, col);
                }
                result.set(row, col, sum);
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_creation() {
        let camera = Camera::new(800, 600);
        assert_eq!(camera.viewport_width, 800);
        assert_eq!(camera.viewport_height, 600);
        assert_eq!(camera.position, Vec2::ZERO);
        assert_eq!(camera.zoom, 1.0);
        assert_eq!(camera.near, -1.0);
        assert_eq!(camera.far, 1.0);
    }

    #[test]
    fn test_camera_default_2d() {
        let camera = Camera::default_2d(1024, 768);
        assert_eq!(camera.viewport_width, 1024);
        assert_eq!(camera.viewport_height, 768);
        assert_eq!(camera.zoom, 1.0);
    }

    #[test]
    fn test_camera_set_viewport() {
        let mut camera = Camera::new(800, 600);
        camera.set_viewport(1920, 1080);
        assert_eq!(camera.viewport_width, 1920);
        assert_eq!(camera.viewport_height, 1080);
    }

    #[test]
    fn test_camera_position() {
        let mut camera = Camera::new(800, 600);
        
        camera.set_position(Vec2::new(100.0, 200.0));
        assert_eq!(camera.position, Vec2::new(100.0, 200.0));

        camera.translate(Vec2::new(50.0, -50.0));
        assert_eq!(camera.position, Vec2::new(150.0, 150.0));
    }

    #[test]
    fn test_camera_zoom() {
        let mut camera = Camera::new(800, 600);
        
        camera.set_zoom(2.0);
        assert_eq!(camera.zoom, 2.0);

        camera.zoom_by(1.5);
        assert_eq!(camera.zoom, 3.0);

        // Test zoom limits
        camera.set_zoom(-1.0);
        assert!(camera.zoom > 0.0); // Should be clamped to minimum
    }

    #[test]
    fn test_screen_to_world_conversion() {
        let camera = Camera::new(800, 600);
        
        // Center of screen should map to camera position (0,0)
        let center_screen = Vec2::new(400.0, 300.0);
        let world_pos = camera.screen_to_world(center_screen);
        assert!((world_pos.x - 0.0).abs() < 0.01);
        assert!((world_pos.y - 0.0).abs() < 0.01);

        // Top-left corner
        let top_left = Vec2::new(0.0, 0.0);
        let world_tl = camera.screen_to_world(top_left);
        assert!(world_tl.x < 0.0); // Should be negative X
        assert!(world_tl.y > 0.0); // Should be positive Y (screen Y flipped)
    }

    #[test]
    fn test_world_to_screen_conversion() {
        let camera = Camera::new(800, 600);
        
        // Camera position (0,0) should map to screen center
        let world_origin = Vec2::ZERO;
        let screen_pos = camera.world_to_screen(world_origin);
        assert!((screen_pos.x - 400.0).abs() < 0.01);
        assert!((screen_pos.y - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_round_trip_coordinate_conversion() {
        let camera = Camera::new(800, 600);
        
        let original_screen = Vec2::new(200.0, 150.0);
        let world_pos = camera.screen_to_world(original_screen);
        let back_to_screen = camera.world_to_screen(world_pos);
        
        assert!((back_to_screen.x - original_screen.x).abs() < 0.1);
        assert!((back_to_screen.y - original_screen.y).abs() < 0.1);
    }

    #[test]
    fn test_visible_bounds() {
        let camera = Camera::new(800, 600);
        
        let (min, max) = camera.visible_bounds();
        
        // For zoom 1.0, visible area should be viewport size
        assert_eq!(max.x - min.x, 800.0);
        assert_eq!(max.y - min.y, 600.0);
        
        // Center should be at camera position
        let center_x = (min.x + max.x) / 2.0;
        let center_y = (min.y + max.y) / 2.0;
        assert!((center_x - camera.position.x).abs() < 0.01);
        assert!((center_y - camera.position.y).abs() < 0.01);
    }

    #[test]
    fn test_visible_bounds_with_zoom() {
        let mut camera = Camera::new(800, 600);
        camera.set_zoom(2.0);
        
        let (min, max) = camera.visible_bounds();
        
        // With 2x zoom, visible area should be half the viewport size
        assert_eq!(max.x - min.x, 400.0);
        assert_eq!(max.y - min.y, 300.0);
    }

    #[test]
    fn test_projection_matrix_orthographic() {
        let camera = Camera::new(800, 600);
        
        // Test that projection matrix is not identity (it should be updated)
        assert_ne!(camera.projection_matrix, Mat4::identity());
        
        // Test matrix properties for orthographic projection
        let matrix = camera.projection_matrix;
        
        // For orthographic projection, [2][2] should be negative (for depth)
        assert!(matrix.get(2, 2) < 0.0);
        
        // [3][3] should be 1.0 for orthographic
        assert_eq!(matrix.get(3, 3), 1.0);
    }

    #[test]
    fn test_view_projection_matrix() {
        let mut camera = Camera::new(800, 600);
        
        // With camera at origin, view-projection should equal projection (view is identity)
        let vp_matrix_origin = camera.view_projection_matrix();
        assert_ne!(vp_matrix_origin, Mat4::identity());
        
        // Move camera and test that view-projection matrix changes
        camera.set_position(Vec2::new(100.0, 50.0));
        let vp_matrix_moved = camera.view_projection_matrix();
        
        // Should be different after moving the camera
        assert_ne!(vp_matrix_moved, vp_matrix_origin);
        assert_ne!(vp_matrix_moved, Mat4::identity());
    }

    #[test]
    fn test_camera_with_position_and_zoom() {
        let mut camera = Camera::new(800, 600);
        camera.set_position(Vec2::new(100.0, 50.0));
        camera.set_zoom(2.0);
        
        let (min, max) = camera.visible_bounds();
        
        // Bounds should be centered on camera position
        let center_x = (min.x + max.x) / 2.0;
        let center_y = (min.y + max.y) / 2.0;
        assert!((center_x - 100.0).abs() < 0.01);
        assert!((center_y - 50.0).abs() < 0.01);
        
        // Size should be affected by zoom
        assert_eq!(max.x - min.x, 400.0); // 800 / 2
        assert_eq!(max.y - min.y, 300.0); // 600 / 2
    }
}

/// Main sprite renderer
pub struct SpriteRenderer {
    #[allow(dead_code)]
    context: RenderContext,
    #[allow(dead_code)]
    batch_renderer: BatchRenderer,
}

impl SpriteRenderer {
    /// Create a new sprite renderer
    pub fn new(canvas: &web_sys::HtmlCanvasElement) -> Result<Self, RendererError> {
        let context = RenderContext::new(canvas)?;
        let batch_renderer = BatchRenderer::new();
        
        Ok(Self {
            context,
            batch_renderer,
        })
    }
    
    /// Render a collection of sprites
    pub fn render(&mut self, _sprites: &[Sprite], _camera: &Camera) -> Result<(), RendererError> {
        // Placeholder implementation
        Ok(())
    }
}