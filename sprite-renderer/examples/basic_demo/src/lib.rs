use sprite_renderer::*;
use wasm_bindgen::prelude::*;
use web_sys::{console, HtmlCanvasElement};

// Entry point for the WebAssembly module
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console::log_1(&"Basic Sprite Renderer Demo starting...".into());
    
    // TODO: Initialize demo when renderer is implemented
    run_demo().unwrap_or_else(|e| {
        console::log_1(&format!("Demo error: {:?}", e).into());
    });
}

// Main demo function
fn run_demo() -> Result<()> {
    // Get canvas element
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    // TODO: Initialize renderer when implemented
    // let mut renderer = SpriteRenderer::new(&canvas)?;
    
    // TODO: Create sprites when implemented
    // let sprite = Sprite::builder()
    //     .position(400.0, 300.0)
    //     .size(64.0, 64.0)
    //     .color(Color::RED)
    //     .build();
    
    // TODO: Set up camera when implemented
    // let camera = Camera::default_2d(800, 600);
    
    // TODO: Render loop when implemented
    // renderer.render(&[sprite], &camera)?;
    
    console::log_1(&"Demo initialized successfully".into());
    Ok(())
}

// Export functions for JavaScript interaction
#[wasm_bindgen]
pub fn resize_canvas(width: u32, height: u32) {
    console::log_1(&format!("Canvas resized to {}x{}", width, height).into());
    // TODO: Handle canvas resize
}

#[wasm_bindgen]
pub fn handle_click(x: f32, y: f32) {
    console::log_1(&format!("Mouse clicked at ({}, {})", x, y).into());
    // TODO: Handle mouse interaction
}