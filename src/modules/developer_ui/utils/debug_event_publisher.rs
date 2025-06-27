//! # Debug Event Publisher for Developer UI
//!
//! This module provides utilities for publishing debug events from developer UI components.
//! It enables debug components to send control events to the application event bus.

use std::rc::Rc;
use std::cell::RefCell;
use crate::modules::application_core::event_bus::{Event, EventBus, EventPriority, get_timestamp_ns};
use crate::modules::application_core::priority_event_bus::PriorityEventBus;
use crate::modules::developer_ui::utils::debug_event_performance_monitor::{DebugEventPerformanceMonitor, create_performance_monitor};

/// Publisher for debug events from developer UI components
pub struct DebugEventPublisher {
    event_bus: Option<Rc<RefCell<PriorityEventBus>>>,
    performance_monitor: Option<DebugPublishingMetrics>,
    event_performance_monitor: Rc<RefCell<DebugEventPerformanceMonitor>>,
}

impl DebugEventPublisher {
    /// Create a new debug event publisher
    pub fn new(event_bus: Option<Rc<RefCell<PriorityEventBus>>>) -> Self {
        Self {
            event_bus,
            performance_monitor: Some(DebugPublishingMetrics::new()),
            event_performance_monitor: Rc::new(RefCell::new(create_performance_monitor())),
        }
    }

    /// Publish a debug event to the application event bus
    pub fn publish<T: Event + 'static>(&mut self, event: T) -> Result<(), DebugPublishError> {
        let start_time = std::time::Instant::now();
        let event_type = event.event_type();

        if let Some(event_bus) = &self.event_bus {
            match event_bus.borrow_mut().publish(event) {
                Ok(()) => {
                    self.record_publish_success(start_time);
                    
                    // Record throughput for performance monitoring
                    if let Ok(mut monitor) = self.event_performance_monitor.try_borrow_mut() {
                        monitor.record_event_processed(event_type);
                    }
                    
                    Ok(())
                }
                Err(error) => {
                    self.record_publish_error();
                    Err(DebugPublishError::EventBusError(error.to_string()))
                }
            }
        } else {
            Err(DebugPublishError::NoEventBus)
        }
    }

    /// Publish a debug control event
    pub fn publish_control_event(&mut self, event: DebugControlEvent) -> Result<(), DebugPublishError> {
        self.publish(event)
    }

    /// Get publishing performance metrics
    pub fn get_metrics(&self) -> Option<&DebugPublishingMetrics> {
        self.performance_monitor.as_ref()
    }

    /// Record successful publish
    fn record_publish_success(&mut self, start_time: std::time::Instant) {
        if let Some(metrics) = &mut self.performance_monitor {
            let duration = start_time.elapsed();
            metrics.record_publish(duration);
            
            // Log performance warning if publish takes too long
            if duration.as_millis() > 1 {
                #[cfg(debug_assertions)]
                web_sys::console::warn_1(&format!(
                    "Debug event publish took {}ms (target: <1ms)", 
                    duration.as_millis()
                ).into());
            }
        }
    }

    /// Record publish error
    fn record_publish_error(&mut self) {
        if let Some(metrics) = &mut self.performance_monitor {
            metrics.record_error();
        }
    }
}

/// Debug control events for application interaction
#[derive(Debug, Clone)]
pub enum DebugControlEvent {
    /// Start audio recording
    StartRecording,
    
    /// Stop audio recording
    StopRecording,
    
    /// Select audio device
    SelectDevice { device_id: String },
    
    /// Change audio settings
    ChangeSettings { 
        sample_rate: Option<u32>,
        buffer_size: Option<u32>,
    },
    
    /// Toggle debug overlay visibility
    ToggleOverlay,
    
    /// Reset application state
    ResetState,
    
    /// Request performance report
    RequestPerformanceReport,
    
    /// Custom debug command
    CustomCommand { 
        command: String,
        parameters: std::collections::HashMap<String, String>,
    },
}

impl Event for DebugControlEvent {
    fn event_type(&self) -> &'static str {
        "DebugControlEvent"
    }

    fn timestamp(&self) -> u64 {
        get_timestamp_ns()
    }

    fn priority(&self) -> EventPriority {
        // Debug control events are high priority for responsiveness
        EventPriority::High
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Performance metrics for debug event publishing
#[derive(Debug, Clone)]
pub struct DebugPublishingMetrics {
    /// Total events published
    pub total_published: u64,
    
    /// Total publish errors
    pub total_errors: u64,
    
    /// Average publish time in microseconds
    pub avg_publish_time_us: f64,
    
    /// Maximum publish time in microseconds
    pub max_publish_time_us: u64,
    
    /// Recent publish times for rolling average
    recent_times: Vec<u64>,
}

impl DebugPublishingMetrics {
    /// Create new metrics tracker
    pub fn new() -> Self {
        Self {
            total_published: 0,
            total_errors: 0,
            avg_publish_time_us: 0.0,
            max_publish_time_us: 0,
            recent_times: Vec::with_capacity(100),
        }
    }

    /// Record a successful publish
    pub fn record_publish(&mut self, duration: std::time::Duration) {
        self.total_published += 1;
        
        let duration_us = duration.as_micros() as u64;
        self.max_publish_time_us = self.max_publish_time_us.max(duration_us);
        
        // Maintain rolling window of recent times
        self.recent_times.push(duration_us);
        if self.recent_times.len() > 100 {
            self.recent_times.remove(0);
        }
        
        // Update rolling average
        self.avg_publish_time_us = self.recent_times.iter().sum::<u64>() as f64 / self.recent_times.len() as f64;
    }

    /// Record a publish error
    pub fn record_error(&mut self) {
        self.total_errors += 1;
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_published + self.total_errors == 0 {
            100.0
        } else {
            (self.total_published as f64 / (self.total_published + self.total_errors) as f64) * 100.0
        }
    }

    /// Check if performance meets requirements
    pub fn meets_performance_requirements(&self) -> bool {
        // Requirements: <1ms (1000μs) average publish time
        self.avg_publish_time_us < 1000.0
    }
}

impl Default for DebugPublishingMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during debug event publishing
#[derive(Debug, Clone)]
pub enum DebugPublishError {
    /// No event bus available
    NoEventBus,
    
    /// Event bus error
    EventBusError(String),
    
    /// Event validation failed
    ValidationError(String),
    
    /// Performance timeout
    PerformanceTimeout,
}

impl std::fmt::Display for DebugPublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DebugPublishError::NoEventBus => write!(f, "No event bus available for debug publishing"),
            DebugPublishError::EventBusError(msg) => write!(f, "Event bus error: {}", msg),
            DebugPublishError::ValidationError(msg) => write!(f, "Event validation failed: {}", msg),
            DebugPublishError::PerformanceTimeout => write!(f, "Debug event publish timed out"),
        }
    }
}

impl std::error::Error for DebugPublishError {}

/// Utility function to create a debug event publisher from an event bus
pub fn create_debug_publisher(event_bus: Option<Rc<RefCell<PriorityEventBus>>>) -> DebugEventPublisher {
    DebugEventPublisher::new(event_bus)
}

/// Helper macro for publishing debug events with error handling
#[macro_export]
macro_rules! publish_debug_event {
    ($publisher:expr, $event:expr) => {
        if let Err(error) = $publisher.publish($event) {
            #[cfg(debug_assertions)]
            web_sys::console::error_1(&format!("Failed to publish debug event: {}", error).into());
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_control_event_creation() {
        let event = DebugControlEvent::StartRecording;
        assert_eq!(event.event_type(), "DebugControlEvent");
        assert_eq!(event.priority(), EventPriority::High);
        assert!(event.timestamp() > 0);
    }

    #[test]
    fn test_debug_publishing_metrics() {
        let mut metrics = DebugPublishingMetrics::new();
        
        // Initially should meet requirements
        assert!(metrics.meets_performance_requirements());
        assert_eq!(metrics.success_rate(), 100.0);
        
        // Record some publishes
        metrics.record_publish(instant::Duration::from_micros(500));
        metrics.record_publish(instant::Duration::from_micros(750));
        
        assert_eq!(metrics.total_published, 2);
        assert_eq!(metrics.total_errors, 0);
        assert!(metrics.avg_publish_time_us < 1000.0);
        assert!(metrics.meets_performance_requirements());
        
        // Record an error
        metrics.record_error();
        assert_eq!(metrics.total_errors, 1);
        assert!(metrics.success_rate() < 100.0);
        assert!(metrics.success_rate() > 50.0);
    }

    #[test]
    fn test_debug_event_publisher_no_bus() {
        let mut publisher = DebugEventPublisher::new(None);
        let event = DebugControlEvent::StartRecording;
        
        let result = publisher.publish_control_event(event);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DebugPublishError::NoEventBus));
    }

    #[test]
    fn test_debug_control_event_variants() {
        let events = vec![
            DebugControlEvent::StartRecording,
            DebugControlEvent::StopRecording,
            DebugControlEvent::SelectDevice { device_id: "test".to_string() },
            DebugControlEvent::ChangeSettings { 
                sample_rate: Some(44100),
                buffer_size: Some(1024),
            },
            DebugControlEvent::ToggleOverlay,
            DebugControlEvent::ResetState,
            DebugControlEvent::RequestPerformanceReport,
            DebugControlEvent::CustomCommand {
                command: "test_command".to_string(),
                parameters: std::collections::HashMap::new(),
            },
        ];

        for event in events {
            assert_eq!(event.event_type(), "DebugControlEvent");
            assert_eq!(event.priority(), EventPriority::High);
        }
    }
}