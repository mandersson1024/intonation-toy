# Testing Guide

This project uses `wasm-pack test --headless --chrome` for all testing. We do NOT use any other form of testing.

## Running Tests

### All Packages
```bash
# Run tests for all packages
./scripts/test-all.sh
```

### Individual Packages
```bash
# Core application
wasm-pack test --headless --chrome pitch-toy

# Observable data library
wasm-pack test --headless --chrome observable-data

# Event dispatcher
wasm-pack test --headless --chrome event-dispatcher

# Egui dev console
wasm-pack test --headless --chrome egui-dev-console
```

## Why wasm-pack test --headless --chrome?

- ✅ Tests in actual browser environment with real Web APIs
- ✅ Supports WebAssembly + JavaScript interop testing
- ✅ Official recommended approach for browser-targeted WebAssembly
- ✅ Automated browser and WebDriver management
- ✅ CI/CD compatible with headless testing
- ✅ No manual test runner configuration needed

## Test Configuration

All tests use `wasm-bindgen-test` with Chrome browser:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_basic_functionality() {
        // Simple unit test
        assert_eq!(2 + 2, 4);
    }

    #[wasm_bindgen_test]
    fn test_data_structure() {
        // Test your data structures
        let data = MyStruct::new(42);
        assert_eq!(data.get_value(), 42);
        
        data.set_value(100);
        assert_eq!(data.get_value(), 100);
    }

    #[wasm_bindgen_test]
    fn test_error_handling() {
        // Test error cases
        let result = risky_function();
        match result {
            Ok(value) => assert!(value > 0),
            Err(e) => assert!(e.contains("expected error message")),
        }
    }

    #[wasm_bindgen_test]
    async fn test_async_function() {
        // Test async functions (common for browser APIs)
        let result = async_operation().await;
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_with_browser_apis() {
        // Test code that uses browser APIs
        use web_sys::console;
        
        // This will work because we're running in a real browser
        console::log_1(&"Test message".into());
        
        // Test Web Audio API, DOM manipulation, etc.
        let window = web_sys::window().unwrap();
        assert!(window.location().href().is_ok());
    }
}
```

### Key Differences from Standard Rust Tests

- Use `#[wasm_bindgen_test]` instead of `#[test]`
- Include `wasm_bindgen_test_configure!(run_in_browser);`
- Import `wasm_bindgen_test::*`
- Can test browser APIs and JavaScript interop
- Support for async tests with real browser environment

## Prerequisites

- Install wasm-pack:
  ```bash
  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
  ```

- Ensure Chrome is installed and available in PATH

## CI/CD

In CI environments, run:
```bash
./scripts/test-all.sh
```

## Important Notes

- We do NOT use `cargo test`
- We do NOT use `cargo test --target wasm32-unknown-unknown` 
- We do NOT use any other testing approach
- All tests must use `#[wasm_bindgen_test]` attributes
- All tests run in Chrome browser environment only