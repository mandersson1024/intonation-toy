# Application Core Module

The Application Core module provides the foundational infrastructure for building modular applications. It handles module registration, lifecycle management, dependency injection, configuration coordination, and error recovery.

## Quick Start

Here's a simple example of creating an application with the Application Core:

```rust
use pitch_toy::modules::application_core::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the application core
    let mut app = ApplicationCore::new()?;
    
    // Register modules
    app.register_module(Box::new(AudioModule::new()))?;
    app.register_module(Box::new(UIModule::new()))?;
    
    // Load configuration
    app.load_configuration()?;
    
    // Start the application
    app.start()?;
    
    // Application runs...
    
    // Graceful shutdown
    app.shutdown()?;
    Ok(())
}
```

## Creating a Module

Modules implement the `Module` trait and can participate in the application lifecycle:

```rust
use pitch_toy::modules::application_core::*;

pub struct AudioModule {
    id: ModuleId,
    state: ModuleState,
    dependencies: Vec<ModuleId>,
}

impl AudioModule {
    pub fn new() -> Self {
        Self {
            id: ModuleId::new("audio_module"),
            state: ModuleState::Unregistered,
            dependencies: vec![],
        }
    }
}

impl Module for AudioModule {
    fn id(&self) -> &ModuleId {
        &self.id
    }
    
    fn name(&self) -> &str {
        "Audio Processing Module"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn dependencies(&self) -> &[ModuleId] {
        &self.dependencies
    }
    
    fn initialize(&mut self, context: &mut ModuleContext) -> Result<(), ModuleError> {
        // Initialize your module here
        context.log_info("Audio module initializing...");
        
        // Register services with dependency injection
        context.register_service::<dyn AudioService>(Box::new(AudioServiceImpl::new()))?;
        
        self.state = ModuleState::Initialized;
        Ok(())
    }
    
    fn start(&mut self, context: &mut ModuleContext) -> Result<(), ModuleError> {
        // Start your module's background tasks
        context.log_info("Audio module starting...");
        
        self.state = ModuleState::Started;
        Ok(())
    }
    
    fn shutdown(&mut self, context: &mut ModuleContext) -> Result<(), ModuleError> {
        // Clean up resources
        context.log_info("Audio module shutting down...");
        
        self.state = ModuleState::Stopped;
        Ok(())
    }
    
    fn state(&self) -> ModuleState {
        self.state
    }
}
```

## Using Dependency Injection

The Application Core provides a dependency injection container for sharing services between modules:

```rust
// Define a service interface
pub trait AudioService: Send + Sync {
    fn get_current_pitch(&self) -> Option<f32>;
    fn start_recording(&mut self) -> Result<(), AudioError>;
}

// Implement the service
pub struct AudioServiceImpl {
    current_pitch: Option<f32>,
    is_recording: bool,
}

impl AudioServiceImpl {
    pub fn new() -> Self {
        Self {
            current_pitch: None,
            is_recording: false,
        }
    }
}

impl AudioService for AudioServiceImpl {
    fn get_current_pitch(&self) -> Option<f32> {
        self.current_pitch
    }
    
    fn start_recording(&mut self) -> Result<(), AudioError> {
        self.is_recording = true;
        Ok(())
    }
}

// In your module initialization:
impl Module for AudioModule {
    fn initialize(&mut self, context: &mut ModuleContext) -> Result<(), ModuleError> {
        // Register the service
        context.register_singleton::<dyn AudioService>(
            Box::new(AudioServiceImpl::new())
        )?;
        Ok(())
    }
}

// In another module, consume the service:
impl Module for UIModule {
    fn initialize(&mut self, context: &mut ModuleContext) -> Result<(), ModuleError> {
        // Resolve the audio service
        let audio_service = context.resolve::<dyn AudioService>()?;
        
        // Use the service
        if let Some(pitch) = audio_service.get_current_pitch() {
            context.log_info(&format!("Current pitch: {:.2} Hz", pitch));
        }
        
        Ok(())
    }
}
```

## Configuration Management

The Application Core provides centralized configuration with type safety and hot updates:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub input_device: Option<String>,
    pub pitch_algorithm: String,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            buffer_size: 1024,
            input_device: None,
            pitch_algorithm: "yin".to_string(),
        }
    }
}

// In your module:
impl Module for AudioModule {
    fn initialize(&mut self, context: &mut ModuleContext) -> Result<(), ModuleError> {
        // Load module-specific configuration
        let config: AudioConfig = context.get_config()
            .unwrap_or_default();
        
        // Use the configuration
        self.configure_audio_engine(config)?;
        
        // Watch for configuration changes
        context.watch_config_changes(|new_config: AudioConfig| {
            // Handle hot configuration updates
            self.reconfigure_audio_engine(new_config);
        });
        
        Ok(())
    }
}

// Update configuration at runtime:
app.update_config("audio_module", "sample_rate", 48000)?;
```

## Event Bus Integration

Modules can communicate through the event bus system:

```rust
use pitch_toy::modules::application_core::events::*;

#[derive(Debug, Clone)]
pub struct PitchDetectedEvent {
    pub frequency: f32,
    pub confidence: f32,
    pub timestamp: Instant,
}

impl Event for PitchDetectedEvent {
    fn event_type(&self) -> &'static str {
        "pitch_detected"
    }
    
    fn priority(&self) -> EventPriority {
        EventPriority::High
    }
}

// In your module:
impl Module for AudioModule {
    fn initialize(&mut self, context: &mut ModuleContext) -> Result<(), ModuleError> {
        // Subscribe to events
        context.subscribe::<ConfigurationChangedEvent>(|event| {
            // Handle configuration changes
        })?;
        
        Ok(())
    }
    
    fn on_pitch_detected(&self, context: &ModuleContext, pitch: f32, confidence: f32) {
        // Publish events
        let event = PitchDetectedEvent {
            frequency: pitch,
            confidence,
            timestamp: Instant::now(),
        };
        
        context.publish(event);
    }
}
```

## Error Handling and Recovery

The Application Core provides robust error handling with module isolation:

```rust
impl Module for AudioModule {
    fn initialize(&mut self, context: &mut ModuleContext) -> Result<(), ModuleError> {
        // Configure error recovery strategy
        context.set_recovery_strategy(RecoveryStrategy::RestartOnFailure {
            max_attempts: 3,
            backoff_ms: 1000,
        });
        
        // Handle recoverable errors
        match self.initialize_audio_device() {
            Ok(_) => {},
            Err(e) if e.is_recoverable() => {
                context.request_recovery(RecoveryAction::Restart);
                return Err(ModuleError::RecoverableError(e.to_string()));
            }
            Err(e) => {
                return Err(ModuleError::FatalError(e.to_string()));
            }
        }
        
        Ok(())
    }
}
```

## Testing Your Modules

The Application Core provides testing utilities for module development:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pitch_toy::modules::application_core::testing::*;

    #[test]
    fn test_audio_module_initialization() {
        let mut test_app = TestApplicationCore::new();
        let mut audio_module = AudioModule::new();
        
        // Register mock services
        test_app.register_mock_service::<dyn AudioService>(
            Box::new(MockAudioService::new())
        );
        
        // Test module initialization
        let result = test_app.test_module_initialize(&mut audio_module);
        assert!(result.is_ok());
        
        assert_eq!(audio_module.state(), ModuleState::Initialized);
    }
    
    #[test]
    fn test_module_error_recovery() {
        let mut test_app = TestApplicationCore::new();
        let mut failing_module = FailingModule::new();
        
        // Test error recovery
        let recovery_action = test_app.test_module_error_recovery(
            &mut failing_module,
            &MockError::new("Device not found")
        );
        
        assert_eq!(recovery_action, RecoveryAction::Restart);
    }
}
```

## Architecture Overview

The Application Core is built on several key components:

- **Module Registry**: Manages module registration, discovery, and dependency tracking
- **Application Lifecycle**: Handles ordered initialization, startup, and shutdown
- **Dependency Injection**: Provides type-safe service registration and resolution
- **Configuration Coordinator**: Manages hierarchical configuration with hot updates
- **Error Recovery**: Isolates module failures and provides recovery strategies
- **Event Bus Integration**: Connects modules through the priority event system

## Performance Characteristics

The Application Core is designed for high performance:

- Module registration: O(1) lookup time
- Service resolution: <50ms for 1000 resolutions
- Configuration updates: <100ms propagation
- Application startup: <100ms coordination overhead
- Memory usage: <5MB for core infrastructure

## Best Practices

1. **Keep modules focused**: Each module should have a single, well-defined responsibility
2. **Use dependency injection**: Avoid direct module dependencies; use service interfaces
3. **Handle errors gracefully**: Design for failure and provide recovery strategies
4. **Test module isolation**: Use the testing utilities to verify module independence
5. **Design for configuration**: Make your modules configurable from day one
6. **Publish meaningful events**: Use the event bus for inter-module communication
