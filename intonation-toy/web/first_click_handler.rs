/// First Click Handler for Microphone Permission
/// 
/// This module handles the browser requirement that getUserMedia() must be called
/// within a user gesture context. It creates a full-screen overlay that captures
/// the user's first click and uses that gesture to request microphone permission.

use crate::common::dev_log;
use crate::web::styling;

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
    use web_sys::{window, HtmlElement, EventTarget, Element};
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
    
    // Style the overlay using CSS class
    overlay.set_attribute("class", "first-click-overlay").unwrap();
    
    // Apply first click styles to document
    styling::apply_first_click_styles();
    
    // Add simple instructions text with header
    let panel_html = "<div id='permission-panel' class='first-click-panel'>
        <h2 class='first-click-title'>Intonation Toy</h2>
        <div class='first-click-description'>Click anywhere to start<br>
        <small style='opacity: 0.7;'>(Microphone permission will be requested)</small>
        </div>
    </div>";
    
    overlay.set_inner_html(panel_html);
    
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
                    // Remove permission-required class to re-enable sidebar controls
                    if let Some(current_class) = body.get_attribute("class") {
                        let new_class = current_class.replace("permission-required", "").trim().to_string();
                        body.set_attribute("class", &new_class).ok();
                    }
                    
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
                            
                            // Remove permission-required class since permission dialog is closed
                            if let Some(window) = web_sys::window() {
                                if let Some(document) = window.document() {
                                    if let Some(body) = document.body() {
                                        if let Some(current_class) = body.get_attribute("class") {
                                            let new_class = current_class.replace("permission-required", "").trim().to_string();
                                            body.set_attribute("class", &new_class).ok();
                                        }
                                    }
                                }
                            }
                            
                            // Display error after a short delay to avoid removal conflicts
                            let timeout_closure = Closure::wrap(Box::new(move || {
                                crate::web::error_message_box::show_error(&crate::shared_types::Error::MicrophonePermissionDenied);
                            }) as Box<dyn FnMut()>);
                            
                            if let Some(window) = web_sys::window() {
                                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                    timeout_closure.as_ref().unchecked_ref(),
                                    100 // 100ms delay
                                );
                                timeout_closure.forget();
                            }
                        }
                    }
                });
            } else {
                dev_log!("✗ Failed to call getUserMedia");
                
                // Remove permission-required class since we're showing an error
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        if let Some(body) = document.body() {
                            if let Some(current_class) = body.get_attribute("class") {
                                let new_class = current_class.replace("permission-required", "").trim().to_string();
                                body.set_attribute("class", &new_class).ok();
                            }
                        }
                    }
                }
                
                crate::web::error_message_box::show_error(&crate::shared_types::Error::BrowserError);
            }
        } else {
            dev_log!("✗ MediaDevices API not available");
            
            // Remove permission-required class since we're showing an error
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(body) = document.body() {
                        if let Some(current_class) = body.get_attribute("class") {
                            let new_class = current_class.replace("permission-required", "").trim().to_string();
                            body.set_attribute("class", &new_class).ok();
                        }
                    }
                }
            }
            
            crate::web::error_message_box::show_error_with_params(&crate::shared_types::Error::BrowserApiNotSupported, &["required audio features"]);
        }
    }) as Box<dyn FnMut(_)>);
    
    // Append overlay to body and add click listener to entire overlay
    if let Some(body) = document.body() {
        // Add permission-required class to body to disable sidebar controls
        let current_class = body.get_attribute("class").unwrap_or_default();
        let new_class = if current_class.is_empty() {
            "permission-required".to_string()
        } else {
            format!("{} permission-required", current_class)
        };
        body.set_attribute("class", &new_class).ok();
        
        body.append_child(&overlay).unwrap();
        dev_log!("✓ First click handler overlay added");
        
        // Add click listener to the entire overlay
        let target: &EventTarget = overlay.as_ref();
        target.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).unwrap();
        
        // Keep the closure alive
        closure.forget();
        
        dev_log!("✓ Click handler attached to entire overlay");
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