//! # Event-Driven Test Signal Generator Component
//!
//! Test signal generator component with real-time event-driven state synchronization.
//! Subscribes to audio events for automatic state management and signal coordination.

#[cfg(debug_assertions)]
use yew::prelude::*;
#[cfg(debug_assertions)]
use web_sys::{AudioContext, OscillatorNode, OscillatorType, AudioContextState, GainNode, MediaStreamAudioDestinationNode};
#[cfg(debug_assertions)]
use wasm_bindgen::JsCast;
#[cfg(debug_assertions)]
use gloo::console;
#[cfg(debug_assertions)]
use std::rc::Rc;
#[cfg(debug_assertions)]
use std::cell::RefCell;

// Event system imports
#[cfg(debug_assertions)]
use crate::modules::developer_ui::hooks::use_event_subscription::use_event_subscription;
#[cfg(debug_assertions)]
use crate::modules::developer_ui::utils::debug_event_publisher::{DebugEventPublisher, DebugControlEvent};
#[cfg(debug_assertions)]
use crate::modules::application_core::priority_event_bus::PriorityEventBus;
#[cfg(debug_assertions)]
use crate::modules::audio_foundations::audio_events::{
    AudioProcessingStateEvent, AudioSessionEvent, AudioPerformanceMetricsEvent
};

// TODO: Update these imports once legacy services are migrated to modules
#[cfg(debug_assertions)]
use crate::legacy::active::services::AudioEngineService;

#[cfg(debug_assertions)]
#[derive(Clone, Debug, PartialEq)]
pub enum SignalType {
    Sine,
    Square,
    Triangle,
    Sawtooth,
}

#[cfg(debug_assertions)]
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

#[cfg(debug_assertions)]
#[derive(Clone, Debug)]
pub struct SignalConfig {
    pub signal_type: SignalType,
    pub frequency: f32,
    pub amplitude: f32,
}

#[cfg(debug_assertions)]
impl Default for SignalConfig {
    fn default() -> Self {
        Self {
            signal_type: SignalType::Sine,
            frequency: 440.0, // A4 note
            amplitude: 0.3,   // 30% volume
        }
    }
}

#[cfg(debug_assertions)]
pub enum Msg {
    ToggleGeneration,
    ChangeSignalType(SignalType),
    ChangeFrequency(f32),
    ChangeAmplitude(f32),
    TogglePipelineMode,
    
    // Event-driven messages
    AudioStateChanged(crate::modules::audio_foundations::audio_events::AudioEngineState),
    AudioSessionUpdate(AudioSessionEvent),
    PerformanceMetricsUpdate(AudioPerformanceMetricsEvent),
}

#[cfg(debug_assertions)]
#[derive(Properties, PartialEq)]
pub struct TestSignalGeneratorProps {
    #[prop_or_default]
    pub audio_engine: Option<Rc<RefCell<AudioEngineService>>>,
    #[prop_or_default]
    pub on_generation_state_change: Option<Callback<bool>>,
    /// Event bus for subscribing to audio events and publishing test signal events
    #[prop_or_default]
    pub event_bus: Option<Rc<RefCell<PriorityEventBus>>>,
}

#[cfg(debug_assertions)]
pub struct TestSignalGenerator {
    is_generating: bool,
    config: SignalConfig,
    audio_context: Option<AudioContext>,
    oscillator_node: Option<OscillatorNode>,
    gain_node: Option<GainNode>,
    media_destination: Option<MediaStreamAudioDestinationNode>,
    pipeline_mode: bool, // true = route through pipeline, false = direct to speakers
    
    // Event-driven state
    debug_publisher: Option<DebugEventPublisher>,
    last_audio_state: Option<crate::modules::audio_foundations::audio_events::AudioEngineState>,
    last_session_event: Option<AudioSessionEvent>,
    last_performance_metrics: Option<AudioPerformanceMetricsEvent>,
}

#[cfg(debug_assertions)]
impl Component for TestSignalGenerator {
    type Message = Msg;
    type Properties = TestSignalGeneratorProps;

    fn create(ctx: &Context<Self>) -> Self {
        let debug_publisher = ctx.props().event_bus.as_ref()
            .map(|event_bus| DebugEventPublisher::new(Some(event_bus.clone())));
        
        Self {
            is_generating: false,
            config: SignalConfig::default(),
            audio_context: None,
            oscillator_node: None,
            gain_node: None,
            media_destination: None,
            pipeline_mode: true, // Default to pipeline mode
            
            // Initialize event-driven state
            debug_publisher,
            last_audio_state: None,
            last_session_event: None,
            last_performance_metrics: None,
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
            
            // Event-driven message handlers
            Msg::AudioStateChanged(new_state) => {
                console::log(&format!("Test Signal Generator: Audio state changed to {:?}", new_state));
                self.last_audio_state = Some(new_state.clone());
                
                // Auto-stop signal generation if audio engine stops
                match new_state {
                    crate::modules::audio_foundations::audio_events::AudioEngineState::Idle |
                    crate::modules::audio_foundations::audio_events::AudioEngineState::Error(_) => {
                        if self.is_generating {
                            console::log("Auto-stopping test signal due to audio engine state change");
                            self.stop_signal_generation_with_context(ctx);
                        }
                    }
                    _ => {}
                }
                
                // Publish test signal state event
                if let Some(ref mut publisher) = self.debug_publisher {
                    let event = DebugControlEvent::CustomCommand {
                        command: "test_signal_state_sync".to_string(),
                        parameters: {
                            let mut params = std::collections::HashMap::new();
                            params.insert("generating".to_string(), self.is_generating.to_string());
                            params.insert("audio_state".to_string(), format!("{:?}", new_state));
                            params
                        },
                    };
                    if let Err(e) = publisher.publish_control_event(event) {
                        console::error(&format!("Failed to publish test signal sync event: {}", e));
                    }
                }
                
                true
            }
            
            Msg::AudioSessionUpdate(session_event) => {
                console::log(&format!("Test Signal Generator: Session event received - {:?}", session_event.event_type));
                self.last_session_event = Some(session_event.clone());
                
                // Adjust signal generation based on session changes
                match session_event.event_type {
                    crate::modules::audio_foundations::audio_events::AudioSessionEventType::Started => {
                        console::log("Audio session started - test signal can be used");
                    }
                    crate::modules::audio_foundations::audio_events::AudioSessionEventType::Stopped => {
                        if self.is_generating {
                            console::log("Audio session stopped - auto-stopping test signal");
                            self.stop_signal_generation_with_context(ctx);
                        }
                    }
                    _ => {}
                }
                
                true
            }
            
            Msg::PerformanceMetricsUpdate(performance_event) => {
                self.last_performance_metrics = Some(performance_event.clone());
                
                // Adjust signal amplitude based on performance if needed
                if performance_event.cpu_usage_percent > 80.0 && self.is_generating {
                    console::warn("High CPU usage detected - consider reducing test signal complexity");
                }
                
                false // Don't trigger re-render for performance updates
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
                
                // Event-driven synchronization status
                <div class="event-sync-status">
                    <h4>{ "🔄 Event Synchronization Status" }</h4>
                    
                    // Audio state synchronization
                    { if let Some(ref audio_state) = self.last_audio_state {
                        html! {
                            <div class="sync-status-item">
                                <span class="sync-label">{ "Audio Engine State:" }</span>
                                <span class={classes!("sync-value",
                                    match audio_state {
                                        crate::modules::audio_foundations::audio_events::AudioEngineState::Processing => "good",
                                        crate::modules::audio_foundations::audio_events::AudioEngineState::Idle => "neutral",
                                        crate::modules::audio_foundations::audio_events::AudioEngineState::Error(_) => "warning",
                                        _ => "neutral"
                                    })}
                                >
                                    { format!("{:?}", audio_state) }
                                </span>
                            </div>
                        }
                    } else {
                        html! {
                            <div class="sync-status-item">
                                <span class="sync-label">{ "Audio Engine State:" }</span>
                                <span class="sync-value neutral">{ "No events received" }</span>
                            </div>
                        }
                    }}
                    
                    // Session synchronization
                    { if let Some(ref session) = self.last_session_event {
                        html! {
                            <div class="sync-status-item">
                                <span class="sync-label">{ "Audio Session:" }</span>
                                <span class={classes!("sync-value",
                                    match session.event_type {
                                        crate::modules::audio_foundations::audio_events::AudioSessionEventType::Started => "good",
                                        crate::modules::audio_foundations::audio_events::AudioSessionEventType::Stopped => "warning",
                                        _ => "neutral"
                                    })}
                                >
                                    { format!("{:?}", session.event_type) }
                                </span>
                            </div>
                        }
                    } else {
                        html! {
                            <div class="sync-status-item">
                                <span class="sync-label">{ "Audio Session:" }</span>
                                <span class="sync-value neutral">{ "No session events" }</span>
                            </div>
                        }
                    }}
                    
                    // Performance monitoring
                    { if let Some(ref perf) = self.last_performance_metrics {
                        html! {
                            <div class="sync-status-item">
                                <span class="sync-label">{ "System Performance:" }</span>
                                <span class={classes!("sync-value",
                                    if perf.cpu_usage_percent > 80.0 { "warning" } 
                                    else if perf.cpu_usage_percent > 60.0 { "neutral" } 
                                    else { "good" })}
                                >
                                    { format!("CPU: {:.1}%", perf.cpu_usage_percent) }
                                </span>
                            </div>
                        }
                    } else {
                        html! {
                            <div class="sync-status-item">
                                <span class="sync-label">{ "System Performance:" }</span>
                                <span class="sync-value neutral">{ "No performance data" }</span>
                            </div>
                        }
                    }}
                    
                    // Debug event publisher status
                    { if let Some(ref publisher) = self.debug_publisher {
                        if let Some(metrics) = publisher.get_metrics() {
                            html! {
                                <div class="sync-status-item">
                                    <span class="sync-label">{ "Event Publishing:" }</span>
                                    <span class={classes!("sync-value",
                                        if metrics.meets_performance_requirements() { "good" } else { "warning" })}
                                    >
                                        { format!("Published: {}, Success: {:.1}%", metrics.total_published, metrics.success_rate()) }
                                    </span>
                                </div>
                            }
                        } else {
                            html! {
                                <div class="sync-status-item">
                                    <span class="sync-label">{ "Event Publishing:" }</span>
                                    <span class="sync-value good">{ "Ready" }</span>
                                </div>
                            }
                        }
                    } else {
                        html! {
                            <div class="sync-status-item">
                                <span class="sync-label">{ "Event Publishing:" }</span>
                                <span class="sync-value neutral">{ "Not available" }</span>
                            </div>
                        }
                    }}
                </div>
            </div>
        }
    }
}

#[cfg(debug_assertions)]
impl TestSignalGenerator {
    fn render_signal_type_button(&self, link: &yew::html::Scope<Self>, signal_type: SignalType) -> Html {
        let is_active = self.config.signal_type == signal_type;
        let signal_type_clone = signal_type.clone();
        
        html! {
            <button
                class={if is_active { "signal-type-btn active" } else { "signal-type-btn" }}
                onclick={link.callback(move |_| Msg::ChangeSignalType(signal_type_clone.clone()))}
            >
                {match signal_type {
                    SignalType::Sine => "∿ Sine",
                    SignalType::Square => "⧄ Square",
                    SignalType::Triangle => "⧋ Triangle",
                    SignalType::Sawtooth => "⩘ Sawtooth",
                }}
            </button>
        }
    }

    fn start_signal_generation(&mut self, ctx: &Context<Self>) {
        console::log!("Starting test signal generation...");
        
        // Create or resume audio context
        if self.audio_context.is_none() {
            match AudioContext::new() {
                Ok(context) => {
                    self.audio_context = Some(context);
                    console::log!("Created new AudioContext for signal generation");
                }
                Err(e) => {
                    console::error!(&format!("Failed to create AudioContext: {:?}", e));
                    return;
                }
            }
        }
        
        if let Some(ref audio_context) = self.audio_context {
            // Resume context if suspended
            if audio_context.state() == AudioContextState::Suspended {
                let context = audio_context.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Err(e) = context.resume().await {
                        console::error!(&format!("Failed to resume AudioContext: {:?}", e));
                    }
                });
            }
            
            if let Err(e) = self.create_signal_chain(audio_context, ctx) {
                console::error!(&format!("Failed to create signal chain: {:?}", e));
                return;
            }
            
            if let Some(ref oscillator) = self.oscillator_node {
                oscillator.start().unwrap_or_else(|e| {
                    console::error!(&format!("Failed to start oscillator: {:?}", e));
                });
                self.is_generating = true;
                console::log!("Signal generation started successfully");
            }
        }
    }

    fn create_signal_chain(&mut self, audio_context: &AudioContext, ctx: &Context<Self>) -> Result<(), wasm_bindgen::JsValue> {
        // Create oscillator
        let oscillator = audio_context.create_oscillator()?;
        oscillator.set_type(self.config.signal_type.to_oscillator_type());
        oscillator.frequency().set_value(self.config.frequency);
        
        // Create gain node for volume control
        let gain = audio_context.create_gain()?;
        gain.gain().set_value(self.config.amplitude);
        
        // Connect oscillator to gain
        oscillator.connect_with_audio_node(&gain)?;
        
        if self.pipeline_mode {
            // Pipeline mode: create MediaStream for routing through audio engine
            let destination = audio_context.create_media_stream_destination()?;
            gain.connect_with_audio_node(&destination)?;
            
            // Pass MediaStream to audio engine if available
            if let Some(ref audio_engine) = ctx.props().audio_engine {
                let stream = destination.stream();
                console::log!("Connecting test signal to audio engine pipeline...");
                
                let engine = audio_engine.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(mut engine_ref) = engine.try_borrow_mut() {
                        if let Err(e) = engine_ref.connect_stream(stream).await {
                            console::error!(&format!("Failed to connect test signal to engine: {:?}", e));
                        } else {
                            console::log!("Test signal connected to audio engine successfully");
                        }
                    } else {
                        console::warn!("Could not borrow audio engine for test signal connection");
                    }
                });
            }
            
            self.media_destination = Some(destination);
        } else {
            // Direct mode: connect directly to speakers
            gain.connect_with_audio_node(&audio_context.destination())?;
            console::log!("Test signal connected directly to speakers");
        }
        
        // Store oscillator end event handler to clean up when finished
        let oscillator_clone = oscillator.clone();
        let ended_callback = {
            let link = ctx.link().clone();
            Closure::wrap(Box::new(move || {
                link.send_message(Msg::ToggleGeneration);
            }) as Box<dyn FnMut()>)
        };
        oscillator.set_onended(Some(ended_callback.as_ref().unchecked_ref()));
        ended_callback.forget();
        
        self.oscillator_node = Some(oscillator);
        self.gain_node = Some(gain);
        
        console::log!(&format!(
            "Signal chain created: {} wave, {:.1} Hz, {:.0}% volume, {} mode",
            match self.config.signal_type {
                SignalType::Sine => "sine",
                SignalType::Square => "square",
                SignalType::Triangle => "triangle",
                SignalType::Sawtooth => "sawtooth",
            },
            self.config.frequency,
            self.config.amplitude * 100.0,
            if self.pipeline_mode { "pipeline" } else { "direct" }
        ));
        
        Ok(())
    }

    fn stop_signal_generation(&mut self) {
        if let Some(ref oscillator) = self.oscillator_node {
            oscillator.stop().unwrap_or_else(|e| {
                console::warn!(&format!("Error stopping oscillator: {:?}", e));
            });
        }
        
        self.oscillator_node = None;
        self.gain_node = None;
        self.media_destination = None;
        self.is_generating = false;
        
        console::log!("Test signal generation stopped");
    }

    fn stop_signal_generation_with_context(&mut self, ctx: &Context<Self>) {
        self.stop_signal_generation();
        
        // If we were in pipeline mode, disconnect from audio engine
        if self.pipeline_mode {
            if let Some(ref audio_engine) = ctx.props().audio_engine {
                let engine = audio_engine.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(mut engine_ref) = engine.try_borrow_mut() {
                        engine_ref.disconnect_stream();
                        console::log!("Test signal disconnected from audio engine");
                    }
                });
            }
        }
    }

    fn update_audio_engine_signal_info(&self, ctx: &Context<Self>) {
        // Update audio engine with current signal configuration for display purposes
        if let Some(ref audio_engine) = ctx.props().audio_engine {
            // This would be used to show signal info in the audio engine UI
            // Implementation depends on audio engine interface
            console::log!(&format!(
                "Signal config updated: {} Hz, {:.1}% volume",
                self.config.frequency,
                self.config.amplitude * 100.0
            ));
        }
    }
}

#[cfg(debug_assertions)]
impl Drop for TestSignalGenerator {
    fn drop(&mut self) {
        self.stop_signal_generation();
    }
} 