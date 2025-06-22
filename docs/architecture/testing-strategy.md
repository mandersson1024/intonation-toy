# Testing Strategy
## Real-time Pitch Visualizer

**Purpose**: Testing approach and coverage requirements

---

## Testing Levels

### Unit Tests (80% coverage)
```rust
#[test]
fn test_pitch_detection_accuracy() {
    // Test with known frequencies
}
```

**Requirements:**
- 80% minimum coverage
- Fast execution (<1s per test)  
- No external dependencies

### Integration Tests
- **Audio Pipeline**: End-to-end WASM audio processing
- **Web Audio**: AudioWorklet integration
- **Cross-browser**: Performance validation
- **Performance**: Latency benchmarks

**Location:** `tests/` and browser test suite at `/web/index.html`

### Browser Test Suite
**Enhanced Testing Interface** at `/web/index.html`:
- Real-time performance monitoring
- Automated test execution
- Stress testing (1000+ iterations)
- Cross-browser compatibility validation

### End-to-End Tests
**Approach:** Manual testing with browser test suite  
**Coverage:** Full user workflows

**Key Scenarios:**
- Microphone permission flow
- Audio processing pipeline  
- Real-time visualization

## Testing Tools

### Rust/WASM
- `cargo test` - Unit tests
- `trunk build` - Build validation
- Browser DevTools - Performance profiling

### Browser Testing
- Built-in test suite at `/web/index.html`
- Manual cross-browser validation

### Manual Testing
- Real instruments via browser
- Cross-browser compatibility
- Device testing (tablets, phones)

## Performance Targets

| Metric | Target | Test Method |
|--------|--------|-------------|
| **Audio Latency** | <50ms | Input-to-output timing |
| **GUI Frame Rate** | 60 FPS | Frame timing stats |
| **Pitch Accuracy** | ±5 cents | Reference tone testing |
| **Memory Usage** | <100MB | Process monitoring |

## Test Environment

### Development
```bash
# Run tests
cargo test              # Rust unit tests
./serve.sh             # Start test server
# Open http://localhost:PORT/web/ for browser tests
```

### CI Pipeline
- Rust unit tests (`cargo test`)
- Build verification (`cargo build`)
- Manual integration testing
- Performance validation 