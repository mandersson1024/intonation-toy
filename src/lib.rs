use yew::prelude::*;

mod modules;
use modules::application_core::*;

#[function_component(App)]
fn app() -> Html {
    let error_service = ErrorServiceFactory::create_default();
    let hello_message = format!("Hello from Application Core! Error service initialized: {}", 
        error_service.is_some());
    
    html! {
        <div>
            <h1>{ "pitch-toy" }</h1>
            <p>{ hello_message }</p>
        </div>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}
