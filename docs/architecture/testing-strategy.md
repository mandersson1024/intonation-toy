# Testing Strategy
## Real-time Pitch Visualizer

**Version**: 1.0  
**Source**: Technical Architecture Document  
**Purpose**: Define testing approaches, coverage requirements, and testing tools

---

## Testing Pyramid

### Unit Tests (80% coverage)
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_pitch_detection_accuracy() {
        // Test with known frequencies
    }
    
    #[test]
    fn test_interval_calculation() {
        // Test interval math
    }
    
    #[test]
    fn test_message_passing() {
        // Test communication reliability
    }
}
```

**Requirements:**
- 80% code coverage minimum
- Located next to source files (`nextToFile: true`)
- Fast execution (<1s per test)
- No external dependencies

### Integration Tests
- **WASM Audio Pipeline**: End-to-end audio processing tests
- **Web Audio Integration**: AudioWorklet and Web Audio API tests
- **Cross-browser Compatibility**: Performance across browsers
- **Performance**: Latency and throughput benchmarks

**Test Location:** `/tests/integration/`
**Tools:** Jest with in-memory browser environment
**Coverage:** Critical integration points

### End-to-End Tests
**Test Location:** `/e2e/`
**Tools:** Cypress or Playwright
**Coverage:** Full user workflows

**Key Scenarios:**
- Microphone permission flow
- Audio processing pipeline
- Real-time visualization
- Cross-browser functionality

## Manual Testing Requirements

### Real-world Usage Testing
- Testing with actual musical instruments via browser
- Child usability testing with target age group (7-year-olds) on tablets/computers
- Cross-browser testing: Chrome, Firefox, Safari, Edge compatibility
- Device testing: Different devices and audio interfaces

### Performance Testing
- Audio latency measurement (target: <50ms)
- Visual frame rate validation (target: 60 FPS)
- Memory usage monitoring
- Cross-browser performance comparison

## Testing Tools

### Rust/WASM Testing
- **cargo test**: Unit tests for Rust code
- **wasm-pack test**: WASM-specific testing
- **Performance profiling**: Browser DevTools and wasm-pack profiling

### JavaScript Testing
- **Jest**: Unit and integration tests
- **Cypress/Playwright**: E2E testing
- **Browser DevTools**: Performance analysis

### Manual Testing Tools
- **BrowserStack**: Cross-browser testing platform
- **Audio analysis tools**: For latency and accuracy measurement
- **Performance monitoring**: Browser DevTools performance tab

## Test Data Requirements

### Audio Test Files
- Known frequency sine waves for accuracy testing
- Musical instrument recordings for real-world validation
- Edge case audio (very quiet, very loud, distorted)

### Reference Data
- Musical interval tables for validation
- Tuning system specifications (12-TET, Just Intonation)
- Browser compatibility matrices

## Performance Targets

| Metric | Target | Test Method |
|--------|--------|-------------|
| **Audio Latency** | <50ms | Input-to-output timing measurement |
| **GUI Frame Rate** | 60 FPS | Frame timing statistics |
| **Pitch Accuracy** | ±5 cents | Reference tone testing |
| **CPU Usage** | <50% single core | System monitoring during tests |
| **Memory Usage** | <100MB | Process memory tracking |

## Continuous Integration

### Pre-commit Testing
- Rust unit tests
- JavaScript unit tests
- Basic integration tests
- Code formatting and linting

### Full Test Suite (CI Pipeline)
- All unit and integration tests
- Cross-browser E2E tests (headless)
- Performance regression tests
- Build verification tests

### Release Testing
- Full manual testing checklist
- Performance benchmarking
- Cross-browser validation
- Real-device testing

## Test Environment Setup

### Development Environment
```bash
# Install test dependencies
cargo install wasm-pack
npm install -g jest cypress

# Run Rust tests
cargo test

# Run JavaScript tests  
npm test

# Run E2E tests
npx cypress run
```

### CI Environment
- Docker containers for consistent testing
- Browser testing with headless browsers
- Performance testing with controlled resources
- Artifact generation for test reports 