use yew::prelude::*;
use crate::legacy::active::services::error_manager::{ApplicationError, ErrorSeverity};

#[derive(Properties, PartialEq)]
pub struct FallbackUIProps {
    pub critical_errors: Vec<ApplicationError>,
    #[prop_or(None)]
    pub fallback_message: Option<String>,
    #[prop_or(true)]
    pub show_browser_recommendations: bool,
}

#[function_component(FallbackUIComponent)]
pub fn fallback_ui_component(props: &FallbackUIProps) -> Html {
    let show_details = use_state(|| false);
    
    let toggle_details = {
        let show_details = show_details.clone();
        Callback::from(move |_: MouseEvent| {
            show_details.set(!*show_details);
        })
    };
    
    let refresh_page = Callback::from(|_| {
        if let Some(window) = web_sys::window() {
            let _ = window.location().reload();
        }
    });
    
    html! {
        <div class="fallback-ui">
            <div class="fallback-container">
                <div class="fallback-header">
                    <div class="fallback-icon">
                        <span class="icon">{"🚫"}</span>
                    </div>
                    <h1>{"Application Cannot Start"}</h1>
                    <p class="fallback-subtitle">
                        {"This application requires a more recent browser to function properly."}
                    </p>
                </div>
                
                <div class="fallback-content">
                    if let Some(ref message) = props.fallback_message {
                        <div class="fallback-message">
                            <h3>{"Issues Detected:"}</h3>
                            <div class="message-content">
                                { message.split('\n').map(|line| html! {
                                    if !line.trim().is_empty() {
                                        <p>{line}</p>
                                    }
                                }).collect::<Html>() }
                            </div>
                        </div>
                    } else {
                        <div class="fallback-message">
                            <h3>{"Critical Issues Detected:"}</h3>
                            <div class="error-list">
                                { for props.critical_errors.iter().map(|error| html! {
                                    <div class="critical-error-item">
                                        <div class="error-header">
                                            <span class="error-icon">{"❌"}</span>
                                            <strong>{&error.message}</strong>
                                        </div>
                                        if let Some(ref details) = error.details {
                                            <p class="error-details">{details}</p>
                                        }
                                    </div>
                                }) }
                            </div>
                        </div>
                    }
                    
                    if props.show_browser_recommendations {
                        <div class="browser-recommendations">
                            <h3>{"Recommended Browsers:"}</h3>
                            <div class="browser-grid">
                                <div class="browser-item">
                                    <div class="browser-icon">{"🌐"}</div>
                                    <div class="browser-info">
                                        <strong>{"Chrome 69+"}</strong>
                                        <a href="https://www.google.com/chrome/" target="_blank" rel="noopener noreferrer">
                                            {"Download Chrome"}
                                        </a>
                                    </div>
                                </div>
                                <div class="browser-item">
                                    <div class="browser-icon">{"🦊"}</div>
                                    <div class="browser-info">
                                        <strong>{"Firefox 76+"}</strong>
                                        <a href="https://www.mozilla.org/firefox/" target="_blank" rel="noopener noreferrer">
                                            {"Download Firefox"}
                                        </a>
                                    </div>
                                </div>
                                <div class="browser-item">
                                    <div class="browser-icon">{"🧭"}</div>
                                    <div class="browser-info">
                                        <strong>{"Safari 14.1+"}</strong>
                                        <span class="platform-note">{"(macOS only)"}</span>
                                    </div>
                                </div>
                                <div class="browser-item">
                                    <div class="browser-icon">{"🔷"}</div>
                                    <div class="browser-info">
                                        <strong>{"Edge 79+"}</strong>
                                        <a href="https://www.microsoft.com/edge/" target="_blank" rel="noopener noreferrer">
                                            {"Download Edge"}
                                        </a>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                    
                    <div class="fallback-actions">
                        <button class="primary-action" onclick={refresh_page}>
                            {"🔄 Refresh Page"}
                        </button>
                        
                        <button class="secondary-action" onclick={toggle_details}>
                            if *show_details {
                                {"Hide Technical Details"}
                            } else {
                                {"Show Technical Details"}
                            }
                        </button>
                    </div>
                    
                    if *show_details {
                        <div class="technical-details">
                            <h4>{"Technical Information:"}</h4>
                            <div class="details-content">
                                { for props.critical_errors.iter().map(|error| html! {
                                    <div class="technical-error">
                                        <div class="error-meta">
                                            <strong>{"Error ID:"}</strong> {&error.id}
                                        </div>
                                        <div class="error-meta">
                                            <strong>{"Category:"}</strong> {format!("{:?}", error.category)}
                                        </div>
                                        <div class="error-meta">
                                            <strong>{"User Agent:"}</strong> {&error.user_agent}
                                        </div>
                                        if !error.recommendations.is_empty() {
                                            <div class="error-recommendations">
                                                <strong>{"Recommendations:"}</strong>
                                                <ul>
                                                    { for error.recommendations.iter().map(|rec| html! {
                                                        <li>{rec}</li>
                                                    }) }
                                                </ul>
                                            </div>
                                        }
                                    </div>
                                }) }
                            </div>
                        </div>
                    }
                </div>
                
                <footer class="fallback-footer">
                    <p>
                        {"Need help? This application requires modern web technologies including "}
                        <strong>{"WebAssembly"}</strong>{", "}
                        <strong>{"Web Audio API"}</strong>{", and "}
                        <strong>{"MediaDevices API"}</strong>
                        {" to function properly."}
                    </p>
                </footer>
            </div>
        </div>
    }
} 