// Live Panel Component - Real-time data visualization and monitoring
//
// This component provides real-time monitoring and visualization of audio system state.

use yew::prelude::*;
use web_sys::HtmlInputElement;
use wasm_bindgen::JsCast;

use crate::audio::console_service::ConsoleAudioService;
use crate::audio::worklet::AudioWorkletState;
use crate::events::AudioEventDispatcher;
use crate::live_data::LiveData;
use super::{TestSignalControls, TestSignalConfig, BackgroundNoiseConfig};

/// Properties for the LivePanel component
#[derive(Properties)]
pub struct LivePanelProps {
    /// Event dispatcher for receiving real-time updates
    pub event_dispatcher: AudioEventDispatcher,
    /// Whether the panel is visible
    pub visible: bool,
    /// Audio service for device operations
    pub audio_service: std::rc::Rc<crate::audio::ConsoleAudioServiceImpl>,
    /// Live data observers for real-time data sharing
    pub live_data: LiveData,
}

impl PartialEq for LivePanelProps {
    fn eq(&self, other: &Self) -> bool {
        self.visible == other.visible && 
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
    /// Current volume level
    volume_level: Option<VolumeLevelData>,
    /// Current pitch data
    pitch_data: Option<PitchData>,
    /// Test signal configuration
    test_signal_config: TestSignalConfig,
    /// Background noise configuration
    background_noise_config: BackgroundNoiseConfig,
    /// Whether output to speakers is enabled
    output_to_speakers: bool,
}

/// Messages for the LivePanel component
#[derive(Debug)]
pub enum LivePanelMsg {
    /// Update volume level
    UpdateVolumeLevel(VolumeLevelData),
    /// Update pitch data
    UpdatePitchData(PitchData),
    /// Update test signal configuration
    UpdateTestSignalConfig(TestSignalConfig),
    /// Update background noise configuration
    UpdateBackgroundNoiseConfig(BackgroundNoiseConfig),
    /// Toggle output to speakers
    ToggleOutputToSpeakers(bool),
}

impl Component for LivePanel {
    type Message = LivePanelMsg;
    type Properties = LivePanelProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut component = Self {
            volume_level: None,
            pitch_data: None,
            test_signal_config: TestSignalConfig::default(),
            background_noise_config: BackgroundNoiseConfig::default(),
            output_to_speakers: false,
        };

        // Set up event subscriptions
        component.setup_event_subscriptions(ctx);
        

        // Trigger initial device refresh
        ctx.props().audio_service.refresh_devices();

        component
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            LivePanelMsg::UpdateVolumeLevel(level) => {
                self.volume_level = Some(level);
                true
            }
            LivePanelMsg::UpdatePitchData(pitch) => {
                self.pitch_data = Some(pitch);
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
            LivePanelMsg::UpdateBackgroundNoiseConfig(config) => {
                self.background_noise_config = config.clone();
                
                // Apply background noise configuration to audio system
                if let Some(worklet_rc) = crate::audio::get_global_audioworklet_manager() {
                    let mut worklet = worklet_rc.borrow_mut();
                    
                    // Convert UI config to audio system config
                    let audio_config = crate::audio::BackgroundNoiseConfig {
                        enabled: config.enabled,
                        level: config.level,
                        noise_type: match config.noise_type {
                            super::TestWaveform::WhiteNoise => crate::audio::TestWaveform::WhiteNoise,
                            super::TestWaveform::PinkNoise => crate::audio::TestWaveform::PinkNoise,
                            _ => crate::audio::TestWaveform::WhiteNoise, // Default to white noise
                        },
                    };
                    
                    worklet.update_background_noise_config(audio_config.clone());
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
                    {self.render_device_list(ctx)}
                    {self.render_audioworklet_status(ctx)}
                    {self.render_test_signal_controls(ctx)}
                    {self.render_background_noise_controls(ctx)}
                    {self.render_global_audio_controls(ctx)}
                    {self.render_performance_metrics(ctx)}
                    {self.render_volume_level()}
                    {self.render_pitch_detection()}
                </div>
            </div>
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        // No cleanup needed
    }
}

impl LivePanel {
    /// Set up event subscriptions for real-time updates
    fn setup_event_subscriptions(&mut self, ctx: &Context<Self>) {
        // Note: Audio devices now come from shared live_data, no subscription needed
        
        // Note: AudioWorklet status now comes from shared live_data, no subscription needed
        
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


    /// Render AudioWorklet status section
    fn render_audioworklet_status(&self, ctx: &Context<Self>) -> Html {
        let audioworklet_status = ctx.props().live_data.audioworklet_status.get();
        let (state_text, state_class) = match audioworklet_status.state {
            AudioWorkletState::Uninitialized => ("Not Initialized", "status-inactive"),
            AudioWorkletState::Initializing => ("Initializing", "status-pending"),
            AudioWorkletState::Ready => ("Ready", "status-neutral"),
            AudioWorkletState::Processing => ("Processing", "status-success"),
            AudioWorkletState::Stopped => ("Stopped", "status-warning"),
            AudioWorkletState::Failed => ("Failed", "status-error"),
        };

        let (processor_status, processor_class) = if audioworklet_status.processor_loaded {
            ("Loaded", "status-success")
        } else {
            ("Not Loaded", "status-inactive")
        };

        html! {
            <div class="live-panel-section">
                <h4 class="live-panel-section-title">{"AudioWorklet Status"}</h4>
                <div class="audioworklet-status">
                    <div class="metric-item">
                        <span class="metric-label">{"State"}</span>
                        <span class={format!("metric-value {}", state_class)}>{state_text}</span>
                    </div>
                    <div class="metric-item">
                        <span class="metric-label">{"Processor"}</span>
                        <span class={format!("metric-value {}", processor_class)}>{processor_status}</span>
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
                                    <span class="metric-value note-with-cents">
                                        <span class="note-name" style="display: inline-block; min-width: 3em; text-align: left;">{format!("{}", pitch.note)}</span>
                                        <span class="cents-value">
                                            {"("}
                                            <span style="display: inline-block; min-width: 2.5em; text-align: right;">{
                                                if pitch.note.cents >= 0.0 {
                                                    format!("+{:.0}", pitch.note.cents)
                                                } else {
                                                    format!("{:.0}", pitch.note.cents)
                                                }
                                            }</span>
                                            {" cents)"}
                                        </span>
                                    </span>
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
    fn render_device_list(&self, ctx: &Context<Self>) -> Html {
        let audio_devices = ctx.props().live_data.audio_devices.get();
        
        html! {
            <div class="live-panel-section">
                <h4 class="live-panel-section-title">{"Audio Devices"}</h4>
                <div class="device-list">
                    <div class="device-section">
                        <h5>{"Input Devices"}</h5>
                        <div class="device-items">
                            {if audio_devices.input_devices.is_empty() {
                                html! {
                                    <div class="device-item">
                                        <span class="device-name permission-required">{"permission required"}</span>
                                    </div>
                                }
                            } else {
                                html! {
                                    <>
                                        {for audio_devices.input_devices.iter().map(|device| {
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
                            {if audio_devices.output_devices.is_empty() {
                                html! {
                                    <div class="device-item">
                                        <span class="device-name permission-required">{"permission required"}</span>
                                    </div>
                                }
                            } else {
                                html! {
                                    <>
                                        {for audio_devices.output_devices.iter().map(|device| {
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
    fn render_performance_metrics(&self, ctx: &Context<Self>) -> Html {
        let performance_metrics = ctx.props().live_data.performance_metrics.get();
        html! {
            <div class="live-panel-section">
                <h4 class="live-panel-section-title">{"Performance Metrics"}</h4>
                <div class="metrics-grid">
                    <div class="metric-item">
                        <span class="metric-label">{"FPS"}</span>
                        <span class="metric-value">{format!("{:.1}", performance_metrics.fps)}</span>
                    </div>
                    <div class="metric-item">
                        <span class="metric-label">{"Memory"}</span>
                        <span class="metric-value">{format!("{:.1} MB", performance_metrics.memory_usage)}</span>
                    </div>
                    <div class="metric-item">
                        <span class="metric-label">{"Audio Latency"}</span>
                        <span class="metric-value">{format!("{:.1} ms", performance_metrics.audio_latency)}</span>
                    </div>
                    <div class="metric-item">
                        <span class="metric-label">{"CPU Usage"}</span>
                        <span class="metric-value">{format!("{:.1}%", performance_metrics.cpu_usage)}</span>
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
    
    /// Convert dB to linear amplitude (0.0 - 1.0)
    fn db_to_amplitude(db: f32) -> f32 {
        if db == -f32::INFINITY {
            0.0
        } else {
            10.0_f32.powf(db / 20.0)
        }
    }
    
    /// Render animated volume bar for an amplitude value
    fn render_volume_bar(db_value: f32, bar_type: &str) -> Html {
        // Convert dB to amplitude (0.0 - 1.0)
        let amplitude = Self::db_to_amplitude(db_value);
        
        // Convert amplitude to percentage (0-100%)
        let percentage = (amplitude * 100.0).max(0.0).min(100.0);
        
        // Determine bar color based on amplitude level
        let bar_class = if amplitude > 0.5 {
            "volume-bar-hot"  // Red: High amplitude (>50%)
        } else if amplitude > 0.25 {
            "volume-bar-warm" // Yellow: Medium amplitude (25-50%)
        } else if amplitude > 0.1 {
            "volume-bar-cool" // Green: Low amplitude (10-25%)
        } else {
            "volume-bar-cold" // Blue: Very low amplitude (<10%)
        };
        
        html! {
            <div class={format!("volume-bar-container volume-bar-{}", bar_type)}>
                <div class="volume-bar-track">
                    <div 
                        class={format!("volume-bar-fill {}", bar_class)}
                        style={format!("width: {}%", percentage)}
                    >
                    </div>
                    // Add level markers for amplitude percentages
                    <div class="volume-bar-markers">
                        <div class="volume-marker volume-marker-cold" style="left: 10%"></div>   // 10% amplitude
                        <div class="volume-marker volume-marker-cool" style="left: 25%"></div>    // 25% amplitude
                        <div class="volume-marker volume-marker-warm" style="left: 50%"></div>    // 50% amplitude
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

    /// Render background noise controls
    fn render_background_noise_controls(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        
        html! {
            <div class="live-panel-section">
                <div class="control-item control-range">
                    <span class="control-label">{"Background noise"}</span>
                    <div class="control-slider-container">
                        <input 
                            type="range" 
                            id="bg-noise-level"
                            min="0.0" 
                            max="1.0" 
                            step="0.01"
                            value={self.background_noise_config.level.to_string()}
                            oninput={{
                                let current_config = self.background_noise_config.clone();
                                link.callback(move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                                    let level = input.value().parse::<f32>().unwrap_or(0.0);
                                    let mut config = current_config.clone();
                                    config.enabled = level > 0.0; // Auto-enable when level > 0
                                    config.level = level;
                                    LivePanelMsg::UpdateBackgroundNoiseConfig(config)
                                })
                            }}
                            class="control-slider"
                        />
                        <span class="control-value">{format!("{:.0}%", self.background_noise_config.level * 100.0)}</span>
                    </div>
                </div>
            </div>
        }
    }

    fn render_global_audio_controls(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        
        html! {
            <div class="live-panel-section">
                <div class="control-item">
                    <span class="control-label">{"Output to speakers"}</span>
                    <div class="button-group-horizontal">
                        <button 
                            class={if !self.output_to_speakers { "button-option active" } else { "button-option" }}
                            onclick={link.callback(|_| LivePanelMsg::ToggleOutputToSpeakers(false))}
                            title="Don't Output to Speakers"
                        >
                            {"🔇"}
                        </button>
                        <button 
                            class={if self.output_to_speakers { "button-option active" } else { "button-option" }}
                            onclick={link.callback(|_| LivePanelMsg::ToggleOutputToSpeakers(true))}
                            title="Output to Speakers"
                        >
                            {"🔊"}
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}