# Pitch-Toy Module Interfaces Specification

**Version:** 1.0  
**Date:** 2025-06-24  
**Purpose:** Define precise Rust interfaces for the modular architecture

## Overview

This document specifies the exact **Rust traits**, **event schemas**, and **public APIs** for each module in the pitch-toy modular architecture. These interfaces form the contracts that enable loose coupling while maintaining type safety and performance.

## Core Event System

### Event Bus Interface

```rust
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Central event bus for inter-module communication
pub trait EventBus: Send + Sync {
    /// Publish an event to all subscribers
    fn publish<T: Event + 'static>(&self, event: T);
    
    /// Subscribe to events of a specific type
    fn subscribe<T: Event + 'static>(&self, handler: Box<dyn EventHandler<T>>);
    
    /// Unsubscribe from events
    fn unsubscribe<T: Event + 'static>(&self, handler_id: usize);
    
    /// Process queued events (call in main loop)
    fn process_events(&self);
}

/// Base trait for all events
pub trait Event: Send + Sync + Clone {
    fn event_type(&self) -> &'static str;
    fn timestamp(&self) -> u64;
    fn priority(&self) -> EventPriority;
}

/// Event handler trait
pub trait EventHandler<T: Event>: Send + Sync {
    fn handle(&self, event: T);
}

/// Event priority for processing order
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Critical = 0,  // Audio processing
    High = 1,      // User interaction
    Normal = 2,    // UI updates
    Low = 3,       // Debug/logging
}
```

### Core Event Schemas

```rust
/// Audio-related events
#[derive(Debug, Clone)]
pub enum AudioEvent {
    /// Device connection/disconnection
    DeviceStateChanged {
        device_id: String,
        device_type: AudioDeviceType,
        state: DeviceState,
        timestamp: u64,
    },
    
    /// Real-time pitch detection result
    PitchDetected {
        frequency: f64,
        confidence: f32,
        amplitude: f32,
        timestamp: u64,
    },
    
    /// Audio processing started/stopped
    ProcessingStateChanged {
        state: ProcessingState,
        sample_rate: u32,
        buffer_size: usize,
        timestamp: u64,
    },
    
    /// Signal generation event
    SignalGenerated {
        waveform: WaveformType,
        frequency: f64,
        amplitude: f32,
        duration_ms: u32,
        timestamp: u64,
    },
    
    /// Audio processing error
    ProcessingError {
        module: String,
        error_type: AudioErrorType,
        message: String,
        timestamp: u64,
    },
}

/// System-wide events
#[derive(Debug, Clone)]
pub enum SystemEvent {
    /// Module lifecycle events
    ModuleStateChanged {
        module_name: String,
        state: ModuleState,
        timestamp: u64,
    },
    
    /// Configuration changes
    ConfigurationChanged {
        section: String,
        key: String,
        old_value: Option<String>,
        new_value: String,
        timestamp: u64,
    },
    
    /// Performance metrics
    PerformanceMetric {
        module: String,
        metric_type: MetricType,
        value: f64,
        timestamp: u64,
    },
}

/// UI and presentation events
#[derive(Debug, Clone)]
pub enum UIEvent {
    /// User interaction
    UserInteraction {
        component: String,
        action: UserAction,
        data: HashMap<String, String>,
        timestamp: u64,
    },
    
    /// Theme change
    ThemeChanged {
        theme_name: String,
        timestamp: u64,
    },
    
    /// UI component state change
    ComponentStateChanged {
        component_id: String,
        property: String,
        value: String,
        timestamp: u64,
    },
}
```

## Module Interface Definitions

## 1. Application Core Module

```rust
/// Application Core - System orchestration and lifecycle management
pub trait ApplicationCore: Send + Sync {
    /// Initialize the application with configuration
    fn initialize(&mut self, config: ApplicationConfig) -> Result<(), CoreError>;
    
    /// Start all registered modules
    fn start(&mut self) -> Result<(), CoreError>;
    
    /// Shutdown gracefully
    fn shutdown(&mut self) -> Result<(), CoreError>;
    
    /// Get current application state
    fn get_state(&self) -> ApplicationState;
    
    /// Access the event bus
    fn event_bus(&self) -> Arc<dyn EventBus>;
    
    /// Register a module with the core
    fn register_module(&mut self, module: Box<dyn Module>) -> Result<ModuleId, CoreError>;
    
    /// Get reference to a specific module
    fn get_module<T: Module + 'static>(&self) -> Option<&T>;
}

/// Base trait for all modules
pub trait Module: Send + Sync {
    /// Unique module identifier
    fn module_id(&self) -> &str;
    
    /// Module initialization
    fn initialize(&mut self, core: &dyn ApplicationCore) -> Result<(), ModuleError>;
    
    /// Start module operations
    fn start(&mut self) -> Result<(), ModuleError>;
    
    /// Stop module operations
    fn stop(&mut self) -> Result<(), ModuleError>;
    
    /// Get current module state
    fn get_state(&self) -> ModuleState;
    
    /// Handle module-specific configuration
    fn configure(&mut self, config: &HashMap<String, String>) -> Result<(), ModuleError>;
}
```

## 2. Audio Foundations Module

```rust
/// Audio Foundations - Core audio processing and device management
pub trait AudioFoundations: Module {
    /// Get the main audio engine
    fn audio_engine(&self) -> &dyn AudioEngine;
    
    /// Get device manager
    fn device_manager(&self) -> &dyn DeviceManager;
    
    /// Get pitch detector
    fn pitch_detector(&self) -> &dyn PitchDetector;
    
    /// Get signal generator
    fn signal_generator(&self) -> &dyn SignalGenerator;
}

/// Core audio processing engine
pub trait AudioEngine: Send + Sync {
    /// Start audio processing
    fn start(&mut self, config: AudioConfig) -> Result<(), AudioError>;
    
    /// Stop audio processing
    fn stop(&mut self) -> Result<(), AudioError>;
    
    /// Get current audio state
    fn get_state(&self) -> AudioEngineState;
    
    /// Process audio buffer (real-time callback)
    fn process_buffer(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), AudioError>;
    
    /// Get current sample rate
    fn sample_rate(&self) -> u32;
    
    /// Get buffer size
    fn buffer_size(&self) -> usize;
}

/// Audio device management
pub trait DeviceManager: Send + Sync {
    /// List available input devices
    fn list_input_devices(&self) -> Result<Vec<AudioDevice>, DeviceError>;
    
    /// List available output devices  
    fn list_output_devices(&self) -> Result<Vec<AudioDevice>, DeviceError>;
    
    /// Set active input device
    fn set_input_device(&mut self, device_id: &str) -> Result<(), DeviceError>;
    
    /// Set active output device
    fn set_output_device(&mut self, device_id: &str) -> Result<(), DeviceError>;
    
    /// Get current input device
    fn current_input_device(&self) -> Option<&AudioDevice>;
    
    /// Get current output device
    fn current_output_device(&self) -> Option<&AudioDevice>;
    
    /// Request microphone permission (WebAssembly)
    fn request_microphone_permission(&self) -> Result<PermissionState, DeviceError>;
}

/// Real-time pitch detection
pub trait PitchDetector: Send + Sync {
    /// Configure pitch detection parameters
    fn configure(&mut self, config: PitchDetectionConfig) -> Result<(), PitchError>;
    
    /// Detect pitch in audio buffer
    fn detect_pitch(&mut self, buffer: &[f32]) -> Result<PitchResult, PitchError>;
    
    /// Get detection algorithm info
    fn algorithm_info(&self) -> AlgorithmInfo;
    
    /// Set minimum confidence threshold
    fn set_confidence_threshold(&mut self, threshold: f32);
}

/// Signal generation for testing
pub trait SignalGenerator: Send + Sync {
    /// Generate sine wave
    fn generate_sine(&self, frequency: f64, amplitude: f32, duration_ms: u32) -> Vec<f32>;
    
    /// Generate sawtooth wave
    fn generate_sawtooth(&self, frequency: f64, amplitude: f32, duration_ms: u32) -> Vec<f32>;
    
    /// Generate square wave
    fn generate_square(&self, frequency: f64, amplitude: f32, duration_ms: u32) -> Vec<f32>;
    
    /// Generate white noise
    fn generate_noise(&self, amplitude: f32, duration_ms: u32) -> Vec<f32>;
    
    /// Generate frequency sweep
    fn generate_sweep(&self, start_freq: f64, end_freq: f64, amplitude: f32, duration_ms: u32) -> Vec<f32>;
}
```

## 3. Graphics Foundations Module

```rust
/// Graphics Foundations - WebGL setup and rendering utilities
pub trait GraphicsFoundations: Module {
    /// Get WebGL context manager
    fn webgl_context(&self) -> &dyn WebGLContext;
    
    /// Get shader manager
    fn shader_manager(&self) -> &dyn ShaderManager;
    
    /// Get render pipeline
    fn render_pipeline(&self) -> &dyn RenderPipeline;
}

/// WebGL context management
pub trait WebGLContext: Send + Sync {
    /// Initialize WebGL context
    fn initialize(&mut self, canvas_id: &str) -> Result<(), GraphicsError>;
    
    /// Get canvas dimensions
    fn canvas_size(&self) -> (u32, u32);
    
    /// Resize canvas
    fn resize(&mut self, width: u32, height: u32) -> Result<(), GraphicsError>;
    
    /// Clear the screen
    fn clear(&self, color: [f32; 4]);
    
    /// Present rendered frame
    fn present(&self);
    
    /// Check WebGL capabilities
    fn get_capabilities(&self) -> GraphicsCapabilities;
}

/// Shader compilation and management
pub trait ShaderManager: Send + Sync {
    /// Compile vertex shader
    fn compile_vertex_shader(&mut self, source: &str) -> Result<ShaderId, ShaderError>;
    
    /// Compile fragment shader
    fn compile_fragment_shader(&mut self, source: &str) -> Result<ShaderId, ShaderError>;
    
    /// Link shader program
    fn link_program(&mut self, vertex: ShaderId, fragment: ShaderId) -> Result<ProgramId, ShaderError>;
    
    /// Use shader program
    fn use_program(&self, program: ProgramId);
    
    /// Set uniform value
    fn set_uniform<T: UniformValue>(&self, program: ProgramId, name: &str, value: T);
    
    /// Get uniform location
    fn get_uniform_location(&self, program: ProgramId, name: &str) -> Option<UniformLocation>;
}

/// Rendering pipeline coordination
pub trait RenderPipeline: Send + Sync {
    /// Begin frame rendering
    fn begin_frame(&mut self);
    
    /// End frame rendering
    fn end_frame(&mut self);
    
    /// Add renderable object
    fn add_renderable(&mut self, renderable: Box<dyn Renderable>);
    
    /// Remove renderable object
    fn remove_renderable(&mut self, id: RenderableId);
    
    /// Set camera parameters
    fn set_camera(&mut self, camera: CameraParams);
    
    /// Render all objects
    fn render(&mut self) -> Result<(), RenderError>;
}
```

## 4. Data Management Module

```rust
/// Data Management - Audio data flow and configuration persistence
pub trait DataManagement: Module {
    /// Get audio buffer manager
    fn audio_buffer_manager(&self) -> &dyn AudioBufferManager;
    
    /// Get configuration manager
    fn config_manager(&self) -> &dyn ConfigManager;
    
    /// Get data flow coordinator
    fn data_flow(&self) -> &dyn DataFlow;
}

/// Real-time audio buffer management
pub trait AudioBufferManager: Send + Sync {
    /// Create audio buffer
    fn create_buffer(&mut self, size: usize, channels: u8) -> Result<BufferId, BufferError>;
    
    /// Write to buffer
    fn write_buffer(&mut self, id: BufferId, data: &[f32], offset: usize) -> Result<(), BufferError>;
    
    /// Read from buffer
    fn read_buffer(&self, id: BufferId, output: &mut [f32], offset: usize) -> Result<usize, BufferError>;
    
    /// Get buffer info
    fn buffer_info(&self, id: BufferId) -> Option<BufferInfo>;
    
    /// Release buffer
    fn release_buffer(&mut self, id: BufferId) -> Result<(), BufferError>;
    
    /// Get buffer utilization stats
    fn get_stats(&self) -> BufferStats;
}

/// Configuration persistence and management
pub trait ConfigManager: Send + Sync {
    /// Load configuration from storage
    fn load_config(&self) -> Result<Configuration, ConfigError>;
    
    /// Save configuration to storage
    fn save_config(&self, config: &Configuration) -> Result<(), ConfigError>;
    
    /// Get configuration value
    fn get<T: ConfigValue>(&self, section: &str, key: &str) -> Option<T>;
    
    /// Set configuration value
    fn set<T: ConfigValue>(&mut self, section: &str, key: &str, value: T) -> Result<(), ConfigError>;
    
    /// Watch for configuration changes
    fn watch(&mut self, callback: Box<dyn Fn(&str, &str, &str)>);
    
    /// Get default configuration
    fn default_config() -> Configuration;
}

/// Data flow coordination between modules
pub trait DataFlow: Send + Sync {
    /// Register data producer
    fn register_producer(&mut self, producer: Box<dyn DataProducer>) -> ProducerId;
    
    /// Register data consumer
    fn register_consumer(&mut self, consumer: Box<dyn DataConsumer>) -> ConsumerId;
    
    /// Connect producer to consumer
    fn connect(&mut self, producer: ProducerId, consumer: ConsumerId) -> Result<(), DataFlowError>;
    
    /// Disconnect producer from consumer
    fn disconnect(&mut self, producer: ProducerId, consumer: ConsumerId) -> Result<(), DataFlowError>;
    
    /// Get data flow statistics
    fn get_flow_stats(&self) -> DataFlowStats;
}

/// Zero-copy data producer for audio processing
pub trait DataProducer: Send + Sync {
    /// Get reference to current audio buffer (zero-copy)
    fn current_buffer(&self) -> Option<&[f32]>;
    
    /// Get mutable reference for writing (zero-copy)  
    fn buffer_for_writing(&mut self) -> Option<&mut [f32]>;
    
    /// Signal that new data is available (metadata only)
    fn notify_data_ready(&self, metadata: BufferMetadata);
}

/// Zero-copy data consumer for audio processing
pub trait DataConsumer: Send + Sync {
    /// Process audio buffer by reference (zero-copy)
    fn process_buffer(&mut self, buffer: &[f32], metadata: BufferMetadata) -> Result<(), ProcessingError>;
    
    /// Check if consumer is ready for new data
    fn is_ready(&self) -> bool;
}

/// Metadata for buffer operations (no audio data copying)
#[derive(Debug, Clone)]
pub struct BufferMetadata {
    pub sample_rate: u32,
    pub channels: u8,
    pub frame_count: usize,
    pub timestamp: u64,
}
```

## 5. Platform Abstraction Module

```rust
/// Platform Abstraction - Browser compatibility and WebAssembly bridges
pub trait PlatformAbstraction: Module {
    /// Get browser compatibility layer
    fn browser_compat(&self) -> &dyn BrowserCompat;
    
    /// Get device capabilities detector
    fn device_capabilities(&self) -> &dyn DeviceCapabilities;
    
    /// Get WebAssembly bridge utilities
    fn wasm_bridge(&self) -> &dyn WasmBridge;
}

/// Cross-browser compatibility layer
pub trait BrowserCompat: Send + Sync {
    /// Detect browser type and version
    fn detect_browser(&self) -> BrowserInfo;
    
    /// Check Web Audio API support
    fn check_web_audio_support(&self) -> CompatibilityResult;
    
    /// Check WebGL support
    fn check_webgl_support(&self) -> CompatibilityResult;
    
    /// Get polyfills needed
    fn required_polyfills(&self) -> Vec<String>;
    
    /// Apply browser-specific optimizations
    fn apply_optimizations(&self) -> Result<(), CompatError>;
}

/// Device capability detection
pub trait DeviceCapabilities: Send + Sync {
    /// Get audio device capabilities
    fn audio_capabilities(&self) -> AudioCapabilities;
    
    /// Get graphics capabilities
    fn graphics_capabilities(&self) -> GraphicsCapabilities;
    
    /// Get performance capabilities
    fn performance_capabilities(&self) -> PerformanceCapabilities;
    
    /// Check feature support
    fn supports_feature(&self, feature: &str) -> bool;
    
    /// Get optimal settings for current device
    fn optimal_settings(&self) -> OptimalSettings;
}

/// WebAssembly-JavaScript bridge utilities
pub trait WasmBridge: Send + Sync {
    /// Register JavaScript function for calling from Rust
    fn register_js_function(&mut self, name: &str, signature: JsSignature);
    
    /// Call JavaScript function from Rust
    fn call_js_function(&self, name: &str, args: &[JsValue]) -> Result<JsValue, JsError>;
    
    /// Export Rust function to JavaScript
    fn export_rust_function(&mut self, name: &str, func: Box<dyn JsCallable>);
    
    /// Handle JavaScript callbacks
    fn handle_js_callback(&self, callback_id: u32, args: &[JsValue]);
    
    /// Convert between Rust and JavaScript types
    fn convert_to_js<T: ToJsValue>(&self, value: T) -> JsValue;
    fn convert_from_js<T: FromJsValue>(&self, value: JsValue) -> Result<T, ConversionError>;
}
```

## 6. Presentation Layer Module

```rust
/// Presentation Layer - UI coordination between HTML and immersive rendering
pub trait PresentationLayer: Module {
    /// Get UI coordinator
    fn ui_coordinator(&self) -> &dyn UICoordinator;
    
    /// Get theme manager
    fn theme_manager(&self) -> &dyn ThemeManager;
    
    /// Get event handler
    fn event_handler(&self) -> &dyn UIEventHandler;
}

/// Cross-platform UI coordination
pub trait UICoordinator: Send + Sync {
    /// Register HTML component
    fn register_html_component(&mut self, component: Box<dyn HtmlComponent>) -> ComponentId;
    
    /// Register immersive component
    fn register_immersive_component(&mut self, component: Box<dyn ImmersiveComponent>) -> ComponentId;
    
    /// Update component state
    fn update_component(&mut self, id: ComponentId, state: ComponentState) -> Result<(), UIError>;
    
    /// Get component state
    fn get_component_state(&self, id: ComponentId) -> Option<ComponentState>;
    
    /// Layout components
    fn layout(&mut self, viewport: Viewport) -> Result<(), UIError>;
    
    /// Render UI frame
    fn render(&mut self) -> Result<(), UIError>;
}

/// Theme management system
pub trait ThemeManager: Send + Sync {
    /// Load theme
    fn load_theme(&mut self, theme_name: &str) -> Result<(), ThemeError>;
    
    /// Get current theme
    fn current_theme(&self) -> &Theme;
    
    /// Get theme value
    fn get_theme_value(&self, category: &str, property: &str) -> Option<ThemeValue>;
    
    /// Set theme value
    fn set_theme_value(&mut self, category: &str, property: &str, value: ThemeValue);
    
    /// List available themes
    fn available_themes(&self) -> Vec<String>;
    
    /// Create custom theme
    fn create_theme(&mut self, name: &str, base_theme: Option<&str>) -> Result<(), ThemeError>;
}

/// User interaction event handling
pub trait UIEventHandler: Send + Sync {
    /// Handle user input
    fn handle_input(&mut self, input: UserInput) -> Result<(), UIError>;
    
    /// Register input handler
    fn register_handler(&mut self, event_type: InputEventType, handler: Box<dyn InputHandler>);
    
    /// Unregister input handler
    fn unregister_handler(&mut self, event_type: InputEventType, handler_id: usize);
    
    /// Get input state
    fn get_input_state(&self) -> InputState;
}
```

## 7. Development Tools Module (Conditional Compilation)

```rust
#[cfg(feature = "debug")]
/// Development Tools - Debug interface and development features
pub trait DevelopmentTools: Module {
    /// Get debug panel
    fn debug_panel(&self) -> &dyn DebugPanel;
    
    /// Get performance monitor
    fn performance_monitor(&self) -> &dyn DebugPerformanceMonitor;
    
    /// Get feature flags
    fn feature_flags(&self) -> &dyn FeatureFlags;
}

#[cfg(feature = "debug")]
/// Debug interface panel
pub trait DebugPanel: Send + Sync {
    /// Show debug panel
    fn show(&mut self);
    
    /// Hide debug panel
    fn hide(&mut self);
    
    /// Add debug widget
    fn add_widget(&mut self, widget: Box<dyn DebugWidget>) -> WidgetId;
    
    /// Remove debug widget
    fn remove_widget(&mut self, id: WidgetId);
    
    /// Update widget data
    fn update_widget(&mut self, id: WidgetId, data: DebugData);
    
    /// Get debug panel state
    fn is_visible(&self) -> bool;
}

#[cfg(feature = "debug")]
/// Development-time performance monitoring
pub trait DebugPerformanceMonitor: Send + Sync {
    /// Start performance measurement
    fn start_measurement(&mut self, name: &str) -> MeasurementId;
    
    /// End performance measurement
    fn end_measurement(&mut self, id: MeasurementId);
    
    /// Record custom metric
    fn record_metric(&mut self, name: &str, value: f64, unit: &str);
    
    /// Get performance report
    fn get_report(&self) -> PerformanceReport;
    
    /// Reset measurements
    fn reset(&mut self);
}

#[cfg(feature = "debug")]
/// Feature flag system for development
pub trait FeatureFlags: Send + Sync {
    /// Check if feature is enabled
    fn is_enabled(&self, feature: &str) -> bool;
    
    /// Enable feature
    fn enable(&mut self, feature: &str);
    
    /// Disable feature
    fn disable(&mut self, feature: &str);
    
    /// Get all feature states
    fn get_all_flags(&self) -> HashMap<String, bool>;
    
    /// Save flag state
    fn save_state(&self) -> Result<(), FeatureFlagError>;
    
    /// Load flag state
    fn load_state(&mut self) -> Result<(), FeatureFlagError>;
}
```

## 8. Performance & Observability Module

```rust
/// Performance & Observability - System monitoring and error tracking
pub trait PerformanceObservability: Module {
    /// Get performance tracker
    fn performance_tracker(&self) -> &dyn PerformanceTracker;
    
    /// Get error reporter
    fn error_reporter(&self) -> &dyn ErrorReporter;
    
    /// Get metrics collector
    fn metrics_collector(&self) -> &dyn MetricsCollector;
}

/// System-wide performance tracking
pub trait PerformanceTracker: Send + Sync {
    /// Start tracking operation
    fn start_tracking(&mut self, operation: &str) -> TrackingId;
    
    /// End tracking operation
    fn end_tracking(&mut self, id: TrackingId);
    
    /// Record timing
    fn record_timing(&mut self, operation: &str, duration_ms: f64);
    
    /// Record memory usage
    fn record_memory_usage(&mut self, module: &str, bytes: usize);
    
    /// Get performance statistics
    fn get_statistics(&self) -> PerformanceStatistics;
    
    /// Set performance thresholds
    fn set_thresholds(&mut self, thresholds: PerformanceThresholds);
}

/// Centralized error reporting and handling
pub trait ErrorReporter: Send + Sync {
    /// Report error
    fn report_error(&mut self, error: &dyn std::error::Error, context: ErrorContext);
    
    /// Report warning
    fn report_warning(&mut self, message: &str, context: ErrorContext);
    
    /// Get error statistics
    fn get_error_stats(&self) -> ErrorStatistics;
    
    /// Set error handler
    fn set_error_handler(&mut self, handler: Box<dyn ErrorHandler>);
    
    /// Clear error log
    fn clear_errors(&mut self);
}

/// System metrics collection and aggregation
pub trait MetricsCollector: Send + Sync {
    /// Collect metric
    fn collect_metric(&mut self, metric: Metric);
    
    /// Get metric value
    fn get_metric(&self, name: &str) -> Option<MetricValue>;
    
    /// Get all metrics
    fn get_all_metrics(&self) -> HashMap<String, MetricValue>;
    
    /// Set metric threshold
    fn set_threshold(&mut self, metric: &str, threshold: Threshold);
    
    /// Export metrics
    fn export_metrics(&self, format: MetricFormat) -> Result<String, MetricError>;
}
```

## Event Implementation Example

```rust
/// Example implementation showing how events flow through the system
impl Event for AudioEvent {
    fn event_type(&self) -> &'static str {
        match self {
            AudioEvent::DeviceStateChanged { .. } => "audio.device.state_changed",
            AudioEvent::PitchDetected { .. } => "audio.pitch.detected",
            AudioEvent::ProcessingStateChanged { .. } => "audio.processing.state_changed",
            AudioEvent::SignalGenerated { .. } => "audio.signal.generated",
            AudioEvent::ProcessingError { .. } => "audio.processing.error",
        }
    }
    
    fn timestamp(&self) -> u64 {
        match self {
            AudioEvent::DeviceStateChanged { timestamp, .. } => *timestamp,
            AudioEvent::PitchDetected { timestamp, .. } => *timestamp,
            AudioEvent::ProcessingStateChanged { timestamp, .. } => *timestamp,
            AudioEvent::SignalGenerated { timestamp, .. } => *timestamp,
            AudioEvent::ProcessingError { timestamp, .. } => *timestamp,
        }
    }
    
    fn priority(&self) -> EventPriority {
        match self {
            AudioEvent::PitchDetected { .. } => EventPriority::Critical,
            AudioEvent::ProcessingStateChanged { .. } => EventPriority::High,
            AudioEvent::DeviceStateChanged { .. } => EventPriority::Normal,
            AudioEvent::SignalGenerated { .. } => EventPriority::Normal,
            AudioEvent::ProcessingError { .. } => EventPriority::High,
        }
    }
}
```

## Key Design Principles

### Real-Time Performance
- **Critical events** (audio processing) get highest priority
- **Lock-free patterns** where possible for audio callbacks
- **Minimal allocation** in hot paths

### Type Safety
- **Strong typing** for all module interfaces
- **Compile-time guarantees** for module communication
- **Zero-cost abstractions** where possible

### Integration Points Summary

### Critical Real-Time Path:
1. **Audio input** → Audio Foundations → **PitchDetected event** → Graphics Foundations
2. **Low latency** event bus with priority queues
3. **Direct buffer sharing** where performance requires it

### Configuration Flow:
1. **UI changes** → Presentation Layer → **ConfigurationChanged event** → Data Management
2. **Persistent storage** updates via Config Manager

This interface specification provides **type-safe, performant, and maintainable** contracts for your modular architecture while preserving the real-time audio processing requirements.

**Next Steps:** Would you like me to implement any specific module or create the foundational event bus system?

