use kube::Client;

#[derive(Clone)]
pub struct AppState {
    pub kube_client: Client,
    pub default_namespace: String,
}
