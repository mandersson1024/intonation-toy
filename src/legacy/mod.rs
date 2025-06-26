// Legacy module declarations
pub mod components {
    pub use super::active::components::*;
}

pub mod services {
    pub use super::active::services::*;
}

pub mod hooks {
    pub use super::active::hooks::*;
}

pub mod active {
    pub mod components;
    pub mod services;
    pub mod hooks;
}