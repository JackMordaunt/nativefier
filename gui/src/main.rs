use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};

struct Model {}

enum Msg {}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Model {}
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {}
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            <div>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
