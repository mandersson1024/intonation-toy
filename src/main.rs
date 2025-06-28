use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

mod audio;
// mod browser_compat;
// mod error_manager;
// mod performance_monitor;
mod modules;
mod types;
mod bootstrap;

use modules::developer_ui::components::debug_interface::DebugInterface;
use modules::audio_foundations::ModularAudioService;
use modules::application_core::ModularErrorService;
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
    
    // Get modular services directly from the bootstrap system
    let audio_engine = use_state(|| {
        if let Some(bootstrap) = bootstrap.as_ref() {
            if bootstrap.is_service_migration_enabled() {
                web_sys::console::log_1(&"✅ Using modular audio service from ApplicationBootstrap".into());
                Rc::new(RefCell::new(ModularAudioService::new()))
            } else {
                web_sys::console::warn_1(&"⚠️ Service migration not enabled - using standalone modular service".into());
                Rc::new(RefCell::new(ModularAudioService::new()))
            }
        } else {
            web_sys::console::warn_1(&"❌ Bootstrap unavailable - using fallback modular audio service".into());
            Rc::new(RefCell::new(ModularAudioService::new()))
        }
    });
    
    // Get modular error service from the bootstrap system
    let error_manager = use_state(|| {
        if let Some(bootstrap) = bootstrap.as_ref() {
            if bootstrap.is_service_migration_enabled() {
                web_sys::console::log_1(&"✅ Using modular error service from ApplicationBootstrap".into());
                Some(Rc::new(RefCell::new(ModularErrorService::new())))
            } else {
                web_sys::console::warn_1(&"⚠️ Service migration not enabled - using standalone modular service".into());
                Some(Rc::new(RefCell::new(ModularErrorService::new())))
            }
        } else {
            web_sys::console::warn_1(&"❌ Bootstrap unavailable - using fallback modular error service".into());
            Some(Rc::new(RefCell::new(ModularErrorService::new())))
        }
    });
    
    // Log modular system health and service migration status on each render
    use_effect(|| {
        if let Some(bootstrap) = bootstrap.as_ref() {
            // Log overall system health
            if bootstrap.is_healthy() {
                web_sys::console::log_1(&"✅ Modular system: All modules healthy".into());
            } else {
                web_sys::console::warn_1(&"⚠️ Modular system: Some modules not healthy".into());
                let states = bootstrap.get_module_states();
                for (id, state) in states {
                    web_sys::console::log_1(&format!("  Module {} -> {:?}", id, state).into());
                }
            }
            
            // Log service migration status
            let migration_status = bootstrap.get_service_migration_status();
            web_sys::console::log_1(&"📊 Service Migration Status:".into());
            for (service, available) in migration_status {
                let status_icon = if available { "✅" } else { "❌" };
                web_sys::console::log_1(&format!("  {} {}", status_icon, service).into());
            }
            
            // Log application state
            let app_state = bootstrap.get_application_state();
            web_sys::console::log_1(&format!("🎯 Application State: {:?}", app_state).into());
        } else {
            web_sys::console::warn_1(&"❌ Modular bootstrap not available".into());
        }
        || {}
    });
    
    // Cleanup on unmount
    use_effect({
        let bootstrap = bootstrap.clone();
        move || {
            || {
                if let Some(mut bootstrap) = bootstrap.take() {
                    let _ = bootstrap.shutdown();
                }
            }
        }
    });
    
    html! {
        <div class="app">
            <DebugInterface 
                audio_engine={Some((*audio_engine).clone())}
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