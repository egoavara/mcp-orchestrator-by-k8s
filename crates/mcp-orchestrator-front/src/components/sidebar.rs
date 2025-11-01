use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Sidebar)]
pub fn sidebar() -> Html {
    html! {
        <aside class="sidebar">
            <div class="sidebar-section">
                <h3 class="sidebar-section-title">{"Cluster Resources"}</h3>
                <nav class="sidebar-nav">
                    <Link<Route> to={Route::NamespaceList} classes="sidebar-link">
                        <span class="sidebar-link-icon">{"📦"}</span>
                        <span class="sidebar-link-text">{"Namespaces"}</span>
                    </Link<Route>>
                    <Link<Route> to={Route::ResourceLimitList} classes="sidebar-link">
                        <span class="sidebar-link-icon">{"⚙️"}</span>
                        <span class="sidebar-link-text">{"Resource Limits"}</span>
                    </Link<Route>>
                </nav>
            </div>
            <div class="sidebar-section">
                <h3 class="sidebar-section-title">{"Namespaced Resources"}</h3>
                <nav class="sidebar-nav">
                    <Link<Route> to={Route::TemplateList} classes="sidebar-link">
                        <span class="sidebar-link-icon">{"📋"}</span>
                        <span class="sidebar-link-text">{"Templates"}</span>
                    </Link<Route>>
                    <Link<Route> to={Route::ServerList} classes="sidebar-link">
                        <span class="sidebar-link-icon">{"🖥️"}</span>
                        <span class="sidebar-link-text">{"Servers"}</span>
                    </Link<Route>>
                    <Link<Route> to={Route::SecretList} classes="sidebar-link">
                        <span class="sidebar-link-icon">{"🔐"}</span>
                        <span class="sidebar-link-text">{"Secrets"}</span>
                    </Link<Route>>
                    <Link<Route> to={Route::AuthorizationList} classes="sidebar-link">
                        <span class="sidebar-link-icon">{"🔑"}</span>
                        <span class="sidebar-link-text">{"Authorizations"}</span>
                    </Link<Route>>
                </nav>
            </div>
        </aside>
    }
}
