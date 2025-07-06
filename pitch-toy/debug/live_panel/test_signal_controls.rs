// Test Signal Controls Component - Interactive test signal generation for volume detection
//
// This component provides interactive controls for generating test signals to validate
// volume detection and pitch analysis. It includes frequency, volume, and noise controls.

use yew::prelude::*;
use web_sys::HtmlInputElement;
use wasm_bindgen::JsCast;
use js_sys;

/// Test signal waveform types
#[derive(Debug, Clone, PartialEq)]
pub enum TestWaveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    WhiteNoise,
    PinkNoise,
}

impl std::fmt::Display for TestWaveform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestWaveform::Sine => write!(f, "Sine"),
            TestWaveform::Square => write!(f, "Square"),
            TestWaveform::Sawtooth => write!(f, "Sawtooth"),
            TestWaveform::Triangle => write!(f, "Triangle"),
            TestWaveform::WhiteNoise => write!(f, "White Noise"),
            TestWaveform::PinkNoise => write!(f, "Pink Noise"),
        }
    }
}

impl From<&str> for TestWaveform {
    fn from(s: &str) -> Self {
        match s {
            "sine" => TestWaveform::Sine,
            "square" => TestWaveform::Square,
            "sawtooth" => TestWaveform::Sawtooth,
            "triangle" => TestWaveform::Triangle,
            "white-noise" => TestWaveform::WhiteNoise,
            "pink-noise" => TestWaveform::PinkNoise,
            _ => TestWaveform::Sine,
        }
    }
}

impl TestWaveform {
    fn to_value(&self) -> &'static str {
        match self {
            TestWaveform::Sine => "sine",
            TestWaveform::Square => "square",
            TestWaveform::Sawtooth => "sawtooth",
            TestWaveform::Triangle => "triangle",
            TestWaveform::WhiteNoise => "white-noise",
            TestWaveform::PinkNoise => "pink-noise",
        }
    }
}

/// Test signal configuration
#[derive(Debug, Clone, PartialEq)]
pub struct TestSignalConfig {
    /// Whether test signal is enabled
    pub enabled: bool,
    /// Signal frequency in Hz (20 - 2000)
    pub frequency: f32,
    /// Signal volume (0.0 - 1.0)
    pub volume: f32,
    /// Noise floor level (0.0 - 0.5)
    pub noise_floor: f32,
    /// Waveform type
    pub waveform: TestWaveform,
    /// Whether to output signal to audio speakers
    pub output_to_speakers: bool,
}

impl Default for TestSignalConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency: 440.0,
            volume: 0.3,
            noise_floor: 0.0,
            waveform: TestWaveform::Sine,
            output_to_speakers: false,
        }
    }
}

/// Messages for test signal controls
#[derive(Debug)]
pub enum TestSignalMsg {
    ToggleEnabled(bool),
    SetFrequency(f32),
    SetVolume(f32),
    SetNoiseFloor(f32),
    SetWaveform(TestWaveform),
    ToggleOutputToSpeakers(bool),
    UpdateSignal,
}

/// Properties for TestSignalControls component
#[derive(Properties, PartialEq)]
pub struct TestSignalControlsProps {
    /// Callback when configuration changes
    pub on_config_change: Callback<TestSignalConfig>,
    /// Initial configuration
    pub config: TestSignalConfig,
}

/// Test Signal Controls Component
pub struct TestSignalControls {
    config: TestSignalConfig,
}

impl Component for TestSignalControls {
    type Message = TestSignalMsg;
    type Properties = TestSignalControlsProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            config: ctx.props().config.clone(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TestSignalMsg::ToggleEnabled(enabled) => {
                self.config.enabled = enabled;
                ctx.props().on_config_change.emit(self.config.clone());
                true
            }
            TestSignalMsg::SetFrequency(frequency) => {
                self.config.frequency = frequency.clamp(20.0, 2000.0);
                ctx.props().on_config_change.emit(self.config.clone());
                true
            }
            TestSignalMsg::SetVolume(volume) => {
                self.config.volume = volume.clamp(0.0, 1.0);
                ctx.props().on_config_change.emit(self.config.clone());
                true
            }
            TestSignalMsg::SetNoiseFloor(noise_floor) => {
                self.config.noise_floor = noise_floor.clamp(0.0, 0.5);
                ctx.props().on_config_change.emit(self.config.clone());
                true
            }
            TestSignalMsg::SetWaveform(waveform) => {
                self.config.waveform = waveform;
                ctx.props().on_config_change.emit(self.config.clone());
                true
            }
            TestSignalMsg::ToggleOutputToSpeakers(output_to_speakers) => {
                self.config.output_to_speakers = output_to_speakers;
                ctx.props().on_config_change.emit(self.config.clone());
                true
            }
            TestSignalMsg::UpdateSignal => {
                ctx.props().on_config_change.emit(self.config.clone());
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        
        html! {
            <div class="live-panel-section">
                <h4 class="live-panel-section-title">{"Test Signal Generator"}</h4>
                <div class="test-signal-controls">
                    
                    // Enable/Disable Toggle
                    <div class="control-item control-toggle">
                        <label class="control-label">
                            <input 
                                type="checkbox" 
                                checked={self.config.enabled}
                                onchange={link.callback(|e: Event| {
                                    let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                                    TestSignalMsg::ToggleEnabled(input.checked())
                                })}
                                class="control-checkbox"
                            />
                            <span class="control-text">{"Enable Test Signal"}</span>
                        </label>
                        <div class={format!("status-indicator {}", 
                            if self.config.enabled { "status-active" } else { "status-inactive" }
                        )}>
                            {if self.config.enabled { "●" } else { "○" }}
                        </div>
                    </div>

                    // Waveform Selector
                    <div class="control-item">
                        <span class="control-label">{"Waveform"}</span>
                        <select 
                            class="control-select"
                            onchange={link.callback(|e: Event| {
                                let target = e.target().unwrap();
                                let value = js_sys::Reflect::get(&target, &"value".into()).unwrap().as_string().unwrap_or_default();
                                TestSignalMsg::SetWaveform(TestWaveform::from(value.as_str()))
                            })}
                        >
                            <option value="sine" selected={self.config.waveform == TestWaveform::Sine}>{"Sine Wave"}</option>
                            <option value="square" selected={self.config.waveform == TestWaveform::Square}>{"Square Wave"}</option>
                            <option value="sawtooth" selected={self.config.waveform == TestWaveform::Sawtooth}>{"Sawtooth Wave"}</option>
                            <option value="triangle" selected={self.config.waveform == TestWaveform::Triangle}>{"Triangle Wave"}</option>
                            <option value="white-noise" selected={self.config.waveform == TestWaveform::WhiteNoise}>{"White Noise"}</option>
                            <option value="pink-noise" selected={self.config.waveform == TestWaveform::PinkNoise}>{"Pink Noise"}</option>
                        </select>
                    </div>

                    // Frequency Slider
                    <div class="control-item">
                        <span class="control-label">{"Frequency"}</span>
                        <div class="control-slider-container">
                            <input 
                                type="range" 
                                min="20" 
                                max="2000" 
                                step="1"
                                value={self.config.frequency.to_string()}
                                class="control-slider"
                                disabled={!self.config.enabled || matches!(self.config.waveform, TestWaveform::WhiteNoise | TestWaveform::PinkNoise)}
                                oninput={link.callback(|e: InputEvent| {
                                    let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                                    TestSignalMsg::SetFrequency(input.value().parse().unwrap_or(440.0))
                                })}
                            />
                            <span class="control-value">{format!("{:.0} Hz", self.config.frequency)}</span>
                        </div>
                    </div>

                    // Volume Slider
                    <div class="control-item">
                        <span class="control-label">{"Volume"}</span>
                        <div class="control-slider-container">
                            <input 
                                type="range" 
                                min="0" 
                                max="100" 
                                step="1"
                                value={(self.config.volume * 100.0).to_string()}
                                class="control-slider"
                                disabled={!self.config.enabled}
                                oninput={link.callback(|e: InputEvent| {
                                    let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                                    let volume = input.value().parse::<f32>().unwrap_or(30.0) / 100.0;
                                    TestSignalMsg::SetVolume(volume)
                                })}
                            />
                            <span class="control-value">{format!("{:.0}%", self.config.volume * 100.0)}</span>
                        </div>
                        <div class="volume-bar-container">
                            <div class="volume-bar-track">
                                <div 
                                    class="volume-bar-fill volume-bar-test"
                                    style={format!("width: {}%", self.config.volume * 100.0)}
                                />
                            </div>
                        </div>
                    </div>

                    // Noise Floor Slider
                    <div class="control-item">
                        <span class="control-label">{"Background Noise"}</span>
                        <div class="control-slider-container">
                            <input 
                                type="range" 
                                min="0" 
                                max="50" 
                                step="1"
                                value={(self.config.noise_floor * 100.0).to_string()}
                                class="control-slider"
                                disabled={!self.config.enabled}
                                oninput={link.callback(|e: InputEvent| {
                                    let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                                    let noise = input.value().parse::<f32>().unwrap_or(0.0) / 100.0;
                                    TestSignalMsg::SetNoiseFloor(noise)
                                })}
                            />
                            <span class="control-value">{format!("{:.0}%", self.config.noise_floor * 100.0)}</span>
                        </div>
                    </div>

                    // Audio Output Toggle
                    <div class="control-item control-toggle">
                        <label class="control-label">
                            <input 
                                type="checkbox" 
                                checked={self.config.output_to_speakers}
                                onchange={link.callback(|e: Event| {
                                    let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                                    TestSignalMsg::ToggleOutputToSpeakers(input.checked())
                                })}
                                class="control-checkbox"
                                disabled={!self.config.enabled}
                            />
                            <span class="control-text">{"Output to Speakers"}</span>
                        </label>
                        <div class={format!("status-indicator {}", 
                            if self.config.output_to_speakers && self.config.enabled { "status-active" } else { "status-inactive" }
                        )}>
                            {if self.config.output_to_speakers && self.config.enabled { "🔊" } else { "🔇" }}
                        </div>
                    </div>

                    // Signal Info Display
                    if self.config.enabled {
                        <div class="control-info">
                            <div class="info-item">
                                <span class="info-label">{"Signal Type:"}</span>
                                <span class="info-value">{self.config.waveform.to_string()}</span>
                            </div>
                            if !matches!(self.config.waveform, TestWaveform::WhiteNoise | TestWaveform::PinkNoise) {
                                <div class="info-item">
                                    <span class="info-label">{"Frequency:"}</span>
                                    <span class="info-value">{format!("{:.1} Hz", self.config.frequency)}</span>
                                </div>
                            }
                            <div class="info-item">
                                <span class="info-label">{"Amplitude:"}</span>
                                <span class="info-value">{format!("{:.1}%", self.config.volume * 100.0)}</span>
                            </div>
                            if self.config.noise_floor > 0.0 {
                                <div class="info-item">
                                    <span class="info-label">{"Noise Level:"}</span>
                                    <span class="info-value">{format!("{:.1}%", self.config.noise_floor * 100.0)}</span>
                                </div>
                            }
                        </div>
                    }
                </div>
            </div>
        }
    }
}