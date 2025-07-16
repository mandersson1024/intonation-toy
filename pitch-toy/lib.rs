use three_d::*;

pub mod audio;
pub mod common;
pub mod platform;
pub mod debug;
pub mod graphics;
pub mod live_data;

use common::dev_log;
use wasm_bindgen::prelude::*;
use egui_dev_console::ConsoleCommandRegistry;
use crate::audio::console_service::ConsoleAudioService;

use platform::{Platform, PlatformValidationResult};
use debug::egui::{EguiMicrophoneButton, EguiLiveDataPanel};

use graphics::SpriteScene;

// Import LiveData type
use live_data::LiveData;



pub async fn run_three_d(
    live_data: LiveData, 
    microphone_permission_setter: impl observable_data::DataSetter<audio::AudioPermission> + Clone + 'static,
    performance_metrics_setter: impl observable_data::DataSetter<debug::egui::live_data_panel::PerformanceMetrics> + Clone + 'static,
    pitch_data_setter: impl observable_data::DataSetter<Option<debug::egui::live_data_panel::PitchData>> + Clone + 'static
) {
    dev_log!("Starting three-d with red sprites");
    
    let window = Window::new(WindowSettings {
        title: "pitch-toy".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    
    let context = window.gl();
    let mut scene = SpriteScene::new(&context, window.viewport());
    let mut gui = three_d::GUI::new(&context);
    
    let mut command_registry = ConsoleCommandRegistry::new();
    crate::platform::commands::register_platform_commands(&mut command_registry);
    crate::audio::register_audio_commands(&mut command_registry);

    let mut dev_console = egui_dev_console::EguiDevConsole::new_with_registry(command_registry);
    let mut microphone_button = EguiMicrophoneButton::new(
        live_data.microphone_permission.clone(),
        microphone_permission_setter,
    );
    
    // Create audio service for LiveDataPanel
    let audio_service = std::rc::Rc::new(audio::create_console_audio_service());
    
    // Set the pitch data setter on the global pitch analyzer (it should be initialized by now)
    audio::set_pitch_data_setter(std::rc::Rc::new(crate::debug::egui::live_data_panel::PitchDataAdapter::new(std::rc::Rc::new(pitch_data_setter))));
    
    // Create LiveDataPanel
    let mut live_data_panel = EguiLiveDataPanel::new(audio_service.clone(), live_data);

    dev_log!("Starting three-d + egui render loop");
    
    // Performance tracking
    let mut frame_count = 0u32;
    let mut last_fps_update = 0.0;
    let mut fps = 0.0;
    
    window.render_loop(move |mut frame_input| {
        // Update FPS counter
        frame_count += 1;
        let current_time = frame_input.accumulated_time as f64;
        
        // Update FPS every second
        if current_time - last_fps_update >= 1000.0 {
            fps = (frame_count as f64) / ((current_time - last_fps_update) / 1000.0);
            frame_count = 0;
            last_fps_update = current_time;
            
            // Update performance metrics
            let metrics = debug::egui::live_data_panel::PerformanceMetrics {
                fps,
                memory_usage: 0.0, // TODO: Implement when Performance.memory is available
                audio_latency: 0.0, // TODO: Get from audio system
                cpu_usage: 0.0, // TODO: Estimate from frame time
            };
            performance_metrics_setter.set(metrics);
        }
        scene.update_viewport(frame_input.viewport);
        scene.render(&mut frame_input.screen());

        gui.update(&mut frame_input.events, frame_input.accumulated_time, frame_input.viewport, frame_input.device_pixel_ratio,
            |gui_context| {
                dev_console.render(gui_context);
                microphone_button.render(gui_context);
                live_data_panel.render(gui_context);
            }
        );
        
        let _ = gui.render();
        FrameOutput::default()
    });
}

/// Application entry point
#[wasm_bindgen(start)]
pub async fn start() {
    // Initialize console logging for development
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    
    dev_log!("Starting Pitch Toy application");
    dev_log!("Build configuration: {}", if cfg!(debug_assertions) { "Development" } else { "Production" });
    dev_log!("{}", Platform::get_platform_info());
    
    // Validate critical platform APIs before proceeding
    if let PlatformValidationResult::MissingCriticalApis(missing_apis) = Platform::check_feature_support() {
        let api_list: Vec<String> = missing_apis.iter().map(|api| api.to_string()).collect();
        dev_log!("✗ CRITICAL: Missing required browser APIs: {}", api_list.join(", "));
        dev_log!("✗ Application cannot start. Please upgrade your browser or use a supported browser:");
        // TODO: Add error screen rendering in future story when UI requirements are defined
        return;
    }

    dev_log!("✓ Platform validation passed - initializing application");
    
    // Create data sources and LiveData directly in start()
    use observable_data::DataSource;
    
    let microphone_permission_source = DataSource::new(audio::AudioPermission::Uninitialized);
    let audio_devices_source = DataSource::new(audio::AudioDevices {
        input_devices: vec![],
        output_devices: vec![],
    });
    let performance_metrics_source = DataSource::new(debug::egui::live_data_panel::PerformanceMetrics::default());
    let volume_level_source = DataSource::new(None::<debug::egui::live_data_panel::VolumeLevelData>);
    let pitch_data_source = DataSource::new(None::<debug::egui::live_data_panel::PitchData>);
    let audioworklet_status_source = DataSource::new(debug::egui::live_data_panel::AudioWorkletStatus::default());
    
    let live_data = live_data::LiveData {
        microphone_permission: microphone_permission_source.observer(),
        audio_devices: audio_devices_source.observer(),
        performance_metrics: performance_metrics_source.observer(),
        volume_level: volume_level_source.observer(),
        pitch_data: pitch_data_source.observer(),
        audioworklet_status: audioworklet_status_source.observer(),
    };
    
    let microphone_permission_setter = microphone_permission_source.setter();
    let audio_devices_setter = audio_devices_source.setter();
    let audioworklet_status_setter = audioworklet_status_source.setter();
    let performance_metrics_setter = performance_metrics_source.setter();
    let pitch_data_setter = pitch_data_source.setter();
    let volume_level_setter = volume_level_source.setter();
    
    // Initialize audio systems first
    if let Err(e) = initialize_audio_systems(
        Some(std::rc::Rc::new(pitch_data_setter.clone())),
        Some(std::rc::Rc::new(volume_level_setter.clone())),
        Some(std::rc::Rc::new(audioworklet_status_setter.clone()))
    ).await {
        dev_log!("✗ Audio system initialization failed: {}", e);
        dev_log!("Application cannot continue without audio system");
        // TODO: Add error screen rendering in future story when UI requirements are defined
        return;
    }
    
    // Create audio service AFTER AudioWorklet initialization
    // Volume level setter is already configured in initialize_audio_systems, so use the regular service
    let audio_service = std::rc::Rc::new({
        let mut service = crate::audio::create_console_audio_service();
        service.set_audio_devices_setter(audio_devices_setter.clone());
        service
    });
    
    // Trigger initial device enumeration
    audio_service.refresh_devices();
    
    // Note: Audio console service is needed for volume level updates in both debug and release builds
    
    // Start three-d application directly
    run_three_d(live_data, microphone_permission_setter, performance_metrics_setter, pitch_data_setter).await;
}

/// Initialize all audio systems in sequence with proper error handling
async fn initialize_audio_systems(
    pitch_data_setter: Option<std::rc::Rc<dyn observable_data::DataSetter<Option<debug::egui::live_data_panel::PitchData>>>>,
    volume_level_setter: Option<std::rc::Rc<dyn observable_data::DataSetter<Option<debug::egui::live_data_panel::VolumeLevelData>>>>,
    audioworklet_status_setter: Option<std::rc::Rc<dyn observable_data::DataSetter<debug::egui::live_data_panel::AudioWorkletStatus>>>
) -> Result<(), String> {
    // Initialize audio system
    audio::initialize_audio_system().await
        .map_err(|e| format!("Audio system initialization failed: {}", e))?;
    dev_log!("✓ Audio system initialized successfully");
    
    // Note: Buffer pool initialization removed - using direct processing with transferable buffers
    
    // Initialize AudioWorklet manager (required)
    audio::worklet::initialize_audioworklet_manager().await
        .map_err(|e| format!("AudioWorklet manager initialization failed: {}", e))?;
    dev_log!("✓ AudioWorklet manager initialized successfully");
    
    // Set the volume level setter if provided
    if let Some(setter) = volume_level_setter {
        audio::set_volume_level_setter(std::rc::Rc::new(crate::debug::egui::live_data_panel::VolumeDataAdapter::new(setter)));
        dev_log!("✓ Volume level setter configured");
    }
    
    // Initialize pitch analyzer (required)
    audio::initialize_pitch_analyzer().await
        .map_err(|e| format!("Pitch analyzer initialization failed: {}", e))?;
    dev_log!("✓ Pitch analyzer initialized successfully");
    
    // Set the pitch data setter if provided
    if let Some(setter) = pitch_data_setter {
        audio::set_pitch_data_setter(std::rc::Rc::new(crate::debug::egui::live_data_panel::PitchDataAdapter::new(setter)));
        dev_log!("✓ Pitch data setter configured");
    }
    
    // Set the AudioWorklet status setter if provided
    if let Some(setter) = audioworklet_status_setter {
        audio::set_audioworklet_status_setter(std::rc::Rc::new(crate::debug::egui::live_data_panel::AudioWorkletStatusAdapter::new(setter)));
        dev_log!("✓ AudioWorklet status setter configured");
    }
    
    Ok(())
}
