/// First Click Handler for Microphone Permission
/// 
/// This module handles the browser requirement that getUserMedia() must be called
/// within a user gesture context. It creates a full-screen overlay that captures
/// the user's first click and uses that gesture to request microphone permission.

use crate::common::dev_log;

/// Sets up a first-click handler overlay for WASM targets
/// 
/// This function creates a full-screen invisible overlay that waits for the user's
/// first click. When clicked, it immediately requests microphone permission and
/// connects the resulting MediaStream to the audio engine.
/// 
/// # Arguments
/// * `permission_granted` - Shared state to track permission status
/// * `engine` - Mutable reference to the audio engine for MediaStream connection
#[cfg(target_arch = "wasm32")]
pub fn setup_first_click_handler(
    permission_granted: std::rc::Rc<std::cell::RefCell<bool>>,
    engine: &mut Option<crate::engine::AudioEngine>,
) {
    use web_sys::{window, HtmlElement, EventTarget};
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    
    let window = match window() {
        Some(w) => w,
        None => {
            dev_log!("⚠ No window object available for first click handler");
            return;
        }
    };
    
    let document = match window.document() {
        Some(d) => d,
        None => {
            dev_log!("⚠ No document object available for first click handler");
            return;
        }
    };
    
    // Create full-screen overlay div
    let overlay = match document.create_element("div") {
        Ok(el) => el.dyn_into::<HtmlElement>().unwrap(),
        Err(_) => {
            dev_log!("⚠ Failed to create overlay div");
            return;
        }
    };
    
    // Style the overlay to be full-screen and invisible
    overlay.set_attribute("style", 
        "position: fixed; top: 0; left: 0; width: 100%; height: 100%; \
         background: transparent; z-index: 9999; cursor: pointer;"
    ).unwrap();
    
    // Add instructions text
    overlay.set_inner_html(
        "<div style='position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); \
         color: #fff; font-family: Arial, sans-serif; font-size: 18px; text-align: center; \
         background: rgba(0,0,0,0.8); padding: 20px; border-radius: 10px;'>
         Click anywhere to start<br>
         <small style='opacity: 0.7;'>(Microphone permission will be requested)</small>
         </div>"
    );
    
    // Get audio context from engine for the permission request
    let audio_context = engine.as_ref()
        .and_then(|e| e.get_audio_context());
    
    if audio_context.is_none() {
        dev_log!("⚠ No audio context available for permission request");
        return;
    }
    
    let audio_context = audio_context.unwrap();
    let permission_granted_clone = permission_granted.clone();
    
    // Get engine reference for passing MediaStream
    let engine_ref = engine.as_ref().map(|e| e.get_audio_context()).flatten();
    
    // Create click handler closure
    let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        let permission_granted = permission_granted_clone.clone();
        let audio_context = audio_context.clone();
        let engine_context = engine_ref.clone();
        
        dev_log!("First click detected - requesting microphone permission");
        
        // Remove the overlay immediately
        if let Some(browser_window) = web_sys::window() {
            if let Some(document) = browser_window.document() {
                if let Some(body) = document.body() {
                    // Use querySelectorAll to find all overlays
                    if let Ok(Some(overlay_element)) = document.query_selector("div[style*='z-index: 9999']") {
                        if let Some(parent) = overlay_element.parent_node() {
                            let _ = parent.remove_child(&overlay_element);
                        }
                    }
                }
            }
        }
        
        // Call getUserMedia directly in the synchronous click handler to preserve user activation
        use web_sys::MediaStreamConstraints;
        use wasm_bindgen_futures::JsFuture;
        
        // Get navigator.mediaDevices
        if let Some(navigator) = web_sys::window().and_then(|w| w.navigator().media_devices().ok()) {
            // Create constraints for audio only
            let mut constraints = MediaStreamConstraints::new();
            constraints.set_audio(&true.into());
            constraints.set_video(&false.into());
            
            // Call getUserMedia synchronously (preserves user gesture context)
            if let Ok(promise) = navigator.get_user_media_with_constraints(&constraints) {
                // Now we can spawn the async part that handles the Promise result
                wasm_bindgen_futures::spawn_local(async move {
                    match JsFuture::from(promise).await {
                        Ok(stream_js) => {
                            dev_log!("✓ Microphone permission granted on first click");
                            *permission_granted.borrow_mut() = true;
                            
                            // Convert JsValue to MediaStream and pass to engine
                            if let Ok(media_stream) = stream_js.dyn_into::<web_sys::MediaStream>() {
                                if let Some(engine_context) = engine_context {
                                    match crate::engine::audio::microphone::connect_existing_mediastream_to_audioworklet(media_stream, &engine_context).await {
                                        Ok(_) => {
                                            dev_log!("✓ MediaStream successfully connected to engine");
                                        }
                                        Err(e) => {
                                            dev_log!("✗ Failed to connect MediaStream to engine: {}", e);
                                        }
                                    }
                                } else {
                                    dev_log!("⚠ No engine context available to connect MediaStream");
                                }
                            } else {
                                dev_log!("✗ Failed to convert stream to MediaStream");
                            }
                        }
                        Err(e) => {
                            dev_log!("✗ Microphone permission failed on first click: {:?}", e);
                        }
                    }
                });
            } else {
                dev_log!("✗ Failed to call getUserMedia");
            }
        } else {
            dev_log!("✗ MediaDevices API not available");
        }
    }) as Box<dyn FnMut(_)>);
    
    // Add event listener to overlay
    let target: &EventTarget = overlay.as_ref();
    target.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).unwrap();
    
    // Keep the closure alive
    closure.forget();
    
    // Append overlay to body
    if let Some(body) = document.body() {
        body.append_child(&overlay).unwrap();
        dev_log!("✓ First click handler overlay added");
    } else {
        dev_log!("⚠ No body element available to append overlay");
    }
}

/// No-op implementation for non-WASM targets
#[cfg(not(target_arch = "wasm32"))]
pub fn setup_first_click_handler(
    _permission_granted: std::rc::Rc<std::cell::RefCell<bool>>,
    _engine: &mut Option<crate::engine::AudioEngine>,
) {
    // No-op for non-wasm targets
}