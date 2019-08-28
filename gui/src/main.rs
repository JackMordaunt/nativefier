use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use yew::services::ConsoleService;

struct Model {
    count: u32,
    console: ConsoleService,
}

enum Msg {
    Increment,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Model {
            count: 0,
            console: ConsoleService::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.console.log("update");

        match msg {
            Msg::Increment => {
                self.count += 1;
                self.console.log("increment!");
                true
            }
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            <div>
                <span>{"hello world!"}</span>
                <button onclick=|_| Msg::Increment>{self.count}</button>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
