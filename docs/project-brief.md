# Pitch Toy - Real-time Pitch Detection and Visualization App

## Project Specification

**Language**: Rust  
**Target Platform**: Browser (WebAssembly)  
**Frontend Framework**: Yew  
**Rendering Backend**: wgpu  
**Build Tool**: Trunk  
**License**: MIT  

## Executive Summary

Pitch Toy is a high-performance, browser-based real-time pitch detection and visualization application. Built with Rust and WebAssembly for maximum performance, it provides musicians, vocalists, and audio enthusiasts with precise pitch feedback and beautiful visualizations. The application emphasizes low-latency audio processing, smooth 60fps graphics, and an extensible architecture for future enhancements.

## Core Value Proposition

- **Real-time Performance**: Sub-50ms latency from microphone to visual feedback
- **Precision**: High-confidence pitch detection with musical note mapping
- **Visual Excellence**: GPU-powered graphics delivering smooth 60fps visualizations
- **Developer Experience**: Comprehensive debugging tools and modular architecture
- **Accessibility**: Cross-platform browser compatibility with no installation required

## Project Goals

### Primary Goals
1. **Accurate Real-time Pitch Detection**: Implement robust pitch detection algorithms with confidence scoring
2. **Responsive Visualization**: Create fluid, real-time visual feedback for pitch and volume
3. **Performance Excellence**: Achieve 60fps rendering with minimal audio latency
4. **Developer Productivity**: Provide comprehensive debugging and development tools

### Secondary Goals
1. **Extensible Architecture**: Enable easy addition of new visualization modes and audio processing features
2. **Themeable Interface**: Support multiple visual themes for different user preferences
3. **Educational Value**: Serve as a reference implementation for Rust/WASM audio applications

## System Architecture

```
[Mic Input] ──▶ [Audio Processor] ──▶ [Event Dispatcher]
                                        │
                                        ▼
                           ┌────────────────────────┐
                           ▼                        ▼
               [Presentation Layer]          [Dev Console]
                     │                            │
                     ▼                            ▼
            [Theme Manager]              [Realtime Meters (Yew)]
                     │
                     ▼
            [Graphics Renderer (wgpu)]
```

### Architecture Principles

1. **Event-Driven Design**: All components communicate via typed events through the Event Dispatcher
2. **Separation of Concerns**: Clear boundaries between audio processing, visualization, and UI management
3. **Performance Isolation**: GPU rendering isolated from audio processing to prevent interference
4. **Modular Development**: Each component can be developed and tested independently
5. **Configuration-Driven**: Build profiles and feature flags control development vs. production behavior

### Data Flow

1. **Audio Input Stream**: Microphone → Audio Processor (real-time buffer processing)
2. **Pitch Events**: Audio Processor → Event Dispatcher → Presentation Layer
3. **Visualization Commands**: Presentation Layer → Graphics Renderer
4. **Debug Data**: All components → Dev Console (development builds only)
5. **Theme Changes**: Theme Manager → All visual components

## Performance Requirements

### Audio Processing
- **Latency Target**: ≤ 30ms (production), ≤ 50ms (development)
- **Sample Rate**: 44.1kHz standard, 22.05kHz - 96kHz (development testing)
- **Buffer Size**: 1024 samples (production), 128-2048 samples (development)
- **Processing Overhead**: ≤ 5% CPU usage on modern devices

### Graphics Rendering
- **Frame Rate**: Consistent 60fps
- **Frame Time**: ≤ 16.67ms per frame
- **GPU Memory**: ≤ 50MB texture and buffer allocation
- **Rendering Resolution**: Adaptive based on display size

### WebAssembly Bundle
- **Production Bundle**: ≤ 500KB compressed
- **Development Bundle**: ≤ 2MB (includes debug symbols and testing framework)
- **Load Time**: ≤ 3 seconds on 3G connection

## Technical Specifications

### Core Dependencies

**Minimal Required (MVP):**
```toml
[dependencies]
yew = { version = "0.21", features = ["csr"] }      # React-like framework for Rust/WASM
web-sys = "0.3"                                     # Web API bindings (getUserMedia, Canvas)
pitch-detection = "0.3"                             # Multi-algorithm pitch detection (YIN, MPM)
rustfft = "6.0"                                     # FFT implementation for spectral analysis
```

**Optional Enhancements:**
```toml
# Add as needed for advanced features:
wgpu = "0.17"                                       # GPU graphics (if Canvas 2D insufficient)
wasm-bindgen = "0.2"                                # Custom Rust-JS bindings (if needed)
serde = { version = "1.0", features = ["derive"] }  # Settings persistence
js-sys = "0.3"                                      # JS globals (if web-sys insufficient)
```

### Browser Compatibility
- **Required APIs**: WebAssembly, Web Audio API (AudioWorklet), getUserMedia
- **Minimum Browser Versions** (AudioWorklet support):
  - Chrome 66+, Firefox 76+, Safari 14.1+, Edge 79+
- **Mobile Support**: iOS Safari 14.5+, Chrome Android 66+
- **Note**: AudioWorklet has more recent browser requirements than basic Web Audio API

## Detailed Module Specifications

### 1. Development Console (Yew Component) - HIGH PRIORITY

**Responsibilities**:
- Interactive debugging interface for development builds
- Command execution system for runtime configuration
- Primary development interface during early implementation phases

**Implementation Details**:
```rust
#[function_component(DevConsole)]
pub fn dev_console() -> Html {
    let console_visible = use_state(|| false);
    let command_history = use_state(Vec::<String>::new);
    let metrics = use_context::<MetricsContext>();
    
    // Console rendering logic
}

pub trait DevCommand: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: Vec<String>) -> DevCommandOutput;
    fn autocomplete(&self, partial: &str) -> Vec<String>;
}

pub struct DevCommandOutput {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}
```

**Built-in Commands**:
- `mic request`: Request microphone permissions (logs errors to console)
- `mic status`: Show current microphone and audio context status
- `signal sine <freq>`: Generate sine wave test signal
- `signal sweep <start> <end> <duration>`: Generate frequency sweep
- `signal off`: Stop test signal and return to microphone input
- `theme set <name>`: Switch active theme

### 2. Debug Overlay (Yew Component)

**Responsibilities**:
- Non-intrusive performance metrics display
- Real-time audio processing statistics
- Frame rate and latency monitoring
- Memory usage tracking

**Implementation Details**:
```rust
#[function_component(DebugOverlay)]
pub fn debug_overlay() -> Html {
    let metrics = use_context::<MetricsContext>().unwrap();
    let fps_counter = use_state(|| FpsCounter::new());
    
    html! {
        <div class="debug-overlay">
            <div class="metric">{"FPS: "}{metrics.current_fps}</div>
            <div class="metric">{"Latency: "}{metrics.audio_latency_ms}{"ms"}</div>
            <div class="metric">{"Volume: "}{metrics.current_volume_db}{"dB"}</div>
            <div class="metric">{"Pitch: "}{metrics.current_frequency}{"Hz"}</div>
        </div>
    }
}
```

### 3. Test Signal Generator Module

**Responsibilities**:
- Generate deterministic audio signals for development and testing
- Provide configurable signal types for algorithm validation
- Replace microphone input during development and automated testing
- Support musical and scientific test scenarios

**Signal Types**:
```rust
pub enum SignalType {
    Sine { frequency: f32, amplitude: f32 },
    Harmonic { 
        fundamental: f32, 
        harmonics: Vec<(u32, f32)>,  // (harmonic_number, amplitude)
    },
    FrequencySweep { 
        start_freq: f32, 
        end_freq: f32, 
        duration_ms: u32,
    },
    MusicalInterval {
        base_freq: f32,
        interval: MusicalInterval,
        tuning: TuningSystem,
    },
    WhiteNoise { amplitude: f32 },
    PinkNoise { amplitude: f32 },
    Composite { signals: Vec<SignalType> },  // Multiple simultaneous signals
}

pub enum MusicalInterval {
    Unison, MinorSecond, MajorSecond, MinorThird, MajorThird,
    PerfectFourth, Tritone, PerfectFifth, MinorSixth, MajorSixth,
    MinorSeventh, MajorSeventh, Octave,
}
```

**Implementation Details**:
```rust
pub struct TestSignalGenerator {
    sample_rate: f32,
    current_signal: Option<SignalType>,
    phase_accumulator: f32,
    sweep_position: f32,
    rng: SmallRng,                    // For noise generation
    output_buffer: Vec<f32>,          // Pre-allocated 128-sample buffer
}

impl TestSignalGenerator {
    pub fn generate_chunk(&mut self) -> &[f32; 128] {
        // Generate 128 samples matching AudioWorklet output format
    }
    
    pub fn set_signal(&mut self, signal: SignalType) {
        // Switch signal type, reset phase accumulators
    }
    
    pub fn stop_signal(&mut self) {
        // Return to silence
    }
}
```

**Key Features**:
- **AudioWorklet compatible**: Generates 128-sample chunks at 44.1kHz/48kHz
- **Phase-coherent**: Smooth transitions between different signals
- **Configurable amplitude**: Test different input levels
- **Musical accuracy**: Support both equal temperament and just intonation
- **Noise injection**: Test algorithm robustness with controlled noise
- **Deterministic output**: Repeatable signals for automated testing

**Development Commands** (via Debug Console):
- `signal sine 440`: Generate A4 sine wave
- `signal harmonic 261.63 [1.0, 0.5, 0.25]`: C4 with harmonics
- `signal sweep 200 800 5000`: 5-second sweep from 200Hz to 800Hz
- `signal interval 440 perfect-fifth equal`: A4 + E5 in equal temperament  
- `signal noise white 0.1`: Low-level white noise
- `signal off`: Return to silence

**Testing Scenarios**:
- **Algorithm validation**: Known frequencies for YIN vs FFT comparison
- **Tuning system testing**: Same musical interval in different tuning systems
- **Confidence scoring**: How algorithms handle noise, harmonics, and edge cases
- **Performance testing**: Consistent load for latency and CPU usage measurement
- **Edge case testing**: Very low/high frequencies, rapid changes, polyphonic content

### 4. Microphone Audio Input Module

**Responsibilities**:
- **Microphone permission management**: Handle getUserMedia requests and errors
- Real-time audio stream processing using Web Audio API AudioWorklet
- Stream reconnection logic

**Web Audio API Strategy**:
- **AudioWorklet**: Modern, performant approach running on audio thread
- **Fixed 128-sample chunks**: Web Audio API limitation, cannot be configured
- **Internal buffering**: Accumulate multiple 128-sample chunks for pitch detection
- **No ScriptProcessorNode**: Deprecated API, avoid for future compatibility

**Implementation Details**:
```rust
pub struct MicrophoneInput {
    stream: Option<MediaStream>,
    audio_context: AudioContext,
    worklet_node: AudioWorkletNode,
    internal_buffer: CircularBuffer<f32>,
    analysis_window_size: usize,  // How many 128-sample chunks to accumulate
    sample_rate: f32,
}

pub struct AudioChunk {
    samples: [f32; 128],  // Fixed size from AudioWorklet
    timestamp: f64,
    channel_count: u32,
}
```

**Key Features**:
- AudioWorklet-based processing for low latency and reliability
- **getUserMedia handling**: Request and manage microphone permissions
- **Public API**: Methods callable from console commands (`request_microphone()`, `get_status()`, etc.)
- Automatic stream recovery on device disconnection
- Internal buffering: accumulate 2-16 chunks (256-2048 samples) before analysis
- Support for standard sample rates (primarily 44.1kHz, 48kHz)
- Real-time volume level monitoring for input validation
- Console logging for stream status and errors (not UI messages)

### 5. Audio Processor Module

**Responsibilities**:
- High-performance pitch detection using time-domain and frequency-domain algorithms
- Musical note mapping with multiple tuning systems (equal temperament, just intonation)
- Pitch confidence estimation and quality assessment
- **Spectral analysis**: FFT-based frequency domain analysis for visualization and enhanced confidence
- Internal buffering and sliding window management for optimal accuracy

**Buffering Strategy**:
- **Receive**: 128-sample chunks from AudioWorklet at ~3ms intervals
- **Accumulate**: Collect 4-16 chunks (512-2048 samples) for analysis
- **Sliding Window**: Overlap analysis windows by 50% for smooth detection
- **Configurable Window Size**: Adjust based on performance vs. accuracy needs

**Implementation Details**:
```rust
pub struct AudioProcessor {
    sample_buffer: CircularBuffer<f32>,
    analysis_window_size: usize,      // 512-2048 samples
    hop_size: usize,                  // Window overlap (typically window_size / 2)
    pitch_detector: Box<dyn PitchDetector>,
    note_mapper: NoteMapper,
    tuning_system: TuningSystem,
    
    // FFT components for spectral analysis
    fft_planner: FftPlanner<f32>,
    fft_buffer: Vec<Complex<f32>>,    // Pre-allocated, reused for each analysis
    window_function: Vec<f32>,        // Hamming/Hann window, computed once
    spectrum_buffer: Vec<f32>,        // Pre-allocated magnitude spectrum output
    
    last_analysis_time: f64,
}

pub struct PitchEvent {
    pub frequency: f32,
    pub note: String,
    pub octave: i8,
    pub cents_offset: f32,
    pub confidence: f32,
    pub volume_db: f32,
    pub timestamp: f64,
}

pub struct VolumeEvent {
    pub rms_level: f32,
    pub peak_level: f32,
    pub db_level: f32,
    pub timestamp: f64,
}

pub struct SpectrumEvent {
    pub frequencies: Arc<Vec<f32>>,   // Shared ownership to avoid copying
    pub magnitudes: Arc<Vec<f32>>,    // Shared ownership to avoid copying
    pub sample_rate: f32,
    pub fft_size: usize,
    pub timestamp: f64,
}

pub enum TuningSystem {
    EqualTemperament { a4_frequency: f32 },  // Standard: 440Hz
    JustIntonation { 
        root_frequency: f32,                 // Base frequency (e.g., C4 = 261.63Hz)
        ratio_set: JustIntonationRatios,     // Harmonic ratios
    },
}

pub struct JustIntonationRatios {
    pub unison: (u32, u32),      // 1:1
    pub minor_second: (u32, u32), // 16:15
    pub major_second: (u32, u32), // 9:8
    pub minor_third: (u32, u32),  // 6:5
    pub major_third: (u32, u32),  // 5:4
    pub perfect_fourth: (u32, u32), // 4:3
    pub tritone: (u32, u32),      // 45:32 or 64:45
    pub perfect_fifth: (u32, u32), // 3:2
    pub minor_sixth: (u32, u32),  // 8:5
    pub major_sixth: (u32, u32),  // 5:3
    pub minor_seventh: (u32, u32), // 16:9
    pub major_seventh: (u32, u32), // 15:8
}
```

**Performance Considerations**:

**Memory Management Strategy**:
- **Pre-allocated buffers**: FFT and spectrum buffers allocated once, reused for all analyses
- **Zero-copy event data**: Use `Arc<Vec<T>>` for large data in events to avoid copying
- **Circular buffer design**: In-place operations on audio samples, slice-based analysis windows
- **Memory pool pattern**: Reuse analysis buffers to minimize allocations during real-time processing

**Potential Copy Operations and Mitigation**:
- **Audio accumulation**: ~8KB per analysis cycle (acceptable for real-time)
- **FFT preparation**: In-place conversion from f32 to Complex<f32> where possible  
- **Event publishing**: Shared ownership via Arc to avoid spectrum data copying (~4KB saved per event)
- **Cross-module boundaries**: Use references and slices instead of owned data where feasible

**Performance Targets**:
- **Analysis latency**: ≤ 5ms for buffer processing and FFT computation
- **Memory overhead**: ≤ 100KB total for all audio processing buffers
- **Allocation frequency**: Zero allocations during steady-state real-time processing
- **Copy elimination**: Large data (>1KB) shared via Arc, small data (<100 bytes) copied

**Algorithm Options** (evaluated for real-time performance):

1. **YIN Algorithm** (Primary for pitch detection)
   - De facto standard for monophonic pitch detection
   - Excellent for musical instruments and vocals
   - Available in `pitch-detection` crate with WASM support
   - Lower CPU overhead than FFT-based methods

2. **FFT Spectral Analysis** (Secondary for visualization and confidence)
   - Real-time spectrum visualization for "Nerds" theme
   - Harmonic analysis for enhanced pitch confidence
   - Peak detection in frequency domain
   - Windowing with Hamming/Hann functions for spectral accuracy

3. **Hybrid Approach** (Recommended implementation)
   - YIN for primary pitch detection (low latency)
   - FFT for spectrum visualization and confidence validation
   - Parallel processing: time-domain and frequency-domain analysis
   - Confidence scoring based on agreement between methods

**Implementation Strategy**:
- Start with `pitch-detection` crate's YIN implementation for core pitch detection
- Add `rustfft` for real-time spectrum analysis and visualization
- Use FFT output for enhanced confidence scoring and harmonic analysis
- Provide spectrum data for "Nerds" theme scientific visualizations

### 6. Event Dispatcher Module

**Responsibilities**:
- Type-safe event routing between all system components
- Event queuing and prioritization for performance-critical paths
- Subscription management with automatic cleanup
- Event logging and debugging support

**Implementation Details**:
```rust
pub trait Event: Send + Sync + 'static {}

pub struct EventDispatcher {
    subscribers: HashMap<TypeId, Vec<Box<dyn EventHandler>>>,
    event_queue: VecDeque<Box<dyn Event>>,
    debug_mode: bool,
}

pub trait EventHandler<T: Event>: Send + Sync {
    fn handle(&self, event: &T);
}
```

### 7. Presentation Layer Module

**Responsibilities**:
- React to audio events and coordinate visual responses
- Manage immersive visualization state and transitions
- Interface with Theme Manager for GPU theme rendering
- **Generate rendering commands**: Translate audio data to GPU graphics commands
- **User interaction handling**: Process input events from GPU-rendered interface
- **Animation coordination**: Manage smooth transitions and visual effects

**Implementation Details**:
```rust
pub struct PresentationLayer {
    current_pitch: Option<PitchEvent>,
    current_volume: Option<VolumeEvent>,
    visualization_mode: VisualizationMode,
    theme: Theme,
    render_queue: Vec<RenderCommand>,
}

pub enum VisualizationMode {
    PitchMeter,
    Spectrum,
    Tuner,
    Waveform,
}

pub enum RenderCommand {
    DrawPitchIndicator { frequency: f32, confidence: f32 },
    DrawVolumeBar { level: f32 },
    DrawSpectrum { frequencies: Vec<f32>, magnitudes: Vec<f32> },
    UpdateBackground { color: Color },
}
```

### 8. Theme Manager Module

**Responsibilities**:
- Centralized theme configuration and switching
- Color palette and styling management
- Runtime theme updates with smooth transitions
- Theme persistence and user preferences

**Implementation Details**:
```rust
pub struct ThemeManager {
    current_theme: Theme,
    available_themes: HashMap<String, Theme>,
    transition_state: Option<ThemeTransition>,
}

pub struct Theme {
    pub name: String,
    pub background_color: Color,
    pub primary_color: Color,
    pub secondary_color: Color,
    pub accent_color: Color,
    pub text_color: Color,
    pub pitch_colors: PitchColorMap,
    pub fonts: FontConfig,
}

pub struct PitchColorMap {
    pub note_colors: HashMap<String, Color>,
    pub confidence_gradient: Vec<Color>,
    pub volume_gradient: Vec<Color>,
}
```

**Built-in Themes**:
- **Kids**: Bright, playful colors with large, friendly visual elements
- **Nerds**: Scientific aesthetic with precise measurements, grid lines, and technical styling for music theory enthusiasts

### 9. Graphics Renderer (wgpu) Module

**Responsibilities**:
- **Primary user interface rendering**: All end-user interactions via GPU graphics
- High-performance GPU-accelerated rendering at 60fps
- **Immersive visualization**: Full-screen pitch detection and spectrum display
- **Interactive controls**: GPU-rendered buttons, sliders, and interface elements
- **Theme rendering**: Complete visual themes ("Kids" and "Nerds") in GPU
- Efficient buffer management and texture handling

**Implementation Details**:
```rust
pub struct GraphicsRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
}

pub struct RenderState {
    pub pitch_indicator_position: f32,
    pub volume_level: f32,
    pub spectrum_data: Vec<f32>,
    pub background_color: [f32; 4],
    pub animation_time: f32,
}
```

**Rendering Capabilities**:
- **Full immersive interface**: Complete user experience rendered via GPU
- **Interactive controls**: GPU-rendered buttons, sliders, and interface elements
- **Real-time pitch visualization**: Smooth interpolated pitch indicators
- **Animated volume meters**: Peak hold and real-time level displays
- **Spectral analysis display**: Customizable frequency spectrum visualization
- **Theme-based rendering**: Complete visual overhaul between "Kids" and "Nerds" themes
- **Particle effects**: Enhanced visual feedback and educational animations
- **Responsive layout**: Adaptive rendering for different screen sizes and orientations


## Implementation Phases

### Phase 1: Foundation
- Set up Rust/WASM project structure with Trunk
- Implement basic Yew application framework
- Build development console with microphone commands
- Create audio input module (accepts pre-configured MediaStream)
- Basic audio buffer processing and event system

### Phase 2: Audio Processing
- Implement YIN-based pitch detection algorithm
- Add musical note mapping and confidence scoring
- Create volume level detection and RMS calculation
- Develop audio processing unit tests and benchmarks

### Phase 3: Visualization Core
- Set up wgpu rendering pipeline
- Implement basic pitch indicator visualization
- Create volume meter with smooth animations
- Add theme system with light/dark themes

### Phase 4: Immersive User Interface
- Develop immersive wgpu-rendered user interface
- Implement theme switching with GPU-rendered themes ("Kids" and "Nerds")
- Create responsive GPU rendering for mobile and desktop
- Build interactive wgpu-based controls and feedback systems

### Phase 5: Enhanced Development Tools
- Enhance development console with advanced commands
- Add debug overlay with performance metrics
- Implement comprehensive logging and diagnostics
- Create automated testing and profiling tools

### Phase 6: Polish & Optimization
- Performance optimization and bundle size reduction
- Cross-browser compatibility testing and fixes
- Documentation and deployment preparation
- User acceptance testing and bug fixes

## Testing Strategy

### Unit Testing
- Audio processing algorithms with known signal inputs
- Mathematical functions (FFT, autocorrelation, note mapping)
- Event system message routing and subscription management
- Theme system color calculations and transitions

### Integration Testing
- End-to-end audio pipeline from microphone to visualization
- Real-time performance under various system loads
- Cross-browser compatibility with automated testing
- Mobile device testing with different screen sizes

### Performance Testing
- Audio latency measurement and optimization
- Graphics rendering frame rate consistency
- Memory usage profiling and leak detection
- WebAssembly bundle size and load time analysis

## Deployment and Distribution

### Build Configurations
- **Development**: Full debugging, source maps, comprehensive logging
- **Staging**: Production optimizations with debug symbols for testing
- **Production**: Maximum optimization, minimal bundle size, error reporting

### Distribution Strategy
- Static file hosting (GitHub Pages, Netlify, Vercel)
- CDN integration for global performance
- Progressive Web App (PWA) capabilities for offline use
- Browser extension version for music software integration

