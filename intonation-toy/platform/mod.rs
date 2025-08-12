pub mod traits;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
mod stubs;

#[cfg(target_arch = "wasm32")]
pub use web::WebImpl as PlatformImpl;

#[cfg(not(target_arch = "wasm32"))]
pub use stubs::StubImpl as PlatformImpl;

pub fn get_platform() -> PlatformImpl {
    PlatformImpl::new()
}