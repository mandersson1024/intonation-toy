//! # Developer UI Startup/Shutdown Event Coordination
//!
//! This module provides coordinated startup and shutdown event management for the Developer UI.
//! It ensures proper initialization order, cleanup sequencing, and event coordination across
//! all debug components and systems.

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::{Duration, Instant};
use crate::modules::application_core::priority_event_bus::PriorityEventBus;
use crate::modules::application_core::event_bus::{Event, EventPriority};
use crate::modules::developer_ui::debug_component_registry::{DebugComponentRegistry, DebugComponentEvent, DebugComponentEventType};
use crate::modules::developer_ui::utils::memory_leak_prevention::MemoryLeakPreventionManager;

/// Developer UI lifecycle event types
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub enum DeveloperUILifecycleEventType {
    StartupInitiated,
    ComponentSystemReady,
    EventBusConnected,
    MemoryLeakPreventionEnabled,
    DebugOverlayActivated,
    StartupCompleted,
    ShutdownInitiated,
    ComponentsDeactivated,
    EventSubscriptionsCleanedUp,
    MemoryLeakPreventionDisabled,
    ShutdownCompleted,
    StartupFailed(String),
    ShutdownFailed(String),
}

/// Developer UI lifecycle event
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct DeveloperUILifecycleEvent {
    pub event_type: DeveloperUILifecycleEventType,
    pub timestamp: u64,
    pub duration_ms: Option<u64>,
    pub component_count: usize,
    pub subscription_count: usize,
}

#[cfg(debug_assertions)]
impl Event for DeveloperUILifecycleEvent {
    fn event_type(&self) -> &'static str {
        "DeveloperUILifecycleEvent"
    }

    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    fn priority(&self) -> EventPriority {
        EventPriority::Medium
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Startup/shutdown coordination state
#[cfg(debug_assertions)]
#[derive(Debug, Clone, PartialEq)]
pub enum CoordinatorState {
    Uninitialized,
    StartingUp,
    Running,
    ShuttingDown,
    Stopped,
    Failed(String),
}

/// Developer UI startup/shutdown coordinator
#[cfg(debug_assertions)]
pub struct StartupShutdownCoordinator {
    state: CoordinatorState,
    event_bus: Option<Rc<RefCell<PriorityEventBus>>>,
    component_registry: Option<Rc<RefCell<DebugComponentRegistry>>>,
    memory_leak_manager: Option<Rc<RefCell<MemoryLeakPreventionManager>>>,
    startup_sequence: Vec<StartupStep>,
    shutdown_sequence: Vec<ShutdownStep>,
    startup_timeout: Duration,
    shutdown_timeout: Duration,
    startup_start_time: Option<Instant>,
    shutdown_start_time: Option<Instant>,
    component_startup_order: Vec<String>,
    failed_components: Vec<String>,
}

/// Startup step definition
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
struct StartupStep {
    name: String,
    description: String,
    timeout: Duration,
    completed: bool,
    start_time: Option<Instant>,
    end_time: Option<Instant>,
    error: Option<String>,
}

/// Shutdown step definition
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
struct ShutdownStep {
    name: String,
    description: String,
    timeout: Duration,
    completed: bool,
    start_time: Option<Instant>,
    end_time: Option<Instant>,
    error: Option<String>,
}

#[cfg(debug_assertions)]
impl StartupShutdownCoordinator {
    /// Create a new startup/shutdown coordinator
    pub fn new() -> Self {
        Self {
            state: CoordinatorState::Uninitialized,
            event_bus: None,
            component_registry: None,
            memory_leak_manager: None,
            startup_sequence: Self::create_default_startup_sequence(),
            shutdown_sequence: Self::create_default_shutdown_sequence(),
            startup_timeout: Duration::from_secs(30),
            shutdown_timeout: Duration::from_secs(10),
            startup_start_time: None,
            shutdown_start_time: None,
            component_startup_order: Vec::new(),
            failed_components: Vec::new(),
        }
    }

    /// Set the event bus for coordination
    pub fn set_event_bus(&mut self, event_bus: Rc<RefCell<PriorityEventBus>>) {
        self.event_bus = Some(event_bus);
    }

    /// Set the component registry for coordination
    pub fn set_component_registry(&mut self, registry: Rc<RefCell<DebugComponentRegistry>>) {
        self.component_registry = Some(registry);
    }

    /// Set the memory leak manager for coordination
    pub fn set_memory_leak_manager(&mut self, manager: Rc<RefCell<MemoryLeakPreventionManager>>) {
        self.memory_leak_manager = Some(manager);
    }

    /// Start the Developer UI system with coordinated startup
    pub async fn startup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.state != CoordinatorState::Uninitialized {
            return Err(format!("Cannot start from state: {:?}", self.state).into());
        }

        self.state = CoordinatorState::StartingUp;
        self.startup_start_time = Some(Instant::now());
        self.failed_components.clear();

        self.publish_lifecycle_event(DeveloperUILifecycleEventType::StartupInitiated);

        // Execute startup sequence
        for step in &mut self.startup_sequence {
            step.start_time = Some(Instant::now());
            
            match self.execute_startup_step(step).await {
                Ok(()) => {
                    step.completed = true;
                    step.end_time = Some(Instant::now());
                    web_sys::console::log_1(&format!("Startup step '{}' completed", step.name).into());
                }
                Err(e) => {
                    step.error = Some(e.to_string());
                    step.end_time = Some(Instant::now());
                    let error_msg = format!("Startup step '{}' failed: {}", step.name, e);
                    web_sys::console::error_1(&error_msg.into());
                    
                    self.state = CoordinatorState::Failed(error_msg.clone());
                    self.publish_lifecycle_event(DeveloperUILifecycleEventType::StartupFailed(error_msg));
                    return Err(error_msg.into());
                }
            }
        }

        self.state = CoordinatorState::Running;
        let startup_duration = self.startup_start_time.unwrap().elapsed();
        
        self.publish_lifecycle_event_with_duration(
            DeveloperUILifecycleEventType::StartupCompleted,
            startup_duration,
        );

        web_sys::console::log_1(&format!("Developer UI startup completed in {:?}", startup_duration).into());
        Ok(())
    }

    /// Shutdown the Developer UI system with coordinated cleanup
    pub async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.state != CoordinatorState::Running {
            return Err(format!("Cannot shutdown from state: {:?}", self.state).into());
        }

        self.state = CoordinatorState::ShuttingDown;
        self.shutdown_start_time = Some(Instant::now());

        self.publish_lifecycle_event(DeveloperUILifecycleEventType::ShutdownInitiated);

        // Execute shutdown sequence
        for step in &mut self.shutdown_sequence {
            step.start_time = Some(Instant::now());
            
            match self.execute_shutdown_step(step).await {
                Ok(()) => {
                    step.completed = true;
                    step.end_time = Some(Instant::now());
                    web_sys::console::log_1(&format!("Shutdown step '{}' completed", step.name).into());
                }
                Err(e) => {
                    step.error = Some(e.to_string());
                    step.end_time = Some(Instant::now());
                    let error_msg = format!("Shutdown step '{}' failed: {}", step.name, e);
                    web_sys::console::warn_1(&error_msg.into());
                    
                    // Continue shutdown even if individual steps fail
                }
            }
        }

        self.state = CoordinatorState::Stopped;
        let shutdown_duration = self.shutdown_start_time.unwrap().elapsed();
        
        self.publish_lifecycle_event_with_duration(
            DeveloperUILifecycleEventType::ShutdownCompleted,
            shutdown_duration,
        );

        web_sys::console::log_1(&format!("Developer UI shutdown completed in {:?}", shutdown_duration).into());
        Ok(())
    }

    /// Execute a single startup step
    async fn execute_startup_step(&mut self, step: &StartupStep) -> Result<(), Box<dyn std::error::Error>> {
        match step.name.as_str() {
            "event_bus_connection" => {
                if self.event_bus.is_none() {
                    return Err("Event bus not available".into());
                }
                self.publish_lifecycle_event(DeveloperUILifecycleEventType::EventBusConnected);
            }
            "component_registry_init" => {
                if let Some(registry) = &self.component_registry {
                    registry.borrow_mut().initialize_all_components()?;
                }
                self.publish_lifecycle_event(DeveloperUILifecycleEventType::ComponentSystemReady);
            }
            "memory_leak_prevention_enable" => {
                if let Some(manager) = &self.memory_leak_manager {
                    manager.borrow_mut().set_leak_detection_enabled(true);
                }
                self.publish_lifecycle_event(DeveloperUILifecycleEventType::MemoryLeakPreventionEnabled);
            }
            "debug_overlay_activation" => {
                if let Some(registry) = &self.component_registry {
                    let mut registry = registry.borrow_mut();
                    registry.update_overlay_state(crate::modules::developer_ui::debug_component_registry::OverlayState {
                        visible: true,
                        position: crate::modules::developer_ui::debug_component_registry::OverlayPosition { x: 10.0, y: 10.0 },
                        size: crate::modules::developer_ui::debug_component_registry::OverlaySize { width: 400.0, height: 300.0 },
                        transparency: 0.9,
                    });
                }
                self.publish_lifecycle_event(DeveloperUILifecycleEventType::DebugOverlayActivated);
            }
            _ => {
                return Err(format!("Unknown startup step: {}", step.name).into());
            }
        }
        Ok(())
    }

    /// Execute a single shutdown step
    async fn execute_shutdown_step(&mut self, step: &ShutdownStep) -> Result<(), Box<dyn std::error::Error>> {
        match step.name.as_str() {
            "deactivate_components" => {
                if let Some(registry) = &self.component_registry {
                    let mut registry = registry.borrow_mut();
                    let active_components = registry.get_active_components();
                    for component_name in active_components {
                        let _ = registry.deactivate_component(&component_name);
                    }
                }
                self.publish_lifecycle_event(DeveloperUILifecycleEventType::ComponentsDeactivated);
            }
            "cleanup_event_subscriptions" => {
                if let Some(registry) = &self.component_registry {
                    registry.borrow_mut().cleanup_event_subscriptions();
                }
                if let Some(manager) = &self.memory_leak_manager {
                    manager.borrow_mut().force_cleanup();
                }
                self.publish_lifecycle_event(DeveloperUILifecycleEventType::EventSubscriptionsCleanedUp);
            }
            "disable_memory_leak_prevention" => {
                if let Some(manager) = &self.memory_leak_manager {
                    manager.borrow_mut().set_leak_detection_enabled(false);
                }
                self.publish_lifecycle_event(DeveloperUILifecycleEventType::MemoryLeakPreventionDisabled);
            }
            _ => {
                return Err(format!("Unknown shutdown step: {}", step.name).into());
            }
        }
        Ok(())
    }

    /// Create default startup sequence
    fn create_default_startup_sequence() -> Vec<StartupStep> {
        vec![
            StartupStep {
                name: "event_bus_connection".to_string(),
                description: "Connect to application event bus".to_string(),
                timeout: Duration::from_secs(5),
                completed: false,
                start_time: None,
                end_time: None,
                error: None,
            },
            StartupStep {
                name: "component_registry_init".to_string(),
                description: "Initialize debug component registry".to_string(),
                timeout: Duration::from_secs(10),
                completed: false,
                start_time: None,
                end_time: None,
                error: None,
            },
            StartupStep {
                name: "memory_leak_prevention_enable".to_string(),
                description: "Enable memory leak prevention".to_string(),
                timeout: Duration::from_secs(2),
                completed: false,
                start_time: None,
                end_time: None,
                error: None,
            },
            StartupStep {
                name: "debug_overlay_activation".to_string(),
                description: "Activate debug overlay".to_string(),
                timeout: Duration::from_secs(3),
                completed: false,
                start_time: None,
                end_time: None,
                error: None,
            },
        ]
    }

    /// Create default shutdown sequence
    fn create_default_shutdown_sequence() -> Vec<ShutdownStep> {
        vec![
            ShutdownStep {
                name: "deactivate_components".to_string(),
                description: "Deactivate all debug components".to_string(),
                timeout: Duration::from_secs(3),
                completed: false,
                start_time: None,
                end_time: None,
                error: None,
            },
            ShutdownStep {
                name: "cleanup_event_subscriptions".to_string(),
                description: "Clean up all event subscriptions".to_string(),
                timeout: Duration::from_secs(3),
                completed: false,
                start_time: None,
                end_time: None,
                error: None,
            },
            ShutdownStep {
                name: "disable_memory_leak_prevention".to_string(),
                description: "Disable memory leak prevention".to_string(),
                timeout: Duration::from_secs(2),
                completed: false,
                start_time: None,
                end_time: None,
                error: None,
            },
        ]
    }

    /// Publish a lifecycle event to the event bus
    fn publish_lifecycle_event(&self, event_type: DeveloperUILifecycleEventType) {
        let component_count = if let Some(registry) = &self.component_registry {
            registry.borrow().component_count()
        } else {
            0
        };

        let subscription_count = if let Some(registry) = &self.component_registry {
            registry.borrow().get_subscription_count()
        } else {
            0
        };

        let event = DeveloperUILifecycleEvent {
            event_type,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            duration_ms: None,
            component_count,
            subscription_count,
        };

        if let Some(event_bus) = &self.event_bus {
            // Note: PriorityEventBus would need a publish method
            // This is a placeholder for the actual implementation
            web_sys::console::log_1(&format!("Publishing lifecycle event: {:?}", event).into());
        }
    }

    /// Publish a lifecycle event with duration information
    fn publish_lifecycle_event_with_duration(&self, event_type: DeveloperUILifecycleEventType, duration: Duration) {
        let component_count = if let Some(registry) = &self.component_registry {
            registry.borrow().component_count()
        } else {
            0
        };

        let subscription_count = if let Some(registry) = &self.component_registry {
            registry.borrow().get_subscription_count()
        } else {
            0
        };

        let event = DeveloperUILifecycleEvent {
            event_type,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            duration_ms: Some(duration.as_millis() as u64),
            component_count,
            subscription_count,
        };

        if let Some(event_bus) = &self.event_bus {
            // Note: PriorityEventBus would need a publish method
            // This is a placeholder for the actual implementation
            web_sys::console::log_1(&format!("Publishing lifecycle event: {:?}", event).into());
        }
    }

    /// Get current coordinator state
    pub fn get_state(&self) -> &CoordinatorState {
        &self.state
    }

    /// Check if startup/shutdown is in progress
    pub fn is_transitioning(&self) -> bool {
        matches!(self.state, CoordinatorState::StartingUp | CoordinatorState::ShuttingDown)
    }

    /// Get startup progress (0.0 to 1.0)
    pub fn get_startup_progress(&self) -> f32 {
        if self.startup_sequence.is_empty() {
            return 1.0;
        }
        
        let completed_steps = self.startup_sequence.iter().filter(|s| s.completed).count();
        completed_steps as f32 / self.startup_sequence.len() as f32
    }

    /// Get shutdown progress (0.0 to 1.0)
    pub fn get_shutdown_progress(&self) -> f32 {
        if self.shutdown_sequence.is_empty() {
            return 1.0;
        }
        
        let completed_steps = self.shutdown_sequence.iter().filter(|s| s.completed).count();
        completed_steps as f32 / self.shutdown_sequence.len() as f32
    }

    /// Get detailed startup status
    pub fn get_startup_status(&self) -> StartupStatus {
        StartupStatus {
            state: self.state.clone(),
            progress: self.get_startup_progress(),
            total_steps: self.startup_sequence.len(),
            completed_steps: self.startup_sequence.iter().filter(|s| s.completed).count(),
            failed_steps: self.startup_sequence.iter().filter(|s| s.error.is_some()).count(),
            elapsed_time: self.startup_start_time.map(|t| t.elapsed()),
            failed_components: self.failed_components.clone(),
        }
    }

    /// Get detailed shutdown status
    pub fn get_shutdown_status(&self) -> ShutdownStatus {
        ShutdownStatus {
            state: self.state.clone(),
            progress: self.get_shutdown_progress(),
            total_steps: self.shutdown_sequence.len(),
            completed_steps: self.shutdown_sequence.iter().filter(|s| s.completed).count(),
            failed_steps: self.shutdown_sequence.iter().filter(|s| s.error.is_some()).count(),
            elapsed_time: self.shutdown_start_time.map(|t| t.elapsed()),
        }
    }
}

/// Startup status information
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct StartupStatus {
    pub state: CoordinatorState,
    pub progress: f32,
    pub total_steps: usize,
    pub completed_steps: usize,
    pub failed_steps: usize,
    pub elapsed_time: Option<Duration>,
    pub failed_components: Vec<String>,
}

/// Shutdown status information
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct ShutdownStatus {
    pub state: CoordinatorState,
    pub progress: f32,
    pub total_steps: usize,
    pub completed_steps: usize,
    pub failed_steps: usize,
    pub elapsed_time: Option<Duration>,
}

#[cfg(test)]
#[cfg(debug_assertions)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_creation() {
        let coordinator = StartupShutdownCoordinator::new();
        assert_eq!(coordinator.state, CoordinatorState::Uninitialized);
        assert!(!coordinator.is_transitioning());
    }

    #[test]
    fn test_startup_progress_calculation() {
        let mut coordinator = StartupShutdownCoordinator::new();
        assert_eq!(coordinator.get_startup_progress(), 0.0);
        
        // Mark first step as completed
        coordinator.startup_sequence[0].completed = true;
        assert!(coordinator.get_startup_progress() > 0.0);
        assert!(coordinator.get_startup_progress() < 1.0);
    }

    #[test]
    fn test_state_transitions() {
        let coordinator = StartupShutdownCoordinator::new();
        
        assert_eq!(coordinator.get_state(), &CoordinatorState::Uninitialized);
        assert!(!coordinator.is_transitioning());
    }
}