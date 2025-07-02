# WASM Testing Setup

This directory contains WebAssembly integration tests using `wasm-bindgen-test`.

## Current Implementation

- **wasm.rs**: Basic WASM functionality tests
  - `test_wasm_build_configuration`: Verifies build configuration detection in WASM
  - `test_wasm_basic_functionality`: Basic WASM functionality validation

## Running Tests

### Native Tests (Phase 1)
```bash
cargo test
```
- Fast feedback for Rust logic
- 42 tests covering audio, console, and platform modules

### WASM Tests (Phase 2)
```bash
# Build WASM tests (verifies compilation)
cargo build --target wasm32-unknown-unknown

# Run WASM tests in browser (requires browser setup)
wasm-pack test --headless --chrome
wasm-pack test --headless --firefox
wasm-pack test --headless --safari
```

## Test Architecture

Following the phased testing strategy from docs/architecture/source-tree.md:

1. **Phase 1 (Current)**: Native tests via `cargo test`
2. **Phase 2 (Current)**: WASM compilation tests via `wasm-pack test`
3. **Phase 3 (Future)**: E2E browser tests via Cypress/Playwright

## Future WASM Tests (TODO)

When WASM-specific functionality is implemented, add tests for:
- Audio processing algorithms in WASM context
- Mathematical utilities and DSP functions
- Data serialization across WASM boundaries
- Inter-module communication
- Performance benchmarks and latency measurements
- Memory management in WASM linear memory

## Notes

- Entry point conflicts resolved with `#[cfg(not(test))]` on main function
- Tests follow YAGNI principle - only test what exists
- Browser environment required for full WASM test execution