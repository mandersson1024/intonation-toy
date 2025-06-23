use yew::prelude::*;

mod audio;
mod browser_compat;
mod error_manager;
mod performance_monitor;
mod components;
mod services;
mod types;
mod hooks;

use components::DebugInterface;
use services::{AudioEngineService, ErrorManager};
use std::rc::Rc;
use std::cell::RefCell;

#[function_component(App)]
fn app() -> Html {
    let audio_engine = use_state(|| Some(Rc::new(RefCell::new(AudioEngineService::new()))));
    let error_manager = use_state(|| Some(Rc::new(RefCell::new(ErrorManager::new()))));
    
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
    yew::Renderer::<App>::new().render();
} 