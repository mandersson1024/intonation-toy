#![cfg(target_arch = "wasm32")]

#[cfg(debug_assertions)]
pub mod debug_panel;
#[cfg(debug_assertions)]
pub mod debug_data;
#[cfg(debug_assertions)]
pub mod data_types;
