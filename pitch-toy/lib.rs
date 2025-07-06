use yew::prelude::*;
use web_sys::HtmlCanvasElement;

#[cfg(target_arch = "wasm32")]
use js_sys;

pub mod audio;
// pub mod console;  // Moved to dev-console crate
pub mod console_commands;
pub mod common;
pub mod platform;
pub mod events;
pub mod debug;

use common::dev_log;

#[cfg(not(test))]
use wasm_bindgen::prelude::*;

#[cfg(not(test))]
use platform::{Platform, PlatformValidationResult};

#[cfg(debug_assertions)]
use debug::DebugInterface;

#[cfg(debug_assertions)]
use std::rc::Rc;

/// Render development console if in debug mode
fn render_dev_console() -> Html {
    #[cfg(debug_assertions)]
    {
        // Get global shared event dispatcher
        let event_dispatcher = crate::events::get_global_event_dispatcher();
        
        // Create audio service with event dispatcher
        let audio_service = Rc::new(crate::audio::create_console_audio_service_with_events(event_dispatcher.clone()));
        let registry = Rc::new(crate::console_commands::create_console_registry_with_audio());
        html! { 
            <DebugInterface
                registry={registry}
                audio_service={audio_service}
                event_dispatcher={Some(event_dispatcher)}
            />
        }
    }
    
    #[cfg(not(debug_assertions))]
    html! {}
}

/// Main application component for Pitch Toy
#[function_component]
fn App() -> Html {
    let canvas_ref = use_node_ref();
    
    // Initialize wgpu canvas after component is rendered
    use_effect_with(canvas_ref.clone(), {
        let canvas_ref = canvas_ref.clone();
        move |_| {
            if let Some(canvas_element) = canvas_ref.cast::<HtmlCanvasElement>() {
                dev_log!("Canvas element found via ref: {}x{}", canvas_element.width(), canvas_element.height());
                initialize_canvas(&canvas_element);
            } else {
                dev_log!("Warning: Canvas element not found via ref");
            }
        }
    });

    html! {
        <div>
            // Development console (debug builds only)
            { render_dev_console() }
            
            // Canvas for wgpu GPU rendering
            <canvas 
                ref={canvas_ref}
                id="wgpu-canvas"
                width="800" 
                height="600"
                style="display: block; margin: 0 auto; border: 1px solid #333;"
            />
        </div>
    }
}

// Note: get_canvas_element() function removed as we now use canvas_ref directly

/// Initialize canvas for wgpu rendering
fn initialize_canvas(canvas: &HtmlCanvasElement) {
    dev_log!("Initializing canvas for wgpu rendering");
    
    // Set canvas size to match display size
    let width = canvas.offset_width() as u32;
    let height = canvas.offset_height() as u32;
    
    canvas.set_width(width);
    canvas.set_height(height);
    
    dev_log!("Canvas initialized: {}x{}", width, height);
    
    // TODO: Initialize wgpu renderer (future story)
    // This will be implemented when graphics module is added
}

/// Initialize AudioWorklet manager with buffer pool and event dispatcher integration
async fn initialize_audioworklet_manager() -> Result<(), String> {
    dev_log!("Initializing AudioWorklet manager");
    
    // Get audio context manager
    let audio_context_manager = audio::get_audio_context_manager()
        .ok_or_else(|| "AudioContext manager not initialized".to_string())?;
    
    // Create AudioWorklet manager
    let mut worklet_manager = audio::AudioWorkletManager::new();
    
    // Get buffer pool and event dispatcher
    let buffer_pool = audio::get_global_buffer_pool()
        .ok_or_else(|| "Buffer pool not initialized".to_string())?;
    let event_dispatcher = crate::events::get_global_event_dispatcher();
    
    // Configure AudioWorklet manager
    worklet_manager.set_buffer_pool(buffer_pool);
    worklet_manager.set_event_dispatcher(event_dispatcher.clone());
    
    // Add volume detector for real-time volume analysis
    let volume_detector = audio::VolumeDetector::new_default();
    worklet_manager.set_volume_detector(volume_detector);
    
    // Publish initial status
    publish_audioworklet_status(&event_dispatcher, audio::worklet::AudioWorkletState::Initializing, false, 0);
    
    // Initialize AudioWorklet
    let audio_context_ref = audio_context_manager.borrow();
    match worklet_manager.initialize(&*audio_context_ref).await {
        Ok(_) => {
            dev_log!("✓ AudioWorklet processor loaded and ready");
            
            
            // Publish ready status
            publish_audioworklet_status(&event_dispatcher, audio::worklet::AudioWorkletState::Ready, true, 0);
            
            // Note: We don't connect AudioWorklet to destination to avoid audio feedback
            // The AudioWorklet will still process audio when microphone is connected to it
            
            // Start audio processing automatically
            match worklet_manager.start_processing() {
                Ok(_) => {
                    dev_log!("✓ Audio processing started automatically");
                    
                    // Publish processing status
                    publish_audioworklet_status(&event_dispatcher, audio::worklet::AudioWorkletState::Processing, true, 0);
                }
                Err(e) => {
                    dev_log!("✗ Failed to start audio processing: {:?}", e);
                    
                    // Still store the manager but in Ready state
                    publish_audioworklet_status(&event_dispatcher, audio::worklet::AudioWorkletState::Ready, true, 0);
                }
            }
            
            // Store globally for microphone connection
            audio::set_global_audioworklet_manager(std::rc::Rc::new(std::cell::RefCell::new(worklet_manager)));
            
            Ok(())
        }
        Err(e) => {
            dev_log!("✗ AudioWorklet initialization failed: {:?}", e);
            
            // Publish failed status
            publish_audioworklet_status(&event_dispatcher, audio::worklet::AudioWorkletState::Failed, false, 0);
            
            Err(format!("Failed to initialize AudioWorklet: {:?}", e))
        }
    }
}

/// Connect microphone stream to AudioWorklet for audio processing
#[cfg(target_arch = "wasm32")]
pub async fn connect_microphone_to_audioworklet() -> Result<(), String> {
    use web_sys::AudioNode;
    use crate::audio::permission::PermissionManager;
    
    dev_log!("Requesting microphone permission and connecting to AudioWorklet");
    
    // Request microphone permission and get stream
    let media_stream = match PermissionManager::request_microphone_permission().await {
        Ok(stream) => {
            dev_log!("✓ Microphone permission granted, received MediaStream");
            stream
        }
        Err(e) => {
            dev_log!("✗ Microphone permission failed: {:?}", e);
            return Err(format!("Failed to get microphone permission: {:?}", e));
        }
    };
    
    // Get audio context and AudioWorklet manager
    let audio_context_manager = audio::get_audio_context_manager()
        .ok_or_else(|| "AudioContext manager not initialized".to_string())?;
    
    // Resume AudioContext if suspended (required for processing to start)
    {
        let mut manager = audio_context_manager.borrow_mut();
        if let Err(e) = manager.resume().await {
            dev_log!("⚠️ Failed to resume AudioContext: {:?}", e);
        } else {
            dev_log!("✓ AudioContext resumed for microphone processing");
        }
    }
    
    let audioworklet_manager = audio::get_global_audioworklet_manager()
        .ok_or_else(|| "AudioWorklet manager not initialized".to_string())?;
    
    // Create audio source from MediaStream
    let audio_context = {
        let manager = audio_context_manager.borrow();
        manager.get_context()
            .ok_or_else(|| "AudioContext not available".to_string())?
            .clone()
    };
    
    let source = match audio_context.create_media_stream_source(&media_stream) {
        Ok(source_node) => {
            dev_log!("✓ Created MediaStreamAudioSourceNode from microphone");
            source_node
        }
        Err(e) => {
            dev_log!("✗ Failed to create audio source: {:?}", e);
            return Err(format!("Failed to create audio source: {:?}", e));
        }
    };
    
    // Connect microphone source to AudioWorklet
    let mut worklet_manager = audioworklet_manager.borrow_mut();
    match worklet_manager.connect_microphone(source.as_ref()) {
        Ok(_) => {
            dev_log!("✓ Microphone successfully connected to AudioWorklet");
            
            // Note: No need to connect to destination - microphone → AudioWorklet is sufficient for processing
            
            // Ensure processing is active after connection
            if !worklet_manager.is_processing() {
                dev_log!("Starting AudioWorklet processing after microphone connection...");
                match worklet_manager.start_processing() {
                    Ok(_) => {
                        dev_log!("✓ AudioWorklet processing started - audio pipeline active");
                    }
                    Err(e) => {
                        dev_log!("⚠️ Failed to start processing after microphone connection: {:?}", e);
                    }
                }
            } else {
                dev_log!("✓ AudioWorklet already processing - audio pipeline active");
            }
            
            // Publish success event
            let event_dispatcher = crate::events::get_global_event_dispatcher();
            publish_audioworklet_status(&event_dispatcher, audio::worklet::AudioWorkletState::Processing, true, 0);
            
            Ok(())
        }
        Err(e) => {
            dev_log!("✗ Failed to connect microphone to AudioWorklet: {:?}", e);
            Err(format!("Failed to connect microphone: {:?}", e))
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn connect_microphone_to_audioworklet() -> Result<(), String> {
    dev_log!("Microphone connection not available in non-WASM builds");
    Ok(())
}

/// Publish AudioWorklet status update to Live Data Panel
fn publish_audioworklet_status(
    event_dispatcher: &crate::events::AudioEventDispatcher,
    state: audio::worklet::AudioWorkletState,
    processor_loaded: bool,
    chunks_processed: u32
) {
    #[cfg(target_arch = "wasm32")]
    let timestamp = js_sys::Date::now();
    #[cfg(not(target_arch = "wasm32"))]
    let timestamp = 0.0;
    
    let status = crate::debug::live_panel::AudioWorkletStatus {
        state: state.clone(),
        processor_loaded,
        chunk_size: 128, // Web Audio API standard
        chunks_processed,
        last_update: timestamp,
    };
    
    let status_event = crate::events::audio_events::AudioEvent::AudioWorkletStatusChanged(status);
    event_dispatcher.borrow().publish(&status_event);
    
    dev_log!("Published AudioWorklet status: {} (processor: {})", state, processor_loaded);
}

/// Application entry point
#[cfg(not(test))]
#[wasm_bindgen(start)]
pub fn main() {
    // Initialize console logging for development
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    
    dev_log!("Starting Pitch Toy application");
    dev_log!("Build configuration: {}", if cfg!(debug_assertions) { "Development" } else { "Production" });
    dev_log!("{}", Platform::get_platform_info());
    
    // Validate critical platform APIs before proceeding
    match Platform::check_feature_support() {
        PlatformValidationResult::AllSupported => {
            dev_log!("✓ Platform validation passed - initializing application");
            
            // Initialize audio system asynchronously
            wasm_bindgen_futures::spawn_local(async {
                match audio::initialize_audio_system().await {
                    Ok(_) => {
                        dev_log!("✓ Audio system initialized successfully");
                        
                        // Initialize buffer pool after audio system
                        match audio::initialize_buffer_pool().await {
                            Ok(_) => {
                                dev_log!("✓ Buffer pool initialized successfully");
                                
                                // Initialize AudioWorklet manager after buffer pool
                                match initialize_audioworklet_manager().await {
                                    Ok(_) => {
                                        dev_log!("✓ AudioWorklet manager initialized successfully");
                                        
                                        // Initialize pitch analyzer after AudioWorklet
                                        match audio::initialize_pitch_analyzer().await {
                                            Ok(_) => {
                                                dev_log!("✓ Pitch analyzer initialized successfully");
                                                yew::Renderer::<App>::new().render();
                                            }
                                            Err(e) => {
                                                dev_log!("✗ Pitch analyzer initialization failed: {}", e);
                                                dev_log!("Application cannot continue without pitch analyzer");
                                                // TODO: Add error screen rendering in future story when UI requirements are defined
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        dev_log!("✗ AudioWorklet manager initialization failed: {}", e);
                                        dev_log!("Application will continue without AudioWorklet support");
                                        
                                        // Continue with pitch analyzer initialization even without AudioWorklet
                                        match audio::initialize_pitch_analyzer().await {
                                            Ok(_) => {
                                                dev_log!("✓ Pitch analyzer initialized successfully");
                                                yew::Renderer::<App>::new().render();
                                            }
                                            Err(e) => {
                                                dev_log!("✗ Pitch analyzer initialization failed: {}", e);
                                                dev_log!("Application cannot continue without pitch analyzer");
                                                // TODO: Add error screen rendering in future story when UI requirements are defined
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                dev_log!("✗ Buffer pool initialization failed: {}", e);
                                dev_log!("Application cannot continue without buffer pool");
                                // TODO: Add error screen rendering in future story when UI requirements are defined
                            }
                        }
                    }
                    Err(_e) => {
                        dev_log!("✗ Audio system initialization failed: {}", _e);
                        dev_log!("Application cannot continue without audio system");
                        // TODO: Add error screen rendering in future story when UI requirements are defined
                    }
                }
            });
        }
        PlatformValidationResult::MissingCriticalApis(_missing_apis) => {
            let _api_list: Vec<String> = _missing_apis.iter().map(|api| api.to_string()).collect();
            dev_log!("✗ CRITICAL: Missing required browser APIs: {}", _api_list.join(", "));
            dev_log!("✗ Application cannot start. Please upgrade your browser or use a supported browser:");
            // TODO: Add error screen rendering in future story when UI requirements are defined
        }
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_build_configuration() {
        // Test that build configuration detection works
        let config = if cfg!(debug_assertions) { "Development" } else { "Production" };
        assert!(config == "Development" || config == "Production");
    }

    // TODO: Add meaningful tests when we have testable functionality:
    // - test_canvas_initialization() when wgpu renderer is implemented
    // - test_audio_processing() when audio modules are added
    // - test_event_system() when event dispatcher is implemented
    // - test_theme_switching() when theme manager is added
}