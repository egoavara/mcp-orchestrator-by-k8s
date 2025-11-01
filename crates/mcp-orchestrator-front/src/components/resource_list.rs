use yew::prelude::*;

#[allow(dead_code)]
#[derive(Properties, PartialEq)]
pub struct ResourceListProps {
    pub children: Children,
}

#[function_component(ResourceList)]
pub fn resource_list(props: &ResourceListProps) -> Html {
    html! {
        <div class="resource-list">
            {for props.children.iter()}
        </div>
    }
}
