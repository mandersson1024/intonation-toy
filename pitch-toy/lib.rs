use three_d::{self, *};
use three_d::egui::Color32;

// Three-layer architecture modules
pub mod engine;
pub mod model;
pub mod presentation;

// Module interfaces
#[path = "module-interfaces/mod.rs"]
pub mod module_interfaces;

// Supporting modules
pub mod common;
pub mod debug;
pub mod live_data;

use common::dev_log;
use wasm_bindgen::prelude::*;
use egui_dev_console::ConsoleCommandRegistry;

use engine::platform::{Platform, PlatformValidationResult};
use debug::egui::{EguiMicrophoneButton, HybridEguiLiveDataPanel};


// Import action system
use action::{Action, ActionTrigger, ActionListener};


// UI Control Action Types
#[derive(Debug, Clone)]
pub struct TestSignalAction {
    pub enabled: bool,
    pub waveform: engine::audio::TestWaveform,
    pub frequency: f32,
    pub volume: f32,
}

#[derive(Debug, Clone)]
pub struct BackgroundNoiseAction {
    pub enabled: bool,
    pub level: f32,
    pub noise_type: engine::audio::TestWaveform,
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

impl Default for UIControlActions {
    fn default() -> Self {
        Self::new()
    }
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



// Legacy run_three_d function removed - using hybrid architecture only

/// Run three-d with three-layer architecture
pub async fn run_three_d_with_layers(
    mut engine: Option<engine::AudioEngine>,
    mut model: Option<model::DataModel>,
    mut presenter: Option<presentation::Presenter>,
    debug_actions: module_interfaces::debug_actions::DebugActionsInterface,
    performance_metrics_setter: impl observable_data::DataSetter<debug::egui::data_types::PerformanceMetrics> + Clone + 'static,
    performance_metrics_observer: observable_data::DataObserver<debug::egui::data_types::PerformanceMetrics>,
    audio_devices_observer: observable_data::DataObserver<engine::audio::AudioDevices>,
    audioworklet_status_observer: observable_data::DataObserver<debug::egui::data_types::AudioWorkletStatus>,
    buffer_pool_stats_observer: observable_data::DataObserver<Option<engine::audio::message_protocol::BufferPoolStats>>,
    microphone_permission_observer: observable_data::DataObserver<engine::audio::AudioPermission>,
    ui_triggers: UIControlTriggers,
    legacy_engine_to_model: std::rc::Rc<module_interfaces::engine_to_model::EngineToModelInterface>,
) {
    dev_log!("Starting three-d with three-layer architecture");
    
    let window = Window::new(WindowSettings {
        title: "pitch-toy".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    
    let context = window.gl();
    let mut gui = three_d::GUI::new(&context);
    
    let mut command_registry = ConsoleCommandRegistry::new();
    crate::engine::platform::commands::register_platform_commands(&mut command_registry);
    crate::engine::audio::register_audio_commands(&mut command_registry);

    let mut dev_console = egui_dev_console::DevConsole::new_with_registry(command_registry);
    
    // Create microphone button
    let microphone_button = EguiMicrophoneButton::new(
        microphone_permission_observer.clone(),
        ui_triggers.microphone_permission.clone(),
        ui_triggers.output_to_speakers.clone(),
        ui_triggers.test_signal.clone(),
        ui_triggers.background_noise.clone(),
    );
    
    // Create hybrid live data
    let hybrid_live_data = live_data::HybridLiveData::new(
        &legacy_engine_to_model,
        audio_devices_observer,
        performance_metrics_observer,
        audioworklet_status_observer,
        buffer_pool_stats_observer,
    );
    
    // Create hybrid debug panel
    let mut hybrid_live_data_panel = HybridEguiLiveDataPanel::new(
        hybrid_live_data,
        debug_actions,
        microphone_button,
    );

    dev_log!("Starting three-d + egui render loop with three-layer architecture");
    
    // Performance tracking
    let mut frame_count = 0u32;
    let mut last_fps_update = 0.0;
    let mut fps = 0.0;
    
    window.render_loop(move |mut frame_input| {
        // Update FPS counter
        frame_count += 1;
        let current_time = frame_input.accumulated_time;
        
        // Update FPS every second
        if current_time - last_fps_update >= 1000.0 {
            fps = (frame_count as f64) / ((current_time - last_fps_update) / 1000.0);
            frame_count = 0;
            last_fps_update = current_time;
            
            // Update performance metrics
            let metrics = debug::egui::data_types::PerformanceMetrics {
                fps,
                memory_usage: 0.0, // Placeholder
                audio_latency: 0.0, // Placeholder
                cpu_usage: 0.0, // Placeholder
            };
            performance_metrics_setter.set(metrics);
        }
        
        // Three-layer update sequence (engine → model → presenter)
        let timestamp = current_time / 1000.0;
        
        // Update engine layer
        if let Some(ref mut engine) = engine {
            engine.update(timestamp);
        }
        
        // Update model layer
        if let Some(ref mut model) = model {
            model.update(timestamp);
        }
        
        // Update presentation layer
        if let Some(ref mut presenter) = presenter {
            presenter.update(timestamp);
            presenter.update_viewport(frame_input.viewport);
        }
        
        // Extract needed values before borrowing screen
        let accumulated_time = frame_input.accumulated_time;
        let viewport = frame_input.viewport;
        let device_pixel_ratio = frame_input.device_pixel_ratio;
        
        gui.update(
            &mut frame_input.events,
            accumulated_time,
            viewport,
            device_pixel_ratio,
            |gui_context| {
                gui_context.set_visuals(egui::Visuals {
                    window_fill: Color32::from_rgba_unmultiplied(27, 27, 27, 240),
                    override_text_color: Some(Color32::from_rgb(255, 255, 255)),
                    ..egui::Visuals::default()
                });
                
                dev_console.render(gui_context);
                hybrid_live_data_panel.render(gui_context);
            }
        );
        
        let mut screen = frame_input.screen();
        screen.clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0));
        
        // Render presentation layer
        if let Some(ref mut presenter) = presenter {
            presenter.render(&context, &mut screen);
        }
        
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

    web_sys::console::log_1(&"DEBUG: ✓ Platform validation passed - initializing three-layer architecture".into());
    
    // Create shared interfaces using Rc for proper interface sharing
    use std::rc::Rc;
    let engine_to_model = Rc::new(module_interfaces::engine_to_model::EngineToModelInterface::new());
    let model_to_engine = Rc::new(module_interfaces::model_to_engine::ModelToEngineInterface::new());
    let model_to_presentation = Rc::new(module_interfaces::model_to_presentation::ModelToPresentationInterface::new());
    let presentation_to_model = Rc::new(module_interfaces::presentation_to_model::PresentationToModelInterface::new());
    let debug_actions = module_interfaces::debug_actions::DebugActionsInterface::new();
    
    // Create data sources for debug GUI only
    use observable_data::DataSource;
    
    let microphone_permission_source = DataSource::new(engine::audio::AudioPermission::Uninitialized);
    let audio_devices_source = DataSource::new(engine::audio::AudioDevices {
        input_devices: vec![],
        output_devices: vec![],
    });
    let performance_metrics_source = DataSource::new(debug::egui::data_types::PerformanceMetrics::default());
    let audioworklet_status_source = DataSource::new(debug::egui::data_types::AudioWorkletStatus::default());
    let buffer_pool_stats_source = DataSource::new(None::<engine::audio::message_protocol::BufferPoolStats>);
    
    let performance_metrics_setter = performance_metrics_source.setter();
    let microphone_permission_setter = microphone_permission_source.setter();
    
    
    // Create three-layer architecture instances
    dev_log!("Creating three-layer architecture instances...");
    
    // Create engine layer
    let engine = match engine::AudioEngine::create().await {
        Ok(engine) => {
            dev_log!("✓ Engine layer created successfully");
            Some(engine)
        }
        Err(e) => {
            dev_log!("✗ Engine layer creation failed: {}", e);
            dev_log!("Application will continue without engine layer");
            None
        }
    };
    
    // Create model layer
    let model = match model::DataModel::create(
        engine_to_model.clone(),
        model_to_engine.clone(),
        model_to_presentation.clone(),
        presentation_to_model.clone(),
    ) {
        Ok(model) => {
            dev_log!("✓ Model layer created successfully");
            Some(model)
        }
        Err(e) => {
            dev_log!("✗ Model layer creation failed: {}", e);
            dev_log!("Application will continue without model layer");
            None
        }
    };
    
    // Create presentation layer
    let presenter = match presentation::Presenter::create(
        model_to_presentation.clone(),
        presentation_to_model.clone(),
    ) {
        Ok(presenter) => {
            dev_log!("✓ Presentation layer created successfully");
            // Sprite scene will be initialized on first render to avoid variable shadowing
            Some(presenter)
        }
        Err(e) => {
            dev_log!("✗ Presentation layer creation failed: {}", e);
            dev_log!("Application will continue without presentation layer");
            None
        }
    };
    
    
    // Create UI control actions
    let ui_control_actions = UIControlActions::new();
    let listeners = ui_control_actions.get_listeners();
    let triggers = ui_control_actions.get_triggers();
    
    // Set up UI and debug action listeners through the engine
    if let Some(ref engine_instance) = engine {
        // Set up UI action listeners with microphone permission setter
        engine_instance.setup_ui_listeners(
            listeners,
            microphone_permission_setter.clone(),
        );
        
        // Set up debug action listeners
        engine_instance.setup_debug_listeners(&debug_actions);
    }
    
    // Start three-d application with three-layer architecture
    run_three_d_with_layers(
        engine,
        model,
        presenter,
        debug_actions,
        performance_metrics_setter,
        performance_metrics_source.observer(),
        audio_devices_source.observer(),
        audioworklet_status_source.observer(),
        buffer_pool_stats_source.observer(),
        microphone_permission_source.observer(),
        triggers,
        engine_to_model.clone()
    ).await;
}


