#![cfg(target_arch = "wasm32")]

use std::fmt;

#[derive(Debug, Clone)]
pub enum AudioError {
    NotSupported(String),
    Generic(String),
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioError::NotSupported(msg) => write!(f, "Not supported: {}", msg),
            AudioError::Generic(msg) => write!(f, "Audio error: {}", msg),
        }
    }
}