// Test Signal Controls Component - Interactive test signal generation for volume detection
//
// This component provides interactive controls for generating test signals to validate
// volume detection and pitch analysis. It includes frequency, volume, and noise controls.

use yew::prelude::*;
use web_sys::HtmlInputElement;
use wasm_bindgen::JsCast;

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
    /// Waveform type
    pub waveform: TestWaveform,
}

impl Default for TestSignalConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency: 440.0,
            volume: 0.3,
            waveform: TestWaveform::Sine,
        }
    }
}

/// Background noise configuration for UI
#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundNoiseConfig {
    /// Whether background noise is enabled
    pub enabled: bool,
    /// Noise level (0.0 - 1.0)
    pub level: f32,
    /// Type of noise
    pub noise_type: TestWaveform, // Only WhiteNoise and PinkNoise are valid
}

impl Default for BackgroundNoiseConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            level: 0.0,
            noise_type: TestWaveform::WhiteNoise,
        }
    }
}

/// Messages for test signal controls
#[derive(Debug)]
pub enum TestSignalMsg {
    ToggleEnabled(bool),
    SetFrequency(f32),
    SetVolume(f32),
    SetWaveform(TestWaveform),
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
            TestSignalMsg::SetWaveform(waveform) => {
                self.config.waveform = waveform;
                self.config.enabled = true; // Automatically enable when selecting a waveform
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
                <div class="control-item">
                    <span class="control-label">{"Test signal"}</span>
                    <div class="button-group-horizontal">
                        <button 
                            class={if !self.config.enabled { "button-option active" } else { "button-option" }}
                            onclick={link.callback(|_| TestSignalMsg::ToggleEnabled(false))}
                            title="No Test Signal"
                        >
                            {"🔇"}
                        </button>
                        <button 
                            class={if self.config.enabled && self.config.waveform == TestWaveform::Sine { "button-option active" } else { "button-option" }}
                            onclick={link.callback(|_| {
                                TestSignalMsg::SetWaveform(TestWaveform::Sine)
                            })}
                            title="Test Signal Sine"
                        >
                            {"〰️"}
                        </button>
                        <button 
                            class={if self.config.enabled && self.config.waveform == TestWaveform::Square { "button-option active" } else { "button-option" }}
                            onclick={link.callback(|_| {
                                TestSignalMsg::SetWaveform(TestWaveform::Square)
                            })}
                            title="Test Signal Square"
                        >
                            {"⬜"}
                        </button>
                        <button 
                            class={if self.config.enabled && self.config.waveform == TestWaveform::Triangle { "button-option active" } else { "button-option" }}
                            onclick={link.callback(|_| {
                                TestSignalMsg::SetWaveform(TestWaveform::Triangle)
                            })}
                            title="Test Signal Triangle"
                        >
                            {"🔺"}
                        </button>
                        <button 
                            class={if self.config.enabled && self.config.waveform == TestWaveform::Sawtooth { "button-option active" } else { "button-option" }}
                            onclick={link.callback(|_| {
                                TestSignalMsg::SetWaveform(TestWaveform::Sawtooth)
                            })}
                            title="Test Signal Sawtooth"
                        >
                            {"📈"}
                        </button>
                    </div>
                </div>
                
                <div class="test-signal-controls">

                    // Frequency Slider
                    <div class="control-item control-range">
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
                    <div class="control-item control-range">
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
                        </div>
                    }
                </div>
            </div>
        }
    }
}