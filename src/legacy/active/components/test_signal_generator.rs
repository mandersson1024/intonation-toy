use yew::prelude::*;
use web_sys::{AudioContext, OscillatorNode, OscillatorType, AudioContextState, GainNode, MediaStreamAudioDestinationNode};
use wasm_bindgen::JsCast;
use gloo::console;

use std::rc::Rc;
use std::cell::RefCell;
use crate::legacy::active::services::AudioEngineService;

#[derive(Clone, Debug, PartialEq)]
pub enum SignalType {
    Sine,
    Square,
    Triangle,
    Sawtooth,
}

impl SignalType {
    fn to_oscillator_type(&self) -> OscillatorType {
        match self {
            SignalType::Sine => OscillatorType::Sine,
            SignalType::Square => OscillatorType::Square, 
            SignalType::Triangle => OscillatorType::Triangle,
            SignalType::Sawtooth => OscillatorType::Sawtooth,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SignalConfig {
    pub signal_type: SignalType,
    pub frequency: f32,
    pub amplitude: f32,
}

impl Default for SignalConfig {
    fn default() -> Self {
        Self {
            signal_type: SignalType::Sine,
            frequency: 440.0, // A4 note
            amplitude: 0.3,   // 30% volume
        }
    }
}

pub enum Msg {
    ToggleGeneration,
    ChangeSignalType(SignalType),
    ChangeFrequency(f32),
    ChangeAmplitude(f32),
    TogglePipelineMode,
}

#[derive(Properties, PartialEq)]
pub struct TestSignalGeneratorProps {
    #[prop_or_default]
    pub audio_engine: Option<Rc<RefCell<AudioEngineService>>>,
    #[prop_or_default]
    pub on_generation_state_change: Option<Callback<bool>>,
}

pub struct TestSignalGenerator {
    is_generating: bool,
    config: SignalConfig,
    audio_context: Option<AudioContext>,
    oscillator_node: Option<OscillatorNode>,
    gain_node: Option<GainNode>,
    media_destination: Option<MediaStreamAudioDestinationNode>,
    pipeline_mode: bool, // true = route through pipeline, false = direct to speakers
}

impl Component for TestSignalGenerator {
    type Message = Msg;
    type Properties = TestSignalGeneratorProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            is_generating: false,
            config: SignalConfig::default(),
            audio_context: None,
            oscillator_node: None,
            gain_node: None,
            media_destination: None,
            pipeline_mode: true, // Default to pipeline mode
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleGeneration => {
                if self.is_generating {
                    self.stop_signal_generation_with_context(ctx);
                } else {
                    self.start_signal_generation(ctx);
                }
                // Notify parent of state change
                if let Some(ref callback) = ctx.props().on_generation_state_change {
                    callback.emit(self.is_generating);
                }
                true
            }
            Msg::ChangeSignalType(signal_type) => {
                self.config.signal_type = signal_type;
                if self.is_generating {
                    // Restart with new signal type
                    self.stop_signal_generation_with_context(ctx);
                    self.start_signal_generation(ctx);
                    // Notify parent of state change (should still be generating)
                    if let Some(ref callback) = ctx.props().on_generation_state_change {
                        callback.emit(self.is_generating);
                    }
                } else {
                    self.update_audio_engine_signal_info(ctx);
                }
                true
            }
            Msg::ChangeFrequency(frequency) => {
                self.config.frequency = frequency;
                if let Some(ref oscillator) = self.oscillator_node {
                    oscillator.frequency().set_value(frequency);
                }
                self.update_audio_engine_signal_info(ctx);
                true
            }
            Msg::ChangeAmplitude(amplitude) => {
                self.config.amplitude = amplitude;
                if let Some(ref gain) = self.gain_node {
                    gain.gain().set_value(amplitude);
                }
                self.update_audio_engine_signal_info(ctx);
                true
            }
            Msg::TogglePipelineMode => {
                self.pipeline_mode = !self.pipeline_mode;
                if self.is_generating {
                    // Restart with new routing
                    self.stop_signal_generation_with_context(ctx);
                    self.start_signal_generation(ctx);
                    // Notify parent of state change (should still be generating)
                    if let Some(ref callback) = ctx.props().on_generation_state_change {
                        callback.emit(self.is_generating);
                    }
                }
                true
            }

        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        html! {
            <div class="test-signal-generator">
                // Pipeline Mode Toggle
                <div class="pipeline-mode-control">
                    <label class="control-label">{"Output Mode"}</label>
                    <div class="mode-toggle">
                        <button
                            class={if self.pipeline_mode { "mode-btn active" } else { "mode-btn" }}
                            onclick={link.callback(|_| Msg::TogglePipelineMode)}
                        >
                            {if self.pipeline_mode { "🔬 Pipeline Mode" } else { "🔊 Direct Output" }}
                        </button>

                    </div>
                </div>

                <div class="generator-controls">
                    // Signal Type Selector
                    <div class="control-group">
                        <label class="control-label">{"Waveform Type"}</label>
                        <div class="signal-type-buttons">
                            {self.render_signal_type_button(&link, SignalType::Sine)}
                            {self.render_signal_type_button(&link, SignalType::Square)}
                            {self.render_signal_type_button(&link, SignalType::Triangle)}
                            {self.render_signal_type_button(&link, SignalType::Sawtooth)}
                        </div>
                    </div>

                    // Frequency Control
                    <div class="control-group">
                        <label class="control-label">
                            {format!("Frequency: {:.1} Hz", self.config.frequency)}
                        </label>
                        <input
                            type="range"
                            class="frequency-slider"
                            min="20"
                            max="2000"
                            step="1"
                            value={self.config.frequency.to_string()}
                            oninput={link.callback(|e: InputEvent| {
                                let input: web_sys::HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                                let freq = input.value().parse::<f32>().unwrap_or(440.0);
                                Msg::ChangeFrequency(freq)
                            })}
                        />
                    </div>

                    // Amplitude Control
                    <div class="control-group">
                        <label class="control-label">
                            {format!("Volume: {:.0}%", self.config.amplitude * 100.0)}
                        </label>
                        <input
                            type="range"
                            class="amplitude-slider"
                            min="0"
                            max="1"
                            step="0.01"
                            value={self.config.amplitude.to_string()}
                            oninput={link.callback(|e: InputEvent| {
                                let input: web_sys::HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                                let amp = input.value().parse::<f32>().unwrap_or(0.3);
                                Msg::ChangeAmplitude(amp)
                            })}
                        />
                    </div>
                </div>

                // Generation Controls
                <div class="generation-controls">
                    <button
                        class={if self.is_generating { "btn-stop" } else { "btn-start" }}
                        onclick={link.callback(|_| Msg::ToggleGeneration)}
                    >
                        {if self.is_generating { "⏹ Stop Generation" } else { "▶ Start Generation" }}
                    </button>
                </div>


            </div>
        }
    }
}

impl TestSignalGenerator {




    fn render_signal_type_button(&self, link: &yew::html::Scope<Self>, signal_type: SignalType) -> Html {
        let is_active = self.config.signal_type == signal_type;
        let signal_type_clone = signal_type.clone();
        
        html! {
            <button
                class={if is_active { "signal-type-btn active" } else { "signal-type-btn" }}
                onclick={link.callback(move |_| Msg::ChangeSignalType(signal_type_clone.clone()))}
                title={format!("{:?} Wave", signal_type)}
            >
                {format!("{:?}", signal_type)}
            </button>
        }
    }



    fn start_signal_generation(&mut self, ctx: &Context<Self>) {
        // Initialize AudioContext if needed
        if self.audio_context.is_none() {
            match AudioContext::new() {
                Ok(ctx) => {
                    console::log!("AudioContext created successfully");
                    self.audio_context = Some(ctx);
                }
                Err(e) => {
                    console::error!("Failed to create AudioContext:", e);
                    return;
                }
            }
        }

        if let Some(audio_context) = &self.audio_context {
            // Resume AudioContext if suspended
            if audio_context.state() == AudioContextState::Suspended {
                let _ = audio_context.resume();
            }

            match self.create_signal_chain(&audio_context.clone(), ctx) {
                Ok(_) => {
                    self.is_generating = true;
                    console::log!("Signal generation started successfully");
                }
                Err(e) => {
                    console::error!("Failed to start signal generation:", e);
                }
            }
        }
    }

    fn create_signal_chain(&mut self, audio_context: &AudioContext, ctx: &Context<Self>) -> Result<(), wasm_bindgen::JsValue> {
        // Create oscillator node
        let oscillator = audio_context.create_oscillator()?;
        oscillator.set_type(self.config.signal_type.to_oscillator_type());
        oscillator.frequency().set_value(self.config.frequency);

        // Create gain node for volume control
        let gain = audio_context.create_gain()?;
        gain.gain().set_value(self.config.amplitude);

        // Connect oscillator to gain
        oscillator.connect_with_audio_node(&gain)?;

        if self.pipeline_mode {
            // Pipeline mode: Route through MediaStream to AudioEngine
            if let Some(audio_engine) = &ctx.props().audio_engine {
                // Check if AudioEngine is initialized
                let engine_ref = audio_engine.borrow();
                let engine_state = engine_ref.get_state();
                drop(engine_ref); // Release borrow before async operation
                
                match engine_state {
                    crate::legacy::active::services::audio_engine::AudioEngineState::Ready | 
                    crate::legacy::active::services::audio_engine::AudioEngineState::Processing => {
                        // Engine is ready, proceed with connection
                        let destination = audio_context.create_media_stream_destination()?;
                        gain.connect_with_audio_node(&destination)?;
                        
                        let media_stream = destination.stream();
                        let engine_clone = audio_engine.clone();
                        
                        wasm_bindgen_futures::spawn_local(async move {
                            let mut engine = engine_clone.borrow_mut();
                            if let Err(e) = engine.connect_stream(media_stream).await {
                                console::error!("Failed to connect stream to AudioEngine:", format!("{:?}", e));
                            } else {
                                console::log!("Signal successfully routed through AudioEngine pipeline");
                            }
                        });
                        
                        self.media_destination = Some(destination);
                    }
                    crate::legacy::active::services::audio_engine::AudioEngineState::Uninitialized => {
                        // Auto-initialize the AudioEngine
                        console::log!("🔧 AudioEngine not initialized. Auto-initializing for signal generation...");
                        let destination = audio_context.create_media_stream_destination()?;
                        gain.connect_with_audio_node(&destination)?;
                        
                        let media_stream = destination.stream();
                        let engine_clone = audio_engine.clone();
                        
                        wasm_bindgen_futures::spawn_local(async move {
                            // Initialize the engine first
                            let init_result = {
                                let mut engine = engine_clone.borrow_mut();
                                engine.initialize().await
                            };
                            
                            match init_result {
                                Ok(_) => {
                                    console::log!("✅ AudioEngine auto-initialized successfully");
                                    // Now connect the stream
                                    let mut engine = engine_clone.borrow_mut();
                                    if let Err(e) = engine.connect_stream(media_stream).await {
                                        console::error!("Failed to connect stream after auto-initialization:", format!("{:?}", e));
                                    } else {
                                        console::log!("🎵 Signal successfully routed through auto-initialized AudioEngine pipeline");
                                    }
                                }
                                Err(e) => {
                                    console::error!("❌ Failed to auto-initialize AudioEngine:", format!("{:?}", e));
                                }
                            }
                        });
                        
                        self.media_destination = Some(destination);
                    }
                    crate::legacy::active::services::audio_engine::AudioEngineState::Initializing => {
                        // Engine is already initializing, wait for it to complete
                        console::log!("🟡 AudioEngine is initializing. Waiting for completion...");
                        let destination = audio_context.create_media_stream_destination()?;
                        gain.connect_with_audio_node(&destination)?;
                        
                        let media_stream = destination.stream();
                        let engine_clone = audio_engine.clone();
                        
                        // Wait a bit and retry connection
                        wasm_bindgen_futures::spawn_local(async move {
                            // Wait for initialization to complete (simple retry mechanism)
                            gloo::timers::future::TimeoutFuture::new(1000).await;
                            
                            let mut engine = engine_clone.borrow_mut();
                            if let Err(e) = engine.connect_stream(media_stream).await {
                                console::warn!("Failed to connect stream while engine was initializing:", format!("{:?}", e));
                            } else {
                                console::log!("🎵 Signal connected after AudioEngine initialization completed");
                            }
                        });
                        
                        self.media_destination = Some(destination);
                    }
                    crate::legacy::active::services::audio_engine::AudioEngineState::Error(msg) => {
                        console::error!(&format!("❌ AudioEngine error: {}. Falling back to direct output.", msg));
                        gain.connect_with_audio_node(&audio_context.destination())?;
                    }
                    _ => {
                        console::warn!("⚠️ AudioEngine in unknown state. Falling back to direct speaker output.");
                        gain.connect_with_audio_node(&audio_context.destination())?;
                    }
                }
            } else {
                console::warn!("No AudioEngine available, falling back to direct output");
                gain.connect_with_audio_node(&audio_context.destination())?;
            }
        } else {
            // Direct mode: Route directly to speakers
            gain.connect_with_audio_node(&audio_context.destination())?;
            console::log!("Signal routed directly to speakers");
        }

        // Start the oscillator
        oscillator.start()?;

        // Update AudioEngine with test signal information
        if let Some(audio_engine) = &ctx.props().audio_engine {
            let mut engine = audio_engine.borrow_mut();
            engine.set_test_signal_info(
                self.config.frequency,
                self.config.amplitude,
                &format!("{:?}", self.config.signal_type),
                true
            );
        }

        // Store references
        self.oscillator_node = Some(oscillator);
        self.gain_node = Some(gain);

        Ok(())
    }

    fn stop_signal_generation(&mut self) {
        if let Some(oscillator) = self.oscillator_node.take() {
            let _ = oscillator.stop();
        }
        self.gain_node = None;
        self.media_destination = None;
        self.is_generating = false;
        console::log!("Signal generation stopped");
    }

    fn stop_signal_generation_with_context(&mut self, ctx: &Context<Self>) {
        self.stop_signal_generation();
        
        // Clear test signal info in AudioEngine
        if let Some(audio_engine) = &ctx.props().audio_engine {
            let mut engine = audio_engine.borrow_mut();
            engine.set_test_signal_info(
                self.config.frequency,
                self.config.amplitude,
                &format!("{:?}", self.config.signal_type),
                false  // Not active anymore
            );
        }
    }

    fn update_audio_engine_signal_info(&self, ctx: &Context<Self>) {
        // Update AudioEngine with current test signal information
        if let Some(audio_engine) = &ctx.props().audio_engine {
            let mut engine = audio_engine.borrow_mut();
            engine.set_test_signal_info(
                self.config.frequency,
                self.config.amplitude,
                &format!("{:?}", self.config.signal_type),
                self.is_generating
            );
        }
    }
}

impl Drop for TestSignalGenerator {
    fn drop(&mut self) {
        self.stop_signal_generation();
        if let Some(ref audio_context) = self.audio_context {
            let _ = audio_context.close();
        }
    }
} 