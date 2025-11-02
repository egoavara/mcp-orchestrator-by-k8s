use crate::components::UserMenu;
use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct NavbarProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Navbar)]
pub fn navbar(props: &NavbarProps) -> Html {
    html! {
        <nav class="navbar">
            <div class="navbar-brand">
                <Link<Route> to={Route::Home}>
                    <h1>{"MCP Orchestrator"}</h1>
                </Link<Route>>
            </div>
            <div class="navbar-right">
                {for props.children.iter()}
                <UserMenu />
            </div>
        </nav>
    }
}
