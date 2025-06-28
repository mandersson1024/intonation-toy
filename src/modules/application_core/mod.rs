//! # Application Core Module
//!
//! The Application Core module provides central coordination and infrastructure
//! for the pitch-toy application. It manages module lifecycle, dependency injection,
//! configuration, and the event bus system.

pub mod event_bus;
pub mod priority_event_bus;
pub mod typed_event_bus;
pub mod buffer_ref;
pub mod web_audio_compat;
pub mod performance_monitor;
pub mod debug_interface;
pub mod module_registry;
pub mod application_lifecycle;
pub mod dependency_injection;
pub mod configuration_coordinator;
pub mod error_recovery;

// Service Layer Migration - Step 2.1
pub mod error_service;
pub mod modular_error_service;
pub mod error_service_bridge;

#[cfg(test)]
pub mod buffer_benchmark;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod event_bus_integration_tests;

#[cfg(test)]
mod comprehensive_tests;

#[cfg(test)]
mod stress_test_framework;

#[cfg(test)]
mod benchmark_suite;

#[cfg(test)]
mod test_infrastructure;

#[cfg(test)]
mod application_core_test_suite;

pub use event_bus::*;
pub use priority_event_bus::*;
pub use typed_event_bus::*;
pub use buffer_ref::*;
pub use web_audio_compat::*;
pub use performance_monitor::*;
pub use debug_interface::*;
pub use module_registry::*;
pub use application_lifecycle::*;
pub use dependency_injection::*;
pub use configuration_coordinator::*;
pub use error_recovery::*;

// Service Layer Migration re-exports
pub use error_service::{
    ErrorService, ErrorServiceFactory, ErrorEvent, RecoveryEvent,
    ServiceError, ErrorCallback, SubscriptionId
};
pub use modular_error_service::{ModularErrorService, ModularErrorServiceFactory};
pub use error_service_bridge::LegacyErrorBridge;