use yew::prelude::*;

#[function_component]
fn App() -> Html {

    let hello_message = "Hello, world!";

    html! {
        <div>
            <h1>{ "pitch-toy" }</h1>
            <p>{ hello_message }</p>
        </div>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    yew::Renderer::<App>::new().render();
}
