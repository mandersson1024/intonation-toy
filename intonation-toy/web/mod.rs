#![cfg(target_arch = "wasm32")]

//! Web platform specific functionality
//! This module contains browser-specific code that handles web APIs and DOM interactions
//! 
//! This module is now WASM-only. For cross-platform access to web functionality,
//! use the platform abstraction layer via `crate::platform`.

pub mod error_message_box;
pub mod first_click_handler;
pub mod main_scene_ui;
pub mod performance;
pub mod styling;
pub mod utils;