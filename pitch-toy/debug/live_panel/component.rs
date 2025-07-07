// Live Panel Component - Real-time data visualization and monitoring
//
// This component provides real-time monitoring and visualization of audio system state.
// It displays audio devices, permission status, performance metrics, and volume levels.

use yew::prelude::*;
use web_sys::{window, HtmlInputElement};
use wasm_bindgen::JsCast;

use crate::audio::{AudioPermission, AudioDevices};
use crate::audio::console_service::ConsoleAudioService;
use crate::audio::worklet::AudioWorkletState;
use crate::events::AudioEventDispatcher;
use super::{TestSignalControls, TestSignalConfig};

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
pub struct VolumeLevelData {
    pub rms_db: f32,
    pub peak_db: f32,
    pub peak_fast_db: f32,
    pub peak_slow_db: f32,
    pub level: crate::audio::VolumeLevel,
    pub confidence_weight: f32,
    pub timestamp: f64,
}

/// Pitch detection data for display
#[derive(Debug, Clone, PartialEq)]
pub struct PitchData {
    pub frequency: f32,
    pub confidence: f32,
    pub note: crate::audio::MusicalNote,
    pub clarity: f32,
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
    volume_level: Option<VolumeLevelData>,
    /// Current pitch data
    pitch_data: Option<PitchData>,
    /// AudioWorklet status
    audioworklet_status: AudioWorkletStatus,
    /// Test signal configuration
    test_signal_config: TestSignalConfig,
    /// Whether output to speakers is enabled
    output_to_speakers: bool,
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
    UpdateVolumeLevel(VolumeLevelData),
    /// Update pitch data
    UpdatePitchData(PitchData),
    /// Update AudioWorklet status
    UpdateAudioWorkletStatus(AudioWorkletStatus),
    /// Update test signal configuration
    UpdateTestSignalConfig(TestSignalConfig),
    /// Toggle output to speakers
    ToggleOutputToSpeakers(bool),
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
            pitch_data: None,
            audioworklet_status: AudioWorkletStatus::default(),
            test_signal_config: TestSignalConfig::default(),
            output_to_speakers: false,
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
            LivePanelMsg::UpdatePitchData(pitch) => {
                self.pitch_data = Some(pitch);
                true
            }
            LivePanelMsg::UpdateAudioWorkletStatus(status) => {
                self.audioworklet_status = status;
                true
            }
            LivePanelMsg::UpdateTestSignalConfig(config) => {
                self.test_signal_config = config.clone();
                
                // Apply test signal configuration to audio system
                if let Some(worklet_rc) = crate::audio::get_global_audioworklet_manager() {
                    let mut worklet = worklet_rc.borrow_mut();
                    
                    // Convert UI config to audio system config
                    let audio_config = crate::audio::TestSignalGeneratorConfig {
                        enabled: config.enabled,
                        frequency: config.frequency,
                        amplitude: config.volume,
                        noise_level: config.noise_floor,
                        waveform: match config.waveform {
                            super::TestWaveform::Sine => crate::audio::TestWaveform::Sine,
                            super::TestWaveform::Square => crate::audio::TestWaveform::Square,
                            super::TestWaveform::Sawtooth => crate::audio::TestWaveform::Sawtooth,
                            super::TestWaveform::Triangle => crate::audio::TestWaveform::Triangle,
                            super::TestWaveform::WhiteNoise => crate::audio::TestWaveform::WhiteNoise,
                            super::TestWaveform::PinkNoise => crate::audio::TestWaveform::PinkNoise,
                        },
                        sample_rate: 48000.0, // Use standard sample rate
                    };
                    
                    worklet.update_test_signal_config(audio_config.clone());
                }
                
                true
            }
            LivePanelMsg::ToggleOutputToSpeakers(enabled) => {
                self.output_to_speakers = enabled;
                
                // Apply output to speakers setting to audio system
                if let Some(worklet_rc) = crate::audio::get_global_audioworklet_manager() {
                    let mut worklet = worklet_rc.borrow_mut();
                    worklet.set_output_to_speakers(enabled);
                }
                
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
                    {self.render_test_signal_controls(ctx)}
                    {self.render_global_audio_controls(ctx)}
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
        
        // Subscribe to pitch detection events
        let link_clone2 = ctx.link().clone();
        ctx.props().event_dispatcher.borrow_mut().subscribe("pitch_detected", move |event| {
            if let crate::events::audio_events::AudioEvent::PitchDetected { frequency, confidence, note, clarity, timestamp } = event {
                let pitch_data = PitchData {
                    frequency,
                    confidence,
                    note,
                    clarity,
                    timestamp,
                };
                link_clone2.send_message(LivePanelMsg::UpdatePitchData(pitch_data));
            }
        });
        
        // Subscribe to volume detection events
        let link_clone3 = ctx.link().clone();
        ctx.props().event_dispatcher.borrow_mut().subscribe("volume_detected", move |event| {
            if let crate::events::audio_events::AudioEvent::VolumeDetected { rms_db, peak_db, peak_fast_db, peak_slow_db, level, confidence_weight, timestamp } = event {
                let volume_data = VolumeLevelData {
                    rms_db,
                    peak_db,
                    peak_fast_db,
                    peak_slow_db,
                    level,
                    confidence_weight,
                    timestamp,
                };
                link_clone3.send_message(LivePanelMsg::UpdateVolumeLevel(volume_data));
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
                    {if let Some(pitch) = &self.pitch_data {
                        html! {
                            <div class="pitch-data">
                                <div class="metric-item">
                                    <span class="metric-label">{"Frequency"}</span>
                                    <span class="metric-value">{format!("{:.2} Hz", pitch.frequency)}</span>
                                </div>
                                <div class="metric-item">
                                    <span class="metric-label">{"Note"}</span>
                                    <span class="metric-value">{format!("{}", pitch.note)}</span>
                                </div>
                                <div class="metric-item">
                                    <span class="metric-label">{"Confidence"}</span>
                                    <span class="metric-value">{format!("{:.1}%", pitch.confidence * 100.0)}</span>
                                </div>
                                <div class="metric-item">
                                    <span class="metric-label">{"Clarity"}</span>
                                    <span class="metric-value">{format!("{:.1}%", pitch.clarity * 100.0)}</span>
                                </div>
                            </div>
                        }
                    } else {
                        html! {
                            <div class="pitch-placeholder">
                                {"No pitch data available"}
                            </div>
                        }
                    }}
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
                            <div class="volume-display">
                                <div class="volume-metric-item">
                                    <span class="metric-label">{"Peak Level"}</span>
                                    <span class="metric-value">{format!("{:.1} dB", volume.peak_db)}</span>
                                    {Self::render_volume_bar(volume.peak_db, "peak")}
                                </div>
                                <div class="metric-item">
                                    <span class="metric-label">{"Level Category"}</span>
                                    <span class="metric-value">{format!("{}", volume.level)}</span>
                                </div>
                                <div class="metric-item">
                                    <span class="metric-label">{"Confidence Weight"}</span>
                                    <span class="metric-value">{format!("{:.1}%", volume.confidence_weight * 100.0)}</span>
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
    
    /// Render animated volume bar for a dB value
    fn render_volume_bar(db_value: f32, bar_type: &str) -> Html {
        // Convert dB to percentage (typical range: -60dB to 0dB)
        let min_db = -60.0;
        let max_db = 0.0;
        let percentage = ((db_value - min_db) / (max_db - min_db) * 100.0).max(0.0).min(100.0);
        
        // Determine bar color based on level
        let bar_class = if db_value > -6.0 {
            "volume-bar-hot"  // Red: Hot/clipping
        } else if db_value > -18.0 {
            "volume-bar-warm" // Yellow: Warm/good
        } else if db_value > -40.0 {
            "volume-bar-cool" // Green: Cool/safe
        } else {
            "volume-bar-cold" // Blue: Cold/quiet
        };
        
        html! {
            <div class={format!("volume-bar-container volume-bar-{}", bar_type)}>
                <div class="volume-bar-track">
                    <div 
                        class={format!("volume-bar-fill {}", bar_class)}
                        style={format!("width: {}%", percentage)}
                    >
                    </div>
                    // Add level markers
                    <div class="volume-bar-markers">
                        <div class="volume-marker volume-marker-cold" style="left: 33.3%"></div>   // -40dB
                        <div class="volume-marker volume-marker-cool" style="left: 70%"></div>     // -18dB
                        <div class="volume-marker volume-marker-warm" style="left: 90%"></div>     // -6dB
                    </div>
                </div>
            </div>
        }
    }

    /// Render test signal controls
    fn render_test_signal_controls(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let on_config_change = link.callback(LivePanelMsg::UpdateTestSignalConfig);
        
        html! {
            <TestSignalControls
                config={self.test_signal_config.clone()}
                on_config_change={on_config_change}
            />
        }
    }

    fn render_global_audio_controls(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        
        html! {
            <div class="live-panel-section">
                <h4 class="live-panel-section-title">{"Global Audio Controls"}</h4>
                <div class="global-audio-controls">
                    <div class="control-item control-toggle">
                        <label class="control-label">
                            <input 
                                type="checkbox" 
                                checked={self.output_to_speakers}
                                onchange={link.callback(|e: Event| {
                                    let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                                    LivePanelMsg::ToggleOutputToSpeakers(input.checked())
                                })}
                                class="control-checkbox"
                            />
                            <span class="control-text">{"Output to Speakers"}</span>
                        </label>
                        <div class={format!("status-indicator {}", 
                            if self.output_to_speakers { "status-active" } else { "status-inactive" }
                        )}>
                            {if self.output_to_speakers { "🔊" } else { "🔇" }}
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}