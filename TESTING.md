# Testing Guide

This project uses `wasm-pack test --node` for all testing. We do NOT use any other form of testing.

## Running Tests

### All Packages
```bash
# Run tests for all packages
./scripts/test-all.sh
```

### Individual Packages
```bash
# Core application
wasm-pack test --node intonation-toy

# Observable data library
wasm-pack test --node observable-data

# Egui dev console
wasm-pack test --node dev-console
```

## Why wasm-pack test --node?

- ✅ Tests WebAssembly compilation and basic functionality
- ✅ Fast execution in Node.js environment
- ✅ No browser dependencies or WebDriver setup needed
- ✅ CI/CD compatible and reliable
- ✅ Official recommended approach for unit testing WebAssembly
- ✅ No manual test runner configuration needed

## Testing Policy

**We do NOT write tests for functionality that requires a real browser environment**, including:
- DOM manipulation
- Web Audio API
- Browser-specific APIs
- JavaScript interop that requires browser context

Unit tests focus on pure logic, data structures, and WebAssembly-compatible functionality only.

## Test Configuration

All tests use `wasm-bindgen-test` with Node.js environment:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    // No wasm_bindgen_test_configure! needed for Node.js

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

    // DO NOT test browser APIs - they won't work in Node.js
    // DO NOT test Web Audio API
    // DO NOT test DOM manipulation
    // DO NOT test window/document objects
}
```

### Key Differences from Standard Rust Tests

- Import `wasm_bindgen_test::*`
- No `wasm_bindgen_test_configure!` needed for Node.js
- Test only pure logic and data structures
- Cannot test browser APIs or JavaScript interop

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