use std::env;

fn main() {
    let target = env::var("TARGET").unwrap_or_default();
    let is_wasm = target.starts_with("wasm32");
    
    // Emit cfg flags for conditional compilation
    if is_wasm {
        println!("cargo:rustc-cfg=web_sys_unstable_apis");
        println!("cargo:rustc-cfg=target_web");
    } else {
        println!("cargo:rustc-cfg=target_native");
    }
    
    // Feature validation
    let has_web = env::var("CARGO_FEATURE_WEB").is_ok();
    let has_test_native = env::var("CARGO_FEATURE_TEST_NATIVE").is_ok();
    
    // Only warn about web features on native if test-native is NOT enabled
    if has_web && !is_wasm && !has_test_native {
        println!("cargo:warning=Web features should only be used with wasm32 targets. Consider using --features test-native for native builds.");
    }
    
    // Ensure test-native is only used with native targets
    if has_test_native && is_wasm {
        println!("cargo:warning=test-native feature should only be used with native targets.");
    }
    
    // Set up appropriate cfg flags for platform abstraction
    if has_test_native {
        println!("cargo:rustc-cfg=platform_stubs");
    } else if has_web || is_wasm {
        // Use web platform for wasm targets or when web feature is enabled
        println!("cargo:rustc-cfg=platform_web");
    } else {
        // Default to stubs for native targets without explicit features
        println!("cargo:rustc-cfg=platform_stubs");
    }
    
    // Rerun build script if features change
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_WEB");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_TEST_NATIVE");
}