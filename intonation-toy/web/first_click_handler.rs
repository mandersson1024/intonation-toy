//! First Click Handler for Microphone Permission
//! 
//! This module handles the browser requirement that getUserMedia() must be called
//! within a user gesture context. It creates a full-screen overlay that captures
//! the user's first click and uses that gesture to request microphone permission.

use crate::common::dev_log;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsCast;


/// Internal function to setup the overlay handler once the element is found
#[cfg(target_arch = "wasm32")]
fn setup_overlay_handler(
    overlay: web_sys::HtmlElement,
    document: web_sys::Document,
    permission_granted: Rc<RefCell<bool>>,
    audio_system_context: Option<Rc<RefCell<crate::engine::audio::AudioSystemContext>>>,
) {
    use web_sys::EventTarget;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    
    let permission_granted_clone = permission_granted.clone();
    let audio_system_context_clone = audio_system_context.clone();
    
    // Create click handler closure
    let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        let permission_granted = permission_granted_clone.clone();
        let audio_system_context = audio_system_context_clone.clone();
        
        dev_log!("First click detected - requesting microphone permission");
        
        // Remove the overlay immediately
        if let Some(browser_window) = web_sys::window() {
            if let Some(document) = browser_window.document() {
                if let Some(body) = document.body() {
                    // Remove permission-required class to re-enable sidebar controls
                    if let Ok(element) = body.dyn_into::<web_sys::Element>() {
                        element.class_list().remove_1("permission-required").ok();
                    }
                    
                    // Hide the first-click overlay
                    if let Ok(Some(overlay_element)) = document.query_selector(".first-click-overlay") {
                        if let Ok(element) = overlay_element.dyn_into::<web_sys::Element>() {
                            if let Err(e) = element.class_list().add_1("first-click-overlay-hidden") {
                                dev_log!("⚠ Failed to hide overlay: {:?}", e);
                            }
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
            let constraints = MediaStreamConstraints::new();
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
                                if let Some(audio_system_context) = audio_system_context {
                                    match crate::engine::audio::microphone::connect_existing_mediastream_to_audioworklet(media_stream, &audio_system_context).await {
                                        Ok(_) => {
                                            dev_log!("✓ MediaStream successfully connected to engine");
                                        }
                                        Err(e) => {
                                            dev_log!("✗ Failed to connect MediaStream to engine: {}", e);
                                        }
                                    }
                                } else {
                                    dev_log!("⚠ No audio system context available to connect MediaStream");
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
                                        if let Ok(element) = body.dyn_into::<web_sys::Element>() {
                                            element.class_list().remove_1("permission-required").ok();
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
                            if let Ok(element) = body.dyn_into::<web_sys::Element>() {
                                element.class_list().remove_1("permission-required").ok();
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
                        if let Ok(element) = body.dyn_into::<web_sys::Element>() {
                            element.class_list().remove_1("permission-required").ok();
                        }
                    }
                }
            }
            
            crate::web::error_message_box::show_error_with_params(&crate::shared_types::Error::BrowserApiNotSupported, &["required audio features"]);
        }
    }) as Box<dyn FnMut(_)>);
    
    // Show overlay and add click listener
    if let Some(body) = document.body() {
        // Add permission-required class to body to disable sidebar controls
        if let Ok(element) = body.dyn_into::<web_sys::Element>() {
            element.class_list().add_1("permission-required").ok();
        }
        
        // Show the existing overlay by removing the hidden class
        if let Ok(element) = overlay.clone().dyn_into::<web_sys::Element>() {
            if let Err(e) = element.class_list().remove_1("first-click-overlay-hidden") {
                dev_log!("⚠ Failed to show first click overlay: {:?}", e);
                return;
            }
        } else {
            dev_log!("⚠ Failed to cast overlay to Element");
            return;
        }
        dev_log!("✓ First click handler overlay shown");
        
        // Remove the preloader overlay now that the first-click overlay is shown
        if let Some(window) = web_sys::window() {
            if let Ok(remove_preloader) = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("removePreloader")) {
                if let Ok(func) = remove_preloader.dyn_into::<js_sys::Function>() {
                    match func.call0(&wasm_bindgen::JsValue::NULL) {
                        Ok(_) => {
                            dev_log!("✓ Preloader removal function called");
                        },
                        Err(e) => {
                            dev_log!("⚠ Failed to call removePreloader: {:?}", e);
                        },
                    }
                } else {
                    dev_log!("⚠ removePreloader is not a function");
                }
            } else {
                dev_log!("⚠ removePreloader function not found in window object");
            }
        }
        
        // Add click listener to the entire overlay
        let target: &EventTarget = overlay.as_ref();
        target.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).unwrap();
        
        // Keep the closure alive
        closure.forget();
        
        dev_log!("✓ Click handler attached to entire overlay");
    } else {
        dev_log!("⚠ No body element available to set permission class");
    }
}

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
    permission_granted: Rc<RefCell<bool>>,
    engine: &mut Option<crate::engine::AudioEngine>,
) {
    use web_sys::{window, HtmlElement};
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
    
    // Get existing first-click overlay, with retry if not found
    let overlay = match document.query_selector(".first-click-overlay") {
        Ok(Some(el)) => match el.dyn_into::<HtmlElement>() {
            Ok(html_el) => html_el,
            Err(_) => {
                dev_log!("⚠ Failed to cast overlay to HtmlElement");
                return;
            }
        },
        Ok(None) => {
            dev_log!("⚠ No first-click overlay found in DOM, scheduling retry");
            
            // Clone the necessary references for the retry closure
            let permission_granted_retry = permission_granted.clone();
            let audio_system_context = engine.as_ref().and_then(|e| e.get_audio_context());
            
            // Use setTimeout to retry after a short delay
            let retry_closure = Closure::once(Box::new(move || {
                dev_log!("Retrying to find first-click overlay after delay");
                
                // Try to find and setup the overlay again
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        if let Ok(Some(el)) = document.query_selector(".first-click-overlay") {
                            if let Ok(overlay) = el.dyn_into::<HtmlElement>() {
                                dev_log!("✓ Found overlay on retry, setting up handler");
                                // Continue with the setup using the found overlay
                                setup_overlay_handler(overlay, document, permission_granted_retry, audio_system_context);
                            } else {
                                dev_log!("⚠ Still couldn't cast overlay to HtmlElement after retry");
                            }
                        } else {
                            dev_log!("⚠ Still no overlay found after retry");
                        }
                    }
                }
            }) as Box<dyn FnOnce()>);
            
            // Schedule retry after 100ms
            if let Err(e) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                retry_closure.as_ref().unchecked_ref(),
                100
            ) {
                dev_log!("⚠ Failed to schedule retry: {:?}", e);
            } else {
                retry_closure.forget();
            }
            
            return;
        },
        Err(_) => {
            dev_log!("⚠ Failed to query for first-click overlay");
            return;
        }
    };
    
    // Get audio system context from engine for the permission request
    let audio_system_context = engine.as_ref()
        .and_then(|e| e.get_audio_context());
    
    if audio_system_context.is_none() {
        dev_log!("⚠ No audio system context available for permission request, scheduling retry");
        
        // Clone the necessary references for the retry closure
        let permission_granted_retry = permission_granted.clone();
        let overlay_clone = overlay.clone();
        let document_clone = document.clone();
        
        // Use setTimeout to retry after a short delay (mirroring overlay retry logic)
        let retry_closure = Closure::once(Box::new(move || {
            dev_log!("Retrying setup after audio context delay");
            
            // Proceed with setup after delay - the overlay handler handles None audio context gracefully.
            // The MediaStream won't be connected to the engine, but the permission dialog will still work.
            // This mirrors the overlay retry pattern where setup continues once the missing element is found.
            setup_overlay_handler(overlay_clone, document_clone, permission_granted_retry, None);
        }) as Box<dyn FnOnce()>);
        
        // Schedule retry after 150ms (middle of the 100-250ms range specified)
        if let Err(e) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            retry_closure.as_ref().unchecked_ref(),
            150
        ) {
            dev_log!("⚠ Failed to schedule audio context retry: {:?}", e);
        } else {
            retry_closure.forget();
        }
        
        return;
    }
    
    // Call the helper function to set up the overlay handler
    setup_overlay_handler(overlay, document, permission_granted, audio_system_context);
}

/// No-op implementation for non-WASM targets
#[cfg(not(target_arch = "wasm32"))]
pub fn setup_first_click_handler(
    _permission_granted: std::rc::Rc<std::cell::RefCell<bool>>,
    _engine: &mut Option<crate::engine::AudioEngine>,
) {
    // No-op for non-wasm targets
}