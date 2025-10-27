use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ResourceCardProps {
    pub title: String,
    pub children: Children,
    #[prop_or_default]
    pub onclick: Option<Callback<()>>,
}

#[function_component(ResourceCard)]
pub fn resource_card(props: &ResourceCardProps) -> Html {
    let onclick = props.onclick.clone().map(|cb| {
        Callback::from(move |_| cb.emit(()))
    });

    html! {
        <div class="card" onclick={onclick}>
            <h3>{&props.title}</h3>
            {for props.children.iter()}
        </div>
    }
}
