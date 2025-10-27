use yew_router::Routable;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    
    #[at("/namespaces")]
    NamespaceList,
    #[at("/namespaces/create")]
    NamespaceCreate,
    #[at("/namespaces/:name")]
    NamespaceDetail { name: String },
    
    #[at("/templates")]
    TemplateList,
    #[at("/templates/create")]
    TemplateCreate,
    #[at("/templates/:namespace/:name")]
    TemplateDetail { namespace: String, name: String },
    
    #[at("/servers")]
    ServerList,
    #[at("/servers/:namespace/:name")]
    ServerDetail { namespace: String, name: String },
    
    #[at("/secrets")]
    SecretList,
    #[at("/secrets/create")]
    SecretCreate,
    #[at("/secrets/:namespace/:name")]
    SecretDetail { namespace: String, name: String },
    #[at("/secrets/:namespace/:name/edit")]
    SecretEdit { namespace: String, name: String },
    
    #[at("/resource-limits")]
    ResourceLimitList,
    #[at("/resource-limits/create")]
    ResourceLimitCreate,
    #[at("/resource-limits/:name")]
    ResourceLimitDetail { name: String },
    
    #[not_found]
    #[at("/404")]
    NotFound,
}
