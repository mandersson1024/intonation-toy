use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PipelineDebuggerProps {}

pub struct PipelineDebugger;

impl Component for PipelineDebugger {
    type Message = ();
    type Properties = PipelineDebuggerProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="pipeline-debugger">
                <h3>{"Processing Pipeline Debugger"}</h3>
                <p>{"Pipeline debugging functionality for audio processing analysis"}</p>
            </div>
        }
    }
}
