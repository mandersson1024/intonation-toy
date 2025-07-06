// Live Panel Component - Real-time data visualization and monitoring
//
// This component provides real-time monitoring and visualization of audio system state.
// It displays audio devices, permission status, performance metrics, and volume levels.

use yew::prelude::*;
use web_sys::window;

use crate::audio::{AudioPermission, AudioDevices};
use crate::audio::console_service::ConsoleAudioService;
use crate::audio::worklet::AudioWorkletState;
use crate::events::AudioEventDispatcher;

/// Properties for the LivePanel component
#[derive(Properties)]
pub struct LivePanelProps {
    /// Event dispatcher for receiving real-time updates
    pub event_dispatcher: AudioEventDispatcher,
    /// Whether the panel is visible
    pub visible: bool,
    /// Current audio permission state
    pub audio_permission: AudioPermission,
    /// Audio service for device operations
    pub audio_service: std::rc::Rc<crate::audio::ConsoleAudioServiceImpl>,
}

impl PartialEq for LivePanelProps {
    fn eq(&self, other: &Self) -> bool {
        self.visible == other.visible && 
        self.audio_permission == other.audio_permission &&
        std::rc::Rc::ptr_eq(&self.audio_service, &other.audio_service)
    }
}

/// Performance metrics for display
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceMetrics {
    pub fps: f64,
    pub memory_usage: f64,
    pub audio_latency: f64,
    pub cpu_usage: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            fps: 0.0,
            memory_usage: 0.0,
            audio_latency: 0.0,
            cpu_usage: 0.0,
        }
    }
}

/// Volume level data for display
#[derive(Debug, Clone, PartialEq)]
pub struct VolumeLevel {
    pub level: f32,
    pub peak: f32,
    pub timestamp: f64,
}

/// AudioWorklet status for display
#[derive(Debug, Clone, PartialEq)]
pub struct AudioWorkletStatus {
    pub state: AudioWorkletState,
    pub processor_loaded: bool,
    pub chunk_size: u32,
    pub chunks_processed: u32,
    pub last_update: f64,
}

impl Default for AudioWorkletStatus {
    fn default() -> Self {
        Self {
            state: AudioWorkletState::Uninitialized,
            processor_loaded: false,
            chunk_size: 128,
            chunks_processed: 0,
            last_update: 0.0,
        }
    }
}

/// State for the LivePanel component
pub struct LivePanel {
    /// Current audio devices
    audio_devices: AudioDevices,
    /// Current audio permission
    audio_permission: AudioPermission,
    /// Performance metrics
    performance_metrics: PerformanceMetrics,
    /// Current volume level
    volume_level: Option<VolumeLevel>,
    /// AudioWorklet status
    audioworklet_status: AudioWorkletStatus,
    /// Performance monitoring interval
    _performance_interval: Option<gloo_timers::callback::Interval>,
}

/// Messages for the LivePanel component
#[derive(Debug)]
pub enum LivePanelMsg {
    /// Update audio devices
    UpdateDevices(AudioDevices),
    /// Update audio permission
    UpdatePermission(AudioPermission),
    /// Update performance metrics
    UpdatePerformanceMetrics(PerformanceMetrics),
    /// Update volume level
    UpdateVolumeLevel(VolumeLevel),
    /// Update AudioWorklet status
    UpdateAudioWorkletStatus(AudioWorkletStatus),
}

impl Component for LivePanel {
    type Message = LivePanelMsg;
    type Properties = LivePanelProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut component = Self {
            audio_devices: AudioDevices::new(),
            audio_permission: ctx.props().audio_permission.clone(),
            performance_metrics: PerformanceMetrics::default(),
            volume_level: None,
            audioworklet_status: AudioWorkletStatus::default(),
            _performance_interval: None,
        };

        // Set up event subscriptions
        component.setup_event_subscriptions(ctx);
        
        // Start performance monitoring
        component.start_performance_monitoring(ctx);

        // Trigger initial device refresh
        ctx.props().audio_service.refresh_devices();

        component
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            LivePanelMsg::UpdateDevices(devices) => {
                self.audio_devices = devices;
                true
            }
            LivePanelMsg::UpdatePermission(permission) => {
                self.audio_permission = permission;
                true
            }
            LivePanelMsg::UpdatePerformanceMetrics(metrics) => {
                self.performance_metrics = metrics;
                true
            }
            LivePanelMsg::UpdateVolumeLevel(level) => {
                self.volume_level = Some(level);
                true
            }
            LivePanelMsg::UpdateAudioWorkletStatus(status) => {
                self.audioworklet_status = status;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if !ctx.props().visible {
            return html! {};
        }

        html! {
            <div class="live-panel">
                <div class="live-panel-header">
                    <h3 class="live-panel-title">{"Live Data Panel"}</h3>
                </div>
                
                <div class="live-panel-content">
                    {self.render_permission_status(ctx)}
                    {self.render_device_list()}
                    {self.render_audioworklet_status()}
                    {self.render_performance_metrics()}
                    {self.render_volume_level()}
                    {self.render_pitch_detection()}
                </div>
            </div>
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        // Clean up performance monitoring
        self._performance_interval = None;
    }
}

impl LivePanel {
    /// Set up event subscriptions for real-time updates
    fn setup_event_subscriptions(&mut self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        let audio_service = ctx.props().audio_service.clone();
        
        // Subscribe to device changes
        audio_service.subscribe_device_changes(Box::new(move |devices| {
            link.send_message(LivePanelMsg::UpdateDevices(devices));
        }));
        
        // Subscribe to AudioWorklet status changes
        let link_clone = ctx.link().clone();
        ctx.props().event_dispatcher.borrow_mut().subscribe("audioworklet_status_changed", move |event| {
            if let crate::events::audio_events::AudioEvent::AudioWorkletStatusChanged(status) = event {
                link_clone.send_message(LivePanelMsg::UpdateAudioWorkletStatus(status));
            }
        });
    }

    /// Start performance monitoring
    fn start_performance_monitoring(&mut self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        
        let interval = gloo_timers::callback::Interval::new(1000, move || {
            let metrics = Self::collect_performance_metrics();
            link.send_message(LivePanelMsg::UpdatePerformanceMetrics(metrics));
        });
        
        self._performance_interval = Some(interval);
    }

    /// Collect current performance metrics
    fn collect_performance_metrics() -> PerformanceMetrics {
        let window = window().unwrap();
        let _performance = window.performance().unwrap();
        
        // Calculate FPS (placeholder implementation)
        let fps = 60.0; // TODO: Implement actual FPS calculation
        
        // Get memory usage (placeholder implementation)
        let memory_usage = 0.0; // TODO: Implement memory usage when Performance.memory is available
        
        // Audio latency (placeholder)
        let audio_latency = 0.0; // TODO: Implement actual audio latency measurement
        
        // CPU usage (placeholder)
        let cpu_usage = 0.0; // TODO: Implement CPU usage estimation
        
        PerformanceMetrics {
            fps,
            memory_usage,
            audio_latency,
            cpu_usage,
        }
    }

    /// Render permission status section
    fn render_permission_status(&self, ctx: &Context<Self>) -> Html {
        let (status_text, status_class) = match ctx.props().audio_permission {
            AudioPermission::Granted => ("Granted", "permission-granted"),
            AudioPermission::Denied => ("Denied", "permission-denied"),
            AudioPermission::Requesting => ("Requesting", "permission-requesting"),
            AudioPermission::Uninitialized => ("Not Requested", "permission-unknown"),
            AudioPermission::Unavailable => ("Unavailable", "permission-unavailable"),
        };

        html! {
            <div class="live-panel-section">
                <h4 class="live-panel-section-title">{"Audio Permission"}</h4>
                <div class={format!("permission-status {}", status_class)}>
                    {status_text}
                </div>
            </div>
        }
    }

    /// Render AudioWorklet status section
    fn render_audioworklet_status(&self) -> Html {
        let (state_text, state_class) = match self.audioworklet_status.state {
            AudioWorkletState::Uninitialized => ("Not Initialized", "status-inactive"),
            AudioWorkletState::Initializing => ("Initializing", "status-pending"),
            AudioWorkletState::Ready => ("Ready", "status-neutral"),
            AudioWorkletState::Processing => ("Processing", "status-success"),
            AudioWorkletState::Stopped => ("Stopped", "status-warning"),
            AudioWorkletState::Failed => ("Failed", "status-error"),
        };

        let (processor_status, processor_class) = if self.audioworklet_status.processor_loaded {
            ("Loaded", "status-success")
        } else {
            ("Not Loaded", "status-inactive")
        };

        html! {
            <div class="live-panel-section">
                <h4 class="live-panel-section-title">{"AudioWorklet Status"}</h4>
                <div class="audioworklet-status">
                    <div class="status-item">
                        <span class="status-label">{"State:"}</span>
                        <span class={format!("status-value {}", state_class)}>{state_text}</span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">{"Processor:"}</span>
                        <span class={format!("status-value {}", processor_class)}>{processor_status}</span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">{"Chunk Size:"}</span>
                        <span class="status-value">{format!("{} samples", self.audioworklet_status.chunk_size)}</span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">{"Chunks Processed:"}</span>
                        <span class="status-value">{self.audioworklet_status.chunks_processed}</span>
                    </div>
                </div>
            </div>
        }
    }

    /// Render pitch detection section
    fn render_pitch_detection(&self) -> Html {
        html! {
            <div class="live-panel-section">
                <h4 class="live-panel-section-title">{"Pitch Detection"}</h4>
                <div class="pitch-detection-data">
                    <div class="pitch-placeholder">
                        {"No pitch data available"}
                    </div>
                </div>
            </div>
        }
    }

    /// Render device list section
    fn render_device_list(&self) -> Html {
        html! {
            <div class="live-panel-section">
                <h4 class="live-panel-section-title">{"Audio Devices"}</h4>
                <div class="device-list">
                    <div class="device-section">
                        <h5>{"Input Devices"}</h5>
                        <div class="device-items">
                            {if self.audio_devices.input_devices.is_empty() {
                                html! {
                                    <div class="device-item">
                                        <span class="device-name permission-required">{"permission required"}</span>
                                    </div>
                                }
                            } else {
                                html! {
                                    <>
                                        {for self.audio_devices.input_devices.iter().map(|device| {
                                            html! {
                                                <div class="device-item">
                                                    <span class="device-name">{&device.1}</span>
                                                </div>
                                            }
                                        })}
                                    </>
                                }
                            }}
                        </div>
                    </div>
                    
                    <div class="device-section">
                        <h5>{"Output Devices"}</h5>
                        <div class="device-items">
                            {if self.audio_devices.output_devices.is_empty() {
                                html! {
                                    <div class="device-item">
                                        <span class="device-name permission-required">{"permission required"}</span>
                                    </div>
                                }
                            } else {
                                html! {
                                    <>
                                        {for self.audio_devices.output_devices.iter().map(|device| {
                                            html! {
                                                <div class="device-item">
                                                    <span class="device-name">{&device.1}</span>
                                                </div>
                                            }
                                        })}
                                    </>
                                }
                            }}
                        </div>
                    </div>
                </div>
            </div>
        }
    }

    /// Render performance metrics section
    fn render_performance_metrics(&self) -> Html {
        html! {
            <div class="live-panel-section">
                <h4 class="live-panel-section-title">{"Performance Metrics"}</h4>
                <div class="metrics-grid">
                    <div class="metric-item">
                        <span class="metric-label">{"FPS"}</span>
                        <span class="metric-value">{format!("{:.1}", self.performance_metrics.fps)}</span>
                    </div>
                    <div class="metric-item">
                        <span class="metric-label">{"Memory"}</span>
                        <span class="metric-value">{format!("{:.1} MB", self.performance_metrics.memory_usage)}</span>
                    </div>
                    <div class="metric-item">
                        <span class="metric-label">{"Audio Latency"}</span>
                        <span class="metric-value">{format!("{:.1} ms", self.performance_metrics.audio_latency)}</span>
                    </div>
                    <div class="metric-item">
                        <span class="metric-label">{"CPU Usage"}</span>
                        <span class="metric-value">{format!("{:.1}%", self.performance_metrics.cpu_usage)}</span>
                    </div>
                </div>
            </div>
        }
    }

    /// Render volume level section
    fn render_volume_level(&self) -> Html {
        html! {
            <div class="live-panel-section">
                <h4 class="live-panel-section-title">{"Volume Level"}</h4>
                <div class="volume-display">
                    {if let Some(volume) = &self.volume_level {
                        html! {
                            <div class="volume-meters">
                                <div class="volume-meter">
                                    <div class="volume-label">{"Level"}</div>
                                    <div class="volume-bar">
                                        <div 
                                            class="volume-fill"
                                            style={format!("width: {}%", volume.level * 100.0)}
                                        />
                                    </div>
                                    <div class="volume-value">{format!("{:.1}%", volume.level * 100.0)}</div>
                                </div>
                                <div class="volume-meter">
                                    <div class="volume-label">{"Peak"}</div>
                                    <div class="volume-bar">
                                        <div 
                                            class="volume-fill volume-peak"
                                            style={format!("width: {}%", volume.peak * 100.0)}
                                        />
                                    </div>
                                    <div class="volume-value">{format!("{:.1}%", volume.peak * 100.0)}</div>
                                </div>
                            </div>
                        }
                    } else {
                        html! {
                            <div class="volume-placeholder">
                                {"No volume data available"}
                            </div>
                        }
                    }}
                </div>
            </div>
        }
    }
}