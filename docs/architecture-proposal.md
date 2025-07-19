# Pitch Toy Architecture Proposal

## Overview

This document outlines a three-layer architecture for the pitch toy application. The architecture follows a clean separation of concerns, with each layer having distinct responsibilities and well-defined interfaces.

## Architecture Layers

### 1. Audio Engine Layer (Foundation)
**Status**: Mostly implemented

**Responsibilities**:
- Hardware interface via browser APIs
- Audio stream capture and processing
- Real-time pitch detection and analysis
- Volume level analysis
- Audio worklet management
- Low-level signal processing
- FFT analysis (roadmap)

**Key Components**:
- Audio Worklet for real-time processing
- Pitch detection
- Volume analysis
- Platform abstraction
- FFT implementation (roadmap)

**Output**: Raw audio metrics (pitch frequency, volume levels, signal quality indicators, frequency spectrum data)

### 2. Data Model Layer (Transformation)
**Status**: To be implemented

**Responsibilities**:
- Transform raw audio data into meaningful representations
- Manage application state
- Apply data smoothing and filtering
- Calculate derived metrics
- Handle temporal patterns and histories
- Provide rich data structures for visualization
- Tuning system management (equal temperament, just intonation, etc.)
- Note detection and musical interpretation

**Key Components**:
- State management
- Data transformers
- Pattern recognition
- History buffers
- Event generation
- Tuning system implementations
- Musical note mapping

**Input**: Raw audio metrics from Engine Layer
**Output**: Structured data models ready for visualization

### 3. Presentation Layer (Visualization)
**Status**: To be implemented

**Responsibilities**:
- WebGL-based rendering (all visuals)
- User interface elements (rendered in WebGL)
- Interactive controls
- Visual representations of audio data
- User input handling
- Animation and transitions
- Theme support (roadmap)

**Key Components**:
- WebGL renderer
- UI components (WebGL-rendered, not HTML)
- Input handlers
- Animation system
- Visual effects
- Theme system (roadmap)

**Input**: Structured data from Model Layer
**Output**: Visual feedback and user interactions

## Data Flow

```
User Input → Presentation Layer
              ↓
         Model Layer
              ↓
         Engine Layer
              ↓
        Audio Hardware
              ↓
         Engine Layer (processing)
              ↓
         Model Layer (transformation)
              ↓
     Presentation Layer (visualization)
              ↓
         User Display
```

## Key Design Principles

1. **Unidirectional Data Flow**: Data flows down through layers, events flow up
2. **Layer Independence**: Each layer can be developed and tested independently
3. **Clear Interfaces**: Well-defined contracts between layers
4. **No Direct Cross-Layer Communication**: Layers only communicate with adjacent layers
5. **Reactive Updates**: Changes propagate automatically through the system
6. **Observable/Action Pattern**: Layers communicate using observable data (for continuous values) and actions (for commands/events)

## Interface Definitions

### Engine → Model Interface
- **Observable Data**: Pitch frequency (Hz)
- **Observable Data**: Volume level (amplitude Peak + RMS)
- **Observable Data**: Signal quality metrics [TODO: should probably be bundled with pitch] [COMMENT: Yes, bundling makes sense. Consider a single "AudioAnalysis" observable containing pitch, quality, and timestamp together]
- **Observable Data**: Timestamp [TODO: timestamp should probably be bundled with other messages, or does it make sense to have it solo?] [COMMENT: Bundle with data messages. Solo timestamps aren't useful - each data point needs its temporal reference]
- **Observable Data**: Error states
- **Observable Data**: FFT data (roadmap)

### Model → Engine Interface
- **Action**: Request microphone permission
- **Action**: Start/stop audio processing
- **Action**: Configure sample rate [TODO: is this necessary?] [COMMENT: Probably not needed initially. Audio context typically handles this automatically. Could be roadmap item for advanced users]
- **Action**: Configure buffer size [TODO: is this necessary?] [COMMENT: May be useful for latency vs performance tradeoffs. Keep as roadmap item - start with sensible defaults]

### Model → Presentation Interface
- **Observable Data**: Transformed visualization data [TODO: elaborate on what this means] [COMMENT: Normalized/scaled values ready for rendering (e.g., pitch mapped to Y-coordinates, volume to size/opacity, note names, cents deviation)]
- **Observable Data**: Application state [TODO: specify] [COMMENT: Recording status, active tuning system, selected visualization mode, error states, permission status]
- **Observable Data**: User-friendly metrics [TODO: specify] [COMMENT: Note name, octave, cents off from nearest note, volume in human-friendly units, pitch stability indicator]
- **Observable Data**: Historical data [TODO: specify] [COMMENT: Rolling buffers of recent pitch/volume values for trails, averages, trends. Time window configurable by visualization needs]
- **Observable Data**: Animation parameters [TODO: specify] [COMMENT: Interpolation values, easing functions, transition states between visualization modes, particle system parameters]

### Presentation → Model Interface
- **Action**: Request microphone permission
- **Action**: User interactions [TODO: specify] [COMMENT: Click/tap on visualization elements, drag gestures, keyboard shortcuts, WebGL canvas interactions]
- **Action**: Configuration changes [TODO: specify] [COMMENT: Select tuning system, change visualization mode, adjust sensitivity, toggle features, set reference pitch]
- **Action**: Control commands [TODO: specify] [COMMENT: Start/stop recording, reset visualization, pause/resume, clear history, trigger calibration]
- **Action**: Theme selection (roadmap)

## Benefits of This Architecture

1. **Modularity**: Each layer can be modified without affecting others
2. **Testability**: Layers can be tested in isolation
3. **Scalability**: New features can be added at the appropriate layer
4. **Maintainability**: Clear separation makes code easier to understand
5. **Flexibility**: Different visualizations can use the same model data

## Implementation Notes

- Layer APIs will be carefully defined in separate documents when implementation begins
- Each layer should maintain clear boundaries and avoid tight coupling
- The current tuning system implementation in the engine will migrate to the Model Layer

## Next Steps

1. Define detailed interfaces between layers
2. Implement the Model Layer foundation
3. Create basic Presentation Layer structure
4. Establish communication protocols
5. Build example visualizations

## Open Questions

1. Should the Model Layer support multiple simultaneous transformations?
2. What specific data structures best serve the Presentation Layer?
3. How should configuration flow through the layers?
4. What error handling strategies should each layer employ?