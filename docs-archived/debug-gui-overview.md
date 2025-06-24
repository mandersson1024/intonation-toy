# Debug GUI Overview

## Introduction

The Debug GUI is a comprehensive developer interface for the pitch-toy audio pitch detection application. It provides real-time monitoring, performance analysis, and debugging capabilities for audio processing workflows. The interface is built using the Yew framework and is designed for technical debugging and functional testing rather than end-user interaction.

## Architecture

The Debug GUI follows a modular component-based architecture with the main `DebugInterface` component serving as the root container that orchestrates multiple specialized debugging panels.

### Core Components

#### 1. DebugInterface (`src/components/debug_interface.rs`)
- **Purpose**: Main container component that orchestrates all debugging panels
- **Layout**: Grid-based responsive layout with 6 distinct sections
- **Update Frequency**: Configurable update interval (default: 1000ms)
- **Features**:
  - Real-time status indicator
  - Responsive grid layout that adapts to screen size
  - Centralized prop passing to child components

#### 2. AudioControlPanel (`src/components/audio_control_panel.rs`)
- **Purpose**: Manages microphone permissions and displays the audio engine's core status.
- **Key Features**:
  - Integrates the `MicrophonePermission` component to handle user consent.
  - Displays a single, clear status for the audio engine (e.g., Uninitialized, Ready, Processing).
  - Automatically initializes and connects the audio engine once permission is granted.
- **State Management**: Primarily tracks the `AudioEngineState`.
- **Integration**: Works with `MicrophonePermission` component for stream access and `AudioEngineService` for engine control.

#### 3. MetricsDisplay (`src/components/metrics_display.rs`)
- **Purpose**: Real-time performance metrics visualization
- **Metrics Tracked**:
  - Total latency (buffer + processing)
  - Processing rate (Hz)
  - Latency compliance status
  - Update frequency and count
- **Features**:
  - Automatic metric updates via intervals
  - Manual refresh capability
  - Color-coded compliance indicators
  - Live monitoring status display

#### 4. DebugPanel (`src/components/debug_panel.rs`)
- **Purpose**: Error state visualization and debugging
- **Key Features**:
  - Error list display with severity indicators
  - Error categorization (Browser, WASM, WebAudio, Media, etc.)
  - Detailed error inspection with expandable views
  - Error history management
  - Refresh and clear functionality
- **Error Categories**: 16 different error categories with visual icons
- **UI Elements**: Interactive error selection, detailed breakdowns

#### 5. AudioInspector (`src/components/audio_inspector.rs`)
- **Purpose**: Deep audio data analysis and visualization
- **Capabilities**:
  - Real-time audio buffer inspection
  - Frequency domain analysis (FFT data)
  - Pitch detection results monitoring
  - Audio data history tracking
- **Data Types**:
  - `AudioBufferData`: Raw sample data with metadata
  - `FrequencyData`: FFT bins and spectral analysis
  - `PitchData`: Detected pitch, confidence, musical note information
- **Features**: Start/stop monitoring, buffer history management, configurable data views

#### 6. PerformanceMonitor (`src/components/performance_monitor.rs`)
- **Purpose**: Comprehensive performance analysis dashboard
- **Monitoring Areas**:
  - Memory usage (heap utilization, GC metrics)
  - Processing breakdown (timing analysis per component)
  - WASM metrics (compilation time, memory pages, function calls)
  - Performance history with scoring system
- **Features**:
  - Real-time monitoring with configurable intervals
  - Performance scoring algorithm
  - Historical performance tracking
  - Visual performance grading (Excellent/Good/Fair/Poor)

#### 7. TestSignalGenerator (`src/components/test_signal_generator.rs`)
- **Purpose**: Signal generation for testing audio processing
- **Current State**: Placeholder with planned capabilities
- **Planned Features**: Waveform generation, frequency sweeps, musical chords

## Development Workflow

### Usage in Development
1. **Initialization**: Load the Debug GUI to assess system readiness
2. **Engine Setup**: Use Audio Control Panel to initialize and connect audio engine
3. **Monitoring**: Enable real-time monitoring across all panels
4. **Testing**: Use test controls and signal generation for validation
5. **Analysis**: Review performance metrics and error logs for optimization

### Testing Support
- **Functional Testing**: Engine controls for start/stop/test operations
- **Performance Testing**: Real-time metrics with compliance checking
- **Error Testing**: Error injection and recovery validation
- **Audio Testing**: Signal generation and data inspection

## Configuration Options

### Update Intervals
- Default: 1000ms (1 second)
- Range: 100ms - 5000ms
- Component-specific overrides available

### Display Options
- Raw buffer display toggle
- Frequency data visualization control
- Pitch data display preferences
- Performance history depth settings

### Monitoring Scope
- Memory statistics inclusion
- Processing breakdown detail level
- WASM metrics visibility
- Performance history tracking
