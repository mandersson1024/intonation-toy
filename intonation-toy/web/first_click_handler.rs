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
    
    // Style the overlay to be full-screen with semi-transparent background
    overlay.set_attribute("style", 
        "position: fixed; top: 0; left: 0; width: 100%; height: 100%; \
         background: rgba(0, 0, 0, 0.5); z-index: 9999; cursor: default;"
    ).unwrap();
    
    // Add instructions text with visual elements
    overlay.set_inner_html(
        "<style>
            #permission-panel {
                transition: background 0.3s ease, box-shadow 0.3s ease;
                cursor: pointer;
            }
            #permission-panel:hover {
                background: rgba(30, 30, 80, 0.9) !important;
                box-shadow: 0 10px 40px rgba(0, 0, 0, 0.6), 0 0 0 2px rgba(100, 100, 255, 0.5);
            }
        </style>
        <div id='permission-panel' class='permission-panel' style='position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); \
         color: #fff; font-family: Arial, sans-serif; font-size: 18px; text-align: center; \
         background: rgba(0,0,0,0.8); padding: 30px; border-radius: 10px; min-width: 400px; \
         box-shadow: 0 5px 25px rgba(0, 0, 0, 0.4); cursor: pointer;'>
         <div style='display: flex; justify-content: center; align-items: center; gap: 40px; margin-bottom: 25px;'>
             <!-- Left Speaker with Red Cross -->
             <div style='position: relative;'>
                 <svg width='70' height='70' viewBox='0 0 70 70' style='display: block;'>
                     <rect x='10' y='15' width='30' height='40' fill='#e5e5e5' rx='2'/>
                     <path d='M40 25 L50 15 L50 55 L40 45 Z' fill='#e5e5e5'/>
                     <path d='M55 25 Q60 35 55 45' stroke='#e5e5e5' stroke-width='2' fill='none'/>
                     <path d='M60 20 Q68 35 60 50' stroke='#e5e5e5' stroke-width='2' fill='none'/>
                 </svg>
                 <svg width='50' height='50' viewBox='0 0 50 50' style='position: absolute; top: 10px; left: 10px;'>
                     <path d='M15 15 L35 35 M35 15 L15 35' stroke='#ef4444' stroke-width='4' stroke-linecap='round'/>
                 </svg>
             </div>
             
             <!-- Center: Microphone and Headphones with Green Checkmark -->
             <div style='position: relative;'>
                 <div style='display: flex; gap: 20px; align-items: center;'>
                     <!-- Microphone -->
                     <svg width='60' height='80' viewBox='0 0 60 80' style='display: block;'>
                         <rect x='20' y='10' width='20' height='35' fill='#e5e5e5' rx='10'/>
                         <path d='M15 35 Q15 50 30 50 Q45 50 45 35' stroke='#e5e5e5' stroke-width='3' fill='none'/>
                         <line x1='30' y1='50' x2='30' y2='65' stroke='#e5e5e5' stroke-width='3'/>
                         <line x1='20' y1='65' x2='40' y2='65' stroke='#e5e5e5' stroke-width='3'/>
                         <circle cx='30' cy='20' r='2' fill='#999'/>
                         <circle cx='30' cy='28' r='2' fill='#999'/>
                         <circle cx='30' cy='36' r='2' fill='#999'/>
                     </svg>
                     
                     <!-- Headphones -->
                     <svg width='70' height='70' viewBox='0 0 70 70' style='display: block;'>
                         <path d='M15 35 Q15 10 35 10 Q55 10 55 35' stroke='#e5e5e5' stroke-width='4' fill='none'/>
                         <rect x='10' y='30' width='12' height='25' fill='#e5e5e5' rx='6'/>
                         <rect x='48' y='30' width='12' height='25' fill='#e5e5e5' rx='6'/>
                         <circle cx='16' cy='42' r='3' fill='#999'/>
                         <circle cx='54' cy='42' r='3' fill='#999'/>
                     </svg>
                 </div>
                 
                 <!-- Green Checkmark -->
                 <svg width='40' height='40' viewBox='0 0 40 40' style='position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%);'>
                     <path d='M10 20 L17 27 L30 14' stroke='#22c55e' stroke-width='4' stroke-linecap='round' stroke-linejoin='round' fill='none'/>
                 </svg>
             </div>
             
             <!-- Right Speaker with Red Cross -->
             <div style='position: relative;'>
                 <svg width='70' height='70' viewBox='0 0 70 70' style='display: block; transform: scaleX(-1);'>
                     <rect x='10' y='15' width='30' height='40' fill='#e5e5e5' rx='2'/>
                     <path d='M40 25 L50 15 L50 55 L40 45 Z' fill='#e5e5e5'/>
                     <path d='M55 25 Q60 35 55 45' stroke='#e5e5e5' stroke-width='2' fill='none'/>
                     <path d='M60 20 Q68 35 60 50' stroke='#e5e5e5' stroke-width='2' fill='none'/>
                 </svg>
                 <svg width='50' height='50' viewBox='0 0 50 50' style='position: absolute; top: 10px; left: 10px;'>
                     <path d='M15 15 L35 35 M35 15 L15 35' stroke='#ef4444' stroke-width='4' stroke-linecap='round'/>
                 </svg>
             </div>
         </div>
         Click here to start<br>
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
                            // Display error after a short delay to avoid removal conflicts
                            let timeout_closure = Closure::wrap(Box::new(move || {
                                crate::web::error_message_box::show_error_message(
                                    "Microphone Access Required",
                                    "Please allow microphone access to use the pitch detection features. Refresh the page and click 'Allow' when prompted."
                                );
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
                crate::web::error_message_box::show_error_message(
                    "Browser Error",
                    "Failed to access microphone API. Please try refreshing the page or using a different browser."
                );
            }
        } else {
            dev_log!("✗ MediaDevices API not available");
            crate::web::error_message_box::show_error_message(
                "Browser Not Supported",
                "Your browser doesn't support the required audio features. Please try a modern browser like Chrome or Firefox."
            );
        }
    }) as Box<dyn FnMut(_)>);
    
    // Append overlay to body first
    if let Some(body) = document.body() {
        body.append_child(&overlay).unwrap();
        dev_log!("✓ First click handler overlay added");
        
        // Now find the panel and add click listener to it
        if let Ok(Some(panel)) = document.query_selector("#permission-panel") {
            let target: &EventTarget = panel.as_ref();
            target.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).unwrap();
            
            // Keep the closure alive
            closure.forget();
            
            dev_log!("✓ Click handler attached to permission panel");
        } else {
            dev_log!("⚠ Could not find permission panel element");
        }
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