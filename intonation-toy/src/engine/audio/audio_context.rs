use web_sys::{AudioContext, AudioContextOptions};
use crate::common::dev_log;
use crate::app_config::STANDARD_SAMPLE_RATE;

/// Create AudioContext with standard sample rate
/// 
/// Creates an AudioContext configured with the application's standard sample rate.
/// This is the first step in audio initialization and should be called early in
/// application startup.
/// 
/// # Returns
/// 
/// Returns `Result<AudioContext, String>` where:
/// - On success: AudioContext ready for worklet module loading
/// - On error: String describing what went wrong
/// 
/// # Example
/// 
/// ```rust
/// let context = create_audio_context()?;
/// ```
pub fn create_audio_context() -> Result<AudioContext, String> {
    dev_log!("Creating AudioContext with standard sample rate");
    
    // Create AudioContext with standard sample rate
    let options = AudioContextOptions::new();
    options.set_sample_rate(STANDARD_SAMPLE_RATE as f32);
    
    let audio_context = AudioContext::new_with_context_options(&options)
        .map_err(|e| format!("Failed to create AudioContext: {:?}", e))?;
    
    dev_log!("✓ AudioContext created with sample rate: {} Hz", STANDARD_SAMPLE_RATE);
    
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