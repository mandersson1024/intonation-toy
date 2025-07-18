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

// Import action system
use action::{Action, ActionTrigger, ActionListener};


// UI Control Action Types
#[derive(Debug, Clone)]
pub struct TestSignalAction {
    pub enabled: bool,
    pub waveform: audio::TestWaveform,
    pub frequency: f32,
    pub volume: f32,
}

#[derive(Debug, Clone)]
pub struct BackgroundNoiseAction {
    pub enabled: bool,
    pub level: f32,
    pub noise_type: audio::TestWaveform,
}

#[derive(Debug, Clone)]
pub struct OutputToSpeakersAction {
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct MicrophonePermissionAction {
    pub request_permission: bool,
}

/// UI Control Actions - Central action registry for UI controls
pub struct UIControlActions {
    pub test_signal: Action<TestSignalAction>,
    pub background_noise: Action<BackgroundNoiseAction>,
    pub output_to_speakers: Action<OutputToSpeakersAction>,
    pub microphone_permission: Action<MicrophonePermissionAction>,
}

impl UIControlActions {
    pub fn new() -> Self {
        Self {
            test_signal: Action::new(),
            background_noise: Action::new(),
            output_to_speakers: Action::new(),
            microphone_permission: Action::new(),
        }
    }
    
    /// Get trigger handles for UI components
    pub fn get_triggers(&self) -> UIControlTriggers {
        UIControlTriggers {
            test_signal: self.test_signal.trigger(),
            background_noise: self.background_noise.trigger(),
            output_to_speakers: self.output_to_speakers.trigger(),
            microphone_permission: self.microphone_permission.trigger(),
        }
    }
    
    /// Get listener handles for audio module
    pub fn get_listeners(&self) -> UIControlListeners {
        UIControlListeners {
            test_signal: self.test_signal.listener(),
            background_noise: self.background_noise.listener(),
            output_to_speakers: self.output_to_speakers.listener(),
            microphone_permission: self.microphone_permission.listener(),
        }
    }
}

/// UI Control Triggers - For UI components to fire actions
#[derive(Clone)]
pub struct UIControlTriggers {
    pub test_signal: ActionTrigger<TestSignalAction>,
    pub background_noise: ActionTrigger<BackgroundNoiseAction>,
    pub output_to_speakers: ActionTrigger<OutputToSpeakersAction>,
    pub microphone_permission: ActionTrigger<MicrophonePermissionAction>,
}

/// UI Control Listeners - For audio module to respond to actions
pub struct UIControlListeners {
    pub test_signal: ActionListener<TestSignalAction>,
    pub background_noise: ActionListener<BackgroundNoiseAction>,
    pub output_to_speakers: ActionListener<OutputToSpeakersAction>,
    pub microphone_permission: ActionListener<MicrophonePermissionAction>,
}



pub async fn run_three_d(
    live_data: LiveData, 
    microphone_permission_setter: impl observable_data::DataSetter<audio::AudioPermission> + Clone + 'static,
    performance_metrics_setter: impl observable_data::DataSetter<debug::egui::live_data_panel::PerformanceMetrics> + Clone + 'static,
    pitch_data_setter: impl observable_data::DataSetter<Option<debug::egui::live_data_panel::PitchData>> + Clone + 'static,
    ui_control_actions: UIControlActions,
    triggers: UIControlTriggers,
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
        triggers.microphone_permission.clone(),
    );
    
    // Create audio service for LiveDataPanel
    let audio_service = std::rc::Rc::new(audio::create_console_audio_service());
    
    // Pitch data setter is now configured during AudioSystemContext initialization
    // No need to set it again here
    
    // Create LiveDataPanel with action triggers
    let mut live_data_panel = EguiLiveDataPanel::new(audio_service.clone(), live_data, triggers.clone());

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
                // Configure egui to remove window shadows
                gui_context.set_visuals(egui::Visuals {
                    window_shadow: egui::Shadow::NONE,
                    popup_shadow: egui::Shadow::NONE,
                    ..egui::Visuals::default()
                });
                
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
    
    web_sys::console::log_1(&"DEBUG: Starting Pitch Toy application".into());
    dev_log!("Build configuration: {}", if cfg!(debug_assertions) { "Development" } else { "Production" });
    dev_log!("{}", Platform::get_platform_info());
    
    // Validate critical platform APIs before proceeding
    web_sys::console::log_1(&"DEBUG: Checking platform feature support...".into());
    if let PlatformValidationResult::MissingCriticalApis(missing_apis) = Platform::check_feature_support() {
        let api_list: Vec<String> = missing_apis.iter().map(|api| api.to_string()).collect();
        web_sys::console::error_1(&format!("✗ CRITICAL: Missing required browser APIs: {}", api_list.join(", ")).into());
        web_sys::console::error_1(&"✗ Application cannot start. Please upgrade your browser or use a supported browser:".into());
        // TODO: Add error screen rendering in future story when UI requirements are defined
        return;
    }

    web_sys::console::log_1(&"DEBUG: ✓ Platform validation passed - initializing application".into());
    
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
    let buffer_pool_stats_source = DataSource::new(None::<audio::message_protocol::BufferPoolStats>);
    
    let live_data = live_data::LiveData {
        microphone_permission: microphone_permission_source.observer(),
        audio_devices: audio_devices_source.observer(),
        performance_metrics: performance_metrics_source.observer(),
        volume_level: volume_level_source.observer(),
        pitch_data: pitch_data_source.observer(),
        audioworklet_status: audioworklet_status_source.observer(),
        buffer_pool_stats: buffer_pool_stats_source.observer(),
    };
    
    let microphone_permission_setter = microphone_permission_source.setter();
    let audio_devices_setter = audio_devices_source.setter();
    let audioworklet_status_setter = audioworklet_status_source.setter();
    let performance_metrics_setter = performance_metrics_source.setter();
    let pitch_data_setter = pitch_data_source.setter();
    let volume_level_setter = volume_level_source.setter();
    let buffer_pool_stats_setter = buffer_pool_stats_source.setter();
    
    // Initialize audio systems first - but don't block the UI if it fails
    web_sys::console::log_1(&"DEBUG: Starting audio system initialization...".into());
    let audio_context = match initialize_audio_systems_new(
        std::rc::Rc::new(pitch_data_setter.clone()),
        std::rc::Rc::new(volume_level_setter.clone()),
        std::rc::Rc::new(audioworklet_status_setter.clone()),
        std::rc::Rc::new(buffer_pool_stats_setter.clone())
    ).await {
        Ok(context) => {
            dev_log!("✓ Audio system initialization completed successfully");
            web_sys::console::log_1(&"✓ Audio system initialization completed successfully".into());
            Some(context)
        }
        Err(e) => {
            dev_log!("✗ Audio system initialization failed: {}", e);
            dev_log!("Application will continue without audio system");
            web_sys::console::warn_1(&format!("Audio system initialization failed: {}", e).into());
            // Continue with UI rendering - audio features will be disabled
            None
        }
    };
    
    // Create audio service AFTER AudioWorklet initialization
    // Volume level setter is configured in initialize_audio_systems_new, so use the regular service
    let audio_service = std::rc::Rc::new({
        let mut service = crate::audio::create_console_audio_service();
        service.set_audio_devices_setter(audio_devices_setter.clone());
        service
    });
    
    // Trigger initial device enumeration - but don't fail if it doesn't work
    audio_service.refresh_devices();
    
    // Note: Audio console service is needed for volume level updates in both debug and release builds
    
    // Create UI control actions
    let ui_control_actions = UIControlActions::new();
    let listeners = ui_control_actions.get_listeners();
    let triggers = ui_control_actions.get_triggers();
    
    // Setup audio module listeners for UI actions
    match audio_context {
        Some(ref context) => {
            audio::setup_ui_action_listeners_with_context(listeners, microphone_permission_setter.clone(), context.clone());
        }
        None => {
            web_sys::console::error_1(&"Error: Audio system initialization failed - UI action listeners cannot be set up".into());
        }
    }
    
    // Start three-d application directly
    run_three_d(live_data, microphone_permission_setter, performance_metrics_setter, pitch_data_setter, ui_control_actions, triggers).await;
}

/// Initialize all audio systems using AudioSystemContext approach
async fn initialize_audio_systems_new(
    pitch_data_setter: std::rc::Rc<dyn observable_data::DataSetter<Option<debug::egui::live_data_panel::PitchData>>>,
    volume_level_setter: std::rc::Rc<dyn observable_data::DataSetter<Option<debug::egui::live_data_panel::VolumeLevelData>>>,
    audioworklet_status_setter: std::rc::Rc<dyn observable_data::DataSetter<debug::egui::live_data_panel::AudioWorkletStatus>>,
    buffer_pool_stats_setter: std::rc::Rc<dyn observable_data::DataSetter<Option<audio::message_protocol::BufferPoolStats>>>
) -> Result<std::rc::Rc<std::cell::RefCell<audio::AudioSystemContext>>, String> {
    // Convert setters to required types with adapters
    let pitch_setter = std::rc::Rc::new(crate::debug::egui::live_data_panel::PitchDataAdapter::new(pitch_data_setter))
        as std::rc::Rc<dyn observable_data::DataSetter<Option<audio::PitchData>>>;
    
    let volume_setter = std::rc::Rc::new(crate::debug::egui::live_data_panel::VolumeDataAdapter::new(volume_level_setter))
        as std::rc::Rc<dyn observable_data::DataSetter<Option<audio::VolumeLevelData>>>;
    
    let status_setter = std::rc::Rc::new(crate::debug::egui::live_data_panel::AudioWorkletStatusAdapter::new(audioworklet_status_setter))
        as std::rc::Rc<dyn observable_data::DataSetter<audio::AudioWorkletStatus>>;
    
    let buffer_stats_setter = buffer_pool_stats_setter;
    
    web_sys::console::log_1(&"DEBUG: Using AudioSystemContext initialization approach".into());
    
    // Use the initialization function
    let context = audio::initialize_audio_system_with_context(
        volume_setter,
        pitch_setter,
        status_setter,
        buffer_stats_setter,
    ).await?;
    
    web_sys::console::log_1(&"DEBUG: ✓ AudioSystemContext initialized successfully".into());
    
    // Store the context for backward compatibility
    let context_rc = std::rc::Rc::new(std::cell::RefCell::new(context));
    
    // Store individual components globally for backward compatibility
    // Store AudioContextManager globally for backward compatibility
    {
        let context_borrowed = context_rc.borrow();
        let manager_rc = context_borrowed.get_audio_context_manager_rc();
        audio::set_global_audio_context_manager(manager_rc);
    }
    
    dev_log!("✓ AudioSystemContext components available globally for backward compatibility");
    
    
    Ok(context_rc)
}

