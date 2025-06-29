// Development console and debugging tools
// Only compiled in debug builds

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct DevConsoleProps {
    pub visible: bool,
}

#[function_component(DevConsole)]
pub fn dev_console(props: &DevConsoleProps) -> Html {
    if !props.visible {
        return html! {};
    }

    html! {
        <div style="
            position: fixed;
            top: 10px;
            right: 10px;
            background: rgba(0, 0, 0, 0.8);
            color: #00ff00;
            padding: 10px;
            border-radius: 5px;
            font-family: monospace;
            font-size: 12px;
            z-index: 1000;
            min-width: 300px;
        ">
            <div style="border-bottom: 1px solid #333; margin-bottom: 5px; padding-bottom: 5px;">
                <strong>{ "🛠️ Dev Console" }</strong>
            </div>
            <div style="color: magenta;">{ "> Type your commands here" }</div>
        </div>
    }
}