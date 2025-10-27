use yew::prelude::*;

#[function_component(Loading)]
pub fn loading() -> Html {
    html! {
        <div class="loading">
            <div class="spinner"></div>
            <p>{"Loading..."}</p>
        </div>
    }
}
