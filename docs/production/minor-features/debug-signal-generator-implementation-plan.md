# Debug Signal Generator Implementation Plan

## Overview

The TestSignalGenerator is a critical debugging component that generates synthetic audio signals for testing the audio processing pipeline. This component enables developers to validate pitch detection, frequency analysis, and performance monitoring without requiring external audio input.

## Requirements

### Functional Requirements
1. **Waveform Generation**: Generate standard waveforms (sine, square, triangle, sawtooth)
2. **Frequency Control**: Precise frequency selection (20Hz - 20kHz range)
3. **Musical Note Generation**: Generate musical notes with accurate tuning
4. **Frequency Sweeps**: Automated frequency sweeps for comprehensive testing
5. **Chord Generation**: Multi-tone signals for complex audio testing
6. **Real-time Control**: Start/stop signal generation with immediate response
7. **Integration**: Seamless integration with existing audio engine

### Technical Requirements
1. **Performance**: Low-latency signal generation (<5ms)
2. **Accuracy**: Frequency accuracy within ±1Hz
3. **Quality**: Clean signals with minimal artifacts
4. **Stability**: Continuous operation without audio dropouts
5. **Memory**: Efficient buffer management for continuous playback

## Component Architecture

### Core Structure
```rust
pub struct TestSignalGenerator {
    // State management
    is_generating: bool,
    current_config: SignalConfig,
    audio_context: Option<web_sys::AudioContext>,
    oscillator_node: Option<web_sys::OscillatorNode>,
    gain_node: Option<web_sys::GainNode>,
    
    // Configuration
    signal_type: SignalType,
    frequency: f32,
    amplitude: f32,
    duration: Option<f32>,
}
```

### Signal Types
```rust
#[derive(Clone, PartialEq)]
pub enum SignalType {
    Sine,
    Square,
    Triangle,
    Sawtooth,
    WhiteNoise,
    Musical(MusicalNote),
    Sweep(SweepConfig),
    Chord(ChordConfig),
}
```

### Configuration Structures
```rust
#[derive(Clone)]
pub struct SweepConfig {
    start_freq: f32,
    end_freq: f32,
    duration_seconds: f32,
    sweep_type: SweepType, // Linear, Logarithmic
}

#[derive(Clone)]
pub struct ChordConfig {
    root_note: MusicalNote,
    chord_type: ChordType, // Major, Minor, Diminished, etc.
    octave: u8,
}

#[derive(Clone)]
pub struct MusicalNote {
    note: Note, // C, C#, D, etc.
    octave: u8,
}
```

## User Interface Design

### Control Panel Layout
1. **Signal Type Selector**
   - Dropdown with waveform options
   - Visual waveform preview icons

2. **Frequency Controls**
   - Numeric input field (Hz)
   - Slider for quick adjustment
   - Musical note selector (C4, A440, etc.)

3. **Amplitude Control**
   - Volume slider (0-100%)
   - Amplitude in dB display

4. **Generation Controls**
   - Start/Stop button (large, prominent)
   - Duration setting (continuous/timed)
   - Quick preset buttons

5. **Advanced Features**
   - Sweep configuration panel
   - Chord builder interface
   - Custom waveform upload

### Status Display
- Current signal parameters
- Generation status indicator
- Real-time frequency/amplitude readout
- Connection status to audio pipeline

## Implementation Phases

### Phase 1: Basic Waveform Generation
**Duration**: 2-3 days
**Deliverables**:
- Basic component structure
- Sine wave generation
- Frequency and amplitude controls
- Start/stop functionality
- Integration with debug interface

**Tasks**:
1. Create component boilerplate with Yew framework
2. Implement Web Audio API integration
3. Add basic sine wave oscillator
4. Create simple UI with frequency/amplitude controls
5. Add start/stop button functionality
6. Test basic signal generation

### Phase 2: Extended Waveforms
**Duration**: 1-2 days
**Deliverables**:
- Multiple waveform types
- Waveform selector UI
- Enhanced controls

**Tasks**:
1. Implement square, triangle, sawtooth waveforms
2. Add waveform type selector dropdown
3. Create waveform preview visualizations
4. Add white noise generation
5. Validate signal quality and accuracy

### Phase 3: Musical Note Generation
**Duration**: 2-3 days
**Deliverables**:
- Musical note mapping
- Note selector interface
- Tuning accuracy validation

**Tasks**:
1. Implement musical note frequency calculations
2. Create note/octave selection interface
3. Add common tuning standards (A440, etc.)
4. Build quick-access note buttons
5. Validate pitch accuracy with existing detector

### Phase 4: Advanced Features
**Duration**: 3-4 days
**Deliverables**:
- Frequency sweep functionality
- Chord generation
- Advanced UI controls

**Tasks**:
1. Implement frequency sweep algorithms
2. Create sweep configuration UI
3. Add chord generation with multiple oscillators
4. Build chord selector interface
5. Add preset management system

### Phase 5: Integration & Polish
**Duration**: 1-2 days
**Deliverables**:
- Full debug GUI integration
- Performance optimization
- Testing and validation

**Tasks**:
1. Optimize performance for continuous operation
2. Add comprehensive error handling
3. Integrate with audio inspector for signal verification
4. Create automated test procedures
5. Polish UI and add documentation

## Technical Integration Points

### Audio Engine Integration
- **AudioContext**: Share context with main audio pipeline
- **MediaStream**: Option to route generated signal through processing chain
- **Performance**: Monitor generation impact on overall performance

### Debug Interface Integration
- **Props**: Receive configuration from parent DebugInterface
- **Events**: Emit generation events for logging
- **State**: Share generation status with other components

### Audio Inspector Coordination
- **Signal Verification**: Generated signals should appear in audio inspector
- **Real-time Monitoring**: Monitor generated signal characteristics
- **Feedback Loop**: Use inspector data to validate signal accuracy

## Testing Strategy

### Unit Tests
- Signal generation accuracy
- Frequency precision validation
- Waveform shape verification
- Performance benchmarks

### Integration Tests
- Debug GUI integration
- Audio pipeline compatibility
- Multi-component interaction
- Error handling scenarios

### Manual Testing
- Real-time generation testing
- UI responsiveness validation
- Cross-browser compatibility
- Performance under load

## Performance Considerations

### Optimization Targets
- **Latency**: <5ms signal start time
- **CPU Usage**: <5% during continuous generation
- **Memory**: <10MB buffer allocation
- **Accuracy**: ±1Hz frequency precision

### Implementation Optimizations
- Efficient oscillator node management
- Minimal DOM updates during generation
- Optimized audio buffer handling
- Smart component re-rendering

## Risk Mitigation

### Technical Risks
- **Browser Compatibility**: Test across major browsers
- **Audio Conflicts**: Ensure clean integration with existing audio
- **Performance Impact**: Monitor and optimize resource usage
- **Timing Accuracy**: Validate precision across different systems

### Mitigation Strategies
- Progressive enhancement for browser features
- Graceful degradation for unsupported features
- Performance monitoring and alerts
- Comprehensive browser testing matrix

## Success Criteria

### Functional Success
- ✅ Generate all planned waveform types
- ✅ Accurate frequency control (±1Hz)
- ✅ Seamless start/stop operation
- ✅ Musical note generation with proper tuning
- ✅ Frequency sweep functionality
- ✅ Basic chord generation

### Integration Success
- ✅ Clean integration with debug interface
- ✅ Compatible with audio inspector
- ✅ No interference with main audio pipeline
- ✅ Responsive UI with good UX

### Performance Success
- ✅ <5ms signal generation latency
- ✅ Stable continuous operation
- ✅ Minimal impact on overall system performance
- ✅ Cross-browser compatibility

## Future Enhancements

### Post-MVP Features
- Custom waveform upload capability
- Advanced chord progressions
- Modulation effects (AM/FM)
- Signal recording and playback
- Automated test sequences
- MIDI file playback support

This implementation plan provides a structured approach to building a robust TestSignalGenerator that will significantly enhance the debugging capabilities of the pitch-toy application. 