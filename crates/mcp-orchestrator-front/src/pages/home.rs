use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Home)]
pub fn home() -> Html {
    html! {
        <div class="container">
            <h1>{"MCP Orchestrator Dashboard"}</h1>
            <p>{"Welcome to the MCP Orchestrator management interface."}</p>

            <div class="dashboard-cards">
                <Link<Route> to={Route::NamespaceList}>
                    <div class="dashboard-card">
                        <h3>{"Namespaces"}</h3>
                        <p>{"Manage organizational containers for your resources"}</p>
                    </div>
                </Link<Route>>

                <Link<Route> to={Route::TemplateList}>
                    <div class="dashboard-card">
                        <h3>{"Templates"}</h3>
                        <p>{"Create and manage MCP server templates"}</p>
                    </div>
                </Link<Route>>

                <Link<Route> to={Route::ServerList}>
                    <div class="dashboard-card">
                        <h3>{"Servers"}</h3>
                        <p>{"Monitor running MCP server instances"}</p>
                    </div>
                </Link<Route>>

                <Link<Route> to={Route::SecretList}>
                    <div class="dashboard-card">
                        <h3>{"Secrets"}</h3>
                        <p>{"Manage sensitive configuration data"}</p>
                    </div>
                </Link<Route>>

                <Link<Route> to={Route::ResourceLimitList}>
                    <div class="dashboard-card">
                        <h3>{"Resource Limits"}</h3>
                        <p>{"Configure CPU and memory constraints"}</p>
                    </div>
                </Link<Route>>
            </div>
        </div>
    }
}
