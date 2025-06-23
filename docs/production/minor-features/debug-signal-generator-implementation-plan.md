# Debug Signal Generator Implementation Plan

## Overview

The TestSignalGenerator is a critical debugging component that generates synthetic audio signals and **routes them through the pitch detection pipeline as a substitute for microphone input**. This component enables developers to validate pitch detection, frequency analysis, and performance monitoring with known test signals instead of requiring external audio input.

## Requirements

### Functional Requirements
1. **Signal Generation**: Generate standard waveforms (sine, square, triangle, sawtooth)
2. **Frequency Control**: Precise frequency selection (20Hz - 20kHz range)
3. **Pipeline Integration**: Route generated signals through the existing audio processing pipeline
4. **Pitch Detection Testing**: Validate pitch detector accuracy with known frequencies
5. **Real-time Control**: Start/stop signal generation with immediate pipeline integration
6. **Audio Inspector Integration**: Generated signals should appear in audio inspector for verification
7. **Performance Testing**: Test audio engine performance with controlled signals

### Technical Requirements
1. **Performance**: Low-latency signal generation (<5ms)
2. **Accuracy**: Frequency accuracy within ±1Hz for pitch detection validation
3. **Pipeline Compatibility**: Seamless integration with existing AudioEngineService
4. **Quality**: Clean signals with minimal artifacts for accurate testing
5. **Stream Substitution**: Replace microphone MediaStream with generated signals

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
    
    // Pipeline Integration
    audio_engine: Option<Rc<RefCell<AudioEngineService>>>,
    generated_stream: Option<web_sys::MediaStream>,
    
    // Configuration
    signal_type: SignalType,
    frequency: f32,
    amplitude: f32,
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
}
```

### Configuration Structure
```rust
#[derive(Clone)]
pub struct SignalConfig {
    signal_type: SignalType,
    frequency: f32,
    amplitude: f32,
    test_mode: TestMode,
}

#[derive(Clone)]
pub enum TestMode {
    ContinuousGeneration,
    PitchAccuracyTest { expected_frequency: f32 },
    FrequencySweep { start_freq: f32, end_freq: f32, duration_seconds: f32 },
}
```

## Implementation Phases

### Phase 1: Basic Signal Generation & Pipeline Integration ✅
**Duration**: 2-3 days
**Status**: COMPLETE
**Deliverables**:
- Basic component structure with Web Audio API integration
- Four waveform types (sine, square, triangle, sawtooth)
- Frequency and amplitude controls
- Start/stop functionality

### Phase 2: Audio Pipeline Integration 🎯 CRITICAL
**Duration**: 1-2 days
**Deliverables**:
- **MediaStream Creation**: Convert generated audio to MediaStream
- **AudioEngine Integration**: Route generated stream through AudioEngineService
- **Pipeline Substitution**: Replace microphone input with test signals
- **AudioInspector Compatibility**: Ensure generated signals appear in inspector

**Tasks**:
1. Create MediaStreamTrack from OscillatorNode output
2. Integrate with AudioEngineService.connect_stream()
3. Add "Test Signal Mode" toggle to AudioControlPanel
4. Verify signals appear in AudioInspector
5. Test pitch detection accuracy with known frequencies



## Technical Integration Points

### AudioEngine Integration
- **MediaStream Generation**: Create MediaStream from OscillatorNode
- **Stream Substitution**: Replace getUserMedia() input with generated stream
- **Pipeline Compatibility**: Ensure generated signals work with existing audio processing
- **Performance Monitoring**: Track impact on overall system performance

### AudioInspector Integration
- **Signal Verification**: Generated signals should appear in real-time display
- **Frequency Analysis**: Validate FFT analysis of generated signals
- **Pitch Detection**: Verify pitch detector accuracy with known frequencies
- **Buffer Monitoring**: Observe audio buffer characteristics

### DebugInterface Integration
- **Control Panel**: Integrate with existing AudioControlPanel
- **Mode Switching**: Toggle between microphone and test signal input
- **Status Display**: Show current signal generation state
- **Results Display**: Show pitch detection accuracy results

## Success Criteria

### Functional Success
- ✅ Generate four waveform types with accurate frequencies
- ✅ Real-time frequency and amplitude control
- 🎯 **Route signals through pitch detection pipeline**
- 🎯 **Validate pitch detector accuracy with known frequencies**
- 🎯 **Replace microphone input for testing purposes**

### Integration Success
- 🎯 **Generated signals appear in AudioInspector**
- 🎯 **Pitch detection results match expected frequencies (±1Hz)**
- 🎯 **Seamless switching between microphone and test signals**
- 🎯 **No interference with existing audio pipeline**

### Performance Success
- ✅ <5ms signal generation latency
- 🎯 **<10ms additional pipeline latency**
- 🎯 **Minimal impact on overall system performance**
- ✅ Cross-browser compatibility

## Testing Strategy

### Integration Tests
- **Pipeline Routing**: Verify signals reach pitch detector
- **Accuracy Validation**: Test known frequencies (440Hz, 880Hz, 1000Hz)
- **AudioInspector Display**: Confirm signals appear in real-time display
- **Performance Impact**: Measure additional latency introduced

### Validation Tests
- **Pitch Detection Accuracy**: ±1Hz tolerance for pure tones
- **Frequency Range**: Test across full 20Hz-20kHz range
- **Waveform Variety**: Validate all four waveform types
- **Real-time Switching**: Smooth transitions between signals

### User Experience Tests
- **Control Responsiveness**: Real-time parameter changes
- **Mode Switching**: Easy toggle between microphone/test modes
- **Error Recovery**: Graceful handling of audio context issues
- **Documentation**: Clear usage instructions

## Risk Mitigation

### Technical Risks
- **MediaStream Creation**: Browser compatibility with generated streams
- **Pipeline Integration**: Potential interference with existing audio processing
- **Performance Impact**: Additional latency or CPU usage
- **Audio Context Conflicts**: Multiple audio contexts causing issues

### Mitigation Strategies
- **Progressive Enhancement**: Fallback for unsupported features
- **Performance Monitoring**: Track and alert on performance degradation
- **Isolated Audio Context**: Separate context for test signal generation
- **Comprehensive Testing**: Validate across major browsers

This implementation plan provides a **focused approach** to building a TestSignalGenerator that serves its actual purpose: **validating the pitch detection pipeline with controlled test signals** rather than being a standalone signal generator. 