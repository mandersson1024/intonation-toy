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
- **Observable Data**: Option<AudioAnalysis> containing:
  - volume_level: Volume { peak: f32, rms: f32 }
  - pitch: enum { Detected(f32, clarity), NotDetected }
  - FFT data: Option<Vec<f32>> (roadmap)
  - Timestamp
- **Observable Data**: Vec<AudioError> (multiple simultaneous errors possible)
  - AudioError variants:
    - MicrophoneNotAvailable
    - ProcessingError(details: String)
    - BrowserApiNotSupported
    - AudioContextInitFailed
    - AudioContextSuspended
- **Observable Data**: PermissionState enum
  - variants:
     NotRequested 
     Requested
     Granted
     Denied

### Model → Engine Interface
- **Action**: RequestMicrophonePermissionAction

### Model → Presentation Interface
- **Observable Data**: volume_level     - Volume { peak: f32, rms: f32 }
- **Observable Data**: pitch            - enum { Detected(f32, clarity), NotDetected }
- **Observable Data**: accuracy         - Accuracy { closest_note, accuracy }
- **Observable Data**: tuning_system    - TuningSystem
- **Observable Data**: errors           - Vec<Error>
- **Observable Data**: permission_state - PermissionState

### Presentation → Model Interface
- **Action**: RequestMicrophonePermissionAction
- **Action**: SetTuningSystemAction { tuning_system: TuningSystem }
- **Action**: SetRootNoteAction { root_note: Note }
- **Action**: IncreaseRootNoteAction
- **Action**: DecreaseRootNoteAction

## Benefits of This Architecture

1. **Modularity**: Each layer can be modified without affecting others
2. **Testability**: Layers can be tested in isolation
3. **Scalability**: New features can be added at the appropriate layer
4. **Maintainability**: Clear separation makes code easier to understand
5. **Flexibility**: Different visualizations can use the same model data

## Implementation Notes

- Layer APIs are defined in this document in the Interface Definitions section
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