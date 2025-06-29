# Coding Standards

## Core Development Principles

### YAGNI (You Aren't Gonna Need It)

**YAGNI is a fundamental principle for this project.** Do not implement features, modules, or infrastructure until they are actually needed for the current task.

#### Key Rules:
- **Never create placeholder files** for future implementation
- **Never implement unused modules** that will create compiler warnings
- **Never build infrastructure** for anticipated future needs
- **Use TODO comments** instead of empty implementations
- **Write stubs only** when code is referenced but incomplete

#### Examples:

**L YAGNI Violation:**
```rust
// Creating unused module files
// src/future_feature.rs - empty file for "future use"
pub struct FutureFeature; // unused, causes warnings
```

** YAGNI Compliant:**
```rust
// TODO: Implement FutureFeature when Task 15 is ready
// This avoids compiler warnings and complexity
```

#### Why YAGNI Matters:
- Prevents compiler warnings that complicate code review
- Keeps codebase focused on current requirements
- Reduces complexity and maintenance burden
- Roadmaps change - unused code becomes technical debt

#### Implementation Guidelines:
- Create files and modules **only when they are actively used**
- Use TODO comments to document planned future work
- Implement stubs only for **referenced but incomplete** functionality
- Remove unused code immediately during refactoring

### Code Quality Standards

#### Rust-Specific Guidelines
- Follow `rustfmt` formatting standards
- Use `clippy` lints for code quality
- Prefer explicit error handling over `.unwrap()`
- Use type annotations for complex generic functions

#### Performance Standards
- Zero-allocation paths for audio processing
- Pre-allocate buffers for real-time operations
- Use `Arc<T>` for large data sharing
- Profile memory usage in development builds

#### Documentation Standards
- Document public APIs with rustdoc comments
- Include code examples for complex functions
- Document performance expectations for critical paths
- Maintain architecture decision records

#### Testing Standards

##### Dual Testing Strategy
- **Native Tests (cargo test)**: Fast feedback for Rust logic
  - Test basic application structure and module imports
  - Validate build configuration detection
  - No browser dependencies for core logic testing
- **WASM Tests (wasm-pack test)**: Real browser environment validation
  - Test WebAssembly compilation and browser APIs
  - Validate web-specific functionality (canvas, Web Audio API)
  - Cross-browser compatibility testing via headless browsers

##### Test Organization
- Unit tests for all audio processing algorithms
- Integration tests for component interactions  
- Performance tests for real-time requirements
- Module structure validation for YAGNI compliance
- Browser API integration testing

##### Testing Commands
- Development workflow: `cargo test` (instant feedback)
- Production validation: `wasm-pack test --headless --firefox` (browser environment)
- Full validation: Run both approaches for comprehensive coverage

## Module Organization

### File Structure
- Keep modules focused on single responsibilities
- Use clear, descriptive naming conventions
- Group related functionality in module directories
- Separate development tools from production code

### Dependency Management
- Minimize external dependencies
- Verify WebAssembly compatibility
- Pin dependency versions for reproducible builds
- Regular security audits of third-party crates

## Error Handling

### Error Strategy
- Use `Result<T, E>` for recoverable errors
- Use `panic!` only for unrecoverable programming errors
- Provide meaningful error messages
- Log errors with context for debugging

### Audio Processing Errors
- Handle microphone permission failures gracefully
- Recover from audio stream interruptions
- Provide user feedback for audio-related issues
- Maintain application stability during audio errors

## Security Guidelines

### WebAssembly Security
- No file system access beyond browser APIs
- Validate all external input data
- Use Rust's memory safety guarantees
- Follow secure coding practices

### Audio Privacy
- Process audio data locally only
- No persistent storage of audio data
- Clear audio buffers on session end
- Respect user privacy preferences