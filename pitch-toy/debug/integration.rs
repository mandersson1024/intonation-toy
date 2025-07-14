// Debug Integration Layer - Component communication and coordination
//
// This module provides the integration layer for coordinating between the three
// debug components: DebugConsole, LivePanel, and PermissionButton.

use yew::prelude::*;
use std::rc::Rc;
use wasm_bindgen::JsCast;

use super::LivePanel;
use crate::audio::ConsoleAudioServiceImpl;
use crate::events::AudioEventDispatcher;

/// Properties for the integrated debug interface
#[derive(Properties)]
pub struct DebugInterfaceProps {
    /// Audio service for audio operations
    pub audio_service: Rc<ConsoleAudioServiceImpl>,
    /// Event dispatcher for real-time updates
    pub event_dispatcher: Option<AudioEventDispatcher>,
}

impl PartialEq for DebugInterfaceProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.audio_service, &other.audio_service)
    }
}

/// Integrated debug interface component
pub struct DebugInterface {
    /// Whether the entire debug interface is visible
    visible: bool,
}

/// Messages for the debug interface
#[derive(Debug)]
pub enum DebugInterfaceMsg {
    /// Toggle entire debug interface visibility
    ToggleVisibility,
}

impl Component for DebugInterface {
    type Message = DebugInterfaceMsg;
    type Properties = DebugInterfaceProps;

    fn create(_: &Context<Self>) -> Self {
        let component = Self {
            visible: true,  // Start with debug interface visible on app start
        };

        component
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            DebugInterfaceMsg::ToggleVisibility => {
                self.visible = !self.visible;
                true
            }                
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <style>
                    {DEBUG_INTERFACE_CSS}
                </style>
                <div class="debug-interface">
                    {self.render_debug_components(ctx)}
                </div>
            </>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        if _first_render {
            self.setup_global_keyboard_handler(ctx);
        }
    }
}

impl DebugInterface {
    /// Set up global keyboard handler for Escape key
    fn setup_global_keyboard_handler(&self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            match event.key().as_str() {
                "Escape" => {
                    event.prevent_default();
                    link.send_message(DebugInterfaceMsg::ToggleVisibility);
                }
                _ => {}
            }
        }) as Box<dyn FnMut(_)>);

        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                document
                    .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
                    .unwrap();
            }
        }
        
        // Keep the closure alive by leaking it (this is acceptable for a global handler)
        closure.forget();
    }



    /// Render the debug components
    fn render_debug_components(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="debug-components">
                {self.render_live_panel(ctx)}
            </div>
        }
    }

    /// Render the live panel
    fn render_live_panel(&self, ctx: &Context<Self>) -> Html {
        if let Some(event_dispatcher) = &ctx.props().event_dispatcher {
            html! {
                <LivePanel
                    event_dispatcher={event_dispatcher.clone()}
                    visible={self.visible}
                    audio_service={ctx.props().audio_service.clone()}
                />
            }
        } else {
            html! {}
        }
    }
}

/// Create the integrated debug interface
pub fn create_debug_interface(
    audio_service: Rc<ConsoleAudioServiceImpl>,
    event_dispatcher: Option<AudioEventDispatcher>,
) -> Html {
    html! {
        <DebugInterface
            audio_service={audio_service}
            event_dispatcher={event_dispatcher}
        />
    }
}

pub struct AudioServiceAdapter {
    audio_service: Rc<ConsoleAudioServiceImpl>,
}

impl AudioServiceAdapter {
    pub fn new(audio_service: Rc<ConsoleAudioServiceImpl>) -> Self {
        Self { audio_service }
    }
}

/// CSS styles for the debug interface
const DEBUG_INTERFACE_CSS: &str = r#"
.debug-interface {
    font-family: monospace;
    font-size: 12px;
}

/* Override dev-console fullscreen modal styles to fit in debug interface */
.debug-interface .dev-console-modal {
    position: static;
    top: auto;
    left: auto;
    right: auto;
    bottom: auto;
    background: rgba(17, 24, 39, 0.95);
    border: 1px solid #374151;
    border-radius: 6px;
    width: 500px;
    height: 200px;
    font-size: 12px;
}

.debug-interface .dev-console-header {
    background: #1f2937;
    padding: 8px 12px;
    border-bottom: 1px solid #374151;
}

.debug-interface .dev-console-title {
    font-size: 14px;
    color: #f9fafb;
}

.debug-interface .dev-console-output-container {
    height: 400px;
    background: #111827;
    padding: 8px;
    font-size: 11px;
}

.debug-interface .dev-console-input-container {
    background: #1f2937;
    padding: 8px;
    border-top: 1px solid #374151;
}

.debug-interface .dev-console-input {
    background: #111827;
    border: 1px solid #374151;
    color: #f9fafb;
    padding: 4px 6px;
    border-radius: 3px;
    font-size: 11px;
}

.debug-interface .dev-console-input:focus {
    border-color: #3b82f6;
}

.debug-interface .dev-console-prompt {
    color: #3b82f6;
}

.debug-interface .dev-console-message {
    font-size: 11px;
    white-space: pre-wrap;
    word-wrap: break-word;
}

.debug-interface .dev-console-message-info {
    color: #e5e7eb;
}

.debug-interface .dev-console-message-success {
    color: #22c55e;
}

.debug-interface .dev-console-message-warning {
    color: #fbbf24;
}

.debug-interface .dev-console-message-error {
    color: #f87171;
}

.debug-interface .dev-console-message-command {
    color: #60a5fa;
    font-weight: bold;
}

.debug-components {
    position: fixed;
    top: 10px;
    right: 10px;
    z-index: 1000;
    display: flex;
    flex-direction: column;
    gap: 10px;
    width: 500px;
}


.live-panel {
    background: rgba(17, 24, 39, 0.95);
    border: 1px solid #374151;
    padding: 0;
    color: #f9fafb;
    font-family: monospace;
    font-size: 12px;
    min-width: 300px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
}

.live-panel-header {
    padding: 8px 12px;
    background: #1f2937;
    border-bottom: 1px solid #374151;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.live-panel-title {
    margin: 0;
    font-size: 14px;
    color: #f9fafb;
}

.live-panel-refresh {
    float: right;
    background: none;
    border: none;
    color: #f9fafb;
    cursor: pointer;
    font-size: 14px;
}

.live-panel-content {
    padding: 12px;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
}

/* Custom scrollbar for live panel */
.live-panel-content::-webkit-scrollbar {
    width: 8px;
}

.live-panel-content::-webkit-scrollbar-track {
    background: #1f2937;
    border-radius: 4px;
}

.live-panel-content::-webkit-scrollbar-thumb {
    background: #374151;
    border-radius: 4px;
}

.live-panel-content::-webkit-scrollbar-thumb:hover {
    background: #4b5563;
}

/* Firefox scrollbar styling */
.live-panel-content {
    scrollbar-width: thin;
    scrollbar-color: #374151 #1f2937;
}

.live-panel-content > div {
    margin-bottom: 15px;
}

.live-panel-section-title {
    margin: 0 0 8px 0;
    color: #d1d5db;
    font-weight: bold;
}

.device-section h5 {
    margin: 8px 0 4px 0;
    font-size: 11px;
    color: #9ca3af;
}

.device-item {
    margin-left: 10px;
    font-size: 11px;
    margin-bottom: 2px;
}

.device-name {
    color: #d1d5db;
    font-weight: bold;
}

.metric-item {
    margin-bottom: 4px;
    display: flex;
    gap: 8px;
}

.metric-label {
    color: #9ca3af;
    font-size: 10px;
    width: 80px;
}

.metric-value {
    color: #d1d5db;
    font-weight: bold;
}

.volume-placeholder {
    color: #6b7280;
    font-style: italic;
    font-size: 11px;
}

.volume-metric-item {
    margin-bottom: 6px;
    display: flex;
    align-items: center;
    gap: 8px;
}

.volume-metric-item .metric-label {
    color: #9ca3af;
    font-size: 10px;
    width: 80px;
    flex-shrink: 0;
}

.volume-metric-item .metric-value {
    color: #d1d5db;
    font-weight: bold;
    width: 60px;
    flex-shrink: 0;
    text-align: right;
}

.volume-bar-container {
    flex: 1;
    margin-left: 8px;
}

.volume-bar-track {
    position: relative;
    width: 100%;
    height: 8px;
    background: #374151;
    border-radius: 4px;
    overflow: hidden;
}

.volume-bar-fill {
    height: 100%;
    transition: width 0.1s ease-out;
    border-radius: 4px;
}

.volume-bar-cold {
    background: linear-gradient(90deg, #1e40af, #3b82f6);
}

.volume-bar-cool {
    background: linear-gradient(90deg, #059669, #10b981);
}

.volume-bar-warm {
    background: linear-gradient(90deg, #d97706, #f59e0b);
}

.volume-bar-hot {
    background: linear-gradient(90deg, #dc2626, #ef4444);
    animation: volume-pulse 0.5s ease-in-out infinite alternate;
}

@keyframes volume-pulse {
    0% { opacity: 0.8; }
    100% { opacity: 1.0; }
}

.volume-bar-markers {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
}

.volume-marker {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background: rgba(255, 255, 255, 0.3);
}

.pitch-placeholder {
    color: #6b7280;
    font-style: italic;
    font-size: 11px;
}

.audioworklet-status {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.status-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 11px;
}

.status-label {
    color: #9ca3af;
    font-size: 10px;
    min-width: 100px;
}

.status-value {
    color: #d1d5db;
    font-weight: bold;
    text-align: right;
}

.status-value.worklet-uninitialized {
    color: #6b7280;
}

.status-value.worklet-initializing {
    color: #f59e0b;
}

.status-value.worklet-ready {
    color: #3b82f6;
}

.status-value.worklet-processing {
    color: #10b981;
}

.status-value.worklet-stopped {
    color: #f59e0b;
}

.status-value.worklet-failed {
    color: #ef4444;
}

/* Test Signal Controls */
.test-signal-controls {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.control-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.control-toggle {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    padding: 8px 0;
    border-bottom: 1px solid #374151;
}

.control-toggle label {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
}

.control-checkbox {
    margin: 0;
}

.control-text {
    font-size: 13px;
    font-weight: 500;
    color: #f3f4f6;
}

.status-indicator {
    font-size: 16px;
    font-weight: bold;
}

.status-active {
    color: #22c55e;
}

.status-inactive {
    color: #6b7280;
}

.control-label {
    font-size: 11px;
    font-weight: 500;
    color: #9ca3af;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.control-slider-container {
    display: flex;
    align-items: center;
    gap: 8px;
}

.control-slider {
    flex: 1;
    height: 4px;
    background: #374151;
    border-radius: 2px;
    outline: none;
    cursor: pointer;
}

.control-slider::-webkit-slider-thumb {
    appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #3b82f6;
    cursor: pointer;
    border: 2px solid #1f2937;
}

.control-slider::-moz-range-thumb {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #3b82f6;
    cursor: pointer;
    border: 2px solid #1f2937;
}

.control-slider:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.control-value {
    font-size: 12px;
    color: #f3f4f6;
    min-width: 50px;
    text-align: right;
    font-family: 'Courier New', monospace;
}

.control-select {
    padding: 4px 8px;
    background: #374151;
    border: 1px solid #4b5563;
    border-radius: 4px;
    color: #f3f4f6;
    font-size: 12px;
    outline: none;
}

.control-select:focus {
    border-color: #3b82f6;
}

.volume-bar-test {
    background: linear-gradient(90deg, #3b82f6 0%, #1d4ed8 100%);
}

.control-info {
    background: #1f2937;
    border-radius: 6px;
    padding: 8px;
    margin-top: 8px;
}

.info-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 2px 0;
}

.info-label {
    font-size: 11px;
    color: #9ca3af;
}

.info-value {
    font-size: 11px;
    color: #f3f4f6;
    font-family: 'Courier New', monospace;
}
"#;