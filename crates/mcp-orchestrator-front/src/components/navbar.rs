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
            <div class="navbar-menu">
                <Link<Route> to={Route::NamespaceList} classes="nav-link">
                    {"Namespaces"}
                </Link<Route>>
                <Link<Route> to={Route::TemplateList} classes="nav-link">
                    {"Templates"}
                </Link<Route>>
                <Link<Route> to={Route::ServerList} classes="nav-link">
                    {"Servers"}
                </Link<Route>>
                <Link<Route> to={Route::SecretList} classes="nav-link">
                    {"Secrets"}
                </Link<Route>>
                <Link<Route> to={Route::ResourceLimitList} classes="nav-link">
                    {"Resource Limits"}
                </Link<Route>>
                {for props.children.iter()}
            </div>
        </nav>
    }
}
