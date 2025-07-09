//! Manual test for Task 8: Camera System
//!
//! This example demonstrates and tests all the camera functionality
//! implemented in Task 8 of the sprite renderer project.

use sprite_renderer::{
    Camera, Vec2, Mat4,
    Result
};

fn main() -> Result<()> {
    println!("=== Task 8 Manual Test: Camera System ===\n");

    // Test 1: Basic camera creation
    println!("1. Testing basic camera creation...");
    let camera = Camera::new(800, 600);
    println!("   Created camera:");
    println!("   - Viewport: {}x{}", camera.viewport_width, camera.viewport_height);
    println!("   - Position: {:?}", camera.position);
    println!("   - Zoom: {}", camera.zoom);
    println!("   - Near/Far: {}/{}", camera.near, camera.far);
    assert_eq!(camera.viewport_width, 800);
    assert_eq!(camera.viewport_height, 600);
    assert_eq!(camera.position, Vec2::ZERO);
    assert_eq!(camera.zoom, 1.0);
    println!("   ✓ Basic camera creation works correctly\n");

    // Test 2: Default 2D camera
    println!("2. Testing Camera::default_2d()...");
    let camera_2d = Camera::default_2d(1024, 768);
    println!("   Default 2D camera:");
    println!("   - Viewport: {}x{}", camera_2d.viewport_width, camera_2d.viewport_height);
    println!("   - Zoom: {}", camera_2d.zoom);
    assert_eq!(camera_2d.viewport_width, 1024);
    assert_eq!(camera_2d.viewport_height, 768);
    assert_eq!(camera_2d.zoom, 1.0);
    println!("   ✓ Default 2D camera creation works correctly\n");

    // Test 3: Viewport management
    println!("3. Testing viewport management...");
    let mut camera = Camera::new(800, 600);
    println!("   Initial viewport: {}x{}", camera.viewport_width, camera.viewport_height);
    
    camera.set_viewport(1920, 1080);
    println!("   After set_viewport(1920, 1080): {}x{}", camera.viewport_width, camera.viewport_height);
    assert_eq!(camera.viewport_width, 1920);
    assert_eq!(camera.viewport_height, 1080);
    println!("   ✓ Viewport management works correctly\n");

    // Test 4: Camera position and movement
    println!("4. Testing camera position and movement...");
    let mut camera = Camera::new(800, 600);
    println!("   Initial position: {:?}", camera.position);
    
    camera.set_position(Vec2::new(100.0, 200.0));
    println!("   After set_position(100, 200): {:?}", camera.position);
    assert_eq!(camera.position, Vec2::new(100.0, 200.0));
    
    camera.translate(Vec2::new(50.0, -100.0));
    println!("   After translate(50, -100): {:?}", camera.position);
    assert_eq!(camera.position, Vec2::new(150.0, 100.0));
    println!("   ✓ Camera position and movement work correctly\n");

    // Test 5: Camera zoom
    println!("5. Testing camera zoom...");
    let mut camera = Camera::new(800, 600);
    println!("   Initial zoom: {}", camera.zoom);
    
    camera.set_zoom(2.0);
    println!("   After set_zoom(2.0): {}", camera.zoom);
    assert_eq!(camera.zoom, 2.0);
    
    camera.zoom_by(1.5);
    println!("   After zoom_by(1.5): {}", camera.zoom);
    assert_eq!(camera.zoom, 3.0);
    
    // Test zoom limits
    camera.set_zoom(-1.0);
    println!("   After set_zoom(-1.0): {} (should be > 0)", camera.zoom);
    assert!(camera.zoom > 0.0);
    
    camera.set_zoom(0.0);
    println!("   After set_zoom(0.0): {} (should be > 0)", camera.zoom);
    assert!(camera.zoom > 0.0);
    println!("   ✓ Camera zoom works correctly with limits\n");

    // Test 6: Coordinate transformations
    println!("6. Testing coordinate transformations...");
    let camera = Camera::new(800, 600);
    
    // Test screen center to world
    let screen_center = Vec2::new(400.0, 300.0);
    let world_center = camera.screen_to_world(screen_center);
    println!("   Screen center {} -> World: {:?}", format!("({}, {})", screen_center.x, screen_center.y), world_center);
    assert!((world_center.x - 0.0).abs() < 0.01);
    assert!((world_center.y - 0.0).abs() < 0.01);
    
    // Test world origin to screen
    let world_origin = Vec2::ZERO;
    let screen_origin = camera.world_to_screen(world_origin);
    println!("   World origin -> Screen: {:?}", screen_origin);
    assert!((screen_origin.x - 400.0).abs() < 0.01);
    assert!((screen_origin.y - 300.0).abs() < 0.01);
    
    // Test round-trip conversion
    let original_screen = Vec2::new(200.0, 150.0);
    let world_pos = camera.screen_to_world(original_screen);
    let back_to_screen = camera.world_to_screen(world_pos);
    println!("   Round-trip: {} -> {:?} -> {:?}", 
        format!("({}, {})", original_screen.x, original_screen.y), 
        world_pos, 
        back_to_screen);
    assert!((back_to_screen.x - original_screen.x).abs() < 0.1);
    assert!((back_to_screen.y - original_screen.y).abs() < 0.1);
    println!("   ✓ Coordinate transformations work correctly\n");

    // Test 7: Visible bounds calculation
    println!("7. Testing visible bounds calculation...");
    let mut camera = Camera::new(800, 600);
    
    let (min, max) = camera.visible_bounds();
    println!("   Zoom 1.0 bounds: min={:?}, max={:?}", min, max);
    println!("   Visible area: {}x{}", max.x - min.x, max.y - min.y);
    assert_eq!(max.x - min.x, 800.0);
    assert_eq!(max.y - min.y, 600.0);
    
    camera.set_zoom(2.0);
    let (min_zoom, max_zoom) = camera.visible_bounds();
    println!("   Zoom 2.0 bounds: min={:?}, max={:?}", min_zoom, max_zoom);
    println!("   Visible area: {}x{}", max_zoom.x - min_zoom.x, max_zoom.y - min_zoom.y);
    assert_eq!(max_zoom.x - min_zoom.x, 400.0);
    assert_eq!(max_zoom.y - min_zoom.y, 300.0);
    
    camera.set_position(Vec2::new(100.0, 50.0));
    let (min_moved, max_moved) = camera.visible_bounds();
    println!("   After moving camera to (100, 50): min={:?}, max={:?}", min_moved, max_moved);
    let center_x = (min_moved.x + max_moved.x) / 2.0;
    let center_y = (min_moved.y + max_moved.y) / 2.0;
    assert!((center_x - 100.0).abs() < 0.01);
    assert!((center_y - 50.0).abs() < 0.01);
    println!("   ✓ Visible bounds calculation works correctly\n");

    // Test 8: Projection matrix properties
    println!("8. Testing projection matrix properties...");
    let camera = Camera::new(800, 600);
    let proj_matrix = camera.projection_matrix;
    
    println!("   Projection matrix generated (orthographic)");
    println!("   Matrix[0,0] (X scale): {}", proj_matrix.get(0, 0));
    println!("   Matrix[1,1] (Y scale): {}", proj_matrix.get(1, 1));
    println!("   Matrix[2,2] (Z scale): {}", proj_matrix.get(2, 2));
    println!("   Matrix[3,3] (W): {}", proj_matrix.get(3, 3));
    
    // Orthographic projection properties
    assert_ne!(proj_matrix, Mat4::identity());
    assert!(proj_matrix.get(2, 2) < 0.0); // Z should be negative for depth
    assert_eq!(proj_matrix.get(3, 3), 1.0); // W should be 1.0 for orthographic
    println!("   ✓ Projection matrix properties are correct\n");

    // Test 9: View-projection matrix combination
    println!("9. Testing view-projection matrix combination...");
    let mut camera = Camera::new(800, 600);
    
    let vp_origin = camera.view_projection_matrix();
    println!("   View-projection matrix at origin generated");
    
    camera.set_position(Vec2::new(100.0, 200.0));
    let vp_moved = camera.view_projection_matrix();
    println!("   View-projection matrix after moving camera generated");
    
    assert_ne!(vp_moved, vp_origin);
    assert_ne!(vp_moved, Mat4::identity());
    println!("   ✓ View-projection matrix combination works correctly\n");

    // Test 10: Coordinate transformation with zoom and position
    println!("10. Testing coordinate transformation with zoom and position...");
    let mut camera = Camera::new(800, 600);
    camera.set_position(Vec2::new(50.0, 25.0));
    camera.set_zoom(2.0);
    
    let screen_center = Vec2::new(400.0, 300.0);
    let world_pos = camera.screen_to_world(screen_center);
    println!("   Camera at (50, 25), zoom 2.0:");
    println!("   Screen center {} -> World: {:?}", format!("({}, {})", screen_center.x, screen_center.y), world_pos);
    
    // Should map to camera position
    assert!((world_pos.x - 50.0).abs() < 0.01);
    assert!((world_pos.y - 25.0).abs() < 0.01);
    
    let back_to_screen = camera.world_to_screen(world_pos);
    println!("   World {:?} -> Screen: {:?}", world_pos, back_to_screen);
    assert!((back_to_screen.x - 400.0).abs() < 0.01);
    assert!((back_to_screen.y - 300.0).abs() < 0.01);
    println!("   ✓ Complex coordinate transformations work correctly\n");

    println!("=== ALL TESTS PASSED! ===");
    println!("Task 8 Camera System implementation is working correctly.");
    println!("- ✓ Camera creation with viewport dimensions");
    println!("- ✓ Default 2D orthographic projection");
    println!("- ✓ Viewport management and dynamic resizing");
    println!("- ✓ Camera position and movement controls");
    println!("- ✓ Zoom functionality with safety limits");
    println!("- ✓ Screen-to-world coordinate transformation");
    println!("- ✓ World-to-screen coordinate transformation");
    println!("- ✓ Visible bounds calculation");
    println!("- ✓ Orthographic projection matrix generation");
    println!("- ✓ View-projection matrix combination");
    println!("- ✓ Complex transformations with position and zoom");

    Ok(())
}