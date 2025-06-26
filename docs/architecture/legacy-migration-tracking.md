# Legacy Migration Tracking

## Overview
This document tracks the migration of legacy components to the new modular architecture as defined in `modular-restructure-architecture.md`.

## Migration Status

### Legend
- 🔄 **Active**: Currently in use, not yet migrated
- ⚠️ **Deprecated**: Replaced by new module, kept for rollback
- 📁 **Archived**: Historical reference, no longer used

## Components Migration Status

### Audio Processing
| Component | Status | Legacy Location | New Module Location | Notes |
|-----------|--------|----------------|-------------------|-------|
| AudioEngine | 🔄 Active | `src/audio/engine.rs` | `modules/audio_foundations/engine.rs` | Core audio processing |
| PitchDetector | 🔄 Active | `src/audio/pitch_detector.rs` | `modules/audio_foundations/pitch_detector.rs` | Real-time pitch detection |
| PerformanceMonitor | 🔄 Active | `src/audio/performance_monitor.rs` | `modules/performance_observability/` | System monitoring |

### UI Components (Yew)
| Component | Status | Legacy Location | New Module Location | Notes |
|-----------|--------|----------------|-------------------|-------|
| DebugPanel | 🔄 Active | `legacy/active/components/debug_panel.rs` | `modules/development_tools/debug_panel.rs` | Debug interface |
| ErrorDisplay | 🔄 Active | `legacy/active/components/error_display.rs` | `modules/presentation_layer/components/` | Error UI |
| AudioControlPanel | 🔄 Active | `legacy/active/components/audio_control_panel.rs` | `modules/presentation_layer/components/` | Audio controls |
| MicrophonePanel | 🔄 Active | `legacy/active/components/microphone_panel.rs` | `modules/presentation_layer/components/` | Mic permissions |

### Services
| Service | Status | Legacy Location | New Module Location | Notes |
|---------|--------|----------------|-------------------|-------|
| ErrorManager | 🔄 Active | `legacy/active/services/error_manager.rs` | `modules/performance_observability/error_reporter.rs` | Error handling |
| AudioEngine Service | 🔄 Active | `legacy/active/services/audio_engine.rs` | `modules/audio_foundations/` | Service wrapper |

### Hooks
| Hook | Status | Legacy Location | New Module Location | Notes |
|------|--------|----------------|-------------------|-------|
| use_error_handler | 🔄 Active | `legacy/active/hooks/use_error_handler.rs` | `modules/presentation_layer/hooks/` | Error handling |
| use_microphone_permission | 🔄 Active | `legacy/active/hooks/use_microphone_permission.rs` | `modules/presentation_layer/hooks/` | Permissions |

## Migration Phases

### Phase 1: Foundation Setup ✅
- [x] Application Core module structure created
- [x] Legacy directory structure established
- [x] Module registry and event bus implemented

### Phase 2: Audio Module Migration (Current)
- [ ] Migrate AudioEngine to Audio Foundations module
- [ ] Refactor PitchDetector with module interface
- [ ] Implement DeviceManager within module boundary
- [ ] Performance testing for audio regression

### Phase 3: Platform & Data Modules
- [ ] Move browser compatibility to Platform Abstraction
- [ ] Create Data Management module for audio buffers
- [ ] Implement configuration persistence
- [ ] Integrate with audio processing pipeline

### Phase 4: Presentation Layer Restructure
- [ ] Migrate Yew components to Presentation Layer
- [ ] Implement UI coordinator
- [ ] Create theme management system
- [ ] Prepare Graphics Foundations structure

### Phase 5: Development Tools & Final Integration
- [ ] Conditionally compile Development Tools
- [ ] Implement feature flag system
- [ ] Complete Performance & Observability integration
- [ ] Final testing and documentation

## Safe Migration Process

### When Moving Components:
1. **Copy** component to new module location
2. **Update** imports to use new module
3. **Test** thoroughly in development
4. **Deploy** and monitor for issues
5. **After 2-3 successful deployments**: Move old component to `legacy/deprecated/`
6. **After 1 week stable**: Move to `legacy/archived/`

### Rollback Procedure:
1. Revert imports to point to `legacy/active/` or `legacy/deprecated/`
2. Update `lib.rs` re-exports
3. Test and deploy

## Directory Structure

```
src/
├── modules/              # New modular architecture
│   ├── application_core/ # ✅ Implemented
│   ├── audio_foundations/     # 🔄 In Progress
│   ├── graphics_foundations/  # 📋 Planned
│   ├── data_management/       # 📋 Planned
│   ├── platform_abstraction/  # 📋 Planned
│   ├── presentation_layer/    # 📋 Planned
│   ├── development_tools/     # 📋 Planned
│   └── performance_observability/ # 📋 Planned
├── legacy/               # Legacy code management
│   ├── active/          # 🔄 Components still being used
│   │   ├── components/  # Yew UI components
│   │   ├── services/    # Business logic services
│   │   └── hooks/       # Custom Yew hooks
│   ├── deprecated/      # ⚠️ Replaced but kept for safety
│   └── archived/        # 📁 Historical reference
└── ...                  # Other core files
```

## Notes
- All legacy imports are handled through `src/legacy/mod.rs` re-exports
- No breaking changes to external API during migration
- Performance benchmarks maintained throughout transition
- Feature flags available for rollback if needed

Last Updated: 2025-06-26