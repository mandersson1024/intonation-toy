# Coding Standards

## Overview

This document defines the coding standards for the Pitch-Toy real-time audio pitch detection application. These standards ensure consistency, maintainability, and quality across the Rust/WebAssembly codebase.

## Rust Language Standards

### General Principles

- **Safety First**: Leverage Rust's memory safety guarantees; avoid `unsafe` code unless absolutely necessary
- **Explicit Over Implicit**: Prefer explicit type annotations and error handling
- **Zero-Cost Abstractions**: Use abstractions that don't compromise performance
- **Idiomatic Rust**: Follow standard Rust conventions and community best practices

### Code Organization

#### Module Structure
```rust
// Module declarations in order of dependency
mod types;      // Type definitions first
mod services;   // Business logic
mod components; // UI components
mod hooks;      // Reusable logic hooks
```

#### Re-exports
- Use `pub use` for clean API surfaces
- Group re-exports logically by functionality
- Export commonly used types at the module level

```rust
// Good: Clean re-exports
pub use engine::AudioEngine;
pub use pitch_detector::PitchDetector;

// Re-export components for easy access
pub use debug_interface::DebugInterface;
pub use audio_control_panel::AudioControlPanel;
```

### Error Handling

#### Error Management Strategy
- Use centralized error management via `ErrorManager` service
- Implement `Result<T, E>` pattern consistently
- Provide user-friendly error messages with recovery suggestions
- Log errors appropriately for debugging

```rust
// Preferred error handling pattern
match audio_engine.start_processing() {
    Ok(result) => handle_success(result),
    Err(error) => {
        error_manager.handle_error(error);
        // Provide fallback behavior
    }
}
```

#### Error Types
- Create domain-specific error types
- Implement `Display` and `Debug` traits
- Include context information for debugging

### Performance Standards

#### WebAssembly Optimization
- Minimize WASM memory allocations in hot paths
- Use `wasm-bindgen` efficiently for JS interop
- Profile and optimize audio processing pipelines

#### Audio Processing Requirements
- Target <50ms audio latency
- Process buffers efficiently (1024-2048 samples)
- Maintain <70% AudioWorklet CPU utilization
- Implement memory-efficient algorithms

```rust
// Example: Efficient audio buffer processing
impl RealtimeProcessor {
    pub fn process_buffer(&mut self, buffer: &[f32]) -> ProcessingResult {
        // Minimize allocations in audio thread
        self.working_buffer.clear();
        self.working_buffer.extend_from_slice(buffer);
        
        // Process in-place when possible
        self.apply_windowing(&mut self.working_buffer);
        self.detect_pitch(&self.working_buffer)
    }
}
```

## Yew Framework Standards

### Component Architecture

#### Component Naming
- Use descriptive, noun-based names: `AudioControlPanel`, `DebugInterface`
- Suffix with `Component` only when disambiguation is needed
- Use PascalCase for component names

#### Component Structure
```rust
#[function_component(ComponentName)]
pub fn component_name(props: &Props) -> Html {
    // 1. State declarations
    let state = use_state(|| initial_value);
    
    // 2. Effect hooks
    use_effect_with_deps(move |_| {
        // Effect logic
        || () // Cleanup
    }, ());
    
    // 3. Event handlers
    let on_click = {
        let state = state.clone();
        Callback::from(move |_| {
            state.set(new_value);
        })
    };
    
    // 4. Render
    html! {
        <div class="component-name">
            // Component JSX
        </div>
    }
}
```

#### Props Design
- Use explicit prop types with `#[derive(Debug, Clone, PartialEq, Properties)]`
- Provide sensible defaults where appropriate
- Document prop behavior with doc comments

```rust
#[derive(Debug, Clone, PartialEq, Properties)]
pub struct AudioControlProps {
    /// Audio engine service instance
    pub audio_engine: Option<Rc<RefCell<AudioEngineService>>>,
    
    /// Update interval in milliseconds (default: 250ms)
    #[prop_or(250)]
    pub update_interval_ms: u32,
    
    /// Optional error handler callback
    #[prop_or_default]
    pub on_error: Option<Callback<String>>,
}
```

### State Management

#### Local Component State
- Use `use_state` for simple component state
- Use `use_reducer` for complex state logic
- Prefer immutable state updates

#### Shared State
- Use `Rc<RefCell<T>>` pattern for shared mutable state
- Implement centralized services for cross-component state
- Consider context providers for deeply nested prop drilling

## Documentation Standards
- Never guess today's date

### Code Documentation
- Never refer to Epic numbers, Story numbers or Acceptance Criteria numbers

#### Doc Comments
- Document all public APIs with `///`
- Include examples for complex functions
- Document error conditions and panics
- Use standard sections: Examples, Panics, Errors, Safety
- Never refer to Epic numbers, Story numbers or Acceptance Criteria numbers

```rust
/// Detects pitch from audio buffer using YIN algorithm
/// 
/// # Arguments
/// * `buffer` - Audio samples normalized to [-1.0, 1.0]
/// * `sample_rate` - Sample rate in Hz
/// 
/// # Returns
/// * `Ok(frequency)` - Detected pitch frequency in Hz
/// * `Err(error)` - Processing error with context
/// 
/// # Examples
/// ```rust
/// let detector = PitchDetector::new();
/// let frequency = detector.detect_pitch(&samples, 44100)?;
/// ```
pub fn detect_pitch(&self, buffer: &[f32], sample_rate: f32) -> Result<f32, PitchError> {
    // Implementation
}
```

#### Inline Comments
- Use sparingly for complex business logic
- Explain "why" not "what"
- Update comments when code changes
- Never refer to Epic numbers, Story numbers or Acceptance Criteria numbers

### Architecture Documentation
- Maintain architecture decision records (ADRs)
- Document performance characteristics
- Include browser compatibility notes

## Testing Standards

### Unit Testing

#### Test Organization
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pitch_detection_accuracy() {
        // Arrange
        let detector = PitchDetector::new();
        let test_signal = generate_sine_wave(440.0, 1024);
        
        // Act
        let result = detector.detect_pitch(&test_signal, 44100);
        
        // Assert
        assert!(result.is_ok());
        let frequency = result.unwrap();
        assert!((frequency - 440.0).abs() < 1.0); // 1Hz tolerance
    }
}
```

#### Test Naming
- Use descriptive test names: `test_pitch_detection_with_noise`
- Follow arrange-act-assert pattern
- Test both success and error cases

### Feature Flag Testing
- Test different feature combinations
- Use conditional compilation for test-specific code
- Mock external dependencies (audio devices, etc.)

## Build and Deployment Standards

### Feature Flags

#### Development Features
```toml
[features]
debug-features = ["debug-logging", "performance-profiling"]
debug-logging = []
performance-profiling = []
```

#### Usage Guidelines
- Use feature flags to conditionally compile expensive debug code
- Provide feature combinations for different deployment targets
- Document feature flag behavior and dependencies

### Build Profiles

#### Development Profile
- Full debugging information
- Fast compilation over optimization
- Debug assertions enabled

#### Release Profile  
- Maximum optimization (`opt-level = 3`)
- Link-time optimization (`lto = "fat"`)
- Minimal debug information

## Browser Compatibility

### Web APIs
- Feature detection before using advanced APIs
- Graceful degradation for unsupported browsers
- Clear error messages for compatibility issues

### Performance Considerations
- Target modern browsers (Chrome 69+, Firefox 76+, Safari 14.1+)
- Optimize for AudioWorklet and WebAssembly performance
- Test across different browser engines

## Security Standards

### Input Validation
- Validate all user inputs and audio data
- Sanitize data passed between WASM and JavaScript
- Implement bounds checking for audio buffers

### Memory Safety
- Leverage Rust's ownership system
- Avoid memory leaks in long-running audio processing
- Monitor memory usage in browser environments

## Code Review Checklist

- [ ] Code follows Rust idioms and conventions
- [ ] Error handling is comprehensive and user-friendly
- [ ] Performance requirements are met (audio latency < 50ms)
- [ ] Public APIs are documented with examples
- [ ] Tests cover both success and error cases
- [ ] Feature flags are used appropriately
- [ ] Browser compatibility is maintained
- [ ] Security considerations are addressed
- [ ] Memory usage is optimized for WASM
- [ ] Code is formatted with `rustfmt`
- [ ] No clippy warnings in release builds