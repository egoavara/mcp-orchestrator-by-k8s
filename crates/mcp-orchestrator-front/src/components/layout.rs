use yew::prelude::*;
use crate::components::Navbar;

#[derive(Properties, PartialEq)]
pub struct LayoutProps {
    pub children: Children,
}

#[function_component(Layout)]
pub fn layout(props: &LayoutProps) -> Html {
    html! {
        <div class="layout">
            <Navbar />
            <main class="main-content">
                {for props.children.iter()}
            </main>
        </div>
    }
}
