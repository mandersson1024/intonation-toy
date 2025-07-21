use three_d::*;
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
use crate::engine::audio::console_service::ConsoleAudioService;

use engine::platform::{Platform, PlatformValidationResult};
use debug::egui::{EguiMicrophoneButton, HybridEguiLiveDataPanel};

use presentation::graphics::SpriteScene;

// Import for hybrid architecture only

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

/// Run three-d with hybrid debug GUI architecture
pub async fn run_three_d_hybrid(
    engine_to_model: module_interfaces::engine_to_model::EngineToModelInterface,
    debug_actions: module_interfaces::debug_actions::DebugActionsInterface,
    performance_metrics_setter: impl observable_data::DataSetter<debug::egui::data_types::PerformanceMetrics> + Clone + 'static,
    performance_metrics_observer: observable_data::DataObserver<debug::egui::data_types::PerformanceMetrics>,
    audio_devices_observer: observable_data::DataObserver<engine::audio::AudioDevices>,
    audioworklet_status_observer: observable_data::DataObserver<debug::egui::data_types::AudioWorkletStatus>,
    buffer_pool_stats_observer: observable_data::DataObserver<Option<engine::audio::message_protocol::BufferPoolStats>>,
    microphone_permission_observer: observable_data::DataObserver<engine::audio::AudioPermission>,
    ui_triggers: UIControlTriggers,
) {
    dev_log!("Starting three-d with hybrid debug GUI architecture");
    
    let window = Window::new(WindowSettings {
        title: "pitch-toy".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    
    let context = window.gl();
    let scene = SpriteScene::new(&context, window.viewport());
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
        &engine_to_model,
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

    dev_log!("Starting three-d + egui render loop with hybrid architecture");
    
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
            let metrics = debug::egui::data_types::PerformanceMetrics {
                fps,
                memory_usage: 0.0, // Placeholder
                audio_latency: 0.0, // Placeholder
                cpu_usage: 0.0, // Placeholder
            };
            performance_metrics_setter.set(metrics);
        }
        
        // Scene doesn't need updating for now
        
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
        scene.render(&mut screen);
        
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
    
    // Create data sources for debug-specific data and bridge components
    use observable_data::DataSource;
    
    let microphone_permission_source = DataSource::new(engine::audio::AudioPermission::Uninitialized);
    let audio_devices_source = DataSource::new(engine::audio::AudioDevices {
        input_devices: vec![],
        output_devices: vec![],
    });
    let performance_metrics_source = DataSource::new(debug::egui::data_types::PerformanceMetrics::default());
    let volume_level_source = DataSource::new(None::<debug::egui::data_types::VolumeLevelData>);
    let pitch_data_source = DataSource::new(None::<debug::egui::data_types::PitchData>);
    let audioworklet_status_source = DataSource::new(debug::egui::data_types::AudioWorkletStatus::default());
    let buffer_pool_stats_source = DataSource::new(None::<engine::audio::message_protocol::BufferPoolStats>);
    
    let microphone_permission_setter = microphone_permission_source.setter();
    let audio_devices_setter = audio_devices_source.setter();
    let audioworklet_status_setter = audioworklet_status_source.setter();
    let performance_metrics_setter = performance_metrics_source.setter();
    let pitch_data_setter = pitch_data_source.setter();
    let volume_level_setter = volume_level_source.setter();
    let buffer_pool_stats_setter = buffer_pool_stats_source.setter();
    
    // Create interfaces for hybrid architecture first
    let engine_to_model = module_interfaces::engine_to_model::EngineToModelInterface::new();
    let debug_actions = module_interfaces::debug_actions::DebugActionsInterface::new();
    
    // Create bridge setters that feed data into both legacy and interface systems
    let audio_analysis_setter = engine_to_model.audio_analysis_setter();
    
    // Create shared state for combining volume and pitch data
    use std::sync::{Arc, Mutex};
    let shared_audio_state = Arc::new(Mutex::new((
        Option::<debug::egui::data_types::VolumeLevelData>::None,
        Option::<debug::egui::data_types::PitchData>::None,
    )));
    
    // Create bridge volume setter that updates both legacy and interface
    let volume_bridge_setter = {
        let legacy_setter = volume_level_setter.clone();
        let interface_setter = audio_analysis_setter.clone();
        let state = shared_audio_state.clone();
        
        struct VolumeBridgeSetter {
            legacy: observable_data::DataSourceSetter<Option<debug::egui::data_types::VolumeLevelData>>,
            interface: observable_data::DataSourceSetter<Option<module_interfaces::engine_to_model::AudioAnalysis>>,
            state: Arc<Mutex<(Option<debug::egui::data_types::VolumeLevelData>, Option<debug::egui::data_types::PitchData>)>>,
        }
        
        impl observable_data::DataSetter<Option<debug::egui::data_types::VolumeLevelData>> for VolumeBridgeSetter {
            fn set(&self, data: Option<debug::egui::data_types::VolumeLevelData>) {
                // Update legacy setter
                self.legacy.set(data.clone());
                
                // Update shared state and interface
                if let Ok(mut state) = self.state.lock() {
                    state.0 = data.clone();
                    
                    // Create combined audio analysis
                    let audio_analysis = module_interfaces::engine_to_model::AudioAnalysis {
                        volume_level: if let Some(volume_data) = &state.0 {
                            module_interfaces::engine_to_model::Volume {
                                peak: volume_data.peak_db,
                                rms: volume_data.rms_db,
                            }
                        } else {
                            module_interfaces::engine_to_model::Volume { peak: 0.0, rms: 0.0 }
                        },
                        pitch: if let Some(pitch_data) = &state.1 {
                            module_interfaces::engine_to_model::Pitch::Detected(pitch_data.frequency, pitch_data.clarity)
                        } else {
                            module_interfaces::engine_to_model::Pitch::NotDetected
                        },
                        fft_data: None,
                        timestamp: js_sys::Date::now() / 1000.0,
                    };
                    self.interface.set(Some(audio_analysis));
                }
            }
        }
        
        std::rc::Rc::new(VolumeBridgeSetter {
            legacy: legacy_setter,
            interface: interface_setter,
            state,
        })
    };
    
    // Create bridge pitch setter that updates both legacy and interface
    let pitch_bridge_setter = {
        let legacy_setter = pitch_data_setter.clone();
        let interface_setter = audio_analysis_setter.clone();
        let state = shared_audio_state.clone();
        
        struct PitchBridgeSetter {
            legacy: observable_data::DataSourceSetter<Option<debug::egui::data_types::PitchData>>,
            interface: observable_data::DataSourceSetter<Option<module_interfaces::engine_to_model::AudioAnalysis>>,
            state: Arc<Mutex<(Option<debug::egui::data_types::VolumeLevelData>, Option<debug::egui::data_types::PitchData>)>>,
        }
        
        impl observable_data::DataSetter<Option<debug::egui::data_types::PitchData>> for PitchBridgeSetter {
            fn set(&self, data: Option<debug::egui::data_types::PitchData>) {
                // Update legacy setter
                self.legacy.set(data.clone());
                
                // Update shared state and interface
                if let Ok(mut state) = self.state.lock() {
                    state.1 = data.clone();
                    
                    // Create combined audio analysis
                    let audio_analysis = module_interfaces::engine_to_model::AudioAnalysis {
                        volume_level: if let Some(volume_data) = &state.0 {
                            module_interfaces::engine_to_model::Volume {
                                peak: volume_data.peak_db,
                                rms: volume_data.rms_db,
                            }
                        } else {
                            module_interfaces::engine_to_model::Volume { peak: 0.0, rms: 0.0 }
                        },
                        pitch: if let Some(pitch_data) = &state.1 {
                            module_interfaces::engine_to_model::Pitch::Detected(pitch_data.frequency, pitch_data.clarity)
                        } else {
                            module_interfaces::engine_to_model::Pitch::NotDetected
                        },
                        fft_data: None,
                        timestamp: if let Some(pitch_data) = data { pitch_data.timestamp } else { js_sys::Date::now() / 1000.0 },
                    };
                    self.interface.set(Some(audio_analysis));
                }
            }
        }
        
        std::rc::Rc::new(PitchBridgeSetter {
            legacy: legacy_setter,
            interface: interface_setter,
            state,
        })
    };
    
    // Create bridge permission setter that updates both legacy and interface
    let permission_bridge_setter = {
        let legacy_setter = microphone_permission_setter.clone();
        let interface_setter = engine_to_model.permission_state_setter();
        
        #[derive(Clone)]
        struct PermissionBridgeSetter {
            legacy: observable_data::DataSourceSetter<engine::audio::AudioPermission>,
            interface: observable_data::DataSourceSetter<module_interfaces::engine_to_model::PermissionState>,
        }
        
        impl observable_data::DataSetter<engine::audio::AudioPermission> for PermissionBridgeSetter {
            fn set(&self, data: engine::audio::AudioPermission) {
                // Update legacy setter
                self.legacy.set(data.clone());
                
                // Convert to interface permission state
                let interface_state = match data {
                    engine::audio::AudioPermission::Uninitialized => module_interfaces::engine_to_model::PermissionState::NotRequested,
                    engine::audio::AudioPermission::Requesting => module_interfaces::engine_to_model::PermissionState::Requested,
                    engine::audio::AudioPermission::Granted => module_interfaces::engine_to_model::PermissionState::Granted,
                    engine::audio::AudioPermission::Denied => module_interfaces::engine_to_model::PermissionState::Denied,
                    engine::audio::AudioPermission::Unavailable => module_interfaces::engine_to_model::PermissionState::Denied, // Map unavailable to denied
                };
                
                self.interface.set(interface_state);
            }
        }
        
        PermissionBridgeSetter {
            legacy: legacy_setter,
            interface: interface_setter,
        }
    };
    
    // Initialize audio systems first - but don't block the UI if it fails
    web_sys::console::log_1(&"DEBUG: Starting audio system initialization...".into());
    let audio_context = match initialize_audio_systems_new(
        pitch_bridge_setter,
        volume_bridge_setter,
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
        let mut service = crate::engine::audio::create_console_audio_service();
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
    
    // Interfaces already created above for bridge setters
    
    // Setup audio module listeners for UI actions (including debug actions)
    match audio_context {
        Some(ref context) => {
            engine::audio::setup_ui_action_listeners_with_context(listeners, permission_bridge_setter.clone(), context.clone());
            // Setup debug action listeners
            engine::audio::context::AudioSystemContext::setup_debug_action_listeners(context, &debug_actions);
        }
        None => {
            web_sys::console::error_1(&"Error: Audio system initialization failed - UI action listeners cannot be set up".into());
        }
    }
    
    // Start three-d application with hybrid architecture
    run_three_d_hybrid(
        engine_to_model,
        debug_actions,
        performance_metrics_setter,
        performance_metrics_source.observer(),
        audio_devices_source.observer(),
        audioworklet_status_source.observer(),
        buffer_pool_stats_source.observer(),
        microphone_permission_source.observer(),
        triggers,
    ).await;
}

/// Initialize all audio systems using AudioSystemContext approach
async fn initialize_audio_systems_new(
    pitch_data_setter: std::rc::Rc<dyn observable_data::DataSetter<Option<debug::egui::data_types::PitchData>>>,
    volume_level_setter: std::rc::Rc<dyn observable_data::DataSetter<Option<debug::egui::data_types::VolumeLevelData>>>,
    audioworklet_status_setter: std::rc::Rc<dyn observable_data::DataSetter<debug::egui::data_types::AudioWorkletStatus>>,
    buffer_pool_stats_setter: std::rc::Rc<dyn observable_data::DataSetter<Option<engine::audio::message_protocol::BufferPoolStats>>>
) -> Result<std::rc::Rc<std::cell::RefCell<engine::audio::AudioSystemContext>>, String> {
    // Convert setters to required types with adapters
    let pitch_setter = std::rc::Rc::new(crate::debug::egui::data_types::PitchDataAdapter::new(pitch_data_setter))
        as std::rc::Rc<dyn observable_data::DataSetter<Option<engine::audio::PitchData>>>;
    
    let volume_setter = std::rc::Rc::new(crate::debug::egui::data_types::VolumeDataAdapter::new(volume_level_setter))
        as std::rc::Rc<dyn observable_data::DataSetter<Option<engine::audio::VolumeLevelData>>>;
    
    let status_setter = std::rc::Rc::new(crate::debug::egui::data_types::AudioWorkletStatusAdapter::new(audioworklet_status_setter))
        as std::rc::Rc<dyn observable_data::DataSetter<engine::audio::AudioWorkletStatus>>;
    
    let buffer_stats_setter = buffer_pool_stats_setter;
    
    web_sys::console::log_1(&"DEBUG: Using AudioSystemContext initialization approach".into());
    
    // Use the initialization function
    let context = engine::audio::initialize_audio_system_with_context(
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
        engine::audio::set_global_audio_context_manager(manager_rc);
    }
    
    dev_log!("✓ AudioSystemContext components available globally for backward compatibility");
    
    
    Ok(context_rc)
}

