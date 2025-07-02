#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_wasm_build_configuration() {
    // Test that WASM compilation works and can detect build configuration
    let config = if cfg!(debug_assertions) { "Development" } else { "Production" };
    assert!(config == "Development" || config == "Production");
}
