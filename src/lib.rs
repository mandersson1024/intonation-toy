use wasm_bindgen::prelude::*;

// Import the `console.log` function from the browser's Web API
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro to provide `println!`-style syntax for `console.log` logging.
macro_rules! console_log {
    ( $( $t:tt )* ) => {
        log(&format!( $( $t )* ))
    }
}

// Import our audio module
pub mod audio;

// Re-export the audio engine for direct access from JavaScript
pub use audio::engine::AudioEngine;

// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("WASM Audio Processing Module Initialized");
}
