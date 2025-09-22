#![cfg(target_arch = "wasm32")]

use web_sys::AudioContext;
use crate::common::dev_log;

pub fn create_audio_context() -> Result<AudioContext, String> {
    dev_log!("Creating AudioContext with browser's default sample rate");

    let audio_context = AudioContext::new()
        .map_err(|e| format!("Failed to create AudioContext: {:?}", e))?;

    dev_log!("✓ AudioContext created with sample rate: {} Hz", audio_context.sample_rate());

    Ok(audio_context)
}

/// Load AudioWorklet processor module
/// 
/// Loads the AudioWorklet processor module into the provided AudioContext.
/// This must be called after creating the AudioContext and before creating
/// AudioWorkletNodes.
/// 
/// # Parameters
/// 
/// * `audio_context` - The AudioContext to load the worklet module into
/// 
/// # Returns
/// 
/// Returns `Result<(), String>` where:
/// - On success: Worklet module is loaded and ready for node creation
/// - On error: String describing what went wrong
/// 
/// # Example
/// 
/// ```rust
/// let context = create_audio_context()?;
/// load_worklet_module(&context).await?;
/// ```
pub async fn load_worklet_module(audio_context: &AudioContext) -> Result<(), String> {
    dev_log!("Loading AudioWorklet processor module");
    
    // Load the AudioWorklet processor module
    let worklet = audio_context.audio_worklet()
        .map_err(|e| format!("Failed to get AudioWorklet: {:?}", e))?;
    
    let module_promise = worklet.add_module("./audio-processor.js")
        .map_err(|e| format!("Failed to load AudioWorklet module: {:?}", e))?;
    
    // Wait for module to load
    let module_future = wasm_bindgen_futures::JsFuture::from(module_promise);
    module_future.await
        .map_err(|e| format!("AudioWorklet module loading failed: {:?}", e))?;
    
    dev_log!("✓ AudioWorklet processor module loaded successfully");
    
    Ok(())
}