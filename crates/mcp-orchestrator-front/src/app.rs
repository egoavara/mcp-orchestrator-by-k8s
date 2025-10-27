use yew::prelude::*;
use yew_router::prelude::*;
use crate::routes::Route;
use crate::components::Layout;
use crate::pages::{
    Home, 
    TemplateList, TemplateDetail, TemplateForm,
    NamespaceList, NamespaceCreate, NamespaceDetail,
    SecretList, SecretCreate, SecretDetail, SecretUpdate,
    ResourceLimitList, ResourceLimitCreate, ResourceLimitDetail,
};

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Layout>
                <Switch<Route> render={switch} />
            </Layout>
        </BrowserRouter>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::NamespaceList => html! { <NamespaceList /> },
        Route::NamespaceCreate => html! { <NamespaceCreate /> },
        Route::NamespaceDetail { name } => html! { <NamespaceDetail name={name} /> },
        Route::TemplateList => html! { <TemplateList /> },
        Route::TemplateCreate => html! { <TemplateForm /> },
        Route::TemplateDetail { namespace, name } => html! { 
            <TemplateDetail namespace={namespace} name={name} /> 
        },
        Route::ServerList => html! { <div class="container"><h2>{"Server List - Coming Soon"}</h2></div> },
        Route::ServerDetail { namespace, name } => html! { <div class="container"><h2>{format!("Server Detail: {}/{} - Coming Soon", namespace, name)}</h2></div> },
        Route::SecretList => html! { <SecretList /> },
        Route::SecretCreate => html! { <SecretCreate /> },
        Route::SecretDetail { namespace, name } => html! { <SecretDetail namespace={namespace} name={name} /> },
        Route::SecretEdit { namespace, name } => html! { <SecretUpdate namespace={namespace} name={name} /> },
        Route::ResourceLimitList => html! { <ResourceLimitList /> },
        Route::ResourceLimitCreate => html! { <ResourceLimitCreate /> },
        Route::ResourceLimitDetail { name } => html! { <ResourceLimitDetail name={name} /> },
        Route::NotFound => html! { <div class="container"><h2>{"404 - Page Not Found"}</h2></div> },
    }
}
