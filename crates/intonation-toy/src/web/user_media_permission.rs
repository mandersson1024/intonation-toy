#![cfg(target_arch = "wasm32")]

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

pub async fn ask_for_permission() -> Result<web_sys::MediaStream, String> {
    // Wait for user click on the overlay
    let document = web_sys::window().unwrap().document().unwrap();
    let overlay = document.query_selector(".first-click-overlay").unwrap().unwrap();
    
    // Create a promise that resolves with the MediaStream
    let (promise, resolve) = {
        let mut resolve_holder = None;
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            resolve_holder = Some(resolve);
        });
        (promise, resolve_holder.unwrap())
    };
    
    // Set up click handler that calls getUserMedia INSIDE the callback
    let resolve_clone = resolve.clone();
    let click_closure = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::MouseEvent| {
        // Request media access INSIDE the click callback - critical for security!
        let constraints = web_sys::MediaStreamConstraints::new();
        constraints.set_audio(&true.into());
        constraints.set_video(&false.into());
        
        let navigator = web_sys::window().and_then(|w| w.navigator().media_devices().ok()).unwrap();
        let media_promise = navigator.get_user_media_with_constraints(&constraints).unwrap();
        
        // Resolve our promise with the media promise
        resolve_clone.call1(&wasm_bindgen::JsValue::NULL, &media_promise).unwrap();
    });
    
    overlay.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref()).unwrap();
    
    // Wait for click - this will resolve with the getUserMedia promise
    let media_promise_js = match wasm_bindgen_futures::JsFuture::from(promise).await {
        Ok(result) => result,
        Err(e) => {
            return Err(format!("Microphone access denied or failed: {:?}", e));
        }
    };
    
    // Clean up the event listener
    overlay.remove_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref()).unwrap();
    click_closure.forget();
    
    // Check if it's already a MediaStream or if it's a Promise we need to await
    if media_promise_js.has_type::<web_sys::MediaStream>() {
        // It's already a MediaStream
        Ok(media_promise_js.dyn_into::<web_sys::MediaStream>().unwrap())
    } else {
        // It's a Promise that resolves to a MediaStream
        let media_promise = media_promise_js.dyn_into::<js_sys::Promise>().unwrap();
        let media_stream_js = wasm_bindgen_futures::JsFuture::from(media_promise).await.unwrap();
        Ok(media_stream_js.dyn_into::<web_sys::MediaStream>().unwrap())
    }
}