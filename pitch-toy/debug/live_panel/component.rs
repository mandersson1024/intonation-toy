// Live Panel Component - Real-time data visualization and monitoring
//
// This component provides real-time monitoring and visualization of audio system state.

use yew::prelude::*;
use web_sys::HtmlInputElement;
use wasm_bindgen::JsCast;

use crate::audio::console_service::ConsoleAudioService;
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





/// State for the LivePanel component
pub struct LivePanel {
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
                    {self.render_test_signal_controls(ctx)}
                    {self.render_background_noise_controls(ctx)}
                    {self.render_global_audio_controls(ctx)}
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
    fn setup_event_subscriptions(&mut self, _ctx: &Context<Self>) {
        // All data now comes from shared live_data, no subscriptions needed
        
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