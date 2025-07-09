use yew::prelude::*;
use web_sys::HtmlCanvasElement;
use three_d::*;

/// Custom 2D bounding box for sprite hit testing with screen coordinates
#[derive(Debug, Clone)]
struct SpriteBounds {
    x: f32,      // Screen X coordinate (left edge)
    y: f32,      // Screen Y coordinate (top edge)
    width: f32,  // Width in screen pixels
    height: f32, // Height in screen pixels
    z: f32,      // Z-depth for layering (higher = closer)
}

impl SpriteBounds {
    fn new(x: f32, y: f32, width: f32, height: f32, z: f32) -> Self {
        Self { x, y, width, height, z }
    }
    
    /// Test if screen coordinates are inside this bounding box
    fn contains_point(&self, screen_x: f32, screen_y: f32) -> bool {
        screen_x >= self.x &&
        screen_x <= self.x + self.width &&
        screen_y >= self.y &&
        screen_y <= self.y + self.height
    }
    
    /// Update position (used for animation)
    fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
}

/// Sprite with bundled hit testing capability
struct HitTestableSprite {
    sprite: Gm<Sprites, ColorMaterial>,
    bounds: SpriteBounds,
    id: String,
}

impl HitTestableSprite {
    fn new(sprite: Gm<Sprites, ColorMaterial>, bounds: SpriteBounds, id: String) -> Self {
        Self { sprite, bounds, id }
    }
    
    /// Test if screen coordinates hit this sprite
    fn hit_test(&self, screen_x: f32, screen_y: f32) -> bool {
        self.bounds.contains_point(screen_x, screen_y)
    }
    
    /// Update sprite position and sync bounding box
    fn set_position(&mut self, world_pos: Vec3, screen_x: f32, screen_y: f32) {
        // Update three-d sprite position
        let centers = vec![world_pos];
        self.sprite.geometry.set_centers(&centers);
        
        // Update bounding box to match screen coordinates
        self.bounds.set_position(screen_x - self.bounds.width / 2.0, screen_y - self.bounds.height / 2.0);
    }
}

/// Hit testing manager for multiple sprites
struct SpriteHitTester {
    sprites: Vec<HitTestableSprite>,
}

impl SpriteHitTester {
    fn new() -> Self {
        Self { sprites: Vec::new() }
    }
    
    fn add_sprite(&mut self, sprite: HitTestableSprite) {
        self.sprites.push(sprite);
    }
    
    /// Find the topmost sprite hit by screen coordinates (highest z-value wins)
    fn hit_test(&self, screen_x: f32, screen_y: f32) -> Option<&str> {
        let mut best_hit: Option<(&str, f32)> = None;
        
        for sprite in &self.sprites {
            if sprite.hit_test(screen_x, screen_y) {
                match best_hit {
                    None => best_hit = Some((&sprite.id, sprite.bounds.z)),
                    Some((_, current_z)) => {
                        if sprite.bounds.z > current_z {
                            best_hit = Some((&sprite.id, sprite.bounds.z));
                        }
                    }
                }
            }
        }
        
        best_hit.map(|(id, _)| id)
    }
    
    /// Get mutable reference to sprite by ID
    fn get_sprite_mut(&mut self, id: &str) -> Option<&mut HitTestableSprite> {
        self.sprites.iter_mut().find(|s| s.id == id)
    }
}

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

/// Initialize canvas for three-d graphics rendering
fn initialize_canvas(canvas: &HtmlCanvasElement) {
    dev_log!("Initializing canvas for three-d hello-world proof-of-concept");
    
    // Set canvas size to match display size
    let width = canvas.offset_width() as u32;
    let height = canvas.offset_height() as u32;
    
    canvas.set_width(width);
    canvas.set_height(height);
    
    dev_log!("Canvas initialized: {}x{}", width, height);
    
    // Initialize three-d hello-world graphics
    #[cfg(not(test))]
    wasm_bindgen_futures::spawn_local({
        let canvas = canvas.clone();
        async move {
            match initialize_three_d_hello_world(&canvas).await {
                Ok(_) => {
                    dev_log!("✓ Three-d hello-world initialized successfully");
                }
                Err(e) => {
                    dev_log!("✗ CRITICAL: Three-d hello-world initialization failed: {}", e);
                    dev_log!("✗ Application cannot continue without WebGL support");
                }
            }
        }
    });
}

/// Initialize three-d hello-world graphics with custom sprite hit testing
#[cfg(not(test))]
async fn initialize_three_d_hello_world(canvas: &HtmlCanvasElement) -> Result<(), String> {
    dev_log!("Initializing three-d hello-world proof-of-concept with custom sprite hit testing");
    
    // Create viewport and camera
    let viewport = Viewport::new_at_origo(canvas.width() as u32, canvas.height() as u32);
    let camera = Camera::new_2d(viewport);
    
    dev_log!("✓ Camera created for coordinate conversion");
    
    // Create hit testing manager
    let hit_tester = SpriteHitTester::new();
    
    // For proof-of-concept, create sprites without WebGL context (simplified)
    // We'll use Canvas2D for rendering and custom bounds for hit testing
    let canvas_center_x = canvas.width() as f32 / 2.0;
    let canvas_center_y = canvas.height() as f32 / 2.0;
    
    // Create red sprite (static)
    let red_bounds = SpriteBounds::new(
        canvas_center_x - 50.0,  // x (centered)
        canvas_center_y - 250.0, // y (above center)
        100.0,                   // width
        100.0,                   // height
        1.0                      // z-depth
    );
    
    // Create green sprite (animated) 
    let green_bounds = SpriteBounds::new(
        canvas_center_x - 50.0,  // x (will be updated during animation)
        canvas_center_y - 50.0,  // y (centered)
        100.0,                   // width
        100.0,                   // height
        2.0                      // z-depth (higher than red)
    );
    
    dev_log!("✓ Custom sprite bounds created for hit testing");
    
    // Create 2D context for visual feedback
    let context_2d = canvas.get_context("2d")
        .map_err(|e| format!("Failed to get 2D context: {:?}", e))?
        .ok_or_else(|| "2D context not available".to_string())?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|e| format!("Failed to cast to 2D context: {:?}", e))?;
    
    dev_log!("✓ Canvas 2D context created for visual feedback");
    
    // Start animation loop with custom sprite hit testing
    start_custom_sprite_animation_loop(context_2d, canvas.clone(), camera, hit_tester, red_bounds, green_bounds).await;
    
    Ok(())
}

// Note: For this proof-of-concept, we're using Canvas2D for rendering
// and three-d Camera for coordinate conversion, with custom hit testing

/// Start animation loop with custom sprite hit testing
#[cfg(not(test))]
async fn start_custom_sprite_animation_loop(
    context: web_sys::CanvasRenderingContext2d,
    canvas: HtmlCanvasElement,
    camera: Camera,
    hit_tester: SpriteHitTester,
    red_bounds: SpriteBounds,
    green_bounds: SpriteBounds,
) {
    dev_log!("Starting animation loop with custom sprite hit testing");
    
    let context_ref = std::rc::Rc::new(context);
    let canvas_ref = std::rc::Rc::new(canvas.clone());
    let time_ref = std::rc::Rc::new(std::cell::RefCell::new(0.0));
    let sprite_x_ref = std::rc::Rc::new(std::cell::RefCell::new(0.0));
    let sprite_y_ref = std::rc::Rc::new(std::cell::RefCell::new(0.0));
    
    // Static red square position
    let red_x = (canvas_ref.width() as f64 / 2.0) - 50.0;
    let red_y = (canvas_ref.height() as f64 / 2.0) - 200.0;
    
    // Store camera and hit tester for interaction
    let camera_ref = std::rc::Rc::new(camera);
    let _hit_tester_ref = std::rc::Rc::new(std::cell::RefCell::new(hit_tester));
    let green_bounds_ref = std::rc::Rc::new(std::cell::RefCell::new(green_bounds));
    let red_bounds_ref = std::rc::Rc::new(red_bounds);
    
    // Add click event listener using custom hit testing
    let canvas_for_click = canvas_ref.clone();
    let camera_for_click = camera_ref.clone();
    let green_bounds_for_click = green_bounds_ref.clone();
    let red_bounds_for_click = red_bounds_ref.clone();
    let click_closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        let rect = canvas_for_click.get_bounding_client_rect();
        let mouse_x = event.client_x() as f64 - rect.left();
        let mouse_y = event.client_y() as f64 - rect.top();
        
        // Convert screen coordinates to normalized coordinates for three-d Camera
        let normalized_x = mouse_x / canvas_for_click.width() as f64;
        let normalized_y = 1.0 - (mouse_y / canvas_for_click.height() as f64); // Flip Y for OpenGL
        
        // Get ray direction from camera using three-d (for demonstration)
        let ray_direction = camera_for_click.view_direction_at_uv_coordinates(
            Vec2::new(normalized_x as f32, normalized_y as f32)
        );
        let camera_position = camera_for_click.position();
        
        // Log the three-d ray information
        web_sys::console::log_1(&format!("🎯 Three-d Ray: pos={:?}, dir={:?}", camera_position, ray_direction).into());
        
        // Use custom hit testing with screen coordinates
        let screen_x = mouse_x as f32;
        let screen_y = mouse_y as f32;
        
        // Test green sprite first (higher z-depth)
        let green_bounds = green_bounds_for_click.borrow();
        if green_bounds.contains_point(screen_x, screen_y) {
            web_sys::console::log_1(&format!("🟢 Nice! You caught the moving green sprite! (Custom hit test at {}, {} with z={})", screen_x, screen_y, green_bounds.z).into());
        }
        // Test red sprite (lower z-depth)
        else if red_bounds_for_click.contains_point(screen_x, screen_y) {
            web_sys::console::log_1(&format!("🔴 You clicked the red square! (Custom hit test at {}, {} with z={})", screen_x, screen_y, red_bounds_for_click.z).into());
        }
        else {
            web_sys::console::log_1(&format!("❌ Clicked empty space at ({}, {})", screen_x, screen_y).into());
        }
    }) as Box<dyn FnMut(_)>);
    
    canvas.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref()).unwrap();
    click_closure.forget(); // Keep the closure alive
    
    // Animation loop using requestAnimationFrame with geometry updates
    let render_closure: std::rc::Rc<std::cell::RefCell<Option<wasm_bindgen::closure::Closure<dyn FnMut()>>>> = 
        std::rc::Rc::new(std::cell::RefCell::new(None));
    let render_closure_clone = render_closure.clone();
    let green_bounds_for_render = green_bounds_ref.clone();
    
    *render_closure_clone.borrow_mut() = Some(wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        let time = *time_ref.borrow();
        
        // Clear canvas
        context_ref.clear_rect(0.0, 0.0, canvas_ref.width() as f64, canvas_ref.height() as f64);
        
        // Calculate sprite position
        let x = (canvas_ref.width() as f64 / 2.0) - 50.0 + (time * 50.0_f64).sin() * 100.0;
        let y = (canvas_ref.height() as f64 / 2.0) - 50.0;
        
        // Update sprite position for ray casting hit detection
        *sprite_x_ref.borrow_mut() = x;
        *sprite_y_ref.borrow_mut() = y;
        
        // Update green sprite bounds for hit testing
        {
            let mut green_bounds = green_bounds_for_render.borrow_mut();
            green_bounds.set_position(x as f32 - 50.0, y as f32 - 50.0);
        }
        
        // Draw static red rectangle (above the green one)
        context_ref.set_fill_style(&wasm_bindgen::JsValue::from_str("#CC6666")); // Less saturated red
        context_ref.fill_rect(red_x, red_y, 100.0, 100.0);
        
        // Draw animated green rectangle
        context_ref.set_fill_style(&wasm_bindgen::JsValue::from_str("green"));
        context_ref.fill_rect(x, y, 100.0, 100.0);
        
        // Update time
        *time_ref.borrow_mut() = time + 0.001;
        
        // Request next frame
        let window = web_sys::window().unwrap();
        let render_closure = render_closure.clone();
        let _ = window.request_animation_frame(
            render_closure.borrow().as_ref().unwrap().as_ref().unchecked_ref()
        );
    }) as Box<dyn FnMut()>));
    
    // Start the animation loop
    let window = web_sys::window().unwrap();
    let _ = window.request_animation_frame(
        render_closure_clone.borrow().as_ref().unwrap().as_ref().unchecked_ref()
    );
    
    dev_log!("✓ Animation loop started with custom sprite hit testing");
}

/// Initialize AudioWorklet manager with buffer pool and event dispatcher integration
#[cfg(not(test))]
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
#[cfg(not(test))]
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