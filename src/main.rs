use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

mod audio;
// mod browser_compat;
// mod error_manager;
// mod performance_monitor;
mod legacy;
mod modules;
mod types;
mod bootstrap;

use legacy::components::DebugInterface;
use legacy::services::{AudioEngineService, ErrorManager};
use bootstrap::ApplicationBootstrap;

#[function_component(App)]
fn app() -> Html {
    // Initialize modular system in parallel with Yew app
    let bootstrap = use_state(|| {
        let mut bootstrap = ApplicationBootstrap::new();
        match bootstrap.register_modules() {
            Ok(_) => {
                if let Err(e) = bootstrap.initialize_and_start() {
                    web_sys::console::error_1(&format!("Module initialization failed: {}", e).into());
                }
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Module registration failed: {}", e).into());
            }
        }
        Some(bootstrap)
    });
    
    // Get audio engine service (now sourced from modular system when available)
    let audio_engine = use_state(|| {
        bootstrap.as_ref()
            .map(|b| b.get_legacy_audio_service())
            .unwrap_or_else(|| {
                web_sys::console::warn_1(&"Using fallback audio service - modular system unavailable".into());
                Rc::new(RefCell::new(AudioEngineService::new()))
            })
    });
    
    // Legacy error manager (unchanged for now)
    let error_manager = use_state(|| Some(Rc::new(RefCell::new(ErrorManager::new()))));
    
    // Log modular system health on each render
    use_effect_with_deps(move |bootstrap| {
        if let Some(bootstrap) = bootstrap.as_ref() {
            if bootstrap.is_healthy() {
                web_sys::console::log_1(&"✅ Modular system: All modules healthy".into());
            } else {
                web_sys::console::warn_1(&"⚠️ Modular system: Some modules not healthy".into());
                let states = bootstrap.get_module_states();
                for (id, state) in states {
                    web_sys::console::log_1(&format!("  {} -> {:?}", id, state).into());
                }
            }
        }
        || {}
    }, bootstrap.clone());
    
    // Cleanup on unmount
    use_effect_with_deps(move |_| {
        || {
            if let Some(mut bootstrap) = bootstrap.take() {
                let _ = bootstrap.shutdown();
            }
        }
    }, ());
    
    html! {
        <div class="app">
            <DebugInterface 
                audio_engine={(*audio_engine).clone()}
                error_manager={(*error_manager).clone()}
                update_interval_ms={250}
            />
        </div>
    }
}

fn main() {
    web_sys::console::log_1(&"🚀 Pitch-Toy starting with modular architecture integration".into());
    yew::Renderer::<App>::new().render();
} 