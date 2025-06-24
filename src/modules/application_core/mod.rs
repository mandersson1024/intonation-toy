//! # Application Core Module
//!
//! The Application Core module provides central coordination and infrastructure
//! for the pitch-toy application. It manages module lifecycle, dependency injection,
//! configuration, and the event bus system.

pub mod event_bus;
pub mod priority_event_bus;
pub mod event_bus_impl;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod event_bus_integration_tests;

pub use event_bus::*;
pub use priority_event_bus::*;
pub use event_bus_impl::*;